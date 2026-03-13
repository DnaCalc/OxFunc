param(
    [Parameter(Mandatory = $true)]
    [string]$Manifest,

    [Parameter(Mandatory = $true)]
    [string]$Out,

    [string]$XllPath = "",

    [switch]$BuildIfMissing
)

$ErrorActionPreference = "Stop"

function Release-ComObjectSafe {
    param([object]$Obj)
    if ($null -ne $Obj) {
        try { [void][System.Runtime.InteropServices.Marshal]::ReleaseComObject($Obj) } catch {}
    }
}

function Close-WorkbookSafe {
    param([object]$Workbook)
    if ($null -ne $Workbook) {
        try { $Workbook.Close($false) | Out-Null } catch {}
    }
}

function Convert-Value2ToString {
    param([object]$Value)
    if ($null -eq $Value) { return "<null>" }
    if ($Value -is [double] -or $Value -is [single] -or $Value -is [decimal]) {
        return ([string]::Format([System.Globalization.CultureInfo]::InvariantCulture, "{0:R}", $Value))
    }
    if ($Value -is [bool]) {
        return $(if ($Value) { "TRUE" } else { "FALSE" })
    }
    if ($Value -is [Array]) { return "[array]" }
    return [string]$Value
}

function Parse-Assignments {
    param([string]$Raw)

    $items = @()
    if ([string]::IsNullOrWhiteSpace($Raw)) { return $items }

    foreach ($entry in ($Raw -split '\|')) {
        $trimmed = $entry.Trim()
        if ([string]::IsNullOrWhiteSpace($trimmed)) { continue }
        $m = [regex]::Match($trimmed, "^\s*([A-Z]{1,3}[0-9]+)\s*:=\s*(.+)$")
        if (-not $m.Success) {
            throw "Invalid setup assignment '$trimmed' (expected CELL:=VALUE_OR_FORMULA)."
        }
        $items += [PSCustomObject]@{
            Cell = $m.Groups[1].Value
            Expr = $m.Groups[2].Value.Trim()
        }
    }
    return $items
}

function Apply-Assignment {
    param(
        [object]$Worksheet,
        [object]$Assignment
    )

    $expr = [string]$Assignment.Expr
    if ($expr.StartsWith("=")) {
        $Worksheet.Range($Assignment.Cell).Formula2 = $expr
        return
    }
    if ($expr -match '^".*"$') {
        $Worksheet.Range($Assignment.Cell).Value = $expr.Substring(1, $expr.Length - 2).Replace('""', '"')
        return
    }
    if ($expr -eq "TRUE") {
        $Worksheet.Range($Assignment.Cell).Value2 = $true
        return
    }
    if ($expr -eq "FALSE") {
        $Worksheet.Range($Assignment.Cell).Value2 = $false
        return
    }

    $n = 0.0
    if ([double]::TryParse($expr, [System.Globalization.NumberStyles]::Float, [System.Globalization.CultureInfo]::InvariantCulture, [ref]$n)) {
        $Worksheet.Range($Assignment.Cell).Value2 = $n
        return
    }

    $Worksheet.Range($Assignment.Cell).Value = $expr
}

function Describe-RangeSnapshot {
    param(
        [object]$Worksheet,
        [string]$Address
    )

    if ([string]::IsNullOrWhiteSpace($Address)) { return "" }
    $range = $Worksheet.Range($Address)
    $cells = New-Object System.Collections.Generic.List[string]
    foreach ($cell in $range.Cells) {
        $addr = $cell.Address($false, $false)
        $text = [string]$cell.Text
        $value2 = Convert-Value2ToString -Value $cell.Value2
        $formula2 = [string]$cell.Formula2
        $cells.Add("${addr}{text=$text;value2=$value2;formula2=$formula2}")
    }
    return ($cells -join "|")
}

$manifestPath = (Resolve-Path -Path $Manifest -ErrorAction Stop).Path
$outPath = [System.IO.Path]::GetFullPath($Out)
$outDir = Split-Path -Path $outPath -Parent
if ($outDir -and -not (Test-Path $outDir)) {
    New-Item -ItemType Directory -Path $outDir | Out-Null
}

if ([string]::IsNullOrWhiteSpace($XllPath)) {
    $XllPath = Join-Path $PSScriptRoot "bin\OxFunc64.xll"
}
$xllPathFull = [System.IO.Path]::GetFullPath($XllPath)

if (-not (Test-Path $xllPathFull)) {
    if ($BuildIfMissing) {
        $buildScript = Join-Path $PSScriptRoot "build-oxfunc-xll.ps1"
        & $buildScript -Profile release
    }
    if (-not (Test-Path $xllPathFull)) {
        throw "XLL not found: $xllPathFull"
    }
}

$scenarios = Import-Csv -Path $manifestPath | Where-Object { $_.status -in @("ready", "seed") }
if (-not $scenarios -or $scenarios.Count -eq 0) {
    throw "Manifest has no runnable scenario rows: $manifestPath"
}

$excel = $null
$workbook = $null
$worksheet = $null

try {
    $excel = New-Object -ComObject Excel.Application
    $excel.Visible = $false
    $excel.DisplayAlerts = $false
    $excel.ScreenUpdating = $false
    $excel.EnableEvents = $false

    $registered = $excel.RegisterXLL($xllPathFull)
    if (-not $registered) {
        throw "RegisterXLL returned false for: $xllPathFull"
    }

    $workbook = $excel.Workbooks.Add()
    $worksheet = $workbook.Worksheets.Item(1)

    $results = New-Object System.Collections.Generic.List[object]

    foreach ($scenario in $scenarios) {
        $executionStatus = "observed"
        $notes = [string]$scenario.notes
        $formulaCell = "B2"
        $formulaText = ""
        $formulaValue2 = ""
        $observeSnapshot = ""

        try {
            $worksheet.Cells.Clear() | Out-Null

            $assignments = Parse-Assignments -Raw ([string]$scenario.setup_values)
            foreach ($assignment in $assignments) {
                Apply-Assignment -Worksheet $worksheet -Assignment $assignment
            }

            $formula = [string]$scenario.formula
            if (-not $formula.StartsWith("=")) { $formula = "=$formula" }
            $worksheet.Range($formulaCell).Formula2 = $formula

            $excel.CalculateFull()

            $formulaRange = $worksheet.Range($formulaCell)
            $formulaText = [string]$formulaRange.Text
            $formulaValue2 = Convert-Value2ToString -Value $formulaRange.Value2
            $observeSnapshot = Describe-RangeSnapshot -Worksheet $worksheet -Address ([string]$scenario.observe_range)
        }
        catch {
            $executionStatus = "failed"
            $notes = [string]$_.Exception.GetType().FullName + ": " + [string]$_.Exception.Message
        }

        $results.Add([PSCustomObject]@{
            scenario_id = [string]$scenario.scenario_id
            lane = [string]$scenario.lane
            execution_status = $executionStatus
            formula = [string]$scenario.formula
            formula_cell = $formulaCell
            formula_text = $formulaText
            formula_value2 = $formulaValue2
            observe_range = [string]$scenario.observe_range
            observe_snapshot = $observeSnapshot
            xll_path = $xllPathFull
            notes = $notes
        })
    }

    $results | Export-Csv -Path $outPath -NoTypeInformation -Encoding UTF8
    Write-Host "XLL nil probe run complete."
    Write-Host "Output: $outPath"
}
finally {
    Close-WorkbookSafe -Workbook $workbook
    Release-ComObjectSafe -Obj $worksheet
    Release-ComObjectSafe -Obj $workbook
    if ($excel -ne $null) {
        try { $excel.Quit() } catch {}
    }
    Release-ComObjectSafe -Obj $excel
    [GC]::Collect()
    [GC]::WaitForPendingFinalizers()
}

param(
    [Parameter(Mandatory = $true)]
    [string]$Manifest,

    [Parameter(Mandatory = $true)]
    [string]$Out,

    [string]$XllPath = "",

    [switch]$BuildIfMissing
)

$ErrorActionPreference = "Stop"

function Get-ExcelChannel {
    $paths = @(
        "HKLM:\SOFTWARE\Microsoft\Office\ClickToRun\Configuration",
        "HKLM:\SOFTWARE\WOW6432Node\Microsoft\Office\ClickToRun\Configuration"
    )
    foreach ($path in $paths) {
        if (Test-Path $path) {
            $props = Get-ItemProperty -Path $path
            if ($props.UpdateChannel) { return [string]$props.UpdateChannel }
            if ($props.CDNBaseUrl) { return [string]$props.CDNBaseUrl }
        }
    }
    return ""
}

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
    if ($null -eq $Value) { return "" }
    if ($Value -is [double] -or $Value -is [single] -or $Value -is [decimal]) {
        return ([string]::Format([System.Globalization.CultureInfo]::InvariantCulture, "{0:R}", $Value))
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

function Apply-ValueAssignment {
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

function Apply-FormulaAssignment {
    param(
        [object]$Worksheet,
        [object]$Assignment
    )

    $expr = [string]$Assignment.Expr
    $formula = if ($expr.StartsWith("=")) { $expr } else { "=$expr" }
    $Worksheet.Range($Assignment.Cell).Formula2 = $formula
}

function Summarize-Cells {
    param(
        [object]$Worksheet,
        [string]$RawCells
    )

    $parts = New-Object System.Collections.Generic.List[string]
    foreach ($cell in ($RawCells -split '\|' | ForEach-Object { $_.Trim() } | Where-Object { $_ })) {
        $range = $Worksheet.Range($cell)
        $parts.Add(("{0}:formula2={1};text={2};value2={3}" -f
            $cell,
            [string]$range.Formula2,
            [string]$range.Text,
            (Convert-Value2ToString -Value $range.Value2)))
    }
    return [string]::Join(" || ", $parts)
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

    $excelVersion = [string]$excel.Version
    $excelBuild = ""
    try { $excelBuild = [string]$excel.Build } catch { $excelBuild = "" }
    $excelVersionFull = if ($excelBuild) { "$excelVersion (build $excelBuild)" } else { $excelVersion }
    $excelChannel = Get-ExcelChannel

    foreach ($scenario in $scenarios) {
        $executionStatus = "observed"
        $errorMessage = ""
        $observedCells = ""
        $traceCount = ""
        $traceDump = ""

        try {
            $worksheet.Cells.Clear() | Out-Null

            $worksheet.Range("Y1").Formula2 = "=OX_TRACE_RESET()"
            $excel.CalculateFull()
            $worksheet.Range("Y1").Clear() | Out-Null

            $setupAssignments = Parse-Assignments -Raw ([string]$scenario.setup_values)
            foreach ($assignment in $setupAssignments) {
                Apply-ValueAssignment -Worksheet $worksheet -Assignment $assignment
            }

            $formulaAssignments = Parse-Assignments -Raw ([string]$scenario.formula_setup)
            foreach ($assignment in $formulaAssignments) {
                Apply-FormulaAssignment -Worksheet $worksheet -Assignment $assignment
            }

            $excel.CalculateFull()

            $worksheet.Range("Y1").Formula2 = "=OX_TRACE_COUNT()"
            $worksheet.Range("Y2").Formula2 = "=OX_TRACE_DUMP()"
            $excel.CalculateFull()

            $observedCells = Summarize-Cells -Worksheet $worksheet -RawCells ([string]$scenario.observe_cells)
            $traceCount = Convert-Value2ToString -Value $worksheet.Range("Y1").Value2
            $traceDump = [string]$worksheet.Range("Y2").Text
        }
        catch {
            $executionStatus = "failed"
            $errorMessage = [string]$_.Exception.GetType().FullName + ": " + [string]$_.Exception.Message
        }

        $results.Add([PSCustomObject]@{
            scenario_id = [string]$scenario.scenario_id
            lane = [string]$scenario.lane
            execution_status = $executionStatus
            observed_cells = $observedCells
            trace_count = $traceCount
            trace_dump = $traceDump
            excel_version = $excelVersionFull
            excel_channel = $excelChannel
            xll_path = $xllPathFull
            notes = [string]$scenario.notes
            error = $errorMessage
        })
    }

    $results | Export-Csv -Path $outPath -NoTypeInformation -Encoding UTF8
    Write-Host "XLL call-pattern probe complete."
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

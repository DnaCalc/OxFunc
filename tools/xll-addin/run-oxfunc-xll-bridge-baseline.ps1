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

    $row = 1
    foreach ($scenario in $scenarios) {
        $nativeCell = "B$row"
        $bridgeCell = "C$row"
        $executionStatus = "observed"
        $notes = [string]$scenario.notes
        $nativeText = ""
        $bridgeText = ""
        $nativeValue2 = ""
        $bridgeValue2 = ""

        try {
            $worksheet.Cells.Clear() | Out-Null

            $assignments = Parse-Assignments -Raw ([string]$scenario.setup_values)
            foreach ($assignment in $assignments) {
                Apply-Assignment -Worksheet $worksheet -Assignment $assignment
            }

            $nativeFormula = [string]$scenario.native_formula
            $bridgeFormula = [string]$scenario.bridge_formula
            if (-not $nativeFormula.StartsWith("=")) { $nativeFormula = "=$nativeFormula" }
            if (-not $bridgeFormula.StartsWith("=")) { $bridgeFormula = "=$bridgeFormula" }

            $worksheet.Range($nativeCell).Formula2 = $nativeFormula
            $worksheet.Range($bridgeCell).Formula2 = $bridgeFormula

            $excel.CalculateFull()

            $nativeRange = $worksheet.Range($nativeCell)
            $bridgeRange = $worksheet.Range($bridgeCell)
            $nativeText = [string]$nativeRange.Text
            $bridgeText = [string]$bridgeRange.Text
            $nativeValue2 = Convert-Value2ToString -Value $nativeRange.Value2
            $bridgeValue2 = Convert-Value2ToString -Value $bridgeRange.Value2
        }
        catch {
            $executionStatus = "failed"
            $notes = [string]$_.Exception.GetType().FullName + ": " + [string]$_.Exception.Message
        }

        $observedEqual = ($nativeValue2 -ceq $bridgeValue2) -or ($nativeText -ceq $bridgeText)
        $expectedRelation = [string]$scenario.expected_relation
        $knownDivergenceClass = [string]$scenario.known_divergence_class

        $relationStatus = "mismatched"
        if ($executionStatus -eq "failed") {
            $relationStatus = "failed"
        }
        elseif ($expectedRelation -eq "equal") {
            $relationStatus = if ($observedEqual) { "matched" } else { "mismatched" }
        }
        elseif ($expectedRelation -eq "known_divergence_not_equal") {
            $relationStatus = if (-not $observedEqual) { "matched" } else { "mismatched" }
        }
        else {
            $relationStatus = "unsupported_expected_relation"
        }

        $results.Add([PSCustomObject]@{
            scenario_id = [string]$scenario.scenario_id
            lane = [string]$scenario.lane
            execution_status = $executionStatus
            native_formula = [string]$scenario.native_formula
            bridge_formula = [string]$scenario.bridge_formula
            native_text = $nativeText
            bridge_text = $bridgeText
            native_value2 = $nativeValue2
            bridge_value2 = $bridgeValue2
            observed_equal = $observedEqual
            expected_relation = $expectedRelation
            relation_status = $relationStatus
            known_divergence_class = $knownDivergenceClass
            excel_version = $excelVersionFull
            excel_channel = $excelChannel
            xll_path = $xllPathFull
            notes = $notes
        })

        $row++
    }

    $results | Export-Csv -Path $outPath -NoTypeInformation -Encoding UTF8
    Write-Host "XLL bridge baseline run complete."
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

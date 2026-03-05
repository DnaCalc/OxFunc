param(
    [Parameter(Mandatory = $true)]
    [string]$Manifest,

    [Parameter(Mandatory = $true)]
    [string]$Out,

    [string[]]$Lanes = @("FP-A", "FP-B")
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

function Parse-FormulaSpec {
    param([string]$Raw)

    if ([string]::IsNullOrWhiteSpace($Raw)) { return $null }
    if ($Raw -eq "none") { return $null }

    $m = [regex]::Match($Raw, "^\s*([A-Z]{1,3}[0-9]+)\s+formula:\s*(=.*)$")
    if (-not $m.Success) { return $null }

    return [PSCustomObject]@{
        Cell = $m.Groups[1].Value
        Formula = $m.Groups[2].Value
    }
}

$manifestPath = Resolve-Path -Path $Manifest -ErrorAction Stop
$outPath = [System.IO.Path]::GetFullPath($Out)
$outDir = Split-Path -Path $outPath -Parent
if ($outDir -and -not (Test-Path $outDir)) {
    New-Item -ItemType Directory -Path $outDir | Out-Null
}

$rows = Import-Csv -Path $manifestPath
if (-not $rows -or $rows.Count -eq 0) {
    throw "Manifest has no data rows: $manifestPath"
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

    $workbook = $excel.Workbooks.Add()
    $worksheet = $workbook.Worksheets.Item(1)

    $excelVersion = [string]$excel.Version
    $excelBuild = ""
    try { $excelBuild = [string]$excel.Build } catch { $excelBuild = "" }
    $excelVersionFull = if ($excelBuild) { "$excelVersion (build $excelBuild)" } else { $excelVersion }
    $excelChannel = Get-ExcelChannel

    $calcVersion = ""
    $checkCompatibility = ""
    try { $calcVersion = [string]$workbook.CalculationVersion } catch { $calcVersion = "" }
    try { $checkCompatibility = [string]$workbook.CheckCompatibility } catch { $checkCompatibility = "" }
    $compatVersion = "default|CalculationVersion=$calcVersion|CheckCompatibility=$checkCompatibility"

    $result = New-Object System.Collections.Generic.List[object]

    foreach ($row in $rows) {
        if ($row.status -ne "seed") { continue }
        if ($Lanes -notcontains $row.lane) { continue }

        try {
            $formulaSpecs = @()
            foreach ($fieldName in @("sheet_setup", "cell_formula_or_action", "comparison_cells")) {
                $spec = Parse-FormulaSpec -Raw ([string]$row.$fieldName)
                if ($null -ne $spec) { $formulaSpecs += $spec }
            }

            if ($formulaSpecs.Count -eq 0) {
                $result.Add([PSCustomObject]@{
                    scenario_id = $row.scenario_id
                    lane = $row.lane
                    mode = "excel-baseline"
                    execution_status = "skipped_no_formula"
                    observed_class = "pending_observation"
                    excel_version = $excelVersionFull
                    excel_channel = $excelChannel
                    compat_version = $compatVersion
                    locale_profile = "en-US"
                    runner_version = "fp-excel-baseline-ps1/0.1.0"
                    artifact_ref = ""
                    notes = "No formula specs parsed from manifest fields."
                })
                continue
            }

            $worksheet.Cells.Clear() | Out-Null
            foreach ($spec in $formulaSpecs) {
                try {
                    $worksheet.Range($spec.Cell).Formula = $spec.Formula
                }
                catch {
                    throw "formula set failed at $($spec.Cell) with '$($spec.Formula)': $($_.Exception.Message)"
                }
            }

            $excel.CalculateFull()

            $primaryCell = $formulaSpecs[0].Cell
            $primaryDisplay = [string]$worksheet.Range($primaryCell).Text
            $primaryFormula = [string]$worksheet.Range($primaryCell).Formula
            $observedClass = if ($primaryDisplay.StartsWith("#")) { "error:$primaryDisplay" } else { "value" }

            $comparisonNote = ""
            if ($formulaSpecs.Count -gt 1) {
                $compParts = @()
                foreach ($spec in $formulaSpecs | Select-Object -Skip 1) {
                    $compParts += "$($spec.Cell):$([string]$worksheet.Range($spec.Cell).Text)"
                }
                $comparisonNote = "; comparisons=" + ($compParts -join "|")
            }

            $result.Add([PSCustomObject]@{
                scenario_id = $row.scenario_id
                lane = $row.lane
                mode = "excel-baseline"
                execution_status = "observed"
                observed_class = $observedClass
                excel_version = $excelVersionFull
                excel_channel = $excelChannel
                compat_version = $compatVersion
                locale_profile = "en-US"
                runner_version = "fp-excel-baseline-ps1/0.1.0"
                artifact_ref = ""
                notes = "primary=$primaryCell display=$primaryDisplay formula=$primaryFormula$comparisonNote"
            })
        }
        catch {
            $result.Add([PSCustomObject]@{
                scenario_id = $row.scenario_id
                lane = $row.lane
                mode = "excel-baseline"
                execution_status = "failed"
                observed_class = "pending_observation"
                excel_version = $excelVersionFull
                excel_channel = $excelChannel
                compat_version = $compatVersion
                locale_profile = "en-US"
                runner_version = "fp-excel-baseline-ps1/0.1.0"
                artifact_ref = ""
                notes = "scenario failure: $($_.Exception.Message)"
            })
        }
    }

    $result | Export-Csv -Path $outPath -NoTypeInformation -Encoding UTF8
    Write-Host "Excel baseline run complete. Rows written: $($result.Count)"
    Write-Host "Output: $outPath"
}
finally {
    if ($workbook -ne $null) {
        try { $workbook.Close($false) | Out-Null } catch {}
    }
    if ($excel -ne $null) {
        try { $excel.Quit() } catch {}
    }
    foreach ($obj in @($worksheet, $workbook, $excel)) {
        if ($obj -ne $null) {
            try { [void][System.Runtime.InteropServices.Marshal]::ReleaseComObject($obj) } catch {}
        }
    }
    [GC]::Collect()
    [GC]::WaitForPendingFinalizers()
}

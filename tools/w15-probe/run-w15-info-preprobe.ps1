param(
    [string]$Manifest = "docs/function-lane/W15_INFO_PRE_SCENARIO_MANIFEST_SEED.csv",
    [string]$Out = ".tmp/w15-info-pre-results.csv",
    [string]$WorkbookTemplate = "",
    [string]$RunLabel = "default"
)

$ErrorActionPreference = "Stop"

function Get-WorkbookDescriptor {
    param([object]$Workbook)

    try { $calcVersion = [string]$Workbook.CalculationVersion } catch { $calcVersion = "" }
    try { $checkCompatibility = [string]$Workbook.CheckCompatibility } catch { $checkCompatibility = "" }
    try { $fileFormat = [string]$Workbook.FileFormat } catch { $fileFormat = "" }
    return "default|CalculationVersion=$calcVersion|CheckCompatibility=$checkCompatibility|FileFormat=$fileFormat"
}

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..\\..")
$manifestPath = Join-Path $repoRoot $Manifest
$outPath = Join-Path $repoRoot $Out
$outDir = Split-Path -Parent $outPath
if (-not (Test-Path $outDir)) {
    New-Item -ItemType Directory -Path $outDir | Out-Null
}

$rows = Import-Csv $manifestPath

$excel = New-Object -ComObject Excel.Application
$excel.Visible = $false
$excel.DisplayAlerts = $false

try {
    if (-not [string]::IsNullOrWhiteSpace($WorkbookTemplate)) {
        $templatePath = if ([System.IO.Path]::IsPathRooted($WorkbookTemplate)) {
            $WorkbookTemplate
        } else {
            Join-Path $repoRoot $WorkbookTemplate
        }
        $templatePath = [System.IO.Path]::GetFullPath($templatePath)
        $workDir = Join-Path $env:TEMP "oxfunc-w15-info"
        if (-not (Test-Path $workDir)) {
            New-Item -ItemType Directory -Path $workDir | Out-Null
        }
        $workPath = Join-Path $workDir ($RunLabel + "-probe" + [System.IO.Path]::GetExtension($templatePath))
        Copy-Item $templatePath $workPath -Force
        $wb = $excel.Workbooks.Open($workPath)
    } else {
        $wb = $excel.Workbooks.Add()
    }
    $ws = $wb.Worksheets.Item(1)
    $compatDescriptor = Get-WorkbookDescriptor -Workbook $wb
    $results = @()

    for ($i = 0; $i -lt $rows.Count; $i++) {
        $row = $rows[$i]
        $excelRow = $i + 1
        $formula = '=INFO("' + $row.type_text + '")'
        $cell = $ws.Cells.Item($excelRow, 1)
        $cell.Formula = $formula
    }

    $ws.Calculate()

    for ($i = 0; $i -lt $rows.Count; $i++) {
        $row = $rows[$i]
        $excelRow = $i + 1
        $cell = $ws.Cells.Item($excelRow, 1)
        $results += [pscustomobject]@{
            scenario_id = $row.scenario_id
            lane = $row.lane
            run_label = $RunLabel
            compat_descriptor = $compatDescriptor
            type_text = $row.type_text
            formula = $cell.Formula
            text = $cell.Text
            value2 = $cell.Value2
            expected_status = $row.expected_status
            expected_observable = $row.expected_observable
        }
    }

    $results | ConvertTo-Csv -NoTypeInformation | Set-Content -Path $outPath -Encoding UTF8
    Get-Content $outPath
}
finally {
    if ($wb -ne $null) {
        $wb.Close($false)
        [System.Runtime.InteropServices.Marshal]::ReleaseComObject($ws) | Out-Null
        [System.Runtime.InteropServices.Marshal]::ReleaseComObject($wb) | Out-Null
    }
    $excel.Quit()
    [System.Runtime.InteropServices.Marshal]::ReleaseComObject($excel) | Out-Null
}

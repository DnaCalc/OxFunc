param(
    [string]$Manifest = "docs/function-lane/W39_SCENARIO_MANIFEST_SEED.csv",
    [string]$Out = ".tmp/w39-dynamic-array-reshape-results.csv"
)

$root = (Resolve-Path (Join-Path $PSScriptRoot "..\\..")).Path
$manifestPath = Join-Path $root $Manifest
$outPath = Join-Path $root $Out
$rows = Import-Csv $manifestPath

$excel = New-Object -ComObject Excel.Application
$excel.Visible = $false
$excel.DisplayAlerts = $false
$wb = $excel.Workbooks.Add()
$ws = $wb.Worksheets.Item(1)
$ws.Cells.NumberFormat = "General"

$results = @()
$rowIndex = 1
foreach ($row in $rows) {
    $cell = $ws.Cells.Item($rowIndex, 6)
    $cell.Formula2 = "=" + $row.formula
    $text = [string]$cell.Text

    $results += [pscustomobject]@{
        scenario_id = $row.scenario_id
        lane = $row.lane
        formula = "=" + $row.formula
        text = $text
        expected_text = $row.expected_text
        matches_expected = ($text -eq $row.expected_text)
        notes = $row.notes
    }
    $rowIndex++
}

$outDir = Split-Path $outPath -Parent
if (-not (Test-Path $outDir)) {
    New-Item -ItemType Directory -Force -Path $outDir | Out-Null
}
$results | Export-Csv -NoTypeInformation -Path $outPath

$wb.Close($false)
$excel.Quit()
[System.Runtime.InteropServices.Marshal]::ReleaseComObject($ws) | Out-Null
[System.Runtime.InteropServices.Marshal]::ReleaseComObject($wb) | Out-Null
[System.Runtime.InteropServices.Marshal]::ReleaseComObject($excel) | Out-Null

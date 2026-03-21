param(
    [string]$Manifest = "docs/function-lane/W40_SCENARIO_MANIFEST_SEED.csv",
    [string]$Out = ".tmp/w40-reference-metadata-results.csv"
)

$root = (Resolve-Path (Join-Path $PSScriptRoot "..\\..")).Path
$manifestPath = Join-Path $root $Manifest
$outPath = Join-Path $root $Out
$rows = Import-Csv $manifestPath

$excel = New-Object -ComObject Excel.Application
$excel.Visible = $false
$excel.DisplayAlerts = $false
$wb = $excel.Workbooks.Add()
$alpha = $wb.Worksheets.Item(1)
$alpha.Name = "Alpha"
$beta = $wb.Worksheets.Add()
$beta.Name = "Beta"
$quarter = $wb.Worksheets.Add()
$quarter.Name = "Quarter 1"

$alpha.Range("A1").Formula = "=1+2"
$alpha.Range("B1").Value2 = "plain"

$results = @()
$rowIndex = 1
foreach ($row in $rows) {
    $cell = $alpha.Cells.Item($rowIndex + 10, 6)
    $cell.Formula2 = "=" + $row.formula
    $excel.CalculateFull()
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
[System.Runtime.InteropServices.Marshal]::ReleaseComObject($alpha) | Out-Null
[System.Runtime.InteropServices.Marshal]::ReleaseComObject($beta) | Out-Null
[System.Runtime.InteropServices.Marshal]::ReleaseComObject($quarter) | Out-Null
[System.Runtime.InteropServices.Marshal]::ReleaseComObject($wb) | Out-Null
[System.Runtime.InteropServices.Marshal]::ReleaseComObject($excel) | Out-Null

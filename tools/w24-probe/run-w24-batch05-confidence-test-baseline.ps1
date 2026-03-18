param(
    [string]$Manifest = "docs/function-lane/W24_BATCH05_CONFIDENCE_TEST_SCENARIO_MANIFEST_SEED.csv",
    [string]$Out = ".tmp/w24-batch05-confidence-test-results.csv"
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
    $cell = $ws.Cells.Item($rowIndex, 1)
    $cell.Formula2 = "=" + ($row.formula -replace "`r?`n", "")
    $text = [string]$cell.Text
    $value2 = $cell.Value2
    $matches = $false
    if ($row.expected_text) {
        $matches = ($text -eq $row.expected_text)
    } elseif ($row.expected_value2 -ne "") {
        $expected = [double]$row.expected_value2
        $tol = if ($row.tolerance -eq "") { 0.0 } else { [double]$row.tolerance }
        $actual = [double]$value2
        $matches = [math]::Abs($actual - $expected) -le $tol
    }
    $results += [pscustomobject]@{
        scenario_id = $row.scenario_id
        lane = $row.lane
        formula = "=" + ($row.formula -replace "`r?`n", "")
        text = $text
        value2 = $value2
        expected_text = $row.expected_text
        expected_value2 = $row.expected_value2
        tolerance = $row.tolerance
        matches_expected = $matches
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

param(
    [string]$Manifest = "docs/function-lane/W33_SCENARIO_MANIFEST_SEED.csv",
    [string]$Out = ".tmp/w33-info-forecast-results.csv"
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

# Fixture cells for the reference-fed lanes.
$ws.Range("A1").Value2 = "x"
$ws.Range("A2").Value2 = 42
$ws.Range("A3").Formula2 = "="""""
$ws.Range("B2").Value2 = 7
$ws.Range("A10").Value2 = 2
$ws.Range("A11").Value2 = 4
$ws.Range("B10").Value2 = 1
$ws.Range("B11").Value2 = 2

$results = @()
$rowIndex = 1
foreach ($row in $rows) {
    $cell = $ws.Cells.Item($rowIndex, 6)
    $cell.Formula2 = "=" + $row.formula
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
        formula = "=" + $row.formula
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

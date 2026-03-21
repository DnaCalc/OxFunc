param(
    [string]$Manifest = "docs/function-lane/W23_ISFORMULA_SCENARIO_MANIFEST_SEED.csv",
    [string]$Out = ".tmp/w23-isformula-results.csv"
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

$ws.Range("A1").Formula = "=1+2"
$ws.Range("A2").Value2 = "plain"
$ws.Range("A3").Formula = "="""""

$results = @()
$rowIndex = 1
foreach ($row in $rows) {
    $cell = $ws.Cells.Item($rowIndex, 5)
    $formula = "=" + $row.formula
    try {
        $cell.Formula = $formula
        $excel.CalculateFull()
        $value2 = $cell.Value2
        $text = [string]$cell.Text
    } catch {
        $value2 = $excel.Evaluate($row.formula)
        $text = [string]$value2
    }
    if ($value2 -is [int] -and [int]$value2 -eq -2146826273) {
        $text = "#VALUE!"
    }
    $results += [pscustomobject]@{
        scenario_id = $row.scenario_id
        lane = $row.lane
        formula = $formula
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

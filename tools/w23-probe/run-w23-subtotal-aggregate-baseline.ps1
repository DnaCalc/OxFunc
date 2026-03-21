param(
    [string]$Manifest = "docs/function-lane/W23_SUBTOTAL_AGGREGATE_SCENARIO_MANIFEST_SEED.csv",
    [string]$Out = ".tmp/w23-subtotal-aggregate-results.csv"
)

$root = (Resolve-Path (Join-Path $PSScriptRoot "..\\..")).Path
$manifestPath = Join-Path $root $Manifest
$outPath = Join-Path $root $Out
$rows = Import-Csv $manifestPath

$excel = New-Object -ComObject Excel.Application
$excel.Visible = $false
$excel.DisplayAlerts = $false
$wb = $excel.Workbooks.Add()
$manualWs = $wb.Worksheets.Item(1)
$manualWs.Name = "manual_block"
$filterWs = $wb.Worksheets.Add()
$filterWs.Name = "filter_block"
$errorWs = $wb.Worksheets.Add()
$errorWs.Name = "error_block"
$largeWs = $wb.Worksheets.Add()
$largeWs.Name = "large_block"

foreach ($sheet in @($manualWs, $filterWs, $errorWs, $largeWs)) {
    $sheet.Cells.NumberFormat = "General"
}

$manualWs.Range("A2").Value2 = 10
$manualWs.Range("A3").Value2 = 20
$manualWs.Range("A4").Formula2 = "=SUBTOTAL(9,A2:A3)"
$manualWs.Range("A5").Value2 = 40
$manualWs.Rows.Item(3).Hidden = $true

$filterWs.Range("D9").Value2 = "vals"
$filterWs.Range("D10").Value2 = 10
$filterWs.Range("D11").Value2 = 20
$filterWs.Range("D12").Formula2 = "=SUBTOTAL(9,D10:D11)"
$filterWs.Range("D13").Value2 = 40
$filterWs.Range("D9:D13").AutoFilter(1, "<>20") | Out-Null

$errorWs.Range("H20").Value2 = 10
$errorWs.Range("H21").Value2 = 20
$errorWs.Range("H22").Formula2 = "=SUBTOTAL(9,H20:H21)"
$errorWs.Range("H23").Value2 = 40
$errorWs.Range("H24").Formula2 = "=NA()"
$errorWs.Rows.Item(21).Hidden = $true

$largeWs.Range("K30").Value2 = 1
$largeWs.Range("K31").Value2 = 9
$largeWs.Range("K32").Value2 = 3
$largeWs.Range("K33").Value2 = 7
$largeWs.Range("K34").Value2 = 5
$largeWs.Rows.Item(31).Hidden = $true

$results = @()
foreach ($row in $rows) {
    $excel.CalculateFull()
    $sheet = switch -Regex ($row.lane) {
        "manual" { $manualWs; break }
        "filter" { $filterWs; break }
        "error" { $errorWs; break }
        "large" { $largeWs; break }
        default { $manualWs }
    }
    $value2 = $sheet.Evaluate($row.formula)
    $text = [string]$value2
    if ($value2 -is [int]) {
        switch ([int]$value2) {
            -2146826246 { $text = "#N/A" }
            -2146826273 { $text = "#VALUE!" }
            -2146826281 { $text = "#DIV/0!" }
            -2146826259 { $text = "#NAME?" }
            -2146826265 { $text = "#NUM!" }
            -2146826288 { $text = "#NULL!" }
        }
    }
    $results += [pscustomobject]@{
        scenario_id = $row.scenario_id
        lane = $row.lane
        formula = "=" + $row.formula
        text = $text
        expected_text = $row.expected_text
        matches_expected = ($text -eq $row.expected_text)
        notes = $row.notes
    }
}

$outDir = Split-Path $outPath -Parent
if (-not (Test-Path $outDir)) {
    New-Item -ItemType Directory -Force -Path $outDir | Out-Null
}
$results | Export-Csv -NoTypeInformation -Path $outPath

$wb.Close($false)
$excel.Quit()
[System.Runtime.InteropServices.Marshal]::ReleaseComObject($largeWs) | Out-Null
[System.Runtime.InteropServices.Marshal]::ReleaseComObject($errorWs) | Out-Null
[System.Runtime.InteropServices.Marshal]::ReleaseComObject($filterWs) | Out-Null
[System.Runtime.InteropServices.Marshal]::ReleaseComObject($manualWs) | Out-Null
[System.Runtime.InteropServices.Marshal]::ReleaseComObject($wb) | Out-Null
[System.Runtime.InteropServices.Marshal]::ReleaseComObject($excel) | Out-Null

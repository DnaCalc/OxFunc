param(
    [string]$Manifest = "docs/function-lane/W23_HOST_PROVIDER_CLASSIFICATION_SCENARIO_MANIFEST_SEED.csv",
    [string]$Out = ".tmp/w23-host-provider-classification-results.csv"
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
    $cell = $ws.Cells.Item($rowIndex, 8)
    $cell.Formula2 = "=" + $row.formula
    Start-Sleep -Milliseconds 500
    $excel.Calculate()
    $cell.EntireColumn.AutoFit() | Out-Null

    $value2 = $cell.Value2
    $text = [string]$cell.Text
    if ($text -eq "########" -and $value2 -is [int]) {
        switch ([int]$value2) {
            -2146826242 { $text = "#BUSY!" }
            -2146826259 { $text = "#NAME?" }
        }
    }
    $results += [pscustomobject]@{
        scenario_id = $row.scenario_id
        lane = $row.lane
        formula = "=" + $row.formula
        text = $text
        expected_text = $row.expected_text
        matches_expected = ($text -eq $row.expected_text)
        value2 = [string]$value2
        value2_type = if ($null -eq $value2) { "" } else { $value2.GetType().FullName }
        font_underline = [string]$cell.Font.Underline
        font_color = [string]$cell.Font.Color
        hyperlinks_count = [string]$cell.Hyperlinks.Count
        number_format = [string]$cell.NumberFormat
        notes = $row.notes
    }
    $rowIndex += 2
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

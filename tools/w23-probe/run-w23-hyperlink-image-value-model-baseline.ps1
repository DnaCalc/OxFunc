param(
    [string]$Out = ".tmp/w23-hyperlink-image-value-model-results.csv"
)

$root = (Resolve-Path (Join-Path $PSScriptRoot "..\\..")).Path
$outPath = Join-Path $root $Out

$excel = New-Object -ComObject Excel.Application
$excel.Visible = $false
$excel.DisplayAlerts = $false
$wb = $excel.Workbooks.Add()
$ws = $wb.Worksheets.Item(1)
$ws.Cells.NumberFormat = "General"

function Get-CellSnapshot {
    param(
        $Worksheet,
        [string]$Address
    )

    $cell = $Worksheet.Range($Address)
    $cell.EntireColumn.AutoFit() | Out-Null
    [pscustomobject]@{
        address = $Address
        text = [string]$cell.Text
        value2 = [string]$cell.Value2
        type_code = [string]$Worksheet.Evaluate("TYPE($Address)")
        cell_contents = [string]$Worksheet.Evaluate("CELL(""contents"",$Address)")
        t_value = [string]$Worksheet.Evaluate("T($Address)")
        n_value = [string]$Worksheet.Evaluate("N($Address)")
        font_underline = [string]$cell.Font.Underline
        hyperlinks_count = [string]$cell.Hyperlinks.Count
    }
}

$syncAsync = $excel.GetType().GetMethod("CalculateUntilAsyncQueriesDone")

$results = @()

$ws.Range("A1").Formula = '=HYPERLINK("https://example.com","Go")'
$ws.Range("B1").Formula = "=A1"
$excel.CalculateFull()
$a1 = Get-CellSnapshot -Worksheet $ws -Address "A1"
$b1 = Get-CellSnapshot -Worksheet $ws -Address "B1"
$results += [pscustomobject]@{
    scenario_id = "W23-HI-001"
    lane = "hyperlink_formula_and_reference"
    primary_formula = [string]$ws.Range("A1").Formula
    primary_text = $a1.text
    primary_value2 = $a1.value2
    primary_type = $a1.type_code
    primary_cell_contents = $a1.cell_contents
    primary_t = $a1.t_value
    primary_n = $a1.n_value
    primary_font_underline = $a1.font_underline
    primary_hyperlinks_count = $a1.hyperlinks_count
    reference_formula = [string]$ws.Range("B1").Formula
    reference_text = $b1.text
    reference_value2 = $b1.value2
    reference_type = $b1.type_code
    reference_cell_contents = $b1.cell_contents
    reference_t = $b1.t_value
    reference_n = $b1.n_value
    reference_font_underline = $b1.font_underline
    reference_hyperlinks_count = $b1.hyperlinks_count
    notes = "Reference preserves plain text value but not hyperlink-style underline/publication treatment."
}

$ws.Range("A3").Formula = '=IMAGE("https://support.content.office.net/en-us/media/2d9e717a-0077-438f-8e5e-f85a1305d4ad.jpg","Sphere")'
$ws.Range("B3").Formula = "=A3"
Start-Sleep -Milliseconds 1000
if ($null -ne $syncAsync) {
    $syncAsync.Invoke($excel, @()) | Out-Null
}
$excel.CalculateFull()
$a3 = Get-CellSnapshot -Worksheet $ws -Address "A3"
$b3 = Get-CellSnapshot -Worksheet $ws -Address "B3"
$results += [pscustomobject]@{
    scenario_id = "W23-HI-002"
    lane = "image_formula_and_reference"
    primary_formula = [string]$ws.Range("A3").Formula
    primary_text = $a3.text
    primary_value2 = $a3.value2
    primary_type = $a3.type_code
    primary_cell_contents = $a3.cell_contents
    primary_t = $a3.t_value
    primary_n = $a3.n_value
    primary_font_underline = $a3.font_underline
    primary_hyperlinks_count = $a3.hyperlinks_count
    reference_formula = [string]$ws.Range("B3").Formula
    reference_text = $b3.text
    reference_value2 = $b3.value2
    reference_type = $b3.type_code
    reference_cell_contents = $b3.cell_contents
    reference_t = $b3.t_value
    reference_n = $b3.n_value
    reference_font_underline = $b3.font_underline
    reference_hyperlinks_count = $b3.hyperlinks_count
    notes = "Current baseline preserves a non-ordinary image payload across reference with TYPE=128."
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

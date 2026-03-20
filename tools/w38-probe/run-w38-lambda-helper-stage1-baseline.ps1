param(
    [string]$Manifest = "docs/function-lane/W38_SCENARIO_MANIFEST_SEED.csv",
    [string]$Out = ".tmp/w38-lambda-helper-stage1-results.csv"
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
    $admissionStatus = "set_ok"
    $stored = ""
    $text = ""
    $value2 = ""
    $matches = $false

    try {
        $cell.Formula2 = "=" + $row.formula
        $stored = [string]$cell.Formula2
        $text = [string]$cell.Text
        $rawValue2 = $cell.Value2
        if ($null -ne $rawValue2) {
            $value2 = [string]$rawValue2
        }
    } catch {
        $admissionStatus = "set_err"
        $value2 = [string]$_.Exception.HResult
    }

    if ($row.expected_status -eq "set_err") {
        $matches = ($admissionStatus -eq "set_err")
    } elseif ($admissionStatus -eq "set_ok") {
        if ($row.expected_text) {
            $matches = ($text -eq $row.expected_text)
        } elseif ($row.expected_value2 -ne "") {
            $expected = [double]$row.expected_value2
            $tol = if ($row.tolerance -eq "") { 0.0 } else { [double]$row.tolerance }
            $actual = [double]$value2
            $matches = [math]::Abs($actual - $expected) -le $tol
        }
    }

    $results += [pscustomobject]@{
        scenario_id = $row.scenario_id
        lane = $row.lane
        formula = "=" + $row.formula
        admission_status = $admissionStatus
        stored_formula2 = $stored
        text = $text
        value2 = $value2
        expected_status = $row.expected_status
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

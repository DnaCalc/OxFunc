param(
    [string]$Manifest = "docs/function-lane/W38_STAGE2_MAP_REDUCE_SCAN_SCENARIO_MANIFEST_SEED.csv",
    [string]$Out = ".tmp/w38-map-reduce-scan-stage2-results.csv"
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
foreach ($row in $rows) {
    $cell = $ws.Range($row.anchor_cell)
    $admissionStatus = "set_ok"
    $stored = ""
    $text = ""
    $value2 = ""
    $spillText = ""
    $matches = $false

    try {
        $cell.Formula2 = "=" + $row.formula
        Start-Sleep -Milliseconds 50
        $stored = [string]$cell.Formula2
        $text = [string]$cell.Text
        $rawValue2 = $cell.Value2
        if ($null -ne $rawValue2) {
            $value2 = [string]$rawValue2
        }
        if ($row.inspect_range) {
            $vals = @()
            foreach ($inspectCell in $ws.Range($row.inspect_range)) {
                $vals += [string]$inspectCell.Text
            }
            $spillText = ($vals -join "|")
        }
    } catch {
        $admissionStatus = "set_err"
        $value2 = [string]$_.Exception.HResult
    }

    if ($row.expected_status -eq "set_err") {
        $matches = ($admissionStatus -eq "set_err")
    } elseif ($admissionStatus -eq "set_ok") {
        $scalarMatch = $false
        if ($row.expected_text) {
            $scalarMatch = ($text -eq $row.expected_text)
        } elseif ($row.expected_value2 -ne "") {
            $expected = [double]$row.expected_value2
            $tol = if ($row.tolerance -eq "") { 0.0 } else { [double]$row.tolerance }
            $actual = [double]$value2
            $scalarMatch = [math]::Abs($actual - $expected) -le $tol
        } else {
            $scalarMatch = $true
        }

        $spillMatch = $true
        if ($row.expected_spill_text) {
            $spillMatch = ($spillText -eq $row.expected_spill_text)
        }

        $matches = ($scalarMatch -and $spillMatch)
    }

    $results += [pscustomobject]@{
        scenario_id = $row.scenario_id
        lane = $row.lane
        anchor_cell = $row.anchor_cell
        formula = "=" + $row.formula
        admission_status = $admissionStatus
        stored_formula2 = $stored
        text = $text
        value2 = $value2
        inspect_range = $row.inspect_range
        spill_text = $spillText
        expected_status = $row.expected_status
        expected_text = $row.expected_text
        expected_value2 = $row.expected_value2
        expected_spill_text = $row.expected_spill_text
        tolerance = $row.tolerance
        matches_expected = $matches
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
[System.Runtime.InteropServices.Marshal]::ReleaseComObject($ws) | Out-Null
[System.Runtime.InteropServices.Marshal]::ReleaseComObject($wb) | Out-Null
[System.Runtime.InteropServices.Marshal]::ReleaseComObject($excel) | Out-Null

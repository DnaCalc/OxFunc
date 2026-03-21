param(
    [string]$Manifest = "docs/function-lane/W41_WEB_TEXT_XML_SCENARIO_MANIFEST_SEED.csv",
    [string]$Out = ".tmp/w41-web-text-xml-results.csv"
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
    Start-Sleep -Milliseconds 200
    $excel.Calculate()

    switch ($row.expected_kind) {
        "spill_text" {
            $observed = if ($cell.HasSpill()) {
                $values = @()
                foreach ($spillCell in $cell.SpillingToRange.Cells) {
                    $values += [string]$spillCell.Text
                }
                $values -join "|"
            } else {
                [string]$cell.Text
            }
        }
        default {
            $observed = [string]$cell.Text
        }
    }

    $results += [pscustomobject]@{
        scenario_id = $row.scenario_id
        lane = $row.lane
        formula = "=" + $row.formula
        expected_kind = $row.expected_kind
        observed = $observed
        expected_value = $row.expected_value
        matches_expected = ($observed -eq $row.expected_value)
        notes = $row.notes
    }
    $rowIndex += 3
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

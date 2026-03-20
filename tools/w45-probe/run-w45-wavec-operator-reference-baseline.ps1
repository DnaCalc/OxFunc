param(
    [string]$OutPath = ".tmp/w45-wavec-operator-reference-results.csv"
)

$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..\")
Set-Location $repoRoot

$manifest = Import-Csv "docs/function-lane/W45_WAVEC_OPERATOR_REFERENCE_SCENARIO_MANIFEST_SEED.csv"
New-Item -ItemType Directory -Force -Path (Split-Path $OutPath) | Out-Null

$excel = New-Object -ComObject Excel.Application
$excel.Visible = $false
$excel.DisplayAlerts = $false

try {
    $wb = $excel.Workbooks.Add()
    $ws = $wb.Worksheets.Item(1)
    $ws.Range("A1").Value2 = 1
    $ws.Range("A2").Value2 = 2
    $ws.Range("B1").Value2 = 3
    $ws.Range("B2").Value2 = 4
    $ws.Range("C3").Value2 = 9

    $rows = foreach ($row in $manifest) {
        $cell = $ws.Range("D1")
        $cell.Clear()
        $cell.Formula2 = $row.formula
        $actualText = [string]$cell.Text
        $match = if ($row.expected_error) {
            $actualText -eq $row.expected_error
        } else {
            $actualText -eq $row.expected_text
        }
        [pscustomobject]@{
            scenario_id = $row.scenario_id
            wave = $row.wave
            formula = $row.formula
            expected_text = $row.expected_text
            expected_error = $row.expected_error
            actual_text = $actualText
            match = $match
        }
    }

    $rows | Export-Csv -NoTypeInformation -Encoding UTF8 -Path $OutPath
    $rows
}
finally {
    if ($wb) { $wb.Close($false) }
    $excel.Quit()
    if ($ws) { [void][System.Runtime.InteropServices.Marshal]::ReleaseComObject($ws) }
    if ($wb) { [void][System.Runtime.InteropServices.Marshal]::ReleaseComObject($wb) }
    if ($excel) { [void][System.Runtime.InteropServices.Marshal]::ReleaseComObject($excel) }
}

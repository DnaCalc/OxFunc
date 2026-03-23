param(
    [string]$Manifest = "docs/function-lane/W46_SCENARIO_MANIFEST_SEED.csv",
    [string]$Out = ".tmp/w46-call-register-id-results.csv"
)

$root = (Resolve-Path (Join-Path $PSScriptRoot "..\\..")).Path
$manifestPath = Join-Path $root $Manifest
$outPath = Join-Path $root $Out
$rows = Import-Csv $manifestPath

$excel = New-Object -ComObject Excel.Application
$excel.Visible = $false
$excel.DisplayAlerts = $false

$results = @()
foreach ($row in $rows) {
    $setupResult = $null
    if (-not [string]::IsNullOrWhiteSpace($row.setup_macro)) {
        $setupResult = $excel.ExecuteExcel4Macro($row.setup_macro)
    }

    $probeExpression = $row.probe_macro.Replace("{setup_result}", [string]$setupResult)
    $probeResult = $excel.ExecuteExcel4Macro($probeExpression)

    $results += [pscustomobject]@{
        scenario_id = $row.scenario_id
        lane = $row.lane
        status = $row.status
        setup_macro = $row.setup_macro
        setup_result = $setupResult
        probe_macro = $probeExpression
        probe_result = $probeResult
        expected = $row.expected_observable
        notes = $row.notes
    }
}

$outDir = Split-Path $outPath -Parent
if (-not (Test-Path $outDir)) {
    New-Item -ItemType Directory -Force -Path $outDir | Out-Null
}
$results | Export-Csv -NoTypeInformation -Path $outPath

$excel.Quit()
[System.Runtime.InteropServices.Marshal]::ReleaseComObject($excel) | Out-Null
[gc]::Collect()
[gc]::WaitForPendingFinalizers()

param(
    [Parameter(Mandatory = $true)]
    [string]$Results,

    [string]$OutReport = "",

    [string]$OutSummaryJson = ""
)

$ErrorActionPreference = "Stop"

$resultsPath = (Resolve-Path -Path $Results -ErrorAction Stop).Path
$rows = Import-Csv -Path $resultsPath
if (-not $rows -or $rows.Count -eq 0) {
    throw "No rows in results file: $resultsPath"
}

$executionObserved = @($rows | Where-Object { $_.execution_status -eq "observed" }).Count
$executionFailedTotal = @($rows | Where-Object { $_.execution_status -eq "failed" }).Count
$executionFailedExpected = @(
    $rows | Where-Object {
        $_.execution_status -eq "failed" -and
        $_.expected_status -eq "failed" -and
        $_.expectation_status -eq "matched"
    }
).Count
$executionFailedUnexpected = $executionFailedTotal - $executionFailedExpected
$expectMatched = @($rows | Where-Object { $_.expectation_status -eq "matched" }).Count
$expectMismatched = @($rows | Where-Object { $_.expectation_status -eq "mismatched" }).Count

if (-not [string]::IsNullOrWhiteSpace($OutReport)) {
    $reportPath = [System.IO.Path]::GetFullPath($OutReport)
    $reportDir = Split-Path -Path $reportPath -Parent
    if ($reportDir -and -not (Test-Path $reportDir)) {
        New-Item -ItemType Directory -Path $reportDir | Out-Null
    }
    $rows | Select-Object scenario_id,mechanism,execution_status,expected_status,expected_observable,expectation_status,observed_value,observed_text,notes |
        Export-Csv -Path $reportPath -NoTypeInformation -Encoding UTF8
    Write-Host "Report: $reportPath"
}

if ([string]::IsNullOrWhiteSpace($OutSummaryJson)) {
    if (-not [string]::IsNullOrWhiteSpace($OutReport)) {
        $OutSummaryJson = [System.IO.Path]::ChangeExtension([System.IO.Path]::GetFullPath($OutReport), ".json")
    }
    else {
        $OutSummaryJson = [System.IO.Path]::ChangeExtension($resultsPath, ".summary.json")
    }
}

$summaryPath = [System.IO.Path]::GetFullPath($OutSummaryJson)
$summaryDir = Split-Path -Path $summaryPath -Parent
if ($summaryDir -and -not (Test-Path $summaryDir)) {
    New-Item -ItemType Directory -Path $summaryDir | Out-Null
}

$summary = [ordered]@{
    results_path = $resultsPath
    row_count = $rows.Count
    execution = @{
        observed = $executionObserved
        failed_total = $executionFailedTotal
        failed_expected = $executionFailedExpected
        failed_unexpected = $executionFailedUnexpected
    }
    expectation = @{
        matched = $expectMatched
        mismatched = $expectMismatched
    }
    gate_status = if ($expectMismatched -eq 0 -and $executionFailedUnexpected -eq 0) { "green" } else { "needs_attention" }
}
$summary | ConvertTo-Json -Depth 6 | Set-Content -Path $summaryPath -Encoding UTF8

Write-Host "ABS entrypoint analysis complete."
Write-Host "Summary: $summaryPath"

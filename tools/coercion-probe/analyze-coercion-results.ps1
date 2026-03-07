param(
    [Parameter(Mandatory = $true)]
    [string]$Results,

    [Parameter(Mandatory = $true)]
    [string]$OutReport,

    [string]$Baseline = "",

    [string]$OutSummaryJson = ""
)

$ErrorActionPreference = "Stop"

$resultsPath = (Resolve-Path -Path $Results -ErrorAction Stop).Path
$rows = Import-Csv -Path $resultsPath

$reportRows = New-Object System.Collections.Generic.List[object]

function Add-Metric {
    param(
        [System.Collections.Generic.List[object]]$List,
        [string]$Metric,
        [string]$Value
    )

    $List.Add([PSCustomObject]@{
        metric = $Metric
        value = $Value
    })
}

Add-Metric -List $reportRows -Metric "rows_total" -Value ([string]$rows.Count)
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
$expectNotSpecified = @($rows | Where-Object { $_.expectation_status -eq "not_specified" }).Count

Add-Metric -List $reportRows -Metric "execution_observed" -Value ([string]$executionObserved)
Add-Metric -List $reportRows -Metric "execution_failed_total" -Value ([string]$executionFailedTotal)
Add-Metric -List $reportRows -Metric "execution_failed_expected" -Value ([string]$executionFailedExpected)
Add-Metric -List $reportRows -Metric "execution_failed_unexpected" -Value ([string]$executionFailedUnexpected)
Add-Metric -List $reportRows -Metric "expectation_matched" -Value ([string]$expectMatched)
Add-Metric -List $reportRows -Metric "expectation_mismatched" -Value ([string]$expectMismatched)
Add-Metric -List $reportRows -Metric "expectation_not_specified" -Value ([string]$expectNotSpecified)

foreach ($laneGroup in ($rows | Group-Object lane | Sort-Object Name)) {
    Add-Metric -List $reportRows -Metric ("lane_count_" + $laneGroup.Name) -Value ([string]$laneGroup.Count)
}

foreach ($axisGroup in ($rows | Group-Object coercion_axis | Sort-Object Name)) {
    Add-Metric -List $reportRows -Metric ("coercion_axis_count_" + $axisGroup.Name) -Value ([string]$axisGroup.Count)
}

$driftCount = 0
if (-not [string]::IsNullOrWhiteSpace($Baseline)) {
    $baselinePath = (Resolve-Path -Path $Baseline -ErrorAction Stop).Path
    $baselineRows = Import-Csv -Path $baselinePath
    $baselineMap = @{}
    foreach ($row in $baselineRows) {
        $baselineMap[[string]$row.scenario_id] = [string]$row.execution_status + "|" + [string]$row.expectation_status
    }

    foreach ($row in $rows) {
        $scenarioId = [string]$row.scenario_id
        if ($baselineMap.ContainsKey($scenarioId)) {
            $curr = [string]$row.execution_status + "|" + [string]$row.expectation_status
            if ($curr -ne $baselineMap[$scenarioId]) {
                $driftCount++
            }
        }
    }
}
Add-Metric -List $reportRows -Metric "drift_count" -Value ([string]$driftCount)

$outReportPath = [System.IO.Path]::GetFullPath($OutReport)
$outReportDir = Split-Path -Path $outReportPath -Parent
if ($outReportDir -and -not (Test-Path $outReportDir)) {
    New-Item -ItemType Directory -Path $outReportDir | Out-Null
}
$reportRows | Export-Csv -Path $outReportPath -NoTypeInformation -Encoding UTF8

if ([string]::IsNullOrWhiteSpace($OutSummaryJson)) {
    $OutSummaryJson = [System.IO.Path]::ChangeExtension($outReportPath, ".json")
}
$summaryPath = [System.IO.Path]::GetFullPath($OutSummaryJson)

$summary = [ordered]@{
    results_path = $resultsPath
    report_path = $outReportPath
    row_count = $rows.Count
    expectation = @{
        matched = $expectMatched
        mismatched = $expectMismatched
        not_specified = $expectNotSpecified
    }
    execution = @{
        observed = $executionObserved
        failed_total = $executionFailedTotal
        failed_expected = $executionFailedExpected
        failed_unexpected = $executionFailedUnexpected
    }
    gate_status = if ($expectMismatched -eq 0 -and $executionFailedUnexpected -eq 0) { "green" } else { "needs_attention" }
    drift_count = $driftCount
}
$summary | ConvertTo-Json -Depth 6 | Set-Content -Path $summaryPath -Encoding UTF8

Write-Host "Coercion analysis complete."
Write-Host "Report: $outReportPath"
Write-Host "Summary: $summaryPath"

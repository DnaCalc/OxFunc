param(
    [Parameter(Mandatory = $true)]
    [string]$Results,

    [Parameter(Mandatory = $true)]
    [string]$OutReport,

    [string]$OutSummaryJson = "",

    [bool]$RequireDualRuns = $true
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
$expectMatched = @($rows | Where-Object { $_.expectation_status -eq "matched" }).Count
$expectMismatched = @($rows | Where-Object { $_.expectation_status -eq "mismatched" }).Count
$expectNotSpecified = @($rows | Where-Object { $_.expectation_status -eq "not_specified" }).Count

Add-Metric -List $reportRows -Metric "execution_observed" -Value ([string]$executionObserved)
Add-Metric -List $reportRows -Metric "execution_failed_total" -Value ([string]$executionFailedTotal)
Add-Metric -List $reportRows -Metric "expectation_matched" -Value ([string]$expectMatched)
Add-Metric -List $reportRows -Metric "expectation_mismatched" -Value ([string]$expectMismatched)
Add-Metric -List $reportRows -Metric "expectation_not_specified" -Value ([string]$expectNotSpecified)

foreach ($runGroup in ($rows | Group-Object run_label | Sort-Object Name)) {
    Add-Metric -List $reportRows -Metric ("run_label_count_" + $runGroup.Name) -Value ([string]$runGroup.Count)
}

foreach ($laneGroup in ($rows | Group-Object lane | Sort-Object Name)) {
    Add-Metric -List $reportRows -Metric ("lane_count_" + $laneGroup.Name) -Value ([string]$laneGroup.Count)
    $laneMismatches = @($laneGroup.Group | Where-Object { $_.expectation_status -eq "mismatched" }).Count
    Add-Metric -List $reportRows -Metric ("lane_mismatches_" + $laneGroup.Name) -Value ([string]$laneMismatches)
}

$runLabels = @($rows | Select-Object -ExpandProperty run_label -Unique)
$hasDefaultRun = $runLabels -contains "default"
$hasCompatRun = $runLabels -contains "compat_template"
$dualRunSatisfied = $hasDefaultRun -and $hasCompatRun
Add-Metric -List $reportRows -Metric "dual_run_has_default" -Value ([string]$hasDefaultRun)
Add-Metric -List $reportRows -Metric "dual_run_has_compat_template" -Value ([string]$hasCompatRun)
Add-Metric -List $reportRows -Metric "dual_run_satisfied" -Value ([string]$dualRunSatisfied)

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
    }
    dual_runs = @{
        require_dual_runs = $RequireDualRuns
        has_default = $hasDefaultRun
        has_compat_template = $hasCompatRun
        dual_run_satisfied = $dualRunSatisfied
    }
    gate_status = if (
        $expectMismatched -eq 0 -and
        ((-not $RequireDualRuns) -or $dualRunSatisfied)
    ) { "green" } else { "needs_attention" }
}
$summary | ConvertTo-Json -Depth 6 | Set-Content -Path $summaryPath -Encoding UTF8

Write-Host "Registration flag analysis complete."
Write-Host "Report: $outReportPath"
Write-Host "Summary: $summaryPath"

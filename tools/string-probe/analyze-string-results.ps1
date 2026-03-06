param(
    [Parameter(Mandatory = $true)]
    [string]$Results,

    [string]$Baseline = "",

    [string]$OutReport = "",

    [string]$OutSummaryJson = ""
)

$ErrorActionPreference = "Stop"

$resultsPath = (Resolve-Path -Path $Results -ErrorAction Stop).Path
$rows = Import-Csv -Path $resultsPath
if (-not $rows -or $rows.Count -eq 0) {
    throw "No rows in results file: $resultsPath"
}

function Has-Column {
    param([object]$row, [string]$name)
    return ($row.PSObject.Properties.Name -contains $name)
}

$hasExpectation = Has-Column -row $rows[0] -name "expectation_status"

$total = $rows.Count
$byExecution = @{}
$rows | Group-Object execution_status | ForEach-Object { $byExecution[$_.Name] = $_.Count }

$byExpectation = @{}
if ($hasExpectation) {
    $rows | Group-Object expectation_status | ForEach-Object { $byExpectation[$_.Name] = $_.Count }
}

$expectedFailureMet = @()
$unexpectedFailure = @()
$expectationMismatch = @()

if ($hasExpectation) {
    $expectedFailureMet = @($rows | Where-Object { $_.execution_status -eq "failed" -and $_.expectation_status -eq "matched" })
    $unexpectedFailure = @($rows | Where-Object { $_.execution_status -eq "failed" -and $_.expectation_status -ne "matched" })
    $expectationMismatch = @($rows | Where-Object { $_.expectation_status -eq "mismatched" })
}
else {
    $unexpectedFailure = @($rows | Where-Object { $_.execution_status -eq "failed" })
}

Write-Host "String result analysis"
Write-Host "  results: $resultsPath"
Write-Host "  total rows: $total"
Write-Host "  execution status counts:"
foreach ($k in ($byExecution.Keys | Sort-Object)) {
    Write-Host "    $k : $($byExecution[$k])"
}
if ($hasExpectation) {
    Write-Host "  expectation status counts:"
    foreach ($k in ($byExpectation.Keys | Sort-Object)) {
        Write-Host "    $k : $($byExpectation[$k])"
    }
    Write-Host "  expected failures matched: $($expectedFailureMet.Count)"
    Write-Host "  expectation mismatches: $($expectationMismatch.Count)"
}
Write-Host "  unexpected failures: $($unexpectedFailure.Count)"

$reportRows = $rows | Select-Object scenario_id,lane,execution_status,expected_status,expected_observable,expectation_status,expectation_detail,primary_value2,primary_text_len,notes
if (-not [string]::IsNullOrWhiteSpace($OutReport)) {
    $outReportPath = [System.IO.Path]::GetFullPath($OutReport)
    $reportDir = Split-Path -Path $outReportPath -Parent
    if ($reportDir -and -not (Test-Path $reportDir)) { New-Item -ItemType Directory -Path $reportDir | Out-Null }
    $reportRows | Export-Csv -Path $outReportPath -NoTypeInformation -Encoding UTF8
    Write-Host "  report: $outReportPath"
}

$driftRows = @()
if (-not [string]::IsNullOrWhiteSpace($Baseline)) {
    $baselinePath = (Resolve-Path -Path $Baseline -ErrorAction Stop).Path
    $baselineRows = Import-Csv -Path $baselinePath

    $currentById = @{}
    foreach ($row in $rows) { $currentById[[string]$row.scenario_id] = $row }
    $baselineById = @{}
    foreach ($row in $baselineRows) { $baselineById[[string]$row.scenario_id] = $row }

    $fields = @("execution_status", "primary_value2", "primary_text_len", "comparison_bools")

    foreach ($id in ($currentById.Keys | Sort-Object)) {
        if (-not $baselineById.ContainsKey($id)) {
            $driftRows += [PSCustomObject]@{ scenario_id = $id; drift_type = "added"; field = ""; baseline = ""; current = "present" }
            continue
        }

        $cur = $currentById[$id]
        $base = $baselineById[$id]
        foreach ($f in $fields) {
            $curVal = [string]$cur.$f
            $baseVal = [string]$base.$f
            if ($curVal -cne $baseVal) {
                $driftRows += [PSCustomObject]@{ scenario_id = $id; drift_type = "changed"; field = $f; baseline = $baseVal; current = $curVal }
            }
        }
    }

    foreach ($id in ($baselineById.Keys | Sort-Object)) {
        if (-not $currentById.ContainsKey($id)) {
            $driftRows += [PSCustomObject]@{ scenario_id = $id; drift_type = "removed"; field = ""; baseline = "present"; current = "" }
        }
    }

    Write-Host "  drift rows: $($driftRows.Count)"

    if (-not [string]::IsNullOrWhiteSpace($OutReport)) {
        $driftPath = [System.IO.Path]::ChangeExtension([System.IO.Path]::GetFullPath($OutReport), ".drift.csv")
        $driftRows | Export-Csv -Path $driftPath -NoTypeInformation -Encoding UTF8
        Write-Host "  drift report: $driftPath"
    }
}

if (-not [string]::IsNullOrWhiteSpace($OutSummaryJson)) {
    $summaryPath = [System.IO.Path]::GetFullPath($OutSummaryJson)
    $summaryDir = Split-Path -Path $summaryPath -Parent
    if ($summaryDir -and -not (Test-Path $summaryDir)) { New-Item -ItemType Directory -Path $summaryDir | Out-Null }

    $summary = [ordered]@{
        results_path = $resultsPath
        total_rows = $total
        execution_status_counts = $byExecution
        expectation_status_counts = $byExpectation
        expected_failures_matched = $expectedFailureMet.Count
        unexpected_failures = $unexpectedFailure.Count
        expectation_mismatches = $expectationMismatch.Count
        drift_rows = $driftRows.Count
    }

    $summary | ConvertTo-Json -Depth 6 | Set-Content -Path $summaryPath -Encoding UTF8
    Write-Host "  summary json: $summaryPath"
}

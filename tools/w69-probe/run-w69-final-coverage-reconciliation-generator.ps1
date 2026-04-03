param(
  [string]$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
)

$ErrorActionPreference = 'Stop'
Set-Location $RepoRoot

$trancheRegisterPath = 'docs/function-lane/W69_SUPPORTED_SURFACE_WITNESS_TRANCHE_REGISTER.csv'
$trancheLedgerPath = 'docs/function-lane/W69_SUPPORTED_SURFACE_WITNESS_TRANCHE_LEDGER.md'
$gapLedgerPath = 'docs/function-lane/W69_SUPPORTED_SURFACE_WITNESS_GAP_LEDGER.md'

$register = Import-Csv -Path $trancheRegisterPath
$totalSupportedRows = 517
$witnessCoveredRows = 10
$remainingWitnessGapRows = 507
$treeState = if ((git status --short).Trim()) { 'dirty' } else { 'clean' }

$trancheCounts = foreach ($group in ($register | Group-Object tranche_id | Sort-Object Name)) {
  $first = $group.Group | Select-Object -First 1
  [ordered]@{
    tranche_id = $group.Name
    tranche_name = $first.tranche_name
    count = [int]$group.Count
    dependency_gate = if ([string]::IsNullOrWhiteSpace($first.dependency_gate)) { $null } else { $first.dependency_gate }
  }
}

$queuedRows = @($register | Where-Object { [string]::IsNullOrWhiteSpace($_.dependency_gate) })
$dependencyBlockedRows = @($register | Where-Object { -not [string]::IsNullOrWhiteSpace($_.dependency_gate) })

$dependencyBlockedByGate = foreach ($group in ($dependencyBlockedRows | Group-Object dependency_gate | Sort-Object Name)) {
  [ordered]@{
    dependency_gate = $group.Name
    count = [int]$group.Count
    tranche_id = ($group.Group | Select-Object -First 1).tranche_id
    tranche_name = ($group.Group | Select-Object -First 1).tranche_name
  }
}

$artifact = [ordered]@{
  witness_snapshot_id = 'oxfunc-semantic-witness-v2-final-supported-surface-reconciliation'
  witness_schema_version = 'v0.reconciliation'
  source_tranche_register_ref = $trancheRegisterPath
  source_tranche_ledger_ref = $trancheLedgerPath
  source_gap_ledger_ref = $gapLedgerPath
  supported_non_deferred_rows = $totalSupportedRows
  witness_covered_rows = $witnessCoveredRows
  remaining_witness_gap_rows = $remainingWitnessGapRows
  queued_in_tranche_rows = @($queuedRows).Count
  dependency_blocked_rows = @($dependencyBlockedRows).Count
  tranche_counts = $trancheCounts
  dependency_blocked_by_gate = $dependencyBlockedByGate
  publication_rule = 'Every supported non-deferred row must be either witness-covered, queued in one of the frozen tranches, or dependency-blocked on a retained live authority.'
  source_commit_short = (git rev-parse --short HEAD).Trim()
  source_commit_full = (git rev-parse HEAD).Trim()
  source_tree_state = $treeState
  note = 'This reconciliation is a publication surface for the frozen W069 support-surface ledger; it is not a second catalog.'
}

$outPath = 'docs/function-lane/W69_FINAL_SUPPORTED_SURFACE_COVERAGE_RECONCILIATION.json'
$artifact | ConvertTo-Json -Depth 8 | Set-Content -Path $outPath -Encoding utf8
Write-Host "Wrote final W069 coverage reconciliation to $outPath"

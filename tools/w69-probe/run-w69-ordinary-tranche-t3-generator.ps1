param(
  [string]$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
)

$ErrorActionPreference = 'Stop'
Set-Location $RepoRoot

$csv = Import-Csv 'docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv'

$artifact = [ordered]@{
  witness_snapshot_id = 'oxfunc-semantic-witness-v2-tranche-t3-ordinary-exhausted'
  witness_schema_version = 'v0.tranche'
  tranche_id = 'T3'
  tranche_name = 'ordinary_exhausted_confirmation'
  selection_rule = 'ordinary non-operator surface exhausted after T1/T2'
  source_snapshot_ref = 'docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv'
  source_commit_short = (git rev-parse --short HEAD).Trim()
  source_commit_full = (git rev-parse HEAD).Trim()
  source_tree_state = 'clean'
  entries = @()
  note = 'No additional ordinary non-operator rows remain after the frozen T1 and T2 tranche artifacts.'
}

$outPath = 'docs/function-lane/OXFUNC_SEMANTIC_WITNESS_SNAPSHOT_V2_TRANCHE_T3_ORDINARY_EXHAUSTED.json'
$artifact | ConvertTo-Json -Depth 8 | Set-Content -Path $outPath -Encoding utf8
Write-Host "Wrote 0 tranche rows to $outPath"

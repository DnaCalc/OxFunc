param(
  [string]$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
)

$ErrorActionPreference = 'Stop'
Set-Location $RepoRoot

$artifact = [ordered]@{
  witness_snapshot_id = 'oxfunc-semantic-witness-v2-tranche-sh2-seam-heavy-empty'
  witness_schema_version = 'v0.tranche'
  tranche_id = 'SH2'
  tranche_name = 'seam_heavy_exhausted_confirmation'
  selection_rule = 'no seam-heavy rows remain after SH1'
  source_inventory_ref = 'docs/function-lane/W69_SEAM_HEAVY_SUPPORTED_SURFACE_INVENTORY.csv'
  source_authoring_rules_ref = 'docs/function-lane/W69_SEAM_HEAVY_WITNESS_AUTHORING_RULES.md'
  source_commit_short = (git rev-parse --short HEAD).Trim()
  source_commit_full = (git rev-parse HEAD).Trim()
  source_tree_state = 'clean'
  entries = @()
  note = 'The retained seam-heavy inventory is already fully covered by the SH1 seed artifact.'
}

$outPath = 'docs/function-lane/OXFUNC_SEMANTIC_WITNESS_SNAPSHOT_V2_TRANCHE_SH2_SEAM_HEAVY_EMPTY.json'
$artifact | ConvertTo-Json -Depth 8 | Set-Content -Path $outPath -Encoding utf8
Write-Host "Wrote 0 seam-heavy rows to $outPath"

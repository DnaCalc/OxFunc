param(
  [string]$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
)

$ErrorActionPreference = 'Stop'
Set-Location $RepoRoot

$snapshotPath = 'docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv'
$registerPath = 'docs/function-lane/W69_SUPPORTED_SURFACE_WITNESS_TRANCHE_REGISTER.csv'
$outPath = 'docs/function-lane/OXFUNC_SEMANTIC_WITNESS_SNAPSHOT_V2_TRANCHE_T2_ORDINARY_CURATED.json'

$csv = Import-Csv $snapshotPath
$register = Import-Csv $registerPath | Where-Object { $_.tranche_id -eq 'T2' }
$snapshotById = @{}
foreach ($row in $csv) {
  $snapshotById[$row.surface_stable_id] = $row
}

$artifact = [ordered]@{
  witness_snapshot_id = 'oxfunc-semantic-witness-v2-tranche-t2-ordinary-curated'
  witness_schema_version = 'v0.tranche'
  tranche_id = 'T2'
  tranche_name = 'ordinary_curated_non_operator'
  selection_rule = 'tranche_register.tranche_id=T2'
  source_snapshot_ref = $snapshotPath
  source_commit_short = (git rev-parse --short HEAD).Trim()
  source_commit_full = (git rev-parse HEAD).Trim()
  source_tree_state = 'clean'
  entries = @()
}

foreach ($r in $register) {
  if (-not $snapshotById.ContainsKey($r.surface_stable_id)) {
    throw "Missing snapshot row for $($r.surface_stable_id)"
  }

  $s = $snapshotById[$r.surface_stable_id]
  $artifact.entries += [ordered]@{
    witness_schema_version = 'v0.tranche'
    tranche_id = 'T2'
    tranche_name = 'ordinary_curated_non_operator'
    surface_stable_id = $s.surface_stable_id
    canonical_surface_name = $s.canonical_surface_name
    metadata_status = $s.metadata_status
    category = $s.category
    special_interface_kind = $s.special_interface_kind
    admission_interface_kind = $s.admission_interface_kind
    arity_min = $s.arity_min
    arity_max = $s.arity_max
    signature_display = "$($s.canonical_surface_name)(...)"
    help_summary = "Ordinary curated-surface tranche seed for $($s.canonical_surface_name)."
    help_detail = "Seeded from the frozen W69 tranche register and parked baseline V1 export; fuller semantic witness enrichment will be filled by later W069 tranche work."
    semantic_modes = @()
    witness_examples = @()
    evidence_refs = @(
      @{ ref_kind='catalog_export_row'; ref_value="$snapshotPath#$($s.surface_stable_id)"; provenance_category='catalog_export' },
      @{ ref_kind='tranche_register_row'; ref_value="$registerPath#$($s.surface_stable_id)"; provenance_category='tranche_register' }
    )
    formal_refs = @()
    contract_refs = @()
    admitted_slice_note = 'Ordinary curated-surface witness seed pending fuller tranche enrichment.'
    orthogonal_validation_status = @('locale_version_sweeps_pending')
    current_support_basis = 'Supported in the parked baseline as an ordinary curated row; witness tranche seeded from the frozen T2 register and V1 snapshot.'
  }
}

$artifact | ConvertTo-Json -Depth 8 | Set-Content -Path $outPath -Encoding utf8
Write-Host "Wrote $($register.Count) tranche rows to $outPath"

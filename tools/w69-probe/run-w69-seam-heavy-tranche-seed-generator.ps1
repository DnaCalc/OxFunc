param(
  [string]$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
)

$ErrorActionPreference = 'Stop'
Set-Location $RepoRoot

$snapshotPath = 'docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv'
$inventoryPath = 'docs/function-lane/W69_SEAM_HEAVY_SUPPORTED_SURFACE_INVENTORY.csv'
$rulesPath = 'docs/function-lane/W69_SEAM_HEAVY_WITNESS_AUTHORING_RULES.md'
$outPath = 'docs/function-lane/OXFUNC_SEMANTIC_WITNESS_SNAPSHOT_V2_TRANCHE_SH1_SEAM_HEAVY.json'

$csv = Import-Csv $snapshotPath
$inventory = Import-Csv $inventoryPath
$snapshotById = @{}
foreach ($row in $csv) {
  $snapshotById[$row.surface_stable_id] = $row
}

function Get-SemanticModes {
  param(
    [string]$specialInterfaceKind
  )

  switch ($specialInterfaceKind) {
    'callable_helper_formation' { return @('callable_helper_formation', 'higher_order') }
    'callable_helper_runtime' { return @('callable_helper_runtime', 'higher_order') }
    'host_subscription_provider' { return @('host_subscription_provider') }
    'presentation_hinting_function' { return @('presentation_hinting') }
    'registered_external_invocation' { return @('registered_external_invocation') }
    'registered_external_registration' { return @('registered_external_registration') }
    'locale_default_profiled_parse' { return @('locale_profiled_parse') }
    'width_conversion_host_profile' { return @('width_conversion') }
    default { return @() }
  }
}

$artifact = [ordered]@{
  witness_snapshot_id = 'oxfunc-semantic-witness-v2-tranche-sh1-seam-heavy'
  witness_schema_version = 'v0.tranche'
  tranche_id = 'SH1'
  tranche_name = 'seam_heavy_supported_surface'
  selection_rule = 'W69_SEAM_HEAVY_SUPPORTED_SURFACE_INVENTORY.csv retained rows'
  source_snapshot_ref = $snapshotPath
  source_inventory_ref = $inventoryPath
  source_authoring_rules_ref = $rulesPath
  source_commit_short = (git rev-parse --short HEAD).Trim()
  source_commit_full = (git rev-parse HEAD).Trim()
  source_tree_state = 'clean'
  entries = @()
}

foreach ($r in $inventory | Sort-Object surface_stable_id) {
  if (-not $snapshotById.ContainsKey($r.surface_stable_id)) {
    throw "Missing snapshot row for $($r.surface_stable_id)"
  }

  $s = $snapshotById[$r.surface_stable_id]
  $modes = Get-SemanticModes -specialInterfaceKind $r.special_interface_kind

  $artifact.entries += [ordered]@{
    witness_schema_version = 'v0.tranche'
    tranche_id = 'SH1'
    tranche_name = 'seam_heavy_supported_surface'
    surface_stable_id = $s.surface_stable_id
    canonical_surface_name = $s.canonical_surface_name
    metadata_status = $s.metadata_status
    category = $s.category
    special_interface_kind = $s.special_interface_kind
    admission_interface_kind = $s.admission_interface_kind
    dependency_gate = $r.dependency_gate
    seam_note = $r.seam_note
    arity_min = $s.arity_min
    arity_max = $s.arity_max
    signature_display = "$($s.canonical_surface_name)(...)"
    help_summary = "Seam-heavy witness seed for $($s.canonical_surface_name)."
    help_detail = "Seeded from the parked baseline V1 export and the seam-heavy authoring rules; dependency gate $($r.dependency_gate) remains visible in the payload."
    semantic_modes = $modes
    witness_examples = @(
      @{
        example_id = "$($s.surface_stable_id)-SEAM-SEED-001"
        summary = "Dependency-gated seam-heavy witness seed."
        outcome_note = "Retained W69 seam gate $($r.dependency_gate) remains explicit in the witness row."
        evidence_ref_hint = "W69-SEAM-HEAVY-$($s.surface_stable_id)"
      }
    )
    evidence_refs = @(
      @{ ref_kind='catalog_export_row'; ref_value="$snapshotPath#$($s.surface_stable_id)"; provenance_category='catalog_export' },
      @{ ref_kind='seam_inventory_row'; ref_value="$inventoryPath#$($s.surface_stable_id)"; provenance_category='tranche_register' }
    )
    formal_refs = @()
    contract_refs = @()
    admitted_slice_note = 'Seam-heavy witness seed pending fuller dependency-gated W069 enrichment.'
    orthogonal_validation_status = @('live_gate_dependency_pending')
    current_support_basis = "Supported in the parked baseline as a seam-heavy row with retained dependency gate $($r.dependency_gate)."
  }
}

$artifact | ConvertTo-Json -Depth 8 | Set-Content -Path $outPath -Encoding utf8
Write-Host "Wrote $($inventory.Count) seam-heavy rows to $outPath"

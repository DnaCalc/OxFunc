param(
  [string]$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
)

$ErrorActionPreference = 'Stop'
Set-Location $RepoRoot

$snapshotPath = 'docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv'
$registerPath = 'docs/function-lane/W69_SUPPORTED_SURFACE_WITNESS_TRANCHE_REGISTER.csv'
$conventionsPath = 'docs/function-lane/W69_OPERATOR_AND_MODELED_WITNESS_CONVENTIONS.md'
$outPath = 'docs/function-lane/OXFUNC_SEMANTIC_WITNESS_SNAPSHOT_V2_TRANCHE_T3_OPERATOR_AND_MODELED.json'

$csv = Import-Csv $snapshotPath
$register = Import-Csv $registerPath | Where-Object { $_.tranche_id -eq 'T3' }
$snapshotById = @{}
foreach ($row in $csv) {
  $snapshotById[$row.surface_stable_id] = $row
}

$opImplicit = $snapshotById['FUNC.OP_IMPLICIT_INTERSECTION']
if (-not $opImplicit) {
  throw 'Missing doc-modeled operator seed row FUNC.OP_IMPLICIT_INTERSECTION in the snapshot export.'
}

function Get-OperatorModes {
  param(
    [string]$surfaceStableId
  )

  switch ($surfaceStableId) {
    'FUNC.OP_ADD' { return @('operator_surface', 'arithmetic_projection') }
    'FUNC.OP_SUBTRACT' { return @('operator_surface', 'arithmetic_projection') }
    'FUNC.OP_MULTIPLY' { return @('operator_surface', 'arithmetic_projection') }
    'FUNC.OP_DIVIDE' { return @('operator_surface', 'arithmetic_projection') }
    'FUNC.OP_POWER' { return @('operator_surface', 'arithmetic_projection') }
    'FUNC.OP_PERCENT' { return @('operator_surface', 'arithmetic_projection') }
    'FUNC.OP_NEGATE' { return @('operator_surface', 'unary_projection') }
    'FUNC.OP_UNARY_PLUS' { return @('operator_surface', 'unary_projection') }
    'FUNC.OP_EQUAL' { return @('operator_surface', 'comparison_projection') }
    'FUNC.OP_NOT_EQUAL' { return @('operator_surface', 'comparison_projection') }
    'FUNC.OP_GREATER_THAN' { return @('operator_surface', 'comparison_projection') }
    'FUNC.OP_GREATER_EQUAL' { return @('operator_surface', 'comparison_projection') }
    'FUNC.OP_LESS_THAN' { return @('operator_surface', 'comparison_projection') }
    'FUNC.OP_LESS_EQUAL' { return @('operator_surface', 'comparison_projection') }
    'FUNC.OP_RANGE_REF' { return @('operator_surface', 'reference_formation') }
    'FUNC.OP_UNION_REF' { return @('operator_surface', 'reference_formation') }
    'FUNC.OP_INTERSECTION_REF' { return @('operator_surface', 'reference_formation') }
    'FUNC.OP_TRIM_REF_BOTH' { return @('operator_surface', 'spill_reference_shaping') }
    'FUNC.OP_TRIM_REF_LEADING' { return @('operator_surface', 'spill_reference_shaping') }
    'FUNC.OP_TRIM_REF_TRAILING' { return @('operator_surface', 'spill_reference_shaping') }
    'FUNC.OP_SPILL_REF' { return @('operator_surface', 'spill_reference_shaping') }
    default { return @('operator_surface') }
  }
}

function Get-OperatorSignatureDisplay {
  param(
    [string]$surfaceStableId,
    [string]$canonicalName
  )

  switch ($surfaceStableId) {
    'FUNC.OP_ADD' { return 'A + B' }
    'FUNC.OP_SUBTRACT' { return 'A - B' }
    'FUNC.OP_MULTIPLY' { return 'A * B' }
    'FUNC.OP_DIVIDE' { return 'A / B' }
    'FUNC.OP_POWER' { return 'A ^ B' }
    'FUNC.OP_PERCENT' { return 'A%' }
    'FUNC.OP_NEGATE' { return '-A' }
    'FUNC.OP_UNARY_PLUS' { return '+A' }
    'FUNC.OP_EQUAL' { return 'A = B' }
    'FUNC.OP_NOT_EQUAL' { return 'A <> B' }
    'FUNC.OP_GREATER_THAN' { return 'A > B' }
    'FUNC.OP_GREATER_EQUAL' { return 'A >= B' }
    'FUNC.OP_LESS_THAN' { return 'A < B' }
    'FUNC.OP_LESS_EQUAL' { return 'A <= B' }
    'FUNC.OP_RANGE_REF' { return 'A:B' }
    'FUNC.OP_UNION_REF' { return 'A, B' }
    'FUNC.OP_INTERSECTION_REF' { return 'A B' }
    'FUNC.OP_TRIM_REF_BOTH' { return 'A @? B' }
    'FUNC.OP_TRIM_REF_LEADING' { return '@A' }
    'FUNC.OP_TRIM_REF_TRAILING' { return 'A@' }
    'FUNC.OP_SPILL_REF' { return 'A#' }
    'FUNC.OP_IMPLICIT_INTERSECTION' { return '@A' }
    default { return $canonicalName }
  }
}

$artifact = [ordered]@{
  witness_snapshot_id = 'oxfunc-semantic-witness-v2-tranche-t3-operator-and-modeled'
  witness_schema_version = 'v0.tranche'
  tranche_id = 'T3'
  tranche_name = 'operator_surface_and_modeled_seed'
  selection_rule = 'W69_SUPPORTED_SURFACE_WITNESS_TRANCHE_REGISTER.csv tranche_id=T3 plus doc_modeled OP_IMPLICIT_INTERSECTION'
  source_snapshot_ref = $snapshotPath
  source_register_ref = $registerPath
  source_conventions_ref = $conventionsPath
  source_commit_short = (git rev-parse --short HEAD).Trim()
  source_commit_full = (git rev-parse HEAD).Trim()
  source_tree_state = 'clean'
  entries = @()
}

foreach ($r in $register | Sort-Object surface_stable_id) {
  if (-not $snapshotById.ContainsKey($r.surface_stable_id)) {
    throw "Missing snapshot row for $($r.surface_stable_id)"
  }

  $s = $snapshotById[$r.surface_stable_id]
  $artifact.entries += [ordered]@{
    witness_schema_version = 'v0.tranche'
    tranche_id = 'T3'
    tranche_name = 'operator_surface_and_modeled_seed'
    surface_stable_id = $s.surface_stable_id
    canonical_surface_name = $s.canonical_surface_name
    metadata_status = $s.metadata_status
    category = $s.category
    special_interface_kind = $s.special_interface_kind
    admission_interface_kind = $s.admission_interface_kind
    signature_display = Get-OperatorSignatureDisplay -surfaceStableId $s.surface_stable_id -canonicalName $s.canonical_surface_name
    help_summary = "Operator witness seed for $($s.canonical_surface_name)."
    help_detail = "Seeded from the frozen T3 operator tranche; operator syntax remains explicit and the witness payload must not collapse into ordinary function-call prose."
    semantic_modes = @(Get-OperatorModes -surfaceStableId $s.surface_stable_id)
    witness_examples = @(
      @{
        example_id = "$($s.surface_stable_id)-OP-SEED-001"
        summary = "Operator surface witness seed."
        outcome_note = "Operator witness seed retains the surfaced operator form."
        evidence_ref_hint = "W69-OPERATOR-$($s.surface_stable_id)"
      }
    )
    evidence_refs = @(
      @{ ref_kind='catalog_export_row'; ref_value="$snapshotPath#$($s.surface_stable_id)"; provenance_category='catalog_export' },
      @{ ref_kind='tranche_register_row'; ref_value="$registerPath#$($s.surface_stable_id)"; provenance_category='tranche_register' }
    )
    formal_refs = @()
    contract_refs = @()
    admitted_slice_note = 'Operator witness seed pending fuller V2 enrichment.'
    orthogonal_validation_status = @('operator_syntax_review_pending')
    current_support_basis = "Supported in the parked baseline as an operator row; seeded through the frozen T3 register."
  }
}

$artifact.entries += [ordered]@{
  witness_schema_version = 'v0.tranche'
  tranche_id = 'T3'
  tranche_name = 'operator_surface_and_modeled_seed'
  surface_stable_id = $opImplicit.surface_stable_id
  canonical_surface_name = $opImplicit.canonical_surface_name
  metadata_status = $opImplicit.metadata_status
  category = $opImplicit.category
  special_interface_kind = $opImplicit.special_interface_kind
  admission_interface_kind = $opImplicit.admission_interface_kind
  signature_display = Get-OperatorSignatureDisplay -surfaceStableId $opImplicit.surface_stable_id -canonicalName $opImplicit.canonical_surface_name
  help_summary = "Doc-modeled operator seed for implicit intersection."
  help_detail = "This row keeps the legacy @ and _xlfn.SINGLE compatibility story explicit while remaining a doc-modeled operator seed aligned to the operator family conventions."
  semantic_modes = @('implicit_intersection', 'doc_modeled_operator', 'context_sensitive_scalarization')
  witness_examples = @(
    @{
      example_id = 'FUNC.OP_IMPLICIT_INTERSECTION-OP-SEED-001'
      summary = 'Doc-modeled operator seed.'
      outcome_note = 'The @ / _xlfn.SINGLE compatibility story stays explicit in the witness row.'
      evidence_ref_hint = 'W14-IMPLICIT-INTERSECTION-BL-20260401'
    }
  )
  evidence_refs = @(
    @{ ref_kind='catalog_export_row'; ref_value="$snapshotPath#FUNC.OP_IMPLICIT_INTERSECTION"; provenance_category='catalog_export' },
    @{ ref_kind='operator_conventions_note'; ref_value=$conventionsPath; provenance_category='authoring_rules' }
  )
  formal_refs = @()
  contract_refs = @()
  admitted_slice_note = 'Doc-modeled operator seed retained for compatibility and context-sensitive scalarization.'
  orthogonal_validation_status = @('legacy_compatibility_review_pending')
  current_support_basis = 'Supported in the parked baseline as a doc-modeled operator seed retained for compatibility.'
}

$artifact | ConvertTo-Json -Depth 8 | Set-Content -Path $outPath -Encoding utf8
Write-Host "Wrote $($artifact.entries.Count) operator/model rows to $outPath"

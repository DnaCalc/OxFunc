param(
  [string]$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
)

$ErrorActionPreference = 'Stop'
Set-Location $RepoRoot

$csv = Import-Csv 'docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv'
$deferred = Import-Csv 'docs/function-lane/W50_DEFERRED_CURRENT_VERSION_INVENTORY.csv' | ForEach-Object { $_.entry_name }
$covered = @('FUNC.GROUPBY','FUNC.HLOOKUP','FUNC.HYPERLINK','FUNC.IF','FUNC.IMAGE','FUNC.LET','FUNC.OP_IMPLICIT_INTERSECTION','FUNC.SUM','FUNC.VLOOKUP','FUNC.XLOOKUP')
$t1 = $csv | Where-Object { $_.surface_stable_id -like 'FUNC.*' -and ($deferred -notcontains $_.canonical_surface_name) -and ($covered -notcontains $_.surface_stable_id) -and $_.metadata_status -eq 'function_meta_extracted' -and $_.special_interface_kind -eq 'ordinary' -and $_.category -ne 'Operators' } | Sort-Object surface_stable_id

$artifact = [ordered]@{
  witness_snapshot_id = 'oxfunc-semantic-witness-v2-tranche-t1-ordinary-extracted'
  witness_schema_version = 'v0.tranche'
  tranche_id = 'T1'
  tranche_name = 'ordinary_extracted_non_operator'
  selection_rule = 'metadata_status=function_meta_extracted AND special_interface_kind=ordinary AND category!=Operators'
  source_snapshot_ref = 'docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv'
  source_commit_short = (git rev-parse --short HEAD).Trim()
  source_commit_full = (git rev-parse HEAD).Trim()
  source_tree_state = 'clean'
  entries = @()
}

foreach ($r in $t1) {
  $artifact.entries += [ordered]@{
    witness_schema_version = 'v0.tranche'
    tranche_id = 'T1'
    tranche_name = 'ordinary_extracted_non_operator'
    surface_stable_id = $r.surface_stable_id
    canonical_surface_name = $r.canonical_surface_name
    metadata_status = $r.metadata_status
    category = $r.category
    special_interface_kind = $r.special_interface_kind
    admission_interface_kind = $r.admission_interface_kind
    arity_min = $r.arity_min
    arity_max = $r.arity_max
    signature_display = "$($r.canonical_surface_name)(...)"
    help_summary = "Ordinary extracted-surface tranche seed for $($r.canonical_surface_name)."
    help_detail = "Seeded from the parked baseline V1 export; full semantic witness enrichment will be filled by later W069 tranche work."
    semantic_modes = @()
    witness_examples = @()
    evidence_refs = @(@{ ref_kind='catalog_export_row'; ref_value="docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv#$($r.surface_stable_id)"; provenance_category='catalog_export' })
    formal_refs = @()
    contract_refs = @()
    admitted_slice_note = 'Ordinary extracted-surface witness seed pending fuller tranche enrichment.'
    orthogonal_validation_status = @('locale_version_sweeps_pending')
    current_support_basis = 'Supported in the parked baseline as an ordinary extracted row; witness tranche seeded from the V1 snapshot.'
  }
}

$outPath = 'docs/function-lane/OXFUNC_SEMANTIC_WITNESS_SNAPSHOT_V2_TRANCHE_T1_ORDINARY_EXTRACTED.json'
$artifact | ConvertTo-Json -Depth 8 | Set-Content -Path $outPath -Encoding utf8
Write-Host "Wrote $($t1.Count) tranche rows to $outPath"

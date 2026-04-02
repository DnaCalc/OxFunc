[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$snapshotPath = Join-Path $repoRoot 'docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv'
$outputPath = Join-Path $repoRoot 'docs/function-lane/OXFUNC_SEMANTIC_WITNESS_SNAPSHOT_V2_SEED_HVLOOKUP.json'
$contractRef = 'docs/function-lane/FUNCTION_SLICE_LOOKUP_AND_LOGICAL_RESIDUALS_CONTRACT_PRELIM.md'
$evidenceId = 'W68-LOOKUP-LOGICAL-RESIDUALS-BL-20260401'
$probeRef = 'tools/w68-probe/run-w68-lookup-logical-baseline.ps1'
$runtimeRef = 'crates/oxfunc_core/src/functions/vhlookup_family.rs'
$formalRef = 'formal/lean/OxFunc/Functions/VhlookupFamily.lean'

function New-ArgSpec($index, $name, $required, $hint, $note) {
    [ordered]@{
        arg_index = $index
        arg_name = $name
        arg_required = $required
        arg_type_hint = $hint
        arg_behavior_note = $note
    }
}

function New-Example($id, $summary, $outcome) {
    [ordered]@{
        example_id = $id
        summary = $summary
        outcome_note = $outcome
        evidence_ref_hint = $evidenceId
    }
}

function New-ProvenanceRef($kind, $value, $category) {
    [ordered]@{
        ref_kind = $kind
        ref_value = $value
        provenance_category = $category
    }
}

function Get-FamilyTemplate($name) {
    switch ($name) {
        'HLOOKUP' {
            return [ordered]@{
                signature_display = 'HLOOKUP(lookup_value, table_array, row_index_num, [range_lookup])'
                arg_specs = @(
                    (New-ArgSpec 1 'lookup_value' $true 'scalar' 'Supports exact and approximate lookup, including the current-baseline wildcard text lane.'),
                    (New-ArgSpec 2 'table_array' $true 'reference_or_array' 'Reference and array structure stays visible through the adapter seam.'),
                    (New-ArgSpec 3 'row_index_num' $true 'number' 'Truncates toward zero before validation.'),
                    (New-ArgSpec 4 'range_lookup' $false 'logical' 'Omitted means approximate-match mode.')
                )
                help_summary = 'Looks for a value in the first row of a table and returns a value from a specified row.'
                help_detail = 'Current-baseline OxFunc support covers exact and approximate numeric lookup, wildcard text lookup on the exact lane, logical lookup values, and row-index validation.'
                semantic_modes = @(
                    'exact_match',
                    'approximate_match_sorted',
                    'wildcard_text_lookup',
                    'row_index_validation'
                )
                witness_examples = @(
                    (New-Example 'W68-HLOOKUP-001' 'Exact numeric HLOOKUP baseline.' 'Returns 20 for HLOOKUP(2,{1,2,3;10,20,30},2,FALSE).'),
                    (New-Example 'W68-HLOOKUP-002' 'Approximate HLOOKUP defaults when range_lookup is omitted.' 'Returns 20 for HLOOKUP(2.9,{1,2,3;10,20,30},2).'),
                    (New-Example 'W68-HLOOKUP-003' 'Exact HLOOKUP supports the current-baseline wildcard lane.' 'Returns 20 for HLOOKUP("b*",{"abc","bcd";10,20},2,FALSE).'),
                    (New-Example 'W68-HLOOKUP-ERR-003' 'Row index beyond table height returns #REF!.' 'Range/index validation is part of the admitted slice.')
                )
                admitted_slice_note = 'Current-baseline OxFunc support for HLOOKUP covers exact and approximate lookup, wildcard matching on the exact lane, logical lookup values, row-index truncation/validation, and the replayed error lanes captured in W68.'
            }
        }
        'VLOOKUP' {
            return [ordered]@{
                signature_display = 'VLOOKUP(lookup_value, table_array, col_index_num, [range_lookup])'
                arg_specs = @(
                    (New-ArgSpec 1 'lookup_value' $true 'scalar' 'Supports exact and approximate lookup, including the current-baseline wildcard text lane.'),
                    (New-ArgSpec 2 'table_array' $true 'reference_or_array' 'Reference and array structure stays visible through the adapter seam.'),
                    (New-ArgSpec 3 'col_index_num' $true 'number' 'Truncates toward zero before validation.'),
                    (New-ArgSpec 4 'range_lookup' $false 'logical' 'Omitted means approximate-match mode.')
                )
                help_summary = 'Looks for a value in the first column of a table and returns a value from a specified column.'
                help_detail = 'Current-baseline OxFunc support covers exact and approximate numeric lookup, wildcard text lookup on the exact lane, logical lookup values, and column-index validation.'
                semantic_modes = @(
                    'exact_match',
                    'approximate_match_sorted',
                    'wildcard_text_lookup',
                    'column_index_validation'
                )
                witness_examples = @(
                    (New-Example 'W68-VLOOKUP-001' 'Exact numeric VLOOKUP baseline.' 'Returns 20 for VLOOKUP(2,{1,10;2,20;3,30},2,FALSE).'),
                    (New-Example 'W68-VLOOKUP-002' 'Approximate VLOOKUP defaults when range_lookup is omitted.' 'Returns 20 for VLOOKUP(2.9,{1,10;2,20;3,30},2).'),
                    (New-Example 'W68-VLOOKUP-003' 'Exact VLOOKUP supports the current-baseline wildcard lane.' 'Returns 20 for VLOOKUP("b*",{"abc",10;"bcd",20},2,FALSE).'),
                    (New-Example 'W68-VLOOKUP-ERR-003' 'Column index beyond table width returns #REF!.' 'Range/index validation is part of the admitted slice.')
                )
                admitted_slice_note = 'Current-baseline OxFunc support for VLOOKUP covers exact and approximate lookup, wildcard matching on the exact lane, logical lookup values, column-index truncation/validation, and the replayed error lanes captured in W68.'
            }
        }
        default {
            throw "Unsupported witness seed family member: $name"
        }
    }
}

function New-WitnessEntry($row) {
    $template = Get-FamilyTemplate $row.canonical_surface_name
    $surfaceId = $row.surface_stable_id

    [ordered]@{
        witness_schema_version = 'v0.seed'
        surface_stable_id = $surfaceId
        canonical_surface_name = $row.canonical_surface_name
        category = $row.category
        metadata_status = $row.metadata_status
        signature_display = $template.signature_display
        arg_specs = $template.arg_specs
        help_summary = $template.help_summary
        help_detail = $template.help_detail
        semantic_modes = $template.semantic_modes
        witness_examples = $template.witness_examples
        admitted_slice_note = $template.admitted_slice_note
        orthogonal_validation_status = @('locale_version_sweeps_pending')
        current_support_basis = "Supported in the parked non-deferred baseline through the retained W68 contract, replay probe, runtime implementation, formal lookup substrate, and V1 export row."
        provenance_refs = @(
            (New-ProvenanceRef 'catalog_export_row' "docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv#$surfaceId" 'catalog_export'),
            (New-ProvenanceRef 'contract_doc' $contractRef 'contract_artifact'),
            (New-ProvenanceRef 'evidence_id' $evidenceId 'native_excel_replay'),
            (New-ProvenanceRef 'replay_probe' $probeRef 'native_excel_replay'),
            (New-ProvenanceRef 'runtime_source' $runtimeRef 'runtime_test'),
            (New-ProvenanceRef 'formal_artifact' $formalRef 'formal_artifact')
        )
        snapshot_generation = $row.snapshot_generation
        source_commit_short = $row.source_commit_short
        source_commit_full = $row.source_commit_full
        source_tree_state = $row.source_tree_state
    }
}

$rows = Import-Csv $snapshotPath | Where-Object { $_.surface_stable_id -in @('FUNC.HLOOKUP', 'FUNC.VLOOKUP') } | Sort-Object surface_stable_id

if ($rows.Count -ne 2) {
    throw "Expected exactly 2 V1 rows for HLOOKUP/VLOOKUP; found $($rows.Count)."
}

$document = [ordered]@{
    witness_snapshot_id = 'oxfunc-semantic-witness-v2-seed-hvlookup'
    witness_schema_version = 'v0.seed'
    source_snapshot_ref = 'docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv'
    seed_family = 'vhlookup'
    entries = @($rows | ForEach-Object { New-WitnessEntry $_ })
}

$json = $document | ConvertTo-Json -Depth 8
[System.IO.File]::WriteAllText($outputPath, $json + [Environment]::NewLine, (New-Object System.Text.UTF8Encoding($false)))

Write-Host "Generated $outputPath from $snapshotPath"

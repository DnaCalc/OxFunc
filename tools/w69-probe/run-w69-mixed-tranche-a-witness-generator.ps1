[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$snapshotPath = Join-Path $repoRoot 'docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv'
$outputPath = Join-Path $repoRoot 'docs/function-lane/OXFUNC_SEMANTIC_WITNESS_SNAPSHOT_V2_SEED_MIXED_TRANCHE_A.json'

function New-ArgSpec($index, $name, $required, $hint, $note) {
    [ordered]@{
        arg_index = $index
        arg_name = $name
        arg_required = $required
        arg_type_hint = $hint
        arg_behavior_note = $note
    }
}

function New-Example($id, $summary, $outcome, $evidenceId) {
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

function Get-SurfaceTemplate($surfaceId) {
    switch ($surfaceId) {
        'FUNC.SUM' {
            return [ordered]@{
                signature_display = 'SUM(number1, [number2], ...)'
                arg_specs = @(
                    (New-ArgSpec 1 'number1' $true 'scalar_or_reference' 'Starts the additive fold and admits ordinary scalar and reference-fed lanes.'),
                    (New-ArgSpec 2 'number2' $false 'scalar_or_reference' 'Represents the repeated optional additive arguments for the bounded witness slice.')
                )
                help_summary = 'Adds numbers, references, and arrays and returns their total.'
                help_detail = 'The first mixed tranche treats SUM as the ordinary scalar/aggregate anchor for downstream witness rendering.'
                semantic_modes = @('scalar_aggregate', 'reference_fold', 'high_arity_repeat')
                witness_examples = @(
                    (New-Example 'W9-SUM-001' 'Ordinary SUM scalar addition baseline.' 'Returns 6 for SUM(1,2,3).' 'W9-XLL-BL-20260308'),
                    (New-Example 'W9-SUM-REF-001' 'Reference-fed SUM lanes stay in the ordinary aggregate family.' 'Current baseline evidence includes aggregate SUM rows through the retained bridge and replay surfaces.' 'W9-XLL-BL-20260308')
                )
                admitted_slice_note = 'The first widened tranche uses SUM as the ordinary aggregate anchor and carries forward the parked baseline aggregate behavior rather than a seam-heavy special case.'
                current_support_basis = 'Supported in the parked baseline as an ordinary aggregate function with retained runtime and formal anchors plus bridge-backed replay evidence.'
                provenance_refs = @(
                    (New-ProvenanceRef 'catalog_export_row' 'docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv#FUNC.SUM' 'catalog_export'),
                    (New-ProvenanceRef 'contract_doc' 'docs/function-lane/FUNCTION_SLICE_SUM_CONTRACT_PRELIM.md' 'contract_artifact'),
                    (New-ProvenanceRef 'evidence_id' 'W9-XLL-BL-20260308' 'native_excel_replay'),
                    (New-ProvenanceRef 'runtime_source' 'crates/oxfunc_core/src/functions/sum.rs' 'runtime_test'),
                    (New-ProvenanceRef 'formal_artifact' 'formal/lean/OxFunc/Functions/Sum.lean' 'formal_artifact')
                )
            }
        }
        'FUNC.IF' {
            return [ordered]@{
                signature_display = 'IF(logical_test, value_if_true, [value_if_false])'
                arg_specs = @(
                    (New-ArgSpec 1 'logical_test' $true 'scalar' 'Condition evaluation drives the branch selection.'),
                    (New-ArgSpec 2 'value_if_true' $true 'value' 'Returned when the logical test evaluates truthy.'),
                    (New-ArgSpec 3 'value_if_false' $false 'value' 'Optional false branch; omission follows the current baseline IF defaulting rules.')
                )
                help_summary = 'Returns one value when a condition is true and another value when it is false.'
                help_detail = 'The first widened tranche uses IF as the control-flow anchor for mixed witness rollout.'
                semantic_modes = @('branching_control', 'omittable_false_branch', 'scalar_condition')
                witness_examples = @(
                    (New-Example 'W10-IF-001' 'Direct true-branch baseline.' 'Returns the true branch when logical_test evaluates true.' 'W10-TENMIX-SEED-20260308'),
                    (New-Example 'W10-IF-002' 'Direct false-branch baseline.' 'Returns the false branch when logical_test evaluates false.' 'W10-TENMIX-SEED-20260308')
                )
                admitted_slice_note = 'The first widened tranche uses IF as the ordinary logical-control representative and carries forward the parked direct-branch baseline.'
                current_support_basis = 'Supported in the parked baseline as the primary logical-control surface, with retained contract, runtime, and formal anchors.'
                provenance_refs = @(
                    (New-ProvenanceRef 'catalog_export_row' 'docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv#FUNC.IF' 'catalog_export'),
                    (New-ProvenanceRef 'contract_doc' 'docs/function-lane/FUNCTION_SLICE_IF_CONTRACT_PRELIM.md' 'contract_artifact'),
                    (New-ProvenanceRef 'evidence_id' 'W10-TENMIX-SEED-20260308' 'native_excel_replay'),
                    (New-ProvenanceRef 'runtime_source' 'crates/oxfunc_core/src/functions/if_fn.rs' 'runtime_test'),
                    (New-ProvenanceRef 'formal_artifact' 'formal/lean/OxFunc/Functions/IfFn.lean' 'formal_artifact')
                )
            }
        }
        'FUNC.XLOOKUP' {
            return [ordered]@{
                signature_display = 'XLOOKUP(lookup_value, lookup_array, return_array, [if_not_found], [match_mode], [search_mode])'
                arg_specs = @(
                    (New-ArgSpec 1 'lookup_value' $true 'scalar' 'Current mixed tranche carries the modern lookup-value entry point.'),
                    (New-ArgSpec 2 'lookup_array' $true 'reference_or_array' 'Lookup vector used for match search.'),
                    (New-ArgSpec 3 'return_array' $true 'reference_or_array' 'Return vector or area paired to the lookup array.'),
                    (New-ArgSpec 4 'if_not_found' $false 'value' 'Optional not-found projection lane.'),
                    (New-ArgSpec 5 'match_mode' $false 'number' 'Controls exact, approximate, and wildcard matching.'),
                    (New-ArgSpec 6 'search_mode' $false 'number' 'Controls forward, reverse, and binary-style search direction.')
                )
                help_summary = 'Searches a range or array and returns a corresponding item from a second range or array.'
                help_detail = 'The first widened tranche uses XLOOKUP as the modern lookup representative beyond the bounded HVLOOKUP family seed.'
                semantic_modes = @('modern_lookup', 'not_found_projection', 'match_mode_selection', 'search_mode_selection')
                witness_examples = @(
                    (New-Example 'W10-XLOOKUP-001' 'Direct exact-match XLOOKUP baseline.' 'Returns the paired return-array value for an exact match.' 'W10-LOOKUP-XLL-20260310'),
                    (New-Example 'W10-XLOOKUP-002' 'Reference-return XLOOKUP lanes remain visible in the widened witness tranche.' 'Retained bridge evidence captures current baseline address/range-composition parity.' 'W10-LOOKUP-XLL-20260310')
                )
                admitted_slice_note = 'The first widened tranche uses XLOOKUP as the modern lookup representative with the parked baseline match/search and return-array behavior.'
                current_support_basis = 'Supported in the parked baseline through the retained XLOOKUP contract, lookup bridge evidence, runtime implementation, and Lean lookup model.'
                provenance_refs = @(
                    (New-ProvenanceRef 'catalog_export_row' 'docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv#FUNC.XLOOKUP' 'catalog_export'),
                    (New-ProvenanceRef 'contract_doc' 'docs/function-lane/FUNCTION_SLICE_XLOOKUP_CONTRACT_PRELIM.md' 'contract_artifact'),
                    (New-ProvenanceRef 'evidence_id' 'W10-LOOKUP-XLL-20260310' 'native_excel_replay'),
                    (New-ProvenanceRef 'runtime_source' 'crates/oxfunc_core/src/functions/xlookup.rs' 'runtime_test'),
                    (New-ProvenanceRef 'formal_artifact' 'formal/lean/OxFunc/Functions/Xlookup.lean' 'formal_artifact')
                )
            }
        }
        'FUNC.LET' {
            return [ordered]@{
                signature_display = 'LET(name1, value1, calculation_or_name2, [value2], ...)'
                arg_specs = @(
                    (New-ArgSpec 1 'name1' $true 'name' 'Introduces the first lexical binding.'),
                    (New-ArgSpec 2 'value1' $true 'value' 'Supplies the first bound value or expression result.'),
                    (New-ArgSpec 3 'calculation_or_name2' $true 'value_or_name' 'Either begins the final calculation or continues the binding chain.')
                )
                help_summary = 'Assigns names to calculation results and reuses them inside a single formula.'
                help_detail = 'The first widened tranche uses LET as the callable/helper formation representative without reopening the cross-repo callable freeze.'
                semantic_modes = @('lexical_binding', 'sequential_name_value_pairs', 'final_calculation_projection')
                witness_examples = @(
                    (New-Example 'W38-LET-001' 'Sequential LET binding baseline.' 'Current baseline evidence pins sequential LET bindings and reuse inside the final calculation.' 'W38-LAMBDA-HELPER-STAGE1-20260319'),
                    (New-Example 'W38-LET-002' 'LET carries the lexical-capture lane into immediately invoked LAMBDA usage.' 'The retained Stage 1 helper packet pins lexical capture through the admitted callable slice.' 'W38-LAMBDA-HELPER-STAGE1-20260319')
                )
                admitted_slice_note = 'The first widened tranche uses LET as the callable/helper representative for lexical binding and final-calculation projection in the admitted Stage 1 helper slice.'
                current_support_basis = 'Supported in the parked baseline through the retained helper Stage 1 contract, native replay evidence, runtime helper implementation, and formal callable-helper substrate.'
                provenance_refs = @(
                    (New-ProvenanceRef 'catalog_export_row' 'docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv#FUNC.LET' 'catalog_export'),
                    (New-ProvenanceRef 'contract_doc' 'docs/function-lane/FUNCTION_SLICE_FUNCTIONAL_LAMBDA_AND_HELPER_STAGE1_CONTRACT_PRELIM.md' 'contract_artifact'),
                    (New-ProvenanceRef 'evidence_id' 'W38-LAMBDA-HELPER-STAGE1-20260319' 'native_excel_replay'),
                    (New-ProvenanceRef 'runtime_source' 'crates/oxfunc_core/src/functions/callable_helpers.rs' 'runtime_test'),
                    (New-ProvenanceRef 'formal_artifact' 'formal/lean/OxFunc/Functions/FunctionalLambdaHelpers.lean' 'formal_artifact')
                )
            }
        }
        'FUNC.IMAGE' {
            return [ordered]@{
                signature_display = 'IMAGE(source, [alt_text], [sizing], [height], [width])'
                arg_specs = @(
                    (New-ArgSpec 1 'source' $true 'text' 'Image source or provider-backed locator string.'),
                    (New-ArgSpec 2 'alt_text' $false 'text' 'Optional alternate text for presentation-aware hosts.'),
                    (New-ArgSpec 3 'sizing' $false 'number' 'Optional sizing mode selection.'),
                    (New-ArgSpec 4 'height' $false 'number' 'Optional explicit height input.'),
                    (New-ArgSpec 5 'width' $false 'number' 'Optional explicit width input.')
                )
                help_summary = 'Returns a host-managed image value from a source URL or equivalent image locator.'
                help_detail = 'The first widened tranche uses IMAGE as the rich-value publication representative rather than an ordinary scalar-return function.'
                semantic_modes = @('rich_value_publication', 'provider_bound_media', 'presentation_aware_output')
                witness_examples = @(
                    (New-Example 'W23-IMAGE-001' 'Support-example IMAGE lane preserves a non-ordinary payload.' 'Current baseline evidence records IMAGE as a retained rich-value/publication surface rather than plain text.' 'W23-HI-VALMODEL-20260321'),
                    (New-Example 'W23-IMAGE-002' 'The seeded IMAGE lane also captures provider/media-bound failure posture.' 'Current host/provider classification evidence includes a #CONNECT! lane on the installed baseline.' 'W23-HOST-CLASS-BL-20260321')
                )
                admitted_slice_note = 'The first widened tranche uses IMAGE as the rich-value and provider-aware publication representative, with witness payloads pointing back to the retained value-model packet rather than flattening the row to ordinary scalar behavior.'
                current_support_basis = 'Supported in the parked baseline through the retained HYPERLINK/IMAGE value-model packet, runtime image implementation, and the HostServiceSurface formal bridge.'
                provenance_refs = @(
                    (New-ProvenanceRef 'catalog_export_row' 'docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv#FUNC.IMAGE' 'catalog_export'),
                    (New-ProvenanceRef 'contract_doc' 'docs/function-lane/FUNCTION_SLICE_HYPERLINK_IMAGE_VALUE_MODEL_PRELIM.md' 'contract_artifact'),
                    (New-ProvenanceRef 'evidence_id' 'W23-HI-VALMODEL-20260321' 'native_excel_replay'),
                    (New-ProvenanceRef 'runtime_source' 'crates/oxfunc_core/src/functions/image_fn.rs' 'runtime_test'),
                    (New-ProvenanceRef 'formal_artifact' 'formal/lean/OxFunc/Functions/HostServiceSurface.lean' 'formal_artifact')
                )
            }
        }
        'FUNC.HYPERLINK' {
            return [ordered]@{
                signature_display = 'HYPERLINK(link_location, [friendly_name])'
                arg_specs = @(
                    (New-ArgSpec 1 'link_location' $true 'text' 'Target URL or workbook-local hyperlink location.'),
                    (New-ArgSpec 2 'friendly_name' $false 'value' 'Optional display value carried alongside hyperlink formatting hints.')
                )
                help_summary = 'Creates a hyperlink and returns the displayed text or value for the linked cell.'
                help_detail = 'The first widened tranche uses HYPERLINK as the presentation-aware value representative.'
                semantic_modes = @('presentation_hinting', 'text_value_projection', 'host_formatting_hint')
                witness_examples = @(
                    (New-Example 'W23-HYPERLINK-001' 'HYPERLINK returns an ordinary value with hyperlink-style formatting hints.' 'Current baseline evidence pins the value boundary as plain text while preserving presentation hints.' 'W23-HI-VALMODEL-20260321'),
                    (New-Example 'W23-HYPERLINK-002' 'Friendly-name lanes stay in the same presentation-aware value family.' 'The retained host/provider classification packet keeps HYPERLINK in the admitted presentation-hinting slice.' 'W23-HOST-CLASS-BL-20260321')
                )
                admitted_slice_note = 'The first widened tranche uses HYPERLINK as the presentation-aware value representative and preserves the distinction between plain value projection and host formatting hints.'
                current_support_basis = 'Supported in the parked baseline through the retained HYPERLINK/IMAGE value-model packet, runtime implementation, and HostServiceSurface formal bridge.'
                provenance_refs = @(
                    (New-ProvenanceRef 'catalog_export_row' 'docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv#FUNC.HYPERLINK' 'catalog_export'),
                    (New-ProvenanceRef 'contract_doc' 'docs/function-lane/FUNCTION_SLICE_HYPERLINK_IMAGE_VALUE_MODEL_PRELIM.md' 'contract_artifact'),
                    (New-ProvenanceRef 'evidence_id' 'W23-HI-VALMODEL-20260321' 'native_excel_replay'),
                    (New-ProvenanceRef 'runtime_source' 'crates/oxfunc_core/src/functions/hyperlink_fn.rs' 'runtime_test'),
                    (New-ProvenanceRef 'formal_artifact' 'formal/lean/OxFunc/Functions/HostServiceSurface.lean' 'formal_artifact')
                )
            }
        }
        'FUNC.GROUPBY' {
            return [ordered]@{
                signature_display = 'GROUPBY(row_fields, values, function, [field_headers], [total_depth], [sort_order], [filter_array], [field_relationship])'
                arg_specs = @(
                    (New-ArgSpec 1 'row_fields' $true 'array_or_reference' 'Grouping keys used to partition the input rows.'),
                    (New-ArgSpec 2 'values' $true 'array_or_reference' 'Value vector or matrix aggregated within each group.'),
                    (New-ArgSpec 3 'function' $true 'callable_or_aggregate' 'Current parked baseline supports the admitted grouped-aggregation callable/aggregate slice.')
                )
                help_summary = 'Groups rows by key fields and returns aggregated results.'
                help_detail = 'The first widened tranche uses GROUPBY as the grouped-aggregation representative for witness rollout.'
                semantic_modes = @('grouped_aggregation', 'header_total_rendering', 'filter_sensitive_sorting')
                witness_examples = @(
                    (New-Example 'W56-GROUPBY-001' 'Native Excel GROUPBY baseline on the admitted sum-backed slice.' 'Current baseline evidence pins grouped aggregation, header rendering, and totals behavior.' 'W56-GROUPED-AGGREGATION-NATIVE-FORMAL-BL-20260331'),
                    (New-Example 'W56-GROUPBY-002' 'The widened witness tranche preserves the OxFml adapter-backed grouped callable lane.' 'Current grouped-aggregation promotion evidence keeps the admitted callable carriage in scope.' 'W55-GROUPED-AGGREGATION-PROMOTION-20260331')
                )
                admitted_slice_note = 'The first widened tranche uses GROUPBY as the grouped-aggregation representative for the admitted current-baseline sum-backed slice, including headers, totals, and filter-sensitive sorting.'
                current_support_basis = 'Supported in the parked baseline through retained grouped-aggregation promotion/native-baseline packets, runtime implementation, and executable Lean grouped-aggregation substrate.'
                provenance_refs = @(
                    (New-ProvenanceRef 'catalog_export_row' 'docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv#FUNC.GROUPBY' 'catalog_export'),
                    (New-ProvenanceRef 'contract_doc' 'docs/function-lane/FUNCTION_SLICE_GROUPBY_CONTRACT_PRELIM.md' 'contract_artifact'),
                    (New-ProvenanceRef 'evidence_id' 'W56-GROUPED-AGGREGATION-NATIVE-FORMAL-BL-20260331' 'native_excel_replay'),
                    (New-ProvenanceRef 'runtime_source' 'crates/oxfunc_core/src/functions/groupby_fn.rs' 'runtime_test'),
                    (New-ProvenanceRef 'formal_artifact' 'formal/lean/OxFunc/Functions/GroupBy.lean' 'formal_artifact')
                )
            }
        }
        'FUNC.OP_IMPLICIT_INTERSECTION' {
            return [ordered]@{
                signature_display = '@ reference_or_array'
                arg_specs = @(
                    (New-ArgSpec 1 'reference_or_array' $true 'reference_or_array' 'The operator projects an implicit-intersection scalar from the current context.')
                )
                help_summary = 'Projects the implicit-intersection result for a reference or array in the current formula context.'
                help_detail = 'The first widened tranche uses OP_IMPLICIT_INTERSECTION as the modeled operator representative.'
                semantic_modes = @('implicit_intersection_projection', 'legacy_single_compatibility', 'context_sensitive_scalarization')
                witness_examples = @(
                    (New-Example 'W14-AT-001' 'Current-baseline @ operator projection lane.' 'The retained baseline pins worksheet-visible implicit-intersection behavior.' 'W14-IMPLICIT-INTERSECTION-BL-20260401'),
                    (New-Example 'W14-SINGLE-001' 'Legacy _xlfn.SINGLE compatibility normalizes to the same semantic surface.' 'Current baseline evidence preserves SINGLE as compatibility syntax rather than a second runtime operator.' 'W14-IMPLICIT-INTERSECTION-BL-20260401')
                )
                admitted_slice_note = 'The first widened tranche uses OP_IMPLICIT_INTERSECTION as the modeled operator representative, including the parked legacy SINGLE compatibility reading.'
                current_support_basis = 'Supported in the parked baseline through the retained implicit-intersection investigation, runtime operator implementation, and Lean implicit-intersection substrate.'
                provenance_refs = @(
                    (New-ProvenanceRef 'catalog_export_row' 'docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv#FUNC.OP_IMPLICIT_INTERSECTION' 'catalog_export'),
                    (New-ProvenanceRef 'contract_doc' 'docs/function-lane/FUNCTION_SLICE_OP_IMPLICIT_INTERSECTION_CONTRACT_PRELIM.md' 'contract_artifact'),
                    (New-ProvenanceRef 'evidence_id' 'W14-IMPLICIT-INTERSECTION-BL-20260401' 'native_excel_replay'),
                    (New-ProvenanceRef 'runtime_source' 'crates/oxfunc_core/src/functions/op_implicit_intersection.rs' 'runtime_test'),
                    (New-ProvenanceRef 'formal_artifact' 'formal/lean/OxFunc/Functions/ImplicitIntersection.lean' 'formal_artifact')
                )
            }
        }
        default {
            throw "Unsupported mixed tranche member: $surfaceId"
        }
    }
}

function New-WitnessEntry($row) {
    $template = Get-SurfaceTemplate $row.surface_stable_id

    [ordered]@{
        witness_schema_version = 'v0.seed'
        surface_stable_id = $row.surface_stable_id
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
        current_support_basis = $template.current_support_basis
        provenance_refs = $template.provenance_refs
        snapshot_generation = $row.snapshot_generation
        source_commit_short = $row.source_commit_short
        source_commit_full = $row.source_commit_full
        source_tree_state = $row.source_tree_state
    }
}

$surfaceIds = @(
    'FUNC.GROUPBY',
    'FUNC.HYPERLINK',
    'FUNC.IF',
    'FUNC.IMAGE',
    'FUNC.LET',
    'FUNC.OP_IMPLICIT_INTERSECTION',
    'FUNC.SUM',
    'FUNC.XLOOKUP'
)

$rows = Import-Csv $snapshotPath | Where-Object { $_.surface_stable_id -in $surfaceIds } | Sort-Object surface_stable_id

if ($rows.Count -ne $surfaceIds.Count) {
    throw "Expected exactly $($surfaceIds.Count) V1 rows for mixed tranche A; found $($rows.Count)."
}

$document = [ordered]@{
    witness_snapshot_id = 'oxfunc-semantic-witness-v2-seed-mixed-tranche-a'
    witness_schema_version = 'v0.seed'
    source_snapshot_ref = 'docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv'
    seed_family = 'mixed_tranche_a'
    entries = @($rows | ForEach-Object { New-WitnessEntry $_ })
}

$json = $document | ConvertTo-Json -Depth 8
[System.IO.File]::WriteAllText($outputPath, $json + [Environment]::NewLine, (New-Object System.Text.UTF8Encoding($false)))

Write-Host "Generated $outputPath from $snapshotPath"

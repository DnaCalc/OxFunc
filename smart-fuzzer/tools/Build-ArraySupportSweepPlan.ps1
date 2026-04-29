[CmdletBinding()]
param(
    [string] $RepoRoot,
    [string] $InventoryPath,
    [string] $OutputDir,
    [switch] $RefreshInventory
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($RepoRoot)) {
    $scriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
    $RepoRoot = (Resolve-Path (Join-Path $scriptRoot "..\..")).Path
}
$RepoRoot = [System.IO.Path]::GetFullPath($RepoRoot)
if (-not $OutputDir) {
    $OutputDir = Join-Path $RepoRoot "smart-fuzzer\cache"
}
$OutputDir = [System.IO.Path]::GetFullPath($OutputDir)
if (-not $InventoryPath) {
    $InventoryPath = Join-Path $OutputDir "dimension-inventory-v0.json"
}
$InventoryPath = [System.IO.Path]::GetFullPath($InventoryPath)

function Join-RepoPath {
    param([string] $RelativePath)
    return Join-Path $RepoRoot $RelativePath
}

function Get-Array {
    param([object] $Value)

    if ($null -eq $Value) {
        return @()
    }
    if ($Value -is [System.Array]) {
        return @($Value)
    }
    return @($Value)
}

function New-StringSet {
    return ,[System.Collections.Generic.HashSet[string]]::new(
        [System.StringComparer]::OrdinalIgnoreCase
    )
}

function Add-Tag {
    param(
        [System.Collections.Generic.HashSet[string]] $Tags,
        [string] $Tag
    )

    if (-not [string]::IsNullOrWhiteSpace($Tag)) {
        [void] $Tags.Add($Tag)
    }
}

function Add-ManyTags {
    param(
        [System.Collections.Generic.HashSet[string]] $Tags,
        [object] $Values
    )

    foreach ($value in (Get-Array $Values)) {
        Add-Tag $Tags ([string] $value)
    }
}

function Get-Tags {
    param([System.Collections.Generic.HashSet[string]] $Tags)
    return @($Tags | Sort-Object)
}

function Add-Count {
    param(
        [hashtable] $Map,
        [string] $Key,
        [int] $By = 1
    )

    if ([string]::IsNullOrWhiteSpace($Key)) {
        $Key = "(blank)"
    }
    if (-not $Map.ContainsKey($Key)) {
        $Map[$Key] = 0
    }
    $Map[$Key] = [int] $Map[$Key] + $By
}

function Sort-CountMap {
    param([hashtable] $Map)

    $ordered = [ordered]@{}
    foreach ($key in ($Map.Keys | Sort-Object)) {
        $ordered[$key] = $Map[$key]
    }
    return $ordered
}

function Get-GitValue {
    param([string[]] $GitArgs)

    try {
        $value = (& git @GitArgs 2>$null)
        if ($LASTEXITCODE -ne 0) {
            return $null
        }
        return (($value -join "`n").Trim())
    }
    catch {
        return $null
    }
}

function Write-JsonArtifact {
    param(
        [string] $Path,
        [object] $Value
    )

    $dir = Split-Path -Parent $Path
    New-Item -ItemType Directory -Force -Path $dir | Out-Null
    $Value | ConvertTo-Json -Depth 32 | Set-Content -LiteralPath $Path -Encoding UTF8
    Write-Host "Wrote $Path"
}

function Get-SourceSignals {
    param([object] $SourceFiles)

    $signals = New-StringSet
    $counts = [ordered]@{
        source_file_count = 0
        coerce_prepared_to_number = 0
        coerce_prepared_to_text = 0
        run_values_only_prepared = 0
        prepare_args_values_only = 0
        eval_binary_numeric_surface = 0
        apply_unary_numeric_array_map_prepared = 0
        broadcast_prepared_pairs = 0
        explicit_eval_array_mentions = 0
        unsupported_array_mentions = 0
    }

    foreach ($relative in (Get-Array $SourceFiles)) {
        if ([string]::IsNullOrWhiteSpace([string] $relative)) {
            continue
        }
        $path = Join-RepoPath ([string] $relative)
        if (-not (Test-Path -LiteralPath $path)) {
            continue
        }
        $counts.source_file_count++
        $text = Get-Content -LiteralPath $path -Raw

        $numberCoerce = [regex]::Matches($text, "coerce_prepared_to_number").Count
        $textCoerce = [regex]::Matches($text, "coerce_prepared_to_text").Count
        $runValues = [regex]::Matches($text, "run_values_only_prepared").Count
        $prepareValues = [regex]::Matches($text, "prepare_args_values_only").Count
        $binarySurface = [regex]::Matches($text, "eval_binary_numeric_surface").Count
        $unaryMap = [regex]::Matches($text, "apply_unary_numeric_array_map_prepared").Count
        $broadcast = [regex]::Matches($text, "broadcast_prepared_pairs").Count
        $arrayMentions = [regex]::Matches($text, "EvalValue::Array|ArrayCellValue|EvalArray").Count
        $unsupportedArray = [regex]::Matches($text, "UnsupportedValueKind\(`"array`"\)|UnsupportedValueKind\(""array""\)|UnsupportedValueKind\('array'\)").Count

        $counts.coerce_prepared_to_number += $numberCoerce
        $counts.coerce_prepared_to_text += $textCoerce
        $counts.run_values_only_prepared += $runValues
        $counts.prepare_args_values_only += $prepareValues
        $counts.eval_binary_numeric_surface += $binarySurface
        $counts.apply_unary_numeric_array_map_prepared += $unaryMap
        $counts.broadcast_prepared_pairs += $broadcast
        $counts.explicit_eval_array_mentions += $arrayMentions
        $counts.unsupported_array_mentions += $unsupportedArray

        if ($numberCoerce -gt 0) { Add-Tag $signals "source_scalar_number_coercion" }
        if ($textCoerce -gt 0) { Add-Tag $signals "source_scalar_text_coercion" }
        if ($runValues -gt 0 -or $prepareValues -gt 0) { Add-Tag $signals "source_values_only_prepared" }
        if ($binarySurface -gt 0) { Add-Tag $signals "source_binary_numeric_surface" }
        if ($unaryMap -gt 0) { Add-Tag $signals "source_unary_array_map" }
        if ($broadcast -gt 0) { Add-Tag $signals "source_broadcast_helper" }
        if ($arrayMentions -gt 0) { Add-Tag $signals "source_mentions_arrays" }
        if ($unsupportedArray -gt 0) { Add-Tag $signals "source_rejects_array_explicitly" }
    }

    return [ordered]@{
        tags = @(Get-Tags $signals)
        counts = $counts
    }
}

function Test-BlockedForArraySweep {
    param([pscustomobject] $Surface)

    $blocked = @(Get-Array $Surface.blocked_or_deferred_lanes)
    return (
        $blocked -contains "external_provider_context_required" -or
        $blocked -contains "cube_provider_context_required" -or
        $blocked -contains "special_interface_context_required" -or
        $blocked -contains "current_version_deferred_inventory"
    )
}

function Get-RiskRecord {
    param(
        [pscustomobject] $Surface,
        [hashtable] $SeededDoneByName,
        [hashtable] $FirstTrancheByName
    )

    $name = ([string] $Surface.canonical_surface_name).ToUpperInvariant()
    $category = [string] $Surface.category
    $functionSurface = $Surface.function_surface
    $refs = $Surface.refs
    $sourceSignals = Get-SourceSignals $refs.source_files
    $sourceTags = @(Get-Array $sourceSignals.tags)
    $risk = New-StringSet
    $score = 0

    if ($SeededDoneByName.ContainsKey($name)) {
        Add-Tag $risk "array_seed_already_reconciled"
        $score -= 60
    }

    if ($FirstTrancheByName.ContainsKey($name)) {
        Add-Tag $risk "first_tranche_selected"
        $score += 120
    }

    if ($functionSurface.special_interface_kind -eq "ordinary" -and
        $functionSurface.runtime_boundary_kind -eq "ordinary_eval") {
        Add-Tag $risk "ordinary_eval_surface"
        $score += 10
    }

    if ($functionSurface.arg_preparation_profile -eq "ValuesOnlyPreAdapter") {
        Add-Tag $risk "values_only_prepared_surface"
        $score += 10
    }

    if ($functionSurface.coercion_lift_profile -eq "Custom") {
        Add-Tag $risk "metadata_custom_lift_profile"
        $score += 18
    }
    elseif ([string] $functionSurface.coercion_lift_profile -match "ScalarOnly") {
        Add-Tag $risk "metadata_scalar_only_lift_profile"
        $score += 16
    }
    elseif ([string] $functionSurface.coercion_lift_profile -match "ArrayElementwise|Lookup|Aggregate") {
        Add-Tag $risk "metadata_declares_some_array_semantics"
        $score += 6
    }

    if ($sourceTags -contains "source_scalar_number_coercion" -or
        $sourceTags -contains "source_scalar_text_coercion") {
        Add-Tag $risk "source_scalar_coercion_site"
        $score += 20
    }

    if (($sourceTags -contains "source_values_only_prepared") -and
        -not ($sourceTags -contains "source_unary_array_map") -and
        -not ($sourceTags -contains "source_broadcast_helper") -and
        -not ($sourceTags -contains "source_binary_numeric_surface")) {
        Add-Tag $risk "source_scalar_prepared_without_obvious_lift_helper"
        $score += 35
    }

    if ($sourceTags -contains "source_rejects_array_explicitly") {
        Add-Tag $risk "source_explicit_array_reject"
        $score += 30
    }

    if ($category -match "Math|Statistical|Date|Financial|Logical|Information|Engineering") {
        Add-Tag $risk "non_text_family_candidate"
        $score += 10
    }

    if ($category -match "Text") {
        Add-Tag $risk "text_family_seeded_or_deferred_after_w080"
        $score -= 30
    }

    if ($category -match "Lookup and reference") {
        Add-Tag $risk "lookup_reference_family_has_existing_seed_context"
        $score -= 8
    }

    if (Test-BlockedForArraySweep $Surface) {
        Add-Tag $risk "blocked_or_deferred_for_array_sweep"
        $score -= 100
    }

    if (@(Get-Array $Surface.known_deviation_tags).Count -gt 0) {
        Add-Tag $risk "known_deviation_or_fresh_confirmation_lane"
        $score -= 40
    }

    $riskBand = "low"
    if ($score -ge 95) {
        $riskBand = "first_tranche"
    }
    elseif ($score -ge 65) {
        $riskBand = "high"
    }
    elseif ($score -ge 35) {
        $riskBand = "medium"
    }
    elseif ($score -lt 0) {
        $riskBand = "deferred_or_seeded"
    }

    return [ordered]@{
        surface_id = $Surface.surface_id
        canonical_surface_name = $Surface.canonical_surface_name
        category = $Surface.category
        arity = $Surface.arity
        function_surface = [ordered]@{
            arg_preparation_profile = $functionSurface.arg_preparation_profile
            coercion_lift_profile = $functionSurface.coercion_lift_profile
            kernel_signature_class = $functionSurface.kernel_signature_class
            special_interface_kind = $functionSurface.special_interface_kind
            runtime_boundary_kind = $functionSurface.runtime_boundary_kind
        }
        risk_band = $riskBand
        priority_score = $score
        risk_reasons = @(Get-Tags $risk)
        source_signals = $sourceSignals
        known_deviation_tags = @(Get-Array $Surface.known_deviation_tags)
        blocked_or_deferred_lanes = @(Get-Array $Surface.blocked_or_deferred_lanes)
        refs = [ordered]@{
            source_files = @(Get-Array $refs.source_files)
            bug_streams = @(Get-Array $refs.bug_streams)
            scenario_manifests = @(Get-Array $refs.scenario_manifests)
        }
    }
}

function New-FormulaSeed {
    param(
        [string] $CaseTag,
        [string] $Formula,
        [string] $Axis,
        [string] $ExpectedProbeClass
    )

    return [ordered]@{
        case_tag = $CaseTag
        formula = $Formula
        axis = $Axis
        expected_probe_class = $ExpectedProbeClass
    }
}

function Get-FirstTrancheFormulaSeeds {
    param([string] $Name)

    switch ($Name.ToUpperInvariant()) {
        "ROUND" {
            return @(
                (New-FormulaSeed "array_number" "=ROUND({1.234,2.345},1)" "one_array_arg:number" "shape_preserving_or_value_error"),
                (New-FormulaSeed "array_digits" "=ROUND(1.234,{0,1})" "one_array_arg:num_digits" "shape_preserving_or_value_error"),
                (New-FormulaSeed "array_both" "=ROUND({1.234;2.345},{0;1})" "two_array_args:same_shape" "shape_preserving_or_value_error")
            )
        }
        "ROUNDDOWN" {
            return @(
                (New-FormulaSeed "array_number" "=ROUNDDOWN({1.234,2.345},1)" "one_array_arg:number" "shape_preserving_or_value_error"),
                (New-FormulaSeed "array_digits" "=ROUNDDOWN(1.234,{0,1})" "one_array_arg:num_digits" "shape_preserving_or_value_error")
            )
        }
        "ROUNDUP" {
            return @(
                (New-FormulaSeed "array_number" "=ROUNDUP({1.234,2.345},1)" "one_array_arg:number" "shape_preserving_or_value_error"),
                (New-FormulaSeed "array_digits" "=ROUNDUP(1.234,{0,1})" "one_array_arg:num_digits" "shape_preserving_or_value_error")
            )
        }
        "TRUNC" {
            return @(
                (New-FormulaSeed "array_number" "=TRUNC({1.234,-2.345},1)" "one_array_arg:number" "shape_preserving_or_value_error"),
                (New-FormulaSeed "array_digits" "=TRUNC(1.234,{0,1})" "one_array_arg:num_digits" "shape_preserving_or_value_error"),
                (New-FormulaSeed "omitted_digits_array_number" "=TRUNC({1.234;2.345})" "array_with_omitted_optional" "shape_preserving_or_value_error")
            )
        }
        "CEILING" {
            return @(
                (New-FormulaSeed "array_number" "=CEILING({1.2,2.3},1)" "one_array_arg:number" "shape_preserving_or_value_error"),
                (New-FormulaSeed "array_significance" "=CEILING(1.2,{0.5,1})" "one_array_arg:significance" "shape_preserving_or_value_error")
            )
        }
        "FLOOR" {
            return @(
                (New-FormulaSeed "array_number" "=FLOOR({1.2,2.3},1)" "one_array_arg:number" "shape_preserving_or_value_error"),
                (New-FormulaSeed "array_significance" "=FLOOR(1.2,{0.5,1})" "one_array_arg:significance" "shape_preserving_or_value_error")
            )
        }
        "CEILING.MATH" {
            return @(
                (New-FormulaSeed "array_number_omitted_optionals" "=CEILING.MATH({-1.2,1.2})" "array_with_omitted_optional" "shape_preserving_or_value_error"),
                (New-FormulaSeed "array_significance" "=CEILING.MATH(-1.2,{1,2})" "one_array_arg:significance" "shape_preserving_or_value_error"),
                (New-FormulaSeed "array_mode" "=CEILING.MATH(-1.2,1,{0,1})" "one_array_arg:mode" "shape_preserving_or_value_error")
            )
        }
        "FLOOR.MATH" {
            return @(
                (New-FormulaSeed "array_number_omitted_optionals" "=FLOOR.MATH({-1.2,1.2})" "array_with_omitted_optional" "shape_preserving_or_value_error"),
                (New-FormulaSeed "array_significance" "=FLOOR.MATH(-1.2,{1,2})" "one_array_arg:significance" "shape_preserving_or_value_error"),
                (New-FormulaSeed "array_mode" "=FLOOR.MATH(-1.2,1,{0,1})" "one_array_arg:mode" "shape_preserving_or_value_error")
            )
        }
        "CEILING.PRECISE" {
            return @(
                (New-FormulaSeed "array_number_omitted_significance" "=CEILING.PRECISE({-1.2,1.2})" "array_with_omitted_optional" "shape_preserving_or_value_error"),
                (New-FormulaSeed "array_significance" "=CEILING.PRECISE(-1.2,{1,2})" "one_array_arg:significance" "shape_preserving_or_value_error")
            )
        }
        "FLOOR.PRECISE" {
            return @(
                (New-FormulaSeed "array_number_omitted_significance" "=FLOOR.PRECISE({-1.2,1.2})" "array_with_omitted_optional" "shape_preserving_or_value_error"),
                (New-FormulaSeed "array_significance" "=FLOOR.PRECISE(-1.2,{1,2})" "one_array_arg:significance" "shape_preserving_or_value_error")
            )
        }
        "ISO.CEILING" {
            return @(
                (New-FormulaSeed "array_number_omitted_significance" "=ISO.CEILING({-1.2,1.2})" "array_with_omitted_optional" "shape_preserving_or_value_error"),
                (New-FormulaSeed "array_significance" "=ISO.CEILING(-1.2,{1,2})" "one_array_arg:significance" "shape_preserving_or_value_error")
            )
        }
        "ATAN2" {
            return @(
                (New-FormulaSeed "array_x" "=ATAN2({1,0},1)" "one_array_arg:x_num" "shape_preserving_or_value_error"),
                (New-FormulaSeed "array_y" "=ATAN2(1,{0,1})" "one_array_arg:y_num" "shape_preserving_or_value_error"),
                (New-FormulaSeed "zero_vector_error_cells" "=ATAN2({0,1},{0,1})" "two_array_args:error_mix" "shape_preserving_or_value_error")
            )
        }
        "BASE" {
            return @(
                (New-FormulaSeed "array_number" "=BASE({15,16},16)" "one_array_arg:number" "shape_preserving_or_value_error"),
                (New-FormulaSeed "array_radix" "=BASE(15,{2,16})" "one_array_arg:radix" "shape_preserving_or_value_error"),
                (New-FormulaSeed "array_min_length" "=BASE(15,16,{2,4})" "one_array_arg:min_length" "shape_preserving_or_value_error")
            )
        }
        "MROUND" {
            return @(
                (New-FormulaSeed "array_number" "=MROUND({1.3,2.7},0.5)" "one_array_arg:number" "shape_preserving_or_value_error"),
                (New-FormulaSeed "array_multiple" "=MROUND(1.3,{0.5,1})" "one_array_arg:multiple" "shape_preserving_or_value_error")
            )
        }
        default { return @() }
    }
}

$dimensionBuilder = Join-Path $RepoRoot "smart-fuzzer\tools\Build-DimensionInventory.ps1"
if ($RefreshInventory -or -not (Test-Path -LiteralPath $InventoryPath)) {
    & powershell -ExecutionPolicy Bypass -File $dimensionBuilder -RepoRoot $RepoRoot -OutputPath $InventoryPath
}

$inventory = Get-Content -LiteralPath $InventoryPath -Raw | ConvertFrom-Json
$surfaces = @(Get-Array $inventory.surfaces)

$seededDoneNames = @(
    "MATCH", "XMATCH", "VLOOKUP", "HLOOKUP", "XLOOKUP",
    "LEFT", "LEFTB", "MID", "MIDB", "RIGHT", "RIGHTB",
    "CHAR", "CODE", "LOWER", "UPPER", "TRIM", "REPT", "TEXTAFTER", "TEXTBEFORE",
    "FIND", "FINDB", "SEARCH", "SEARCHB", "REPLACE", "REPLACEB", "PROPER", "SUBSTITUTE"
)
$seededDoneByName = @{}
foreach ($name in $seededDoneNames) {
    $seededDoneByName[$name] = $true
}

$firstTrancheNames = @(
    "ROUND",
    "ROUNDDOWN",
    "ROUNDUP",
    "TRUNC",
    "CEILING",
    "CEILING.MATH",
    "CEILING.PRECISE",
    "FLOOR",
    "FLOOR.MATH",
    "FLOOR.PRECISE",
    "ISO.CEILING",
    "ATAN2",
    "BASE",
    "MROUND"
)
$firstTrancheByName = @{}
foreach ($name in $firstTrancheNames) {
    $firstTrancheByName[$name] = $true
}

$candidateRows = @()
$byCategory = @{}
$byRiskBand = @{}
$byRiskReason = @{}
foreach ($surface in $surfaces) {
    $record = Get-RiskRecord -Surface $surface -SeededDoneByName $seededDoneByName -FirstTrancheByName $firstTrancheByName
    $candidateRows += $record
    Add-Count $byCategory ([string] $record.category)
    Add-Count $byRiskBand ([string] $record.risk_band)
    foreach ($reason in (Get-Array $record.risk_reasons)) {
        Add-Count $byRiskReason ([string] $reason)
    }
}

$sortedCandidates = @(
    $candidateRows |
        Sort-Object `
            @{ Expression = { - [int] $_.priority_score } },
            @{ Expression = { [string] $_.category } },
            @{ Expression = { [string] $_.canonical_surface_name } }
)

$firstTrancheRows = @()
foreach ($name in $firstTrancheNames) {
    $row = $candidateRows | Where-Object { ([string] $_.canonical_surface_name).ToUpperInvariant() -eq $name } | Select-Object -First 1
    if ($null -eq $row) {
        continue
    }

    $firstTrancheRows += [ordered]@{
        surface_id = $row.surface_id
        canonical_surface_name = $row.canonical_surface_name
        category = $row.category
        priority_score = $row.priority_score
        risk_reasons = $row.risk_reasons
        source_signals = $row.source_signals.tags
        formula_seeds = @(Get-FirstTrancheFormulaSeeds ([string] $row.canonical_surface_name))
    }
}

$replayMatrixRows = @()
foreach ($row in $firstTrancheRows) {
    $formulaSeeds = @(Get-Array $row.formula_seeds)
    $replayMatrixRows += [ordered]@{
        surface_id = $row.surface_id
        canonical_surface_name = $row.canonical_surface_name
        matrix_axes = @(
            "scalar_control",
            "single_array_argument_each_scalar_position",
            "same_shape_multi_array_when_formula_seed_exists",
            "omitted_optional_with_array_required_arg_when_applicable",
            "array_contains_error_cell",
            "array_contains_blank_or_empty_cell",
            "reference_area_vs_inline_array_contrast",
            "shape_mismatch_or_broadcast_probe"
        )
        excel_formula_seed_count = $formulaSeeds.Count
        excel_formula_seeds = $formulaSeeds
        telemetry_keys = @(
            "function",
            "arg_position",
            "array_shape",
            "array_cell_band",
            "optional_state",
            "reference_vs_inline",
            "local_outcome_class",
            "excel_outcome_class",
            "comparison_class"
        )
    }
}

$candidateInventory = [ordered]@{
    schema_version = "oxfunc.smart_fuzzer.array_support_candidate_inventory.v0"
    authority = "derived_w090_planning_artifact_not_semantic_truth"
    generated_utc = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
    git_revision = Get-GitValue @("rev-parse", "HEAD")
    git_status_short_digest_source = Get-GitValue @("status", "--short")
    inputs = [ordered]@{
        dimension_inventory = $InventoryPath
        source_tree = "crates/oxfunc_core/src/functions"
        w080_seed_owner = "docs/worksets/W080_FUNCTION_ARRAY_SUPPORT_REVIEW.md"
        w090_workset = "docs/worksets/W090_FUNCTION_ARRAY_SUPPORT_SYSTEMATIC_SWEEP.md"
    }
    summary = [ordered]@{
        surface_count = $candidateRows.Count
        candidate_rows_above_medium = @($candidateRows | Where-Object { [int] $_.priority_score -ge 35 }).Count
        first_tranche_surface_count = $firstTrancheRows.Count
        seeded_or_prior_reconciled_surface_count = @($candidateRows | Where-Object { @(Get-Array $_.risk_reasons) -contains "array_seed_already_reconciled" }).Count
        by_category = Sort-CountMap $byCategory
        by_risk_band = Sort-CountMap $byRiskBand
        by_risk_reason = Sort-CountMap $byRiskReason
    }
    rows = $sortedCandidates
}

$firstTranche = [ordered]@{
    schema_version = "oxfunc.smart_fuzzer.array_support_first_tranche.v0"
    authority = "derived_w090_planning_artifact_not_run_evidence"
    generated_utc = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
    git_revision = Get-GitValue @("rev-parse", "HEAD")
    tranche_id = "w090-tranche-a-math-scalar-numeric-array-lift"
    execution_state = "planned_not_run"
    selection_rationale = @(
        "non_text_successor_after_w080",
        "ordinary_values_only_surfaces",
        "source_scalar_coercion_without_obvious_array_lift_helper",
        "bounded_family_size",
        "high_expected_excel_spill_signal_from_adjacent_scalar_function_behavior"
    )
    non_goals = @(
        "full_math_function_surface_claim",
        "text_family_replay",
        "lookup_family_replay",
        "POWER_bug_assumption_without_fresh_confirmation",
        "repair_inside_planning_bead"
    )
    surfaces = $firstTrancheRows
}

$replayMatrix = [ordered]@{
    schema_version = "oxfunc.smart_fuzzer.array_support_replay_matrix.v0"
    authority = "derived_w090_planning_artifact_not_run_evidence"
    generated_utc = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
    git_revision = Get-GitValue @("rev-parse", "HEAD")
    comparison_policy = [ordered]@{
        pass_class = "exact_typed_bit_match"
        tolerance_pass_allowed = $false
        pass_rows_are_coverage_telemetry_only = $true
        failure_rows_promote_to_bug_stream = $true
    }
    rows = $replayMatrixRows
}

$highlightsLines = @(
    "# W090 Array Support Sweep Generated Highlights",
    "",
    "Generated: $($candidateInventory.generated_utc)",
    "Git revision: $($candidateInventory.git_revision)",
    "",
    "## Candidate Inventory",
    "- surfaces: $($candidateInventory.summary.surface_count)",
    "- medium-or-higher candidates: $($candidateInventory.summary.candidate_rows_above_medium)",
    "- seeded/prior reconciled surfaces: $($candidateInventory.summary.seeded_or_prior_reconciled_surface_count)",
    "- first-tranche surfaces: $($candidateInventory.summary.first_tranche_surface_count)",
    "",
    "## First Tranche",
    "- tranche id: $($firstTranche.tranche_id)",
    "- status: planned_not_run",
    "- selected surfaces: $((@($firstTrancheRows | ForEach-Object { $_.canonical_surface_name }) -join ', '))",
    "",
    "## Artifact Economy",
    "- passing rows remain coverage telemetry only",
    "- unexpected mismatches promote to BUG-FUNC streams or narrower repair beads",
    "- POWER remains a fresh-confirmation sentinel, not an assumed current bug"
)

New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null
Write-JsonArtifact (Join-Path $OutputDir "array-support-candidate-inventory-v0.json") $candidateInventory
Write-JsonArtifact (Join-Path $OutputDir "array-support-first-tranche-v0.json") $firstTranche
Write-JsonArtifact (Join-Path $OutputDir "array-support-replay-matrix-v0.json") $replayMatrix
$highlightsPath = Join-Path $OutputDir "array-support-highlights-v0.md"
$highlightsLines | Set-Content -LiteralPath $highlightsPath -Encoding UTF8
Write-Host "Wrote $highlightsPath"

Write-Host "Surfaces inventoried: $($candidateInventory.summary.surface_count)"
Write-Host "Medium-or-higher candidates: $($candidateInventory.summary.candidate_rows_above_medium)"
Write-Host "Seeded/prior reconciled surfaces: $($candidateInventory.summary.seeded_or_prior_reconciled_surface_count)"
Write-Host "First-tranche surfaces: $($candidateInventory.summary.first_tranche_surface_count)"
Write-Host "First tranche: $((@($firstTrancheRows | ForEach-Object { $_.canonical_surface_name }) -join ', '))"

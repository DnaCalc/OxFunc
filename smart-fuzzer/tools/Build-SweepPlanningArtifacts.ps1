[CmdletBinding()]
param(
    [string] $RepoRoot,
    [string] $InventoryPath,
    [string] $OutputDir
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

function Get-MutatorTags {
    param([pscustomobject] $Surface)

    $tags = New-StringSet
    Add-Tag $tags "scalar_value_kind_mutator"
    Add-Tag $tags "execution_seam_mutator"

    $arityProbes = @(Get-Array $Surface.arity.probe_tags)
    if ($arityProbes.Count -gt 0) { Add-Tag $tags "arity_edge_mutator" }
    if ($arityProbes -contains "omitted_optional_suffix" -or $arityProbes -contains "explicit_missing_optional") {
        Add-Tag $tags "optional_missing_mutator"
    }
    if ($Surface.arity.shape -eq "variadic_known_min") { Add-Tag $tags "variadic_budget_mutator" }

    if (@(Get-Array $Surface.numeric_axes.bands).Count -gt 0) {
        Add-Tag $tags "numeric_band_mutator"
        Add-Tag $tags "numeric_boundary_mutator"
    }
    if (@(Get-Array $Surface.text_axes.bands).Count -gt 0) { Add-Tag $tags "text_band_mutator" }
    if (@(Get-Array $Surface.array_axes.shape_bands).Count -gt 2) { Add-Tag $tags "array_shape_mutator" }
    if (@(Get-Array $Surface.reference_axes.reference_bands).Count -gt 1) { Add-Tag $tags "reference_fixture_mutator" }
    if (@(Get-Array $Surface.context_axes.context_bands).Count -gt 2) { Add-Tag $tags "context_fixture_mutator" }
    if (@(Get-Array $Surface.known_deviation_tags).Count -gt 0) { Add-Tag $tags "known_deviation_reference_mutator" }
    if (@(Get-Array $Surface.blocked_or_deferred_lanes).Count -gt 0) { Add-Tag $tags "blocked_lane_sentinel_mutator" }

    $category = [string] $Surface.category
    $name = [string] $Surface.canonical_surface_name
    if ($category -match "(?i)lookup|reference" -or $name -match "(?i)LOOKUP|MATCH|INDEX|OFFSET") {
        Add-Tag $tags "lookup_family_mutator"
    }
    if ($category -match "(?i)financial" -or $name -match "(?i)PMT|RATE|IRR|XIRR|NPV|FV|PV") {
        Add-Tag $tags "financial_solver_mutator"
    }
    if ($category -match "(?i)date|time") { Add-Tag $tags "date_time_mutator" }
    if ($category -match "(?i)text" -or $name -match "(?i)LEFT|RIGHT|MID|TEXT|FIND|SEARCH|REGEX") {
        Add-Tag $tags "text_search_slice_mutator"
    }
    if ($category -match "(?i)statistical" -or $name -match "(?i)NORM|BETA|GAMMA|DIST|INV") {
        Add-Tag $tags "statistical_distribution_mutator"
    }
    if ($category -match "(?i)operators" -or $name -match "^OP_") {
        Add-Tag $tags "operator_form_mutator"
    }

    return Get-Tags $tags
}

function Get-MandatoryBasisTags {
    param([pscustomobject] $Surface)

    $tags = New-StringSet
    foreach ($tag in (Get-Array $Surface.arity.probe_tags | Select-Object -First 6)) {
        Add-Tag $tags "arity:$tag"
    }
    foreach ($tag in (Get-Array $Surface.value_type_axes.universal_probes | Select-Object -First 7)) {
        Add-Tag $tags "value:$tag"
    }
    foreach ($tag in (Get-Array $Surface.value_type_axes.profiled_focus_probes | Select-Object -First 4)) {
        Add-Tag $tags "focus:$tag"
    }
    foreach ($tag in (Get-Array $Surface.numeric_axes.bands | Select-Object -First 4)) {
        Add-Tag $tags "numeric:$tag"
    }
    foreach ($tag in (Get-Array $Surface.text_axes.bands | Select-Object -First 4)) {
        Add-Tag $tags "text:$tag"
    }
    foreach ($tag in (Get-Array $Surface.array_axes.shape_bands | Select-Object -First 4)) {
        Add-Tag $tags "array:$tag"
    }
    foreach ($tag in (Get-Array $Surface.reference_axes.reference_bands | Select-Object -First 4)) {
        Add-Tag $tags "reference:$tag"
    }
    foreach ($tag in (Get-Array $Surface.context_axes.context_bands | Select-Object -First 4)) {
        Add-Tag $tags "context:$tag"
    }
    foreach ($tag in (Get-Array $Surface.execution_seams)) {
        Add-Tag $tags "seam:$tag"
    }
    Add-Tag $tags "comparison:exact_typed_bit_match"

    return Get-Tags $tags
}

function Get-LocalBudget {
    param([pscustomobject] $Surface)

    $budget = 50
    $budget += 5 * (@(Get-Array $Surface.arity.probe_tags).Count)
    $budget += 3 * (@(Get-Array $Surface.value_type_axes.universal_probes).Count)
    $budget += 12 * (@(Get-Array $Surface.value_type_axes.profiled_focus_probes).Count)
    $budget += 8 * (@(Get-Array $Surface.numeric_axes.bands).Count)
    $budget += 6 * (@(Get-Array $Surface.text_axes.bands).Count)
    $budget += 8 * (@(Get-Array $Surface.array_axes.shape_bands).Count)
    $budget += 8 * (@(Get-Array $Surface.reference_axes.reference_bands).Count)
    $budget += 4 * (@(Get-Array $Surface.context_axes.context_bands).Count)

    if ($Surface.arity.shape -eq "variadic_known_min") { $budget += 250 }
    if (@(Get-Array $Surface.known_deviation_tags).Count -gt 0) { $budget += 500 }
    if (@(Get-Array $Surface.blocked_or_deferred_lanes).Count -gt 0) { $budget = [Math]::Max(40, [Math]::Min($budget, 250)) }
    if ((Get-Array $Surface.risk_and_selection_tags) -contains "family_priority_candidate") { $budget += 100 }
    if ((Get-Array $Surface.risk_and_selection_tags) -contains "bug_stream_mentioned") { $budget += 200 }

    return [Math]::Min($budget, 2500)
}

function Get-ExcelQuota {
    param([pscustomobject] $Surface)

    $blocked = @(Get-Array $Surface.blocked_or_deferred_lanes)
    if ($blocked -contains "external_provider_context_required" -or
        $blocked -contains "cube_provider_context_required" -or
        $blocked -contains "special_interface_context_required" -or
        $blocked -contains "async_realtime_provider_deferred" -or
        $blocked -contains "formula_binding_scope_deferred") {
        return 0
    }

    $quota = 3
    if (@(Get-Array $Surface.known_deviation_tags).Count -gt 0) { $quota += 12 }
    if ((Get-Array $Surface.risk_and_selection_tags) -contains "bug_stream_mentioned") { $quota += 10 }
    if ((Get-Array $Surface.risk_and_selection_tags) -contains "family_priority_candidate") { $quota += 5 }
    if ($Surface.arity.shape -eq "unknown_arity") { $quota = [Math]::Min($quota, 2) }
    if ($blocked.Count -gt 0) { $quota = [Math]::Min($quota, 3) }

    return [Math]::Min($quota, 40)
}

function New-ArtifactEnvelope {
    param(
        [string] $SchemaVersion,
        [object] $Body
    )

    $bodyMap = [ordered]@{
        schema_version = $SchemaVersion
        authority = "derived_w089_planning_artifact_not_run_evidence"
        generated_utc = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
        git_revision = Get-GitValue @("rev-parse", "HEAD")
        git_status_short_digest_source = Get-GitValue @("status", "--short")
        inventory_ref = $InventoryPath
    }
    foreach ($property in $Body.GetEnumerator()) {
        $bodyMap[$property.Key] = $property.Value
    }
    return $bodyMap
}

function Write-JsonArtifact {
    param(
        [string] $Path,
        [object] $Value
    )

    $dir = Split-Path -Parent $Path
    New-Item -ItemType Directory -Force -Path $dir | Out-Null
    $Value | ConvertTo-Json -Depth 24 | Set-Content -LiteralPath $Path -Encoding UTF8
    Write-Host "Wrote $Path"
}

if (-not (Test-Path -LiteralPath $InventoryPath)) {
    $builder = Join-Path $RepoRoot "smart-fuzzer\tools\Build-DimensionInventory.ps1"
    & powershell -ExecutionPolicy Bypass -File $builder -RepoRoot $RepoRoot -OutputPath $InventoryPath
}

$inventory = Get-Content -LiteralPath $InventoryPath -Raw | ConvertFrom-Json
$surfaces = @(Get-Array $inventory.surfaces)

$matrixRows = @()
$localTotal = 0
$excelTotal = 0
$mutatorCounts = @{}
$categoryLocalBudget = @{}
$categoryExcelQuota = @{}
$blockedLaneCounts = @{}
$blockedRows = @()

foreach ($surface in $surfaces) {
    $mutators = @(Get-MutatorTags $surface)
    $basis = @(Get-MandatoryBasisTags $surface)
    $localBudget = Get-LocalBudget $surface
    $excelQuota = Get-ExcelQuota $surface
    $localTotal += $localBudget
    $excelTotal += $excelQuota

    foreach ($mutator in $mutators) { Add-Count $mutatorCounts $mutator }
    Add-Count $categoryLocalBudget ([string] $surface.category) $localBudget
    Add-Count $categoryExcelQuota ([string] $surface.category) $excelQuota

    $blocked = @(Get-Array $surface.blocked_or_deferred_lanes)
    foreach ($lane in $blocked) { Add-Count $blockedLaneCounts ([string] $lane) }
    if ($blocked.Count -gt 0) {
        $fixtureRequirement = if ($blocked -contains "async_realtime_provider_deferred") {
            "async_subscription_provider_fixture_required"
        } elseif ($blocked -contains "formula_binding_scope_deferred") {
            "formula_binding_or_callable_fixture_required"
        } elseif ($blocked -contains "external_provider_context_required" -or $blocked -contains "cube_provider_context_required") {
            "provider_fixture_required"
        } elseif ($blocked -contains "locale_profile_context_required") {
            "locale_profile_required"
        } elseif (($blocked -contains "volatile_context_control_required") -and ([string] $surface.canonical_surface_name) -match "^(RAND|RANDBETWEEN|RANDARRAY|NOW|TODAY)$") {
            "recalc_control_required"
        } elseif ($blocked -contains "host_interaction_context_required") {
            "workbook_or_host_context_fixture_required"
        } elseif ($blocked -contains "volatile_context_control_required") {
            "recalc_control_required"
        } else {
            "metadata_or_seam_review_required"
        }

        $blockedRows += [ordered]@{
            surface_id = $surface.surface_id
            canonical_surface_name = $surface.canonical_surface_name
            category = $surface.category
            lanes = $blocked
            fixture_requirement = $fixtureRequirement
        }
    }

    $matrixRows += [ordered]@{
        surface_id = $surface.surface_id
        canonical_surface_name = $surface.canonical_surface_name
        category = $surface.category
        arity_shape = $surface.arity.shape
        mandatory_basis_tags = $basis
        mutator_tags = $mutators
        selection_reason_tags = @(Get-Array $surface.risk_and_selection_tags)
        known_deviation_tags = @(Get-Array $surface.known_deviation_tags)
        blocked_or_deferred_lanes = $blocked
        local_case_budget = $localBudget
        excel_candidate_quota = $excelQuota
    }
}

$matrix = New-ArtifactEnvelope "oxfunc.smart_fuzzer.generator_matrix.v0" ([ordered]@{
    summary = [ordered]@{
        surface_count = @($matrixRows).Count
        total_planned_local_case_budget = $localTotal
        total_planned_excel_candidate_quota = $excelTotal
        by_mutator = Sort-CountMap $mutatorCounts
    }
    rows = $matrixRows
})

$localBudget = New-ArtifactEnvelope "oxfunc.smart_fuzzer.local_dry_run_budget.v0" ([ordered]@{
    execution_state = "planning_only"
    total_planned_local_case_budget = $localTotal
    by_category = Sort-CountMap $categoryLocalBudget
    local_outcome_classes = @(
        "local_value_scalar",
        "local_value_array",
        "local_reference_like",
        "local_worksheet_error",
        "local_bind_reject",
        "local_seam_reject",
        "local_generator_invalid",
        "local_panic_or_harness_failure",
        "local_timeout"
    )
    planned_artifacts = @(
        "manifest.json",
        "local_cases.jsonl",
        "local_outcomes.jsonl",
        "coverage_rollup.json",
        "local_invalid_cases.jsonl",
        "excel_candidate_pool.jsonl",
        "telemetry.jsonl"
    )
})

$excelBudget = New-ArtifactEnvelope "oxfunc.smart_fuzzer.excel_candidate_budget.v0" ([ordered]@{
    execution_state = "planning_only"
    total_planned_excel_candidate_quota = $excelTotal
    formula_batch_size = 5000
    workbook_chunk_target = 50000
    by_category = Sort-CountMap $categoryExcelQuota
    comparison_classes = @(
        "exact_typed_bit_match",
        "known_expected_deviation",
        "unexpected_mismatch",
        "excel_harness_blocked",
        "oxfml_seam_blocked",
        "context_provider_blocked",
        "invalid_generator_case",
        "unstable_or_non_reproducible"
    )
    statistical_profile_classes = @(
        "statistical_profile_consistent",
        "statistical_profile_mismatch",
        "statistical_profile_inconclusive"
    )
    tolerance_pass_allowed = $false
})

$blockedMap = New-ArtifactEnvelope "oxfunc.smart_fuzzer.blocked_seam_map.v0" ([ordered]@{
    execution_state = "planning_only"
    blocked_or_deferred_surface_count = @($blockedRows).Count
    by_lane = Sort-CountMap $blockedLaneCounts
    rows = $blockedRows
})

$roadmapTraceTemplate = New-ArtifactEnvelope "oxfunc.smart_fuzzer.roadmap_trace_template.v0" ([ordered]@{
    execution_state = "planning_only"
    sections = @(
        "explored_function_families",
        "arity_coverage",
        "value_kind_coverage",
        "numeric_text_bands",
        "array_reference_shape_coverage",
        "context_and_seam_coverage",
        "blocked_or_deferred_lanes",
        "known_deviations",
        "unexpected_mismatches",
        "sparse_areas_to_target_next",
        "promoted_failure_packets"
    )
    compact_pass_counters = @(
        "generated_count",
        "local_evaluated_count",
        "excel_evaluated_count",
        "exact_typed_bit_match_count"
    )
    generator_matrix_ref = Join-Path $OutputDir "generator-matrix-v0.json"
    local_budget_ref = Join-Path $OutputDir "local-dry-run-budget-v0.json"
    excel_budget_ref = Join-Path $OutputDir "excel-candidate-budget-v0.json"
    blocked_seam_map_ref = Join-Path $OutputDir "blocked-seam-map-v0.json"
})

Write-JsonArtifact (Join-Path $OutputDir "generator-matrix-v0.json") $matrix
Write-JsonArtifact (Join-Path $OutputDir "local-dry-run-budget-v0.json") $localBudget
Write-JsonArtifact (Join-Path $OutputDir "excel-candidate-budget-v0.json") $excelBudget
Write-JsonArtifact (Join-Path $OutputDir "blocked-seam-map-v0.json") $blockedMap
Write-JsonArtifact (Join-Path $OutputDir "roadmap-trace-template-v0.json") $roadmapTraceTemplate

Write-Host "Surfaces planned: $(@($matrixRows).Count)"
Write-Host "Planned local case budget: $localTotal"
Write-Host "Planned Excel candidate quota: $excelTotal"
Write-Host "Blocked/deferred surfaces: $(@($blockedRows).Count)"

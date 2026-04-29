[CmdletBinding()]
param(
    [string] $RepoRoot,
    [string] $OutputPath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($RepoRoot)) {
    $scriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
    $RepoRoot = (Resolve-Path (Join-Path $scriptRoot "..\..")).Path
}
$RepoRoot = [System.IO.Path]::GetFullPath($RepoRoot)
if (-not $OutputPath) {
    $OutputPath = Join-Path $RepoRoot "smart-fuzzer\cache\dimension-inventory-v0.json"
}
$OutputPath = [System.IO.Path]::GetFullPath($OutputPath)

function Join-RepoPath {
    param([string] $RelativePath)
    return Join-Path $RepoRoot $RelativePath
}

function Get-RepoRelativePath {
    param([string] $Path)

    $full = [System.IO.Path]::GetFullPath($Path)
    $root = [System.IO.Path]::GetFullPath($RepoRoot)
    if (-not $root.EndsWith([System.IO.Path]::DirectorySeparatorChar)) {
        $root = $root + [System.IO.Path]::DirectorySeparatorChar
    }

    if ($full.StartsWith($root, [System.StringComparison]::OrdinalIgnoreCase)) {
        return ($full.Substring($root.Length) -replace "\\", "/")
    }

    return $full
}

function Read-RequiredCsv {
    param([string] $RelativePath)

    $path = Join-RepoPath $RelativePath
    if (-not (Test-Path -LiteralPath $path)) {
        throw "Required input not found: $RelativePath"
    }

    return @(Import-Csv -LiteralPath $path)
}

function Read-OptionalCsv {
    param([string] $RelativePath)

    $path = Join-RepoPath $RelativePath
    if (-not (Test-Path -LiteralPath $path)) {
        return @()
    }

    return @(Import-Csv -LiteralPath $path)
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
        [string[]] $Values
    )

    foreach ($value in $Values) {
        Add-Tag $Tags $value
    }
}

function Get-Tags {
    param([System.Collections.Generic.HashSet[string]] $Tags)
    return @($Tags | Sort-Object)
}

function Add-MapSetValue {
    param(
        [hashtable] $Map,
        [string] $Key,
        [string] $Value
    )

    if ([string]::IsNullOrWhiteSpace($Key) -or [string]::IsNullOrWhiteSpace($Value)) {
        return
    }

    if (-not $Map.ContainsKey($Key)) {
        $Map[$Key] = New-StringSet
    }

    [void] $Map[$Key].Add($Value)
}

function Get-MapSetValues {
    param(
        [hashtable] $Map,
        [string] $Key
    )

    if (-not $Map.ContainsKey($Key)) {
        return @()
    }

    return @($Map[$Key] | Sort-Object)
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

function Test-FunctionMention {
    param(
        [string] $Text,
        [string] $PathText,
        [string] $Name
    )

    if ([string]::IsNullOrWhiteSpace($Name)) {
        return $false
    }

    $escaped = [regex]::Escape($Name)
    if ($Name.Length -le 3) {
        return ($Text -match "(?<![A-Z0-9_])$escaped\s*\(")
    }

    if ($Text -match "(?<![A-Z0-9_])$escaped(?![A-Z0-9_])") {
        return $true
    }

    $lowerName = [regex]::Escape($Name.ToLowerInvariant())
    return ($PathText -match "(?i)(^|[\\/_-])$lowerName($|[\\/_-])")
}

function Get-ArityShape {
    param(
        [string] $Min,
        [string] $Max
    )

    if ([string]::IsNullOrWhiteSpace($Min) -or [string]::IsNullOrWhiteSpace($Max)) {
        return "unknown_arity"
    }

    $minValue = [int] $Min
    $maxValue = [int] $Max
    if ($minValue -eq $maxValue) {
        return "exact"
    }
    if ($maxValue -ge 255) {
        return "variadic_known_min"
    }
    return "optional_suffix_range"
}

function Get-ArityProbeTags {
    param(
        [string] $Min,
        [string] $Max
    )

    $tags = New-StringSet
    if ([string]::IsNullOrWhiteSpace($Min) -or [string]::IsNullOrWhiteSpace($Max)) {
        Add-Tag $tags "metadata_gap_arity_probe"
        return Get-Tags $tags
    }

    $minValue = [int] $Min
    $maxValue = [int] $Max
    Add-ManyTags $tags @("argc_below_min", "argc_at_min", "argc_at_max", "argc_above_max")

    if ($maxValue -gt $minValue) {
        Add-ManyTags $tags @(
            "omitted_optional_suffix",
            "explicit_missing_optional",
            "empty_argument_optional"
        )
    }

    if ($maxValue -ge 255) {
        Add-ManyTags $tags @(
            "variadic_budget_low",
            "variadic_budget_mid",
            "variadic_budget_high"
        )
    }

    return Get-Tags $tags
}

function Test-AnyMatch {
    param(
        [string] $Text,
        [string] $Pattern
    )

    return (-not [string]::IsNullOrWhiteSpace($Text)) -and ($Text -match $Pattern)
}

function Get-ValueKindAxes {
    param([pscustomobject] $Row)

    $basis = @(
        "scalar_number",
        "scalar_text",
        "scalar_logical",
        "scalar_error",
        "blank_cell",
        "empty_cell",
        "missing_arg"
    )

    $focus = New-StringSet
    $profileText = "$($Row.arg_preparation_profile) $($Row.coercion_lift_profile) $($Row.kernel_signature_class) $($Row.category) $($Row.special_interface_kind) $($Row.runtime_boundary_kind) $($Row.fec_dependency_profile) $($Row.surface_fec_dependency_profile)"

    if ($profileText -match "(?i)array|aggregate|lookup|custom|dynamic|database|statistical|financial|math|engineering") {
        Add-ManyTags $focus @("array_literal", "array_spill")
    }
    if ($profileText -match "(?i)ref|lookup|database|aggregate|caller|formula|sheet|cell") {
        Add-ManyTags $focus @("reference_single_cell", "reference_area", "reference_multi_area")
    }
    if ($profileText -match "(?i)callable|lambda|higher_order") {
        Add-Tag $focus "callable_or_lambda"
    }
    if ($profileText -match "(?i)presentation|rich|image|hyperlink") {
        Add-Tag $focus "presentation_or_rich_value"
    }
    if ($profileText -match "(?i)provider|cube|rtd|external") {
        Add-Tag $focus "provider_context_value"
    }

    return [ordered]@{
        universal_probes = $basis
        profiled_focus_probes = Get-Tags $focus
    }
}

function Get-NumericAxes {
    param([pscustomobject] $Row)

    $profileText = "$($Row.category) $($Row.kernel_signature_class) $($Row.coercion_lift_profile) $($Row.canonical_surface_name)"
    $focus = Test-AnyMatch $profileText "(?i)num|math|trigonometry|financial|statistical|engineering|date|time|bit|rate|power|log|distribution|solver"
    if (-not $focus) {
        return [ordered]@{
            enabled = $false
            bands = @()
        }
    }

    return [ordered]@{
        enabled = $true
        bands = @(
            "positive_zero",
            "negative_zero",
            "tiny_magnitude",
            "small_integer",
            "large_integer",
            "two_to_53_adjacent",
            "fraction_near_integer",
            "half_fraction",
            "ordinary_fraction",
            "large_finite",
            "overflow_neighborhood",
            "underflow_neighborhood",
            "date_serial_low",
            "date_serial_leap_boundary",
            "date_serial_high",
            "rate_near_zero",
            "rate_high",
            "solver_seed_sensitive"
        )
        local_only_bands = @("nan_local_only", "infinity_local_only")
    }
}

function Get-TextAxes {
    param([pscustomobject] $Row)

    $profileText = "$($Row.category) $($Row.kernel_signature_class) $($Row.coercion_lift_profile) $($Row.canonical_surface_name) $($Row.fec_dependency_profile)"
    $focus = Test-AnyMatch $profileText "(?i)text|string|lookup|reference|information|logical|database|web|xml|regex|locale|custom"
    if (-not $focus) {
        return [ordered]@{
            enabled = $false
            bands = @()
        }
    }

    return [ordered]@{
        enabled = $true
        bands = @(
            "empty_string",
            "whitespace_only",
            "numeric_looking_text",
            "date_looking_text",
            "boolean_looking_text",
            "error_looking_text",
            "case_variant",
            "unicode_sample",
            "normalization_sensitive",
            "wildcard_chars",
            "regex_like_chars",
            "delimiter_heavy",
            "long_string"
        )
    }
}

function Get-ArrayAxes {
    param([pscustomobject] $Row)

    $profileText = "$($Row.category) $($Row.coercion_lift_profile) $($Row.kernel_signature_class) $($Row.arg_preparation_profile) $($Row.canonical_surface_name)"
    $focus = Test-AnyMatch $profileText "(?i)array|aggregate|lookup|database|statistical|financial|math|custom|filter|sort|take|drop|hstack|vstack|choose|index|match|xlookup"
    if (-not $focus) {
        return [ordered]@{
            enabled = $false
            shape_bands = @("scalar_control", "single_cell_array")
        }
    }

    return [ordered]@{
        enabled = $true
        shape_bands = @(
            "scalar_control",
            "single_cell_array",
            "row_vector",
            "column_vector",
            "small_matrix",
            "mixed_type_matrix",
            "contains_errors",
            "contains_blanks",
            "shape_mismatch_pair",
            "spill_size_edge_sample",
            "grid_limit_sample"
        )
    }
}

function Get-ReferenceAxes {
    param([pscustomobject] $Row)

    $profileText = "$($Row.category) $($Row.arg_preparation_profile) $($Row.fec_dependency_profile) $($Row.surface_fec_dependency_profile) $($Row.host_interaction_class) $($Row.canonical_surface_name)"
    $focus = Test-AnyMatch $profileText "(?i)ref|lookup|database|aggregate|caller|workbook|formula|sheet|cell|offset|index|address|areas|subtotal"
    if (-not $focus) {
        return [ordered]@{
            enabled = $false
            reference_bands = @("reference_vs_array_literal_contrast")
        }
    }

    return [ordered]@{
        enabled = $true
        reference_bands = @(
            "single_cell",
            "rectangular_area",
            "same_sheet_multi_area",
            "cross_sheet_reference",
            "whole_row",
            "whole_column",
            "spill_anchor",
            "structured_reference",
            "external_reference_blocked",
            "reference_vs_array_literal_contrast"
        )
    }
}

function Get-ContextAxes {
    param([pscustomobject] $Row)

    $bands = New-StringSet
    Add-ManyTags $bands @(
        "excel_version_channel",
        "workbook_compatibility_version"
    )

    $profileText = "$($Row.category) $($Row.determinism_class) $($Row.volatility_class) $($Row.host_interaction_class) $($Row.fec_dependency_profile) $($Row.surface_fec_dependency_profile) $($Row.runtime_boundary_kind) $($Row.special_interface_kind)"
    if ($profileText -match "(?i)date|time") { Add-Tag $bands "date_system" }
    if ($profileText -match "(?i)locale|width|profile|text") { Add-Tag $bands "locale_profile" }
    if ($profileText -match "(?i)caller|workbook|sheet|cell|reference") { Add-Tag $bands "caller_location" }
    if ($profileText -match "(?i)volatile|random|time|external") { Add-Tag $bands "calculation_mode"; Add-Tag $bands "volatile_recalc" }
    if ($profileText -match "(?i)provider|cube|rtd|external|host") { Add-Tag $bands "host_provider_capability" }
    if ($profileText -match "(?i)reference|lookup|database|aggregate|spill|array") { Add-Tag $bands "worksheet_neighborhood" }

    return [ordered]@{
        context_bands = Get-Tags $bands
    }
}

function Get-ExecutionSeams {
    param([pscustomobject] $Row)

    $seams = New-StringSet
    Add-ManyTags $seams @("direct_oxfunc_value", "oxfml_prepared_call", "excel_formula_text")

    $profileText = "$($Row.runtime_boundary_kind) $($Row.special_interface_kind) $($Row.host_interaction_class) $($Row.fec_dependency_profile) $($Row.surface_fec_dependency_profile) $($Row.category)"
    if ($profileText -match "(?i)xll|registered|callable|lambda|presentation|width") {
        Add-Tag $seams "xll_bridge_future"
    }
    if ($profileText -match "(?i)provider|cube|rtd|external|host") {
        Add-Tag $seams "provider_host_future"
    }

    return Get-Tags $seams
}

function Get-KnownDeviationTags {
    param([string] $Name)

    $tags = New-StringSet
    switch ($Name.ToUpperInvariant()) {
        "PMT" { Add-Tag $tags "expected_known_financial_exactness_drift" }
        "PPMT" { Add-Tag $tags "expected_known_financial_exactness_drift" }
        "IPMT" { Add-Tag $tags "expected_known_financial_exactness_drift" }
        "POWER" { Add-Tag $tags "fresh_confirmation_required_before_known_bug" }
        "OP_POWER" { Add-Tag $tags "fresh_confirmation_required_before_known_bug" }
    }
    return Get-Tags $tags
}

function Get-BlockedOrDeferredLanes {
    param(
        [pscustomobject] $Row,
        [string[]] $DeferredRefs
    )

    $lanes = New-StringSet
    if ($DeferredRefs.Count -gt 0) { Add-Tag $lanes "current_version_deferred_inventory" }
    if ($Row.metadata_status -eq "catalog_only") { Add-Tag $lanes "catalog_only_metadata_gap" }
    if ([string]::IsNullOrWhiteSpace($Row.arity_min) -or [string]::IsNullOrWhiteSpace($Row.arity_max)) {
        Add-Tag $lanes "arity_metadata_gap"
    }
    if ($Row.special_interface_kind -and $Row.special_interface_kind -ne "ordinary") {
        Add-Tag $lanes "special_interface_context_required"
    }
    if ($Row.host_interaction_class -and $Row.host_interaction_class -ne "None") {
        Add-Tag $lanes "host_interaction_context_required"
    }
    if ($Row.fec_dependency_profile -match "(?i)ExternalProvider" -or $Row.surface_fec_dependency_profile -match "(?i)ExternalProvider") {
        Add-Tag $lanes "external_provider_context_required"
    }
    if ($Row.fec_dependency_profile -match "(?i)LocaleProfile" -or $Row.surface_fec_dependency_profile -match "(?i)LocaleProfile" -or $Row.runtime_boundary_kind -match "(?i)locale") {
        Add-Tag $lanes "locale_profile_context_required"
    }
    if ($Row.category -match "(?i)cubes") {
        Add-Tag $lanes "cube_provider_context_required"
    }
    if ($Row.volatility_class -and $Row.volatility_class -ne "NonVolatile") {
        Add-Tag $lanes "volatile_context_control_required"
    }

    return Get-Tags $lanes
}

function Get-RiskAndSelectionTags {
    param(
        [pscustomobject] $Row,
        [string[]] $BugRefs,
        [string[]] $BacklogRefs,
        [string[]] $ScenarioRefs,
        [string[]] $SourceRefs
    )

    $tags = New-StringSet
    if ($BugRefs.Count -gt 0) { Add-Tag $tags "bug_stream_mentioned" }
    if ($BacklogRefs.Count -gt 0) { Add-Tag $tags "w51_backlog_mentioned" }
    if ($ScenarioRefs.Count -gt 0) { Add-Tag $tags "seed_manifest_mentioned" } else { Add-Tag $tags "no_seed_manifest_mentions" }
    if ($SourceRefs.Count -gt 0) { Add-Tag $tags "source_file_mentioned" }
    if ($Row.metadata_status -ne "function_meta_extracted") { Add-Tag $tags "metadata_not_extracted" }
    if ($Row.coercion_lift_profile -match "(?i)custom|array|reference|aggregate|lookup") { Add-Tag $tags "complex_coercion_or_lift_profile" }
    if ([string]::IsNullOrWhiteSpace($Row.arity_min) -or [string]::IsNullOrWhiteSpace($Row.arity_max)) { Add-Tag $tags "metadata_missing_arity" }
    if ($Row.interesting -eq "true") { Add-Tag $tags "catalog_interesting" }
    if ($Row.category -match "(?i)financial|statistical|lookup|reference|text|database|operator") { Add-Tag $tags "family_priority_candidate" }

    return Get-Tags $tags
}

function ConvertTo-CountMap {
    param(
        [object[]] $Rows,
        [string] $PropertyName
    )

    $map = [ordered]@{}
    foreach ($group in ($Rows | Group-Object $PropertyName | Sort-Object Name)) {
        $key = $group.Name
        if ([string]::IsNullOrWhiteSpace($key)) {
            $key = "(blank)"
        }
        $map[$key] = $group.Count
    }
    return $map
}

$libraryPath = "docs\function-lane\OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv"
$catalogPath = "docs\function-lane\FUNCTION_CATALOG_CURRENT_BASELINE_LOCAL.csv"
$bugRegisterPath = "docs\bugs\BUG_STREAM_REGISTER.csv"
$w50DeferredPath = "docs\function-lane\W50_DEFERRED_CURRENT_VERSION_INVENTORY.csv"
$w51BacklogPath = "docs\function-lane\W51_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_INVENTORY.csv"

$libraryRows = Read-RequiredCsv $libraryPath
$catalogRows = Read-RequiredCsv $catalogPath
$bugRows = Read-RequiredCsv $bugRegisterPath
$w50Rows = Read-OptionalCsv $w50DeferredPath
$w51Rows = Read-OptionalCsv $w51BacklogPath

$surfaceByName = @{}
foreach ($row in $libraryRows) {
    if (-not [string]::IsNullOrWhiteSpace($row.canonical_surface_name)) {
        $surfaceByName[$row.canonical_surface_name.ToUpperInvariant()] = $row.surface_stable_id
    }
}
$surfaceNames = @($surfaceByName.Keys | Sort-Object)

$scenarioRefsBySurface = @{}
$scenarioFiles = @(Get-ChildItem -LiteralPath (Join-RepoPath "docs\function-lane") -Filter "*SCENARIO_MANIFEST_SEED.csv" -File)
foreach ($file in $scenarioFiles) {
    $content = Get-Content -LiteralPath $file.FullName -Raw
    $relative = Get-RepoRelativePath $file.FullName
    foreach ($match in [regex]::Matches($content, "\b([A-Z][A-Z0-9\.]{1,31})\s*\(")) {
        $name = $match.Groups[1].Value.ToUpperInvariant()
        if ($surfaceByName.ContainsKey($name)) {
            Add-MapSetValue $scenarioRefsBySurface $surfaceByName[$name] $relative
        }
    }
}

$deferredRefsBySurface = @{}
$deferredFiles = @(Get-ChildItem -LiteralPath (Join-RepoPath "docs\function-lane") -Filter "*DEFERRED*INVENTORY*.csv" -File)
foreach ($file in $deferredFiles) {
    $content = Get-Content -LiteralPath $file.FullName -Raw
    $relative = Get-RepoRelativePath $file.FullName
    foreach ($name in $surfaceNames) {
        if (Test-FunctionMention -Text $content -PathText $file.Name -Name $name) {
            Add-MapSetValue $deferredRefsBySurface $surfaceByName[$name] $relative
        }
    }
}
foreach ($row in $w50Rows) {
    $entryName = $row.entry_name
    if (-not $entryName) { $entryName = $row.function_name }
    if (-not [string]::IsNullOrWhiteSpace($entryName)) {
        $key = $entryName.ToUpperInvariant()
        if ($surfaceByName.ContainsKey($key)) {
            Add-MapSetValue $deferredRefsBySurface $surfaceByName[$key] $w50DeferredPath
        }
    }
}

$bugRefsBySurface = @{}
foreach ($bug in $bugRows) {
    $haystack = "$($bug.title) $($bug.notes) $($bug.stream_path)"
    foreach ($name in $surfaceNames) {
        if (Test-FunctionMention -Text $haystack -PathText $bug.stream_path -Name $name) {
            Add-MapSetValue $bugRefsBySurface $surfaceByName[$name] $bug.bug_id
        }
    }
}

$backlogRefsBySurface = @{}
foreach ($row in $w51Rows) {
    $entryName = $row.entry_name
    if (-not $entryName) { $entryName = $row.function_name }
    if (-not [string]::IsNullOrWhiteSpace($entryName)) {
        $key = $entryName.ToUpperInvariant()
        if ($surfaceByName.ContainsKey($key)) {
            Add-MapSetValue $backlogRefsBySurface $surfaceByName[$key] $w51BacklogPath
        }
    }
}

$sourceRefsBySurface = @{}
$sourceFiles = @(Get-ChildItem -LiteralPath (Join-RepoPath "crates\oxfunc_core\src\functions") -Filter "*.rs" -File)
foreach ($file in $sourceFiles) {
    $content = Get-Content -LiteralPath $file.FullName -Raw
    $relative = Get-RepoRelativePath $file.FullName
    foreach ($match in [regex]::Matches($content, "FUNC\.[A-Z0-9\._]+")) {
        $surfaceId = $match.Value.ToUpperInvariant()
        Add-MapSetValue $sourceRefsBySurface $surfaceId $relative
    }
}

$catalogByName = @{}
foreach ($catalog in $catalogRows) {
    if (-not [string]::IsNullOrWhiteSpace($catalog.function_name)) {
        $catalogByName[$catalog.function_name.ToUpperInvariant()] = $catalog
    }
}

$surfaces = @()
foreach ($row in ($libraryRows | Sort-Object canonical_surface_name)) {
    $surfaceId = $row.surface_stable_id
    $name = $row.canonical_surface_name
    $catalog = $null
    if ($catalogByName.ContainsKey($name.ToUpperInvariant())) {
        $catalog = $catalogByName[$name.ToUpperInvariant()]
    }

    $scenarioRefs = @(Get-MapSetValues $scenarioRefsBySurface $surfaceId)
    $deferredRefs = @(Get-MapSetValues $deferredRefsBySurface $surfaceId)
    $bugRefs = @(Get-MapSetValues $bugRefsBySurface $surfaceId)
    $backlogRefs = @(Get-MapSetValues $backlogRefsBySurface $surfaceId)
    $sourceRefs = @(Get-MapSetValues $sourceRefsBySurface $surfaceId)
    $arityShape = Get-ArityShape -Min $row.arity_min -Max $row.arity_max

    $surfaces += [ordered]@{
        surface_id = $surfaceId
        canonical_surface_name = $name
        category = $row.category
        metadata = [ordered]@{
            metadata_status = $row.metadata_status
            snapshot_id = $row.snapshot_id
            snapshot_generation = $row.snapshot_generation
            source_commit_short = $row.source_commit_short
            source_tree_state = $row.source_tree_state
            version_marker = $row.version_marker
            interesting = $row.interesting
        }
        function_surface = [ordered]@{
            entry_kind = $row.entry_kind
            registration_source_kind = $row.registration_source_kind
            special_interface_kind = $row.special_interface_kind
            admission_interface_kind = $row.admission_interface_kind
            runtime_boundary_kind = $row.runtime_boundary_kind
            arg_preparation_profile = $row.arg_preparation_profile
            coercion_lift_profile = $row.coercion_lift_profile
            kernel_signature_class = $row.kernel_signature_class
            determinism_class = $row.determinism_class
            volatility_class = $row.volatility_class
            host_interaction_class = $row.host_interaction_class
            thread_safety_class = $row.thread_safety_class
            fec_dependency_profile = $row.fec_dependency_profile
            surface_fec_dependency_profile = $row.surface_fec_dependency_profile
        }
        arity = [ordered]@{
            min = $row.arity_min
            max = $row.arity_max
            shape = $arityShape
            shape_note = $row.arity_shape_note
            probe_tags = @(Get-ArityProbeTags -Min $row.arity_min -Max $row.arity_max)
        }
        value_type_axes = Get-ValueKindAxes $row
        numeric_axes = Get-NumericAxes $row
        text_axes = Get-TextAxes $row
        array_axes = Get-ArrayAxes $row
        reference_axes = Get-ReferenceAxes $row
        context_axes = Get-ContextAxes $row
        execution_seams = @(Get-ExecutionSeams $row)
        comparison_policy = [ordered]@{
            pass_class = "exact_typed_bit_match"
            non_pass_classes = @(
                "known_expected_deviation",
                "unexpected_mismatch",
                "excel_harness_blocked",
                "oxfml_seam_blocked",
                "context_provider_blocked",
                "invalid_generator_case",
                "unstable_or_non_reproducible"
            )
            tolerance_pass_allowed = $false
        }
        coverage_counter_dimensions = @(
            "surface_id",
            "canonical_surface_name",
            "category",
            "metadata_status",
            "special_interface_kind",
            "runtime_boundary_kind",
            "arity_shape",
            "arity_probe",
            "value_kind_vector",
            "numeric_band",
            "text_band",
            "array_shape_band",
            "reference_band",
            "context_band",
            "execution_seam",
            "local_outcome_class",
            "excel_comparison_class",
            "known_deviation_tag",
            "blocked_or_deferred_lane",
            "selection_reason"
        )
        known_deviation_tags = @(Get-KnownDeviationTags $name)
        blocked_or_deferred_lanes = @(Get-BlockedOrDeferredLanes -Row $row -DeferredRefs $deferredRefs)
        risk_and_selection_tags = @(Get-RiskAndSelectionTags -Row $row -BugRefs $bugRefs -BacklogRefs $backlogRefs -ScenarioRefs $scenarioRefs -SourceRefs $sourceRefs)
        refs = [ordered]@{
            library_context = $libraryPath
            catalog = if ($catalog) { $catalogPath } else { $null }
            bug_streams = $bugRefs
            w51_backlog = $backlogRefs
            deferred_inventories = $deferredRefs
            scenario_manifests = $scenarioRefs
            source_files = $sourceRefs
            interface_contract_ref = $row.interface_contract_ref
            name_resolution_table_ref = $row.name_resolution_table_ref
            source_catalog_ref = $row.source_catalog_ref
        }
    }
}

$summaryRows = @(
    $surfaces |
        ForEach-Object {
            [pscustomobject]@{
                category = $_.category
                metadata_status = $_.metadata.metadata_status
                special_interface_kind = $_.function_surface.special_interface_kind
                runtime_boundary_kind = $_.function_surface.runtime_boundary_kind
                arity_shape = $_.arity.shape
            }
        }
)

$inventory = [ordered]@{
    schema_version = "oxfunc.smart_fuzzer.dimension_inventory.v0"
    authority = "derived_exploration_inventory_not_semantic_truth"
    generated_utc = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
    git_revision = Get-GitValue @("rev-parse", "HEAD")
    git_status_short_digest_source = Get-GitValue @("status", "--short")
    inputs = [ordered]@{
        library_context = $libraryPath
        catalog = $catalogPath
        bug_register = $bugRegisterPath
        w50_deferred_inventory = $w50DeferredPath
        w51_in_scope_not_complete_inventory = $w51BacklogPath
        scenario_manifest_count = $scenarioFiles.Count
        deferred_inventory_count = $deferredFiles.Count
        source_file_count = $sourceFiles.Count
    }
    comparison_policy = [ordered]@{
        default_pass_class = "exact_typed_bit_match"
        tolerance_pass_allowed = $false
    }
    summary = [ordered]@{
        surface_count = $surfaces.Count
        by_category = ConvertTo-CountMap -Rows $summaryRows -PropertyName "category"
        by_metadata_status = ConvertTo-CountMap -Rows $summaryRows -PropertyName "metadata_status"
        by_special_interface_kind = ConvertTo-CountMap -Rows $summaryRows -PropertyName "special_interface_kind"
        by_runtime_boundary_kind = ConvertTo-CountMap -Rows $summaryRows -PropertyName "runtime_boundary_kind"
        by_arity_shape = ConvertTo-CountMap -Rows $summaryRows -PropertyName "arity_shape"
        known_deviation_surface_count = @($surfaces | Where-Object { $_.known_deviation_tags.Count -gt 0 }).Count
        blocked_or_deferred_surface_count = @($surfaces | Where-Object { $_.blocked_or_deferred_lanes.Count -gt 0 }).Count
    }
    surfaces = $surfaces
}

$outputDir = Split-Path -Parent $OutputPath
New-Item -ItemType Directory -Force -Path $outputDir | Out-Null
$inventory | ConvertTo-Json -Depth 24 | Set-Content -LiteralPath $OutputPath -Encoding UTF8

Write-Host "Wrote $OutputPath"
Write-Host "Surfaces inventoried: $($surfaces.Count)"
Write-Host "Known-deviation surfaces: $($inventory.summary.known_deviation_surface_count)"
Write-Host "Blocked/deferred surfaces: $($inventory.summary.blocked_or_deferred_surface_count)"
Write-Host "Arity shapes:"
foreach ($key in $inventory.summary.by_arity_shape.Keys) {
    Write-Host ("  {0}: {1}" -f $key, $inventory.summary.by_arity_shape[$key])
}

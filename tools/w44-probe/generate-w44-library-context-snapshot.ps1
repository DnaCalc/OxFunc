param(
    [string]$OutCsv = "docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv"
)

$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..\")
Set-Location $repoRoot
$snapshotGeneration = Get-Date -Format "yyyy-MM-dd"
$sourceCommitShort = (git rev-parse --short HEAD).Trim()
$sourceCommitFull = (git rev-parse HEAD).Trim()
$sourceTreeState = if ([string]::IsNullOrWhiteSpace((git status --porcelain))) { "clean" } else { "dirty" }
$documentedCompleteSnapshotRefreshIds = @{}
$staleInventoryPath = "docs/function-lane/W44_DOCUMENTED_COMPLETE_SNAPSHOT_STALE_INVENTORY.csv"
if (Test-Path $staleInventoryPath) {
    Import-Csv $staleInventoryPath | ForEach-Object {
        if (-not [string]::IsNullOrWhiteSpace($_.surface_stable_id)) {
            $documentedCompleteSnapshotRefreshIds[$_.surface_stable_id] = $_
        }
    }
}
$xlcallCatalogPath = "docs/function-lane/XLCALL_CODE_CATALOG.csv"
$xlcallByStableId = @{}
if (Test-Path $xlcallCatalogPath) {
    Import-Csv $xlcallCatalogPath |
        Where-Object {
            $_.xlcall_category -eq "built_in_function" -and
            $_.oxfunc_match_status -eq "matched_current_catalog" -and
            -not [string]::IsNullOrWhiteSpace($_.oxfunc_surface_stable_id)
        } |
        ForEach-Object {
            $xlcallByStableId[$_.oxfunc_surface_stable_id] = $_
        }
}

$metaPattern = 'function_id:\s*"(?<id>[^"]+)"[\s\S]*?arity:\s*(?:Arity::exact\((?<exact>\d+)\)|Arity\s*\{\s*min:\s*(?<min>\d+),\s*max:\s*(?<max>\d+)\s*\})[\s\S]*?determinism:\s*DeterminismClass::(?<det>\w+)[\s\S]*?volatility:\s*VolatilityClass::(?<vol>\w+)[\s\S]*?host_interaction:\s*HostInteractionClass::(?<host>\w+)[\s\S]*?thread_safety:\s*ThreadSafetyClass::(?<thread>\w+)[\s\S]*?arg_preparation_profile:\s*ArgPreparationProfile::(?<arg>\w+)[\s\S]*?coercion_lift_profile:\s*CoercionLiftProfile::(?<coercion>\w+)[\s\S]*?kernel_signature_class:\s*KernelSignatureClass::(?<kernel>\w+)[\s\S]*?fec_dependency_profile:\s*FecDependencyProfile::(?<fec>\w+)[\s\S]*?surface_fec_dependency_profile:\s*FecDependencyProfile::(?<surface>\w+)'
$reshapeMetaPattern = 'pub const \w+_META:\s*FunctionMeta\s*=\s*reshape_meta!\("(?<id>[^"]+)",\s*(?<min>\d+),\s*(?<max>\d+)\);'
$engineeringMetaPattern = 'pub const \w+_META:\s*FunctionMeta\s*=\s*engineering_meta!\("(?<id>[^"]+)",\s*(?<min>\d+),\s*(?<max>\d+)\);'

$metaById = @{}
Get-ChildItem "crates/oxfunc_core/src/functions" -Filter *.rs | ForEach-Object {
    $text = Get-Content $_.FullName -Raw
    foreach ($m in [regex]::Matches($text, $metaPattern)) {
        $arityMin = if ($m.Groups['exact'].Success) { $m.Groups['exact'].Value } else { $m.Groups['min'].Value }
        $arityMax = if ($m.Groups['exact'].Success) { $m.Groups['exact'].Value } else { $m.Groups['max'].Value }
        $metaById[$m.Groups['id'].Value] = [ordered]@{
            arity_min = $arityMin
            arity_max = $arityMax
            arg_preparation_profile = $m.Groups['arg'].Value
            coercion_lift_profile = $m.Groups['coercion'].Value
            kernel_signature_class = $m.Groups['kernel'].Value
            determinism_class = $m.Groups['det'].Value
            volatility_class = $m.Groups['vol'].Value
            host_interaction_class = $m.Groups['host'].Value
            thread_safety_class = $m.Groups['thread'].Value
            fec_dependency_profile = $m.Groups['fec'].Value
            surface_fec_dependency_profile = $m.Groups['surface'].Value
            metadata_status = "function_meta_extracted"
        }
    }
    foreach ($m in [regex]::Matches($text, $reshapeMetaPattern)) {
        if (-not $metaById.ContainsKey($m.Groups['id'].Value)) {
            $metaById[$m.Groups['id'].Value] = [ordered]@{
                arity_min = $m.Groups['min'].Value
                arity_max = $m.Groups['max'].Value
                arg_preparation_profile = "ValuesOnlyPreAdapter"
                coercion_lift_profile = "Custom"
                kernel_signature_class = "Custom"
                determinism_class = "Deterministic"
                volatility_class = "NonVolatile"
                host_interaction_class = "None"
                thread_safety_class = "SafePure"
                fec_dependency_profile = "None"
                surface_fec_dependency_profile = "RefOnly"
                metadata_status = "function_meta_extracted"
            }
        }
    }
    foreach ($m in [regex]::Matches($text, $engineeringMetaPattern)) {
        if (-not $metaById.ContainsKey($m.Groups['id'].Value)) {
            $metaById[$m.Groups['id'].Value] = [ordered]@{
                arity_min = $m.Groups['min'].Value
                arity_max = $m.Groups['max'].Value
                arg_preparation_profile = "ValuesOnlyPreAdapter"
                coercion_lift_profile = "Custom"
                kernel_signature_class = "Custom"
                determinism_class = "Deterministic"
                volatility_class = "NonVolatile"
                host_interaction_class = "None"
                thread_safety_class = "SafePure"
                fec_dependency_profile = "None"
                surface_fec_dependency_profile = "RefOnly"
                metadata_status = "function_meta_extracted"
            }
        }
    }
}

$complexFamilyCuratedMetaById = @{
    "FUNC.COMPLEX" = @{ arity_min = "2"; arity_max = "3" }
    "FUNC.IMABS" = @{ arity_min = "1"; arity_max = "1" }
    "FUNC.IMAGINARY" = @{ arity_min = "1"; arity_max = "1" }
    "FUNC.IMARGUMENT" = @{ arity_min = "1"; arity_max = "1" }
    "FUNC.IMCONJUGATE" = @{ arity_min = "1"; arity_max = "1" }
    "FUNC.IMCOS" = @{ arity_min = "1"; arity_max = "1" }
    "FUNC.IMCOSH" = @{ arity_min = "1"; arity_max = "1" }
    "FUNC.IMCOT" = @{ arity_min = "1"; arity_max = "1" }
    "FUNC.IMCSC" = @{ arity_min = "1"; arity_max = "1" }
    "FUNC.IMCSCH" = @{ arity_min = "1"; arity_max = "1" }
    "FUNC.IMDIV" = @{ arity_min = "2"; arity_max = "2" }
    "FUNC.IMEXP" = @{ arity_min = "1"; arity_max = "1" }
    "FUNC.IMLN" = @{ arity_min = "1"; arity_max = "1" }
    "FUNC.IMLOG10" = @{ arity_min = "1"; arity_max = "1" }
    "FUNC.IMLOG2" = @{ arity_min = "1"; arity_max = "1" }
    "FUNC.IMPOWER" = @{ arity_min = "2"; arity_max = "2" }
    "FUNC.IMPRODUCT" = @{ arity_min = "1"; arity_max = "255" }
    "FUNC.IMREAL" = @{ arity_min = "1"; arity_max = "1" }
    "FUNC.IMSEC" = @{ arity_min = "1"; arity_max = "1" }
    "FUNC.IMSECH" = @{ arity_min = "1"; arity_max = "1" }
    "FUNC.IMSIN" = @{ arity_min = "1"; arity_max = "1" }
    "FUNC.IMSINH" = @{ arity_min = "1"; arity_max = "1" }
    "FUNC.IMSQRT" = @{ arity_min = "1"; arity_max = "1" }
    "FUNC.IMSUB" = @{ arity_min = "2"; arity_max = "2" }
    "FUNC.IMSUM" = @{ arity_min = "1"; arity_max = "255" }
    "FUNC.IMTAN" = @{ arity_min = "1"; arity_max = "1" }
}
foreach ($stableId in $complexFamilyCuratedMetaById.Keys) {
    if (-not $metaById.ContainsKey($stableId)) {
        $arity = $complexFamilyCuratedMetaById[$stableId]
        $metaById[$stableId] = [ordered]@{
            arity_min = $arity.arity_min
            arity_max = $arity.arity_max
            arg_preparation_profile = "ValuesOnlyPreAdapter"
            coercion_lift_profile = "Custom"
            kernel_signature_class = "Custom"
            determinism_class = "Deterministic"
            volatility_class = "NonVolatile"
            host_interaction_class = "None"
            thread_safety_class = "SafePure"
            fec_dependency_profile = "None"
            surface_fec_dependency_profile = "None"
            metadata_status = "function_meta_curated"
        }
    }
}

function Get-SpecialInterfaceKind {
    param(
        [string]$StableId,
        [string]$CanonicalName
    )

    switch ($StableId) {
        "FUNC.CALL" { return "registered_external_invocation" }
        "FUNC.REGISTER.ID" { return "registered_external_registration" }
        "FUNC.RTD" { return "host_subscription_provider" }
        "FUNC.OP_IMPLICIT_INTERSECTION" { return "implicit_intersection_operator" }
        "FUNC.NOW" { return "presentation_hinting_function" }
        "FUNC.TODAY" { return "presentation_hinting_function" }
        "FUNC.HYPERLINK" { return "presentation_hinting_function" }
        "FUNC.ASC" { return "width_conversion_host_profile" }
        "FUNC.DBCS" { return "width_conversion_host_profile" }
        "FUNC.JIS" { return "width_conversion_host_profile" }
        "FUNC.NUMBERVALUE" { return "locale_default_profiled_parse" }
        "FUNC.TRANSLATE" { return "provider_language_request" }
    }

    switch ($CanonicalName) {
        "LET" { return "callable_helper_formation" }
        "LAMBDA" { return "callable_helper_formation" }
        "ISOMITTED" { return "callable_helper_runtime" }
        "MAP" { return "callable_helper_runtime" }
        "REDUCE" { return "callable_helper_runtime" }
        "SCAN" { return "callable_helper_runtime" }
        "BYROW" { return "callable_helper_runtime" }
        "BYCOL" { return "callable_helper_runtime" }
        "MAKEARRAY" { return "callable_helper_runtime" }
        default { return "ordinary" }
    }
}

function Get-InterfaceContractRef {
    param(
        [string]$StableId,
        [string]$CanonicalName
    )

    switch ($StableId) {
        "FUNC.CALL" { return "docs/function-lane/FUNCTION_SLICE_CALL_REGISTER_ID_UDF_REGISTRATION_SEAM_PRELIM.md" }
        "FUNC.REGISTER.ID" { return "docs/function-lane/FUNCTION_SLICE_CALL_REGISTER_ID_UDF_REGISTRATION_SEAM_PRELIM.md" }
        "FUNC.RTD" { return "docs/function-lane/FUNCTION_SLICE_RTD_CONTRACT_PRELIM.md" }
        "FUNC.OP_IMPLICIT_INTERSECTION" { return "docs/function-lane/IMPLICIT_INTERSECTION_OPERATOR_INVESTIGATION.md" }
        "FUNC.NOW" { return "docs/function-lane/FUNCTION_SLICE_NOW_CONTRACT_PRELIM.md" }
        "FUNC.TODAY" { return "docs/function-lane/FUNCTION_SLICE_TODAY_CONTRACT_PRELIM.md" }
        "FUNC.HYPERLINK" { return "docs/function-lane/FUNCTION_SLICE_HYPERLINK_IMAGE_VALUE_MODEL_PRELIM.md" }
        "FUNC.ASC" { return "docs/function-lane/FUNCTION_SLICE_WIDTH_CONVERSION_HOST_PROFILE_CONTRACT_PRELIM.md" }
        "FUNC.DBCS" { return "docs/function-lane/FUNCTION_SLICE_WIDTH_CONVERSION_HOST_PROFILE_CONTRACT_PRELIM.md" }
        "FUNC.JIS" { return "docs/function-lane/FUNCTION_SLICE_WIDTH_CONVERSION_HOST_PROFILE_CONTRACT_PRELIM.md" }
        "FUNC.NUMBERVALUE" { return "docs/function-lane/FUNCTION_SLICE_NUMBERVALUE_LOCALE_DEFAULT_CONTRACT_PRELIM.md" }
        "FUNC.TRANSLATE" { return "docs/function-lane/FUNCTION_SLICE_TRANSLATE_PROVIDER_LANGUAGE_CONTRACT_PRELIM.md" }
    }

    if ($StableId -like "FUNC.OP_*") {
        return "docs/worksets/W045_NON_AT_OPERATOR_UNIVERSE_CLOSURE_PASS.md"
    }

    switch ($CanonicalName) {
        "LET" { return "docs/worksets/W038_FUNCTIONAL_LAMBDA_AND_HELPER_FAMILY.md" }
        "LAMBDA" { return "docs/worksets/W038_FUNCTIONAL_LAMBDA_AND_HELPER_FAMILY.md" }
        "ISOMITTED" { return "docs/worksets/W038_FUNCTIONAL_LAMBDA_AND_HELPER_FAMILY.md" }
        "MAP" { return "docs/worksets/W038_FUNCTIONAL_LAMBDA_AND_HELPER_FAMILY.md" }
        "REDUCE" { return "docs/worksets/W038_FUNCTIONAL_LAMBDA_AND_HELPER_FAMILY.md" }
        "SCAN" { return "docs/worksets/W038_FUNCTIONAL_LAMBDA_AND_HELPER_FAMILY.md" }
        "BYROW" { return "docs/worksets/W038_FUNCTIONAL_LAMBDA_AND_HELPER_FAMILY.md" }
        "BYCOL" { return "docs/worksets/W038_FUNCTIONAL_LAMBDA_AND_HELPER_FAMILY.md" }
        "MAKEARRAY" { return "docs/worksets/W038_FUNCTIONAL_LAMBDA_AND_HELPER_FAMILY.md" }
        "CHOOSECOLS" { return "docs/function-lane/FUNCTION_SLICE_DYNAMIC_ARRAY_SHAPING_AND_RESHAPING_FAMILY_CONTRACT_PRELIM.md" }
        "CHOOSEROWS" { return "docs/function-lane/FUNCTION_SLICE_DYNAMIC_ARRAY_SHAPING_AND_RESHAPING_FAMILY_CONTRACT_PRELIM.md" }
        "DROP" { return "docs/function-lane/FUNCTION_SLICE_DYNAMIC_ARRAY_SHAPING_AND_RESHAPING_FAMILY_CONTRACT_PRELIM.md" }
        "EXPAND" { return "docs/function-lane/FUNCTION_SLICE_DYNAMIC_ARRAY_SHAPING_AND_RESHAPING_FAMILY_CONTRACT_PRELIM.md" }
        "FILTER" { return "docs/function-lane/FUNCTION_SLICE_DYNAMIC_ARRAY_SHAPING_AND_RESHAPING_FAMILY_CONTRACT_PRELIM.md" }
        "SORT" { return "docs/function-lane/FUNCTION_SLICE_DYNAMIC_ARRAY_SHAPING_AND_RESHAPING_FAMILY_CONTRACT_PRELIM.md" }
        "SORTBY" { return "docs/function-lane/FUNCTION_SLICE_DYNAMIC_ARRAY_SHAPING_AND_RESHAPING_FAMILY_CONTRACT_PRELIM.md" }
        "TAKE" { return "docs/function-lane/FUNCTION_SLICE_DYNAMIC_ARRAY_SHAPING_AND_RESHAPING_FAMILY_CONTRACT_PRELIM.md" }
        "TOCOL" { return "docs/function-lane/FUNCTION_SLICE_DYNAMIC_ARRAY_SHAPING_AND_RESHAPING_FAMILY_CONTRACT_PRELIM.md" }
        "TOROW" { return "docs/function-lane/FUNCTION_SLICE_DYNAMIC_ARRAY_SHAPING_AND_RESHAPING_FAMILY_CONTRACT_PRELIM.md" }
        "TRANSPOSE" { return "docs/function-lane/FUNCTION_SLICE_DYNAMIC_ARRAY_SHAPING_AND_RESHAPING_FAMILY_CONTRACT_PRELIM.md" }
        "UNIQUE" { return "docs/function-lane/FUNCTION_SLICE_DYNAMIC_ARRAY_SHAPING_AND_RESHAPING_FAMILY_CONTRACT_PRELIM.md" }
        "VSTACK" { return "docs/function-lane/FUNCTION_SLICE_DYNAMIC_ARRAY_SHAPING_AND_RESHAPING_FAMILY_CONTRACT_PRELIM.md" }
        "WRAPCOLS" { return "docs/function-lane/FUNCTION_SLICE_DYNAMIC_ARRAY_SHAPING_AND_RESHAPING_FAMILY_CONTRACT_PRELIM.md" }
        "WRAPROWS" { return "docs/function-lane/FUNCTION_SLICE_DYNAMIC_ARRAY_SHAPING_AND_RESHAPING_FAMILY_CONTRACT_PRELIM.md" }
        default { return "" }
    }
}

function Get-AdmissionInterfaceKind {
    param(
        [string]$StableId,
        [string]$CanonicalName
    )

    switch ($StableId) {
        "FUNC.CALL" { return "macro_or_host_registered_call" }
        "FUNC.REGISTER.ID" { return "registered_external_lookup" }
        "FUNC.RTD" { return "host_subscription_call" }
        "FUNC.OP_IMPLICIT_INTERSECTION" { return "operator_form" }
        "FUNC.OP_ADD" { return "operator_form" }
    }

    switch ($CanonicalName) {
        "LET" { return "helper_formation" }
        "LAMBDA" { return "helper_formation" }
        "ISOMITTED" { return "callable_runtime_helper" }
        "MAP" { return "higher_order_call" }
        "REDUCE" { return "higher_order_call" }
        "SCAN" { return "higher_order_call" }
        "BYROW" { return "higher_order_call" }
        "BYCOL" { return "higher_order_call" }
        "MAKEARRAY" { return "higher_order_call" }
        default { return "ordinary_call" }
    }
}

function Get-PreparationOwner {
    param(
        [string]$StableId,
        [string]$CanonicalName
    )

    switch ($StableId) {
        "FUNC.CALL" { return "oxfml_then_oxfunc_then_host_registered_external" }
        "FUNC.REGISTER.ID" { return "oxfml_then_oxfunc_then_host_registered_external" }
        "FUNC.RTD" { return "host_above_oxfunc_then_oxfunc_projection" }
        "FUNC.OP_IMPLICIT_INTERSECTION" { return "oxfml_then_oxfunc" }
        "FUNC.OP_ADD" { return "oxfml_then_oxfunc" }
    }

    switch ($CanonicalName) {
        "LET" { return "oxfml_then_oxfunc" }
        "LAMBDA" { return "oxfml_then_oxfunc" }
        "ISOMITTED" { return "oxfml_then_oxfunc" }
        "MAP" { return "oxfml_then_oxfunc" }
        "REDUCE" { return "oxfml_then_oxfunc" }
        "SCAN" { return "oxfml_then_oxfunc" }
        "BYROW" { return "oxfml_then_oxfunc" }
        "BYCOL" { return "oxfml_then_oxfunc" }
        "MAKEARRAY" { return "oxfml_then_oxfunc" }
        default { return "oxfml_then_oxfunc" }
    }
}

function Get-RuntimeBoundaryKind {
    param(
        [string]$StableId,
        [string]$CanonicalName
    )

    switch ($StableId) {
        "FUNC.CALL" { return "registered_external_provider_projection" }
        "FUNC.REGISTER.ID" { return "registered_external_provider_projection" }
        "FUNC.RTD" { return "host_provider_projection" }
        "FUNC.OP_IMPLICIT_INTERSECTION" { return "caller_context_scalarization" }
        "FUNC.OP_ADD" { return "ordinary_eval" }
        "FUNC.NOW" { return "extended_value_with_presentation_hint" }
        "FUNC.TODAY" { return "extended_value_with_presentation_hint" }
        "FUNC.HYPERLINK" { return "extended_value_with_presentation_hint" }
        "FUNC.ASC" { return "typed_host_width_conversion_mode" }
        "FUNC.DBCS" { return "typed_host_width_conversion_mode" }
        "FUNC.JIS" { return "typed_host_width_conversion_mode" }
        "FUNC.NUMBERVALUE" { return "ordinary_eval_with_locale_defaults" }
        "FUNC.TRANSLATE" { return "host_provider_projection" }
    }

    switch ($CanonicalName) {
        "LET" { return "callable_helper_runtime_after_formation" }
        "LAMBDA" { return "callable_helper_runtime_after_formation" }
        "ISOMITTED" { return "callable_helper_runtime_after_formation" }
        "MAP" { return "callable_helper_runtime" }
        "REDUCE" { return "callable_helper_runtime" }
        "SCAN" { return "callable_helper_runtime" }
        "BYROW" { return "callable_helper_runtime" }
        "BYCOL" { return "callable_helper_runtime" }
        "MAKEARRAY" { return "callable_helper_runtime" }
        default { return "ordinary_eval" }
    }
}

function Get-ArityShapeNote {
    param(
        [string]$StableId,
        [string]$CanonicalName
    )

    switch ($StableId) {
        "FUNC.CALL" { return "either numeric register_id target or direct library/procedure[/type_text] target; trailing args stay raw for host external invocation" }
        "FUNC.REGISTER.ID" { return "library name, procedure name/ordinal, optional type_text; returns numeric register id from host registration seam" }
        "FUNC.RTD" { return "prog_id, server_name, then ordered topic strings" }
        "FUNC.OP_IMPLICIT_INTERSECTION" { return "single operand/operator source" }
        "FUNC.OP_ADD" { return "binary operator operands" }
        "FUNC.NOW" { return "nullary ordinary call; extended path returns numeric value plus number_format hint" }
        "FUNC.TODAY" { return "nullary ordinary call; extended path returns numeric value plus number_format hint" }
        "FUNC.HYPERLINK" { return "link location plus optional friendly name; extended path returns text value plus style=hyperlink hint" }
        "FUNC.ASC" { return "single text arg; host profile supplies pass-through or narrow conversion mode" }
        "FUNC.DBCS" { return "single text arg; host profile supplies pass-through or narrow conversion mode" }
        "FUNC.JIS" { return "single text arg; host profile supplies widen or unavailable mode" }
        "FUNC.NUMBERVALUE" { return "text plus optional decimal/group separators; omitted separators come from locale profile" }
        "FUNC.TRANSLATE" { return "text plus optional source/target language tags; same-language local passthrough, otherwise provider query" }
    }

    switch ($CanonicalName) {
        "LET" { return "odd-style helper shape: final arg is body; preceding args form name/value pairs" }
        "LAMBDA" { return "trailing arg is body; preceding args are parameter names" }
        "ISOMITTED" { return "single argument; meaning is helper-runtime-sensitive" }
        "MAP" { return "trailing arg callable; preceding args are mapped arrays" }
        "REDUCE" { return "initial accumulator, array, callable" }
        "SCAN" { return "initial accumulator, array, callable" }
        "BYROW" { return "array plus callable applied per row" }
        "BYCOL" { return "array plus callable applied per column" }
        "MAKEARRAY" { return "rows, cols, callable producing each coordinate cell" }
        default { return "" }
    }
}

$defaultMeta = [ordered]@{
    arity_min = ""
    arity_max = ""
    arg_preparation_profile = ""
    coercion_lift_profile = ""
    kernel_signature_class = ""
    determinism_class = ""
    volatility_class = ""
    host_interaction_class = ""
    thread_safety_class = ""
    fec_dependency_profile = ""
    surface_fec_dependency_profile = ""
    metadata_status = "catalog_only"
}

function Get-ManualMetaOverride {
    param([string]$StableId)

    switch ($StableId) {
        "FUNC.ASC" {
            return [ordered]@{
                arity_min = "1"
                arity_max = "1"
                arg_preparation_profile = "ValuesOnlyPreAdapter"
                coercion_lift_profile = "None"
                kernel_signature_class = "Custom"
                determinism_class = "Deterministic"
                volatility_class = "NonVolatile"
                host_interaction_class = "ApplicationState"
                thread_safety_class = "HostSerialized"
                fec_dependency_profile = "Composite"
                surface_fec_dependency_profile = "Composite"
                metadata_status = "function_meta_curated"
            }
        }
        "FUNC.DBCS" {
            return [ordered]@{
                arity_min = "1"
                arity_max = "1"
                arg_preparation_profile = "ValuesOnlyPreAdapter"
                coercion_lift_profile = "None"
                kernel_signature_class = "Custom"
                determinism_class = "Deterministic"
                volatility_class = "NonVolatile"
                host_interaction_class = "ApplicationState"
                thread_safety_class = "HostSerialized"
                fec_dependency_profile = "Composite"
                surface_fec_dependency_profile = "Composite"
                metadata_status = "function_meta_curated"
            }
        }
        "FUNC.JIS" {
            return [ordered]@{
                arity_min = "1"
                arity_max = "1"
                arg_preparation_profile = "ValuesOnlyPreAdapter"
                coercion_lift_profile = "None"
                kernel_signature_class = "Custom"
                determinism_class = "Deterministic"
                volatility_class = "NonVolatile"
                host_interaction_class = "ApplicationState"
                thread_safety_class = "HostSerialized"
                fec_dependency_profile = "Composite"
                surface_fec_dependency_profile = "Composite"
                metadata_status = "function_meta_curated"
            }
        }
        "FUNC.NUMBERVALUE" {
            return [ordered]@{
                arity_min = "1"
                arity_max = "3"
                arg_preparation_profile = "ValuesOnlyPreAdapter"
                coercion_lift_profile = "Custom"
                kernel_signature_class = "Custom"
                determinism_class = "Deterministic"
                volatility_class = "NonVolatile"
                host_interaction_class = "None"
                thread_safety_class = "SafePure"
                fec_dependency_profile = "LocaleProfile"
                surface_fec_dependency_profile = "LocaleProfile"
                metadata_status = "function_meta_curated"
            }
        }
        "FUNC.TRANSLATE" {
            return [ordered]@{
                arity_min = "1"
                arity_max = "3"
                arg_preparation_profile = "ValuesOnlyPreAdapter"
                coercion_lift_profile = "Custom"
                kernel_signature_class = "Custom"
                determinism_class = "Deterministic"
                volatility_class = "NonVolatile"
                host_interaction_class = "ExternalProvider"
                thread_safety_class = "HostSerialized"
                fec_dependency_profile = "ExternalProvider"
                surface_fec_dependency_profile = "ExternalProvider"
                metadata_status = "function_meta_curated"
            }
        }
        "FUNC.SUMIF" {
            return [ordered]@{
                arity_min = "2"
                arity_max = "3"
                arg_preparation_profile = "RefsVisibleInAdapter"
                coercion_lift_profile = "Custom"
                kernel_signature_class = "Custom"
                determinism_class = "Deterministic"
                volatility_class = "NonVolatile"
                host_interaction_class = "None"
                thread_safety_class = "SafePure"
                fec_dependency_profile = "RefOnly"
                surface_fec_dependency_profile = "RefOnly"
                metadata_status = "function_meta_curated"
            }
        }
        "FUNC.BESSELI" {
            return [ordered]@{
                arity_min = "2"
                arity_max = "2"
                arg_preparation_profile = "ValuesOnlyPreAdapter"
                coercion_lift_profile = "Custom"
                kernel_signature_class = "Custom"
                determinism_class = "Deterministic"
                volatility_class = "NonVolatile"
                host_interaction_class = "None"
                thread_safety_class = "SafePure"
                fec_dependency_profile = "None"
                surface_fec_dependency_profile = "None"
                metadata_status = "function_meta_curated"
            }
        }
        "FUNC.BESSELJ" {
            return [ordered]@{
                arity_min = "2"
                arity_max = "2"
                arg_preparation_profile = "ValuesOnlyPreAdapter"
                coercion_lift_profile = "Custom"
                kernel_signature_class = "Custom"
                determinism_class = "Deterministic"
                volatility_class = "NonVolatile"
                host_interaction_class = "None"
                thread_safety_class = "SafePure"
                fec_dependency_profile = "None"
                surface_fec_dependency_profile = "None"
                metadata_status = "function_meta_curated"
            }
        }
        "FUNC.BESSELK" {
            return [ordered]@{
                arity_min = "2"
                arity_max = "2"
                arg_preparation_profile = "ValuesOnlyPreAdapter"
                coercion_lift_profile = "Custom"
                kernel_signature_class = "Custom"
                determinism_class = "Deterministic"
                volatility_class = "NonVolatile"
                host_interaction_class = "None"
                thread_safety_class = "SafePure"
                fec_dependency_profile = "None"
                surface_fec_dependency_profile = "None"
                metadata_status = "function_meta_curated"
            }
        }
        "FUNC.BESSELY" {
            return [ordered]@{
                arity_min = "2"
                arity_max = "2"
                arg_preparation_profile = "ValuesOnlyPreAdapter"
                coercion_lift_profile = "Custom"
                kernel_signature_class = "Custom"
                determinism_class = "Deterministic"
                volatility_class = "NonVolatile"
                host_interaction_class = "None"
                thread_safety_class = "SafePure"
                fec_dependency_profile = "None"
                surface_fec_dependency_profile = "None"
                metadata_status = "function_meta_curated"
            }
        }
        default { return $null }
    }
}

$functionRows = Import-Csv "docs/function-lane/FUNCTION_CATALOG_CURRENT_BASELINE_LOCAL.csv" | ForEach-Object {
    $stableId = "FUNC.$($_.function_name.ToUpperInvariant())"
    $gatingRef = if ([string]::IsNullOrWhiteSpace($_.version_marker)) { "oxfunc.local.gating.current_baseline.default.v1" } else { "oxfunc.local.gating.version_marker.$($_.version_marker)" }
    $manualMeta = Get-ManualMetaOverride -StableId $stableId
    $meta = if ($metaById.ContainsKey($stableId)) {
        $metaById[$stableId]
    } elseif ($null -ne $manualMeta) {
        $manualMeta
    } else {
        $defaultMeta
    }
    [pscustomobject]([ordered]@{
        snapshot_id = "oxfunc-libctx-v1"
        snapshot_generation = $snapshotGeneration
        source_commit_short = $sourceCommitShort
        source_commit_full = $sourceCommitFull
        source_tree_state = $sourceTreeState
        lane_id = "oxfunc"
        entry_kind = "built_in_function"
        registration_source_kind = "built_in_catalog_function"
        surface_stable_id = $stableId
        xlcall_builtin_symbol = if ($xlcallByStableId.ContainsKey($stableId)) { $xlcallByStableId[$stableId].xlcall_symbol } else { "" }
        xlcall_builtin_code = if ($xlcallByStableId.ContainsKey($stableId)) { $xlcallByStableId[$stableId].xlcall_numeric_code } else { "" }
        canonical_surface_name = $_.function_name
        name_resolution_table_ref = "docs/function-lane/W28_FUNCTION_NAME_LOCALIZATION_LIBRARY_SEED.csv"
        semantic_trait_profile_ref = "oxfunc.local.profile.function_surface.current_baseline.v1"
        gating_profile_ref = $gatingRef
        version_marker = $_.version_marker
        category = $_.category
        interesting = $_.interesting
        arity_min = $meta.arity_min
        arity_max = $meta.arity_max
        arg_preparation_profile = $meta.arg_preparation_profile
        coercion_lift_profile = $meta.coercion_lift_profile
        kernel_signature_class = $meta.kernel_signature_class
        determinism_class = $meta.determinism_class
        volatility_class = $meta.volatility_class
        host_interaction_class = $meta.host_interaction_class
        thread_safety_class = $meta.thread_safety_class
        fec_dependency_profile = $meta.fec_dependency_profile
        surface_fec_dependency_profile = $meta.surface_fec_dependency_profile
        metadata_status = $meta.metadata_status
        special_interface_kind = Get-SpecialInterfaceKind -StableId $stableId -CanonicalName $_.function_name
        admission_interface_kind = Get-AdmissionInterfaceKind -StableId $stableId -CanonicalName $_.function_name
        preparation_owner = Get-PreparationOwner -StableId $stableId -CanonicalName $_.function_name
        runtime_boundary_kind = Get-RuntimeBoundaryKind -StableId $stableId -CanonicalName $_.function_name
        arity_shape_note = Get-ArityShapeNote -StableId $stableId -CanonicalName $_.function_name
        interface_contract_ref = Get-InterfaceContractRef -StableId $stableId -CanonicalName $_.function_name
        source_catalog_ref = "docs/function-lane/FUNCTION_CATALOG_CURRENT_BASELINE_LOCAL.csv"
    })
}

$functionRows | Where-Object { $_.surface_stable_id -in @("FUNC.ASC", "FUNC.DBCS", "FUNC.JIS") } | ForEach-Object {
    $_.arity_min = "1"
    $_.arity_max = "1"
    $_.arg_preparation_profile = "ValuesOnlyPreAdapter"
    $_.coercion_lift_profile = "None"
    $_.kernel_signature_class = "Custom"
    $_.determinism_class = "Deterministic"
    $_.volatility_class = "NonVolatile"
    $_.host_interaction_class = "ApplicationState"
    $_.thread_safety_class = "HostSerialized"
    $_.fec_dependency_profile = "Composite"
    $_.surface_fec_dependency_profile = "Composite"
    $_.metadata_status = "function_meta_curated"
}

$functionRows | Where-Object { $_.surface_stable_id -eq "FUNC.NUMBERVALUE" } | ForEach-Object {
    $_.arity_min = "1"
    $_.arity_max = "3"
    $_.arg_preparation_profile = "ValuesOnlyPreAdapter"
    $_.coercion_lift_profile = "Custom"
    $_.kernel_signature_class = "Custom"
    $_.determinism_class = "Deterministic"
    $_.volatility_class = "NonVolatile"
    $_.host_interaction_class = "None"
    $_.thread_safety_class = "SafePure"
    $_.fec_dependency_profile = "LocaleProfile"
    $_.surface_fec_dependency_profile = "LocaleProfile"
    $_.metadata_status = "function_meta_curated"
}

$functionRows | Where-Object { $_.surface_stable_id -eq "FUNC.TRANSLATE" } | ForEach-Object {
    $_.arity_min = "1"
    $_.arity_max = "3"
    $_.arg_preparation_profile = "ValuesOnlyPreAdapter"
    $_.coercion_lift_profile = "Custom"
    $_.kernel_signature_class = "Custom"
    $_.determinism_class = "Deterministic"
    $_.volatility_class = "NonVolatile"
    $_.host_interaction_class = "ExternalProvider"
    $_.thread_safety_class = "HostSerialized"
    $_.fec_dependency_profile = "ExternalProvider"
    $_.surface_fec_dependency_profile = "ExternalProvider"
    $_.metadata_status = "function_meta_curated"
}

$functionRows | ForEach-Object {
    if ($xlcallByStableId.ContainsKey($_.surface_stable_id)) {
        $_.xlcall_builtin_symbol = $xlcallByStableId[$_.surface_stable_id].xlcall_symbol
        $_.xlcall_builtin_code = $xlcallByStableId[$_.surface_stable_id].xlcall_numeric_code
    }
}

$operatorIds = Import-Csv "tools/xll-addin/oxfunc_xll/export_specs.csv" |
    Where-Object { $_.function_id -like "FUNC.OP_*" } |
    Select-Object -ExpandProperty function_id -Unique |
    Sort-Object

$operatorRows = foreach ($operatorId in $operatorIds) {
    $canonical = $operatorId -replace '^FUNC\.', ''
    $meta = if ($metaById.ContainsKey($operatorId)) { $metaById[$operatorId] } else { $defaultMeta }
    [pscustomobject]([ordered]@{
        snapshot_id = "oxfunc-libctx-v1"
        snapshot_generation = $snapshotGeneration
        source_commit_short = $sourceCommitShort
        source_commit_full = $sourceCommitFull
        source_tree_state = $sourceTreeState
        lane_id = "oxfunc"
        entry_kind = "built_in_operator"
        registration_source_kind = "built_in_operator_export"
        surface_stable_id = $operatorId
        xlcall_builtin_symbol = ""
        xlcall_builtin_code = ""
        canonical_surface_name = $canonical
        name_resolution_table_ref = "oxfunc.local.names.operators.current_baseline.v1"
        semantic_trait_profile_ref = "oxfunc.local.profile.operator_surface.current_baseline.v1"
        gating_profile_ref = "oxfunc.local.gating.current_baseline.default.v1"
        version_marker = ""
        category = "Operators"
        interesting = "true"
        arity_min = $meta.arity_min
        arity_max = $meta.arity_max
        arg_preparation_profile = $meta.arg_preparation_profile
        coercion_lift_profile = $meta.coercion_lift_profile
        kernel_signature_class = $meta.kernel_signature_class
        determinism_class = $meta.determinism_class
        volatility_class = $meta.volatility_class
        host_interaction_class = $meta.host_interaction_class
        thread_safety_class = $meta.thread_safety_class
        fec_dependency_profile = $meta.fec_dependency_profile
        surface_fec_dependency_profile = $meta.surface_fec_dependency_profile
        metadata_status = $meta.metadata_status
        special_interface_kind = Get-SpecialInterfaceKind -StableId $operatorId -CanonicalName $canonical
        admission_interface_kind = Get-AdmissionInterfaceKind -StableId $operatorId -CanonicalName $canonical
        preparation_owner = Get-PreparationOwner -StableId $operatorId -CanonicalName $canonical
        runtime_boundary_kind = Get-RuntimeBoundaryKind -StableId $operatorId -CanonicalName $canonical
        arity_shape_note = Get-ArityShapeNote -StableId $operatorId -CanonicalName $canonical
        interface_contract_ref = Get-InterfaceContractRef -StableId $operatorId -CanonicalName $canonical
        source_catalog_ref = "tools/xll-addin/oxfunc_xll/export_specs.csv"
    })
}

$implicitIntersectionRow = [pscustomobject]([ordered]@{
    snapshot_id = "oxfunc-libctx-v1"
    snapshot_generation = $snapshotGeneration
    source_commit_short = $sourceCommitShort
    source_commit_full = $sourceCommitFull
    source_tree_state = $sourceTreeState
    lane_id = "oxfunc"
    entry_kind = "built_in_operator"
    registration_source_kind = "doc_modeled_operator"
    surface_stable_id = "FUNC.OP_IMPLICIT_INTERSECTION"
    xlcall_builtin_symbol = ""
    xlcall_builtin_code = ""
    canonical_surface_name = "OP_IMPLICIT_INTERSECTION"
    name_resolution_table_ref = "oxfunc.local.names.operators.current_baseline.v1"
    semantic_trait_profile_ref = "oxfunc.local.profile.operator_surface.current_baseline.v1"
    gating_profile_ref = "oxfunc.local.gating.current_baseline.default.v1"
    version_marker = ""
    category = "Operators"
    interesting = "true"
    arity_min = "1"
    arity_max = "1"
    arg_preparation_profile = "RefsVisibleInAdapter"
    coercion_lift_profile = "Custom"
    kernel_signature_class = "Custom"
    determinism_class = "Deterministic"
    volatility_class = "NonVolatile"
    host_interaction_class = "WorkbookState"
    thread_safety_class = "HostSerialized"
    fec_dependency_profile = "Composite"
    surface_fec_dependency_profile = "Composite"
    metadata_status = "doc_modeled"
    special_interface_kind = "implicit_intersection_operator"
    admission_interface_kind = "operator_form"
    preparation_owner = "oxfml_then_oxfunc"
    runtime_boundary_kind = "caller_context_scalarization"
    arity_shape_note = "single operand/operator source"
    interface_contract_ref = "docs/function-lane/IMPLICIT_INTERSECTION_OPERATOR_INVESTIGATION.md"
    source_catalog_ref = "docs/function-lane/IMPLICIT_INTERSECTION_OPERATOR_INVESTIGATION.md"
})

$rows = @($functionRows) + @($operatorRows)
if (-not ($rows | Where-Object { $_.surface_stable_id -eq "FUNC.OP_IMPLICIT_INTERSECTION" })) {
    $rows += $implicitIntersectionRow
}

$rows |
    Where-Object {
        $documentedCompleteSnapshotRefreshIds.ContainsKey($_.surface_stable_id) -and
        $_.metadata_status -eq "catalog_only"
    } |
    ForEach-Object {
        $_.metadata_status = "function_meta_curated"
    }

$columnOrder = @(
    "snapshot_id",
    "snapshot_generation",
    "source_commit_short",
    "source_commit_full",
    "source_tree_state",
    "lane_id",
    "entry_kind",
    "registration_source_kind",
    "surface_stable_id",
    "xlcall_builtin_symbol",
    "xlcall_builtin_code",
    "canonical_surface_name",
    "name_resolution_table_ref",
    "semantic_trait_profile_ref",
    "gating_profile_ref",
    "version_marker",
    "category",
    "interesting",
    "arity_min",
    "arity_max",
    "arg_preparation_profile",
    "coercion_lift_profile",
    "kernel_signature_class",
    "determinism_class",
    "volatility_class",
    "host_interaction_class",
    "thread_safety_class",
    "fec_dependency_profile",
    "surface_fec_dependency_profile",
    "metadata_status",
    "special_interface_kind",
    "admission_interface_kind",
    "preparation_owner",
    "runtime_boundary_kind",
    "arity_shape_note",
    "interface_contract_ref",
    "source_catalog_ref"
)

$rows |
    Sort-Object entry_kind, surface_stable_id |
    Select-Object $columnOrder |
    Export-Csv $OutCsv -NoTypeInformation

$grouped = $rows | Group-Object entry_kind | ForEach-Object { "$($_.Name)=$($_.Count)" }
Write-Output ("wrote " + $rows.Count + " rows: " + ($grouped -join ", "))

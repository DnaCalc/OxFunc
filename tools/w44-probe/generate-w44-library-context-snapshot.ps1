param(
    [string]$OutCsv = "docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv"
)

$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..\")
Set-Location $repoRoot

$metaPattern = 'function_id:\s*"(?<id>[^"]+)"[\s\S]*?arity:\s*Arity\s*\{\s*min:\s*(?<min>\d+),\s*max:\s*(?<max>\d+)\s*\}[\s\S]*?determinism:\s*DeterminismClass::(?<det>\w+)[\s\S]*?volatility:\s*VolatilityClass::(?<vol>\w+)[\s\S]*?host_interaction:\s*HostInteractionClass::(?<host>\w+)[\s\S]*?thread_safety:\s*ThreadSafetyClass::(?<thread>\w+)[\s\S]*?arg_preparation_profile:\s*ArgPreparationProfile::(?<arg>\w+)[\s\S]*?coercion_lift_profile:\s*CoercionLiftProfile::(?<coercion>\w+)[\s\S]*?kernel_signature_class:\s*KernelSignatureClass::(?<kernel>\w+)[\s\S]*?fec_dependency_profile:\s*FecDependencyProfile::(?<fec>\w+)[\s\S]*?surface_fec_dependency_profile:\s*FecDependencyProfile::(?<surface>\w+)'

$metaById = @{}
Get-ChildItem "crates/oxfunc_core/src/functions" -Filter *.rs | ForEach-Object {
    $text = Get-Content $_.FullName -Raw
    foreach ($m in [regex]::Matches($text, $metaPattern)) {
        $metaById[$m.Groups['id'].Value] = [ordered]@{
            arity_min = $m.Groups['min'].Value
            arity_max = $m.Groups['max'].Value
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
}

function Get-SpecialInterfaceKind {
    param(
        [string]$StableId,
        [string]$CanonicalName
    )

    switch ($StableId) {
        "FUNC.RTD" { return "host_subscription_provider" }
        "FUNC.OP_IMPLICIT_INTERSECTION" { return "implicit_intersection_operator" }
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
        "FUNC.RTD" { return "docs/function-lane/FUNCTION_SLICE_RTD_CONTRACT_PRELIM.md" }
        "FUNC.OP_IMPLICIT_INTERSECTION" { return "docs/function-lane/IMPLICIT_INTERSECTION_OPERATOR_INVESTIGATION.md" }
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
        default { return "" }
    }
}

function Get-AdmissionInterfaceKind {
    param(
        [string]$StableId,
        [string]$CanonicalName
    )

    switch ($StableId) {
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
        "FUNC.RTD" { return "host_provider_projection" }
        "FUNC.OP_IMPLICIT_INTERSECTION" { return "caller_context_scalarization" }
        "FUNC.OP_ADD" { return "ordinary_eval" }
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
        "FUNC.RTD" { return "prog_id, server_name, then ordered topic strings" }
        "FUNC.OP_IMPLICIT_INTERSECTION" { return "single operand/operator source" }
        "FUNC.OP_ADD" { return "binary operator operands" }
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

$functionRows = Import-Csv "docs/function-lane/FUNCTION_CATALOG_CURRENT_BASELINE_LOCAL.csv" | ForEach-Object {
    $stableId = "FUNC.$($_.function_name.ToUpperInvariant())"
    $gatingRef = if ([string]::IsNullOrWhiteSpace($_.version_marker)) { "oxfunc.local.gating.current_baseline.default.v1" } else { "oxfunc.local.gating.version_marker.$($_.version_marker)" }
    $meta = if ($metaById.ContainsKey($stableId)) { $metaById[$stableId] } else { $defaultMeta }
    [pscustomobject]([ordered]@{
        snapshot_id = "oxfunc-libctx-v1"
        snapshot_generation = "2026-03-20"
        lane_id = "oxfunc"
        entry_kind = "built_in_function"
        surface_stable_id = $stableId
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

$operatorIds = Import-Csv "tools/xll-addin/oxfunc_xll/export_specs.csv" |
    Where-Object { $_.function_id -like "FUNC.OP_*" } |
    Select-Object -ExpandProperty function_id -Unique |
    Sort-Object

$operatorRows = foreach ($operatorId in $operatorIds) {
    $canonical = $operatorId -replace '^FUNC\.', ''
    $meta = if ($metaById.ContainsKey($operatorId)) { $metaById[$operatorId] } else { $defaultMeta }
    [pscustomobject]([ordered]@{
        snapshot_id = "oxfunc-libctx-v1"
        snapshot_generation = "2026-03-20"
        lane_id = "oxfunc"
        entry_kind = "built_in_operator"
        surface_stable_id = $operatorId
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
    snapshot_generation = "2026-03-20"
    lane_id = "oxfunc"
    entry_kind = "built_in_operator"
    surface_stable_id = "FUNC.OP_IMPLICIT_INTERSECTION"
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

$rows | Sort-Object entry_kind, surface_stable_id | Export-Csv $OutCsv -NoTypeInformation

$grouped = $rows | Group-Object entry_kind | ForEach-Object { "$($_.Name)=$($_.Count)" }
Write-Output ("wrote " + $rows.Count + " rows: " + ($grouped -join ", "))

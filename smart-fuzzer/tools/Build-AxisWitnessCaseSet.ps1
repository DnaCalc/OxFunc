param(
    [string]$InventoryPath = "smart-fuzzer/cache/dimension-inventory-v0.json",
    [string]$OutputPath = "smart-fuzzer/cache/axis-witness-case-set-v0.json",
    [switch]$IncludeKnownDeviationReferencePairs
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$ScriptPath = Split-Path -Parent $MyInvocation.MyCommand.Path
$RepoRoot = Resolve-Path (Join-Path $ScriptPath "..\..")
$RepoRoot = $RepoRoot.Path
Set-Location $RepoRoot

function New-JsonArray {
    param([AllowNull()][object]$Items)
    $list = New-Object 'System.Collections.ArrayList'
    if ($null -eq $Items) {
        return $list
    }
    if ($Items -is [System.Collections.IEnumerable] -and
        -not ($Items -is [string]) -and
        -not ($Items -is [System.Collections.IDictionary])) {
        foreach ($item in $Items) {
            [void]$list.Add($item)
        }
    } else {
        [void]$list.Add($Items)
    }
    return ,[object[]]$list.ToArray()
}

function New-NumberValue {
    param([double]$Value)
    return [ordered]@{ kind = "number"; value = $Value }
}

function New-TextValue {
    param([string]$Value)
    return [ordered]@{ kind = "text"; value = $Value }
}

function New-LogicalValue {
    param([bool]$Value)
    return [ordered]@{ kind = "logical"; value = $Value }
}

function New-ErrorValue {
    param([string]$Code)
    return [ordered]@{ kind = "error"; code = $Code }
}

function New-MissingValue {
    return [ordered]@{ kind = "missing_arg" }
}

function New-EmptyCellValue {
    return [ordered]@{ kind = "empty_cell" }
}

function New-ReferenceValue {
    param(
        [string]$Kind,
        [string]$Target
    )
    return [ordered]@{
        kind = "reference"
        reference_kind = $Kind
        target = $Target
    }
}

function New-ArrayValue {
    param([object]$Rows)
    return [ordered]@{ kind = "array"; rows = (New-JsonArray $Rows) }
}

function New-Row {
    param([object]$Cells)
    return New-JsonArray $Cells
}

function New-ArrayValueFromCellRows {
    param([object[][]]$Rows)

    $rowList = New-Object 'System.Collections.ArrayList'
    foreach ($row in $Rows) {
        $cellList = New-Object 'System.Collections.ArrayList'
        foreach ($cell in $row) {
            [void]$cellList.Add($cell)
        }
        [void]$rowList.Add($cellList)
    }
    return [ordered]@{ kind = "array"; rows = $rowList }
}

function New-Fixture {
    param(
        [string]$Target,
        [object]$Value
    )
    return [ordered]@{
        target = $Target
        value = $Value
    }
}

$cases = New-Object 'System.Collections.Generic.List[object]'
$trancheCaseIds = New-Object 'System.Collections.Generic.List[string]'
$surfaceNames = New-Object 'System.Collections.Generic.HashSet[string]'
$axisGroups = New-Object 'System.Collections.Generic.HashSet[string]'
$axisTags = New-Object 'System.Collections.Generic.HashSet[string]'

function Add-AxisCase {
    param(
        [string]$PairId,
        [string]$Role,
        [string]$AxisGroup,
        [string]$AxisTag,
        [string]$FunctionId,
        [string]$Name,
        [string]$Formula,
        [object]$CaseArgs,
        [string]$ExpectedProbeClass,
        [string[]]$CoverageTags,
        [object]$CellFixture = @(),
        [string]$FormulaCell = "",
        [Nullable[double]]$NowSerial = $null,
        [Nullable[double]]$RandomValue = $null,
        [string[]]$KnownDeviationTags = @()
    )

    $caseId = "w089-axis-{0}-{1}" -f $PairId, $Role
    $case = [ordered]@{
        schema_version = "oxfunc.smart_fuzzer.axis_witness_case.v0"
        run_id = "assigned_by_runner"
        tranche_id = "w089-axis-witness-sweep-v0"
        case_id = $caseId
        axis_pair_id = "w089-axis-$PairId"
        axis_role = $Role
        axis_group = $AxisGroup
        axis_tag = $AxisTag
        function_id = $FunctionId
        canonical_surface_name = $Name
        category = $AxisGroup
        case_tag = "axis_witness_$AxisTag"
        axis = $AxisTag
        expected_probe_class = $ExpectedProbeClass
        formula_text = $Formula
        args = (New-JsonArray $CaseArgs)
        cell_fixture = (New-JsonArray $CellFixture)
        formula_cell = if ([string]::IsNullOrWhiteSpace($FormulaCell)) { $null } else { $FormulaCell }
        now_serial = if ($null -eq $NowSerial) { $null } else { [double]$NowSerial }
        random_value = if ($null -eq $RandomValue) { $null } else { [double]$RandomValue }
        coverage_tags = (New-JsonArray $CoverageTags)
        known_deviation_tags = (New-JsonArray $KnownDeviationTags)
        blocked_or_deferred_lanes = (New-JsonArray @())
    }
    [void]$cases.Add($case)
    [void]$trancheCaseIds.Add($caseId)
    [void]$surfaceNames.Add($Name)
    [void]$axisGroups.Add($AxisGroup)
    [void]$axisTags.Add($AxisTag)
}

function Add-AxisPair {
    param(
        [string]$PairId,
        [string]$AxisGroup,
        [string]$AxisTag,
        [string]$ExpectedProbeClass,
        [string[]]$CoverageTags,
        [hashtable]$Control,
        [hashtable]$Variant
    )
    Add-AxisCase -PairId $PairId -Role "control" -AxisGroup $AxisGroup -AxisTag $AxisTag `
        -FunctionId $Control.FunctionId -Name $Control.Name -Formula $Control.Formula `
        -CaseArgs $Control.Args -ExpectedProbeClass $ExpectedProbeClass -CoverageTags $CoverageTags `
        -CellFixture $(if ($Control.ContainsKey("CellFixture")) { $Control.CellFixture } else { @() }) `
        -FormulaCell $(if ($Control.ContainsKey("FormulaCell")) { $Control.FormulaCell } else { "" }) `
        -NowSerial $(if ($Control.ContainsKey("NowSerial")) { $Control.NowSerial } else { $null }) `
        -RandomValue $(if ($Control.ContainsKey("RandomValue")) { $Control.RandomValue } else { $null }) `
        -KnownDeviationTags $(if ($Control.ContainsKey("KnownDeviationTags")) { $Control.KnownDeviationTags } else { @() })
    Add-AxisCase -PairId $PairId -Role "variant" -AxisGroup $AxisGroup -AxisTag $AxisTag `
        -FunctionId $Variant.FunctionId -Name $Variant.Name -Formula $Variant.Formula `
        -CaseArgs $Variant.Args -ExpectedProbeClass $ExpectedProbeClass -CoverageTags $CoverageTags `
        -CellFixture $(if ($Variant.ContainsKey("CellFixture")) { $Variant.CellFixture } else { @() }) `
        -FormulaCell $(if ($Variant.ContainsKey("FormulaCell")) { $Variant.FormulaCell } else { "" }) `
        -NowSerial $(if ($Variant.ContainsKey("NowSerial")) { $Variant.NowSerial } else { $null }) `
        -RandomValue $(if ($Variant.ContainsKey("RandomValue")) { $Variant.RandomValue } else { $null }) `
        -KnownDeviationTags $(if ($Variant.ContainsKey("KnownDeviationTags")) { $Variant.KnownDeviationTags } else { @() })
}

$matrix2x2 = New-ArrayValueFromCellRows @(
    @((New-NumberValue 1), (New-NumberValue 2)),
    @((New-NumberValue 3), (New-NumberValue 4))
)
$rowVector = New-ArrayValueFromCellRows (, @((New-NumberValue 1), (New-NumberValue 2)))
$columnVector = New-ArrayValueFromCellRows @(
    @((New-NumberValue 1)),
    @((New-NumberValue 2))
)
$absRowVector = New-ArrayValueFromCellRows (, @((New-NumberValue -3), (New-NumberValue 4)))
$countblankArraySubstitute = New-ArrayValueFromCellRows @(
    @((New-TextValue "")),
    @((New-NumberValue 1))
)

Add-AxisPair "function-identity" "function_surface" "function_identity" "different_function_same_argument" @("surface:function_identity", "category:math") `
    @{ FunctionId = "FUNC.ABS"; Name = "ABS"; Formula = "=ABS(-3)"; Args = @((New-NumberValue -3)) } `
    @{ FunctionId = "FUNC.SIGN"; Name = "SIGN"; Formula = "=SIGN(-3)"; Args = @((New-NumberValue -3)) }

Add-AxisPair "arity-optional" "arity" "optional_argument_present_vs_omitted" "optional_argument_changes_result" @("arity:omitted_optional_suffix", "arity:argc_at_max") `
    @{ FunctionId = "FUNC.TRUNC"; Name = "TRUNC"; Formula = "=TRUNC(12.34)"; Args = @((New-NumberValue 12.34)) } `
    @{ FunctionId = "FUNC.TRUNC"; Name = "TRUNC"; Formula = "=TRUNC(12.34,1)"; Args = @((New-NumberValue 12.34), (New-NumberValue 1)) }

Add-AxisPair "arity-explicit-missing" "arity" "explicit_missing_optional" "missing_optional_middle_argument" @("arity:explicit_missing_optional", "value:missing_arg", "array:small_matrix") `
    @{ FunctionId = "FUNC.TAKE"; Name = "TAKE"; Formula = "=TAKE({1,2;3,4},,1)"; Args = @($matrix2x2, (New-MissingValue), (New-NumberValue 1)) } `
    @{ FunctionId = "FUNC.TAKE"; Name = "TAKE"; Formula = "=TAKE({1,2;3,4},1,1)"; Args = @($matrix2x2, (New-NumberValue 1), (New-NumberValue 1)) }

Add-AxisPair "arity-variadic" "arity" "variadic_argument_count" "variadic_count_changes_result" @("arity:variadic_budget_low", "function:aggregate") `
    @{ FunctionId = "FUNC.SUM"; Name = "SUM"; Formula = "=SUM(1,2)"; Args = @((New-NumberValue 1), (New-NumberValue 2)) } `
    @{ FunctionId = "FUNC.SUM"; Name = "SUM"; Formula = "=SUM(1,2,3)"; Args = @((New-NumberValue 1), (New-NumberValue 2), (New-NumberValue 3)) }

Add-AxisPair "value-coercion" "value_type" "scalar_text_numeric_vs_non_numeric" "coercion_success_vs_error" @("value:scalar_text", "coercion:text_to_number") `
    @{ FunctionId = "FUNC.SUM"; Name = "SUM"; Formula = "=SUM(""2"",1)"; Args = @((New-TextValue "2"), (New-NumberValue 1)) } `
    @{ FunctionId = "FUNC.SUM"; Name = "SUM"; Formula = "=SUM(""x"",1)"; Args = @((New-TextValue "x"), (New-NumberValue 1)) }

Add-AxisPair "logical" "logical_error_blank" "logical_true_vs_false" "logical_condition_changes_branch" @("value:scalar_logical", "function:conditional") `
    @{ FunctionId = "FUNC.IF"; Name = "IF"; Formula = "=IF(TRUE,1,2)"; Args = @((New-LogicalValue $true), (New-NumberValue 1), (New-NumberValue 2)) } `
    @{ FunctionId = "FUNC.IF"; Name = "IF"; Formula = "=IF(FALSE,1,2)"; Args = @((New-LogicalValue $false), (New-NumberValue 1), (New-NumberValue 2)) }

Add-AxisPair "error" "logical_error_blank" "error_value_vs_number" "error_predicate_changes_result" @("value:scalar_error", "comparison:error_code_equality") `
    @{ FunctionId = "FUNC.ISERROR"; Name = "ISERROR"; Formula = "=ISERROR(#VALUE!)"; Args = @((New-ErrorValue "Value")) } `
    @{ FunctionId = "FUNC.ISERROR"; Name = "ISERROR"; Formula = "=ISERROR(1)"; Args = @((New-NumberValue 1)) }

Add-AxisPair "blank-vs-empty-text" "logical_error_blank" "blank_cell_vs_empty_text" "blankness_distinction" @("value:blank_cell", "value:empty_string", "reference:single_cell") `
    @{ FunctionId = "FUNC.ISBLANK"; Name = "ISBLANK"; Formula = "=ISBLANK(A1)"; Args = @((New-ReferenceValue "A1" "A1")); CellFixture = @((New-Fixture "A1" (New-EmptyCellValue))); FormulaCell = "J10" } `
    @{ FunctionId = "FUNC.ISBLANK"; Name = "ISBLANK"; Formula = "=ISBLANK("""")"; Args = @((New-TextValue "")) }

Add-AxisPair "numeric-zero-edge" "numeric" "zero_to_zero_power_edge" "numeric_edge_error_vs_number" @("numeric:positive_zero", "numeric:small_integer") `
    @{ FunctionId = "FUNC.POWER"; Name = "POWER"; Formula = "=POWER(0,0)"; Args = @((New-NumberValue 0), (New-NumberValue 0)) } `
    @{ FunctionId = "FUNC.POWER"; Name = "POWER"; Formula = "=POWER(2,3)"; Args = @((New-NumberValue 2), (New-NumberValue 3)) }

Add-AxisPair "numeric-sign-band" "numeric" "tiny_negative_vs_zero" "numeric_band_changes_sign" @("numeric:tiny_magnitude", "numeric:positive_zero") `
    @{ FunctionId = "FUNC.SIGN"; Name = "SIGN"; Formula = "=SIGN(0)"; Args = @((New-NumberValue 0)) } `
    @{ FunctionId = "FUNC.SIGN"; Name = "SIGN"; Formula = "=SIGN(-0.0000000001)"; Args = @((New-NumberValue -0.0000000001)) }

Add-AxisPair "text-empty-whitespace" "text" "empty_string_vs_whitespace" "text_length_band" @("text:empty_string", "text:whitespace_only") `
    @{ FunctionId = "FUNC.LEN"; Name = "LEN"; Formula = "=LEN("""")"; Args = @((New-TextValue "")) } `
    @{ FunctionId = "FUNC.LEN"; Name = "LEN"; Formula = "=LEN(""  "")"; Args = @((New-TextValue "  ")) }

Add-AxisPair "text-case" "text" "case_variant" "case_sensitive_compare" @("text:case_variant") `
    @{ FunctionId = "FUNC.EXACT"; Name = "EXACT"; Formula = "=EXACT(""abc"",""abc"")"; Args = @((New-TextValue "abc"), (New-TextValue "abc")) } `
    @{ FunctionId = "FUNC.EXACT"; Name = "EXACT"; Formula = "=EXACT(""abc"",""ABC"")"; Args = @((New-TextValue "abc"), (New-TextValue "ABC")) }

Add-AxisPair "array-scalar-vs-array" "array" "scalar_control_vs_array_literal" "array_lift_changes_shape" @("array:scalar_control", "array:row_vector") `
    @{ FunctionId = "FUNC.ABS"; Name = "ABS"; Formula = "=ABS(-3)"; Args = @((New-NumberValue -3)) } `
    @{ FunctionId = "FUNC.ABS"; Name = "ABS"; Formula = "=ABS({-3,4})"; Args = @($absRowVector) }

Add-AxisPair "array-orientation" "array" "row_vector_vs_column_vector" "orientation_changes_rows_result" @("array:row_vector", "array:column_vector") `
    @{ FunctionId = "FUNC.ROWS"; Name = "ROWS"; Formula = "=ROWS({1,2})"; Args = @($rowVector) } `
    @{ FunctionId = "FUNC.ROWS"; Name = "ROWS"; Formula = "=ROWS({1;2})"; Args = @($columnVector) }

Add-AxisPair "reference-vs-array" "reference" "reference_vs_array_literal_contrast" "range_only_policy_contrast" @("reference:reference_vs_array_literal_contrast", "reference:single_cell") `
    @{ FunctionId = "FUNC.COUNTBLANK"; Name = "COUNTBLANK"; Formula = "=COUNTBLANK(A1)"; Args = @((New-ReferenceValue "A1" "A1")); CellFixture = @((New-Fixture "A1" (New-EmptyCellValue))); FormulaCell = "J10" } `
    @{ FunctionId = "FUNC.COUNTBLANK"; Name = "COUNTBLANK"; Formula = "=LET(d,{"""";1},COUNTBLANK(d))"; Args = @($countblankArraySubstitute) }

Add-AxisPair "reference-area-shape" "reference" "single_cell_vs_rectangular_area" "reference_shape_changes_columns" @("reference:single_cell", "reference:rectangular_area") `
    @{ FunctionId = "FUNC.COLUMNS"; Name = "COLUMNS"; Formula = "=COLUMNS(A1)"; Args = @((New-ReferenceValue "A1" "A1")) } `
    @{ FunctionId = "FUNC.COLUMNS"; Name = "COLUMNS"; Formula = "=COLUMNS(A1:B2)"; Args = @((New-ReferenceValue "Area" "A1:B2")) }

Add-AxisPair "reference-multi-area" "reference" "same_sheet_multi_area" "multi_area_changes_area_count" @("reference:same_sheet_multi_area") `
    @{ FunctionId = "FUNC.AREAS"; Name = "AREAS"; Formula = "=AREAS(A1)"; Args = @((New-ReferenceValue "A1" "A1")) } `
    @{ FunctionId = "FUNC.AREAS"; Name = "AREAS"; Formula = "=AREAS((A1,B2:B3))"; Args = @((New-ReferenceValue "MultiArea" "(A1,B2:B3)")) }

Add-AxisPair "caller-location" "context" "caller_location" "caller_location_changes_row" @("context:caller_location") `
    @{ FunctionId = "FUNC.ROW"; Name = "ROW"; Formula = "=ROW()"; Args = @(); FormulaCell = "A5" } `
    @{ FunctionId = "FUNC.ROW"; Name = "ROW"; Formula = "=ROW()"; Args = @(); FormulaCell = "A6" }

Add-AxisPair "comparison-error-code" "comparison" "error_code_equality" "error_code_changes_number" @("comparison:error_code_equality", "value:scalar_error") `
    @{ FunctionId = "FUNC.ERROR.TYPE"; Name = "ERROR.TYPE"; Formula = "=ERROR.TYPE(#N/A)"; Args = @((New-ErrorValue "NA")) } `
    @{ FunctionId = "FUNC.ERROR.TYPE"; Name = "ERROR.TYPE"; Formula = "=ERROR.TYPE(#VALUE!)"; Args = @((New-ErrorValue "Value")) }

if ($IncludeKnownDeviationReferencePairs) {
    Add-AxisPair "known-deviation-reference" "exploration_feedback" "known_expected_deviation_reference" "known_deviation_reference_mismatch" @("selection:known_deviation_tag", "function:financial") `
        @{ FunctionId = "FUNC.PMT"; Name = "PMT"; Formula = "=PMT(0.05/12,360,200000)"; Args = @((New-NumberValue (0.05 / 12.0)), (New-NumberValue 360), (New-NumberValue 200000)); KnownDeviationTags = @("expected_known_financial_exactness_drift") } `
        @{ FunctionId = "FUNC.PMT"; Name = "PMT"; Formula = "=PMT(0.01/12,12,1000)"; Args = @((New-NumberValue (0.01 / 12.0)), (New-NumberValue 12), (New-NumberValue 1000)); KnownDeviationTags = @("expected_known_financial_exactness_drift") }
}

$blockedAxisWitnesses = @(
    [ordered]@{ axis_group = "context"; axis_tag = "workbook_compatibility_version"; fixture_requirement = "alternate_workbook_compatibility_fixture"; reason = "captured in metadata but not controllable by the pure value comparator" },
    [ordered]@{ axis_group = "context"; axis_tag = "date_system"; fixture_requirement = "workbook_date_system_fixture"; reason = "requires workbook-level date-system control" },
    [ordered]@{ axis_group = "context"; axis_tag = "locale_profile"; fixture_requirement = "LocaleFormatContext plus matching Excel locale fixture"; reason = "default runner does not switch host locale" },
    [ordered]@{ axis_group = "context"; axis_tag = "volatile_recalc"; fixture_requirement = "controlled recalc/statistical comparator"; reason = "RAND/RANDBETWEEN/RANDARRAY are not per-draw bit-exact comparators" },
    [ordered]@{ axis_group = "context"; axis_tag = "host_provider_capability"; fixture_requirement = "typed host/provider fixture"; reason = "provider/cube/web/IMAGE/INFO-style calls need host capabilities" },
    [ordered]@{ axis_group = "execution_seam"; axis_tag = "oxfml_prepared_call"; fixture_requirement = "OxFml prepared-call runner"; reason = "this batch compares direct OxFunc value calls against Excel Formula2" },
    [ordered]@{ axis_group = "execution_seam"; axis_tag = "xll_bridge_future"; fixture_requirement = "XLL bridge harness"; reason = "outside current smart-fuzzer pure value comparator" },
    [ordered]@{ axis_group = "execution_seam"; axis_tag = "provider_host_future"; fixture_requirement = "provider host harness"; reason = "outside current smart-fuzzer pure value comparator" },
    [ordered]@{ axis_group = "reference"; axis_tag = "cross_sheet_reference"; fixture_requirement = "multi-sheet local resolver and workbook fixture"; reason = "not admitted by the current single-sheet runner" },
    [ordered]@{ axis_group = "reference"; axis_tag = "structured_reference"; fixture_requirement = "table/structured-reference workbook fixture"; reason = "requires workbook object model setup" },
    [ordered]@{ axis_group = "reference"; axis_tag = "spill_anchor"; fixture_requirement = "spill-neighborhood workbook fixture"; reason = "requires preserving a produced spill anchor before the target formula" },
    [ordered]@{ axis_group = "value_type"; axis_tag = "callable_or_lambda"; fixture_requirement = "formula-binding or callable fixture"; reason = "LET/LAMBDA/helper values are outside direct value-call comparison" },
    [ordered]@{ axis_group = "value_type"; axis_tag = "presentation_or_rich_value"; fixture_requirement = "extended-value host publication fixture"; reason = "requires extended return-surface comparator" },
    [ordered]@{ axis_group = "value_type"; axis_tag = "provider_context_value"; fixture_requirement = "provider/context fixture"; reason = "requires provider value source" },
    [ordered]@{ axis_group = "comparison"; axis_tag = "statistical_profile_class"; fixture_requirement = "aggregate stochastic comparator"; reason = "not a per-case exact typed equality lane" }
)

foreach ($blocked in $blockedAxisWitnesses) {
    [void]$axisGroups.Add([string]$blocked.axis_group)
    [void]$axisTags.Add([string]$blocked.axis_tag)
}

$inventorySummary = $null
$resolvedInventory = Join-Path $RepoRoot $InventoryPath
if (Test-Path $resolvedInventory) {
    $inventorySummary = (Get-Content $resolvedInventory -Raw -Encoding UTF8 | ConvertFrom-Json).summary
}

$tranches = New-JsonArray @([ordered]@{
    tranche_id = "w089-axis-witness-sweep-v0"
    category = "axis_witness"
    surface_count = $surfaceNames.Count
    case_count = $cases.Count
    case_ids = $trancheCaseIds
})

$result = [ordered]@{
    schema_version = "oxfunc.smart_fuzzer.axis_witness_case_set.v0"
    authority = "derived_w089_execution_input_not_semantic_truth"
    generated_utc = (Get-Date).ToUniversalTime().ToString("o")
    source_inventory = $InventoryPath
    source_inventory_summary = $inventorySummary
    tranche_id = "w089-axis-witness-sweep-v0"
    comparison_policy = "exact_typed_bit_match_no_tolerance"
    execution_surfaces = @("direct_oxfunc_value", "excel_formula_text")
    include_known_deviation_reference_pairs = [bool]$IncludeKnownDeviationReferencePairs
    cases = $cases
    tranches = $tranches
    blocked_axis_witnesses = $blockedAxisWitnesses
    summary = [ordered]@{
        case_count = $cases.Count
        pair_count = [int]($cases.Count / 2)
        runnable_axis_group_count = $axisGroups.Count
        covered_axis_tags = @($axisTags | Sort-Object)
        blocked_axis_witness_count = $blockedAxisWitnesses.Count
    }
}

$resolvedOutput = Join-Path $RepoRoot $OutputPath
$parent = Split-Path -Parent $resolvedOutput
if (-not (Test-Path $parent)) {
    [void](New-Item -ItemType Directory -Path $parent -Force)
}
$result | ConvertTo-Json -Depth 100 | Set-Content -Path $resolvedOutput -Encoding UTF8
Write-Host "wrote $resolvedOutput"
Write-Host "cases=$($cases.Count) pairs=$([int]($cases.Count / 2)) blocked_axis_witnesses=$($blockedAxisWitnesses.Count)"

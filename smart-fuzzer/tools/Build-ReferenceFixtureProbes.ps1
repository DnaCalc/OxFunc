[CmdletBinding(PositionalBinding = $false)]
param(
    [string] $RepoRoot,
    [string] $OutputPath,
    [string] $TrancheId = "reference-fixture-probes-v0"
)

# Synthesize reference-fixture probe cases for the unswept reference
# functions and the harness-blocked ranking / lookup-over-reference
# surfaces in the function status map.
#
# These surfaces could not be tested by the value-only structural sweep
# because they take a *reference* argument (a table / vector / cell), not
# an inline value. The plumbing to evaluate a reference case already
# exists on both sides:
#   - the Excel runner writes each `cell_fixture` to its target via
#     Range.Value2 / Range, then places the formula at J10;
#   - the local evaluator (array_tranche_local_eval) resolves a
#     `reference` arg through CaseResolver: ReferenceResolver keyed by the
#     cell_fixture target.
# So this generator only has to emit, per surface, a valid reference-bearing
# (formula_text, args, cell_fixture) triple where the formula references the
# same targets the args name and the fixtures populate.
#
# Each case keeps inline numeric literals to short integers (bit-exact
# plumbing rule, CHARTER §4.1) and routes all data through cell_fixture.
#
# Scope: reference FUNCTIONS + ranking. Reference OPERATORS are handled by
# a sibling generator (different dispatch path).

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($RepoRoot)) {
    $scriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
    $RepoRoot = (Resolve-Path (Join-Path $scriptRoot "..\..")).Path
}
$RepoRoot = [System.IO.Path]::GetFullPath($RepoRoot)
if ([string]::IsNullOrWhiteSpace($OutputPath)) {
    $OutputPath = Join-Path $RepoRoot "smart-fuzzer\cache\reference-fixture-probes-v0.json"
}

# ---- value / fixture / reference constructors (mirror Build-AxisWitnessCaseSet) ----
function New-NumberValue  { param([double]$V) [ordered]@{ kind = "number";  value = $V } }
function New-TextValue    { param([string]$V) [ordered]@{ kind = "text";    value = $V } }
function New-LogicalValue { param([bool]$V)   [ordered]@{ kind = "logical"; value = $V } }
function New-ErrorValue   { param([string]$C) [ordered]@{ kind = "error";   code = $C } }
function New-EmptyCellValue {                 [ordered]@{ kind = "empty_cell" } }
function New-RefValue {
    param([string]$Kind, [string]$Target)
    [ordered]@{ kind = "reference"; reference_kind = $Kind; target = $Target }
}
function New-ArrayValueFromCellRows {
    param([object[][]]$Rows)
    $rowList = New-Object 'System.Collections.ArrayList'
    foreach ($row in $Rows) {
        $cellList = New-Object 'System.Collections.ArrayList'
        foreach ($cell in $row) { [void]$cellList.Add($cell) }
        [void]$rowList.Add($cellList)
    }
    [ordered]@{ kind = "array"; rows = $rowList }
}
function New-Fixture { param([string]$Target, [object]$Value) [ordered]@{ target = $Target; value = $Value } }

# ---- shared fixture grids -------------------------------------------------
# A 4x2 mixed key/value table at A1:B4: numeric key, text payload.
function Grid-KeyText {
    New-Fixture "A1:B4" (New-ArrayValueFromCellRows @(
        @((New-NumberValue 1), (New-TextValue "a")),
        @((New-NumberValue 2), (New-TextValue "b")),
        @((New-NumberValue 3), (New-TextValue "c")),
        @((New-NumberValue 4), (New-TextValue "d"))
    ))
}
# A 2x4 row-oriented variant at A1:D2 for HLOOKUP.
function Grid-KeyTextRow {
    New-Fixture "A1:D2" (New-ArrayValueFromCellRows @(
        @((New-NumberValue 1), (New-NumberValue 2), (New-NumberValue 3), (New-NumberValue 4)),
        @((New-TextValue "a"), (New-TextValue "b"), (New-TextValue "c"), (New-TextValue "d"))
    ))
}
# Column vector of numbers at A1:A4.
function Col-Nums { param([double[]]$Vals, [string]$Target)
    $rows = @(); foreach ($v in $Vals) { $rows += ,@((New-NumberValue $v)) }
    New-Fixture $Target (New-ArrayValueFromCellRows $rows)
}
# Column vector of text at B1:B4.
function Col-Text { param([string[]]$Vals, [string]$Target)
    $rows = @(); foreach ($v in $Vals) { $rows += ,@((New-TextValue $v)) }
    New-Fixture $Target (New-ArrayValueFromCellRows $rows)
}

# ---- curated per-surface case specs --------------------------------------
# Each spec: FunctionId, Name, Category, Tag, Formula, Args[], CellFixture[]
$specs = @(
    # --- lookup family ---
    @{ N="VLOOKUP"; Cat="Lookup and reference functions"; Tag="exact_match"; F="=VLOOKUP(3,A1:B4,2,FALSE)";
       Args=@((New-NumberValue 3),(New-RefValue "Area" "A1:B4"),(New-NumberValue 2),(New-LogicalValue $false)); Fix=@((Grid-KeyText)) }
    @{ N="VLOOKUP"; Cat="Lookup and reference functions"; Tag="not_found_na"; F="=VLOOKUP(9,A1:B4,2,FALSE)";
       Args=@((New-NumberValue 9),(New-RefValue "Area" "A1:B4"),(New-NumberValue 2),(New-LogicalValue $false)); Fix=@((Grid-KeyText)) }
    @{ N="HLOOKUP"; Cat="Lookup and reference functions"; Tag="exact_match"; F="=HLOOKUP(3,A1:D2,2,FALSE)";
       Args=@((New-NumberValue 3),(New-RefValue "Area" "A1:D2"),(New-NumberValue 2),(New-LogicalValue $false)); Fix=@((Grid-KeyTextRow)) }
    @{ N="XLOOKUP"; Cat="Lookup and reference functions"; Tag="exact_match"; F="=XLOOKUP(3,A1:A4,B1:B4)";
       Args=@((New-NumberValue 3),(New-RefValue "Area" "A1:A4"),(New-RefValue "Area" "B1:B4")); Fix=@((Col-Nums @(1,2,3,4) "A1:A4"),(Col-Text @("a","b","c","d") "B1:B4")) }
    @{ N="LOOKUP"; Cat="Lookup and reference functions"; Tag="vector_form"; F="=LOOKUP(3,A1:A4,B1:B4)";
       Args=@((New-NumberValue 3),(New-RefValue "Area" "A1:A4"),(New-RefValue "Area" "B1:B4")); Fix=@((Col-Nums @(1,2,3,4) "A1:A4"),(Col-Text @("a","b","c","d") "B1:B4")) }

    # --- positional reference ---
    @{ N="OFFSET"; Cat="Lookup and reference functions"; Tag="scalar_offset"; F="=OFFSET(A1,1,1)";
       Args=@((New-RefValue "A1" "A1"),(New-NumberValue 1),(New-NumberValue 1)); Fix=@((Grid-KeyText)) }
    @{ N="CHOOSE"; Cat="Lookup and reference functions"; Tag="scalar_values"; F="=CHOOSE(2,10,20,30)";
       Args=@((New-NumberValue 2),(New-NumberValue 10),(New-NumberValue 20),(New-NumberValue 30)); Fix=@() }

    # --- aggregation over reference ---
    @{ N="AGGREGATE"; Cat="Math and trigonometry functions"; Tag="sum_ref"; F="=AGGREGATE(9,0,A1:A4)";
       Args=@((New-NumberValue 9),(New-NumberValue 0),(New-RefValue "Area" "A1:A4")); Fix=@((Col-Nums @(1,2,3,4) "A1:A4")) }
    @{ N="SUBTOTAL"; Cat="Math and trigonometry functions"; Tag="sum_ref"; F="=SUBTOTAL(9,A1:A4)";
       Args=@((New-NumberValue 9),(New-RefValue "Area" "A1:A4")); Fix=@((Col-Nums @(1,2,3,4) "A1:A4")) }

    # --- cell / position info ---
    @{ N="CELL"; Cat="Information functions"; Tag="row_info"; F="=CELL(""row"",A1)";
       Args=@((New-TextValue "row"),(New-RefValue "A1" "A1")); Fix=@((New-Fixture "A1" (New-NumberValue 7))) }
    @{ N="COLUMN"; Cat="Lookup and reference functions"; Tag="ref_column"; F="=COLUMN(A1)";
       Args=@((New-RefValue "A1" "A1")); Fix=@((New-Fixture "A1" (New-NumberValue 7))) }
    @{ N="FORMULATEXT"; Cat="Lookup and reference functions"; Tag="value_cell_na"; F="=FORMULATEXT(A1)";
       Args=@((New-RefValue "A1" "A1")); Fix=@((New-Fixture "A1" (New-NumberValue 7))) }
    @{ N="ISFORMULA"; Cat="Information functions"; Tag="value_cell_false"; F="=ISFORMULA(A1)";
       Args=@((New-RefValue "A1" "A1")); Fix=@((New-Fixture "A1" (New-NumberValue 7))) }
    @{ N="SHEET"; Cat="Information functions"; Tag="ref_sheet"; F="=SHEET(A1)";
       Args=@((New-RefValue "A1" "A1")); Fix=@((New-Fixture "A1" (New-NumberValue 7))) }
    @{ N="SHEETS"; Cat="Information functions"; Tag="ref_sheets"; F="=SHEETS(A1:B2)";
       Args=@((New-RefValue "Area" "A1:B2")); Fix=@((New-Fixture "A1:B2" (New-ArrayValueFromCellRows @(@((New-NumberValue 1),(New-NumberValue 2)),@((New-NumberValue 3),(New-NumberValue 4)))))) }

    # --- IFNA (value-or-reference; scalar probes) ---
    @{ N="IFNA"; Cat="Logical functions"; Tag="na_branch"; F="=IFNA(NA(),5)";
       Args=@((New-ErrorValue "NA"),(New-NumberValue 5)); Fix=@() }
    @{ N="IFNA"; Cat="Logical functions"; Tag="passthrough_branch"; F="=IFNA(3,5)";
       Args=@((New-NumberValue 3),(New-NumberValue 5)); Fix=@() }

    # --- ranking over a reference ---
    @{ N="RANK"; Cat="Statistical functions"; Tag="descending_default"; F="=RANK(3,A1:A4)";
       Args=@((New-NumberValue 3),(New-RefValue "Area" "A1:A4")); Fix=@((Col-Nums @(1,2,3,4) "A1:A4")) }
    @{ N="RANK.EQ"; Cat="Statistical functions"; Tag="descending_default"; F="=RANK.EQ(3,A1:A4)";
       Args=@((New-NumberValue 3),(New-RefValue "Area" "A1:A4")); Fix=@((Col-Nums @(1,2,3,4) "A1:A4")) }
    @{ N="RANK.AVG"; Cat="Statistical functions"; Tag="descending_default"; F="=RANK.AVG(3,A1:A4)";
       Args=@((New-NumberValue 3),(New-RefValue "Area" "A1:A4")); Fix=@((Col-Nums @(1,2,3,4) "A1:A4")) }

    # --- regression / distribution over reference ---
    @{ N="FORECAST"; Cat="Statistical functions"; Tag="linear_fit"; F="=FORECAST(5,A1:A4,B1:B4)";
       Args=@((New-NumberValue 5),(New-RefValue "Area" "A1:A4"),(New-RefValue "Area" "B1:B4")); Fix=@((Col-Nums @(2,4,6,8) "A1:A4"),(Col-Nums @(1,2,3,4) "B1:B4")) }
    @{ N="FORECAST.LINEAR"; Cat="Statistical functions"; Tag="linear_fit"; F="=FORECAST.LINEAR(5,A1:A4,B1:B4)";
       Args=@((New-NumberValue 5),(New-RefValue "Area" "A1:A4"),(New-RefValue "Area" "B1:B4")); Fix=@((Col-Nums @(2,4,6,8) "A1:A4"),(Col-Nums @(1,2,3,4) "B1:B4")) }
    @{ N="FREQUENCY"; Cat="Statistical functions"; Tag="bins"; F="=FREQUENCY(A1:A4,B1:B2)";
       Args=@((New-RefValue "Area" "A1:A4"),(New-RefValue "Area" "B1:B2")); Fix=@((Col-Nums @(1,2,3,4) "A1:A4"),(Col-Nums @(2,4) "B1:B2")) }
    @{ N="PROB"; Cat="Statistical functions"; Tag="range_prob"; F="=PROB(A1:A4,B1:B4,2,3)";
       Args=@((New-RefValue "Area" "A1:A4"),(New-RefValue "Area" "B1:B4"),(New-NumberValue 2),(New-NumberValue 3)); Fix=@((Col-Nums @(1,2,3,4) "A1:A4"),(Col-Nums @(0.1,0.2,0.3,0.4) "B1:B4")) }
)

$cases = New-Object 'System.Collections.Generic.List[object]'
$seq = 0
foreach ($spec in $specs) {
    $seq += 1
    $caseId = ("reffix-{0:D5}-{1}-{2}" -f $seq, ($spec.N.ToLower() -replace '[^a-z0-9]','_'), $spec.Tag)
    $case = [ordered]@{
        schema_version = "oxfunc.smart_fuzzer.scenario_seed_case.v0"
        run_id = "assigned_by_runner"
        tranche_id = $TrancheId
        case_id = $caseId
        function_id = "FUNC.$($spec.N)"
        canonical_surface_name = $spec.N
        case_tag = "reference_fixture_probe:$($spec.Tag)"
        axis = "reference_fixture_probe"
        expected_probe_class = "reference_value_call"
        formula_text = $spec.F
        args = @($spec.Args)
        cell_fixture = @($spec.Fix)
        formula_cell = $null
        category = $spec.Cat
        blocked_or_deferred_lanes = @()
        known_deviation_tags = @()
    }
    $cases.Add($case) | Out-Null
}

$surfaces = (@($cases | ForEach-Object { $_.canonical_surface_name } | Sort-Object -Unique))
$out = [ordered]@{
    schema_version = "oxfunc.smart_fuzzer.scenario_seed_case_set.v0"
    authority = "non_semantic_exploration_input"
    generated_utc = (Get-Date).ToUniversalTime().ToString("o")
    generator = "Build-ReferenceFixtureProbes.ps1"
    tranche_id = $TrancheId
    comparison_policy = "exact_typed_bit_match_no_tolerance"
    cases = $cases.ToArray()
    tranches = @([ordered]@{ tranche_id = $TrancheId; case_ids = @($cases | ForEach-Object { $_.case_id }) })
    skipped = @()
    summary = [ordered]@{
        case_count = $cases.Count
        surfaces_covered = $surfaces.Count
    }
}
$out | ConvertTo-Json -Depth 14 | Set-Content -LiteralPath $OutputPath -Encoding UTF8

Write-Host "Cases:    $($cases.Count)"
Write-Host "Surfaces: $($surfaces.Count)  ($($surfaces -join ', '))"
Write-Host "Output:   $OutputPath"

# Function Slice Contract (Prelim) - Operator Compare/Concat Family

Workset: `W45`

## 1. Scope
This slice covers the non-`@` scalar operator rows:
1. `OP_CONCAT`
2. `OP_EQUAL`
3. `OP_NOT_EQUAL`
4. `OP_LESS_THAN`
5. `OP_LESS_EQUAL`
6. `OP_GREATER_THAN`
7. `OP_GREATER_EQUAL`

## 2. Admitted Current-Baseline Slice
1. all rows are binary operator surfaces with exact arity `2`,
2. operands use `values_only_pre_adapter`,
3. `OP_CONCAT` coerces operands to text and concatenates them in order,
4. comparison rows operate on scalar prepared values and return logical results,
5. comparison text lanes are case-insensitive on the admitted slice,
6. direct mixed-type comparisons on the admitted slice follow empirically observed Excel type ordering rather than numeric-text coercion,
7. blank-cell comparisons are admitted and follow the currently pinned context-sensitive blank coercion lanes:
   - blank vs number -> `0`
   - blank vs text -> `""`
   - blank vs logical -> `FALSE`

## 3. Out Of Slice
1. array-lifted comparison operator semantics,
2. spill-sensitive comparison publication,
3. locale/collation-sensitive text ordering beyond the current installed baseline,
4. parser/token precedence ownership questions.

## 4. Current Baseline Findings
1. `="a"&1 -> "a1"`
2. `=1&TRUE -> "1TRUE"`
3. `="a"="A" -> TRUE`
4. `=1="1" -> FALSE`
5. `=FALSE<TRUE -> TRUE`
6. `="10">2 -> TRUE`
7. blank direct cell compared to `0` yields `TRUE`
8. blank direct cell compared to `""` yields `TRUE`
9. `TRUE > 0 -> TRUE`

## 5. Artifacts
1. runtime module: `crates/oxfunc_core/src/functions/operator_compare_concat_family.rs`
2. native manifest: `docs/function-lane/W45_WAVEB_OPERATOR_COMPARE_CONCAT_SCENARIO_MANIFEST_SEED.csv`
3. runtime requirements: `docs/function-lane/W45_WAVEB_RUNTIME_REQUIREMENTS.md`
4. probe runner: `tools/w45-probe/run-w45-waveb-operator-compare-concat-baseline.ps1`
5. execution record: `docs/function-lane/W45_EXECUTION_RECORD.md`

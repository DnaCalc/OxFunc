# Function Slice Contract (Prelim) - Operator Reference Family

Workset: `W45`

## 1. Scope
This slice covers the non-`@` reference-composition rows:
1. `OP_RANGE_REF`
2. `OP_INTERSECTION_REF`
3. `OP_UNION_REF`
4. `OP_SPILL_REF`
5. `OP_TRIM_REF_LEADING`
6. `OP_TRIM_REF_TRAILING`
7. `OP_TRIM_REF_BOTH`

## 2. Admitted Current-Baseline Slice
1. `OP_RANGE_REF`, `OP_INTERSECTION_REF`, and `OP_UNION_REF` operate on reference-visible operands,
2. `OP_RANGE_REF` and `OP_INTERSECTION_REF` currently admit A1/area/whole-row/whole-column operands that parse through the local A1 reference substrate,
3. `OP_UNION_REF` currently admits structural multi-area target formation through the existing parenthesized target representation already consumed by `INDEX`,
4. `OP_TRIM_REF_*` currently admit structural reference-target normalization only,
5. `OP_SPILL_REF` remains the explicit spill-anchor operator slice already owned in the function lane and is reconciled into `W45` as part of the reference family.

## 3. Out Of Slice
1. `@` / implicit intersection,
2. fully general multi-area resolver materialization,
3. mixed-prefix range/intersection composition,
4. CSE publication and scalarization interaction,
5. parser-only whitespace ownership beyond the admitted trim-normalization slice.

## 4. Current Baseline Findings
1. `SUM((A1:B2)) -> 10`
2. `SUM((B2:A1)) -> 10`
3. `SUM((A1:C3 B2:D4)) -> 13`
4. `ROWS((A1:C3 B2:D4)) -> 2`
5. no-overlap intersection yields `#NULL!`
6. `SUM((A1,B1)) -> 4`
7. `INDEX((A1:A2,B1:B2),2,1,2) -> 4`
8. whitespace-trimmed reference forms such as `SUM(( A1 ))` remain transparent on the seeded slice

## 5. Artifacts
1. runtime modules:
   - `crates/oxfunc_core/src/functions/operator_reference_family.rs`
   - `crates/oxfunc_core/src/functions/op_spill_ref.rs`
2. native manifest: `docs/function-lane/W45_WAVEC_OPERATOR_REFERENCE_SCENARIO_MANIFEST_SEED.csv`
3. runtime requirements: `docs/function-lane/W45_WAVEC_RUNTIME_REQUIREMENTS.md`
4. probe runner: `tools/w45-probe/run-w45-wavec-operator-reference-baseline.ps1`
5. execution record: `docs/function-lane/W45_EXECUTION_RECORD.md`

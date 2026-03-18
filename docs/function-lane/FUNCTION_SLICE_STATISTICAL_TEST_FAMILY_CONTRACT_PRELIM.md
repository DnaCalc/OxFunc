# Function Slice Contract (Preliminary) - Statistical Test Family

Status: `provisional`
Workset: `W24`
Primary Functions: `CHISQ.TEST`, `CHITEST`, `F.TEST`, `FTEST`, `T.TEST`, `TTEST`, `ZTEST`

## 1. Scope
1. close the admitted current-baseline slice for the statistical test family and legacy aliases,
2. bind the integrated runtime and Lean substrate to a replayable native worksheet packet,
3. replace the old incorrect exact-shape doctrine for `CHISQ.TEST` with the empirically pinned equal-cardinality reshape rule.

## 2. Admitted Current-Baseline Slice
1. `CHISQ.TEST`, `CHITEST`
   - numeric scalar-or-array inputs,
   - `#N/A` on the degenerate `1x1` lane,
   - equal data-point count required,
   - second argument reshaped row-major to the first argument's layout when cardinalities match.
2. `F.TEST`, `FTEST`
   - numeric samples,
   - text, logical, and empty survivors ignored in array/reference inputs,
   - worksheet errors propagated from the sample arrays.
3. `T.TEST`, `TTEST`
   - `tails ∈ {1,2}`,
   - `type ∈ {1,2,3}`,
   - paired test with equal expanded cardinality and pairwise numeric filtering,
   - worksheet errors propagated from the sample arrays.
4. `ZTEST`
   - legacy alias for `Z.TEST`,
   - delegates to the modern `Z.TEST` semantics already pinned in `W24` Batch 05.

## 3. Metadata Shape
1. determinism: `deterministic`
2. volatility: `nonvolatile`
3. host_interaction: `none`
4. thread_safety: `safe_pure`
5. arg_preparation_profile: `refs_visible_in_adapter`
6. coercion_lift_profile: `custom`
7. fec_dependency_profile: `ref_only`
8. surface_fec_dependency_profile: `ref_only`

## 4. Evidence Basis
1. Rust runtime kernel and unit tests in `crates/oxfunc_core/src/functions/statistical_tests_family.rs`
2. Legacy `ZTEST` alias shim in `crates/oxfunc_core/src/functions/test_alias_family.rs`
3. Lean metadata/bindings in `formal/lean/OxFunc/Functions/StatisticalTestsFamily.lean` and `formal/lean/OxFunc/Functions/TestAliasFamily.lean`
4. Native worksheet packet in `docs/function-lane/W24_BATCH07_STATISTICAL_TESTS_SCENARIO_MANIFEST_SEED.csv`
5. Runtime harness in `tools/w24-probe/run-w24-batch07-statistical-tests-baseline.ps1`
6. Packet execution record in `docs/function-lane/W24_BATCH07_STATISTICAL_TESTS_EXECUTION_RECORD.md`

## 5. Scope Boundary
1. The closure is bounded to the admitted current-baseline statistical-test slice.
2. Broader statistical-family harmonization remains outside this packet.

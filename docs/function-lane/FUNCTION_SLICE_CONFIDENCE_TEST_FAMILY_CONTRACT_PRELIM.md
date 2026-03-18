# Function Slice Contract (Preliminary) - Confidence/Test Helper Family

Status: `provisional`
Workset: `W24`
Primary Functions: `CONFIDENCE.T`, `Z.TEST`

## 1. Scope
1. close the admitted current-baseline slice for `CONFIDENCE.T` and `Z.TEST`,
2. bind the integrated runtime and Lean substrate to a replayable native worksheet packet,
3. make the scalar and survivor-policy boundaries explicit.

## 2. Admitted Current-Baseline Slice
1. `CONFIDENCE.T`
   - scalar numeric alpha, standard deviation, and size,
   - direct dependence on the existing `T.INV.2T` substrate,
   - invalid alpha, sigma, and size domain mapped to `#NUM!`.
2. `Z.TEST`
   - numeric sample values from array/range input,
   - optional supplied sigma or default sample-standard-deviation path,
   - text, logical, and blank survivors ignored,
   - error cells in the sample array propagated.

## 3. Semantics
1. `CONFIDENCE.T(alpha, sigma, size)` computes the two-tailed Student-t confidence interval width.
2. `Z.TEST(array, x, [sigma])` computes the one-tailed normal probability from the sample mean.
3. `Z.TEST` ignores non-numeric non-error survivors in the sample array for the admitted slice.
4. `Z.TEST` propagates an error survivor from the sample array.

## 4. Metadata Shape
1. determinism: `deterministic`
2. volatility: `nonvolatile`
3. host_interaction: `none`
4. thread_safety: `safe_pure`
5. `CONFIDENCE.T` arg_preparation_profile: `values_only_pre_adapter`
6. `Z.TEST` arg_preparation_profile: `refs_visible_in_adapter`
7. `Z.TEST` fec_dependency_profile: `ref_only`
8. `Z.TEST` surface_fec_dependency_profile: `ref_only`

## 5. Evidence Basis
1. Rust runtime kernel and unit tests in `crates/oxfunc_core/src/functions/confidence_test_family.rs`
2. Lean metadata/binding in `formal/lean/OxFunc/Functions/ConfidenceTestFamily.lean`
3. Native worksheet packet in `docs/function-lane/W24_BATCH05_CONFIDENCE_TEST_SCENARIO_MANIFEST_SEED.csv`
4. Runtime harness in `tools/w24-probe/run-w24-batch05-confidence-test-baseline.ps1`
5. Packet execution record in `docs/function-lane/W24_BATCH05_CONFIDENCE_TEST_EXECUTION_RECORD.md`

## 6. Scope Boundary
1. The closure is bounded to the observed scalar and survivor-policy slice.
2. Broader statistical-family harmonization remains outside this packet.

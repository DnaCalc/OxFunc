# Function Slice Contract (Preliminary) - Information Predicates Family

Status: `provisional`
Workset: `W33`
Primary Functions: `ISBLANK`, `ISERR`, `ISERROR`, `ISLOGICAL`, `ISNA`, `ISNONTEXT`, `ISODD`, `ISREF`, `ISTEXT`

## 1. Scope
1. close the admitted current-baseline worksheet slice for the nine newly promoted information predicates,
2. keep the family split explicit between values-only predicates and the reference-visible `ISREF` lane,
3. bind the family to a replayable native worksheet packet and Lean metadata/binding alignment.

## 2. Admitted Current-Baseline Slice
1. `ISBLANK`
   - returns `TRUE` only for a true blank/empty-cell payload,
   - returns `FALSE` for empty-string literals and formulas returning `""`.
2. `ISERR`
   - returns `TRUE` for worksheet errors except `#N/A`,
   - returns `FALSE` for `#N/A` and non-error values.
3. `ISERROR`
   - returns `TRUE` for any worksheet error,
   - returns `FALSE` for non-error values.
4. `ISLOGICAL`
   - returns `TRUE` only for logical payloads.
5. `ISNA`
   - returns `TRUE` only for the `#N/A` error.
6. `ISNONTEXT`
   - returns `FALSE` only for text payloads,
   - blank, numeric, logical, and error payloads are `TRUE`.
7. `ISTEXT`
   - returns `TRUE` only for text payloads.
8. `ISODD`
   - follows the `ISEVEN`-style scalar coercion seam in the admitted slice,
   - numeric text is coerced,
   - direct logical arguments are rejected with `#VALUE!`,
   - blank-like payloads map to numeric `0` before parity and therefore yield `FALSE`.
9. `ISREF`
   - returns `TRUE` for direct reference arguments and reference-returning function results,
   - returns `FALSE` for scalar and array-value results such as `HSTACK(...)`,
   - does not require dereferencing to determine truth in the admitted slice.

## 3. Explicit Family Split
1. `ISBLANK`, `ISERR`, `ISERROR`, `ISLOGICAL`, `ISNA`, `ISNONTEXT`, `ISODD`, and `ISTEXT`
   - `arg_preparation_profile = values_only_pre_adapter`
2. `ISREF`
   - `arg_preparation_profile = refs_visible_in_adapter`

## 4. Metadata Shape
1. determinism: `deterministic`
2. volatility: `nonvolatile`
3. host_interaction: `none`
4. thread_safety: `safe_pure`
5. coercion_lift_profile:
   - `custom` for `ISODD`
   - `none` for the remaining members
6. fec dependency:
   - `none` / `ref_only` for the values-only members
   - `ref_only` / `ref_only` for `ISREF`

## 5. Explicitly Out Of Slice
1. locale and version sweeps beyond the current reference baseline,
2. caller-context-sensitive multi-cell reference scalarization outside the admitted direct reference lanes,
3. host metadata query semantics such as `ISFORMULA`, which remain in `W023`.

## 6. Evidence Basis
1. Rust runtime/tests: `crates/oxfunc_core/src/functions/is_predicates_family.rs`
2. Lean metadata/binding: `formal/lean/OxFunc/Functions/IsPredicatesFamily.lean`
3. native packet: `docs/function-lane/W33_SCENARIO_MANIFEST_SEED.csv`
4. runtime harness: `tools/w33-probe/run-w33-info-forecast-baseline.ps1`
5. packet execution record: `docs/function-lane/W33_EXECUTION_RECORD.md`

# Function Slice Contract (Preliminary) - Functional Lambda Helpers Stage 2 (`MAP`, `REDUCE`, `SCAN`)

Status: `provisional`
Workset: `W38`
Primary Functions: `MAP`, `REDUCE`, `SCAN`

## 1. Scope
1. extend `W38` beyond direct `LET` / `LAMBDA` lanes into the first higher-order helper slice,
2. pin the admitted current-baseline worksheet behavior of `MAP`, `REDUCE`, and `SCAN` over array-constant inputs,
3. keep `BYROW`, `BYCOL`, and `MAKEARRAY` for later `W38` phases.

## 2. Admitted Current-Baseline Stage 2 Slice
1. `MAP`
   - applies the supplied lambda elementwise over one or more arrays,
   - spills a result array on the worksheet surface in the admitted slice,
   - for mismatched array lengths in the seeded row-vector lane, missing partner elements materialize as `#N/A` in the result array rather than forcing whole-formula failure,
   - lambda arity mismatch returns `#VALUE!` in the admitted runtime-mismatch lane,
   - malformed lambda declaration with an extra body argument is rejected at formula admission.
2. `REDUCE`
   - folds the input array into one accumulated scalar,
   - uses the supplied initial accumulator in the admitted slice,
   - lambda arity mismatch returns `#VALUE!`.
3. `SCAN`
   - returns the intermediate accumulated results as a spilled array,
   - in the admitted slice, the worksheet-visible result does not include the initial accumulator as a separate leading output element,
   - lambda arity mismatch returns `#VALUE!`.

## 3. Stage 2 Seams Made Explicit
1. These helpers prove that callable/helper semantics are not just about direct invocation; they shape array publication semantics as well.
2. The packet still does not require the final callable carrier to be locked, but it does require a future OxFunc callable substrate that can support higher-order helper invocation.
3. The seeded evidence suggests a three-part split:
   - helper admission and lambda-shape validation,
   - helper-driven callable invocation across array elements,
   - worksheet spill/publication of the helper result.
4. `ISOMITTED` is still only observed in the "present argument" direction in the admitted Stage 2 slice.

## 4. Metadata Shape
1. determinism: `deterministic`
2. volatility: `nonvolatile`
3. host_interaction: `none`
4. fec dependency:
   - `admission_and_prepared_call_sensitive`
   - `array_publication_sensitive`
5. callable/value-universe dependency:
   - `required`

## 5. Explicitly Out Of Stage 2 Slice
1. `BYROW`
2. `BYCOL`
3. `MAKEARRAY`
4. defined-name lambdas,
5. user-defined or interop-provided callable values,
6. final cross-repo callable carrier lock.

## 6. Evidence Basis
1. native packet:
   - `docs/function-lane/W38_STAGE2_MAP_REDUCE_SCAN_SCENARIO_MANIFEST_SEED.csv`
   - `.tmp/w38-map-reduce-scan-stage2-results.csv`
2. runtime harness:
   - `tools/w38-probe/run-w38-map-reduce-scan-stage2-baseline.ps1`
3. packet execution record:
   - `docs/function-lane/W38_EXECUTION_RECORD.md`

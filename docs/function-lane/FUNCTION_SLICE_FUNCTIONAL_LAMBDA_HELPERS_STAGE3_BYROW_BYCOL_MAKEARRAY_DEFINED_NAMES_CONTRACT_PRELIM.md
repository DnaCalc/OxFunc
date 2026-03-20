# Function Slice Contract (Preliminary) - Functional Lambda Helpers Stage 3 (`BYROW`, `BYCOL`, `MAKEARRAY`, Defined Name callable preservation)

Status: `provisional`
Workset: `W38`
Primary Functions: `BYROW`, `BYCOL`, `MAKEARRAY`

## 1. Scope
1. extend `W38` beyond Stage 2 into the remaining inventory members:
   - `BYROW`
   - `BYCOL`
   - `MAKEARRAY`
2. characterize the admitted current-baseline worksheet behavior for workbook Defined Names whose values are callable lambdas,
3. keep fuller UDF/add-in/interoperable callable transport out of scope.

## 2. Admitted Current-Baseline Stage 3 Slice
1. `BYROW`
   - invokes the supplied lambda once per row of the source array,
   - publishes one result per source row,
   - requires the lambda body to produce a scalar worksheet result in the admitted slice,
   - returns `#CALC!` when the supplied lambda body produces a non-scalar row result in the seeded lane,
   - returns `#VALUE!` on runtime lambda-arity mismatch,
   - rejects malformed lambda declarations at formula admission.
2. `BYCOL`
   - invokes the supplied lambda once per column of the source array,
   - publishes one result per source column,
   - requires the lambda body to produce a scalar worksheet result in the admitted slice,
   - returns `#CALC!` when the supplied lambda body produces a non-scalar column result in the seeded lane,
   - returns `#VALUE!` on runtime lambda-arity mismatch,
   - rejects malformed lambda declarations at formula admission.
3. `MAKEARRAY`
   - invokes the supplied lambda over generated 1-based row and column coordinates,
   - spills a result array on the worksheet surface in the admitted slice,
   - returns `#VALUE!` on runtime lambda-arity mismatch,
   - observes both generated coordinate arguments as present rather than omitted in the seeded `ISOMITTED` lane.
4. Defined Name callable preservation
   - workbook Defined Names may preserve callable lambda values on the admitted current-baseline slice,
   - direct invocation through the Defined Name works,
   - higher-order helper use through the Defined Name works,
   - lexical capture preserved inside a Defined Name callable remains visible in the admitted slice,
   - bare publication of the Defined Name callable yields `#CALC!` rather than a worksheet-display callable value.

## 3. Stage 3 Seams Made Explicit
1. `BYROW` and `BYCOL` show that higher-order helper semantics include scalarity requirements on helper return shape, not just invocation.
2. `MAKEARRAY` shows that helper invocation can be driven by generated coordinate arguments rather than by existing source-array elements.
3. Defined Name callable preservation is part of the first-pass Excel-supported callable surface and should not be grouped with deferred UDF/interoperable callable transport.
4. The admitted Stage 3 slice still does not require a final shared callable carrier lock, but it strengthens the need for a future executable callable substrate in OxFunc core.

## 4. Metadata Shape
1. determinism: `deterministic`
2. volatility: `nonvolatile`
3. host_interaction: `none`
4. fec dependency:
   - `admission_and_prepared_call_sensitive`
   - `array_publication_sensitive`
   - `defined_name_resolution_sensitive`
5. callable/value-universe dependency:
   - `required`

## 5. Explicitly Out Of Stage 3 Slice
1. sheet-scoped name variants not yet empirically characterized,
2. UDF/add-in/interoperable callable origins or returns,
3. final cross-repo callable carrier lock,
4. final callable/provider-stage refinement beyond the generic staged model.

## 6. Evidence Basis
1. native packet:
   - `docs/function-lane/W38_STAGE3_BYROW_BYCOL_MAKEARRAY_DEFINED_NAMES_SCENARIO_MANIFEST_SEED.csv`
   - `.tmp/w38-stage3-byrow-bycol-makearray-defined-names-results.csv`
2. runtime harness:
   - `tools/w38-probe/run-w38-stage3-byrow-bycol-makearray-defined-names-baseline.ps1`
3. packet execution record:
   - `docs/function-lane/W38_EXECUTION_RECORD.md`

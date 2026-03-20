# Function Slice Contract (Preliminary) - Functional Lambda And Helper Family Stage 1

Status: `provisional`
Workset: `W38`
Primary Functions: `LET`, `LAMBDA`, `ISOMITTED`

## 1. Scope
1. open the helper/callable family with a first admitted current-baseline worksheet slice,
2. pin the parts of the family that are already observable without locking the full cross-repo callable carrier,
3. keep `BYROW`, `BYCOL`, `MAP`, `REDUCE`, `SCAN`, and `MAKEARRAY` explicit as later `W38` phases rather than pretending Stage 1 closes the whole family.

## 2. Admitted Current-Baseline Stage 1 Slice
1. `LET`
   - supports sequential local name binding over scalar and array values,
   - later names can reference earlier names,
   - nested `LET` expressions preserve access to outer bindings unless shadowed by a new inner binding,
   - duplicate local names are rejected at formula-admission time rather than being deferred to a runtime worksheet error.
2. `LAMBDA`
   - supports immediate invocation in ordinary worksheet formulas,
   - supports one- and multi-argument immediate invocation in the admitted slice,
   - preserves lexical capture of surrounding `LET` bindings in the admitted slice,
   - a bare uninvoked lambda evaluates to `#CALC!` on the worksheet surface,
   - arity mismatch on ordinary direct invocation returns `#VALUE!`,
   - duplicate parameter names are rejected at formula-admission time.
3. `ISOMITTED`
   - returns `FALSE` for present arguments in the admitted direct lambda lanes,
   - returns `FALSE` on the seeded top-level direct-call lane `ISOMITTED(1)`,
   - `ISOMITTED()` with no argument is rejected at formula-admission time,
   - ordinary direct lambda under-application does not create an omission channel; it fails with `#VALUE!` before any useful omitted-argument behavior is exposed.

## 3. Stage 1 Seams Made Explicit
1. Helper syntax, sequential binding, and lexical environment formation remain primarily OxFml-owned concerns.
2. OxFunc still needs a first-class callable value in the semantic value universe for the longer-term family target.
3. The current Stage 1 packet does not require the final callable transport/carrier to be locked.
4. The seeded evidence already shows that some helper/lambda behavior happens at formula-admission time, not purely at value-evaluation time:
   - duplicate `LET` names,
   - duplicate `LAMBDA` parameter names,
   - zero-argument `ISOMITTED()`.
5. The admitted Stage 1 slice therefore depends on an honest split between:
   - parse/admission behavior,
   - callable creation and immediate invocation,
   - worksheet-surface publication behavior (`#CALC!`, `#VALUE!`).

## 4. Metadata Shape
1. determinism: `deterministic`
2. volatility: `nonvolatile`
3. host_interaction: `none`
4. thread_safety:
   - not yet promoted for the full family,
   - Stage 1 is still treated as `custom/helper-seam-sensitive`
5. fec dependency:
   - `admission_and_prepared_call_sensitive`
6. callable/value-universe dependency:
   - `required`

## 5. Explicitly Out Of Stage 1 Slice
1. final callable carrier or ABI across the OxFml/OxFunc seam,
2. defined-name lambda values,
3. user-defined or interop-provided lambda values,
4. helper-driven omitted-argument semantics beyond the direct invocation lanes seeded here,
5. higher-order helper members:
   - `BYROW`
   - `BYCOL`
   - `MAP`
   - `REDUCE`
   - `SCAN`
   - `MAKEARRAY`

## 6. Evidence Basis
1. native packet:
   - `docs/function-lane/W38_SCENARIO_MANIFEST_SEED.csv`
   - `.tmp/w38-lambda-helper-stage1-results.csv`
2. runtime harness:
   - `tools/w38-probe/run-w38-lambda-helper-stage1-baseline.ps1`
3. packet execution record:
   - `docs/function-lane/W38_EXECUTION_RECORD.md`
4. seam notes:
   - `docs/function-lane/OXFML_OXFUNC_HIDDEN_MACHINERY_SEAM_EXPLICIT_MODEL.md`
   - `docs/function-lane/OXFML_OXFUNC_NEXT_ROUND_STABILIZATION_TOPICS.md`

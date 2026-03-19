# WORKSET - Information Predicates And Forecast Compatibility Closure (W33)

## 1. Purpose
Close the newly exposed ordinary-function catalog gaps that became explicit after `W28` promoted the corrected local current-baseline function catalog.

This packet targets a small, likely-closeable family:
1. missing `IS*` information predicates, and
2. the `FORECAST` / `FORECAST.LINEAR` compatibility pair.

## 2. Provenance
Opened after `W28` resolved the support-surface anomaly set against live Excel and promoted:
1. `docs/function-lane/FUNCTION_CATALOG_CURRENT_BASELINE_LOCAL.csv`
2. `docs/function-lane/W28_FUNCTION_NAME_EXISTENCE_PROBE_RESULTS.csv`

Relevant supporting artifacts:
1. `docs/function-lane/W28_EXECUTION_RECORD.md`
2. `docs/function-lane/W24_BATCH09_REGRESSION_FORECAST_EXECUTION_RECORD.md`
3. `docs/function-lane/FUNCTION_SLICE_REGRESSION_FORECAST_FAMILY_CONTRACT_PRELIM.md`

## 3. Scope
Machine-readable inventory:
1. `docs/function-lane/W33_INFORMATION_PREDICATES_AND_FORECAST_INVENTORY.csv`

Current total:
1. `11` functions.

Members:
1. `FORECAST`
2. `FORECAST.LINEAR`
3. `ISBLANK`
4. `ISERR`
5. `ISERROR`
6. `ISLOGICAL`
7. `ISNA`
8. `ISNONTEXT`
9. `ISODD`
10. `ISREF`
11. `ISTEXT`

## 4. Why These Look Closeable
1. Live Excel existence probing on `2026-03-19` proved all eleven names are real worksheet functions on the installed baseline.
2. The `IS*` group is a coherent information-predicate family with likely shared coercion/reference-policy seams.
3. `FORECAST` and `FORECAST.LINEAR` are now explicit catalog members, and the support surface indicates they share one naming-history article.
4. This packet is narrower and more executable than the host-sensitive, provider-sensitive, or reopened finance packets.

## 5. In Scope
1. empirical worksheet characterization of the admitted current-baseline semantics for the eleven functions,
2. runtime implementation and dispatch/export wiring where missing,
3. Lean/formal alignment for the family substrate,
4. contract, conformance, runtime-requirements, scenario-manifest, execution-record, and evidence-registry artifacts,
5. explicit handling of whether `FORECAST` is:
   - a semantic alias of `FORECAST.LINEAR`,
   - or a distinct compatibility surface with separate observable behavior.

## 6. Out Of Scope
1. broad regression/forecast-family expansion beyond the forecast pair,
2. host-sensitive `ISFORMULA` or database/query functions from `W023`,
3. locale/profile/provider-sensitive packets (`W030`, `W031`),
4. reopened finance repair work in `W032`.

## 7. Gate Criteria
This workset can only be reported `scope_complete` when:
1. all eleven functions have reproducible native worksheet evidence,
2. the `IS*` family has an explicit shared contract/admission statement,
3. `FORECAST` vs `FORECAST.LINEAR` is pinned empirically and reflected honestly in the contract/conformance surfaces,
4. runtime, export, and Lean/binding artifacts are integrated,
5. no known function-semantic gap remains in declared current-baseline scope for these eleven members.

## 8. Final Status
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`
5. open_lanes:
   - none in declared `W33` scope

## 9. Closure Result
1. The nine information predicates are now integrated on the admitted current-baseline slice:
   - values-only members: `ISBLANK`, `ISERR`, `ISERROR`, `ISLOGICAL`, `ISNA`, `ISNONTEXT`, `ISODD`, `ISTEXT`
   - ref-visible member: `ISREF`
2. `FORECAST` and `FORECAST.LINEAR` are now pinned as semantically identical on the admitted current-baseline scalar/vector slice, with `FORECAST` carried as a compatibility surface rather than an independently divergent kernel.
3. The packet evidence is recorded in:
   - `docs/function-lane/W33_EXECUTION_RECORD.md`
   - `.tmp/w33-info-forecast-results.csv`

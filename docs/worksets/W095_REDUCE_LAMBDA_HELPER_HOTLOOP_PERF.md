# W095 REDUCE Lambda-Helper Hot-Loop Performance

Status: `stopped_at_oxfunc_local_gate`

## 1. Purpose

Process DnaOneCalc `HANDOFF_OXFUNC_REDUCE_HOTLOOP_PERF.md` by reducing avoidable OxFunc-side allocation pressure in REDUCE / lambda-helper hot loops while preserving Excel-visible helper semantics.

## 2. Problem Statement

The upstream Mandelbrot probe exercises a small lambda body many times through `REDUCE(SEQUENCE(...), LAMBDA(...))`. The arithmetic is cheap, but the helper path currently pays repeated costs for:

1. full upfront iterable materialization,
2. per-cell `PreparedArgValue` construction before invocation,
3. 1x3 accumulator row allocation through `HSTACK`,
4. downstream OxFml callable invocation overhead.

## 3. Scope

In scope:

1. lazy iterable consumption in OxFunc lambda helpers,
2. numeric-scalar iterable specialization where it does not change semantics,
3. small-row array allocation reduction if the value-type representation can be changed safely,
4. perf replay hooks and evidence against the DnaOneCalc Mandelbrot probe,
5. documentation of downstream OxFml invocation-cache dependency.

Out of scope:

1. changing Excel REDUCE semantics,
2. host editor debounce/runtime split,
3. OxFml expression evaluation caching,
4. non-parity early-exit behavior unless explicitly split into an opt-in helper.

## 4. OxFunc-Local Slices

The initial W095 slice replaces helper-side full iterable materialization with `PreparedIterableSource`, so `MAP`, `REDUCE`, and `SCAN` consume one prepared item at a time. It also avoids cloning a full source `EvalArray` in `BYROW` and `BYCOL` when the argument is already an array.

The follow-on W095 slice adds:

1. a numeric-array fast path for `REDUCE`,
2. borrowed/inline argument sources for `HSTACK`,
3. borrowed/inline argument sources for `VSTACK`.

`REDUCE.STOP` and other early-exit sentinels are not part of this W095 pass because they are non-parity extension behavior.

The broader `EvalArray` slice adds inline storage for arrays with up to `8` cells and an iterator constructor for callers that can produce cells without first allocating a flat `Vec`. The W095 hot paths now use that constructor for BYROW row-vector arguments, BYCOL column-vector arguments, HSTACK outputs, and VSTACK outputs.

The callable batching slice adds the OxFml-facing setup-hoisting seam:

1. `CallableBatchMode::SequentialStateful` for REDUCE and SCAN accumulator loops,
2. `CallableBatchMode::Independent` for MAP, BYROW, BYCOL, and MAKEARRAY repeated calls,
3. `CallableInvocationBatch` as the stateful argument/result exchange object,
4. `CallableInvoker::invoke_many(...)` as the overridable method with a default fallback through existing `invoke(...)`,
5. focused tests proving REDUCE and SCAN use sequential-stateful batching and MAP/BYROW/BYCOL/MAKEARRAY use independent batching.

The sequential-stateful mode is explicitly ordered. It is intended to let OxFml hoist callable setup, not to parallelize or reorder REDUCE/SCAN calls.

## 5. Validation Evidence

1. `cargo fmt --manifest-path crates\oxfunc_core\Cargo.toml`: passed.
2. `cargo test --manifest-path crates\oxfunc_core\Cargo.toml --lib callable_helpers`: passed, `29` passed, `0` failed, `1246` filtered out.
3. `cargo test --manifest-path crates\oxfunc_core\Cargo.toml --lib hstack`: passed, `3` passed, `0` failed, `1269` filtered out.
4. `cargo test --manifest-path crates\oxfunc_core\Cargo.toml --lib vstack`: passed, `1` passed, `0` failed, `1271` filtered out.
5. `cargo test --release -p dnaonecalc-host --test mandelbrot_perf_probe -- --ignored --nocapture`: passed, `2` passed, `0` failed.
6. Host-visible Mandelbrot replay after the inline `EvalArray` storage slice improved from the earlier W095 replay: full-size `100x60x30` end-to-end printed `5.7998668s`, and the probe also printed a `100x60x30` row with `5.5361486s` / `30.76us` per inner iteration.

## 6. Perf Interpretation

The current OxFunc-local slices are validated and improved the host-visible replay, but W095 is still target-partial because sibling-repo invocation caching and post-specialization replay remain outside this repo.

1. OxFml callable invocation / expression evaluation per REDUCE step,
2. any remaining ordinary `HSTACK` result-array construction costs after inline storage,
3. host-side editor debounce/runtime split outside OxFunc.

## 7. Tracking

1. Inbound handoff: `docs/handoffs/HANDOFF-OXFUNC-007_reduce_hotloop_perf.md`
2. Upstream source: `../DnaOneCalc/docs/HANDOFF_OXFUNC_REDUCE_HOTLOOP_PERF.md`
3. Related sibling handoff: `../DnaOneCalc/docs/HANDOFF_OXFML_LAMBDA_INVOCATION_PERF.md`
4. Downstream OxFunc-to-OxFml handoff: `docs/handoffs/HO-FN-015_callable_batching_invocation_seam.md`
5. Bead: `oxf-goj3`

## 8. Reporting Contract

All W095 reports must include:

1. `execution_state`,
2. `scope_completeness`,
3. `target_completeness`,
4. `integration_completeness`,
5. explicit `open_lanes` while any axis remains partial.

Current status axes:

1. `scope_completeness`: `scope_partial`
2. `target_completeness`: `target_partial`
3. `integration_completeness`: `partial`
4. `open_lanes`: OxFml `invoke_many(...)` specialization through `HO-FN-015`, post-OxFml-specialization DnaOneCalc perf replay, and landed-ref promotion.

## 9. Pre-Closure Verification Checklist

W095 is stopped at the OxFunc-local gate, not reported as full cross-repo performance completion.

| # | Check | Yes/No | Result |
|---|-------|--------|--------|
| 1 | Function contract rows complete and promoted for all in-scope functions? | no | W095 is a helper/performance seam, not a function-contract promotion lane. |
| 2 | Lean obligations for each slice class satisfied or explicitly aligned per formalization strategy? | no | No Lean claim is made for this performance seam. |
| 3 | Rust implementation and required tests pass for all in-scope functions? | yes | Focused callable-helper, HSTACK, VSTACK, and DnaOneCalc perf-probe commands passed as recorded above. |
| 4 | At least one deterministic replay artifact exists per in-scope function behavior? | yes | Focused tests plus DnaOneCalc Mandelbrot replay evidence are recorded. |
| 5 | Evidence links complete and reproducible? | yes | Commands and target files are recorded in this workset and handoff docs. |
| 6 | Version scope explicit on both axes? | no | This is not an Excel semantic parity claim and does not change app/workbook version semantics. |
| 7 | Public-doc vs empirical discrepancies recorded and resolved in favor of empirical Excel behavior? | yes | No Excel semantic discrepancy is introduced; REDUCE.STOP was excluded. |
| 8 | XLL verification-seam limitations documented where material? | yes | Not material to this helper performance seam. |
| 9 | Cross-repo impact assessed and handoff filed if evaluator-facing clauses affected? | yes | `HO-FN-015` filed for OxFml `invoke_many(...)` specialization. |
| 10 | No known semantic gap remains in declared scope? | yes | Local helper semantics are covered by focused tests; performance target remains partial. |
| 11 | Completion language audit passed? | yes | W095 is reported as stopped/partial, not fully complete. |
| 12 | `docs/IN_PROGRESS_FEATURE_WORKLIST.md` updated? | yes | IP-22 updated. |
| 13 | Execution-state blocker surface updated? | yes | Bead `oxf-goj3` updated. |

## 10. Completion Claim Self-Audit

1. Scope re-read: pass for OxFunc-local W095 slices; full cross-repo performance target remains partial.
2. Gate criteria re-read: fail for full performance completion because OxFml specialization and post-specialization replay remain open.
3. Silent scope reduction check: pass; REDUCE.STOP is explicitly out of scope and downstream OxFml work is filed.
4. "Looks done but is not" pattern check: pass; this record keeps target/integration partial.
5. Result: W095 is stopped at the OxFunc-local gate with downstream handoff filed, not reported as full completion.

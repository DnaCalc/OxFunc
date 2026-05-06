# HANDOFF-OXFUNC-007 - REDUCE / Lambda-Helper Hot-Loop Performance

Status: `stopped_at_oxfunc_local_gate`
Direction: `DnaOneCalc -> OxFunc`
Source repo/workset: `DnaOneCalc / Performance investigation`
Target repo/workset: `OxFunc/W095`
Filed date: `2026-05-05`
Acknowledged date: `2026-05-06`
Upstream source: `../DnaOneCalc/docs/HANDOFF_OXFUNC_REDUCE_HOTLOOP_PERF.md`
Related downstream handoff: `../DnaOneCalc/docs/HANDOFF_OXFML_LAMBDA_INVOCATION_PERF.md`
OxFunc downstream handoff: `docs/handoffs/HO-FN-015_callable_batching_invocation_seam.md`
Tracking bead: `oxf-goj3`

## OxFunc acknowledgement

OxFunc acknowledges the DnaOneCalc report that REDUCE / lambda-helper workloads can spend most of their time in function-dispatch and small allocation machinery rather than arithmetic. The reported Mandelbrot-shaped workload is:

1. `REDUCE` over `SEQUENCE(maxIter)`,
2. a 1x3 row accumulator carrying `(x, y, n)`,
3. lambda-body destructuring through `INDEX`,
4. accumulator repacking through `HSTACK`.

The first OxFunc-local slice targets avoidable helper-side allocation without changing Excel-visible REDUCE semantics.

## OxFunc-local slices

W095 starts with ask A from the handoff:

1. replace full `Vec<PreparedArgValue>` materialization in lambda helper iteration with a one-item-at-a-time `PreparedIterableSource`,
2. keep `MAP`, `REDUCE`, and `SCAN` semantics while avoiding full upfront iterable materialization,
3. preserve shape/capacity hints from source arrays,
4. avoid cloning the full source array in `BYROW` and `BYCOL` when the input is already an array.

W095 also advances the requested REDUCE and H/VSTACK optimization lanes:

1. `REDUCE` uses a numeric-array fast path that bypasses the generic array-cell conversion branch for numeric-only iterables.
2. `HSTACK` uses borrowed/inline argument sources rather than cloning array arguments or materializing scalar arguments as temporary 1x1 arrays.
3. `VSTACK` uses the same borrowed/inline argument-source shape for stack assembly.
4. `REDUCE.STOP` or any other early-exit sentinel is explicitly out of scope for this pass.

W095 then advances the broader `EvalArray` storage lane:

1. `EvalArray` stores arrays with up to `8` cells inline.
2. `EvalArray::from_cells_iter(...)` allows callers to construct small arrays without first materializing a flat `Vec`.
3. BYROW row-vector and BYCOL column-vector helper arguments use the iterator constructor.
4. HSTACK and VSTACK stack outputs use the iterator constructor, so common small stack outputs such as `HSTACK(x, y, n)` can avoid the intermediate flat `Vec` and store inline.

W095 also adds the OxFml-facing callable batching seam:

1. `CallableBatchMode` distinguishes `Independent` from `SequentialStateful` helper batches.
2. `CallableInvocationBatch` lets a helper produce one argument slice, accept the result, and then produce the next slice.
3. `CallableInvoker::invoke_many(...)` has a default fallback that checks callable arity and calls existing `invoke(...)` for each prepared slice.
4. REDUCE and SCAN use `SequentialStateful` batches because each result affects later arguments.
5. MAP, BYROW, BYCOL, and MAKEARRAY use `Independent` batches.
6. The seam is for setup hoisting and cache reuse; it does not permit reordering REDUCE/SCAN calls or changing accumulator semantics.

## Deferred lanes

1. OxFml `invoke_many(...)` specialization through `HO-FN-015`,
2. post-OxFml-specialization DnaOneCalc perf replay,
3. landed-ref promotion.
4. `REDUCE.STOP` is parked out of scope rather than open for this W095 pass.

## Validation evidence

1. `cargo fmt --manifest-path crates\oxfunc_core\Cargo.toml`: passed.
2. `cargo test --manifest-path crates\oxfunc_core\Cargo.toml --lib callable_helpers`: passed, `29` passed, `0` failed, `1246` filtered out.
3. `cargo test --manifest-path crates\oxfunc_core\Cargo.toml --lib hstack`: passed, `3` passed, `0` failed, `1269` filtered out.
4. `cargo test --manifest-path crates\oxfunc_core\Cargo.toml --lib vstack`: passed, `1` passed, `0` failed, `1271` filtered out.
5. `cargo test --release -p dnaonecalc-host --test mandelbrot_perf_probe -- --ignored --nocapture`: passed, `2` passed, `0` failed.
6. The release perf probe emitted the existing Cargo PDB filename-collision warning for `dnaonecalc-host`.
7. Host-visible Mandelbrot replay after the inline `EvalArray` storage slice improved from the earlier W095 replay: full-size `100x60x30` end-to-end printed `5.7998668s`, and the probe also printed a `100x60x30` row with `5.5361486s` / `30.76us` per inner iteration.

## Perf interpretation

The W095 local slices remove avoidable helper-side materialization and stack-input/result allocation, and the DnaOneCalc release probe shows a material improvement after inline `EvalArray` storage. Remaining dominant lanes are likely:

1. OxFml callable invocation / expression evaluation per REDUCE step,
2. remaining ordinary `HSTACK`/array result construction costs after inline storage,
3. host-side editor debounce/runtime split outside OxFunc.

## Status report

execution_state: `stopped_at_oxfunc_local_gate_downstream_handoff_filed`

scope_completeness: `scope_partial`

target_completeness: `target_partial`

integration_completeness: `partial`

open_lanes:
1. OxFml `invoke_many(...)` specialization through `HO-FN-015`,
2. post-OxFml-specialization DnaOneCalc perf replay,
3. landed-ref promotion.

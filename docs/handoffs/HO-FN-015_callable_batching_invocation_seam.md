# HO-FN-015 - Callable Batching Invocation Seam

Status: `filed`
Direction: `OxFunc -> OxFml`
Source repo/workset: `OxFunc/W095`
Target repo/workset: `OxFml lambda invocation performance follow-up`
Filed date: `2026-05-06`
Related inbound: `HANDOFF-OXFUNC-007`
Related upstream note: `../DnaOneCalc/docs/HANDOFF_OXFUNC_REDUCE_HOTLOOP_PERF.md`

## Purpose

Tell OxFml that W095 added the callable batching seam needed to specialize
lambda-helper hot loops without changing Excel-visible helper semantics.

## OxFunc Surface

`crates/oxfunc_core/src/functions/callable_helpers.rs` now exposes:

1. `CallableBatchMode`,
2. `CallableInvocationBatch`,
3. `CallableInvoker::invoke_many(...)`.

`invoke_many(...)` has a default fallback that preserves existing behavior by:

1. asking the batch for the next prepared argument slice,
2. applying the same callable arity check used by `invoke_callable_prepared`,
3. calling existing `invoke(...)`,
4. feeding the result back into the batch,
5. repeating until the batch reports no next slice.

## Batch Modes

1. `CallableBatchMode::SequentialStateful`

   Used by `REDUCE` and `SCAN`. These batches are ordered accumulator loops.
   OxFml may hoist setup, but must not reorder, parallelize, or skip calls.
   Each accepted result can affect the next argument slice.

2. `CallableBatchMode::Independent`

   Used by `MAP`, `BYROW`, `BYCOL`, and `MAKEARRAY`. These batches represent
   repeated helper invocations whose argument slices are not accumulator-state
   dependent. OxFml may still need to preserve trace ordering and visible error
   publication order.

## Requested OxFml Follow-Up

Specialize `OxFmlCallableInvoker::invoke_many(...)` so it can hoist repeated
per-call setup out of the helper loop:

1. registry borrow,
2. callable binding lookup and clone,
3. closure/capture setup,
4. parameter-name binding setup,
5. resolver setup,
6. trace allocation and per-call tracing scaffolding where safe.

## Non-Claims

1. This handoff does not ask OxFml to change lambda semantics.
2. This handoff does not enable `REDUCE.STOP` or any early-exit sentinel.
3. This handoff does not claim the DnaOneCalc Mandelbrot workload is at the final target runtime.
4. This handoff does not permit REDUCE/SCAN call reordering or parallelism.

## Evidence

OxFunc W095 focused validation:

1. `cargo fmt --manifest-path crates\oxfunc_core\Cargo.toml`: passed.
2. `cargo test --manifest-path crates\oxfunc_core\Cargo.toml --lib callable_helpers`: passed, `29` passed, `0` failed, `1246` filtered out.
3. Focused tests prove REDUCE and SCAN use `SequentialStateful` batch mode.
4. Focused tests prove MAP, BYROW, BYCOL, and MAKEARRAY use `Independent` batch mode.

W095 DnaOneCalc perf replay after inline `EvalArray` storage:

1. `cargo test --release -p dnaonecalc-host --test mandelbrot_perf_probe -- --ignored --nocapture`: passed, `2` passed, `0` failed.
2. `Mandelbrot 100x60x30 end-to-end`: `5.7998668s`.
3. `100x60x30` row: `5.5361486s`, `30.76us` per inner iteration.

## Status report

execution_state: `filed`

scope_completeness: `scope_partial`

target_completeness: `target_partial`

integration_completeness: `partial`

open_lanes:
1. OxFml `invoke_many(...)` specialization,
2. post-OxFml-specialization DnaOneCalc perf replay,
3. landed-ref promotion.

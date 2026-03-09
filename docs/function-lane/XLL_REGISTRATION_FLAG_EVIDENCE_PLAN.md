# XLL Registration Flag Evidence Plan

Status: `provisional`
Workset: `W11`

## 1. Purpose
Capture an evidence-first plan for XLL registration flags that are expected to depend on function profile axes, while deferring direct signature-builder mapping until empirical closure.

## 2. Deferred-Mapping Policy
1. Do not immediately map profile flags into registration type-text modifiers or register options.
2. First produce reproducible empirical evidence that each flag changes worksheet-observable behavior in the expected direction.
3. Only after evidence closure, add rule-based mapping into `crates/oxfunc_core/src/xll_export_specs.rs` and generated registration logic.
4. During evidence collection, use explicit experimental alias registrations only (no profile-to-signature-builder mapping).

## 3. Flag Hypotheses To Validate
1. Volatile registration flag is required for explicitly volatile functions (for example `NOW`) to participate in recalc invalidation cycles.
2. Thread-safe registration flag is required for functions classified `safe_pure` to enable correct multi-threaded recalc admission behavior.
3. Macro-type registration is required for functions that need macro-sheet-equivalent privileges (for example caller/context-sensitive or dereference behavior beyond direct value arguments).

## 4. Experimental Matrix
1. Volatile flag experiment:
   - target functions: `NOW` (primary), `TODAY` (follow-on).
   - compare variants: no volatile flag vs volatile-flag registration via aliases (`ox_NOW_F_BASE` vs `ox_NOW_F_VOL`).
   - probes: repeated `CalculateFull`/`CalculateFullRebuild`, save/reopen/recalc, dependency-isolated recalcs.
   - expected signal: volatile-flag variant updates according to recalc semantics while non-flag variant does not.
2. Thread-safe flag experiment:
   - target functions: `ABS`, `OP_ADD`, `ISNUMBER` (`safe_pure`) and one `host_serialized` control (`NOW`).
   - compare variants: thread-safe flag on/off under multi-thread recalc settings.
   - probes: deterministic parity, registration acceptance, and explicit multi-thread scheduling traces/instrumented probes.
   - expected signal: thread-safe-eligible functions remain correct with thread-safe flag; inappropriate functions should remain disallowed/serialized by policy.
3. Macro-type experiment:
   - target functions: initial alias pair on `INDIRECT` (`ox_INDIRECT_F_BASE` vs `ox_INDIRECT_F_MACRO`) plus follow-on caller/context probes.
   - compare variants: macro-type flag on/off (`#`) for same export implementation.
   - probes: caller-context/ref-resolution scenarios and class-2 callback-sensitive functions (`CELL`/`INFO`/`GET.CELL`-style evidence candidates).
   - expected signal: macro-required behavior only succeeds under macro-type registration where Excel requires it.

## 5. Candidate Expansion Functions
1. Add `TODAY`, `RAND`, `OFFSET`, and `CELL` as follow-on probes if primary signals are inconclusive.
2. Keep one nonvolatile/non-context control in each pack to detect harness drift.

## 6. Acceptance Criteria Before Mapping
1. Each flag has at least one positive and one control scenario with reproducible outcomes.
2. Outcomes are stable across both required version axes (app build/channel and workbook compatibility template/default).
3. Evidence artifacts are registered in `docs/function-lane/FUNCTION_LANE_EVIDENCE_ID_REGISTRY.md`.
4. Rule mapping is then added to profile->registration generation with explicit regression tests.

## 7. Required Artifacts
1. Scenario manifest:
   - `docs/function-lane/XLL_REGISTRATION_FLAG_SCENARIO_MANIFEST_SEED.csv`.
2. Runtime requirements + execution record:
   - `docs/function-lane/XLL_REGISTRATION_FLAG_PROBE_RUNTIME_REQUIREMENTS.md`
   - `docs/function-lane/XLL_REGISTRATION_FLAG_EXECUTION_RECORD.md`.
3. Dual-run Excel result CSVs and analyzer summaries:
   - `.tmp/xll-registration-flags-results-default.csv`
   - `.tmp/xll-registration-flags-results-compat.csv`
   - `.tmp/xll-registration-flags-results-excel.csv`
   - `.tmp/xll-registration-flags-analysis-report.csv`
   - `.tmp/xll-registration-flags-analysis-summary.json`.
4. Replay scripts:
   - `tools/xll-addin/run-registration-flag-evidence.ps1`
   - `tools/xll-addin/run-registration-flag-suite.ps1`
   - `tools/xll-addin/analyze-registration-flag-results.ps1`.
5. Bridge registration snapshots (`tools/xll-addin/oxfunc_xll/export_specs.csv`) and alias-registration notes.

## 8. Current Seed Observation (2026-03-09)
1. `ox_NOW_F_VOL` showed recalc deltas under incremental `Calculate()` as expected.
2. `ox_NOW_F_BASE` also showed recalc deltas in the same harness, so non-volatile behavior is not yet isolated/confirmed.
3. `$` and `#` seed probes currently establish registration/admission parity only; they do not yet close concurrency or macro-required behavior proof.

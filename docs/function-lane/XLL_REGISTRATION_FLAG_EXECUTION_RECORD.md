# XLL Registration Flag Execution Record

Status: `complete-provisional`
Workset: `W11`
Evidence ID: `W11-XLL-FLAGS-BL-20260309`

## 1. Purpose
Track empirical evidence for XLL registration flags (`!`, `$`, `#`) before enabling profile-derived mapping in export/signature generation.

## 2. Executed Scope
Execution date:
1. `2026-03-09`
2. `2026-03-10` (volatile-export follow-up rerun)
3. `2026-03-10` (RAND ordinary-export follow-up rerun)

Executed commands:
1. `cargo test -p oxfunc_core`
2. `cargo check --manifest-path tools/xll-addin/oxfunc_xll/Cargo.toml`
3. `powershell -File tools/xll-addin/build-oxfunc-xll.ps1 -Profile release`
4. `powershell -File tools/xll-addin/run-registration-flag-suite.ps1 -OutDir .tmp`

Primary inputs:
1. `docs/function-lane/XLL_REGISTRATION_FLAG_SCENARIO_MANIFEST_SEED.csv`
2. `tools/xll-addin/run-registration-flag-evidence.ps1`
3. `tools/xll-addin/analyze-registration-flag-results.ps1`

Primary outputs:
1. `.tmp/xll-registration-flags-results-default.csv`
2. `.tmp/xll-registration-flags-results-compat.csv`
3. `.tmp/xll-registration-flags-results-excel.csv`
4. `.tmp/xll-registration-flags-analysis-report.csv`
5. `.tmp/xll-registration-flags-analysis-summary.json`
6. `.tmp/xll-registration-flags-results-default.csv.run-metadata.json`
7. `.tmp/xll-registration-flags-results-compat.csv.run-metadata.json`

## 3. Completeness Axes
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_partial`
4. integration_completeness: `partial`
5. open_lanes:
   - `W11-VOL`: experimental non-volatile alias still changed across incremental recalc; broader volatility-control behavior remains unresolved even though ordinary `ox_NOW()` export alignment is now positive.
   - `W11-TS`: no multi-thread scheduling trace evidence yet (only scalar parity/registration acceptance).
   - `W11-MAC`: seed lane verifies registration acceptance only; macro-required behavior is not yet demonstrated.
   - broader profile-derived mapping beyond ordinary `volatile_full` exports is intentionally deferred.

## 4. Results Summary
1. total rows: `18` (`9` scenarios x `2` run labels).
2. execution observed: `18`; execution failed: `0`.
3. expectation matched: `16`; expectation mismatched: `2`.
4. dual-run requirement: `satisfied` (`default` + `compat_template`).
5. analyzer gate: `needs_attention`.

## 5. Findings
1. Experimental alias registration path is operational:
   - `ox_NOW_F_BASE`, `ox_NOW_F_VOL`, `ox_ABS_F_BASE`, `ox_ABS_F_TS`, `ox_INDIRECT_F_BASE`, `ox_INDIRECT_F_MACRO` are callable.
2. Volatile lane:
   - `ox_NOW_F_VOL()` changed across incremental recalcs (`matched`).
   - `ox_NOW_F_BASE()` also changed across incremental recalcs (`mismatched` against expected non-change).
   - `NOW()` vs ordinary `ox_NOW()` now both changed across incremental recalcs in both run labels (`matched`).
   - `RAND()` vs ordinary `ox_RAND()` now both changed across incremental recalcs in both run labels (`matched`).
   - ordinary profile-derived exports for `volatile_full` functions now emit `!` from core metadata, which closes the user-facing `ox_NOW()` and `ox_RAND()` discrepancy while leaving the experimental control-alias question open.
3. Thread-safe lane:
   - `$` and non-`$` ABS aliases both returned parity-correct scalar results.
   - this is registration/semantic parity only, not concurrency evidence.
4. Macro-type lane:
   - `#` and non-`#` INDIRECT aliases both admitted and produced the same seed-bounded result (`#VALUE!` on Ox side due reference-return bridge limits).
   - this does not prove macro-required behavior; only admission parity is shown.

## 6. Decision Status
1. Ordinary `volatile_full` export mapping is now enabled in profile-derived signature generation for user-facing exports (`ox_NOW`, `ox_TODAY`, `ox_RAND`).
2. Experimental/control volatile mapping, thread-safe mapping, and macro mapping remain **deferred** from broader profile-derived signature generation.
3. W11 evidence is captured and reproducible, but the experimental non-volatile volatile-control alias is still not isolated enough to close the full registration-flag lane.

## 7. XLL Verification-Seam Limitations
1. This record documents XLL registration evidence only; it does not prove full function-semantic parity.
2. Known seam limitations are tracked in `docs/function-lane/XLL_VERIFICATION_SEAM_LIMITATIONS.md`.
3. Relevant current limits for W11:
   - volatile evidence is still confounded by the non-volatile control alias changing across recalc,
   - thread-safe evidence does not yet include concurrency traces,
   - macro-type evidence is admission/parity-only under current reference-return bridge bounds.

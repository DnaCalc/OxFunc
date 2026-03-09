# WORKSET - TUX1000 XLL Registration Flags Evidence (W11)

## 1. Purpose
Build empirical closure for XLL registration flags that should be driven from function profiles:
1. volatile (`!`),
2. thread-safe (`$`),
3. macro-type (`#`).

This workset is evidence-first and explicitly defers profile-to-signature mapping changes.

## 2. Position and Dependencies
Program position:
1. post-W10 follow-on packet (`W11`).

Dependencies:
1. W9 Rust XLL bridge and generated export flow.
2. W10 function profile classification for volatile/thread safety/host interaction.
3. existing Excel replay harnesses and dual-run compatibility template workflow.

## 3. Scope
In scope:
1. experimental alias registration path in XLL bridge runtime (opt-in only).
2. scenario manifests and replay scripts for flag comparisons.
3. execution record and evidence registry updates.

Out of scope:
1. profile-derived mapping in `xll_export_specs` (deferred until evidence closure).
2. production-hardening of registration flag handling.

## 4. Deliverables
1. `docs/function-lane/XLL_REGISTRATION_FLAG_EVIDENCE_PLAN.md`
2. `docs/function-lane/XLL_REGISTRATION_FLAG_SCENARIO_MANIFEST_SEED.csv`
3. `docs/function-lane/XLL_REGISTRATION_FLAG_PROBE_RUNTIME_REQUIREMENTS.md`
4. `docs/function-lane/XLL_REGISTRATION_FLAG_EXECUTION_RECORD.md`
5. `tools/xll-addin/run-registration-flag-evidence.ps1`
6. `tools/xll-addin/run-registration-flag-suite.ps1`
7. `tools/xll-addin/analyze-registration-flag-results.ps1`

## 5. Gate Model
### G1 - Probe Infrastructure Closure
Pass when:
1. manifest + runner + analyzer execute and produce expected artifact files.

### G2 - Volatile Evidence Closure
Pass when:
1. non-volatile alias and volatile alias show expected recalc delta behavior.

### G3 - Thread-safe Evidence Closure
Pass when:
1. thread-safe registration parity is verified,
2. at least one explicit multi-thread scheduling/concurrency probe is captured.

### G4 - Macro-type Evidence Closure
Pass when:
1. macro-type positive/control behavior is observed on a macro-required function lane.

### G5 - Mapping Readiness
Pass when:
1. G2-G4 are closed with reproducible artifacts,
2. profile-derived mapping rules are codified with regression checks.

## 6. Status
Execution state:
1. `in_progress`.

Claim confidence:
1. `provisional`.

Assurance maturity:
1. `exercised` (pending G2-G4 closure).

Current gate snapshot (`2026-03-09`):
1. `G1` probe infrastructure: `closed`.
2. `G2` volatile evidence: `partial` (`ox_NOW_F_BASE` changed unexpectedly under current harness).
3. `G3` thread-safe evidence: `partial` (parity observed; no concurrency trace yet).
4. `G4` macro-type evidence: `partial` (admission parity only; no macro-required behavior case yet).
5. `G5` mapping readiness: `open`.

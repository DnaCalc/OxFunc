# WORKSET - TUX1000 Moderate Function Expansion (W12)

## 1. Purpose
Implement one additional moderate-complexity function batch to expand function-lane coverage while generating stronger evidence inputs for deferred W11 registration-flag mapping.

Primary intent:
1. increase semantic breadth without jumping to high-quirk extremes,
2. pressure-test profile classification fields against real implementations,
3. create better volatile/thread-safe/macro evidence candidates before enabling profile->registration flag mapping.

## 2. Position and Dependencies
Program position:
1. post-W11 expansion packet (`W12`).

Dependencies:
1. W10 mixed-seam implementation pattern and tooling.
2. W11 registration-flag evidence harness and execution model.
3. profile-derived XLL export generation lane from `FunctionMeta` metadata.

## 3. Function Scope (Proposed 15)
1. `AVERAGE`
2. `COUNT`
3. `COUNTA`
4. `IFERROR`
5. `ROUND`
6. `TEXTJOIN`
7. `TODAY`
8. `RAND`
9. `OFFSET`
10. `CELL`
11. `AND`
12. `CLEAN`
13. `DATE`
14. `EXACT`
15. `HSTACK`

Selection rationale:
1. Aggregates/coercion: `AVERAGE`, `COUNT`, `COUNTA`.
2. Error-flow/laziness: `IFERROR`.
3. Numeric kernel + coercion: `ROUND`.
4. Text variadic + coercion: `TEXTJOIN`, `CLEAN`.
5. Volatile class signal: `TODAY`, `RAND`.
6. Reference/context/macro pressure: `OFFSET`, `CELL`.
7. Logical/date/text-compare seams: `AND`, `DATE`, `EXACT`.
8. Dynamic-array assembly seam: `HSTACK`.

CELL-specific execution note:
1. run empirical probes for `CELL` behavior and option set before substantial implementation work.
2. use the probe results to sequence the remaining `CELL` semantic lanes; bounded subset coverage does not count as closure.

## 4. Expected Profile Pressure
1. explicit volatility (`TODAY`, `RAND`) with stronger control/positive evidence for `!` flag.
2. host/context and potential macro-type lanes (`CELL`, `OFFSET`) to strengthen `#` evidence.
3. thread-safety contrast across deterministic pure kernels versus host/context-sensitive functions for `$` evidence planning.
4. aggregate direct-vs-range coercion distinctions in additional families.

## 5. Deliverables
1. fifteen new function-slice contracts (`docs/function-lane/FUNCTION_SLICE_*_CONTRACT_PRELIM.md`).
2. runtime implementations and dispatch integration in `crates/oxfunc_core/src/functions/*`.
3. Lean modules for admitted kernel/surface scope in `formal/lean/OxFunc/Functions/*`.
4. W12 scenario manifests + replay requirements + execution record.
5. evidence registry and correlation ledger updates.
6. explicit W11 follow-back note with candidate scenarios unlocked by W12 functions.

## 6. Gate Model
### G1 - Classification Closure
Pass when:
1. all fifteen W12 functions have explicit profile fields (determinism/volatility/thread-safety/arg prep/coercion/kernel/FEC profiles).

### G2 - Runtime/Formal Pairing Closure
Pass when:
1. each W12 function has runtime and declared formal scope artifacts.

### G3 - Empirical Closure
Pass when:
1. W12 manifests replay on dual-run labels (`default`, `compat_template`) with mismatch classification.
2. `CELL` empirical pre-implementation probe pack is executed and scope-bounds are explicitly recorded before `CELL` runtime admission.

### G4 - W11 Follow-back Readiness
Pass when:
1. W12 outputs include concrete volatile and macro-required candidate scenarios for W11 (`TODAY`/`RAND`/`CELL`/`OFFSET`).

### G5 - Promotion Readiness
Pass when:
1. W12 functions reach full empirically aligned Excel semantics for the declared version axes,
2. no known semantic gaps remain except external XLL verification-seam limits,
3. contract/formal/runtime/evidence artifacts are aligned for the full function behavior claim.

## 7. Status
Execution state:
1. `in_progress`.

Claim confidence:
1. `provisional`.

Assurance maturity:
1. `exercised`.

## 8. W11 Follow-back Plan (After W12)
1. extend volatile evidence matrix with `TODAY` and `RAND` flagged/unflagged alias pairs.
2. add macro-required evidence probes on `CELL`/caller-context behavior and `OFFSET` reference-return scenarios.
3. add at least one concurrency-oriented thread-safety probe for a deterministic pure function vs host-sensitive control.
4. reassess readiness of profile->registration flag mapping after updated evidence summary.

## 9. Progress Notes
1. all fifteen function-slice contracts are now published in `docs/function-lane/FUNCTION_SLICE_*_CONTRACT_PRELIM.md` for the W12 set.
2. runtime slices are implemented in `crates/oxfunc_core/src/functions/*` and admitted through shared dispatch/export metadata.
3. Lean slices are implemented in `formal/lean/OxFunc/Functions/*` and `lake build` passes.
4. the `CELL` preprobe ran green across both required run labels and usefully narrowed the next semantic lanes.
5. dual-run Excel replay for the current W12 suite ran green across `34` observed rows with `expectation_mismatched=0`.
6. W11 follow-back readiness is improved by explicit `TODAY`/`RAND` volatile candidates and `CELL`/`OFFSET` caller-context candidates, but W12 remains open until the known semantic gaps are closed.

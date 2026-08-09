# BUG-FUNC-046: BESSELJ internal cosine inheritance drift

## Summary
- **Bug id**: `BUG-FUNC-046`
- **Opened**: 2026-08-09
- **Status**: in_progress
- **Owner workset**: `W109`

## Source Refs
- **Reported against ref**: `e73115a`
- **Reproduced on ref**: live Excel 16.0 build 20228 x64, Compatibility Version 2
- **Introduced in ref**: unknown
- **Fixed in ref**: none; the current J0-only working-tree candidate is superseded

## Ownership And Root Cause
- **Ownership class**: OxFunc-owned bug
- **Root cause class**: wrong_composition
- **Root cause summary**: Both `bessj0` and `bessj1` asymptotic cosine sites inherit worksheet COS, while J0 additionally publishes `cosine*p` through an x87 double-rounded multiply. The last linked J0/J2 pair is blocked on the newly exposed shared worksheet-COS substrate BUG-FUNC-047.

## Why Did We Get This Wrong?
- **Spec already correct and code was wrong?**: yes
- **Spec vague or missing?**: no
- **Code once correct and later regressed?**: no
- **Explanation**: The earlier Bessel regression deliberately excluded known large-argument cosine inheritance rows instead of registering the remaining cross-substrate gap.

## Reproduction
1. The original four witnesses showed `BESSELJ(50,0)`, `(150,0)`, `(50,2)`, and `(150,2)` missing Excel by `1`, `1`, `2`, and `8` ULP.
2. A fresh 794-row held-out replay scores platform trig `454/794`, the current J0-only working-tree route `732/794`, both J0/J1 cosine sites through current `excel_cos` `791/794`, and both cosine sites plus J0-only x87 `cosine*p` `792/794`.
3. Replacing only the two failing shared-COS phase results with their live worksheet bits yields `794/794`; no Bessel-body staging candidate closes those last two rows.
4. Exact mandatory pins are `BESSELJ(150−1 ULP,0)=0xbf495d8a81b9c8bf`, its order-2 recurrence result `0xbf18c693cd8c2560`, and `BESSELJ(108.43896102905273438,2)=0xbfa9b1eac88983f1`.

## Similar-Risk Scan
1. The broad candidate matrix checks J0 versus J1, cosine versus sine, P/Q Horner staging, setup, recurrence, and x87 product masks.
2. Both cosine sites are required; all sine substitutions and recurrence masks are inert. Only J0's `cosine*p` needs x87 double rounding.
3. Exact-phase decomposition exposed a cross-function worksheet COS gap, now registered as BUG-FUNC-047 / G4-07 and linked as this bug's dependency.

## Fix Plan
1. Identify and land the general shared worksheet-COS graph under BUG-FUNC-047; reject per-phase patches with adjacent/random held-out controls.
2. Route both J0 and J1 asymptotic cosine sites through the corrected substrate and use x87 double-rounded `cosine*p` only in J0.
3. Add the mandatory exact pins and retain the full 794-row replay as a gate.
4. Run focused and full core tests, then persist the provenance-bearing replay result.

## Validation
1. Executable candidate: `792/794`; oracle-informed decomposition: `794/794` against live Excel build 20228/CV2.
2. Shared COS graph, production integration, focused tests, and full-suite results remain pending.

## Evidence
1. `docs/bugs/streams/BUG-FUNC-024_bessely_current_baseline_exactness_drift.md`
2. `docs/function-lane/W109_TRIG_IDENTIFICATION_20260711.md`
3. `crates/oxfunc_core/src/functions/bessel_convert_family.rs`
4. `smart-fuzzer/tools/calc_graph_racer/src/bin/race_besselj_internal_trig.rs`
5. `BUG-FUNC-047` / bead `oxf-jwh5.5.1`

## Closure Checklist
- [ ] fix landed or non-OxFunc ownership recorded
- [ ] validation recorded
- [x] root cause class recorded
- [x] similar-risk scan recorded
- [ ] spec/matrix/contract updated if required
- [ ] handoff filed if required
- [ ] linked reports updated

# BUG-FUNC-047: worksheet COS phase/reduction residual

## Summary
- **Bug id**: `BUG-FUNC-047`
- **Opened**: 2026-08-09
- **Status**: in_progress
- **Owner workset**: `W109`

## Source Refs
- **Reported against ref**: `e73115a`
- **Reproduced on ref**: live Excel 16.0 build 20228 x64, Compatibility Version 2, `-NoCache`
- **Introduced in ref**: unknown
- **Fixed in ref**: none

## Ownership And Root Cause
- **Ownership class**: OxFunc-owned bug
- **Root cause class**: numeric_algorithm_exactness_gap
- **Root cause summary**: Fresh exact-phase witnesses refute universal bit identity of the current fFCOS-based `excel_cos` model. The missing phase/reduction or publication detail is not yet identified.

## Why Did We Get This Wrong?
- **Spec already correct and code was wrong?**: no; the prior model was over-generalized from a `5425/5425` corpus that did not contain the newly exposed phase class.
- **Spec vague or missing?**: yes; it recorded the surviving representative as a universal graph without an adjacent-phase doubt gate around Bessel-derived phases.
- **Code once correct and later regressed?**: no evidence.

## Reproduction
1. At phase `0x4062a6de04ab6900`, worksheet COS returns `0xbf86a0d99f46996e`; current `excel_cos` returns `0xbf86a0d99f46996d`.
2. At phase `0x4062a6de04ab6902`, worksheet COS returns `0xbf86a0d99f461970`; current `excel_cos` returns `0xbf86a0d99f46196f`.
3. Immediate neighboring phases in the 25-row ladder remain exact. Current fFCOS scores `17/25`; all tried published/x87 reduction, precision-control, and store variants score no better.
4. Injecting the two live COS results into the otherwise unchanged Bessel body closes the remaining linked J0/J2 witnesses and takes that decomposition from `792/794` to `794/794`.

## Similar-Risk Scan
1. Fresh J0/J1 reduced-phase controls at `x=108.43896102905273438` and the J1 phase at `150−1 ULP` match platform, current fFCOS, and worksheet COS, so this is not a blanket Bessel-only offset.
2. The existing SIN model matches all `25/25` phase-ladder rows; the new discrepancy is presently COS-specific.
3. Any repair must replay worksheet COS itself and all known internal COS inheritors, including BESSELJ and GAMMA reflection.

## Active Identification Plan
1. Extend the clean-room candidate space across FCOS versus FSINCOS, x87 precision control, argument/reduction stores, exact constants, and publication boundaries.
2. Use candidate-disagreement selection to capture adjacent and randomized phase rows with full build/CV/NoCache provenance.
3. Reject point patches and promote only a general executable graph that passes banked and fresh held-out gates.

## Validation
1. Current executable model: `17/25` focused COS phases and `792/794` dependent BESSELJ rows.
2. Oracle-informed decomposition only: `794/794`; this is evidence localization, not an implementation candidate.

## Evidence
1. `smart-fuzzer/tools/calc_graph_racer/src/bin/race_besselj_internal_trig.rs`
2. `smart-fuzzer/work/w109/G4-besselj/batch-besselj-cos-phase-followup-scratch.json`
3. `docs/function-lane/W109_TRIG_IDENTIFICATION_20260711.md` (superseded universal-COS claim)
4. `BUG-FUNC-046` / bead `oxf-jwh5.5.1`

## Closure Checklist
- [ ] fix landed or non-OxFunc ownership recorded
- [ ] validation recorded
- [x] root cause class recorded
- [x] similar-risk scan recorded
- [ ] spec/matrix/contract updated if required
- [ ] handoff filed if required
- [x] linked reports updated

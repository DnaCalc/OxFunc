# BUG-FUNC-047: worksheet COS odd-quadrant publication residual

## Summary
- **Bug id**: `BUG-FUNC-047`
- **Opened**: 2026-08-09
- **Status**: `closed_signed_off`
- **Owner workset**: `W109`

## Source Refs
- **Reported against ref**: `e73115a`
- **Reproduced on ref**: live Excel 16.0 build 20228 x64, Compatibility Version 2, `-NoCache`
- **Introduced in ref**: unknown
- **Fixed in ref**: `ed9f222`
- **Ref notes**: The tangent-square odd-quadrant graph, exact COS pins,
  dependent BESSELJ composition, and deterministic replay tooling landed
  together in `ed9f222`.

## Ownership And Root Cause
- **Ownership class**: OxFunc-owned bug
- **Root cause class**: numeric_algorithm_exactness_gap
- **Root cause summary**: The FPREM1 reduction and even-quadrant FCOS branches
  were correct, but the former odd-quadrant raw-FSIN publication subgraph was
  over-generalized. Excel reconstructs the signed sine magnitude as
  `sqrt(tan(r)^2 / (1 + tan(r)^2))` from FPTAN in continuous x87 PC64/RN
  arithmetic, with no binary64 spills before the final publication.

## Why Did We Get This Wrong?
- **Spec already correct and code was wrong?**: no; the prior model was over-generalized from a `5425/5425` corpus that did not contain the newly exposed phase class.
- **Spec vague or missing?**: yes; it recorded the surviving representative as a universal graph without an adjacent-phase doubt gate around Bessel-derived phases.
- **Code once correct and later regressed?**: no evidence.

## Reproduction
1. At phase `0x4062a6de04ab6900`, worksheet COS returns
   `0xbf86a0d99f46996e`; the former `excel_cos` returned
   `0xbf86a0d99f46996d`.
2. At phase `0x4062a6de04ab6902`, worksheet COS returns
   `0xbf86a0d99f461970`; the former `excel_cos` returned
   `0xbf86a0d99f46196f`.
3. Immediate neighboring phases in the 25-row ladder remain exact. The former
   fFCOS graph scores `17/25`; all tried published/x87 reduction,
   precision-control, and store variants score no better.
4. Injecting the two live COS results into the otherwise unchanged Bessel body closes the remaining linked J0/J2 witnesses and takes that decomposition from `792/794` to `794/794`.
5. The general tangent-square graph scores `1027/1027` on the adjacent/random
   discovery battery and `514/514` on a pre-frozen oracle-blind disagreement
   hold-out; the former raw-FSIN branch scores `1023/1027` and `0/514`.

## Similar-Risk Scan
1. Fresh J0/J1 reduced-phase controls at `x=108.43896102905273438` and the J1 phase at `150−1 ULP` match platform, the former graph, and worksheet COS, so this is not a blanket Bessel-only offset.
2. The existing SIN model matches all `25/25` phase-ladder rows; the new discrepancy is presently COS-specific.
3. Direct consumers are worksheet COS, SEC's already identified reciprocal
   composition, and BESSELJ. GAMMA reflection consumes SIN, not COS, so it was
   not reopened by this lane.
4. The dependent production BESSELJ replay is `794/794`; no per-phase patch is
   present.

## Fix And Landing
1. The clean-room search covered FCOS/FSINCOS, x87 precision control,
   argument/reduction stores, exact constants, public polynomial families, 960
   explicit reducer graphs, and 1728 alternate reduced-trig graphs.
2. The selected graph preserves the tiny guard and FPREM1 reduction, keeps FCOS
   on even quadrants, and on odd quadrants evaluates `t=FPTAN(r)`, `t2=t*t`,
   `d=1+t2`, `m=FSQRT(t2/d)` continuously in PC64/RN before applying the
   tangent/residue and quadrant signs.
3. Ref `ed9f222` lands that graph without phase-specific conditions. Exact
   phase pins, the original corpus, the 1027-row discovery battery, the frozen
   514-row hold-out, and dependent BESSELJ replay are permanent gates.
4. Cross-repo impact was assessed. No FEC/F3E boundary or evaluator-facing
   clause changed, so no handoff is required.

## Validation
1. Selected production graph: `1027/1027` discovery + `1020/1020` prior G4-01
   validation + `514/514` frozen oracle-blind hold-out = `2561/2561` exact.
   The original 24-row threshold ladder remains separate retained guard
   evidence.
2. Former raw-FSIN odd branch: `1023/1027`; other rejected families are
   recorded in the ruled-out ledger. On the candidate-disagreement hold-out,
   former baseline `0/514` versus selected production `514/514`.
3. Focused COS exact-bit test: `1/1` passed. Full `cargo test -p oxfunc_core`:
   library `1520` passed, `0` failed, `4` ignored; every integration and
   doc-test target shown by the run passed.
4. Lean alignment: `lake build` passed `492` jobs.
5. Build/CV/provenance: Excel 16.0 build 20228 x64, workbook Compatibility
   Version 2, `cell_value2_bulk`, `-NoCache`, Excel process count `0` before
   and after the frozen hold-out.
6. Answer SHA256 values:
   - adjacent discovery:
     `049E4674A7E51F338D1C999BC430B1A2F5DDC3BD06D780EF11F0F18BC6118F32`
   - randomized discovery:
     `6B3AA7A860DA67AA3D59062AF9C3B3A9F8B4E21555063554993FC65772FE60D6`
   - frozen hold-out:
     `942194EA15D9AC8D63E5FF0696D1B14595524E7437E5EC8C548A71BFE489943C`

## Evidence
1. `smart-fuzzer/tools/calc_graph_racer/src/bin/race_besselj_internal_trig.rs`
2. `smart-fuzzer/work/w109/G4-besselj/batch-cos-adjacent-disagreement-20260809.json`
3. `smart-fuzzer/work/w109/G4-besselj/answers-cos-adjacent-disagreement-20260809.json`
4. `smart-fuzzer/work/w109/G4-besselj/batch-cos-random-disagreement-20260809.json`
5. `smart-fuzzer/work/w109/G4-besselj/answers-cos-random-disagreement-20260809.json`
6. `smart-fuzzer/work/w109/G4-besselj/batch-cos-tangent-square-heldout-20260809-manifest.json`
7. `smart-fuzzer/work/w109/G4-besselj/answers-cos-tangent-square-heldout-20260809.json`
8. `docs/function-lane/W109_TRIG_IDENTIFICATION_20260711.md`
9. `BUG-FUNC-046` / bead `oxf-jwh5.5`

## Closure Checklist
- [x] fix landed or non-OxFunc ownership recorded (`ed9f222`)
- [x] validation recorded
- [x] root cause class recorded
- [x] similar-risk scan recorded
- [x] spec/matrix/contract updated if required
- [x] cross-repo impact assessed; no handoff required
- [x] linked reports updated

## Closure Verification (2026-08-09)

Status axes for the declared G4-01 correction / `BUG-FUNC-047` / G4-07 slice:

- `execution_state: complete`
- `scope_completeness: scope_complete`
- `target_completeness: target_complete`
- `integration_completeness: integrated`
- `open_lanes: []` within this discrepancy slice. Wider W109 and orthogonal
  alternate application/channel/CV validation remain outside this closure.

Pre-Closure Verification Checklist (`OPERATIONS.md` Section 12):

1. contract rows complete/promoted for the slice: yes; FDEF-067 and the
   corrected trig identification bind the graph at
   `provisional_w109_aligned` scope.
2. Lean/formal alignment satisfied: yes; the executable publication-route tag
   is present and the `492`-job build passed.
3. Rust implementation and required tests pass: yes; focused, full-core,
   2561-row COS, and 794-row dependent production gates pass.
4. deterministic replay artifact exists: yes; the discovery and frozen
   held-out batches, manifests, metadata, answers, and racer are retained.
5. evidence links complete and reproducible: yes; artifact paths and answer
   hashes are recorded above.
6. both version axes explicit: yes; Excel 16.0 build 20228 x64 and workbook
   Compatibility Version 2 are the declared reference profile.
7. public-doc/empirical divergence handled in favor of Excel: yes; the former
   universal raw-FSIN inference is explicitly superseded.
8. XLL seam limitation documented where material: yes; it is not material to
   this direct worksheet-oracle/core-kernel slice.
9. cross-repo impact assessed: yes; no FEC/F3E or evaluator-facing change, so
   no handoff is required.
10. no known semantic gap remains in the declared discrepancy slice: yes.
11. completion-language audit passed: yes; only G4-01's correction and G4-07
    are closed here.
12. in-progress worklist updated: yes; wider W109 remains partial.
13. execution-state surface updated: yes; bead `oxf-jwh5.5.1` records the
    landed ref and validation closure.

Completion Claim Self-Audit (`OPERATIONS.md` Section 14):

1. scope re-read: pass; only the shared worksheet-COS publication residual is
   claimed.
2. gate criteria re-read: pass; implementation, exact replay, tests, formal
   alignment, evidence, and landed ref are present.
3. silent scope reduction: pass; the guard, all quadrants, adjacent/random
   disagreement classes, and dependent consumers remain covered.
4. looks-done-but-is-not patterns: pass; no point patch, stub, compile-only
   path, unsupported proof claim, or unacknowledged handoff supports closure.
5. result: pass for the declared G4-01 correction / BUG-FUNC-047 / G4-07 slice.

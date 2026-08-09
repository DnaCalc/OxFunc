# BUG-FUNC-046: BESSELJ internal cosine inheritance drift

## Summary
- **Bug id**: `BUG-FUNC-046`
- **Opened**: 2026-08-09
- **Status**: `closed_signed_off`
- **Owner workset**: `W109`

## Source Refs
- **Reported against ref**: `e73115a`
- **Reproduced on ref**: live Excel 16.0 build 20228 x64, Compatibility Version 2
- **Introduced in ref**: unknown
- **Fixed in ref**: `ed9f222`
- **Ref notes**: The shared COS repair, both BESSELJ cosine routes, J0-only
  x87 product staging, exact regression pins, and deterministic replay tooling
  landed together in `ed9f222`.

## Ownership And Root Cause
- **Ownership class**: OxFunc-owned bug
- **Root cause class**: wrong_composition
- **Root cause summary**: Both `bessj0` and `bessj1` asymptotic cosine sites
  inherit worksheet COS, while J0 additionally publishes `cosine*p` through
  an x87 double-rounded multiply. The earlier implementation routed only J0
  through the then-incomplete COS substrate and omitted the product boundary.

## Why Did We Get This Wrong?
- **Spec already correct and code was wrong?**: yes
- **Spec vague or missing?**: no
- **Code once correct and later regressed?**: no
- **Explanation**: The earlier Bessel regression deliberately excluded known large-argument cosine inheritance rows instead of registering the remaining cross-substrate gap.

## Reproduction
1. The original four witnesses showed `BESSELJ(50,0)`, `(150,0)`, `(50,2)`, and `(150,2)` missing Excel by `1`, `1`, `2`, and `8` ULP.
2. A fresh 794-row held-out replay scores platform trig `454/794`, the former
   J0-only working-tree route `732/794`, both J0/J1 cosine sites through the
   former `excel_cos` `791/794`, and both cosine sites plus J0-only x87
   `cosine*p` `792/794`.
3. Replacing only the two failing shared-COS phase results with their live worksheet bits yields `794/794`; no Bessel-body staging candidate closes those last two rows. BUG-FUNC-047 subsequently identified the general tangent-square COS graph without phase patches.
4. Exact mandatory pins are `BESSELJ(150−1 ULP,0)=0xbf495d8a81b9c8bf`, its order-2 recurrence result `0xbf18c693cd8c2560`, and `BESSELJ(108.43896102905273438,2)=0xbfa9b1eac88983f1`.
5. Rebuilding the racer against production ref `ed9f222` scores the actual
   `besselj_kernel` `794/794`, with zero residuals across every class, order,
   x-band, and seed/recurrence partition.

## Similar-Risk Scan
1. The broad candidate matrix checks J0 versus J1, cosine versus sine, P/Q Horner staging, setup, recurrence, and x87 product masks.
2. Both cosine sites are required; all sine substitutions and recurrence masks are inert. Only J0's `cosine*p` needs x87 double rounding.
3. Exact-phase decomposition exposed a cross-function worksheet COS gap,
   registered and repaired as BUG-FUNC-047 / G4-07.
4. `BESSELY` remains on its independent Y-family trigonometric sites and its
   existing 93-row signed-off grid; no Y-site reroute was indicated.

## Fix And Landing
1. BUG-FUNC-047 identified and landed the general shared worksheet-COS graph;
   adjacent/random discovery and a frozen oracle-blind held-out reject phase
   patches and the former raw-FSIN odd branch.
2. Ref `ed9f222` routes both J0 and J1 asymptotic cosine sites through the
   corrected substrate and uses x87-double-rounded `cosine*p` only in J0.
3. The mandatory exact pins and the full 794-row production replay are
   permanent gates.
4. Cross-repo impact was assessed. No FEC/F3E boundary or evaluator-facing
   clause changed, so no handoff is required.

## Validation
1. Production `besselj_kernel` at `ed9f222`: `794/794` exact against live Excel
   16.0 build 20228 x64/CV2 Value2 `-NoCache`; former platform route `454/794`,
   J0-only route `732/794`, both former-COS routes `791/794`, and those routes
   plus J0 x87 product `792/794`.
2. Focused BESSELJ exact-bit test: `1/1` passed.
3. Full `cargo test -p oxfunc_core`: library `1520` passed, `0` failed, `4`
   ignored; every integration and doc-test target shown by the run passed.
4. Lean alignment: `lake build` passed `492` jobs.
5. Answer SHA256:
   `079184AAA18C22C0116BA4703A00F194D3C496A22963A9FA9DA1BEAE2D571E29`.

## Evidence
1. `docs/bugs/streams/BUG-FUNC-024_bessely_current_baseline_exactness_drift.md`
2. `docs/function-lane/W109_TRIG_IDENTIFICATION_20260711.md`
3. `crates/oxfunc_core/src/functions/bessel_convert_family.rs`
4. `smart-fuzzer/tools/calc_graph_racer/src/bin/race_besselj_internal_trig.rs`
5. `smart-fuzzer/work/w109/G4-besselj/answers-besselj-internal-trig-heldout-20260809.json`
6. `BUG-FUNC-047` / bead `oxf-jwh5.5.1`

## Closure Checklist
- [x] fix landed or non-OxFunc ownership recorded (`ed9f222`)
- [x] validation recorded
- [x] root cause class recorded
- [x] similar-risk scan recorded
- [x] spec/matrix/contract updated if required
- [x] cross-repo impact assessed; no handoff required
- [x] linked reports updated

## Closure Verification (2026-08-09)

Status axes for the declared `BUG-FUNC-046` / G4-06 discrepancy slice:

- `execution_state: complete`
- `scope_completeness: scope_complete`
- `target_completeness: target_complete`
- `integration_completeness: integrated`
- `open_lanes: []` within this bug slice. Wider W109 and orthogonal alternate
  application/channel/CV validation remain outside this closure.

Pre-Closure Verification Checklist (`OPERATIONS.md` Section 12):

1. contract rows complete/promoted for the slice: yes; FDEF-067 and the Bessel
   contract bind both COS sites and the J0-only product at
   `provisional_w109_aligned` scope.
2. Lean/formal alignment satisfied: yes; the executable site-route tag is
   present and the `492`-job build passed.
3. Rust implementation and required tests pass: yes; focused, full-core, and
   794-row production replay gates pass.
4. deterministic replay artifact exists: yes; the frozen 794-row batch,
   manifest, metadata, answers, and racer are retained.
5. evidence links complete and reproducible: yes; the exact artifact path and
   SHA256 are recorded above.
6. both version axes explicit: yes; Excel 16.0 build 20228 x64 and workbook
   Compatibility Version 2 are the declared reference profile.
7. public-doc/empirical divergence handled in favor of Excel: yes; production
   follows the reproducible black-box graph.
8. XLL seam limitation documented where material: yes; it is not material to
   this direct worksheet-oracle/core-kernel slice.
9. cross-repo impact assessed: yes; no FEC/F3E or evaluator-facing change, so
   no handoff is required.
10. no known semantic gap remains in the declared discrepancy slice: yes.
11. completion-language audit passed: yes; only G4-06 is closed here.
12. in-progress worklist updated: yes; wider W109 remains partial.
13. execution-state surface updated: yes; bead `oxf-jwh5.5` records the landed
    ref and validation closure.

Completion Claim Self-Audit (`OPERATIONS.md` Section 14):

1. scope re-read: pass; only BESSELJ internal COS/product composition is
   claimed.
2. gate criteria re-read: pass; implementation, exact replay, tests, formal
   alignment, evidence, and landed ref are present.
3. silent scope reduction: pass; both seeds, recurrence consumers, and all
   staged axes in the 794-row corpus remain covered.
4. looks-done-but-is-not patterns: pass; no phase patch, stub, compile-only
   path, unsupported proof claim, or unacknowledged handoff supports closure.
5. result: pass for the declared `BUG-FUNC-046` / G4-06 slice.

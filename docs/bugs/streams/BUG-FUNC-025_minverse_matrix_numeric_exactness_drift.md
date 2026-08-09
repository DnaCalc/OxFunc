# BUG-FUNC-025: MINVERSE matrix numeric exactness drift

## Summary
- **Bug id**: `BUG-FUNC-025`
- **Opened**: `2026-04-30`
- **Status**: `closed_signed_off`
- **Owner workset**: `W089`
- **Bead**: `oxf-dzfk`
- **Split from**: `BUG-FUNC-023`

## Source Refs
- **Reported against ref**: `w089-comprehensive-seed-20260430-004`
- **Reproduced on ref**: `oxf-i45e-w089-repair-20260430-001`
- **Introduced in ref**: `unknown`
- **Fixed in ref**: `bce3558`

## Ownership And Root Cause
- **Ownership class**: `OxFunc-owned bug`
- **Root cause class**: `matrix_numeric_algorithm_exactness_gap`
- **Root cause summary**: after the earlier Gauss-Jordan-to-Doolittle repair,
  the remaining implementation still evaluated each LU/solve arithmetic site
  in ordinary binary64. Current-reference Excel publishes each of the eight
  factor/elimination/solve sites through `RN53(RN64(op))`, then canonicalizes
  completed numeric zero cells to positive zero. The earlier scalar `1x1`
  publication note remains a separate OxFml/DNA Calc publication-seam concern.

## Reproduction
Run:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File smart-fuzzer\tools\Run-ArraySupportTranche.ps1 `
  -RunId oxf-i45e-w089-repair-20260430-001 `
  -CaseSetPath smart-fuzzer\cache\scenario-seed-executable-cases-v0.json
```

Representative row:

1. `=MINVERSE({1,2;3,4})`: local
   `array:2x2:[number:0xbffffffffffffffe|number:0x3feffffffffffffe|number:0x3ff7ffffffffffff|number:0xbfdfffffffffffff]`,
   Excel
   `array:2x2:[number:0xbfffffffffffffff|number:0x3fefffffffffffff|number:0x3ff7ffffffffffff|number:0xbfdffffffffffffe]`.

## Repair Direction
1. Build a compact Excel probe grid over small `1x1`, `2x2`, and `3x3`
   matrices, including integer, fractional, pivoting, near-singular, and
   identity-adjacent lanes.
2. Compare Gauss-Jordan, LU-solve, and any selected public numerical method
   against the Excel grid before changing the kernel.
3. Repair by matrix algorithm/rounding path only; do not nudge individual
   witness cells.
4. Keep the comparison policy as `exact_typed_bit_match_no_tolerance`.

## 2026-05-10 W097 R-F Cell-Ref Re-Sweep

W097 R-F replayed the witness and a `45`-matrix band of 2x2 / 3x3 /
4x4 random and structured matrices under cell-ref Excel input
plumbing. Each result cell is read scalar-by-scalar via
`INDEX(MINVERSE(<range>), r, c)`. Tranche record:
`smart-fuzzer/planning/W097-R-F-minverse-cell-ref-resweep.md`.

Witness `=MINVERSE({1,2;3,4})` reproduces bit-for-bit — three of the
four result cells drift by exactly one ULP, the `(1,0)` cell is
exact, matching the historical `BUG-FUNC-025` witness pair:

| (r, c) | local bits             | Excel bits             | ULP   |
| ------ | ---------------------- | ---------------------- | ----- |
| (0, 0) | `0xbffffffffffffffe`   | `0xbfffffffffffffff`   | `1`   |
| (0, 1) | `0x3feffffffffffffe`   | `0x3fefffffffffffff`   | `1`   |
| (1, 0) | `0x3ff7ffffffffffff`   | `0x3ff7ffffffffffff`   | `0`   |
| (1, 1) | `0xbfdfffffffffffff`   | `0xbfdffffffffffffe`   | `1`   |

Per-kind summary across `45` matrices / `440` cells: matches `217`,
drifts `223`, kind drift `0`, blocked `0`.

Highlights:

- **Identity and diagonal matrices** (any size): bit-exact across
  every cell. Algorithm-choice impact zero.
- **Random matrices** (well-conditioned): typically `0..7` ULP per
  cell. One `4x4` random outlier reached `2050` ULP.
- **Hilbert matrices**: drift grows with `n` — `22` ULP for `3x3`,
  `352` ULP for `4x4`. This reflects condition-number amplification
  of the Gauss-Jordan rounding-path delta, not a kernel bug.
- **Diagonally-dominant matrices**: ~`1..2` ULP per cell.

The R-F case set is the appropriate regression-validation gate when
a future repair lands a different matrix-inversion substrate
(LU-solve / Crout / Cholesky). Anything worse than the per-kind
floor recorded above is a regression.

## 2026-07-13 W109 Kernel Landed — Gauss-Jordan → Doolittle LU

**Historical checkpoint, superseded by the 2026-08-09 exact graph and sign-off
below.**

The inversion kernel `inverse_kernel` in
`crates/oxfunc_core/src/functions/matrix_family.rs` has been replaced. It
previously ran **Gauss-Jordan on `[A|I]`** (the algorithm W109 had already
ruled out); it now runs the identified **Doolittle LU + partial pivot +
per-column unit-vector solve with division-form back-substitution, plain
binary64** (each multiply/subtract rounds separately — no FMA contraction;
the multiplier is a true division). The determinant kernel already carried
the same LU elimination, so the two matrix kernels are now consistent.

Verified end-to-end through the compiled surface (`eval_surface_value_call`
via `matrix_local_eval`), scored against cached live-Excel bits
(build 20131):

| Corpus | Gauss-Jordan (old) | Doolittle LU (new) |
| ------ | ------------------ | ------------------ |
| 3x3 (159 cells, 21 matrices) | `80` | `150` |
| 4x4 (448 cells, 28 matrices, `m4b`) | `102` | `448` — **perfect** |

Net **+416 cells** (`182 → 598` of `607`). The old-kernel passing set is a
strict subset of the new (0 regressions, confirmed cell-by-cell).
`cargo test -p oxfunc_core --lib` = `1502/1502` green; the 11 matrix unit
tests (singular→#NUM guard, seed inverse, non-square→#VALUE) all preserved —
the `EPS = 1e-12` singularity threshold is unchanged, so only the algorithm
changed.

**Residual (still open at this checkpoint):** `9` cells on the 3x3 corpus (+ 2
of the 4 on the `{1,2;3,4}` witness) drift by exactly `+1`/`+2` ULP. All are
ill-conditioned
small-determinant cases (tridiag det 4, `[[1,2,3],[4,5,6],[7,8,10]]` det -3,
near-identity 1e-8) where Excel lands 1 ULP off the exact representable
value. Ruled out as explanations this round: the full 32-variant solve-ordering
sweep (all ≤150), and x87 80-bit extended registers (best 110, strictly worse
— MINVERSE is plain SSE2 double, not a legacy x87 body). The residual
*direction flips* (Excel further from exact on tridiag/integer, closer on
near-identity), so it is a genuinely different op-graph for those cells, not a
uniform extra/fewer rounding. Tracked as a targeted decoder probe in
`docs/function-lane/DISCREPANCY_RULED_OUT_LEDGER.csv`. **4x4 has zero residual.**

The W097 R-F per-kind drift floor recorded above no longer applies as-is: the
Gauss-Jordan condition-number amplification (Hilbert 4x4 → 352 ULP) is gone;
the new gate is the m4b 448/448 bit-exact result plus the 3x3 150/159.

## 2026-08-09 W109 Exact Graph And Sign-Off

The July conclusion that MINVERSE was a plain-SSE2 body was too strong. Its
continuous-x87 candidate family was genuinely ruled out, but that experiment
did not test the legacy compiler pattern that stores every individual
operation after first rounding it at x87 PC64. A new 256-mask racer tested
those sites independently in the already-landed right-looking Doolittle graph:

1. factor division;
2. elimination multiply and subtract;
3. forward-solve multiply and subtract;
4. backward-solve multiply and subtract;
5. final back-substitution division.

The banked `607` cells initially selected only the final divide because all
other mask bits were observationally inert there. A deterministic, bank-
disjoint `576`-row matrix battery then selected all eight x87-double-rounded
sites; it also exposed five internal negative-zero results where Excel
publishes positive zero. Because that battery informed the model, it is
explicitly retired into refinement rather than cited as publication evidence.

The final graph was frozen as full mask `0xff` plus completed-output `+0`
normalization. A second generator excluded every banked/refinement matrix and
selected `32` fresh disagreements for each single missing site, `64` signed-
zero rows, and `96` collapse controls. Fresh Excel 16.0 build 20228 x64,
workbook Compatibility Version 2, matrix `Range.Value2`/`INDEX`, `-NoCache`
capture scored the frozen graph `416/416`; every single-missing-site graph lost
its targeted rows. The compiled production surface now replays:

| Evidence set | Production exact |
| --- | ---: |
| banked 2x2/3x3/4x4 cells | `607/607` |
| retired refinement battery | `576/576` |
| frozen disjoint publication gate | `416/416` |
| combined | `1599/1599` |

The fix and nine exact pins landed in `bce3558`. Focused tests pass; full
`oxfunc_core` validation passes `1521` tests with `4` ignored. The route does
not change FEC/F3E admission or array-result shape. `MINVERSE(5)` final-cell
scalar appearance therefore remains under `CSC-0024` / `HO-FN-010`, and parent
BUG-FUNC-023 remains open for that downstream seam.

### Durable artifact hashes

1. Banked answers:
   - `G5-01-answers-minverse.json`: `B84C867AC5B9DA701BFB7E320E6670D05D2675BD72C0CDEFA39286E0E649809B`
   - `G5-01-answers-minverse-r1.json`: `5A7D75012B9C64A3F406DF0C5114D93691DD66DA5436CDC749BC6699650EF2AA`
   - `G5-01-answers-m4b.json`: `DEED368EA23EF6ED1B10D773BD31606FEE4A0481325492AD66F801EB983828B5`
2. Retired refinement set under `smart-fuzzer/work/w109/G5-01-minverse/`:
   - batch: `8E22395B0F56E430AAF3249533A22C7358FCFC48A2433C7F62DAB0D788518F7E`
   - meta: `BCD85503DB214A13169258F1EB70A428A5BBF3986076FCA91FC58EAB6BEBA5F5`
   - manifest: `CC16E197135311BACDBE1B4B368380A504018F1FAB642D338FAF8769763E315E`
   - answers: `65BD93C9D7D54F34B2B6D5A15A38770920BBCA934DE6D7123371A5DFA393D34B`
3. Frozen publication set in the same directory:
   - batch: `2D27FBDC9B27D3DBC1311C125D49017C67DAED348C9551B199A13621EB367375`
   - meta: `E86B55E3D832C7971C61340F3C4220C5911409F79DB980DB0A045099E38B2F91`
   - manifest: `67CD7BF5758FF878AA61911A186E357770A1F1C3A4013641B4430A463A3FC7DA`
   - answers: `6564878727E68DCE3E18655288245EBCE1BC161D3684E519F4C442C92BCAA4EE`
4. Frozen generator source: `D074D5D40E249176C315B6B93BDD2EEA15F96A2F652F2347FD27B252D711AE7F`.
5. Direct production scorer source: `13B9CF5B38B3035D299A22DB25E08C043B87C724F5BBFE0DCF48CEEAC5647FFE`.

## Evidence
1. `smart-fuzzer/runs/w089-comprehensive-seed-20260430-004/`
2. `smart-fuzzer/runs/oxf-i45e-w089-repair-20260430-001/`
3. Parent stream: `docs/bugs/streams/BUG-FUNC-023_w089_non_statistical_exactness_and_matrix_shape_drift.md`
4. Bead: `oxf-dzfk`
5. W092 freshness replay:
   - `smart-fuzzer/runs/w092-scenario-math-cycle-001/` reproduced
     `=MINVERSE({1,2;3,4})` as `known_residual`.
   - The same run also classified `=MINVERSE(5)` and `=MMULT(5,2)` as
     `adapter_or_seam_mismatch` under `HO-FN-010`, not as matrix-kernel repair
     targets.
6. W097 R-F cell-ref re-replay:
   - `smart-fuzzer/runs/W097-R-F-minverse-cellref/` (`45` matrices,
     `440` per-cell comparisons; witness reproduced bit-for-bit;
     per-kind drift floor recorded).
   - Tranche record:
     `smart-fuzzer/planning/W097-R-F-minverse-cell-ref-resweep.md`.
   - Driver: `smart-fuzzer/tools/Run-MinverseResweep.ps1`.
   - Local matrix evaluator:
     `smart-fuzzer/tools/pmt_ppmt_local_eval/src/bin/matrix_local_eval.rs`.

## Closure Checklist
- [x] fix landed or non-OxFunc ownership recorded
- [x] validation recorded
- [x] root cause recorded
- [x] similar-risk scan recorded
- [x] spec/matrix/contract updated if required
- [x] handoff filed if required (`not_required`: no evaluator-facing seam changed;
  existing `HO-FN-010` remains independently open)

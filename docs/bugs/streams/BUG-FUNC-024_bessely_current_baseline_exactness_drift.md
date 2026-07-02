# BUG-FUNC-024: BESSELY current-baseline exactness drift

## Summary
- **Bug id**: `BUG-FUNC-024`
- **Opened**: `2026-04-30`
- **Status**: `closed_signed_off` (2026-07-02; 93/93 live-Excel bit-exact, run
  `oxf-xp6p-bugfunc024-repair-validation-20260702`)
- **Owner workset**: `W089`
- **Bead**: `oxf-xp6p`
- **Split from**: `BUG-FUNC-023`

## Source Refs
- **Reported against ref**: `w089-comprehensive-seed-20260430-004`
- **Reproduced on ref**: `oxf-i45e-w089-repair-20260430-001`
- **Introduced in ref**: `unknown`
- **Fixed in ref**: `oxf-xp6p-bugfunc024-repair-validation-20260702` (local kernel fix,
  this branch)

## Ownership And Root Cause
- **Ownership class**: `OxFunc-owned bug`
- **Root cause class**: `numeric_algorithm_exactness_gap`
- **Root cause summary**: `BESSELY` still follows the current local Bessel
  approximation path rather than Excel's current-baseline publication path for
  the affected scalar row. The row is not an array-admission problem and should
  not be repaired by a formula-specific lookup.

## Reproduction
Run:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File smart-fuzzer\tools\Run-ArraySupportTranche.ps1 `
  -RunId oxf-i45e-w089-repair-20260430-001 `
  -CaseSetPath smart-fuzzer\cache\scenario-seed-executable-cases-v0.json
```

Representative row:

1. `=BESSELY(2.5,1)`: local `number:0x3fc2ad722ba3570c`,
   Excel `number:0x3fc2ad720e3ee754`.

## Repair Direction
1. Build a compact Excel probe grid over `x` and non-negative integer order,
   including values around the current branch boundaries and recurrence lanes.
2. Compare current local Bessel `Y` components against the Excel grid and
   high-accuracy public mathematical references.
3. Repair by substrate/algorithm only; do not patch this witness as a special
   case.
4. Keep the comparison policy as `exact_typed_bit_match_no_tolerance`.

## 2026-05-10 W097 R-E Cell-Ref Re-Sweep

W097 R-E re-replayed the witness and a `93`-case `(x, n)` band around
it under cell-ref Excel input plumbing. Tranche record:
`smart-fuzzer/planning/W097-R-E-bessely-cell-ref-resweep.md`.

The witness `=BESSELY(2.5, 1)` reproduces bit-for-bit:

- local `0x3fc2ad722ba3570c`
- Excel `0x3fc2ad720e3ee754`
- ULP distance `493,121,464`

Per-`n` ULP histogram across the `(x, n)` band:

| `n` | rows | matches | drifts | ULP min  | ULP median | ULP max  |
| --: | ---: | ------: | -----: | -------: | ---------: | -------: |
|  `0`|  `20` |     `1` |   `19` | `7.3E3`  |    `6.1E5` | `4.3E6`  |
|  `1`|  `20` |     `0` |   `20` | `3.3E6`  |    `4.9E8` | `2.1E12` |
|  `2`|  `20` |     `0` |   `20` | `2.6E5`  |    `2.0E8` | `4.2E11` |
|  `3`|  `11` |     `0` |   `11` | `2.0E6`  |    `5.7E7` | `1.4E12` |
|  `5`|  `11` |     `0` |   `11` | `2.0E6`  |    `5.6E7` | `1.3E12` |
| `10`|  `11` |     `0` |   `11` | `1.1E6`  |    `7.6E8` | `5.8E11` |

Direction: the BESSELY drift surface is broad and large. The recorded
witness is roughly the median of the `n=1` band; the kernel is
approximately uniformly off across the entire `(x, n)` sample. The
drift grows with `|n|` and stays in the `10^6..10^12` ULP range across
most of the surface. This confirms the "substrate/algorithm only; do
not patch this witness as a special case" repair direction recorded
in this stream.

## Evidence
1. `smart-fuzzer/runs/w089-comprehensive-seed-20260430-004/`
2. `smart-fuzzer/runs/oxf-i45e-w089-repair-20260430-001/`
3. Parent stream: `docs/bugs/streams/BUG-FUNC-023_w089_non_statistical_exactness_and_matrix_shape_drift.md`
4. Bead: `oxf-xp6p`
5. W092 freshness replay:
   - `smart-fuzzer/runs/w092-scenario-engineering-cycle-001/` reproduced
     `=BESSELY(2.5,1)` with the same local and Excel numeric digests.
   - `smart-fuzzer/runs/w092-scenario-engineering-cycle-002/` records the same
     row as `known_residual` after the smart-fuzzer comparator was narrowed for
     this already-promoted exactness lane.
6. W097 R-E cell-ref re-replay:
   - `smart-fuzzer/runs/W097-R-E-bessely-cellref/` (`93`-case band, witness
     bit-for-bit reproduced).
   - Tranche record:
     `smart-fuzzer/planning/W097-R-E-bessely-cell-ref-resweep.md`.
   - Driver: `smart-fuzzer/tools/Run-BesselyResweep.ps1`.

## 2026-07-02 Repair: three OxFunc port typos + two Excel-side table quirks

The entire drift surface (`10^6..10^12` ULP) came down to coefficient-level defects in
`crates/oxfunc_core/src/functions/bessel_convert_family.rs` against Excel's Numerical
Recipes-derived tables. Root-caused by solving per-coefficient deltas as linear systems
against live-Excel bit witnesses (W097 93-case grid + fresh 92-row BESSELJ/I/K probes).

OxFunc port typos (bugs on our side):

1. `bessy1` `x<8` denominator: NR's degree-6 polynomial lost its `0.3549632885e3`
   term (the Horner table jumped from `1.020426050e5` straight to `1.0`). Error grows
   `~x^10`; this alone was the `=BESSELY(2.5,1)` witness (`4.9e8` ULP) and, seeded
   through the upward recurrence, the whole order>=2 lane.
2. `bessj1`/`bessy1` `x>=8` asymptotic `Q1` third coefficient transcribed as
   `8.449199096e-5` instead of NR's `0.8449199096e-5` (= `8.449199096e-6`, 10x).
3. `bessy0` `x<8` log-term grouping: Excel associates `c*(J0(x)*ln x)`, not
   `(c*J0(x))*ln x` (1 ULP at `x=1.5`, `x=3`).

Excel-side table quirks (Excel's J and Y asymptotic tables are separately-typed copies
with *different* typos; matched deliberately, per the never-accept-divergence policy):

4. Excel's **Y0** `Q0` last coefficient is `-0.934945152e-7` (`…945…`); its **J0** copy
   keeps NR's `-0.934935152e-7`. (`4773` ULP at `BESSELY(10,0)` before matching.)
5. Excel's **J0** `P0` `y^1` coefficient transposes NR's `…628627` into `…628267`
   (`-0.1098628267e-2`), and Excel's **J1** `P1` table is six entries — an inserted
   duplicate of the `y^2` coefficient `-0.3516396496e-4` shifting the remaining NR
   terms down one slot. This costs Excel `~8e-6` absolute accuracy near `x=8`; OxFunc
   now reproduces it bit-for-bit.

## Validation (live Excel 16.0 b19929, cell-ref plumbing, 2026-07-02)

- BESSELY: `Run-BesselyResweep.ps1` run
  `smart-fuzzer/runs/oxf-xp6p-bugfunc024-repair-validation-20260702/` — `93` cases,
  `93` exact typed bit matches, `0` drifts (was `1`/`92`).
- BESSELJ: 56-row two-way probe (`tools/elem-probe/run-elem-probe.ps1`; orders
  `0,1,2,3,5,10`, `x` in `[1.5,400]`) — all bit-exact **except** `BESSELJ(50,0)`,
  `BESSELJ(150,0)` (1 ULP) and `BESSELJ(50,2)` (2 ULP, recurrence-inherited), which
  are **not Bessel defects**: Excel's `COS` is 1 ULP off ours at the reduced arguments
  `49.214601836` / `149.214601836` (probed directly: Excel `COS` `0x…5409`/`0x…d970`
  vs UCRT/Rust `0x…5408`/`0x…d96f`, `SIN` exact). J0 inherits `cos` at full weight
  (`cos*ans1` dominant). Tracked with the large-argument trig lane (BUG-FUNC-027
  CLASS-C3); those rows close automatically when SIN/COS parity lands.
- Similar-risk scan: BESSELI/BESSELK (shared family, `x>=8` previously unprobed) —
  18/18 live rows bit-exact; no action.
- Lane A: witness-table regressions
  `bessel_convert_family::tests::bessely_matches_live_excel_bits_across_all_lanes`
  (15 witnesses) and `besselj_matches_live_excel_bits_across_all_lanes` (18 witnesses).

## Closure Checklist
- [x] fix landed or non-OxFunc ownership recorded
- [x] validation recorded (93/93 BESSELY resweep + BESSELJ/I/K probes, 2026-07-02)
- [x] root cause recorded (port typos + Excel-side divergent J/Y table copies)
- [x] similar-risk scan recorded (BESSELI/BESSELK 18/18 exact; BESSELJ residual rows
      reassigned to the trig lane with direct COS witnesses)
- [x] spec/matrix/contract updated if required (catalog G3 rows removed; G4 trig row
      updated with COS witnesses; KED-BESSEL-001 closed)
- [x] handoff filed if required (none — BESSELJ residual folded into BUG-FUNC-027 C3)

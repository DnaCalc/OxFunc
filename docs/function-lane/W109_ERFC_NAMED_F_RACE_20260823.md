# ERFC named-F race — 2026-08-23

Lane: W109. Plan: `W109_ERFC_BODY_ATTACK_PLAN_20260823.md`.
Racer: `smart-fuzzer/tools/calc_graph_racer/src/bin/race_erfc_named_f.rs`.
Oracle: frozen discovery only. Heldouts not named, not opened.

## Status axes

- `execution_state`: `in_progress`
- `scope_completeness`: `scope_partial`
- `target_completeness`: `target_partial`
- `integration_completeness`: `partial`
- `open_lanes`: ERF/ERFC.PRECISE body still unidentified; no production
  landing from this race

## Corpus

15,556 distinct nonnegative z rows: 870 direct `ERFC.PRECISE` (erfcp, erfcm,
b7erfc, b8erfc, b11c) merged with implied Q from `answers-b24-normref.json`
(`Q = 2·NORMSDIST` on all-negative x, `z = |x|·RN(1/√2)`). Direct bits win
on the one conflict (`z ≈ 1.1842387490730035`, 6 ULP). That conflict means
the G-F3 round-trip is not a free extra ERFC oracle at every z.

## Scoreboard (exact / n, max ULP)

| Graph | z<0.5 | [0.5,4) | z≥4 | all |
|---|---|---|---|---|
| NSWC DERFC · `excel_exp` | 1370/1864 max 4 | **2535/7741 max 7** | 1466/5951 max 183 | **5371/15556** |
| NSWC DERFC · host exp | 1370/1864 max 4 | 2531/7741 max 7 | 1468/5951 max 183 | 5369/15556 |
| Cody erfcx · unsplit `excel_exp` | 1085/1864 max 4 | 2512/7741 max 11 | **1498/5951 max 183** | 5095/15556 |
| Cody erfcx · unsplit host exp | 1085/1864 max 4 | 2509/7741 max 11 | 1492/5951 max 183 | 5086/15556 |
| `libm::erfc` | 1318/1864 max 3 | 1976/7741 max 12 | 112/5951 max 492 | 3406/15556 |
| production `excel_erfc` fit | 1318/1864 max 3 | 1587/7741 max 37 | 119/5951 max 488 | 3024/15556 |
| Cody CALERF **split** exp | 1081/1864 max 4 | 1797/7741 max 12 | 111/5951 huge | 2989/15556 |

Sources: NSWC TR 92/425 DERFC/DERFC0 (Morris; public NSWC library).
Cody 1969 / SPECFUN CALERF (netlib specfun/erf). Unsplit means
`excel_exp(−RN53(z·z)) · F(z)` with F = published erfcx / DERFC0.

## What this kills or constrains

1. **Do not land NSWC or Cody.** Best named graph is 34.5% exact. Max 183
   ULP on the tail is a systematic F bias (worst row `z ≈ 12.1235`,
   same binade, 183 ULP apart), not a flush miss.
2. **The production `libm*(1+corr(s))` fit is worse than raw libm** on
   this dense corpus (3024 vs 3406). Fitting is the wrong object. Do not
   enlarge the Horner table.
3. **Split-argument exp is dead again** (Cody AINT/16): tail max ULP in
   the 4e15 class from XBIG hard-zero vs Excel's subnormal flush.
4. **Identified `excel_exp` vs host exp is a ~10-row effect**, not the
   body. The leftover is F.
5. **Cody C/D ±1 ULP scan on the mid-band is noise, not a last-bit
   rounding of the published decimals.** Baseline 2512/7741. Best single
   poke is `C[2] −1` → 2526 (+14). Several pokes lose 30–90 exact. A
   true “Excel stored this decimal one ULP off” identification would
   move hundreds of rows, not fourteen.
6. **None of the five F-body pin z values is bit-exact** on any named
   graph. Closest: NSWC at z=0.75 is 1 ULP; z=5 is 5 ULP for every
   unsplit family.

## Test 1 — NSWC Horner association / store-site (2026-08-23)

Racer: `race_erfc_nswc_assoc.rs`. Same 15,556-row corpus. Coefficients
unchanged. Axes from `race_erf_precise_public_small`.

| Cfg | small | mid | tail | all |
|---|---|---|---|---|
| Native53 (named-F baseline) | 1370/1864 | 2535/7741 | 1466/5951 | **5371/15556** max 183 |
| X87Continuous Horner | 1325/1864 | **3072/7741** | 1468/5951 | 5865/15556 max 184 |
| X87EveryOp53 / Pc53 | 1370 | 2535 | 1466 | = native |
| best: X87Continuous + x87-DR `z²` + store `u/v` + cuts 0.5 / 1.5 / 3.5 | 1325/1864 | **3332/7741** | 1470/5951 | **6127/15556** max 89 |

Pins on the best cfg: 2–4 ULP, none exact.

**Verdict: constraint, not an identification.** x87-continuous Horner
beats SSE native by +494 exact (mid-band). Fortran cuts at 1 and 2 are
not Excel’s: forcing small/mid at 0.5 / 1.5 adds more mid-band hits.
Still 39% exact, max 7 ULP on mid and 89 on tail. Do not land a
non-published cut. Do not land x87 NSWC.

## Test 2 — SLATEC / MATH77 Chebyshev DERFC (2026-08-23)

Racer: `race_erfc_slatec_cheb.rs`. Coefficients from netlib MATH77
`derf.f` (Fullerton FNLIB packet). Unsplit `excel_exp`. INITDS at
`0.1·eps` gave nterf=12, nterc2=23, nterfc=24.

| Graph | small | mid | tail | all |
|---|---|---|---|---|
| SLATEC Chebyshev unsplit | **1419/1864** max 2 | 2431/7741 max 6 | 1299/5951 max 183 | 5149/15556 |
| MATH77 IEEE (Cody PS/QS on (0.5,1]) | 1419/1864 | 2578/7741 max 7 | 1299/5951 | 5296/15556 |
| SLATEC nter 12/24/25 | 1419/1864 | 2457/7741 | 1294/5951 | 5170/15556 |
| NSWC native (test 0) | 1370/1864 | 2535/7741 | 1466/5951 | 5371/15556 |
| NSWC x87 best (test 1) | 1325/1864 | 3332/7741 | 1470/5951 | **6127/15556** |

Pins: 2–4 ULP, none exact.

**Verdict: kill Chebyshev-FNLIB as the ERFC body.** It loses overall to
NSWC and is far from the x87-NSWC ceiling. It *does* win the small band
(1419 vs NSWC 1370) — the Fullerton ERF Chebyshev is a better named
P-side series than NSWC A[21], still not bit-exact. No DCSEVL
association pass: even a +500 mid-band store-site lift would not beat
x87 NSWC or hit the pins. Do not mix SLATEC F with NSWC cuts.

## Test 3 — extra RN53 stores on branch 190, small-z (2026-08-23)

`check_erf190` 1024-config spill mask on 1508 ERF.PRECISE `z<0.5` rows
(b9train/erfp/erfm/b7/b8/b10/b11; heldout untouched):

Best 850/1508 (56%). Mask: `zz` stored, series extended, `j` extended,
`gam1` ext then return-dbl, `g` ext, `w` ext, `(w*g)*inner`, inner
extended. Misses are the known ±1 ULP comb.

`race_erf190_inner.rs` then raced inner A `0.5+(0.5−j)` vs B `RN53(1−j)`
and w = chain / reuse-z / `excel_exp(0.5 ln)`, with `gam1(½)` **pinned**
to `h = 0x3fc06eba8214db6b` (no new constant). Ceiling 532/1508 P-side
(worse than the 1024-mask best — pinning + dropping the winning series
spill is a regression). A vs B tied to 1 row (j is invisible on almost
the whole bank). `excel_exp(0.5 ln)` max 16 ULP, killed. ERFC as
ordinary `1−P`: 383/515, max 2 ULP.

**Verdict: constraint only.** Extra RN53 stores on the published 190
dataflow do not close small-z ERF/ERFC. The leftover is the same comb.
Do not invent a tiny-only constant. Do not land.

## After all three

No production kernel change. Body still scaffolding. Three-axis:
`scope_partial` / `target_partial` / `partial`. Heldouts sealed.

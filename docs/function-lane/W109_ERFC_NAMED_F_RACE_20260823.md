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

## Next (still public-source)

1. Association / store-site race of the NSWC DERFC0 Horner (per-op RN53
   vs x87 PC64 vs stage-spill), same style as
   `race_erf_precise_public_small`.
2. Named Chebyshev packet: SLATEC / MATH77 `DERFC` (Fullerton FNLIB,
   netlib). Different form from Cody/NSWC rationals.
3. Do not PSLQ the implied `F = Q / excel_exp(−z²)` until the 0.9-ULP
   comb is subtracted; raw PSLQ will fit the comb.
4. Small-z stays on extra RN53 stores in branch 190, not a new rational.

No kernel change. Heldouts remain sealed.

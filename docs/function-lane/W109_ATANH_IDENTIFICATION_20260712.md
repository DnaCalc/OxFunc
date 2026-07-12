# W109: ATANH identification (2026-07-12)

Live oracle Excel 16.0 build 20131. Corpora: G4-hyp (107) + G4-02 band (77)
+ gap (146) + switch (42) = ~350 distinct rows. A two-approach multi-agent
workflow lane (atanh-A, atanh-B) OVERTURNED the prior "custom odd rational"
scoping; confirmed and extended by a Rust x87 racer (check_atanh_full.rs).

## The overturned premise

The `+6.3..-3.4` ULP equioscillation on `|x|<0.5` was read as a minimax
rational's approximation error. It is NOT: it is the **conditioning floor of
the ratio `(1+x)/(1-x)`**, which -> 1 as x -> 0, amplifying the argument's
rounding into a ULP error that GROWS toward small x (the opposite of a
polynomial's error, which vanishes). No odd polynomial `x·P(x²)` or rational
`x·P/Q` of any moderate degree fits (LS residual >= 1380 ULP RMS to deg 19,
`0/52` bit-exact) — the residual is non-smooth ratio-rounding noise, not
approximation error. Exact Cephes P/Q scores `3/32` on the band.

## The kernel (three regions)

- **Region C** (`|x| >= ~1.25e-4`, incl. the whole catalog mid-small band and
  the near-1 rows): `ATANH(x) = 0.5·ln((1+x)/(1-x))`, ratio formed in binary64
  (double-rounding LOAD-BEARING — a higher-precision ratio scores `22/57`),
  ln via the x87 CRT chain. **163/163 bit-exact.** PROMOTED into atanh.rs.
- **Region B** (`|x| <= ~9.0e-5`): Excel's x87 `fyl2xp1` ln1p pair
  `0.5·(ln1p(x)-ln1p(-x))`, extended temporaries with a SINGLE final store —
  **175/175 bit-exact** on every live region-B row. **PROMOTED** into `atanh.rs`
  via the production `excel_atanh_small` helper (`excel_numeric`); the naive ratio
  catastrophically cancels here (thousands of ULP). Passthrough (`atanh(x)->x` for
  subnormal x) is emergent from this form; the pair is exactly odd (negative row =
  sign-flip). The SSE2 double-double log1p pair scores only `133/175` here, so the
  provider is decisively the x87 hardware `fyl2xp1`, consistent with Excel's
  EXP/LN/POWER transcendental family.
- **Passthrough** (`|x| <~ 1.66e-8`): returns `x`; emergent from region-B
  accuracy (`atanh(x) -> x` while `x³/3 < ½ ulp`), not a hard branch. Corpus:
  `x=1.0985e-8` passes, `x=1.42805e-8` does not.
- **Transition/switch band** (3 distinct `|x|`: `9.563e-5, 9.9996e-5, 1.0137e-4`;
  6 rows): NEITHER the pair NOR the ratio matches. Sharpened 2026-07-12: the pair
  is exact up to `9.02e-5` and the ratio is exact from `1.0745e-4` up, with NO
  rows in `(9.02e-5, 9.56e-5)` — a clean gap containing exactly these 3 values.
  In the band, Excel is `+1` ULP above the x87 extended pair on ALL 6 rows, and
  the SSE2 log1p pair / the binary64 fdlibm-style argument `2x+2x²/(1-x)` via
  fyl2xp1 each hit `4/6` (the same 4). This is a sub-½-ULP internal-log1p switch
  regime; 3 points cannot disambiguate the exact staging without overfitting.
  Open, gated on dense probes.

## Excel ATANH is NOT globally odd

Live: `ATANH(-0.2) = 0xbfc9f323ecbf984a`, but `-ATANH(0.2) = 0xbfc9f323ecbf9849`
— 1 ULP apart. Excel evaluates the SIGNED ratio directly; the negative argument
rounds independently. The prior `abs().atanh().copysign()` forced an oddness
Excel does not have, i.e. it introduced a divergence. The signed ratio removes it.

## Scores

Piecewise (x87 pair below T, ratio-log above): best `344/350` at `T≈1.0e-4..1.05e-4`,
the 6 residuals all in the switch band. **Region B `175/175` and region C `163/163`
both PROMOTED** (2026-07-12): the production `atanh_kernel` now uses `excel_atanh_small`
(x87 fyl2xp1 pair) below `1.05e-4` and the x87 ratio-log at/above it. Full corpus
`344/350`; the only open rows are the 6 band rows at `+1` ULP.

## Next

Close the band: harvest dense adjacent-double live probes across `[8e-5, 1.3e-4]`
both signs (≥60/side) to pin the exact B→C switch double and identify the band
micro-path (candidates: SSE2 double-double log1p, fdlibm-style argument, or a third
transitional path — each currently `4/6`). Low ROI (6 rows) vs the rest of W109, so
deferred to a cleanup pass. ACOTH (G4-03) inherits the region-B x87 fyl2xp1 path —
see the ACOTH racer follow-up; literal `ATANH(1/x)` forming `1/x` first is ruled out
(`18/57`).

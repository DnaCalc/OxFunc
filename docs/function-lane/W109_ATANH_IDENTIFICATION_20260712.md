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
- **Region B** (tiny `|x|`, roughly `1.66e-8 .. ~1e-4`): a distinct accurate
  path, empirically Excel's x87 `fyl2xp1` ln1p-difference `0.5·(ln1p(x)-ln1p(-x))`;
  the naive ratio catastrophically cancels here (millions of ULP). The x87
  fyl2xp1-pair reproduces most of it; residual `±1` ULP rows are the microcode
  vs Excel's exact log1p. NOT yet bit-exact; platform path retained (no regression).
- **Passthrough** (`|x| <~ 1.66e-8`): returns `x`; emergent from region-B
  accuracy (`atanh(x) -> x` while `x³/3 < ½ ulp`), not a hard branch. Corpus:
  `x=1.0985e-8` passes, `x=1.42805e-8` does not.
- **Transition band** (`|x| ~ 1e-4`): 6 rows where NEITHER the pair NOR the
  ratio matches, and where Excel is provably **not odd** (miss `+521` at `+x`
  vs `+3589` at `-x`). A genuinely distinct micro-path; open.

## Excel ATANH is NOT globally odd

Live: `ATANH(-0.2) = 0xbfc9f323ecbf984a`, but `-ATANH(0.2) = 0xbfc9f323ecbf9849`
— 1 ULP apart. Excel evaluates the SIGNED ratio directly; the negative argument
rounds independently. The prior `abs().atanh().copysign()` forced an oddness
Excel does not have, i.e. it introduced a divergence. The signed ratio removes it.

## Scores

Piecewise (x87 pair below T, ratio-log above): best `344/350` at `T≈9.5e-5..1.05e-4`,
the 6 residuals all in the non-odd transition band. Region C alone `163/163`.
Promotion boundary chosen conservatively at `1.25e-4` (region C proven `163/163`
there) with the platform path retained below (strict improvement, zero regression;
full corpus `186 -> 301/350`, region C `50 -> 163/163`).

## Next

Pin the exact B->C switch and the transition micro-path; port the x87 fyl2xp1
ln1p-pair into a production helper to close region B; then lower the promotion
floor to Excel's true switch and reach `350/350`. ACOTH inherits region B's
difficulty (literal `ATANH(1/x)` ruled out — forming `1/x` first scores `18/57`).

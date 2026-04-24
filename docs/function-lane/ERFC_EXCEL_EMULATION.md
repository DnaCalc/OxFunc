# ERFC / ERFC.PRECISE — Excel emulation notes

## Policy

DnaCalc is an Excel calculation emulator. For direct-function exactness
work, the acceptance criterion is bit-exact reproduction of Excel's
observed output doubles, even when Excel is numerically inferior to
correctly-rounded `libm` / scientific math results. Mathematical
correctness is diagnostic, not acceptance.

## Current rule (commit `<this>`, supersedes `4bedeac`)

Empirical correction-polynomial fit to Excel's observed ratio
`corr(s) = excel/libm - 1` at `s = 1/x²`, applied as:

```
excel_erfc(x) = libm::erfc(x) * (1 + corr(s))  for x >= 1.25
excel_erfc(x) = libm::erfc(x)                  otherwise
```

Piecewise in two regions split at the fdlibm-style boundary `x = 2.857`:

| region | x range | degree | samples | coefficients |
|---|---|---|---|---|
| B | [1.25, 2.857) | 8 | 25 | 9 coeffs |
| A | [2.857, ∞) | 2 | 20 | 3 coeffs |

Normalization: `u = 2*(s - s_lo) / (s_hi - s_lo) - 1`, Horner evaluation
ascending-degree. Fitted via weighted least-squares: matched-anchor
points carry weight 1e12 so the fit is forced through them exactly,
preserving every previously-green witness. All other points weight 1.

**No Windows-only path**, no UCRT FFI, no `cfg`-gating of the hot kernel.
The fit was calibrated using Excel values captured on Windows (including
UCRT-anchored x=3, 4, 8) but the compiled kernel is deterministic on
every platform.

## Evidence summary (widened 48-point positive witness set)

| kernel variant | exact matches | notes |
|---|---|---|
| libm::erfc baseline | 9 | |
| Windows-MSVC UCRT at x>=3 (commit `4bedeac`) | 12 | platform-only |
| **correction-fit kernel (this commit)** | **20** | cross-platform, 0 regressions |

Newly matched by the correction fit vs commit `4bedeac`:
`1.5, 1.8, 2.15, 2.25, 2.4, 2.5, 2.99, 3.25`

All 12 previously-green anchors preserved:
`0, 0.5, 1, 1.25, 1.85, 1.95, 2.75, 2.8, 3, 3.001, 4, 8`

Negative tail: every tested negative (−1.25 … −10) continues to match
via libm unchanged — negatives route through `libm::erfc` directly.

## Remaining blocked set

28 positive-x points where the correction-fit kernel still does not
reproduce Excel's bits:

```
1.6, 1.7, 1.75, 1.9, 2.0, 2.05, 2.1, 2.35, 2.45, 2.55, 2.6, 2.65, 2.7,
2.85, 2.9, 2.95, 2.999, 3.005, 3.01, 3.02, 3.5, 3.75, 4.5, 5, 6, 7, 9, 10
```

Their exact Excel bits are captured in `#[ignore]`d
`erfc_remaining_blocked_excel_witnesses`. Worst residual is 6 ULP.

The residual pattern is chaotic at the 1-ULP level: `libm/excel - 1` is
±(0.5–3) × 2⁻⁵² with sign and magnitude varying non-monotonically between
adjacent x values (e.g. near x=3 the 0.001-resolution sweep produced
`2.999 B / 3.000 M / 3.001 M / 3.005 B / 3.01 B / 3.02 B`). No smooth
polynomial captures that chaos. Further progress on this residual likely
requires either:

1. Excel's actual polynomial coefficients from an authoritative source.
2. A much denser witness set (O(100s) of positive-x points) and a
   higher-degree piecewise fit with region subdivision near every
   observed Matched/Blocked transition.

## Threshold behavior

Near the region boundary x = 2.857:

- The fit training used Region B samples up to x = 2.85 and Region A
  samples from x = 2.9. Runtime x in (2.85, 2.857) extrapolates slightly
  outside Region B training range (u slightly below -1); runtime x in
  [2.857, 2.9) extrapolates slightly above Region A training range
  (u slightly above +1). Both polynomials remain bounded in these
  regions — no DnaOneCalc witness yet to confirm what Excel does there.

## Verification protocol

After any change to `excel_erfc`:

1. Run the bit-exact in-tree witnesses (both platforms):
   ```
   cargo test -p oxfunc_core --lib special_dist_family::tests::erfc
   ```
   All non-ignored tests must pass. No compile-time OS gating.

2. Request a DnaOneCalc proof batch across at minimum:
   - Matched anchors (must stay Matched): 0, 0.5, 1, 1.25, 1.85, 1.95,
     2.75, 2.8, 3, 3.001, 4, 8 and negative mirrors.
   - Newly-matched positives (claim): 1.5, 1.8, 2.15, 2.25, 2.4, 2.5,
     2.99, 3.25.
   - Threshold probes: 2.85, 2.855, 2.857, 2.86 (not in witness set —
     extrapolated).
   - Several blocked points to confirm no weird drift: 1.9, 2.5, 3.5,
     5, 10.

3. Any flip from Matched → Blocked at a previously-matched anchor is a
   regression blocking the commit.

## Reversibility

All relevant code is in
`crates/oxfunc_core/src/functions/special_dist_family.rs`:

- `const ERFC_B_S_MIN` / `MAX`, `ERFC_B_COEFFS`
- `const ERFC_A_S_MIN` / `MAX`, `ERFC_A_COEFFS`
- `const ERFC_BOUNDARY_X`
- `fn erfc_horner`, `fn excel_erfc`

Revert is `git revert <this>` or restoring `erfc_kernel` to
`Ok(libm::erfc(x))` and removing the constants.

## Fit methodology (for reproduction)

Scratch probe `probe_rational_fit_attempt` (not committed) builds the fit
from the ALL_WITNESSES table. Steps:

1. For each witness `(x, excel)`, compute `corr = excel / libm::erfc(x) - 1`.
2. Partition by region at x = 2.857.
3. Normalize `s` to `u ∈ [-1, 1]` in each region.
4. Weight each sample: `1e12` if `libm::erfc(x) == excel` (or UCRT matches
   on Windows-MSVC and x >= 3), else `1`.
5. Solve normal-equation weighted least squares for polynomial
   coefficients (degrees 8 and 2 selected via sweep for max hits with
   zero regressions).

To recalibrate after a new witness batch lands, extend `ALL_WITNESSES`
in the probe file, rerun `cargo test -p oxfunc_core --lib
probe_rational_fit_attempt -- --ignored --nocapture`, read out the new
`s_min`/`s_max`/coefficient values, and replace the constants.

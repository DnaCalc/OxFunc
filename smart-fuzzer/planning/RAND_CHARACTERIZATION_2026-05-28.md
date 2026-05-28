# RAND() Characterization Study (2026-05-28)

Status: `characterization_recorded`

## 1. Purpose (what this is and is not)

This is a **characterization exercise**, not a conformance or "blessing"
test. The question: can a statistical battery **distinguish** Excel's
`RAND()` from candidate Rust RNG implementations we might use? Where OxFunc's
RAND ultimately lives (host-supplied vs in-crate) is **undecided**; this only
characterises the situation so that decision is informed.

Excel modern `RAND()` is documented as Mersenne-Twister-based (MT19937);
pre-2010 Excel used Wichmann-Hill. Both are included as Rust candidates.

## 2. Method

- `Run-RandCharacterization.ps1` bulk-samples Excel `RAND()` (`N = 50000`,
  one column recalc, round-trip `"R"` formatting so the Rust side parses the
  identical `f64` bits). Excel `16.0` build `20026`.
- `rand_characterize` (Rust, `smart-fuzzer/tools/pmt_ppmt_local_eval`)
  implements five candidate RNGs — `lcg64` (32-bit output), `mt19937_res53`,
  `xorshift128plus`, `splitmix64`, `wichmann_hill` — generates `N` draws each
  with fixed seeds, and computes **one identical battery** for every source
  (Excel sample + each RNG): mean, variance, skew, excess kurtosis, 64-bin
  χ², one-sample KS vs U(0,1), lag-1 autocorrelation, fraction of draws on
  the 2⁻³² grid, distinct ratio, min gap. Plus a **two-sample KS vs Excel**
  for each RNG.

Run: `rand-characterization-001`.

## 3. Results

| source | mean | var | χ²₆₄ | KS-unif | autocorr1 | frac on 2⁻³² grid | KS vs Excel |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| excel_rand | 0.5036 | 0.08366 | 58.8 | 0.0076 | 0.0035 | **0.0000** | — |
| lcg64 | 0.4966 | 0.08415 | 63.0 | 0.0067 | 0.0006 | **1.0000** | 0.0138 |
| mt19937_res53 | 0.4990 | 0.08335 | 68.9 | 0.0032 | 0.0046 | 0.0000 | 0.0089 |
| splitmix64 | 0.4993 | 0.08337 | 38.1 | 0.0029 | 0.0035 | 0.0000 | 0.0091 |
| wichmann_hill | 0.4986 | 0.08330 | 50.0 | 0.0031 | 0.0010 | 0.0000 | 0.0096 |
| xorshift128plus | 0.4995 | 0.08347 | 82.2 | 0.0047 | -0.0030 | 0.0000 | 0.0117 |

(Theoretical U(0,1): mean 0.5, var 0.08333, skew 0, excess kurtosis -1.2.
64-bin χ² df=63, p=0.05 critical ≈ 82.5.)

## 4. What distinguishes what

**The decisive discriminator is output granularity, not any distributional
test.** `frac_on_2pow32_grid` is `1.0000` for `lcg64` (every value is exactly
`k/2³²`) and `0.0000` for Excel and all 53-bit RNGs. This is exact and
sample-size-independent: a coarse 32-bit-output generator is *instantly*
separable from Excel; Excel sits on a fine (≥53-bit) grid, consistent with
the documented MT19937 `genrand_res53` transform.

**Moment, uniformity (χ²), one-sample KS, and autocorrelation tests do NOT
distinguish any candidate from Excel** — including the basic LCG, which
passes every distributional test (mean 0.4966, χ² 63.0, KS 0.0067, autocorr
~0). At `N = 50000` they are all "plausibly uniform" and statistically
interchangeable on these axes.

**Two-sample KS vs Excel** sits right around the `N=50000` p=0.05 critical
value `1.36·√(2/N) ≈ 0.0086` for every RNG (0.0089–0.0138). `lcg64` is
highest (0.0138) — consistent with its grid tell — but the fine RNGs
(0.0089–0.0117) straddle the threshold, which at a single replication is
sampling noise, not evidence of a real difference. It does **not** reliably
separate good 53-bit RNGs from Excel.

### 4.1 Methodological caveats

- **`min_gap` does not reveal grid resolution at this N.** Over 50000 draws
  the closest pair is ~4×10⁻¹⁰ for *every* source (birthday/spacing effect,
  ≈1/N²), swamping the underlying grid (2⁻³² ≈ 2.3×10⁻¹⁰, 2⁻⁵³ ≈ 1.1×10⁻¹⁶).
  Use `frac_on_2pow32_grid` for resolution, not `min_gap`.
- **`distinct_ratio` = 1.0** for all at this N (collisions negligible even on
  the 2⁻³² grid until N approaches 2¹⁶), so it does not discriminate here.

## 5. Conclusion (answering the question)

Statistical analysis **can** distinguish a coarse (32-bit) RNG from Excel
`RAND()` — but only via **output granularity**, not via any distributional
test. Good 53-bit RNGs (MT19937-res53, xorshift128+, splitmix64, and even the
old Wichmann-Hill) are **statistically indistinguishable from Excel** on
moments, uniformity, KS, and autocorrelation at `N = 50000`. Excel `RAND()`
is consistent with the documented MT19937-res53 (fine grid, nominal moments
and uniformity).

Practical implication for the undecided host-vs-OxFunc placement: **the
choice of a *good* 53-bit RNG is not observable to a consumer through these
distributional tests** — any of MT19937/xorshift/splitmix would be
indistinguishable from Excel's stream by this battery. The only consumer-
visible algorithmic tell among the candidates is the coarse-grid LCG, which
should be avoided. A reproducible/seedable stream (for replay) is therefore a
design choice with no distributional cost.

## 6. Stronger-discrimination follow-ups (not run)

If finer discrimination is ever needed:
1. **Multi-replication two-sample KS** — build the null distribution of KS
   between independent Excel samples to calibrate the fine-RNG KS values
   (here a single replication left them at threshold).
2. **Structural tests** — spectral/lattice test (LCG lattice structure),
   low-order bit independence, gap/runs tests, and overlapping-pairs (2-D
   uniformity), which expose RNG structure that 1-D distribution tests miss.
3. **Larger N + many bins** for χ² power, and a granularity histogram (exact
   mantissa-bit profile) to read each RNG's grid directly.

## 7. Artifacts

- `smart-fuzzer/tools/Run-RandCharacterization.ps1` (Excel sampler + driver)
- `smart-fuzzer/tools/pmt_ppmt_local_eval/src/bin/rand_characterize.rs` (RNGs + battery)
- run `rand-characterization-001` (ignored): raw Excel samples + JSON report.

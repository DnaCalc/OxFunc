# Excel ↔ Ideal-Math Deviation Catalog

Status: `active_accumulating`
Started: `2026-06-22`

## Purpose

A standing catalog of functions that have a **clear mathematical / semantic definition**
but where **Excel's actual evaluation deviates from the mathematically-most-accurate `f64`
result** — and OxFunc, by its bit-exact-emulation charter, *deliberately reproduces Excel's
deviation*.

This is **interesting, durable knowledge about Excel's numerical character**, not a bug list.
The deviation is the *correct* OxFunc behaviour (matching Excel is the goal); this catalog
exists to make Excel's non-ideal choices **explicit and explained**, so that:

- a future reader is not surprised that OxFunc returns `#NUM!` / a 1-ULP-off value where the
  textbook math is finite / more accurate;
- a future kernel change does not "improve" accuracy and silently break Excel parity;
- the set of Excel's known numerical idioms (overflow-in-the-naive-formula, `pow`-not-`sqrt`,
  integer-exponent repeated-multiplication, …) is collected in one place.

## What belongs here (and what does not)

**Belongs here:** a function with a clean math definition where Excel's evaluation is *less
accurate than* — or *algorithmically different from* — the most-accurate `f64` evaluation, and
OxFunc matches Excel. Excel is the deviating party; OxFunc agrees with Excel.

**Does NOT belong here:**
- **Open OxFunc≠Excel discrepancies** (OxFunc has a *bug*) → those live in
  [`OXFUNC_EXCEL_DISCREPANCY_CATALOG.md`](OXFUNC_EXCEL_DISCREPANCY_CATALOG.md) and are removed
  on sign-off. This catalog is the opposite: OxFunc *agrees* with Excel; the entry stays.
- **FP-surface normalization quirks** (signed-zero display, `#DIV/0!` vs `±Inf`, flush-to-zero)
  → those are in
  [`function-lane/FLOATING_POINT_LEAN_EXCEL_DEVIATION_LEDGER.csv`](function-lane/FLOATING_POINT_LEAN_EXCEL_DEVIATION_LEDGER.csv).
- **Cases where Excel is *more* accurate than the naive formula** (e.g. a proprietary
  high-accuracy method) → note them under "Inverse class" below if striking, but the focus
  here is Excel-deviates-from-most-accurate.
- **General rules of thumb** → [`OXFUNC_FIX_LEARNING_LOG.md`](OXFUNC_FIX_LEARNING_LOG.md).

## Maintenance

- **Accumulating, not open-tracking.** Entries are permanent facts about Excel; do NOT remove
  an entry when the OxFunc kernel that reproduces it lands — landing the reproduction is what
  *adds* the entry's confidence, not what retires it.
- **Add as discovered.** When an Excel-vs-ideal-math deviation is established (ideally
  oracle-verified against live Excel), add a row.
- **Evidence is live-Excel.** Each entry cites the Excel build, the oracle probe, and the
  OxFunc kernel/policy + bead/commit that reproduces it.

## Entry schema

Each entry records: **id · function(s) · math definition · most-accurate f64 evaluation ·
Excel's deviation (what + magnitude) · cause (Excel's internal method) · input domain ·
OxFunc reproduction (kernel/policy) · evidence**.

---

## Entries

### XMD-001 — ASINH overflows to `#NUM!` where the result is finite

- **Function:** `ASINH(x)`
- **Math definition:** `asinh(x) = ln(x + √(x²+1))`; finite and well-defined for *all* finite
  `x` (e.g. `asinh(1e308) ≈ 709.89`).
- **Most-accurate f64:** a scaled/`ln1p`-based kernel returns the finite value across the whole
  `f64` range without overflow.
- **Excel's deviation:** returns **`#NUM!`** for `|x|` beyond the point where `x²` overflows,
  i.e. `|x| > √(f64::MAX) ≈ 1.3407807929942596e154` (finite at/below `1.3e154`, `#NUM!`
  at/above `1.4e154`). Excel is *strictly less capable* than the math here.
- **Cause:** Excel evaluates the literal `ln(x + √(x²+1))`; the `x²` (or `x²+1`) intermediate
  overflows `f64` → `#NUM!`, rather than using an overflow-free reformulation.
- **Input domain:** `|x| > √(f64::MAX) ≈ 1.34e154`.
- **OxFunc reproduction:** `asinh_kernel` forms the same `x*x` and returns `#NUM!` when it is
  non-finite — flipping at exactly Excel's input.
- **Evidence:** live Excel 16.0 build 20026 (`.tmp/asinh-sqrtpi-oracle2.ps1`); bead `oxf-7m1k`,
  commit `77431cf`.

### XMD-002 — SQRTPI overflows to `#NUM!` where the result is representable

- **Function:** `SQRTPI(n) = √(n·π)`
- **Math definition:** finite for all `n ≥ 0`; the *result* `√(n·π)` is representable up to
  `n = f64::MAX` (`≈ 1.77e154`).
- **Most-accurate f64:** compute `√(n·π)` without forming the overflowing `n·π` product.
- **Excel's deviation:** returns **`#NUM!`** once the `n·π` *intermediate* overflows, i.e.
  `n > f64::MAX/π ≈ 5.7222e307` (finite at/below `5.72e307`, `#NUM!` at/above `5.8e307`) —
  even though the final square root would be finite.
- **Cause:** Excel forms `n·π` in `f64` first; that product overflows → `#NUM!`.
- **Input domain:** `n > f64::MAX/π ≈ 5.72e307`.
- **OxFunc reproduction:** `sqrtpi_kernel` forms the same `n*π` and returns `#NUM!` when
  non-finite.
- **Evidence:** live Excel 16.0 build 20026 (`.tmp/asinh-sqrtpi-oracle2/3.ps1`); bead
  `oxf-7m1k`, commit `77431cf`.

### XMD-003 — SQRTPI computed via `pow`, not correctly-rounded `sqrt`

- **Function:** `SQRTPI(n) = √(n·π)`
- **Math definition / most-accurate f64:** the IEEE **correctly-rounded** square root of the
  `f64` product `n·π` (what `sqrtsd` / Rust `.sqrt()` gives).
- **Excel's deviation:** Excel computes `(n·π)^0.5` through its **`pow`** routine, which is
  **not** correctly-rounded. It agrees with correctly-rounded `sqrt` at normal magnitudes but
  is **1 ULP off at scattered points near overflow** — e.g. the `n·π == f64::MAX` input
  `0x7fd45f306dc9c882`: correctly-rounded `sqrt` = `0x5fefffffffffffff` (the *more* accurate
  value), Excel's `pow` = `0x5ff0000000000000` (1 ULP high); also `0x7fd45f306dc9c880`.
- **Cause:** `pow(x, 0.5)` (via `exp(0.5·ln x)` or similar) carries rounding error that a
  dedicated correctly-rounded `sqrt` does not. **Not** x87/extended precision — 64-bit Excel
  and Rust are both SSE2.
- **Input domain:** scattered single inputs, concentrated near the overflow boundary
  (`n ≈ 5.7e307`); negligible practical reach, but real.
- **OxFunc reproduction:** `sqrtpi_kernel` uses `(n·π).powf(0.5)` (Rust's `powf` reproduces
  Excel's `pow` bit-for-bit: 30/30 sampled inputs).
- **Evidence:** live Excel 16.0 build 20026 (`.tmp/sqrtpi-broad-oracle.ps1` +
  `sqrtpi-boundary-oracle.ps1`); bead `oxf-quxx`. See heuristic in `OXFUNC_FIX_LEARNING_LOG`.

### XMD-004 — POWER publishes integer exponents via repeated multiplication

- **Function:** `POWER(x, n)` / the `^` operator
- **Math definition / most-accurate f64:** `x^n` via the transcendental path
  (`exp(n·ln x)` / `powf`).
- **Excel's deviation:** for an **exact-integer** exponent `n`, Excel computes `x^n` by
  **repeated multiplication (binary exponentiation)**, producing a bit-different — and
  "rounder", Excel-matching — result than the transcendental path. (Excel here is not
  necessarily *less* accurate; it is *algorithmically different* from the obvious
  most-accurate single-call evaluation.)
- **Cause:** integer-exponent special-casing in Excel's power evaluation.
- **Input domain:** exact-integer exponents.
- **OxFunc reproduction:** `power_kernel` reads `POWER_META.precision_rounding_profile =
  PrecisionRoundingProfile::IntegerExponentPublication` (W105 `.8`) and uses the
  binary-exponentiation publication path.
- **Evidence:** live Excel 16.0 build 20026 (e.g. `POWER(1.05,10)`); W105 bead `oxf-y2uw.8`,
  commit `edf7c47`.

### XMD-005 — Circular trig → `#NUM!` for `|x| ≥ 2²⁷`

- **Functions:** `SIN`, `COS`, `TAN`, `COT`, `SEC`, `CSC`.
- **Math definition:** periodic; finite (where defined) for every finite `x` — argument
  reduction mod 2π is exact in principle.
- **Most-accurate f64:** a correctly-reduced kernel returns a finite value for any finite `x`.
- **Excel's deviation:** returns **`#NUM!`** once `|x| ≥ 2²⁷ = 134217728` (`134217727` finite,
  `134217728` `#NUM!`, both signs). Excel simply refuses large-argument trig — strictly less
  capable than the math.
- **Cause:** Excel's argument-reduction routine caps at `|x| < 2²⁷`; beyond it → `#NUM!` (a hard
  doctrine guard, not an overflow).
- **Input domain:** `|x| ≥ 2²⁷`.
- **OxFunc reproduction:** `ExcelRealPolicy::CIRCULAR_TRIG` (`ArgDomainGuard::CircularTrigOverflow`,
  `EXCEL_TRIG_MAX_ABS_ARG = 2²⁷`) in `excel_numeric.rs`, read at every dispatch site.
- **Evidence:** live Excel 16.0 build 20026 (bisected to `2²⁷`); BUG-FUNC-027 B2, commit
  `cec679d`. Confidence **high**.

### XMD-006 — `MOD` → `#NUM!` once the quotient `|n/d| ≥ 1.1259e12`

- **Function:** `MOD(n, d)`.
- **Math definition / most-accurate f64:** `n − d·⌊n/d⌋` is finite for any finite `n`, `d≠0`,
  however large the quotient.
- **Excel's deviation:** **`#NUM!`** once the *quotient* magnitude reaches the `d`-independent
  exact double `1125900000000` (`0x4270624de9b00000`) — `MOD(1125899999999.9998,1)` finite,
  `MOD(1125900000000,1)` `#NUM!`. The limit is on the quotient, not on `|n|`.
- **Cause:** Excel recovers the remainder from `n − d·INT(n/d)` and cannot once `INT(n/d)`
  exceeds its internal magnitude limit.
- **Input domain:** `|n/d| ≥ 1.1259e12`.
- **OxFunc reproduction:** `mod_kernel` guards `(n/d).abs() ≥ MOD_QUOTIENT_NUM_LIMIT =
  1_125_900_000_000.0`.
- **Evidence:** live Excel 16.0 build 20026 (bisected boundary); BUG-FUNC-027 B1, commits
  `8dea9cd`/`534c2e6`. Confidence **high**.

### XMD-007 — `ATAN2` → `#NUM!` when the `y/x` ratio overflows

- **Function:** `ATAN2(x, y)`.
- **Math definition / most-accurate f64:** the four-quadrant angle is finite everywhere off the
  origin — `→ ±π/2` as `|y/x| → ∞`. A 2-argument `atan2` returns it without forming the ratio.
- **Excel's deviation:** **`#NUM!`** exactly when `x ≠ 0` and the implicit ratio `y/x` overflows
  to `∞` (`ATAN2(1e-200,1e108)` finite, `ATAN2(1e-200,1e109)` `#NUM!`); the axis case `x = 0`
  stays valid (`±π/2`).
- **Cause:** Excel forms the `y/x` ratio first; the division overflows → `#NUM!`.
- **Input domain:** `x ≠ 0` and `|y/x|` overflows f64.
- **OxFunc reproduction:** `atan2_kernel` guards `x != 0 && (y/x).is_infinite() → #NUM!`.
- **Evidence:** live Excel 16.0 build 20026 (18/18 bit-exact); BUG-FUNC-027 B3, commit `8dea9cd`.
  Confidence **high**.

### XMD-008 — Excel never publishes `±Inf`: overflow / non-finite real result → `#NUM!`

- **Functions:** `EXP`, `SINH`, `COSH`, `FACT`, `FACTDOUBLE`, `DEGREES`, `PERMUTATIONA` (and the
  overflow arm of `POWER`/`^`) — the `ExcelRealPolicy::FINITE` family.
- **Math definition / most-accurate f64:** for large arguments the result is genuinely
  out-of-range; the f64-faithful representation is `±Inf` (e.g. `EXP(1000)`, `FACT(171)≈1.24e309`,
  `SINH(-326648)`, `POWER(10,700)`).
- **Excel's deviation:** Excel **never publishes `±Inf`/`NaN`** — any non-finite real result →
  **`#NUM!`** (`EXP(1000)=#NUM!`, `FACT(171)=#NUM!` vs `FACT(170)` finite). One nuance: in
  `POWER`/`^`, a `±Inf` from a **negative exponent over a sub-unit base** (`1 ÷ underflowed-to-0`)
  → **`#DIV/0!`** instead (`POWER(0.001,-700)`), consistent with `0^negative → #DIV/0!`.
- **Cause:** Excel's no-infinities error-surface convention; sign-of-exponent selects
  `#NUM!` vs `#DIV/0!` for `POWER`.
- **Input domain:** finite inputs whose IEEE result is non-finite.
- **OxFunc reproduction:** `ExcelRealPolicy::FINITE` on each `*_META.real_result_policy`, mapped
  to `#NUM!` by `ExcelRealPolicy::publish` (`excel_numeric.rs`); `power_kernel`'s explicit
  `is_infinite() → (exp<0 ? #DIV/0! : #NUM!)`.
- **Evidence:** live Excel 16.0 build 20026; BUG-FUNC-027 CLASS-A3/A4/A5, bead `oxf-vgxs`,
  commit `b0b2419`. Confidence **high**.
- **Note:** this is the *error-surface convention* class (`+Inf → #NUM!`), distinct from
  XMD-001/002/005/006/007 where the *true result is finite* but Excel's *intermediate* overflows.

### XMD-009 — `POWER`/`^` negative-base reciprocal-odd-root via `−exp(p·ln(−x))`

- **Functions:** `POWER(x, p)` and `^`, for `x < 0` and `p ≈ 1/q` (odd `q`, `3 ≤ q ≤ 255`).
- **Math definition / most-accurate f64:** the real odd root of a negative base; a
  correctly-rounded evaluation gives the clean value (`POWER(-8, 1/3) = -2.0`).
- **Excel's deviation:** Excel routes these through the transcendental composite
  `−exp(p·ln(−x))`, carrying its rounding error → **`POWER(-8,1/3) = -1.9999999999999998`**,
  `POWER(-27,1/3) = -2.9999999999999996` (not `-2`/`-3`).
- **Cause:** integer-root cases go through `exp`/`ln`, not a correctly-rounded root.
- **Input domain:** `x < 0`, `p` within tolerance of a reciprocal odd integer.
- **OxFunc reproduction:** `power_kernel` → `detect_reciprocal_odd_integer` then
  `-((p * (-x).ln()).exp())`.
- **Evidence:** live Excel 16.0 build 20026 (bit-exact test
  `power_kernel_matches_excel_negative_base_reciprocal_odd_root_rows`, `power_fn.rs`). Confidence
  **high**. Companion to XMD-004.

### XMD-010 — `BESSELY`/`BESSELJ` order 0, `x<8`: Excel uses Numerical Recipes' truncated `2/π`

- **Functions:** `BESSELY` order 0 (and the shared `bessj0` substrate), `0 < x < 8`.
- **Math definition / most-accurate f64:** `Y₀(x)` with the full-precision
  `2/π = 0.6366197723675814…`.
- **Excel's deviation:** Excel ships the Numerical-Recipes algorithm verbatim, including its
  **truncated 9-digit** constant `2/π = 0.636619772` (and a rounded `bessj0` coefficient) — *less*
  accurate than exact `2/π`. Using the exact constant makes OxFunc more accurate but diverges by
  up to **~6.2e11 ULP**.
- **Cause:** Excel's Bessel implementation is NR with NR's hard-coded literals.
- **Input domain:** order `n=0`, `0<x<8`.
- **OxFunc reproduction:** `const NR_2_OVER_PI: f64 = 0.636_619_772` in `bessel_convert_family.rs`
  (test `bessely0_small_x_bit_exact_after_nr_constant`).
- **Evidence:** live Excel 16.0 build 20026; BUG-FUNC-024, bead `oxf-xp6p`, commit `5d4b855`.
  Confidence **high** for this sub-lane.
- **Scope caveat:** ONLY the order-0/`x<8` truncated-constant sub-lane is reproduced. The parent
  `BESSELY` (order ≥1, `x≥8`) is still open and is the *opposite* case (Excel's more-accurate
  proprietary method) — see Inverse class.

### XMD-011 — `ERFC`/`ERFC.PRECISE` via Excel's less-accurate proprietary polynomial *(partial)*

- **Functions:** `ERFC(x)`, `ERFC.PRECISE(x)`, positive tail `x ≥ 1.25`.
- **Math definition / most-accurate f64:** correctly-rounded `erfc` (e.g. `libm::erfc`).
- **Excel's deviation:** Excel's bits come from a proprietary polynomial **numerically inferior to
  correctly-rounded libm** — chaotic ±(0.5–3)×2⁻⁵² noise, worst residual ~6 ULP. The
  `ERFC_EXCEL_EMULATION.md` acceptance criterion states it: bit-exact reproduction of Excel "even
  when Excel is numerically inferior to correctly-rounded libm".
- **Cause:** Excel's own erfc polynomial, not libm.
- **Input domain:** `x ≥ 1.25` positive tail (negatives route through libm and already match).
- **OxFunc reproduction:** `special_dist_family.rs` `excel_erfc(x) = libm::erfc(x)·(1+corr(s))`
  piecewise fit. **Status: PARTIALLY reproduced** (~20/48 positive witnesses bit-exact; ~28 still
  open in the discrepancy catalog). The *deviation direction* is established; full bit-exact
  reproduction is in progress.
- **Evidence:** `docs/function-lane/ERFC_EXCEL_EMULATION.md`; commit `4bedeac`. Confidence **high
  on the deviation**; reproduction **partial**.

---

## Inverse class & pending candidates

**Excel is *more* accurate than naive f64** (OxFunc must work harder to match; tracked as open
work in the discrepancy catalog, not reproduced here):
- `BESSELY`/`BESSELJ` order ≥1, `x≥8` — Excel's more-accurate proprietary method beats the NR
  5-term asymptotic (`J1(10)`: Excel ~5e-7 off true vs OxFunc NR ~6e-6); OxFunc still 1e8–1e11
  ULP off. BUG-FUNC-024.
- `LOG(x, 10)` / `LOG(x, 2)` — Excel uses dedicated `log10`/`log2` (more accurate than naive
  `ln(x)/ln(base)`); OxFunc matches by using `log10`/`log2` directly. (Already matched — Excel is
  the accurate party.)

**Excel is *less* accurate but NOT yet reproduced** (future XMD feeders — they fit this catalog's
bar, but OxFunc currently sits on the *true* value, so they are OPEN `OxFunc≠Excel` rows in the
discrepancy catalog; promote to a full XMD entry once reproduced): `ATANH` (mid-small args,
Excel's own routine ~2–3 ULP off true), `ACOTH` (Excel ~1 ULP high), and the ±1-ULP set
`COMBIN`/`COMBINA`/`PERMUT`/`ERF.PRECISE`. The solver substrate (`IRR`/`RATE`/`YIELD`/`ODDF*`),
the distribution family (`BETAINV`/`CHIDIST`/`TDIST`/…), `PMT`/`PPMT`, and `MINVERSE` are Excel's
proprietary iteration/substrate — algorithmically different, open.

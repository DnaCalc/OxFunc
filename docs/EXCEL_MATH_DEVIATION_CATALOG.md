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

---

## Inverse class (Excel is *more* accurate than the naive formula) — pointers only

For completeness, the opposite phenomenon — Excel using a **higher-accuracy proprietary
method** than a naive `f64` formula, forcing OxFunc to work *harder* to match — is tracked in
the bug streams, not here. Known examples to fold in if this catalog grows to cover both
directions: `BESSELY`/Bessel family (Excel's more-accurate method), `ACOTH`/`ATANH` (Excel's
own `ln`-substrate routines). These are noted so the catalog's *less-accurate* scope stays
clear.

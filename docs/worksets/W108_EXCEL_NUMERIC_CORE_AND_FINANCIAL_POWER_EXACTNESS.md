# W108 Excel Numeric Core and Financial/POWER Exactness Campaign

Status: `planned`
Opened: `2026-07-03`
Supersedes/absorbs: W103 PMT-family exactness lane (`oxf-acdw.4`)
Anchor bug: `BUG-FUNC-015` / `oxf-fckb` (reopened)
Related discrepancy cluster: `oxf-jbi3` XNPV, `oxf-pzav` YIELDDISC, plus the wider
libm-rooted numeric-drift beads (ERFC, FORECAST, CHISQ.TEST, MINVERSE, statistical)
that share the same substrate and can adopt the core later.

## 1. Purpose

Make Excel's numeric model **explicit and first-class** in OxFunc, and on that
foundation bring the financial time-value family and the `POWER` surface to
**bit-exact** parity with Excel, with clear code that reads like Excel's actual
algorithm.

This workset is the productization of the W103 investigation (2026-07-03). It does
not repeat the exploratory probing; it records the settled findings, the
architecture, and the batch-oracle + bead plan.

## 2. Settled findings (2026-07-03 investigation, live Excel 16.0 b20131, 64-bit)

Established bit-exact against live Excel via cell-ref `Value2` plumbing (25 targeted
probes + an 855-cell factor grid + intermediate-precision discriminators):

1. **FV / PV already match Excel bit-exact** — factor `powi` (integer n) / `powf`
   (fractional n), term `(F-1)/r`. No change needed; they are the reference.
2. **The TVM family is split BY FUNCTION, not by module.** FV/PV use the
   `powi/powf` kernel; **PMT (and the payment feeding IPMT/PPMT/CUMPRINC) uses an
   `exp(n*log1p(r))` / `expm1` chain** — a genuinely different substrate. OxFunc's
   current PMT reuses the FV `powi` kernel, which is catastrophically wrong on the
   common loan/mortgage regime (up to `5.5e8` ULP; e.g. `PMT(1e-9,120,1e5)`).
3. **64-bit Excel is pure IEEE-754 double (SSE2). No x87, no FMA, no wide
   intermediates.** Proven directly: `=A1*A1-B1`, `=A1*B1-C1`, `=A1*B1+C1*D1`,
   `=SUMPRODUCT(...)`, `=A1^2-B1` all publish `0x0` where any extended/fused
   intermediate would publish a nonzero residual. **The fix is fully portable;
   no x87 softfloat is needed** (an earlier x87 hypothesis is retracted).
4. **Excel's elementary functions are correctly-rounded, not UCRT.** On every
   probe where UCRT `exp` differs from the correctly-rounded value, Excel published
   the correctly-rounded bits (E1-E4). UCRT `log1p` misrounds ~21% and `expm1` ~5%,
   so Rust `f64::exp/ln_1p/exp_m1` (= UCRT) will not match Excel in general. **A
   correctly-rounded `exp`/`expm1`/`log1p`/`log` is required.** (Open sub-question:
   correctly-rounded vs Intel SVML/AVX2 — they usually coincide; Bead A resolves it.)
5. **NPER is solved.** `nper = CR_log(ratio) / log(1.0 + r)` — numerator `ln(ratio)`
   correctly-rounded, denominator plain `ln(1+r)` (NOT `log1p`), f64 divide.
   Bit-exact on P20 + four confirmation inputs.
6. **PPMT/IPMT/CUMPRINC use a dedicated internal principal path, not
   `PMT - IPMT`.** Proven: built-in `PPMT(...,1,...)` = `...0723`, standalone
   `PMT - IPMT` = `...0724`, `CUMPRINC(1,1)` = `...0722` — three distinct values
   1 ULP apart, stable across sessions. Each function has its own pure-double
   composition (running-balance / geometric-factor form) to be pinned in Bead A.
7. **POWER is a separate but related surface bug.** `POWER(1.0041666,360)` in
   OxFunc = `0x...30fb`, 377 ULP off Excel's `0x...2f82` (= `powi`, which OxFunc's
   own FV-internal path already matches). OxFunc's POWER surface diverges from its
   own correct `powi`. Routing POWER (and the `^` operator) through the explicit
   primitives fixes it. Bundled here because it shares the `powi/powf` primitives.

## 3. Architecture — three explicit layers

The root cause across these discrepancies (and the wider Bessel/stat/trig
residuals) is uniform: OxFunc leans on the platform libm (Rust std -> UCRT) and
ad-hoc per-function power/composition code, while Excel evaluates on a small,
specific primitive set with specific pure-double compositions. The design makes
that model explicit.

1. **Excel elementary primitives** — a documented module reproducing Excel's
   elementary ops bit-exact: `excel_exp`, `excel_expm1`, `excel_log1p`,
   `excel_log`, and the confirmed `excel_powi` / `excel_powf`. Each carries its
   algorithm explicitly (correctly-rounded via double-double + rounding test, or
   the identified SVML sequence) with a comment citing the oracle evidence.
2. **Function compositions** — each worksheet surface (PMT, PPMT, IPMT, CUMPRINC,
   CUMIPMT, NPER, POWER, `^`) written to read like Excel's algorithm on the
   primitives, with the exact operation order explicit and pinned to witnesses.
3. **Corpus validation** — a large batched Excel oracle corpus; the Rust layers
   proven bit-exact against it (the BUG-FUNC-024 BESSEL methodology, scaled up:
   differential corpus, zero mismatches).

The primitives layer is deliberately reusable: once it exists, the statistical
(BUG-FUNC-021), special-function, and trig lanes can adopt it to close their own
libm-rooted residuals. This campaign scopes only the financial + POWER consumers.

## 4. Batch-oracle strategy

Use the live Excel oracle in a few **large, versioned batches**, not many ad-hoc
runs. Each batch is a single COM session over a broad grid, saved as a canonical
corpus under `smart-fuzzer/runs/<run_id>/` with a manifest (Excel version/build,
plumbing mode, git rev). All numeric inputs via cell `Range.Value2`; log exact
input bits. Three planned batches (Bead A):

- **B1 elementary**: `EXP`, `LN`, `SQRT`, `POWER`, `LOG10` over dense grids,
  including inputs chosen where correctly-rounded != UCRT != SVML, to classify each
  routine. Resolves the CR-vs-SVML fork.
- **B2 financial**: `FV/PV/PMT/IPMT/PPMT/CUMPRINC/CUMIPMT/NPER` over dense
  `(r, n, per, type, fv)` grids, including tiny-r, large-n, fractional-n, negative
  regimes, to fit and pin every internal composition bit-exact.
- **B3 POWER/operator**: `POWER` and `^` over integer/fractional/negative
  exponent and base grids to pin the `powi`/`powf`/`exp-log` routing boundaries.

## 5. Beads (broad, dependency-ordered)

- **Bead A — Excel numeric-model characterization** (`oxf-*`, no code): run B1/B2/B3
  batch corpora; produce a written "Excel numeric model" spec identifying the exact
  primitive and composition for every in-scope function; resolve CR-vs-SVML.
  Foundation.
- **Bead B — Excel elementary-primitives core** (depends on A): implement bit-exact
  `exp`/`expm1`/`log1p`/`log` (+ confirm `powi`/`powf`) as a documented numeric-core
  module; differential-validate (1e6-1e8 samples) against the B1 corpus, zero
  mismatch.
- **Bead C — Financial + POWER surfaces on the primitives** (depends on A, B):
  re-express PMT/PPMT/IPMT/CUMPRINC/CUMIPMT/NPER (verify FV/PV) and fix POWER + `^`
  with explicit compositions, bit-exact against the B2/B3 corpora. Closes
  BUG-FUNC-015 and the POWER anomaly. May split C-financial / C-power if large.

## 6. Quality bar

- **Bit-exact, no tolerance.** Every in-scope surface reproduces Excel's f64 bits on
  the full corpus; residuals are bugs, not accepted.
- **Explicit algorithm.** Code reads like Excel's method; the primitive choice and
  composition order are visible and commented with the oracle witness that fixes
  them. No magic constants without provenance.
- **No backward-compat constraints** (per owner): free to restructure the financial
  modules and unify the two annuity kernels behind the explicit primitives.
- **Regression witnesses**: the 25 probe + confirmation rows become inline
  `function_spec!`/bit witnesses; the batch corpora are the differential gate.

## 7. Doctrine Axes

scope_completeness: `scope_partial`
target_completeness: `target_partial`
integration_completeness: `none`
open_lanes: `[W108-A_oracle_characterization, CR_vs_SVML_fork, W108-B_primitives_core, W108-C_surface_reimpl, POWER_surface_anomaly]`

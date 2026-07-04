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
7. **POWER is NOT a bug (corrected 2026-07-03, Bead C).** The reported
   `POWER(1.0041666,360) = 0x...30fb` "377-ULP anomaly" was a **probe-harness
   artifact**: `serde_json` parses the decimal literal `1.0041666666666667` to
   `0x...112` (1 ULP high), and `powi` of that wrong base gives `0x...30fb`. Excel,
   Rust `str::parse::<f64>()` (which OxFunc's own text->number coercion uses), and
   Python all round to the true `0x...111`. Given the correctly-rounded base,
   `power_kernel`/`eval_power_surface` already return `0x...2f82` = Excel, and
   `POWER(1+r, n) == FV(r,n,0,-1,0)` bit-for-bit on 84/84 grid points. `power_fn.rs`
   is unchanged. Lesson: the JSON probe harness can misround float literals —
   always confirm args round-trip to the exact `arg_bits` (see learning log).

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

scope_completeness: `scope_substantial`
target_completeness: `target_substantial`
integration_completeness: `partial`
open_lanes: `[IPMT_PPMT_near_payoff_stable_path, PPMT_CUM_dedicated_op_order, bespoke_transcendental_floor]`

## 9. Bead C outcome (2026-07-03): financial kernels on the CR core — LANDED

PMT/IPMT/PPMT/NPER (`financial_time_value_family.rs`) and CUMPRINC/CUMIPMT
(`cumulative_finance_family.rs`) rewritten onto `crate::excel_numeric` with the
`log1p`-chain compositions. FV/PV untouched (`powi`/`powf`, already bit-exact).
Zero-rate guards tightened from `< 1e-12` to exact `== 0.0` to match Excel.

Verified independently against the 242-case live-Excel corpus (`w108-b2-financial`)
and the full `oxfunc_core` suite (1577 passed, 0 failed; PMT witness
`PMT(0.05/12,360,200000)` closes bit-exact `0xc090c692af15f63a`):

| metric | before | after |
|---|---:|---:|
| exact vs Excel (242) | 56 | **81** |
| within 1 ULP | 95 | **170** |
| within 3 ULP | 122 | **219** |
| max drift | ~2.25e15 ULP | (same 2 degenerate rows; rest ≤ ~1268) |

The catastrophic `powi`-vs-chain drift is eliminated on ordinary financial inputs
(PMT was 3/42 exact with a 5.5e8-ULP tail; PPMT 0/51). Remaining OPEN discrepancies
(smaller = better, still tracked bugs, NOT accepted):
- **IPMT/PPMT near-payoff** (`per ≈ nper`): last-period cancellation, up to ~1268
  ULP of a near-zero result; a fully-stable last-period path is unsolved.
- **PMT `n=1023` subnormal** (PMT-0029): Excel flushes `(1+r)^-1023` to `0x0`, we
  emit the subnormal — degenerate extreme-n row.
- **PPMT/CUMPRINC/CUMIPMT dedicated per-period op-order**: ≤5-10 ULP accumulation
  lane (each function's internal principal uses a distinct op-order, only partially
  recovered).
- **General ≤2-3 ULP**: Excel's bespoke internal transcendental last bit (§8),
  where the faithful CR core rounds opposite (~0.5% of inputs).

BUG-FUNC-015 magnitude collapsed (NUM-L -> NUM-S) but stays OPEN. POWER confirmed a
non-bug (§2.7). Core module `crate::excel_numeric` is reusable for the wider
libm-rooted cluster (stats/special) later.

## 8. W108-A resolution (2026-07-03): the CR-vs-SVML fork — RESOLVED (neither)

The elementary-function batch (5620-point live-Excel EXP/LN grid with hard-to-round
targeting) plus a full reference hunt (faithful no-FMA glibc-2.29 `__exp`/`__log`
port, real Intel MKL VML via ctypes, MSVC-compiled probes, UCRT/numpy) settled it:

**Excel's `exp`/`ln` are a proprietary Microsoft ~0.502-ULP double-double routine —
neither correctly-rounded, nor UCRT, nor glibc, nor SVML/MKL-VML, nor fdlibm.** No
stock library reproduces it: union of glibc-2.29 + MKL-VML + UCRT matches only 22 of
36 hard-midpoint deviations; **14 are matched by nobody**. Dense black-box
fingerprinting is aliased/intractable (the sign(k) reconstruction-order hypothesis
maxed at 15/36). Reproducing the last bit would require disassembling Excel's binary.

**Decision — transcendental core = CORRECTLY-ROUNDED (Track B).** Bead B builds a
deterministic, portable correctly-rounded `exp`/`expm1`/`log1p`/`log` (double-double +
rounding test). This is bit-exact vs Excel on >=99.78% of ordinary inputs and strictly
better than today's UCRT path.

**Policy note (unchanged): every residual below remains an OPEN discrepancy = a BUG.**
We are not abandoning bit-exact parity. `EXP`/`LN` (and every financial surface that
inherits their behaviour) stay *mismatched* and tracked as discrepancies; smaller
differences are better, so reducing the drift is the improvement, not an acceptance.
The distinction is only tractability: matching Excel's exact bits requires its
proprietary binary, so these are **low-tractability open bugs**, characterized as:
- magnitude: exactly 1 ULP (EXP/LN); financial ≤2-3 ULP after composition;
- frequency: EXP 0.22% / LN 0.00% of ordinary inputs (EXP 2.7% / LN 0.53% adversarial);
  financial ~30% of an adversarial grid, far rarer on realistic parameters;
- locus: inputs whose true value is within ~0.005 ULP of a rounding midpoint;
- direction: away from 1.0, tracking sign(k), k = round(x/ln2), propagated through the
  annuity composition.

**Financial family is NOT fully decoupled (corrected 2026-07-03 after Bead-C
Probe-B).** The earlier "decoupled / 100% achievable" claim held only for a handful of
hand-checked intermediates. A 242-case corpus + a decisive isolation probe
(`PMT(r,n,0,1,0)` isolating `expm1(n*log1p(r))`, tested across composition orders ×
{UCRT,CR}) showed Excel's INTERNAL `log1p`/`exp`/`expm1` at financial arguments is the
same bespoke ~0.502-ULP routine — 6/11 residual cases match neither UCRT nor CR under
any op-order, some landing past CR. So the PMT-family inherits the same low-tractability
floor. **Best-achievable pass (committed):** switch PMT/PPMT/IPMT/NPER to the `log1p`
chain on the correctly-rounded core + the identified compositions — eliminates the
catastrophic `powi`-vs-chain drift, reaches ~70% exact / ~95% within 1 ULP (incl. every
realistic mortgage/loan witness), and reduces the remaining discrepancy to a documented
≤2-3 ULP bug. FV/PV/POWER stay on `powi` (already bit-exact); two-kernel split kept.

Evidence: `smart-fuzzer/runs/w108a-elementary-cr-vs-svml/`,
`smart-fuzzer/runs/w108a-reference-hunt/`, `smart-fuzzer/runs/w108-b2-financial/`.

## 10. W108 Phase A (2026-07-04): the transcendentals CRACKED — x87 CRT — LANDED

§8's "unreproducible without Excel's binary" conclusion is **SUPERSEDED**. A dedicated
reverse-engineering pass (`C:/Temp/ExcelExpFunction`, ~1.4M tokens, 294 live-Excel rows
over 3 adversarial rounds) identified the exact algorithm:

**64-bit Excel `EXP`/`LN`/`LOG10`/`LOG`/`POWER` are the legacy Microsoft x87 CRT
transcendental sequence** (`fpw32/tran/i386/87tran.asm`: `fFEXP`/`fFLN`/`fFLOGm`),
executed with control word `0x133F` (precision-control 64-bit, round-nearest, all
exceptions masked), then stored to binary64 with one final round:
- `EXP(x)`: `FLDL2E`/`FMULP` → `FRNDINT`/`FSUB` (exact reduction) → `FABS`/`F2XM1` →
  `1+w` → reciprocal-if-`f<0` (the invert path) → `FSCALE` → `FSTP qword`.
- `LN(x)` / `LOG10(x)`: `FLDLN2`/`FLDLG2` then a single fused `FYL2X`.
- The ~0.502-ULP "away-from-1.0" `sign(k)` bias is `FLDL2E` (rounded up) carried through
  `×2^k`. The `F2XM1`/`FYL2X` last bit is CPU microcode → on the hardest ~1-in-30 rows,
  parity is a host-CPU property (validated: AMD Zen2; reference impl 294/294 live Excel).

This is NOT any modern libm (not UCRT, glibc, MKL-VML, SVML) and NOT correctly-rounded —
so §8's Track-B correctly-rounded core, while a good portable fallback, is NOT what Excel
does. It is retained as `exp_portable`/`log_portable` (non-x86_64 fallback + the financial
substrate + expm1/log1p/sqrt basis).

**Implementation (LANDED):**
- New `crate::excel_numeric::x87` — the single contained site of x87 inline assembly.
  `exp`, `ln`, `log10` execute the literal `87tran` chains on the host CPU.
- `excel_exp`/`excel_log`/`excel_log10` now dispatch to x87 on `x86_64`, portable core
  elsewhere. Worksheet `EXP`/`LN`/`LOG10`/`LOG(x,base)` route through them.
- **`LOG(x, base)` = `ln(x)/ln(base)` for EVERY base** (dropped the wrong base-2/base-10
  special-casing). Confirmed by a 218-row live sweep, incl. `LOG(1000,10)=2.9999999999999996`
  (Excel's own imprecision) while dedicated `LOG10(1000)=3` — genuinely different paths.
- Validation: 249/249 in-crate corpus (`x87_excel_ground_truth.tsv`) + a fresh 396-row
  live Excel sweep (`x87lab`): EXP 24/24, LN 19/19, LOG10 120/120, LOG 218/218 bit-exact
  (excl. one subnormal-domain edge where Excel flushes `5e-324` to 0 → `#NUM!`).

**POWER (fractional exponent) — RESOLVED, bit-exact (Phase D, BUG-FUNC-042):** Excel
`POWER(x,y)` fractional path is `exp(y·ln x)` via x87 exp/ln with a **double-rounded** product,
plus two subtleties that close it to bit-exact: (a) for `y<0`, Excel computes the positive
power, stores it to f64, then takes ONE x87 double-rounded reciprocal (the 1-ULP residual —
`exp(y·ln x)` was the right function at the wrong point; `C:/Temp/ExcelExpFunction`
POWER_REPORT, 315/315); (b) exponent `|y|==0.5` is the correctly-rounded hardware `sqrt`, NOT
`exp(0.5·ln x)` — a case the reference missed (it only tested `0.5` on negative bases), caught
by an OxFunc live sweep. Full model 400/400 fresh + 315 ground truth. Landed in `power_kernel`
via `excel_numeric::{excel_pow_positive, excel_x87_recip}` and new `x87::{mul, recip}` (the
PC=64 double-rounded FMUL/FDIV). Replaces `powf`. Spec + test suite:
`docs/EXCEL_POWER_SPEC_AND_TEST_CASES.md`.

**Financial family — FROZEN on the portable core pending Phase C.** PMT/PPMT/IPMT/NPER/CUM
explicitly use `exp_portable`/`log_portable`/`excel_expm1`/`excel_log1p` (unchanged from
Bead C). Migrating them to x87 primitives (`FYL2XP1` for log1p, `F2XM1` for expm1) with a
fresh live-Excel re-validation is **Phase C** — likely the path to bit-exact financials,
since Excel's internal `log1p`/`expm1` are almost certainly the same x87 primitives.

Evidence: `C:/Temp/ExcelExpFunction/REPORT.md` (algorithm + 294-row validation),
`crate::excel_numeric::x87`, scratch `x87lab` (396+220 live-Excel probe rows).

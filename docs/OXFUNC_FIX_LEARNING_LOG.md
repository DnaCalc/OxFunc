# OxFunc Fix Learning Log

Status: `active`

General, transferable lessons from making OxFunc match Excel bit-for-bit. This is **not**
a changelog or a step counter — it holds reusable rules of thumb only. Per-discrepancy
status lives in [`OXFUNC_EXCEL_DISCREPANCY_CATALOG.md`](OXFUNC_EXCEL_DISCREPANCY_CATALOG.md);
durable per-fix history lives in git and the bug-stream register.

Keep entries short, principle-shaped, and de-duplicated. Add a lesson only when it will
change how a *future, unrelated* fix is approached.

## Verification

- **Verify against live Excel before closing — a fix can look done locally but be an
  over-correction.** Two W102A "fixes" rejected what Excel accepts: `GAMMA.INV(0)` →
  `#NUM!` vs Excel `0`, and the regex escape set rejected anchors/whitespace/escaped
  metacharacters Excel admits. Local tests were green; only the Excel oracle caught them.
- **Bit-exact comparison requires `Range.Value2` cell-ref plumbing.** Never pass a numeric
  input to Excel as formula-literal text beyond ~15 significant digits — Excel's parser
  re-rounds it and the comparator blames a kernel drift that is the harness's fault.
- A 4-witness sample can mislead. For "which inputs does Excel admit/reject" questions
  (regex escapes, domain guards), sweep a **battery**, not a handful.
- **A dynamic-array result spills into adjacent cells — keep the comparator's helper cells
  out of the spill range.** Probing `LINEST`/`LOGEST` with the `ERROR.TYPE` check one column
  to the right read a false `#SPILL!`/error because the array spilled onto the check cell.
  Place helper formulas well clear of the anchor, or read the spilled cells directly.
- A single baseline witness that matches bit-exact does **not** clear a fuzzer-flagged
  surface — the original flag may be an edge input. Mark "cleared on baseline", and only
  fully clear once the original case is reproduced-and-fixed or proven stale.
- **Record ULP from the bit pattern, never by eyeballing decimal strings.** BUG-FUNC-009's
  RATE residual was logged as "~1 ULP" because OxFunc `0.0041666445363460975` and Excel
  `0.004166644536345589` agree to ~13 printed digits. The actual bit distance is **586 ULP**
  (`0x…485999` vs `0x…48574f`) — a NUM-L drift, not NUM-S. A shared decimal prefix hides
  the tail; compute the distance from `f64::to_bits`.

## Error codes & non-finite results

- **Excel never publishes `Inf`/`NaN`.** A kernel that overflows to `±Inf` or produces
  `NaN` must map to a worksheet error — usually `#NUM!`.
- **But not every function errors on overflow — some saturate.** `COTH`/`TANH`/`FISHERINV`
  return `±1` for large argument; `SECH`/`CSCH` → `0`. So the non-finite rule is **per
  function**, never blanket across a shared surface helper.
- **Declare the result-publication rule on the meta, not at the call site.** A real kernel's
  Excel behaviour (argument-domain guard + non-finite handling) is one declarative value:
  `FunctionMeta::real_result_policy` (`ExcelRealPolicy::{PASS,FINITE,SATURATE_SIGN,CIRCULAR_TRIG}`).
  A unary function reaches evaluation via three tables (calc-surface, scalar-apply, by-index
  `eval_*_surface`); reading the *same* meta field at each site makes divergence impossible.
  The cautionary tale: the scalar-apply table silently lacked the overflow guard that the
  other two paths had, for months — a per-site guard is a per-site bug waiting to happen.
  Still verify with both a scalar **and** an array-lift witness, since they take different paths.
- **The error *code* is specific.** Overflow → `#NUM!`; a `±Inf` from `1/underflow`
  (negative exponent over a sub-unit base) → `#DIV/0!`. Verify the code, don't assume.
- **Error propagation preserves the incoming code.** `f(NA())` should stay `#N/A`, not be
  re-classified to `#VALUE!` or stringified.

## Numeric kernels

- **Catastrophic cancellation hides in `x - 1.0`.** For tiny `x`, `z = x - 1.0` loses `x`
  entirely, so a later `coeff/(z+1)` divides by zero. Lift small arguments with a recurrence
  (e.g. `lnΓ(x) = lnΓ(x+1) - ln(x)`) before the main approximation.
- **Integer/pole tests must be magnitude-relative.** A fixed absolute tolerance (`|x-round(x)| < 1e-12`)
  falsely collapses tiny non-integers (`-1e-200`) onto `0`. Scale the tolerance by `|x|`.
- **Argument reduction is the usual trig culprit.** Large-argument `SIN`/`TAN` drift is
  Cody-Waite vs extended-precision-π; near-boundary `ATANH`/`ACOTH` wants a `log1p` form.
- **Scattered near-boundary 1-ULP gaps vs a correctly-rounded op → suspect Excel uses
  `pow`/`exp·log`, not x87.** When a kernel using a correctly-rounded op (e.g. `sqrt`) matches
  Excel everywhere except a few points near a boundary/overflow where it's 1 ULP off, Excel is
  probably computing it via its (non-correctly-rounded) `pow` routine. 64-bit Excel and Rust
  are both SSE2 — a correctly-rounded `sqrt` is identical on both, so a difference means Excel
  isn't using `sqrt`. `SQRTPI(n) = (n·π)^0.5` via `pow`; Rust `(n·π).powf(0.5)` matched Excel
  30/30 where `.sqrt()` missed the near-overflow points (`oxf-quxx`). Probe `powf` against the
  oracle *before* chasing extended-precision/x87 emulation. (Excel-deviates-from-ideal-math
  cases like this are catalogued in
  [`EXCEL_MATH_DEVIATION_CATALOG.md`](EXCEL_MATH_DEVIATION_CATALOG.md).)
  **CAVEAT (W108 x87 discovery):** the "not x87" framing above is now known to be too strong —
  Excel's `POWER`/`EXP`/`LN` ARE the x87 CRT chain. `powf(x, 0.5)` matched Excel for `SQRTPI`
  because the exponent `0.5` is a special case (`powf(_,0.5)` returns `sqrt`, and Excel's
  `exp(0.5·ln x)` coincides there); for GENERAL fractional exponents `powf` matches Excel only
  ~5% while `exp(y·ln x)` matches ~86% (see the x87 entries below). Probe `powf` first still
  holds as cheap triage, but a `powf` miss now points AT the x87 chain, not away from it.
- **Drift that grows like a clean power of the argument is a coefficient-table defect, and
  you can *solve* for it instead of guessing.** Model `excel − local` as `Σ δ_k·y^k` through
  the kernel's own structure and solve the linear system against live-Excel bit witnesses
  (5–20 rows suffice); a delta that comes out consistent across all rows *is* the wrong/missing
  coefficient. BUG-FUNC-024 found five distinct table defects this way (a dropped Horner term,
  a 10× exponent slip, a digit transposition, a `…935↔…945` digit, and a duplicated-line
  6-entry table) that no amount of "compare against the NR book" would have settled.
- **Excel's sibling functions keep separately-typed copies of "the same" table — with
  different typos.** Excel's Y0/Y1 asymptotic tables match NR-with-truncated-2/π, but its
  J0/J1 copies of the *same* P/Q polynomials each carry their own transcription errors.
  Bit-matching one lane proves nothing about its sibling's constants; verify each function's
  copy against the oracle independently, and never "unify" tables that Excel keeps separate.
- **A residual no table delta can fit may be inherited from Excel's libm.** After the
  J-side tables were exact, two rows stayed 1 ULP off; probing `COS` directly at the exact
  reduced arguments showed Excel's `COS` (not the Bessel code) is 1 ULP off UCRT there,
  at full weight through the `cos·P` term. Before blaming a kernel's last ULP, evaluate the
  transcendental sub-calls through the worksheet functions at the exact intermediate
  arguments — and if the host libm is the cause, file it with the trig lane, not the kernel.

- **64-bit Excel ARITHMETIC is pure IEEE-754 double, but the TRANSCENDENTAL FUNCTIONS
  are x87.** The two are separate questions and have opposite answers. Worksheet `+`,
  `*`, `SUMPRODUCT` etc. are pure SSE2 double, no 80-bit intermediates, no FMA — proven
  with discriminator formulas whose published double differs under wide-vs-double:
  `=A1*A1-B1` with `A1=1+2^-27, B1=1+2^-26` publishes exactly `0.0` under pure-double
  but `~5.5e-17` under x87; all returned `0.0` (W108). But `EXP`/`LN`/`LOG10`/`LOG`/
  `POWER` are implemented internally with the **legacy Microsoft x87 CRT transcendental
  sequence** (`87tran.asm`, control word `0x133F`, precision-control 64-bit) — 80-bit
  `FLDL2E`/`F2XM1`/`FSCALE` and `FLDLN2`/`FYL2X`. So "pure double" applies to the
  operators, NOT to the elementary functions. (See `crate::excel_numeric::x87` and
  `C:/Temp/ExcelExpFunction`.)
- **Excel's elementary functions are the x87 CRT chain — NOT correctly-rounded, and not
  any modern libm.** (This CORRECTS the earlier W108-A claim that Excel publishes the CR
  value; that was wrong.) `EXP`/`LN` are `~0.502` ULP faithful with a systematic
  "away-from-1.0" 1-ULP bias on hard near-midpoint inputs (`sign(k)` where `k=round(x/ln2)`),
  the fingerprint of `FLDL2E` rounded up carried through `×2^k`. They match neither UCRT,
  glibc, MKL, nor the correctly-rounded value. Reproduce them **bit-for-bit by executing
  the x87 instructions on the host CPU** (`crate::excel_numeric::excel_exp/excel_log/
  excel_log10`, validated 249/249 + a fresh 396-row live-Excel sweep). Rust `f64::exp/ln/
  log10` = UCRT and cannot match Excel on hard cases, so worksheet transcendentals must
  route through the x87 backend. **The x87 microcode (`F2XM1`/`FYL2X`) is CPU-specific**,
  so on the ~1-in-30 hardest rows parity is a host-CPU property (validated on AMD Zen2).
- **Excel `LOG(x, base)` is `ln(x)/ln(base)` for EVERY base — but the dedicated
  `LOG10()` worksheet function is `fldlg2` and differs.** Do not special-case `LOG(_,10)`
  or `LOG(_,2)` to a dedicated log: a live-Excel sweep of 218 rows showed `ln/ln` exact
  for all bases, including Excel's own imprecision `LOG(1000,10)=2.9999999999999996`,
  while `LOG10(1000)=3` exactly. `LOG` and `LOG10` are genuinely different Excel code
  paths; each OxFunc surface must use its own.
- **Excel `POWER(x, y)` (fractional exponent, positive base) is `exp(y·ln x)` via the
  x87 exp/ln with f64 intermediates — NOT `powf`, NOT the fused x87 `x^y` (`FYL2X`+`F2XM1`)
  chain.** A 220-row live sweep: `exp(y·ln x)` matched 86% (rest 1 ULP), `powf` 5%, the
  fused chain 5%; `exp(y·ln x)` strictly beat `powf` on 84/90 head-to-head rows. The 14%
  residual (all exactly 1 ULP) is an unresolved intermediate-precision detail — Excel's
  exact `POWER` composition is a puzzle catalogued for a dedicated pass. Integer exponents
  keep the validated `powi` publication path (a separate Excel quirk).
- **Excel's financial functions split by primitive, not by module.** FV/PV compute
  `(1+r)^n` via `powi` (integer n) / `powf` (fractional n) + `(F-1)/r`; PMT uses an
  `exp(n*log1p(r))`/`expm1` chain; NPER uses `ln(1+r)` (NOT `log1p`) with a
  correctly-rounded numerator `ln`; PPMT/IPMT/CUMPRINC use a dedicated internal
  principal path, not `PMT-IPMT` (proven: the three give values 1 ULP apart). Do not
  assume one shared kernel across a function family — probe each surface.
- **Reverse-engineer composite functions through isolation lanes that make the engine
  publish its own intermediates.** `FV(r,n,0,-1,0)` = Excel's internal `(1+r)^n` factor
  bit-exact (the `pmt=0, pv=-1` terms are exact); `FV(r,n,-1,0,type)` = the annuity
  term. This let W108 classify the FV factor as `powi` and isolate PMT's chain without
  guessing.

## Doctrine

- **"OxFunc more accurate than Excel" is still a bug.** When OxFunc returns the exact
  integer and Excel is `±1` ULP off (`COMBIN`, `PERMUT`, …), the repair direction is
  **match Excel**, not stay mathematically correct. (See [[feedback_excel_imprecision_is_still_a_bug]].)
- **Repair by numerical substrate, not per-case.** Fixing one witness with a lookup-table
  patch is forbidden; fix the family's kernel so the whole band converges.
- **Classify before scaffolding.** If a discrepancy needs reference/spill/host/locale
  context, it is Category 1 — publish it to the context-sensitive catalog and evaluate it
  downstream; do not fake the context in a local rig (ODR-FN-002).

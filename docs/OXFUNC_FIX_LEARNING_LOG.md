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

## Error codes & non-finite results

- **Excel never publishes `Inf`/`NaN`.** A kernel that overflows to `±Inf` or produces
  `NaN` must map to a worksheet error — usually `#NUM!`. Shared guard: `finite_or_num`.
- **But not every function errors on overflow — some saturate.** `COTH`/`TANH`/`FISHERINV`
  return `±1` for large argument; `SECH`/`CSCH` → `0`. So apply the non-finite→`#NUM!`
  guard **per function**, never blanket across a shared surface helper.
- **A surface can route through several dispatch tables — guard all of them or guard the
  kernel.** Unary functions reach evaluation via the calc-surface dispatch, a scalar-apply
  table, and a by-index table (`eval_*_surface`). A `Result`-returning kernel guarded once
  covers every path (all callers go through it); an `f64` kernel forces a guard at each
  call site (or a signature refactor). Verify a guard with both a scalar **and** an
  array-lift witness, since they can take different paths.
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

## Doctrine

- **"OxFunc more accurate than Excel" is still a bug.** When OxFunc returns the exact
  integer and Excel is `±1` ULP off (`COMBIN`, `PERMUT`, …), the repair direction is
  **match Excel**, not stay mathematically correct. (See [[feedback_excel_imprecision_is_still_a_bug]].)
- **Repair by numerical substrate, not per-case.** Fixing one witness with a lookup-table
  patch is forbidden; fix the family's kernel so the whole band converges.
- **Classify before scaffolding.** If a discrepancy needs reference/spill/host/locale
  context, it is Category 1 — publish it to the context-sensitive catalog and evaluate it
  downstream; do not fake the context in a local rig (ODR-FN-002).

# Broad Scalar Explorer — 2026-05-09 Cycles

Status: `run_summary`

Owning workset: `docs/worksets/W092_SPARK_GUIDED_SMART_FUZZER_LONG_RUN.md`

This run summary documents a fresh smart-fuzzer cycle that walks a wider
single-arg/two-arg numeric scalar invocation space than the existing
manifest-seed and array-successor generators. The W092 plateau gates were
based on the manifest-seed and array-successor universes, not on this
broader scalar invocation space, so this cycle is consistent with the
existing ledger rather than a re-claim against those gates.

## 1. Tooling

A new local Rust binary lives under
`smart-fuzzer/tools/pmt_ppmt_local_eval/src/bin/broad_scalar_explorer.rs`
with the supporting PowerShell driver
`smart-fuzzer/tools/Run-BroadScalarExploration.ps1`. The local explorer:

1. picks one of `~50` simple scalar functions per case (math, transcendental,
   gamma/erf, hyperbolic, power, mod/round/log-base, combinatorics),
2. samples each argument from a per-family band picker
   (subnormals are excluded because Excel's formula literal parser rejects
   `~5e-324`; non-finites are excluded for the same reason),
3. evaluates locally through `oxfunc_core::functions::surface_dispatch::eval_surface_value_call`,
4. records the bit pattern of every numeric outcome,
5. emits one Excel candidate per fresh `(function, bucket, outcome_kind)` area
   up to a candidate cap.

The PowerShell driver evaluates candidates in a single batched
`Range.Formula2` write against live Excel COM (`Excel.Application`,
`Workbooks.Add`), reads back `Range.Value2` and `ERROR.TYPE(...)` for kind
classification, and compares typed digests bit-for-bit.

## 2. Cycles Executed

All on Excel `16.0` build `19929`, workbook compatibility `2`. Local
throughput stays around `~3.3e5 cases/sec` on the running machine.

1. `broad-scalar-cycle-003`: `1,000,000` cases, `600` Excel-sampled.
   `338` exact bit matches, `214` literal-encoding drift, `48`
   unexpected mismatches.
2. `broad-scalar-cycle-004`: `1,500,000` cases, `600` Excel-sampled.
   `340` matches, `208` literal-encoding drift, `52` unexpected.
3. `broad-scalar-cycle-005`: `1,500,000` cases, `600` Excel-sampled.
   `328` matches, `223` literal-encoding drift, `49` unexpected.
4. `broad-scalar-cycle-006`: `2,000,000` cases, `600` Excel-sampled.
   `336` matches, `214` literal-encoding drift, `50` unexpected.
5. `broad-scalar-cycle-007`: `2,000,000` cases, `600` Excel-sampled.
   `319` matches, `233` literal-encoding drift, `48` unexpected.
6. `broad-scalar-cycle-008`: `2,000,000` cases, `600` Excel-sampled.
   `334` matches, `213` literal-encoding drift, `53` unexpected.
7. `broad-scalar-cycle-009`: `1,500,000` cases, `600` Excel-sampled.
   `337` matches, `216` literal-encoding drift, `47` unexpected.

Aggregate: `11,500,000` local cases, `4,200` Excel-sampled candidates,
`2,332` exact bit matches, `1,521` literal-encoding-drift rows,
`347` unexpected mismatches, `0` Excel harness-blocked rows.

The seven cycles agree on which classes of mismatch are recurring witnesses;
classification is consistent across seeds.

## 3. Mismatch Classes Surfaced

The unexpected rows cluster into the following classes. Each class has at
least three independent witness rows across cycles unless noted.

### 3.1 GAMMALN / GAMMALN.PRECISE return `+Inf` for tiny positive `x`

Examples:

1. `=GAMMALN(1E-300)` — local `+Inf`, Excel `690.7755278982137`.
2. `=GAMMALN(1E-200)` — local `+Inf`, Excel `460.51701859880916`.
3. `=GAMMALN.PRECISE(1E-300)` — local `+Inf`, Excel `690.7755278982137`.
4. `=GAMMALN.PRECISE(1E-100)` — local `+Inf`, Excel `230.25850929940458`.

Source: `crates/oxfunc_core/src/functions/special_dist_family.rs::ln_gamma_positive`.
The Lanczos partial-fraction sum diverges as `z = x - 1` approaches `-1`
because the `coeff / (z + i)` term with `i = 1` becomes singular; `acc.ln()`
then yields `+Inf`. The standard fix is the recurrence
`ln Γ(x) = -ln(x) + ln Γ(x + 1)` for `0 < x < 0.5` so the Lanczos input is
shifted away from the recurrence pole.

### 3.2 GAMMA returns `#NUM!` for tiny non-zero `x` (false pole detection)

Example: `=GAMMA(-1E-200)` — local `#NUM!`, Excel `-1.000000000000001E+200`.

`crates/oxfunc_core/src/functions/special_dist_family.rs::is_integer_like`
flags `|x − round(x)| < 1e-12` as integer-like. For `x = -1e-200` this is
trivially true even though `-1e-200` is not actually a non-positive integer
pole. The detection threshold needs to be tightened (for example, to a
relative tolerance) so tiny non-integer values reach the reflection branch.

### 3.3 GAMMA numeric drift in negative-non-integer band

Example: `=GAMMA(-1.00012)` — `237441` ULP drift; `=GAMMA(-1.00061)` —
`110592` ULP drift. Reflection-branch precision drift; the magnitude is
ordinary library-precision Lanczos accumulator noise.

### 3.4 SINH / COSH return `+Inf` instead of `#NUM!` on overflow

Examples: `=SINH(-326648.33)` — local `-Inf`, Excel `#NUM!`;
`=COSH(-24230)` — local `+Inf`, Excel `#NUM!`. OxFunc's hyperbolic kernels
do not map IEEE overflow to `WorksheetErrorCode::Num`.

### 3.5 POWER overflow / underflow does not map to `#NUM!` / `#DIV/0!`

Examples: `=POWER(10, 700)` — local `+Inf`, Excel `#NUM!`;
`=POWER(0.001, -700)` — local `+Inf`, Excel `#DIV/0!`. The POWER kernel
returns IEEE infinities rather than the Excel-spec error codes for the
overflow and divide-on-negative-exponent corners.

### 3.6 PERMUTATIONA returns `+Inf` on overflow

Example: `=PERMUTATIONA(163, 150)` — local `+Inf`, Excel `#NUM!`.
`PERMUTATIONA(n, k) = n^k` is computed in floating point without an explicit
overflow-to-`#NUM!` guard.

### 3.7 FISHERINV returns `NaN` for large `|z|` instead of saturating to `±1`

Examples: `=FISHERINV(817.81)` — local `NaN`, Excel `1.0`;
`=FISHERINV(714.11)` — same pattern. The kernel evaluates `(e^(2z)-1) / (e^(2z)+1)` directly,
hitting `Inf/Inf = NaN`. Excel saturates to `1.0` at large positive `z`.

### 3.8 MROUND with `num=0, multiple < 0` returns `#NUM!` instead of `0`

Examples: `=MROUND(0, -4.2)` — local `#NUM!`, Excel `0`;
`=MROUND(0, -0.0354)` — same. Excel's spec rejects only when `num` and
`multiple` have *different* signs; a `0` num has no sign and the result is
`0`. OxFunc's sign check probably treats negative multiple plus
non-negative num as a different-sign rejection.

### 3.9 MOD `#NUM!` threshold mismatch

Examples: `=MOD(1.005E14, 1)` — local `0.828125`, Excel `#NUM!`;
`=MOD(-4.44E14, 0.288)` — local `0`, Excel `#NUM!`. Excel returns `#NUM!`
when `INT(n / divisor)` exceeds an internal threshold (documented as
`262144` in older guides, but the empirical surface is broader). OxFunc
performs a direct IEEE remainder and returns the (mathematically correct)
small residue.

### 3.10 MOD numeric drift

Examples: `=MOD(-9.26E9, 1.86)` — `9.84E9` ULP drift;
`=MOD(-78170.05, 1)` — `786432` ULP drift. The OxFunc remainder kernel
appears to round to a coarser representable, as if using `f32`-step
intermediate stages or `n - INT(n/d)*d` with intermediate truncation.

### 3.11 Trig family returns `#NUM!` at large arguments

Examples: `=COS(7.68E14)` — local `-0.519...`, Excel `#NUM!`;
`=TAN(-1.51E9)` — local `2.49`, Excel `#NUM!`;
`=SIN(9.51E14)` — local `-0.881`, Excel `#NUM!`. Excel imposes an
argument-magnitude threshold (`~2^48` historically per published guides) and
returns `#NUM!` above it. OxFunc's libm-based reduction returns a value
straight through.

### 3.12 Trig family numeric drift in moderate-large argument band

Examples: `=TAN(797601.58)` — `1.31E7` ULP drift;
`=SIN(961281.44)` — `1.82E6` ULP drift;
`=COT(-307.07)` — `12693` ULP drift;
`=SEC(-π/2 nearly)` — bit-pattern drift large because both sides are near a
singularity but Excel's reduction uses higher-precision π. This is the
classical Cody-Waite-vs-double-precision-π argument-reduction precision
delta; closing it requires extended-precision π reduction.

### 3.13 ATANH precision near `±1`

Examples: `=ATANH(-0.999999999)` — `3.14E7` ULP drift;
`=ATANH(-0.9999999999999990)` — `1.48E13` ULP drift. The kernel likely
uses `0.5 * ln((1+x)/(1-x))`; the `log1p`-based formulation
`0.5 * log1p(2x / (1-x))` is the standard near-boundary recovery.

### 3.14 ACOTH near `1`

Examples: `=ACOTH(1.001)` — `11244` ULP drift;
`=ACOTH(1 + 1.11e-16)` — local `17.56`, Excel `#NUM!`. Two issues: a small
near-1 precision drift, and a domain edge where Excel collapses
`x ≈ 1.0 + ULP` to `1.0` and rejects, while OxFunc accepts.

### 3.15 ACOSH near `1`

Example: `=ACOSH(1.000000000000001)` — local `4.71E-8`, Excel `0.0`. Same
near-1 collapse: Excel rounds the argument and returns `0`, OxFunc carries
the precision through.

### 3.16 ATAN2 boundary `(tiny, huge negative)`

Example: `=ATAN2(-1E-200, -6E199)` — local `-π/2`, Excel `#NUM!`. Excel's
ATAN2 imposes a magnitude-spread guard that OxFunc does not.

## 4. Doctrine Notes

These are W092 generation findings, not function-phase-complete claims.
The discovered classes split into three normative buckets:

1. **Definite OxFunc-owned bugs**: 3.1, 3.2, 3.4, 3.5, 3.6, 3.7, 3.8.
   These return wrong type or wrong value where the spec/Excel both have
   a single clear answer. Each row needs minimization, a focused unit
   test, and a kernel correction.
2. **Excel-spec Excel-version edge cases**: 3.9, 3.11, 3.16.
   Excel imposes argument-magnitude or quotient-magnitude guards that
   OxFunc has not modelled. The repair is to mirror Excel's threshold
   under a versioned axis fact in `docs/function-lane/...`.
3. **Numeric algorithm precision drift**: 3.3, 3.10, 3.12, 3.13, 3.14, 3.15.
   These are classical floating-point algorithm-choice differences (Lanczos
   accumulator, libm trig reduction, atanh/acosh near boundary). Repair is
   substrate-by-substrate without changing the no-tolerance comparison
   policy. `BUG-FUNC-021` already covers this discipline for statistical
   distribution kernels; the same approach should apply here.

## 5. Promotion

A new bug stream
`docs/bugs/streams/BUG-FUNC-027_broad_scalar_invocation_space_findings.md`
records this run's findings and breaks down the classes with witness
formulas. The W092 ledger and `IN_PROGRESS_FEATURE_WORKLIST.md` IP-18 row
should reference the new stream rather than re-using `BUG-FUNC-021` /
`BUG-FUNC-024` / `BUG-FUNC-025`, all of which are scoped to specific
families and not to this broader probe surface.

The fuzzer-side artifact contract is satisfied: each cycle has
`rollup.json`, `comparisons/excel_sample_comparisons.jsonl`, and
`failure_packets/<case>.json` for every non-pass row.

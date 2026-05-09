# BUG-FUNC-027: Broad scalar invocation-space findings

## Summary
- **Bug id**: `BUG-FUNC-027`
- **Opened**: `2026-05-09`
- **Status**: `open`
- **Owner workset**: `W092`
- **Bead**: pending allocation through `br`

## Source Refs
- **Reported against ref**: working tree at `2026-05-09` for the W092 broad
  scalar smart-fuzzer cycles `broad-scalar-cycle-003` through
  `broad-scalar-cycle-009`
- **Reproduced on ref**: same working tree
- **Introduced in ref**: `unknown`
- **Fixed in ref**: `unfixed`

## Ownership And Root Cause
- **Ownership class**: split — see Section "Mismatch Classes"
- **Root cause class**: split — see Section "Mismatch Classes"
- **Root cause summary**: a wide single-arg/two-arg numeric scalar
  smart-fuzzer cycle of `11.5M` local OxFunc evaluations and `4,200` Excel
  comparison samples revealed several recurring classes of OxFunc-vs-Excel
  divergence that are not covered by `BUG-FUNC-021` (statistical
  distribution exactness), `BUG-FUNC-023..025` (W089 non-statistical and
  matrix), or `BUG-FUNC-015` (PMT/PPMT financial). The classes split
  cleanly into definite kernel bugs, Excel-spec argument-domain guards
  that OxFunc has not modelled, and standard floating-point algorithm
  precision drift; they are bundled under one stream because they share a
  smart-fuzzer provenance but each class stands on its own minimization
  and repair lane.

## Reproduction
The local explorer is built from
`smart-fuzzer/tools/pmt_ppmt_local_eval/src/bin/broad_scalar_explorer.rs`:

```powershell
& "smart-fuzzer\tools\Run-BroadScalarExploration.ps1" `
  -RunId broad-scalar-cycle-003 -CaseCount 1000000 -Seed 17 -CandidateLimit 600
```

The seven cycles `broad-scalar-cycle-003 .. broad-scalar-cycle-009` are
preserved under `smart-fuzzer/runs/`; their `rollup.json` and
`failure_packets/` directories carry minimal reproducers.

Excel environment: `16.0` build `19929`, workbook compatibility `2`.

## Mismatch Classes

Each subclass below has at least three independent witness rows across the
seven cycles unless noted as `singleton_witness`.

### CLASS-A1: GAMMALN tiny positive returns +Inf

- **Ownership**: `OxFunc-owned bug`
- **Root cause**: `initial_impl_gap`
- **Source**: `crates/oxfunc_core/src/functions/special_dist_family.rs::ln_gamma_positive`
  uses Lanczos with `z = x - 1`; the partial-fraction term `coeff / (z + 1)`
  diverges as `x → 0+`, so `acc.ln()` returns `+Inf`.
- **Repair direction**: apply the recurrence
  `ln Γ(x) = -ln(x) + ln Γ(x + 1)` while `0 < x < some threshold` (e.g.
  `x < 0.5`) before calling Lanczos.
- **Witness**:
  - `=GAMMALN(1E-300)` — local `+Inf`, Excel `690.7755278982137`.
  - `=GAMMALN.PRECISE(1E-300)` — same divergence.

### CLASS-A2: GAMMA tiny non-zero falsely classified as pole

- **Ownership**: `OxFunc-owned bug`
- **Root cause**: `initial_impl_gap`
- **Source**: `is_integer_like(x) := |x − round(x)| < 1e-12` in
  `special_dist_family.rs`; for tiny `x` the rounded integer is `0` and the
  fixed `1e-12` absolute threshold trivially flags `x = -1e-200`.
- **Repair direction**: replace the absolute threshold with a relative or
  ULP-scaled check that distinguishes a genuine non-positive integer from a
  small non-integer.
- **Witness**: `=GAMMA(-1E-200)` — local `#NUM!`, Excel `-1.000000000000001E+200`.

### CLASS-A3: SINH / COSH overflow does not map to #NUM!

- **Ownership**: `OxFunc-owned bug`
- **Root cause**: `initial_impl_gap`
- **Witness**: `=SINH(-326648.33)` local `-Inf`, Excel `#NUM!`;
  `=COSH(-24230)` local `+Inf`, Excel `#NUM!`.
- **Repair direction**: in the SINH/COSH kernels add an explicit
  `if !value.is_finite()` guard that returns `WorksheetErrorCode::Num`.

### CLASS-A4: POWER overflow / underflow not mapped to error code

- **Ownership**: `OxFunc-owned bug`
- **Root cause**: `initial_impl_gap`
- **Witness**: `=POWER(10, 700)` local `+Inf`, Excel `#NUM!`;
  `=POWER(0.001, -700)` local `+Inf`, Excel `#DIV/0!`.
- **Repair direction**: post-evaluate the IEEE result and map `+Inf`/`-Inf`
  to `WorksheetErrorCode::Num`; for negative-exponent-with-zero-or-nearly-zero
  base map to `WorksheetErrorCode::Div0` consistent with the existing
  `BUG-FUNC-005` pattern for `0^0`.

### CLASS-A5: PERMUTATIONA overflow not mapped to #NUM!

- **Ownership**: `OxFunc-owned bug`
- **Root cause**: `initial_impl_gap`
- **Witness**: `=PERMUTATIONA(163, 150)` local `+Inf`, Excel `#NUM!`.
- **Repair direction**: in `permutationa_fn.rs` post-evaluate `n^k` and
  map `+Inf` to `WorksheetErrorCode::Num`.

### CLASS-A6: FISHERINV does not saturate to ±1 at large |z|

- **Ownership**: `OxFunc-owned bug`
- **Root cause**: `initial_impl_gap`
- **Witness**: `=FISHERINV(817.81)` local `NaN`, Excel `1.0`;
  `=FISHERINV(714.11)` same. Direct `(e^(2z)-1)/(e^(2z)+1)` produces
  `Inf/Inf = NaN` once `2z` exceeds `~709`.
- **Repair direction**: pre-clamp or use `tanh(z)` form to saturate to
  `+1` for `z >= +threshold` and `-1` for `z <= -threshold`.

### CLASS-A7: MROUND with `num=0, multiple<0` returns #NUM! instead of 0

- **Ownership**: `OxFunc-owned bug`
- **Root cause**: `spec_mismatch`
- **Witness**: `=MROUND(0, -4.2)` local `#NUM!`, Excel `0`;
  `=MROUND(0, -0.0354)` same.
- **Repair direction**: in `mround.rs` short-circuit `num == 0.0` to
  `Ok(0.0)` before the sign-comparison rejection.

### CLASS-B1: MOD `#NUM!` threshold mismatch

- **Ownership**: `OxFunc-owned bug` with Excel-version axis flavour
- **Root cause**: `spec_mismatch`
- **Witness**: `=MOD(1.005E14, 1)` local `0.828125`, Excel `#NUM!`;
  `=MOD(-4.44E14, 0.288)` local `0`, Excel `#NUM!`.
- **Repair direction**: model Excel's `INT(n / divisor)` magnitude guard.
  Map to `WorksheetErrorCode::Num` when the implicit quotient overflows
  the Excel-defined threshold.

### CLASS-B2: trig family `#NUM!` at large argument

- **Ownership**: `OxFunc-owned bug` with Excel-version axis flavour
- **Root cause**: `spec_mismatch`
- **Witness**: `=COS(7.68E14)`, `=TAN(-1.51E9)`, `=SIN(9.51E14)` all
  return numbers locally but `#NUM!` in Excel.
- **Repair direction**: introduce an Excel-doctrine guard returning
  `WorksheetErrorCode::Num` when `|arg|` exceeds the empirically pinned
  threshold (the published `2^48` is one candidate; final value pinned
  through a focused empirical sweep on the reference baseline).

### CLASS-B3: ATAN2 boundary on (tiny, huge-negative)

- **Ownership**: needs triage
- **Root cause**: `spec_mismatch`
- **Witness**: `=ATAN2(-1E-200, -6E199)` local `-π/2`, Excel `#NUM!`.
  Singleton-class so far; needs broader (y, x) magnitude-spread sweep
  before promotion direction is decided.

### CLASS-C1: GAMMA negative-non-integer numeric drift

- **Ownership**: `OxFunc-owned bug`
- **Root cause**: `numeric_algorithm_exactness_gap`
- **Witness**: `=GAMMA(-1.00012)` `237441` ULP; `=GAMMA(-1.00061)`
  `110592` ULP. Reflection-formula precision under the no-tolerance
  policy; family-level repair under the same discipline as `BUG-FUNC-021`.

### CLASS-C2: MOD numeric drift

- **Ownership**: `OxFunc-owned bug`
- **Root cause**: `numeric_algorithm_exactness_gap`
- **Witness**: `=MOD(-9.26E9, 1.86)` `9.84E9` ULP;
  `=MOD(-78170.05, 1)` `786432` ULP. Suggests an intermediate-truncation
  step in OxFunc's MOD kernel.

### CLASS-C3: trig family precision drift in moderate-large band

- **Ownership**: `OxFunc-owned bug`
- **Root cause**: `numeric_algorithm_exactness_gap`
- **Witness**: `=TAN(797601.58)` `1.31E7` ULP; `=SIN(961281.44)` `1.82E6` ULP;
  `=COT(-307.07)` `12693` ULP. Classical Cody-Waite-vs-double-precision-π
  argument-reduction delta. Closing it requires an extended-precision π
  reduction in the kernel.

### CLASS-C4: ATANH near ±1 precision

- **Ownership**: `OxFunc-owned bug`
- **Root cause**: `numeric_algorithm_exactness_gap`
- **Witness**: `=ATANH(-0.999999999)` `3.14E7` ULP;
  `=ATANH(-0.9999999999999990)` `1.48E13` ULP.
- **Repair direction**: switch to the `log1p`-based formulation
  `0.5 * log1p(2x / (1-x))` near boundary.

### CLASS-C5: ACOTH and ACOSH near 1

- **Ownership**: `OxFunc-owned bug`
- **Root cause**: `numeric_algorithm_exactness_gap`
- **Witness**: `=ACOTH(1.001)` `11244` ULP; `=ACOTH(1+ULP)` local finite,
  Excel `#NUM!`; `=ACOSH(1+1e-15)` local non-zero, Excel `0`. Two related
  issues: small-near-boundary precision and an Excel-side argument-collapse
  threshold.

## 2026-05-09 Plumbing Caveat And Cell-Ref Re-Replay

The seven cycles `003..009` were run under the legacy
formula-literal-text plumbing. That harness path absorbs a
`~1e-12 * scale` "encoding drift" class because Excel's formula parser is
not always correctly-rounded for long decimal literals. See
`smart-fuzzer/planning/EXCEL_RUNNER_PLUMBING_NOTE.md`.

`Run-BroadScalarExploration.ps1` was refactored to write numeric inputs
through `Range.Value2` (bit-exact f64 round-trip) and reference them from
the formula. Cycle `broad-scalar-cycle-010-cellref` re-ran the same seed
and candidate volume as cycle `003`:

| Run                              | exact match | encoding-drift | unexpected |
| -------------------------------- | ----------: | -------------: | ---------: |
| `cycle-003` literal-text         |        `338` |          `214` |       `48` |
| `cycle-010-cellref` cell-ref     |        `468` |            `0` |      `132` |

`+130` rows became exact (the encoding-drift bucket was real harness
artefact); `+84` rows became newly-visible kernel drifts the
`1e-12 * scale` tolerance was hiding.

Newly-visible 1-ULP rows include `=COMBIN(23, 10)` where OxFunc returns
the exact integer `1144066.0` and Excel returns `1144066.0000000002` —
in this case OxFunc is *more* accurate than Excel under bit-exact
comparison. Similar 1-ULP witnesses appeared for `PERMUT`, `PHI`,
`GAUSS`, `FACTDOUBLE`, `ERF.PRECISE`, and `ERFC.PRECISE`.

The CLASS-C* "numeric_algorithm_exactness_gap" subclasses below were
characterized under the legacy plumbing; their measured ULP magnitudes
should be re-measured under cell-ref plumbing before any subclass is
closed. The CLASS-A* and CLASS-B* subclasses are unaffected because they
turn on kind-drift or large-magnitude divergence rather than fine ULP
counts.

## Evidence
1. `smart-fuzzer/runs/broad-scalar-cycle-003/` (literal-text, plumbing-flagged)
2. `smart-fuzzer/runs/broad-scalar-cycle-004/` (literal-text)
3. `smart-fuzzer/runs/broad-scalar-cycle-005/` (literal-text)
4. `smart-fuzzer/runs/broad-scalar-cycle-006/` (literal-text)
5. `smart-fuzzer/runs/broad-scalar-cycle-007/` (literal-text)
6. `smart-fuzzer/runs/broad-scalar-cycle-008/` (literal-text)
7. `smart-fuzzer/runs/broad-scalar-cycle-009/` (literal-text)
8. `smart-fuzzer/runs/broad-scalar-cycle-010-cellref/` (cell-ref plumbing reference run)
9. Run summary: `smart-fuzzer/planning/BROAD_SCALAR_EXPLORATION_2026-05-09.md`
10. Plumbing rule: `smart-fuzzer/planning/EXCEL_RUNNER_PLUMBING_NOTE.md`
11. Local explorer source: `smart-fuzzer/tools/pmt_ppmt_local_eval/src/bin/broad_scalar_explorer.rs`
12. Driver: `smart-fuzzer/tools/Run-BroadScalarExploration.ps1`

## Closure Checklist
- [ ] CLASS-A1..A7 minimized into focused tests and repair landed
- [ ] CLASS-B1..B3 Excel-doctrine threshold pinned and modelled
- [ ] CLASS-C1..C5 substrate-by-substrate kernel correction landed
- [ ] follow-up beads opened for each class group and tracked in `.beads/`
- [ ] handoff to OxFml not required so far (no seam-side surface affected)

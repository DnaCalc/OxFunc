# BUG-FUNC-027: Broad scalar invocation-space findings

## Summary
- **Bug id**: `BUG-FUNC-027`
- **Opened**: `2026-05-09`
- **Status**: `open` (CLASS-A landed + Excel-verified 2026-06-19; CLASS-B/C remain)
- **Owner workset**: `W092`
- **Bead**: `oxf-vgxs` (unary non-finite audit follow-up); CLASS-B/C beads pending

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
- **FIXED (2026-06-20).** The guard is on the **quotient** `|n/d|`, not `|n|`
  (`MOD(2^45, 2^10)` → `0` despite `|n|=2^45`; `MOD(2^51, 2^10)` → `#NUM!` at
  quotient `2^41`), and the boundary is a **precise, d-independent threshold**.
  Bisected against live Excel 16.0 b20026 to the exact double: `MOD(q,1)` flips
  from a number to `#NUM!` between `1125899999999.9998` (`0x4270624de9afffff`)
  and `1125900000000` (`0x4270624de9b00000`). `mod_kernel` now returns `#NUM!`
  when `|number/divisor| >= 1_125_900_000_000.0`. 11/11 bit-exact vs Excel incl.
  both witnesses, the boundary, and the quotient rule under several divisors.
  Regression test `mod_fn::tests::mod_large_quotient_matches_excel_num_threshold`.

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
- **FIXED (2026-06-20).** The rule *is* clean: Excel returns `#NUM!` exactly when
  `x != 0` and `y/x` overflows to `∞`. Confirmed bit-exact on normal-range inputs
  — `ATAN2(x=1e-200, y=1e108)` (`|y/x|=1e308`, finite) → number, `y=1e109`
  (`|y/x|=∞`) → `#NUM!`; the axis case `x == 0` stays `±π/2`. The earlier
  "`x=1e-309, y=1` → `π/2` despite `|y/x|=∞`" reading was a `Value2` **denormal
  storage artifact** (the cell did not hold `1e-309`), not a real counterexample.
  `atan2_kernel` now returns `#NUM!` when `x != 0 && (y/x).is_infinite()`.
  Regression test `atan2::tests::atan2_overflowing_ratio_is_num_axis_stays_finite`.

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
- **FIXED (2026-06-21).** The kernel computed `number - divisor*floor(number/divisor)`,
  which catastrophically cancels for large quotients (the two terms are ~equal and
  ~`1e10`, so the `O(1)` remainder loses up to `~9.5e10` ULP). Replaced with the exact
  IEEE remainder (`%` == fmod, no rounding) plus Excel's divisor-sign adjustment:
  `r = n % d; if r != 0 && sign(r) != sign(d) { r + d } else { r }`. 8/8 probed cases
  bit-exact vs live Excel 16.0 b20026 incl. all three witnesses. The `#NUM!` quotient
  guard (CLASS-B1) is unchanged. Regression
  `mod_fn::tests::mod_large_quotient_is_bit_exact_via_fmod`. Removed from catalog G4.

### CLASS-C3: trig family precision drift in moderate-large band

- **Ownership**: `OxFunc-owned bug`
- **Root cause**: `numeric_algorithm_exactness_gap`
- **Witness**: `=TAN(797601.58)` `1.31E7` ULP; `=SIN(961281.44)` `1.82E6` ULP;
  `=COT(-307.07)` `12693` ULP. Classical Cody-Waite-vs-double-precision-π
  argument-reduction delta. Closing it requires an extended-precision π
  reduction in the kernel.
- **COS witnesses at moderate arguments (2026-07-02, BUG-FUNC-024 spillover).**
  Excel `COS` is `1` ULP off UCRT/Rust at cell-ref inputs `49.214601836`
  (Excel `0x3fdfcbaf84b75409` vs local `…5408`) and `149.214601836`
  (Excel `0xbf86a0d99f45d970` vs local `…d96f`); `SIN` is bit-exact at both.
  Downstream: `BESSELJ(50,0)`, `BESSELJ(150,0)` (`1` ULP) and `BESSELJ(50,2)`
  (`2` ULP) inherit exactly this through `cos(x-0.785398164)*P0` and close
  automatically when COS parity lands (Bessel tables themselves signed off
  under BUG-FUNC-024).

### CLASS-C4: ATANH near ±1 precision

- **Ownership**: `OxFunc-owned bug`
- **Root cause**: `numeric_algorithm_exactness_gap`
- **Witness**: `=ATANH(-0.999999999)` `3.14E7` ULP;
  `=ATANH(-0.9999999999999990)` `1.48E13` ULP.
- **Repair direction**: switch to the `log1p`-based formulation
  `0.5 * log1p(2x / (1-x))` near boundary.
- **FIXED (2026-06-21).** The defect was **lost odd symmetry**, not boundary precision:
  the platform libm gives `atanh(-x) != -atanh(x)` near -1 (up to `~1.5e13` ULP), while
  Excel's ATANH is exactly odd. OxFunc already matched Excel bit-for-bit on the
  non-negative side, so `atanh_kernel` now computes `|x|.atanh().copysign(x)`. 3/3 probed
  witnesses bit-exact vs live Excel 16.0 b20026 (incl. `-0.999999999`,
  `-0.9999999999999990`). Regression
  `atanh::tests::atanh_is_odd_symmetric_and_bit_exact_near_minus_one`.
- **Residual (2026-06-21, open).** Broader probing found ATANH is *not* fully bit-exact:
  mid-small args drift `2`–`3` ULP (`ATANH(0.2)` = `…9849` vs OxFunc/true `…984c`;
  `ATANH(0.1)` 2 ULP), exact at `0.5` and near `±1`. Excel's ATANH is its own
  approximation, *less* accurate than correctly-rounded; OxFunc (Rust `atanh`) is at the
  true value. Not an ln-precision gap (see C5: Excel's `LN` is correctly-rounded).
  Reclassified NUM-S on catalog G4; matching needs Excel's exact ATANH routine.
- **Candidate rejected and reverted (2026-07-10 expanded sign-off).** The W108 x87
  breakthrough invalidated the earlier assumption that worksheet `LN` was simply
  correctly rounded. Black-box decomposition on Excel 16.0 b20131 shows
  `ATANH(x) == 0.5*LN((1+x)/(1-x))` bit-for-bit at `x=0.1` and `x=0.2`, while
  the log-difference form differs. `atanh_kernel` now uses that graph through
  the reproduced x87 worksheet-LN backend and retained the already-pinned odd
  symmetry. Although both bounded residual witnesses became exact, the required
  368-case expansion matched only `297` and regressed `71`, catastrophically
  collapsing tiny inputs and drifting near the boundary. The candidate was
  reverted. The restored odd-symmetric platform path matches `235/368`; ATANH
  remains an M2 piecewise-kernel search.

### CLASS-C5: ACOTH and ACOSH near 1

- **Ownership**: `OxFunc-owned bug`
- **Root cause**: `numeric_algorithm_exactness_gap`
- **Witness**: `=ACOTH(1.001)` `11244` ULP; `=ACOTH(1+ULP)` local finite,
  Excel `#NUM!`; `=ACOSH(1+1e-15)` local non-zero, Excel `0`. Two related
  issues: small-near-boundary precision and an Excel-side argument-collapse
  threshold.
- **Cell-ref resweep (2026-06-20) — the near-1 "argument-collapse" half is a
  STALE HARNESS ARTIFACT (signed off).** Re-probed with exact `Range.Value2`
  inputs, Excel returns `ACOTH(1+ULP) = 18.36840028483855` (finite, **not**
  `#NUM!`) and `ACOSH(1+1e-15) = 4.712160905917527e-08` (non-zero, **not** `0`),
  and OxFunc matches **bit-exactly**. The original `#NUM!`/`0` came from
  formula-literal text: Excel's parser rounds the `1+ULP` literal down to exactly
  `1.0`, so it really evaluated `ACOTH(1.0)`/`ACOSH(1.0)`. Removed from catalog G1.
  The remaining `ACOTH(1.001)` ~11244-ULP **numeric** drift is unaffected and
  stays on the catalog G4 ACOTH row.
- **FIXED (2026-06-21) — catastrophic band closed.** The large-`|x|` `~1.2e14` ULP band
  came from the direct `0.5*ln((x+1)/(x-1))` ratio losing precision; replaced with the
  odd-symmetric `0.5*ln1p(2/(|x|-1))` form (ACOTH is odd like ATANH). Notably Excel's
  `ACOTH(x)` is **not** `ATANH(1/x)` (they differ ~39 ULP at `x=1.001`). Bit-exact vs
  live Excel 16.0 b20026 across the probed range (`1.001 .. 1e6` and negatives) **except**
  `ACOTH(5)`/`ACOTH(10)` (open, not accepted): 6 double-precision forms all miss
  `ACOTH(5)` by 1 ULP in the same direction (Excel's own/extended-precision `ln`), and
  `ACOTH(10)` is bit-exact only under `atanh(1/x)` — which regresses `ACOTH(1.001)` by
  39 ULP — so no single double form matches every point. Reclassified NUM-S on catalog
  G4. **ln-substrate investigation (2026-06-21) — decisive.** A validated correctly-rounded
  double-double `ln`/`exp` (verified against CR `ln(2)` = `0x3fe62e42fefa39ef`, non-trivial
  `lo` limb) gives the *same* `…984c` as the f64 ln1p form — the true value — while Excel
  returns `…984d` (1 ULP **high**). So a better/extended `ln` cannot match Excel here: the
  error is in Excel's own ACOTH routine, not ln precision. Excel's `LN` is correctly-rounded
  (8/8 probed points match OxFunc), and `ACOTH(5)` ≠ `LN(1.5)/2` ≠ `ATANH(1/5)` in Excel
  (Excel maps the same real value to `ATANH(0.2)=…9849` and `ACOTH(5)=…984d`). Closure needs
  Excel's exact ACOTH routine. Probe harness `tools/elem-probe/run-elem-probe.ps1`. Regression
  `acoth::tests::acoth_large_and_negative_args_bit_exact`. Probe harness:
  `tools/elem-probe/run-elem-probe.ps1`.

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

## 2026-05-10 W097 R-A Cell-Ref Re-Sweep Of CLASS-C*

W097 R-A (`smart-fuzzer/planning/W097-R-A-broad-scalar-cell-ref-resweep.md`)
re-replayed CLASS-C* across five additional fresh-seed cell-ref cycles
(`broad-scalar-cycle-011-cellref` through `broad-scalar-cycle-015-cellref`)
in addition to the reference run `broad-scalar-cycle-010-cellref`. The
revised per-subclass measurement is:

| Subclass | Direction        | Original ULP             | Re-measured (six cell-ref cycles) | Notes |
| -------- | ---------------- | ------------------------ | ---------------------------------- | ----- |
| `C1` GAMMA neg-non-int | **shrinks ~100x** | `237,441` (`-1.00012`); `110,592` (`-1.00061`) | `1,290` (`-1.00012`); `1,540` (`-1.00035`); max in band `2,050` | Kernel drift is real but two orders smaller; repair direction unchanged, urgency lowered |
| `C2` MOD               | **persists**      | `9.84E9` (`-9.26E9, 1.86`)                      | max `9.51E10` (`9.65E9, -0.374`); median `2.95E5`               | Kernel drift confirmed; intermediate truncation in MOD substrate |
| `C3` trig moderate-large | **grows**       | `1.31E7` (`TAN(797601.58)`)                    | max `3.34E12` (`COT/TAN/SEC/CSC` in `~10^5..10^6` band)         | Cody-Waite-vs-extended-π drift up to a full radian-band; repair scope widens |
| `C3.h` (new) hyperbolic overflow | **new**     | n/a                                              | `COTH(x)` returns NaN locally / `±1` in Excel for `|x|>>700`    | Kind-class subclass; saturation guard analogous to CLASS-A3 |
| `C4` ATANH near `±1`   | **stable**        | `1.48E13` (`ATANH(-0.999...9)`)                | `1.48E13` reproduced; max `1.48E13`; median `1`                 | log1p reformulation remains correct repair |
| `C5` ACOTH/ACOSH near 1 | **broadens**     | `11,244` (`ACOTH(1.001)`)                      | `11,244` reproduced; new band `ACOTH(|x|>>1)` up to `1.20E14`   | Add `ACOTH(x) = ATANH(1/x)` series for large argument |

Per-cycle rollups:

| Cycle                            | Seed | Excel sampled | Matches | Unexpected |
| -------------------------------- | ---: | ------------: | ------: | ---------: |
| `broad-scalar-cycle-010-cellref` | `17` |         `600` |   `468` |      `132` |
| `broad-scalar-cycle-011-cellref` | `23` |         `800` |   `593` |      `207` |
| `broad-scalar-cycle-012-cellref` | `31` |         `800` |   `614` |      `186` |
| `broad-scalar-cycle-013-cellref` | `41` |         `800` |   `602` |      `198` |
| `broad-scalar-cycle-014-cellref` | `53` |         `800` |   `603` |      `197` |
| `broad-scalar-cycle-015-cellref` | `61` |         `800` |   `601` |      `199` |

Cell-ref `match-rate` is stable around `~75%` and unexpected-mismatch
fraction is stable around `~25%` across all five fresh seeds, i.e.
seed variance does not blur the underlying class structure.

A new "OxFunc-more-accurate-than-Excel" pattern is now visible in the
`unexpected_mismatch` channel: `26` rows across the six cycles are
combinatorial functions where OxFunc returns the exact integer and
Excel returns the integer `±1` ULP (e.g. `=COMBIN(23,10) → 1,144,066`
local, `1,144,066.0000000002` in Excel; `=COMBIN(9,6) → 84` local,
`83.99999999999999` in Excel; `=COMBINA(41,16) → 41,648,951,840,265`
local, `41,648,951,840,265.01` in Excel; and similar for `COMBINA(9,6)`).
Per CHARTER §4.1 (2026-05-28 update), these are OxFunc bugs in the
numeric-drift class with repair direction match-Excel — OxFunc must reproduce
Excel's floating-point result even when Excel is less accurate than the
mathematical integer. They are tracked as a follow-up classification group
in this stream pending a focused repair lane; the earlier "not OxFunc bugs"
framing is superseded by the doctrine update.

## Evidence
1. `smart-fuzzer/runs/broad-scalar-cycle-003/` (literal-text, plumbing-flagged)
2. `smart-fuzzer/runs/broad-scalar-cycle-004/` (literal-text)
3. `smart-fuzzer/runs/broad-scalar-cycle-005/` (literal-text)
4. `smart-fuzzer/runs/broad-scalar-cycle-006/` (literal-text)
5. `smart-fuzzer/runs/broad-scalar-cycle-007/` (literal-text)
6. `smart-fuzzer/runs/broad-scalar-cycle-008/` (literal-text)
7. `smart-fuzzer/runs/broad-scalar-cycle-009/` (literal-text)
8. `smart-fuzzer/runs/broad-scalar-cycle-010-cellref/` (cell-ref plumbing reference run)
9. `smart-fuzzer/runs/broad-scalar-cycle-011-cellref/` (cell-ref, seed 23)
10. `smart-fuzzer/runs/broad-scalar-cycle-012-cellref/` (cell-ref, seed 31)
11. `smart-fuzzer/runs/broad-scalar-cycle-013-cellref/` (cell-ref, seed 41)
12. `smart-fuzzer/runs/broad-scalar-cycle-014-cellref/` (cell-ref, seed 53)
13. `smart-fuzzer/runs/broad-scalar-cycle-015-cellref/` (cell-ref, seed 61)
14. W097 R-A tranche record: `smart-fuzzer/planning/W097-R-A-broad-scalar-cell-ref-resweep.md`
15. Run summary: `smart-fuzzer/planning/BROAD_SCALAR_EXPLORATION_2026-05-09.md`
16. Plumbing rule: `smart-fuzzer/planning/EXCEL_RUNNER_PLUMBING_NOTE.md`
17. Local explorer source: `smart-fuzzer/tools/pmt_ppmt_local_eval/src/bin/broad_scalar_explorer.rs`
18. Driver: `smart-fuzzer/tools/Run-BroadScalarExploration.ps1`

## 2026-06-19 CLASS-A Landed (live Excel 16.0 build 20026)

All CLASS-A kind/error subclasses are fixed, focus-tested, and verified against live
Excel via the `array_tranche_local_eval` + COM harness:

| Subclass | Witness | Was | Now / Excel | Commit |
| -------- | ------- | --- | ----------- | ------ |
| A1 GAMMALN tiny | `GAMMALN(1E-300)` | `+Inf` | `690.7755278982137` (bit-exact) | `8278b86` |
| A2 GAMMA tiny-neg pole | `GAMMA(-1E-200)` | `#NUM!` | `~ -1E200` finite (fine ULP = C1) | `8278b86` |
| A3 SINH/COSH overflow | `SINH(-326648.33)` | `±Inf` | `#NUM!` | `b0b2419` |
| A4 POWER overflow | `POWER(10,700)` / `POWER(0.001,-700)` | `+Inf` | `#NUM!` / `#DIV/0!` | `b0b2419` |
| A5 PERMUTATIONA overflow | `PERMUTATIONA(163,150)` | `+Inf` | `#NUM!` | `b0b2419` |
| A6 FISHERINV saturation | `FISHERINV(817.81)` | `NaN` | `1` | `b0b2419` |
| A7 MROUND zero-sign | `MROUND(0,-4.2)` | `#NUM!` | `0` | already in the 039 batch |

A new shared `finite_or_num` guard (`excel_numeric.rs`) maps non-finite scalar results
to `#NUM!`; it is applied per-function because saturating functions (COTH/TANH) return
`±1` and must NOT use it. The same non-finite leak in other unary kernels (EXP, ...) is
tracked separately as bead `oxf-vgxs`.

## Closure Checklist
- [x] CLASS-A1..A7 minimized into focused tests and repair landed (2026-06-19)
- [ ] CLASS-B1..B3 Excel-doctrine threshold pinned and modelled
- [ ] CLASS-C1..C5 substrate-by-substrate kernel correction landed
- [x] follow-up beads opened for each class group and tracked in `.beads/` (CLASS-A audit `oxf-vgxs`; B/C beads pending)
- [ ] handoff to OxFml not required so far (no seam-side surface affected)

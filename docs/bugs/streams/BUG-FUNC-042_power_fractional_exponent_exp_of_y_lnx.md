# BUG-FUNC-042 — POWER fractional exponent: Excel is `exp(y·ln x)`, not `powf`

Status: `RESOLVED` — bit-exact, landed in `power_kernel` (commit `af79125`)
Owner workset: W108 (Phase D)
Catalog: resolved (removed from the open tracker)
Reproduced on: live Excel 16.0 build 20131, 64-bit, AMD Zen2, Value2 cell-ref plumbing

## Resolution (2026-07-04)

`POWER` is now bit-exact (715/715 live rows: 315 reverse-engineering ground truth +
400 fresh confirmation). The fractional path is `exp(y·ln x)` via the x87 `exp`/`ln` with
a **double-rounded** product; for `y < 0` Excel computes the positive power, stores it to
`binary64`, then takes **one** x87 double-rounded reciprocal (`exp(y·ln x)` was the right
function at the wrong point — the earlier 1-ULP residual). Two corrections beyond the
first-pass `exp(y·ln x)`:
1. the `y<0` **reciprocal staging** (`C:/Temp/ExcelExpFunction/POWER_REPORT.md`, 315/315);
2. exponent `|y| == 0.5` is the correctly-rounded hardware **`sqrt`**, NOT `exp(0.5·ln x)`
   — found by an OxFunc live sweep the reference missed (it only tested `0.5` on negative
   bases). `base.sqrt()` (SSE2 SQRTSD) is CPU-independent.

Implemented: `power_kernel` (integer→binexp, fractional→`excel_pow_positive` with the
`0.5→sqrt` case, `y<0`→`excel_x87_recip`), new x87 primitives `x87::{mul,recip}` (the
double-rounded PC=64 FMUL/FDIV). Full spec + validated test suite:
`docs/EXCEL_POWER_SPEC_AND_TEST_CASES.md`.

Original characterization (kept for history) follows.

## Summary

`POWER(x, y)` (and `x ^ y`) for a **fractional exponent with a positive base**
currently calls `f64::powf` (UCRT). Live Excel computes it as **`exp(y · ln x)`**
using the x87 `exp`/`ln` (`crate::excel_numeric::x87`) with **f64 intermediates** —
not `powf`, and not the fused x87 `x^y` (`FYL2X` + `F2XM1`) chain.

The divergence is large for big `|y|` (log error amplified by the exponent): up
to ~125 ULP on the sampled adversarial set. This was masked before because the
earlier "POWER non-bug" check used typical/integer exponents; integer exponents
take the validated `powi` publication path and are unaffected.

## Composition bake-off (220-row live Excel sweep)

| candidate | bit-exact |
|-----------|-----------|
| `exp(y · ln x)`, f64 intermediates (x87 exp/ln) | **189 / 220 (86%)** |
| `powf(x, y)` (current) | 12 / 220 |
| fused x87 `2^(y · log2 x)` (`FYL2X`+`F2XM1`) | ~10 / 220 |
| `y · ln x` at 80-bit then exp | 22 / 220 |

Head-to-head (90 rows): `exp(y·ln x)` strictly closer to Excel than `powf` on
**84/90**, tied 5, worse 1 (a 1-ULP row where `powf` was exact). So the exp model
dominates `powf`.

## What's unresolved

The ~14% residual is **all exactly 1 ULP**. Each sub-op (x87 `exp`, x87 `ln`,
the f64 `*`) is individually bit-exact to Excel, yet the composition lands 1 ULP
off on these rows. The deciding intermediate-precision detail (a different
internal `ln`? a fused reduction inside Excel's POWER?) is unknown — the same
"bespoke internal routine" class that EXP/LN needed a full reverse-engineering
pass to crack (`C:/Temp/ExcelExpFunction`).

## Decision (Phase A, 2026-07-04)

Kept `powf` in `power_kernel` (no partial 86% swap into the bit-exact commit). A
dedicated Phase-D pass should either reverse-engineer the last 1-ULP detail
(target 100%) or land `exp(y·ln x)` as the best-achievable improvement.

## Witnesses (exact doubles; columns: Excel / `exp(y·ln x)` / `powf` bit patterns)

### AGREE — the exp model reproduces Excel (powf usually does not)

| base | exp | Excel bits | exp(y·ln x) bits | powf bits | exp Δulp | powf Δulp |
|---|---|---|---|---|---|---|
| 384.626 | -1.04133 | 0x3f60a779471f9ac7 | 0x3f60a779471f9ac7 | 0x3f60a779471f9ac6 | 0 | 1 |
| 0.301307 | 11.6864 | 0x3eab5e6fde24daaa | 0x3eab5e6fde24daaa | 0x3eab5e6fde24daaf | 0 | 5 |
| 1.92168 | 6.67538 | 0x40539236de4d2e6b | 0x40539236de4d2e6b | 0x40539236de4d2e69 | 0 | 2 |
| 0.0126124 | 12.7759 | 0x3ae50efaefc785ec | 0x3ae50efaefc785ec | 0x3ae50efaefc785ed | 0 | 1 |
| 0.000815102 | 9.77687 | 0x39a9ac5893c70c29 | 0x39a9ac5893c70c29 | 0x39a9ac5893c70c31 | 0 | 8 |

### DIVERGE — the 1-ULP puzzle (exp model is the closest known)

| base | exp | Excel bits | exp(y·ln x) bits | powf bits | exp Δulp | powf Δulp |
|---|---|---|---|---|---|---|
| 1237.28 | -1.18233 | 0x3f2ceb593d898c6a | 0x3f2ceb593d898c69 | 0x3f2ceb593d898c61 | 1 | 9 |
| 0.000170605 | -13.0213 | 0x4a1fc17fcee62254 | 0x4a1fc17fcee62253 | 0x4a1fc17fcee622d0 | 1 | 124 |
| 0.00736075 | -8.51268 | 0x43b3fa20152f46be | 0x43b3fa20152f46bd | 0x43b3fa20152f46d8 | 1 | 26 |
| 0.240842 | -23.1929 | 0x42e8d6a12bc87d53 | 0x42e8d6a12bc87d52 | 0x42e8d6a12bc87d5a | 1 | 7 |
| 0.1769 | -19.1416 | 0x42ec89fc4a92a306 | 0x42ec89fc4a92a305 | 0x42ec89fc4a92a302 | 1 | 4 |
| 59.8749 | -20.6746 | 0x384eb120beb7818a | 0x384eb120beb7818b | 0x384eb120beb781df | 1 | 85 |
| 76.5796 | -16.6718 | 0x396928d25ee06551 | 0x396928d25ee06550 | 0x396928d25ee06542 | 1 | 15 |
| 903.572 | -10.2134 | 0x39aa29f7f8c66b59 | 0x39aa29f7f8c66b58 | 0x39aa29f7f8c66b70 | 1 | 23 |
| 231.915 | -5.0228 | 0x3d7728fb5a070288 | 0x3d7728fb5a070289 | 0x3d7728fb5a070283 | 1 | 5 |
| 1.54636 | -14.3919 | 0x3f5ee5316e66d47c | 0x3f5ee5316e66d47d | 0x3f5ee5316e66d47c | 1 | 0 |

Reproduce with the `x87lab` scratch harness (`x87lab4.rs` composition bake-off,
`compare_pow.ps1` driving Excel through `smart-fuzzer/tools/CellRefBatch.psm1`).

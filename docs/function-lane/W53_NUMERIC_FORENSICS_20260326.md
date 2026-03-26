# W53 Numeric Forensics - 2026-03-26

Purpose: preserve the exact numeric readback and investigation notes for the `W053` worksheet-value mismatches, including the final explanation and repair path once confirmed.

## 1. Exact Excel `Value2` Readback

Verified on `2026-03-26` through local Excel COM using round-trip (`"R"`) formatting on `Value2`:

1. `=ASINH(1)` -> `0.8813735870195429`
2. `=PV(0.05,10,-100)` -> `772.1734929184813`
3. `=FV(0.05,10,-100)` -> `1257.789253554883`
4. `=PMT(0.05,10,1000)` -> `-129.50457496545667`

Current OxFunc seam readback:

1. `ASINH(1)` -> `0.881373587019543`
2. `PV(0.05,10,-100)` -> `772.1734929184817`
3. `FV(0.05,10,-100)` -> `1257.789253554884`
4. `PMT(0.05,10,1000)` -> `-129.50457496545664`

ULP deltas against the current Excel baseline:

1. `ASINH`: `1` ULP
2. `PV`: `3` ULP
3. `FV`: `4` ULP
4. `PMT`: `1` ULP

## 2. `ASINH` Investigation

### 2.1 Current Kernel

OxFunc currently delegates `ASINH` to Rust's `f64::asinh()` in `crates/oxfunc_core/src/functions/asinh.rs`.

### 2.2 Candidate Excel-Like Publication Formula

The following Excel-side comparison sweep was run against live Excel:

1. sample points: `-10`, `-1`, `-0.5`, `-0.1`, `0`, `0.1`, `0.5`, `1`, `10`
2. additional magnitude probes: `-1E20`, `-1E10`, `-1E-10`, `1E-10`, `1E10`, `1E20`
3. compared:
   - Rust/.NET style `asinh`
   - `sign(x) * ln(|x| + sqrt(x^2 + 1))`

Observed result:

1. `f64::asinh()` matches Excel on most sampled points but misses `x = -1`, `x = 1`, `x = -1E-10`, and `x = 1E-10`.
2. `sign(x) * ln(|x| + sqrt(x^2 + 1))` matched every sampled Excel `Value2` result exactly.
3. The small-input lane is especially diagnostic: Excel returns `ASINH(1E-10) = 1.000000082690371E-10`, while `f64::asinh()` returns `1E-10`.

Current conservative reading:

1. `ASINH` has a strong OxFunc-local repair candidate.
2. The likely Excel publication model for current-baseline `ASINH` is closer to `sign(x) * ln(|x| + hypot(x, 1))` than to the platform libm `asinh`.
3. That repair has now been applied locally in `crates/oxfunc_core/src/functions/asinh.rs`.
4. Verification after the repair:
   - `cargo test --manifest-path crates/oxfunc_core/Cargo.toml asinh -- --nocapture` passed.
   - `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --test oxfml_seam_integration -- --nocapture` no longer reports `FN-ASINH-01`.

## 3. `PV`, `FV`, and `PMT` Investigation

### 3.1 Why The First High-Precision Pass Was Insufficient

An early calculation using exact decimal literals (`0.05`, `10`, `-100`, `1000`) lands on the Excel side for all three functions, but that is not yet enough for a code change because OxFunc runtime inputs are already parsed into binary `f64`.

### 3.2 Exact `f64`-Input Rational Reconstruction

Recomputing the three closed-form identities from the exact binary `f64` inputs currently passed by OxFunc gives:

1. `PV` exact-from-`f64` -> nearest `f64` `772.1734929184812`
2. `FV` exact-from-`f64` -> nearest `f64` `1257.7892535548829`
3. `PMT` exact-from-`f64` -> nearest `f64` `-129.5045749654567`

This means:

1. Excel is not simply "exact arithmetic on the same `f64` inputs, then round once at the end" for these rows.
2. OxFunc's current double-only formula order is also not matching Excel.
3. The current baseline likely depends on a different numeric discipline for these functions:
   - decimal-literal normalization,
   - extended/intermediate precision,
   - or a published-result policy above naive binary closed forms.

### 3.3 Current Conservative Reading

An additional Excel invariance probe was also run:

1. `PV`, `FV`, and `PMT` were evaluated with the rate supplied as:
   - a formula literal `0.05`,
   - a worksheet cell containing typed value `0.05`,
   - a worksheet formula `=0.1/2`,
   - a worksheet formula `=SQRT(0.0025)`.
2. Excel published the same `Value2` result for all four supply forms in each function.
3. The worksheet cells themselves round-tripped as `0.05`.

So the current evidence rules out a narrow "literal-only parsing quirk" explanation, but it still does not fully pin the finance-family arithmetic/publication model.

### 3.4 Candidate Models Checked And Rejected

The following local candidate models were checked against a broader live-Excel grid including:

1. `rate = 0.05`, `nper = 10`, both `type = 0` and `type = 1`,
2. `rate = 0.01`, `nper = 48`,
3. `rate = 0.08/12`, `nper = 10`,
4. `PV`, `FV`, and `PMT` on each lane.

Rejected as a universal repair model:

1. **Current closed-form `f64` kernel**
   - matches OxFunc's current residuals, but remains off the Excel baseline on the disputed rows.
2. **Decimal-literal exact closed forms**
   - improves the seeded `0.05` rows and exactly matches the sampled `PMT(0.08/12,10,10000)` row,
   - but is materially worse on other checked rows such as the `0.01/48` `PV` / `FV` lanes.
3. **`log1p` / `expm1` stable-growth closed forms**
   - not a general improvement across the checked grid.
4. **Per-period iterative recurrence**
   - improves some `FV` / `PMT` rows,
   - but is materially worse on `PV` and on the near-cancelling `FV(0.08/12,10,-1037.0320893591607,10000)` lane.
5. **`Microsoft.VisualBasic.Financial` reference implementation**
   - matches OxFunc's current closed-form results closely,
   - and therefore does not explain the observed Excel-vs-OxFunc drift.

Current reading after these checks:

1. there is no single simple substitute model yet that reproduces the current Excel baseline across the tested finance lanes,
2. the remaining `PV` / `FV` / `PMT` gaps are therefore still real, but not yet safely repairable by a narrow formula swap,
3. a richer finance publication packet is still required before any runtime change.

1. `PV`, `FV`, and `PMT` remain real OxFunc-local parity/publication residuals.
2. They are not fixture errors.
3. They are not OxFml seam defects anymore.
4. They are not yet ready for an implementation change without a broader evidence packet, because the exact numeric model responsible for the current Excel results is still not pinned honestly.

### 3.5 Shared `POWER` Publication Probe

An additional live-Excel probe then targeted the shared exponentiation path directly:

1. `POWER(1.05,10)` -> `1.6288946267774416`
2. `POWER(1.01,48)` -> `1.6122260776824653`
3. `POWER(1+0.08/12,10)` -> `1.0687026403740616`

Comparison against local models:

1. platform `powf` missed all three rows,
2. naive left-to-right repeated multiplication matched only the monthly row,
3. exponentiation by squaring matched all three rows exactly at the `f64` bit level.

This reclassifies the finance drift:

1. the remaining `PV` / `FV` / `PMT` mismatches were not finance-family-specific,
2. they inherited the same integer-growth publication drift already visible in `POWER`,
3. the correct repair target was the shared power publication path, not an isolated finance formula rewrite.

### 3.6 Repair Outcome

OxFunc now:

1. uses the Excel-matching exponentiation-by-squaring publication path for integer-valued exponents in `crates/oxfunc_core/src/functions/power_fn.rs`,
2. routes the finance growth helper through that shared path in `crates/oxfunc_core/src/functions/financial_time_value_family.rs`.

Verification after the repair:

1. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml power_fn -- --nocapture` passed.
2. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml financial_time_value_family -- --nocapture` passed.
3. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --test oxfml_seam_integration -- --nocapture` passed.
4. `lake build` passed after adding the matching Lean executable alignment layer in `formal/lean/OxFunc/Functions/PowerFn.lean` and `formal/lean/OxFunc/Functions/FinancialTimeValueFamily.lean`.

## 4. Current Owner Split

1. `ASINH`: repaired locally and removed from the residual seam-failure set.
2. `PV`, `FV`, `PMT`: repaired locally after the shared `POWER` publication path was characterized and aligned.

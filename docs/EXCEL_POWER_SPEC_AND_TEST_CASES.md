# Microsoft Excel `POWER(x, y)` — behavior and bit-exact test cases

*Self-contained reference. Target: Excel 16.0 build 20131, 64-bit, Windows (AMD Zen2).
All floating-point values are IEEE-754 `binary64`; inputs were fed to Excel as exact
doubles via a worksheet cell (`Range.Value2`), never as decimal formula text, and results
were read back as raw 64-bit patterns. Decimal renderings are for readability only — the
hex bit patterns are authoritative.*

---

## 1. One-line summary

Excel's `POWER(x, y)` (and the `^` operator) is **not** a single library `pow`. It is a
regime-switched function:

- **integer exponent** → repeated multiplication (binary exponentiation), *not* `pow`;
- **fractional exponent, positive base** → `exp(y · ln x)` computed with Excel's internal
  **x87** `exp`/`ln` and `binary64` intermediates — **not** `powf`, and **not** the fused
  x87 `x^y` instruction sequence;
- **negative base** → error, except exact reciprocal-odd-integer roots, which are
  `-exp(y · ln|x|)`;
- **domain edges** (`0^0`, `0^negative`, overflow) → specific worksheet errors.

**Status: RESOLVED — bit-exact.** The full algorithm reproduces live Excel on 715/715 rows
(315 reverse-engineering ground truth + 400 fresh confirmation). Two subtleties complete the
fractional path: (a) for **`y < 0`** Excel computes the *positive* power, stores it to
`binary64`, then takes **one** x87 double-rounded reciprocal — `exp(y·ln x)` was the right
function evaluated at the wrong point (§4); (b) for exponent **exactly `0.5`** Excel uses the
correctly-rounded hardware `sqrt`, not `exp(0.5·ln x)` (§4a).

---

## 2. Background: how Excel evaluates `POWER`

Excel's elementary transcendentals (`EXP`, `LN`, and the `exp`/`ln` used inside `POWER`)
are the **legacy Microsoft x87 CRT sequence** (`87tran.asm`), executed with the x87 control
word `0x133F` — precision-control 64-bit, round-to-nearest-even, all exceptions masked —
then stored to `binary64` with one final round. (Excel's ordinary `+`/`*` arithmetic, by
contrast, is pure SSE2 `binary64`; only the transcendental *functions* are x87.)

`POWER(x, y)` dispatches by the shape of `x` and `y`:

| Case | What Excel computes |
|------|---------------------|
| `y` an exact integer | `x^y` by LSB-first binary exponentiation (square-and-multiply), reciprocal if `y<0`. Gives exact integers where math is exact (`POWER(3,3)=27`), and its own rounding elsewhere. |
| `y = ±0.5` | positive power is the correctly-rounded hardware `sqrt(|x|)` (SSE2 SQRTSD), NOT `exp(0.5·ln x)`; for `y=-0.5` reciprocate. |
| `y` fractional (other), `x > 0` | `p = exp_x87(RN53(RN64(\|y\| · ln_x87 x)))` — x87 `ln`, x87 **double-rounded** product, x87 `exp`. If `y<0`: `r = RN53(RN64(1/p))` (x87 double-rounded reciprocal); publish `r`. |
| `y` fractional, `x < 0` | `#NUM!`, **except** when `1/y` rounded to 15 significant decimal digits is an **odd** integer within signed int32 (a real root): then `-POWER(\|x\|, y)`. |
| `x = 0`, `y = 0` | `#NUM!` |
| `x = 0`, `y < 0` | `#DIV/0!` |
| overflow (result `> ~1.8e308`) | `#NUM!` |
| underflow (result `< smallest double`, `y<0`) | `0` (or `#DIV/0!` if it went via `1/0`) |

Note the two logarithms in the fractional path use the **same** internal `ln`, and the
product `y · ln x` is rounded to `binary64` (an 80-bit product does *not* match Excel — see
§4), before the final x87 `exp`.

---

## 3. Reference candidates compared in this document

For each fractional/positive-base test case, three independent implementations are shown:

- **`excel`** — the value Excel returns (ground truth).
- **`exp(y·ln x)`** — `exp(y * ln(x))` using an x87 `exp`/`ln` with `binary64` intermediates.
  This is the model that best matches Excel.
- **`powf`** — the platform `pow(x, y)` (Windows UCRT / a correctly-rounded-ish libm). This
  is what a naive implementation uses; it is frequently far from Excel.

---

## 4. The resolution

A first pass found `exp(y·ln x)` (x87, `binary64` intermediates) matched a 220-row
fractional/positive-base sweep only **86%**, with the residual **all exactly 1 ULP** —
and all on **negative exponents**. Two facts closed the gap:

**(a) `y < 0` is reciprocal-staged.** The 1-ULP-exactly signature rules out any
product-level difference (that would be multi-ULP). Excel never evaluates `exp` at the
negative argument: it computes the **positive** power `p = exp_x87(RN53(RN64(|y|·ln x)))`,
**stores `p` to `binary64`**, then takes **one** x87 double-rounded reciprocal
`r = RN53(RN64(1/p))`. The extra `binary64` store of `p` perturbs the final rounding by
±½ ULP on ~28% of negative-`y` rows — exactly the observed residual. (Also settled: the
`|y|·ln x` product and the `1/p` divide are both x87 **double-rounded** RN64→RN53, not
single-rounded SSE; and the negative-base odd-root test is a **decimal** 15-sig-digit fuzz,
not binary.) This model scored **315/315** across the reverse-engineering ground truth.

**(b) exponent `0.5` is `sqrt`.** A fresh confirmation sweep found the one remaining class:
Excel evaluates `POWER(x, 0.5)` as the **correctly-rounded hardware `sqrt(x)`** (SSE2
SQRTSD), not `exp(0.5·ln x)` — the latter is 1 ULP low, e.g. `POWER(2, 0.5)` = √2 =
`0x3ff6a09e667f3bcd` vs `exp(0.5·ln 2)` = `0x…bcc`. It applies to `|y| == 0.5` (so `y=-0.5`
reciprocates `sqrt`); every other tested fractional exponent (0.25, 0.75, 1/3, 1.5, 2.5, …)
uses `exp∘ln`. Because it is hardware `sqrt`, the `0.5` path is CPU-independent (no x87
microcode caveat). With (a)+(b) the full algorithm reproduces live Excel **400/400** on a
fresh sweep spanning both signs, negative-base roots, integer, subnormal, and error rows.

The only residual caveat is the shared x87 `exp`/`ln` per-CPU-family microcode on the
hardest ~1-in-2000 general-fractional inputs (same as `EXP`/`LN`); validated on AMD Zen2.

---

## 5. Test cases

### 5A. Domain and error semantics

| Call | Excel result |
|------|--------------|
| `POWER(2, 3)` | `8` |
| `POWER(2, -3)` | `0.125` |
| `POWER(2, -1022)` | `2.2250738585072014e-308` (smallest normal) |
| `POWER(2, -1023)` | `0` (underflow) |
| `POWER(0, 0)` | `#NUM!` |
| `POWER(0, -1)` | `#DIV/0!` |
| `POWER(-1, 0.5)` | `#NUM!` |
| `POWER(10, 700)` | `#NUM!` (overflow) |
| `POWER(10, -700)` | `0` (underflow) |
| `POWER(0.001, -700)` | `#DIV/0!` (`1 / underflow`) |

### 5B. Integer exponent — binary exponentiation, *not* `pow`

Excel publishes an integer-exponent power by repeated multiplication; the result differs
from `pow(x, y)` in the low bits. (Bit patterns are the exact Excel doubles.)

| Call | Excel result (decimal) | Excel result (hex) |
|------|------------------------|--------------------|
| `POWER(1.05, 10)` | `1.6288946267774416` | `0x3ffa0ff3cfea3a51` |
| `POWER(1.01, 48)` | `1.6122260776824653` | `0x3ff9cbad92567903` |
| `POWER(1.0066666666666666, 10)` | `1.0687026403740616` | `0x3ff11967f098e319` |

### 5C. Negative base — exact reciprocal-odd-integer roots only

`POWER(negative, y)` is `#NUM!` unless `1/y` (to 15 significant decimal digits) is an odd
integer within signed int32 (a real root), evaluated as `-POWER(|x|, y)`:

| Call | Excel result |
|------|--------------|
| `POWER(-8, 1/3)` | `-1.9999999999999998` |
| `POWER(-27, 1/3)` | `-2.9999999999999996` |
| `POWER(-32, 1/5)` | `-2` |
| `POWER(-8, -1/3)` | `-0.5` (exact — via the double-rounded reciprocal tie) |
| `POWER(-8, 2/3)` | `#NUM!` (2/3 → 1/y=1.5, not an integer) |
| `POWER(-8, 0.5)` | `#NUM!` (1/y = 2 is even) |

### 5C-b. Exponent `0.5` — hardware `sqrt`, positive base

Excel evaluates `POWER(x, 0.5)` as the correctly-rounded `sqrt(x)`, NOT `exp(0.5·ln x)`:

| Call | Excel result (bits) | note |
|------|---------------------|------|
| `POWER(2, 0.5)` | `0x3ff6a09e667f3bcd` (√2) | `exp(0.5·ln 2)` gives `0x…bcc`, 1 ULP low |
| `POWER(16, 0.5)` | `4` | exact |
| `POWER(4, -0.5)` | `0.5` | `sqrt` then reciprocal |

### 5D. Fractional exponent, positive base — AGREE cases

`exp(y·ln x)` reproduces Excel bit-for-bit; `powf` does not (its ULP gap is shown). Columns:
input base and exponent (hex bits, then decimal), then the three implementations' result
bits, then the ULP distance of `powf` from Excel.

| base (hex) | base | exp (hex) | exp | excel = exp(y·ln x) | powf | powf Δulp |
|---|---|---|---|---|---|---|
| `0x40780a045b93fb26` | 384.626 | `0xbff0a944c2b86230` | -1.04133 | `0x3f60a779471f9ac7` | `0x3f60a779471f9ac6` | 1 |
| `0x3fd3489e12d7954b` | 0.301307 | `0x40275f6d3a0c9c18` | 11.6864 | `0x3eab5e6fde24daaa` | `0x3eab5e6fde24daaf` | 5 |
| `0x3ffebf325a893646` | 1.92168 | `0x401ab395f64cd6bc` | 6.67538 | `0x40539236de4d2e6b` | `0x40539236de4d2e69` | 2 |
| `0x3f89d4841507ae27` | 0.0126124 | `0x40298d491b26e5e0` | 12.7759 | `0x3ae50efaefc785ec` | `0x3ae50efaefc785ed` | 1 |
| `0x3f4ab5913d1078a7` | 0.000815102 | `0x40238dc160b01950` | 9.77687 | `0x39a9ac5893c70c29` | `0x39a9ac5893c70c31` | 8 |
| `0x3fe30d69b73dac74` | 0.595387 | `0x400799a462ae97b0` | 2.95002 | `0x3fcbb97a725deac6` | `0x3fcbb97a725deac5` | 1 |
| `0x403addad114f5bb0` | 26.8659 | `0x402f852ec4afd910` | 15.7601 | `0x449c5549659e0037` | `0x449c5549659e0048` | 17 |
| `0x3f33fccfa2606a54` | 0.000304986 | `0x4026798bea923d64` | 11.2374 | `0x37bb12b3555d34d3` | `0x37bb12b3555d347f` | 84 |
| `0x4084c396e35ad7ed` | 664.449 | `0x4016ff2e6b34e54c` | 5.7492 | `0x434df3bb6cff4b27` | `0x434df3bb6cff4b23` | 4 |
| `0x40528e3ad74fd644` | 74.2223 | `0xc01ffb4b94abae6c` | -7.99541 | `0x3cd3f3209ce5902c` | `0x3cd3f3209ce59030` | 4 |
| `0x3f1801c7d5488414` | 9.15793e-05 | `0x4019032dcfe74638` | 6.2531 | `0x3ab159f7f1b498a2` | `0x3ab159f7f1b498c1` | 31 |
| `0x3f42cf37380348d0` | 0.000574018 | `0xc03371760cd57ead` | -19.4432 | `0x4d0437b0d8770bc1` | `0x4d0437b0d8770be2` | 33 |
| `0x3fed08280280bcb7` | 0.907246 | `0x402711b6bd9f5910` | 11.5346 | `0x3fd4d2d0824fe919` | `0x3fd4d2d0824fe918` | 1 |
| `0x3f096a0881090b3d` | 4.84737e-05 | `0x402236523cea7f84` | 9.1061 | `0x37c66e6ddfce5fab` | `0x37c66e6ddfce5f93` | 24 |
| `0x3fd209f85ffa21c4` | 0.281859 | `0x4032b5b635508294` | 18.7098 | `0x3dcc34fe7285b98e` | `0x3dcc34fe7285b994` | 6 |
| `0x4078bb6dce1d82ec` | 395.714 | `0xc019607668acc938` | -6.3442 | `0x3c8329a4c0519610` | `0x3c8329a4c05195f6` | 26 |
| `0x3f5809260e284d61` | 0.00146702 | `0xc014b1d7437d9108` | -5.17367 | `0x42f9fa5c396a3fbb` | `0x42f9fa5c396a3fba` | 1 |
| `0x3f7089d9fe3857a5` | 0.00403772 | `0x40362ea307d21034` | 22.1822 | `0x34e848972519bd2c` | `0x34e848972519bd19` | 19 |
| `0x3f3b5f01d0169560` | 0.00041765 | `0xc035628cf5bf874a` | -21.385 | `0x4ef09f93cfba3aee` | `0x4ef09f93cfba3afa` | 12 |
| `0x3fa4fdb253411d7d` | 0.040998 | `0xc0224474cf6af0fb` | -9.1337 | `0x42910a3e438790fe` | `0x42910a3e438790f6` | 8 |

### 5E. Fractional exponent, positive base — the former "puzzle" rows (now resolved)

These are the cases where the naive `exp(y·ln x)` column is **1 ULP** off Excel. They are all
`y < 0`: the **reciprocal-staged** model (positive power stored to f64, then one double-rounded
`1/p`) reproduces every one of them bit-exact — the `exp(y·ln x)` column below is shown only to
document why the staging is required. `powf` is farther still (its ULP gap shown).

| base (hex) | base | exp (hex) | exp | excel | exp(y·ln x) | powf | exp Δ | powf Δ |
|---|---|---|---|---|---|---|---|---|
| `0x4093551c15e0b619` | 1237.28 | `0xbff2ead4957fb090` | -1.18233 | `0x3f2ceb593d898c6a` | `0x3f2ceb593d898c69` | `0x3f2ceb593d898c61` | 1 | 9 |
| `0x3f265c8a48107e4d` | 0.000170605 | `0xc02a0aee3a890266` | -13.0213 | `0x4a1fc17fcee62254` | `0x4a1fc17fcee62253` | `0x4a1fc17fcee622d0` | 1 | 124 |
| `0x3f7e264ec94d00a6` | 0.00736075 | `0xc021067e218bf438` | -8.51268 | `0x43b3fa20152f46be` | `0x43b3fa20152f46bd` | `0x43b3fa20152f46d8` | 1 | 26 |
| `0x3fced3ed2b492e8a` | 0.240842 | `0xc03731630b339700` | -23.1929 | `0x42e8d6a12bc87d53` | `0x42e8d6a12bc87d52` | `0x42e8d6a12bc87d5a` | 1 | 7 |
| `0x3fc6a4a6d7430cf7` | 0.1769 | `0xc033244213b88aa0` | -19.1416 | `0x42ec89fc4a92a306` | `0x42ec89fc4a92a305` | `0x42ec89fc4a92a302` | 1 | 4 |
| `0x404deffe2214465f` | 59.8749 | `0xc034acb0e18b10b2` | -20.6746 | `0x384eb120beb7818a` | `0x384eb120beb7818b` | `0x384eb120beb781df` | 1 | 85 |
| `0x40532517d1c1fb3f` | 76.5796 | `0xc030abfb8e49bf0a` | -16.6718 | `0x396928d25ee06551` | `0x396928d25ee06550` | `0x396928d25ee06542` | 1 | 15 |
| `0x408c3c945addefe4` | 903.572 | `0xc0246d43645fac32` | -10.2134 | `0x39aa29f7f8c66b59` | `0x39aa29f7f8c66b58` | `0x39aa29f7f8c66b70` | 1 | 23 |
| `0x406cfd471e2f293b` | 231.915 | `0xc01417599d85c258` | -5.0228 | `0x3d7728fb5a070288` | `0x3d7728fb5a070289` | `0x3d7728fb5a070283` | 1 | 5 |
| `0x3ff8bde32998cbaa` | 1.54636 | `0xc02cc8a1467a59f2` | -14.3919 | `0x3f5ee5316e66d47c` | `0x3f5ee5316e66d47d` | `0x3f5ee5316e66d47c` | 1 | 0 |
| `0x3f6e803d5db62413` | 0.00372326 | `0xc030033b15d12eed` | -16.0126 | `0x48027f985f82651e` | `0x48027f985f82651f` | `0x48027f985f82651a` | 1 | 4 |
| `0x3f99aab88ff29092` | 0.0250653 | `0xc0288a44c835a4f8` | -12.2701 | `0x4403151823fcd1f1` | `0x4403151823fcd1f2` | `0x4403151823fcd1f0` | 1 | 1 |
| `0x3f3287fefb2f4e22` | 0.000282764 | `0xc031e2a7f1e5730c` | -17.8854 | `0x4d1c89c671ecc325` | `0x4d1c89c671ecc324` | `0x4d1c89c671ecc341` | 1 | 28 |
| `0x40580dc391256fe1` | 96.2151 | `0xc02ad70b653c5d1e` | -13.42 | `0x3a680636a7cc3af3` | `0x3a680636a7cc3af4` | `0x3a680636a7cc3abf` | 1 | 52 |
| `0x3f35555c1c5db44a` | 0.000325522 | `0xc0027ddbd2dd7298` | -2.31145 | `0x419b701886193c80` | `0x419b701886193c81` | `0x419b701886193c97` | 1 | 23 |
| `0x3fea8dd63b8fda6d` | 0.829814 | `0xc034be75d6e58f7f` | -20.744 | `0x4047f7cce8d618df` | `0x4047f7cce8d618de` | `0x4047f7cce8d618e1` | 1 | 2 |

**Observation → resolution.** Every row here has a **negative exponent** — the tell that led
to the reciprocal-staging fix (§4a). The reciprocal-staged model (compute the positive power
`p = exp(|y|·ln x)`, store to f64, then `RN53(RN64(1/p))`) reproduces the Excel column on all
of them bit-exact. The `exp(y·ln x)` column above is the *un-staged* value, kept to show the
1-ULP gap that staging closes; `powf` is 0–124 ULP off (amplified by `|y|`).

---

## 6. How to validate an implementation

1. Feed each `base`/`exp` pair to Excel as **exact doubles via cell values** (`Range.Value2`),
   never as decimal formula text (Excel's parser re-rounds long decimal literals and would
   corrupt the comparison). Read `=POWER(A1, B1)` back as its 64-bit pattern.
2. Compare bit patterns, not decimal strings — a shared 15-digit decimal prefix routinely
   hides a multi-ULP tail.
3. Expect the **full algorithm** (§2, with the §4 reciprocal staging and the `0.5→sqrt`
   special case) to be bit-exact on every row: 5A–5C, 5C-b, 5D, and 5E. The bare `exp(y·ln x)`
   column in 5E is the un-staged value and is 1 ULP off on those `y<0` rows by design.
4. The general-fractional path's x87 `exp`/`ln` is CPU-microcode-sensitive on the hardest
   inputs (`F2XM1`/`FYL2X`), so its bit-exactness is a property of the host x86-64 CPU family
   (validated on AMD Zen2). The `0.5→sqrt` path uses hardware SQRTSD and is CPU-independent.

---

## 7. Status

**RESOLVED — bit-exact.** The full algorithm (§2) reproduces live Excel on 715/715 rows
(315 reverse-engineering ground truth + 400 fresh confirmation). Adopted in OxFunc
(`power_kernel` → `crate::excel_numeric::excel_pow_positive` / `excel_x87_recip`), replacing
`powf`. The only residual is the shared x87 `exp`/`ln` per-CPU-family microcode caveat on the
hardest general-fractional inputs (same as `EXP`/`LN`); the integer path (binary exponentiation)
and the `0.5→sqrt` path are exact and CPU-independent.

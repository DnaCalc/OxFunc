# Function Slice - Engineering Conversions And Bessel Family Contract (Prelim)

Status: `active`
Owner lane: `OxFunc`
Workset: `W059`

## 1. Purpose
Define the current-phase contract for the `W059` engineering radix conversion family plus the Bessel quartet.

## 2. Covered Surface
1. `DEC2BIN`
2. `DEC2HEX`
3. `DEC2OCT`
4. `BIN2DEC`
5. `BIN2HEX`
6. `BIN2OCT`
7. `HEX2BIN`
8. `HEX2DEC`
9. `HEX2OCT`
10. `OCT2BIN`
11. `OCT2DEC`
12. `OCT2HEX`
13. `BESSELI`
14. `BESSELJ`
15. `BESSELK`
16. `BESSELY`

## 3. Engineering Radix Contract
1. all twelve radix functions use the ordinary values-only pre-adapter seam,
2. decimal-to-radix functions truncate `number` and optional `places` toward zero,
3. positive outputs use minimal digits unless `places` is supplied,
4. negative decimal inputs ignore `places` and return the fixed-width ten-character signed form for the target radix,
5. source-text conversions trim leading whitespace only; trailing whitespace remains invalid,
6. ten-character source strings use Excel's fixed-width signed interpretation:
   - binary: `10` bits
   - octal: `30` bits
   - hexadecimal: `40` bits
7. target overflow and invalid source syntax return `#NUM!`.

## 4. Bessel Contract
1. the quartet uses the ordinary values-only pre-adapter seam and custom numeric kernels,
2. order arguments truncate toward zero before evaluation,
3. negative orders return `#NUM!`,
4. `BESSELK` and `BESSELY` reject `x <= 0` with `#NUM!`,
5. `BESSELI` and `BESSELJ` accept signed `x` and preserve odd-order sign parity,
6. orders `0` and `1` use compact approximation kernels; higher orders use recurrence.

## 5. Runtime / Formal Anchors
Runtime anchors:
1. `crates/oxfunc_core/src/functions/engineering_radix_family.rs`
2. `crates/oxfunc_core/src/functions/bessel_convert_family.rs`

Formal anchors:
1. `formal/lean/OxFunc/Functions/EngineeringRadixFamily.lean`
2. `formal/lean/OxFunc/Functions/BesselConvertFamily.lean`

Native replay anchors:
1. `docs/function-lane/W59_SCENARIO_MANIFEST_SEED.csv`
2. `tools/w59-probe/run-w59-engineering-conversions-bessel-baseline.ps1`
3. `.tmp/w59-engineering-conversions-bessel-results.csv`

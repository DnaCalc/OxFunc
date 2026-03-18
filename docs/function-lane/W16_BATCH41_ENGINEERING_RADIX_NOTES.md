# W16 Batch 41 - Engineering Radix Conversion Family

Status: `in_progress-provisional`
Workset: `W16`
Evidence ID: `W16-BATCH41-ENGINEERING-RADIX-20260316`

## Scope
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

## Native Excel Baseline
Probe artifact:
1. `.tmp/w16-batch41-engineering-radix-probe.csv`

## Current Semantics Baseline
1. All twelve functions use the ordinary values-only pre-adapter seam and custom kernel path, matching the local `BASE` / `DECIMAL` family shape.
2. Decimal-to-radix functions truncate the numeric `number` and optional `places` arguments toward zero in the current local baseline.
3. Positive outputs use minimal digits unless `places` is supplied; `places` left-pads with ASCII `0` and returns `#NUM!` when the result would exceed the requested width.
4. Negative decimal inputs ignore `places` and return the fixed-width ten-character two's-complement form for the target radix.
5. Binary, octal, and hexadecimal text inputs accept uppercase and lowercase ASCII digits and trim leading whitespace only; trailing whitespace remains invalid and returns `#NUM!`.
6. Text-input conversions treat ten-character inputs as fixed-width signed values using the Excel engineering-family sign-bit rule (`10` binary bits, `30` octal bits, `40` hexadecimal bits).
7. Cross-radix conversions reject source strings longer than ten characters and reject target overflows with `#NUM!`.

## Pinned Local Test Rows
1. `DEC2BIN(9,4) -> "1001"`
2. `DEC2BIN(-100,2) -> "1110011100"` because negative inputs ignore `places`
3. `DEC2HEX(100,4) -> "0064"`
4. `DEC2HEX(-54,3) -> "FFFFFFFFCA"`
5. `DEC2OCT(58,3) -> "072"`
6. `DEC2OCT(-100) -> "7777777634"`
7. `BIN2DEC("1100100") -> 100`
8. `BIN2DEC("1111111111") -> -1`
9. `BIN2HEX("11111011",4) -> "00FB"`
10. `BIN2HEX("1111111111",1) -> "FFFFFFFFFF"`
11. `BIN2OCT("1001",3) -> "011"`
12. `HEX2BIN("F",8) -> "00001111"`
13. `HEX2DEC("FFFFFFFF5B") -> -165`
14. `HEX2OCT("FFFFFFFF5B",1) -> "7777777533"`
15. `OCT2BIN("3",4) -> "0011"`
16. `OCT2DEC("54") -> 44`
17. `OCT2HEX("100",4) -> "0040"`

## Integration Note
1. The family is now wired through the shared Rust dispatch/export surfaces and the root Lean import.
2. Additional empirical characterization may still be needed for undocumented coercion corners beyond the pinned rows above.

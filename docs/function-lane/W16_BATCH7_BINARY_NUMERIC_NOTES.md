# W16 Batch 7 - Binary Numeric Scalar Functions

Status: `in_progress-provisional`
Workset: `W16`
Evidence ID: `W16-BATCH7-BINARY-NUMERIC-20260315`

## Scope
1. `POWER`
2. `MOD`
3. `QUOTIENT`
4. `MROUND`

## Native Excel Baseline
Probe artifact:
1. `.tmp/w16-batch7-binary-probe.csv`

Pinned lanes:
1. `POWER(2,3) -> 8`
2. `POWER(0,-1) -> #DIV/0!`
3. `POWER(-1,0.5) -> #NUM!`
4. `MOD(-3,2) -> 1`
5. `MOD(3,-2) -> -1`
6. `MOD(3,0) -> #DIV/0!`
7. `QUOTIENT(-7,3) -> -2`
8. `QUOTIENT(1,0) -> #DIV/0!`
9. `MROUND(10,3) -> 9`
10. `MROUND(-10,-3) -> -9`
11. `MROUND(10,-3) -> #NUM!`
12. `MROUND(1.25,0.5) -> 1.5`
13. `MROUND(5,0) -> 0`

## Implementation Notes
1. All four functions fit the existing values-only exact-binary numeric seam.
2. `POWER` requires two special domain lanes:
   - `0` to a negative power is `#DIV/0!`
   - negative base with non-integer exponent is `#NUM!`
3. `MOD` follows Excel's divisor-sign rule rather than Rust `%`.
4. `QUOTIENT` truncates division toward zero.
5. `MROUND` rejects sign mismatch with `#NUM!` and returns `0` when `multiple = 0`.

# W16 Batch 33 - Decimal Base-Decode Function

Status: `in_progress-provisional`
Workset: `W16`
Evidence ID: `W16-BATCH33-DECIMAL-20260315`

## Scope
1. `DECIMAL`

## Native Excel Baseline
Probe artifact:
1. `.tmp/w16-batch33-roman-decimal-probe.csv`

Pinned lanes:
1. `DECIMAL("FF",16) -> 255`
2. `DECIMAL("111",2) -> 7`
3. `DECIMAL("Z",36) -> 35`
4. `DECIMAL("G",16) -> #NUM!`
5. `DECIMAL("10",1) -> #NUM!`
6. `DECIMAL("10",37) -> #NUM!`
7. `DECIMAL("",16) -> 0`
8. `DECIMAL("ff",16) -> 255`
9. `DECIMAL(TRUE,2) -> #NUM!`
10. `DECIMAL(10,2) -> 2`
11. `DECIMAL(10.5,2) -> #NUM!`
12. `DECIMAL("10",2.9) -> 2`
13. `DECIMAL("  10",2) -> 2`
14. `DECIMAL(CHAR(9)&"10",2) -> 2`
15. `DECIMAL("10"&CHAR(9),2) -> #NUM!`

## Current Implementation Notes
1. `DECIMAL` uses values-only preparation and text coercion for the digit string, so direct numeric arguments are textified before validation.
2. The radix is truncated toward zero before validation and must land in `2..36`.
3. The current baseline accepts leading whitespace but rejects trailing whitespace.
4. Empty text after leading-whitespace trimming returns `0`.
5. Any invalid digit for the selected radix returns `#NUM!`.

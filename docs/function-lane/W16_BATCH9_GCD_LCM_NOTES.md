# W16 Batch 9 - Integer Common-Divisor Functions

Status: `in_progress-provisional`
Workset: `W16`
Evidence ID: `W16-BATCH9-GCD-LCM-20260315`

## Scope
1. `GCD`
2. `LCM`

## Native Excel Baseline
Probe artifact:
1. `.tmp/w16-batch9-gcd-lcm-probe.csv`

Pinned lanes:
1. `GCD(24,36) -> 12`
2. `GCD(24.9,36.2) -> 12`
3. `GCD(0,5) -> 5`
4. `GCD(0,0) -> 0`
5. `GCD(-1,5) -> #NUM!`
6. `LCM(6,8) -> 24`
7. `LCM(6.9,8.1) -> 24`
8. `LCM(0,5) -> 0`
9. `LCM(0,0) -> 0`
10. `LCM(-1,5) -> #NUM!`

## Implementation Notes
1. Both functions are variadic values-only integer reducers.
2. Numeric inputs are truncated toward zero before the integer algorithm runs.
3. Negative inputs are `#NUM!`.

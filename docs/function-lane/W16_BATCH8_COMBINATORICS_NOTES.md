# W16 Batch 8 - Discrete Combinatorics Functions

Status: `in_progress-provisional`
Workset: `W16`
Evidence ID: `W16-BATCH8-COMBINATORICS-20260315`

## Scope
1. `COMBIN`
2. `COMBINA`
3. `MULTINOMIAL`

## Native Excel Baseline
Probe artifacts:
1. `.tmp/w16-batch8-combinatorics-probe.csv`
2. `.tmp/w16-batch8-combinatorics-edge-probe.csv`

Pinned lanes:
1. `COMBIN(5,2) -> 10`
2. `COMBIN(5.9,2.2) -> 10`
3. `COMBIN(5,6) -> #NUM!`
4. `COMBINA(4,3) -> 20`
5. `COMBINA(5.9,2.2) -> 15`
6. `COMBINA(0,0) -> 1`
7. `COMBINA(0,1) -> #NUM!`
8. `MULTINOMIAL(1,2,3) -> 60`
9. `MULTINOMIAL(1.9,2.9,3.1) -> 60`
10. `MULTINOMIAL(5) -> 1`
11. `MULTINOMIAL(0,0) -> 1`
12. `MULTINOMIAL(1,-1,2) -> #NUM!`

## Implementation Notes
1. The batch reuses the factorial and nonnegative-truncation substrate from Batch 6.
2. `COMBIN` and `COMBINA` fit the exact-binary numeric helper seam.
3. `MULTINOMIAL` is variadic and stays on the values-only scalar path for the current batch.

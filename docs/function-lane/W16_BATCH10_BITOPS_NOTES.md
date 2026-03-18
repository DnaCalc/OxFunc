# W16 Batch 10 - Bitwise Integer Functions

Status: `in_progress-provisional`
Workset: `W16`
Evidence ID: `W16-BATCH10-BITOPS-20260315`

## Scope
1. `BITAND`
2. `BITOR`
3. `BITXOR`
4. `BITLSHIFT`
5. `BITRSHIFT`

## Native Excel Baseline
Probe artifacts:
1. `.tmp/w16-batch10-bitops-probe.csv`
2. `.tmp/w16-batch10-bitops-edge-probe.csv`

Pinned lanes:
1. `BITAND(6,3) -> 2`
2. `BITOR(6,3) -> 7`
3. `BITXOR(6,3) -> 5`
4. `BITLSHIFT(1,3) -> 8`
5. `BITLSHIFT(16,-1) -> 8`
6. `BITRSHIFT(8,3) -> 1`
7. `BITRSHIFT(8,-1) -> 16`
8. negative operands are `#NUM!`
9. operands above `281474976710655` are `#NUM!`
10. `|shift| > 53` is `#NUM!`

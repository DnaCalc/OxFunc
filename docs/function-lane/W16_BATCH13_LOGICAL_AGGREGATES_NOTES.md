# W16 Batch 13 - Logical Aggregate Functions

Status: `in_progress-provisional`
Workset: `W16`
Evidence ID: `W16-BATCH13-LOGICAL-AGGREGATES-20260315`

## Scope
1. `OR`
2. `XOR`

## Native Excel Baseline
Probe artifact:
1. `.tmp/w16-batch13-logical-probe.csv`

Pinned lanes:
1. `OR(TRUE,FALSE) -> TRUE`
2. `OR(A1:A4) -> TRUE` when the range contains ignored text, a logical `TRUE`, a numeric `0`, and a blank
3. `OR("x") -> #VALUE!`
4. `OR(A1:A2) -> #VALUE!` when every reference-derived item is ignored
5. `XOR(TRUE,FALSE,TRUE) -> FALSE`
6. `XOR(A1:A4) -> TRUE` under the same mixed reference-derived lane
7. `XOR("x") -> #VALUE!`
8. `XOR(A1:A2) -> #VALUE!` when every reference-derived item is ignored

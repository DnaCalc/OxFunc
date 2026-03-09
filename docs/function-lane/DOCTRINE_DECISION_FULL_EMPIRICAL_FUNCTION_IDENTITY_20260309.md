# Doctrine Decision - Full Empirical Function Identity

Status: `active`
Date: `2026-03-09`
Owner lane: `OxFunc`

## 1. Purpose
Record the OxFunc doctrine decision that function implementations may not be treated as complete while any known Excel-semantic gap remains open.

## 2. Decision
1. OxFunc function implementations target full semantic identity with Excel for the declared version axes.
2. Partial, bounded, seed-only, or scaffolding semantics do not count as implemented functions.
3. When public documentation and empirical Excel behavior differ, OxFunc records the discrepancy explicitly and implements the empirically observed Excel behavior.
4. The only allowed limitation is in the XLL test/verification seam, where host-surface reproduction may be incomplete even though OxFunc runtime semantics must still target full Excel parity.
5. XLL verification-seam limitations must be documented centrally in seam artifacts and repeated in function verification records wherever those limitations materially qualify a function claim.

## 3. Reporting Consequences
1. A function with any known semantic gap remains `work-in-progress`.
2. Function slices and function-batch worksets with known semantic gaps must report `scope_partial`.
3. `complete for declared scope` is not valid for semantically bounded function slices or batches.
4. Cross-cutting scaffolding work may still be useful, but it must not be reported as function implementation closure.

## 4. Immediate Repository Implications
1. W12 and any similar packets must be reported as in progress while known semantic gaps remain.
2. Function contract docs should preserve explicit open-lane notes, but those notes do not convert partial semantics into completion.
3. Future workset design may still use exploratory packets, but packet closure must not imply function closure unless the full empirically determined Excel semantics are covered.

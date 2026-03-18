# W16 Batch 6 - Factorial Family Notes

Status: `active`
Owner lane: `OxFunc`
Relationship: sixth executable family inside `W016`

## 1. Family Members
1. `FACT`
2. `FACTDOUBLE`

## 2. Why This Family Follows Batch 5
1. both functions remain deterministic, nonvolatile, host-independent, and values-only at the adapter seam,
2. both fit the existing unary numeric scalar-or-array-elementwise lift path,
3. the family adds pure numeric combinatoric surface without introducing reference or host seams,
4. the only subtlety is truncation and negative-boundary behavior, which native Excel probes pin cleanly.

## 3. Empirical Findings
Native Excel COM probe on `2026-03-15` observed:
1. `FACT(5)` -> `120`
2. `FACT(5.9)` -> `120`
3. `FACT(-1)` -> `#NUM!`
4. `FACTDOUBLE(6)` -> `48`
5. `FACTDOUBLE(6.9)` -> `48`
6. `FACTDOUBLE(-1)` -> `1`
7. `FACTDOUBLE(-1.1)` -> `#NUM!`
8. `FACTDOUBLE(-0.1)` -> `1`
9. `FACTDOUBLE(0)` -> `1`

Probe artifact:
1. `.tmp/w16-batch6-factorial-probe.csv`

## 4. Current Local Artifact Shape
Runtime artifacts:
1. `crates/oxfunc_core/src/functions/factorial_common.rs`
2. `crates/oxfunc_core/src/functions/fact.rs`
3. `crates/oxfunc_core/src/functions/factdouble.rs`

Formal artifacts:
1. `formal/lean/OxFunc/Functions/Fact.lean`
2. `formal/lean/OxFunc/Functions/FactDouble.lean`

## 5. What Is Still Open
1. packet-level empirical replay remains thin relative to the full `W016` inventory,
2. per-function contract docs are still batched rather than individually expanded,
3. no `function-phase-complete` claim is made yet for any member of this family.

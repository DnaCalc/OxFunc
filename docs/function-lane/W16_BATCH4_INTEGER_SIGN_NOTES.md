# W16 Batch 4 - Integer Rounding and Sign Family Notes

Status: `active`
Owner lane: `OxFunc`
Relationship: fourth executable family inside `W016`

## 1. Family Members
1. `INT`
2. `SIGN`
3. `EVEN`
4. `ODD`

## 2. Why This Family Follows Batch 3
1. all four functions remain deterministic, nonvolatile, host-independent, and values-only at the adapter seam,
2. all four fit the existing unary numeric scalar-or-array-elementwise lift path,
3. the family extends the unary Q-capable math surface without introducing new domain errors or caller-context seams,
4. the only subtlety is negative-number rounding direction, which native Excel probes pin cleanly.

## 3. Empirical Findings
Native Excel COM probe on `2026-03-15` observed:
1. `INT(1.9)` -> `1`
2. `INT(-1.1)` -> `-2`
3. `SIGN(-2)` -> `-1`
4. `SIGN(0)` -> `0`
5. `EVEN(1.5)` -> `2`
6. `EVEN(-1.5)` -> `-2`
7. `ODD(1.5)` -> `3`
8. `ODD(-1.5)` -> `-3`
9. `ODD(2)` -> `3`

Probe artifact:
1. `.tmp/w16-batch4-integer-sign-probe.csv`

## 4. Current Local Artifact Shape
Runtime artifacts:
1. `crates/oxfunc_core/src/functions/int_fn.rs`
2. `crates/oxfunc_core/src/functions/sign_fn.rs`
3. `crates/oxfunc_core/src/functions/even_fn.rs`
4. `crates/oxfunc_core/src/functions/odd_fn.rs`

Formal artifacts:
1. `formal/lean/OxFunc/Functions/IntFn.lean`
2. `formal/lean/OxFunc/Functions/Sign.lean`
3. `formal/lean/OxFunc/Functions/Even.lean`
4. `formal/lean/OxFunc/Functions/Odd.lean`

## 5. What Is Still Open
1. packet-level empirical replay remains thin relative to the full `W016` inventory,
2. per-function contract docs are still batched rather than individually expanded,
3. no `function-phase-complete` claim is made yet for any member of this family.

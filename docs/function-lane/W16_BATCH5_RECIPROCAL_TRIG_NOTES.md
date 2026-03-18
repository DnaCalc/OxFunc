# W16 Batch 5 - Reciprocal Trig Family Notes

Status: `active`
Owner lane: `OxFunc`
Relationship: fifth executable family inside `W016`

## 1. Family Members
1. `CSC`
2. `CSCH`
3. `SEC`
4. `SECH`

## 2. Why This Family Follows Batch 4
1. all four functions remain deterministic, nonvolatile, host-independent, and values-only at the adapter seam,
2. all four fit the existing unary numeric scalar-or-array-elementwise lift path,
3. the family extends the unary Q-capable math surface without introducing new caller-context or host seams,
4. the only subtlety is singularity handling for reciprocal forms, which native Excel probes pin cleanly.

## 3. Empirical Findings
Native Excel COM probe on `2026-03-15` observed:
1. `CSC(1)` -> `1.1883951057781212`
2. `CSC(0)` -> `#DIV/0!`
3. `SEC(1)` -> `1.8508157176809255`
4. `SEC(PI()/2)` -> `16324552277619072`
5. `CSCH(1)` -> `0.8509181282393216`
6. `CSCH(0)` -> `#DIV/0!`
7. `SECH(1)` -> `0.6480542736638855`
8. `SECH(0)` -> `1`

Probe artifact:
1. `.tmp/w16-batch5-reciprocal-trig-probe.csv`

## 4. Current Local Artifact Shape
Runtime artifacts:
1. `crates/oxfunc_core/src/functions/csc.rs`
2. `crates/oxfunc_core/src/functions/csch.rs`
3. `crates/oxfunc_core/src/functions/sec.rs`
4. `crates/oxfunc_core/src/functions/sech.rs`

Formal artifacts:
1. `formal/lean/OxFunc/Functions/Csc.lean`
2. `formal/lean/OxFunc/Functions/Csch.lean`
3. `formal/lean/OxFunc/Functions/Sec.lean`
4. `formal/lean/OxFunc/Functions/Sech.lean`

## 5. What Is Still Open
1. packet-level empirical replay remains thin relative to the full `W016` inventory,
2. per-function contract docs are still batched rather than individually expanded,
3. no `function-phase-complete` claim is made yet for any member of this family.

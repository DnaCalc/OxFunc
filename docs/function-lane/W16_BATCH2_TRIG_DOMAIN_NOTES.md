# W16 Batch 2 - Unary/Trig Domain Family Notes

Status: `active`
Owner lane: `OxFunc`
Relationship: second executable family inside `W016`

## 1. Family Members
1. `ACOT`
2. `ACOTH`
3. `ASINH`
4. `ATAN2`
5. `ATANH`
6. `COT`
7. `COTH`

## 2. Why This Family Follows Batch 1
1. all seven functions remain deterministic, nonvolatile, host-independent, and values-only at the adapter seam,
2. six of the seven fit the existing unary numeric scalar-or-array-elementwise lift path,
3. `ATAN2` adds a simple `nums -> num` binary helper without introducing a new FEC/OxFml seam,
4. the family extends the math/trig tranche while staying inside already-admitted XLL `U`/`Q` export shapes.

## 3. Empirical Domain Findings
Native Excel COM probe on `2026-03-15` observed:
1. `ACOT(0)` -> `1.5707963267948966`
2. `ACOTH(1)` and `ACOTH(-1)` -> `#NUM!`
3. `ACOTH(2)` -> `0.5493061443340549`
4. `ATANH(1)` and `ATANH(-1)` -> `#NUM!`
5. `ATANH(0.5)` -> `0.5493061443340549`
6. `COT(0)` -> `#DIV/0!`
7. `COTH(0)` -> `#DIV/0!`
8. `ATAN2(0,0)` -> `#DIV/0!`
9. `ATAN2(1,0)` -> `0`
10. `ATAN2(0,1)` -> `1.5707963267948966`

Probe artifact:
1. `.tmp/w16-batch2-trig-probe.csv`

## 4. Current Local Artifact Shape
Runtime artifacts:
1. `crates/oxfunc_core/src/functions/acot.rs`
2. `crates/oxfunc_core/src/functions/acoth.rs`
3. `crates/oxfunc_core/src/functions/asinh.rs`
4. `crates/oxfunc_core/src/functions/atan2.rs`
5. `crates/oxfunc_core/src/functions/atanh.rs`
6. `crates/oxfunc_core/src/functions/cot.rs`
7. `crates/oxfunc_core/src/functions/coth.rs`

Formal artifacts:
1. `formal/lean/OxFunc/Functions/Acot.lean`
2. `formal/lean/OxFunc/Functions/Acoth.lean`
3. `formal/lean/OxFunc/Functions/Asinh.lean`
4. `formal/lean/OxFunc/Functions/Atan2.lean`
5. `formal/lean/OxFunc/Functions/Atanh.lean`
6. `formal/lean/OxFunc/Functions/Cot.lean`
7. `formal/lean/OxFunc/Functions/Coth.lean`

## 5. What Is Still Open
1. packet-level empirical replay remains thin relative to the full `W016` inventory,
2. per-function contract docs are still batched rather than individually expanded,
3. no `function-phase-complete` claim is made yet for any member of this family.

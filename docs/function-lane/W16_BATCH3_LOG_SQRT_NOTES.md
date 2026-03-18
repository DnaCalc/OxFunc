# W16 Batch 3 - Log and Square-Root Family Notes

Status: `active`
Owner lane: `OxFunc`
Relationship: third executable family inside `W016`

## 1. Family Members
1. `LN`
2. `LOG10`
3. `SQRT`
4. `SQRTPI`

## 2. Why This Family Follows Batch 2
1. all four functions remain deterministic, nonvolatile, host-independent, and values-only at the adapter seam,
2. all four fit the existing unary numeric scalar-or-array-elementwise lift path,
3. the family adds more Q-capable unary-number exports without introducing a new admission or caller-context seam,
4. the remaining work is only domain/error mapping, which native Excel probes pin cleanly.

## 3. Empirical Domain Findings
Native Excel COM probe on `2026-03-15` observed:
1. `LN(1)` -> `0`
2. `LN(0)` -> `#NUM!`
3. `LOG10(10)` -> `1`
4. `LOG10(0)` -> `#NUM!`
5. `SQRT(4)` -> `2`
6. `SQRT(-1)` -> `#NUM!`
7. `SQRTPI(1)` -> `1.7724538509055159`
8. `SQRTPI(-1)` -> `#NUM!`

Probe artifact:
1. `.tmp/w16-batch3-log-sqrt-probe.csv`

## 4. Current Local Artifact Shape
Runtime artifacts:
1. `crates/oxfunc_core/src/functions/ln_fn.rs`
2. `crates/oxfunc_core/src/functions/log10_fn.rs`
3. `crates/oxfunc_core/src/functions/sqrt_fn.rs`
4. `crates/oxfunc_core/src/functions/sqrtpi.rs`

Formal artifacts:
1. `formal/lean/OxFunc/Functions/Ln.lean`
2. `formal/lean/OxFunc/Functions/Log10.lean`
3. `formal/lean/OxFunc/Functions/Sqrt.lean`
4. `formal/lean/OxFunc/Functions/SqrtPi.lean`

## 5. What Is Still Open
1. packet-level empirical replay remains thin relative to the full `W016` inventory,
2. per-function contract docs are still batched rather than individually expanded,
3. no `function-phase-complete` claim is made yet for any member of this family.

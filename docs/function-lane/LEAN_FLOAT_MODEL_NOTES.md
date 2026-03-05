# Lean Float Model Notes (W2 Input)

Status: `active`
Lean toolchain: `leanprover/lean4:v4.28.0`

## 1. Purpose
Record what Lean provides for binary64 floating-point behavior, and how OxFunc should treat this in W2.

## 2. Source Snapshot
Primary local source:
1. `Init/Data/Float.lean` in Lean `v4.28.0`

Key facts from source:
1. `Float` is documented as IEEE-754 binary64.
2. arithmetic and classification operators are `opaque` extern calls (`add`, `sub`, `mul`, `div`, `isNaN`, `isFinite`, `isInf`, `ofBits`, `toBits`).
3. `Float` equality via `==` is IEEE-style (`NaN != NaN`, `0.0 == -0.0`).

Implication:
1. Lean `Float` is an executable runtime interface, not a kernel-reducible formal theory.

## 3. Local Probe Outcomes
Probe commands run under `lake env lean` show:
1. `Float.ofBits`/`Float.toBits` preserve ordinary finite values, `+0`, `-0`, and infinities.
2. Distinct NaN inputs round-trip to a canonical quiet NaN bit-pattern in this runtime path.
3. `1.0/0.0` and `-1.0/0.0` classify as infinite; `0.0/0.0` classifies as NaN.

Interpretation:
1. Lean runtime behavior is useful as an executable comparison baseline.
2. It must not be assumed identical to Excel worksheet-observable behavior without empirical comparison.

## 4. W2 Modeling Posture
1. Avoid introducing custom full UInt64 FP64 formal theory by default.
2. First establish empirical Lean-vs-Excel equivalence/divergence classes per scenario.
3. Maintain explicit divergence records in `FLOATING_POINT_LEAN_EXCEL_DEVIATION_LEDGER.csv`.
4. Introduce deeper custom FP theory only if required by unresolved high-impact divergences.

## 5. Honesty Rule for Claims
1. If Lean and Excel differ on observable behavior, document the divergence explicitly.
2. Do not claim parity where only one side has been measured.
3. Keep all claims version-scoped (Excel build/channel, Compatibility Version, Lean toolchain version).

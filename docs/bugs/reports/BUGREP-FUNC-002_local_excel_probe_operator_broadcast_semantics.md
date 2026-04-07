# BUGREP-FUNC-002: Local Excel probe found ordinary-operator broadcast semantics

## Intake
- **Report id**: `BUGREP-FUNC-002`
- **Filed**: 2026-04-07
- **Source channel**: local empirical probe
- **Reporter/source**: `OxFunc` local Excel comparison run during `W073`
- **Reported against ref**: `9e9c573a46d97e248a0373938fb53dcac916fac2`
- **Reported against kind**: commit
- **Reported against note**: Observed on the current working tree atop `HEAD` while probing broadcast-sensitive operator formulas against desktop Excel. No release tag was in play for this local comparison run.
- **Canonical bug id**: `BUG-FUNC-002`
- **Status**: triaged

## Observed Symptom
Desktop Excel broadcasts ordinary operator inputs across singleton dimensions and
pads non-broadcastable coordinates with `#N/A`, while the current OxFunc
arithmetic surface only handled scalar/array plus same-shape array/array and the
current compare/concat surface remained scalar-only.

## Reproduction
1. Assign these formulas through `Formula2` on the current Excel baseline:
   - `={1,2}+{1;2}`
   - `={"a","b"}&{"x";"y"}`
   - `={1,2}={1;2}`
   - `={1,2}+{1,2,3}`
2. Expected result:
   broadcast/outer-product spill grids, with `#N/A` only at coordinates neither
   operand can supply.
3. Actual local OxFunc result before this follow-up:
   - arithmetic rejected mismatched row/column shapes,
   - compare/concat rejected array inputs entirely.

## Initial Ownership Read
- **Initial classification**: shared seam gap
- **Reason**: OxFunc owns ordinary operator semantic truth and runtime behavior,
  while OxFml is the downstream seam consumer currently carrying temporary
  operator fallbacks that depend on this exact lane being described honestly.

## Links
1. `docs/bugs/streams/BUG-FUNC-001_binary_operator_array_lift_value_surface_gap.md`
2. `docs/bugs/streams/BUG-FUNC-002_ordinary_operator_broadcast_semantics_gap.md`
3. `docs/worksets/W073_OPERATOR_VALUE_SURFACE_AND_ARRAY_LIFT_EXPANSION.md`
4. `docs/worksets/W074_ORDINARY_OPERATOR_BROADCAST_RECONCILIATION.md`
5. `docs/function-lane/W45_EXECUTION_RECORD.md`
6. `tools/w45-probe/run-w45-wavea-operator-arithmetic-baseline.ps1`
7. `tools/w45-probe/run-w45-waveb-operator-compare-concat-baseline.ps1`

## Triage Notes
Local probe results showed:
1. arithmetic and compare/concat share the same broader broadcast rule,
2. row-by-column combinations produce 2-D outer-product spill grids,
3. extra coordinates beyond a non-singleton extent surface `#N/A`,
4. reference operators remain a separate structural family and were not the new
   defect proven by this probe.

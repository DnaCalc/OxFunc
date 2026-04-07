# W45 Wave B Runtime Requirements - Operator Compare/Concat Family

Workset: `W45`
Evidence ID: `W45-OP-CMP-WAVEB-20260320`

## 1. Goal
1. pin the admitted compare/concat operator slice against the current Excel baseline,
2. keep mixed-type ordering and blank-comparison behavior explicit rather than inferred from other families,
3. capture array broadcast spill grids and `#N/A` padding where the current baseline exposes them.

## 2. Required Artifacts
1. scenario manifest: `docs/function-lane/W45_WAVEB_OPERATOR_COMPARE_CONCAT_SCENARIO_MANIFEST_SEED.csv`
2. native result output: `.tmp/w45-waveb-operator-compare-concat-results.csv`
3. probe runner: `tools/w45-probe/run-w45-waveb-operator-compare-concat-baseline.ps1`
4. execution record: `docs/function-lane/W45_EXECUTION_RECORD.md`

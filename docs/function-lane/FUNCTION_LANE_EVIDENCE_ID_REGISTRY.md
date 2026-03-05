# Function Lane Evidence ID Registry (Provisional)

Status: `active`
Owner lane: `OxFunc`

Purpose:
1. keep local evidence IDs stable and traceable before Foundation promotion,
2. avoid ad-hoc ID drift across function-lane docs.

## Registry Rows

| evidence_id | scope | status | source_artifacts | notes |
| --- | --- | --- | --- | --- |
| `W1-FA-BL-20260305` | W1 `PI()` admission-boundary baseline (COM + file-ingress) | provisional | `docs/function-lane/FORMULA_ADMISSION_BEHAVIOR_NOTES.md`; `tools/formula-admission-probe/run-formula-admission-baseline.ps1`; `.tmp/formula-admission-results.csv` | Excel `16.0 (build 19725)`, channel `http://officecdn.microsoft.com/pr/492350f6-3a01-4f97-b9c0-c7c6ddf67d60`, compatibility `default|CalculationVersion=191029|CheckCompatibility=False|FileFormat=51`, locale `en-US`. |

## Rules
1. IDs are immutable once referenced from conformance or correlation rows.
2. IDs remain `provisional` until multi-build/channel + compatibility coverage is complete or Foundation accepts a promoted `EMP-*` ID.
3. If promoted, append a row mapping local ID -> Foundation ID rather than reusing/removing the local ID.

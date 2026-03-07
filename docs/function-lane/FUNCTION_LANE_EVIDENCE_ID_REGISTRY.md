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
| `W3-VU-BL-20260305` | W3 value-universe baseline taxonomy/spec + Lean/Rust scaffold closure | provisional | `docs/function-lane/VALUE_UNIVERSE_PRELIM_SPEC.md`; `docs/function-lane/VALUE_UNIVERSE_TAG_TABLE.csv`; `docs/function-lane/VALUE_UNIVERSE_RESEARCH_AND_OPEN_QUESTIONS.md`; `crates/oxfunc_core/src/value.rs`; `formal/lean/OxFunc/ValueUniverse.lean` | Baseline taxonomy run under W3 on 2026-03-05; Rust tests (`cargo test -p oxfunc_core`) and Lean build (`lake build`) passed. |
| `W7-STR-BL-20260305` | W7 string normalization/comparison/limit baseline (worksheet + interop + persistence lanes) | provisional | `docs/function-lane/STRING_BEHAVIOR_RESEARCH_NOTES.md`; `docs/function-lane/STRING_EXECUTION_RECORD.md`; `docs/function-lane/STRING_NORMALIZATION_AND_COMPARISON_POLICY_MAP.md`; `docs/function-lane/STRING_SCENARIO_MANIFEST_SEED.csv`; `tools/string-probe/run-string-excel-baseline.ps1`; `.tmp/string-results-excel.csv` | Excel `16.0 (build 19725)`, channel `http://officecdn.microsoft.com/pr/492350f6-3a01-4f97-b9c0-c7c6ddf67d60`, compatibility `default|CalculationVersion=191029|CheckCompatibility=False|FileFormat=51` plus CSV reopen `FileFormat=6`, locale `en-US`. |
| `W4-COERCE-BL-20260307` | W4 coercion/ref-resolution empirical baseline (scalar/array/aggregate/ref seam lanes) | provisional | `docs/function-lane/COERCION_SCENARIO_MANIFEST_SEED.csv`; `docs/function-lane/COERCION_PROBE_RUNTIME_REQUIREMENTS.md`; `docs/function-lane/COERCION_EXECUTION_RECORD.md`; `tools/coercion-probe/run-coercion-excel-baseline.ps1`; `.tmp/coercion-results-excel.csv`; `.tmp/coercion-analysis-report.csv` | Excel `16.0 (build 19725)`, channel `http://officecdn.microsoft.com/pr/492350f6-3a01-4f97-b9c0-c7c6ddf67d60`, locale `en-US`, run label `default`. Baseline captured `18` rows (`17` observed + `1` intentional admission failure) with `expectation_mismatched=0`; includes external-open-state row (`CO4-018`) showing closed `#REF!` vs open resolved value. |

## Rules
1. IDs are immutable once referenced from conformance or correlation rows.
2. IDs remain `provisional` until multi-build/channel + compatibility coverage is complete or Foundation accepts a promoted `EMP-*` ID.
3. If promoted, append a row mapping local ID -> Foundation ID rather than reusing/removing the local ID.

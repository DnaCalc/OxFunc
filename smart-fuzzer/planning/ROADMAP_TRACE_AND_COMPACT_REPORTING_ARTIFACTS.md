# Roadmap Trace And Compact Reporting Artifacts

Status: `planning_artifact_ready`

Owning bead: `oxf-1avj.6`

Owning workset:
`docs/worksets/W089_SMART_FUZZER_SWEEPING_INVOCATION_SPACE_EXPLORATION.md`

This note defines the compact reporting surface for W089. The core rule is that
ordinary passing cases are telemetry, not prose records.

## 1. Builder

Default command:

```powershell
powershell -ExecutionPolicy Bypass -File smart-fuzzer\tools\Build-SweepPlanningArtifacts.ps1
```

Output:

```text
smart-fuzzer/cache/roadmap-trace-template-v0.json
```

## 2. Required Run Artifacts

Later W089 execution should write:

1. `manifest.json`
2. `dimension_inventory.json`
3. `generator_matrix.json`
4. `local_budget.json`
5. `excel_candidate_budget.json`
6. `blocked_seam_map.json`
7. `coverage_rollup.json`
8. `roadmap_trace.json`
9. `roadmap_trace.md`
10. `telemetry.jsonl`
11. `failure_packets/`

## 3. Roadmap Trace Sections

The roadmap trace should summarize:

1. explored function families,
2. arity coverage,
3. value-kind coverage,
4. numeric/text bands,
5. array and reference shape coverage,
6. context and seam coverage,
7. blocked or deferred lanes,
8. known deviations observed separately from unexpected mismatches,
9. sparse areas to target next,
10. promoted failure packets.

## 4. Pass Telemetry

Passing cases should be represented by counters:

1. `generated_count`,
2. `local_evaluated_count`,
3. `excel_evaluated_count`,
4. `exact_typed_bit_match_count`,
5. dimension counters from the taxonomy,
6. selection reason counters.

Per-case pass rows may exist in JSONL for sampling and replay, but they should
not become durable narrative artifacts.

## 5. Current Status Axes

1. `execution_state`: `reporting_artifacts_ready`
2. `scope_completeness`: `scope_partial`
3. `target_completeness`: `target_partial`
4. `integration_completeness`: `partial`
5. `open_lanes`: execution approval, mismatch triage protocol

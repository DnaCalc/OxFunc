# Local Evaluator Dry-Run Budget

Status: `planning_artifact_ready`

Owning bead: `oxf-1avj.3`

Owning workset:
`docs/worksets/W089_SMART_FUZZER_SWEEPING_INVOCATION_SPACE_EXPLORATION.md`

This note defines the high-volume local-evaluation plan for W089. It does not
authorize a local sweep. It defines the budgets and artifacts that a later
execution bead should use.

## 1. Builder

Default command:

```powershell
powershell -ExecutionPolicy Bypass -File smart-fuzzer\tools\Build-SweepPlanningArtifacts.ps1
```

Output:

```text
smart-fuzzer/cache/local-dry-run-budget-v0.json
```

## 2. Purpose

The local dry run should use fast OxFunc/OxFml evaluation to discover:

1. local outcome classes,
2. error-code diversity,
3. array and reference shape outcomes,
4. generator invalid-case rates,
5. candidate rows worth sending to Excel.

The dry run is not a function-completion proof and should not produce
per-passing-case prose.

## 3. Budget Rules

Budgets come from the generator matrix:

1. every surface receives a mandatory basis budget,
2. numeric/text/array/reference/context axes add proportional local budget,
3. variadic rows get capped low/mid/high budget bands,
4. known PMT/PPMT/IPMT drift rows get extra local sampling as reference
   mismatch lanes,
5. blocked/provider/special-interface rows get sentinel budgets only unless a
   concrete fixture exists.

## 4. Planned Artifacts

A later local run should write:

1. `manifest.json`,
2. `local_cases.jsonl`,
3. `local_outcomes.jsonl`,
4. `coverage_rollup.json`,
5. `local_invalid_cases.jsonl`,
6. `excel_candidate_pool.jsonl`,
7. `telemetry.jsonl`.

All ordinary passing rows remain compact telemetry.

## 5. Local Outcome Classes

Local outcome classes:

1. `local_value_scalar`,
2. `local_value_array`,
3. `local_reference_like`,
4. `local_worksheet_error`,
5. `local_bind_reject`,
6. `local_seam_reject`,
7. `local_generator_invalid`,
8. `local_panic_or_harness_failure`,
9. `local_timeout`.

`local_panic_or_harness_failure` and `local_timeout` are harness or safety
signals, not Excel divergences.

## 6. Current Status Axes

1. `execution_state`: `local_budget_ready`
2. `scope_completeness`: `scope_partial`
3. `target_completeness`: `target_partial`
4. `integration_completeness`: `partial`
5. `open_lanes`: Excel candidate budget, blocked seam classification,
   execution approval, mismatch triage protocol

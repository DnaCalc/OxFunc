# Smart-Fuzzer Planning — Index

Status: `index`

This is the entry point for `smart-fuzzer/planning/`. The directory mixes
live authority docs with point-in-time planning gates and run notes that
have since been executed. This index marks which is which so a reader does
not mistake a historical plan for the current state.

For the current state of every surface, start with the derived view:
**`FUNCTION_STATUS_MAP.md`** (rebuild with `smart-fuzzer/tools/Build-FunctionStatusMap.ps1`).

## Current — authority and design

These are live and binding for ongoing work:

- [SMART_FUZZER_DESIGN.md](SMART_FUZZER_DESIGN.md) — design authority; goal, severity grading (CHARTER §4.1), pipeline.
- [EXCEL_RUNNER_PLUMBING_NOTE.md](EXCEL_RUNNER_PLUMBING_NOTE.md) — binding cell-ref plumbing rule and witness.
- [CASE_SCHEMA_V0.md](CASE_SCHEMA_V0.md) — invocation-case record schema.
- [RUN_ARTIFACT_CONTRACT.md](RUN_ARTIFACT_CONTRACT.md) — run output / rollup / failure-packet contract.
- [UNEXPECTED_MISMATCH_TRIAGE_AND_MINIMIZATION_PROTOCOL.md](UNEXPECTED_MISMATCH_TRIAGE_AND_MINIMIZATION_PROTOCOL.md) — how to triage and minimize a finding.
- [SPARK_LONG_RUN_SMART_FUZZER_GUIDE.md](SPARK_LONG_RUN_SMART_FUZZER_GUIDE.md) — W092 controlling run guide and cycle ledger.

## Current — derived state and live findings

- [FUNCTION_STATUS_MAP.md](FUNCTION_STATUS_MAP.md) — per-surface status across the 517 in-scope rows.
- [UNSWEPT_STRUCTURAL_SWEEP_FINDINGS_2026-05-28.md](UNSWEPT_STRUCTURAL_SWEEP_FINDINGS_2026-05-28.md) — first structural sweep of the unswept set; sources BUG-FUNC-028.

## W097 cell-ref re-sweep (complete; retained as evidence)

The re-measurement of known exactness streams under cell-ref plumbing. The
workset is complete; these are kept as the evidence trail.

- [KNOWN_MISMATCH_RESWEEP_PLAN.md](KNOWN_MISMATCH_RESWEEP_PLAN.md) — the plan.
- `W097-R-A` … `W097-R-GH` — per-tranche run records.

## Historical — W089 planning-gate artifacts

Point-in-time planning outputs from the W089 gate sequence. They were
consumed by execution (W089/W090/W092) and the live coverage picture is
now in `FUNCTION_STATUS_MAP.md`. Retained as audit trail; do not treat as
current plans.

- DIMENSION_INVENTORY_AND_COVERAGE_TAXONOMY.md
- GENERATOR_MATRIX_AND_TYPED_MUTATOR_PLAN.md
- LOCAL_EVALUATOR_DRY_RUN_BUDGET.md
- EXCEL_CANDIDATE_SELECTION_AND_BATCHING_BUDGET.md
- BLOCKED_DEFERRED_SEAM_CLASSIFICATION_MAP.md
- ROADMAP_TRACE_AND_COMPACT_REPORTING_ARTIFACTS.md
- FIRST_SWEEP_EXECUTION_GATE.md
- SWEEPING_INVOCATION_SPACE_RUN_PLAN.md
- AXIS_WITNESS_SWEEP_RUN_PLAN.md

## Historical — run notes

Point-in-time summaries of specific runs. The findings they describe live
in `BUG-FUNC-*` streams and `FUNCTION_STATUS_MAP.md`; the run artifacts
themselves are under `smart-fuzzer/runs/` (gitignored). Retained as audit
trail.

- COMPREHENSIVE_SMART_FUZZER_RUN_20260430.md
- EXPANDED_RUN_ROADMAP.md
- ARRAY_SUPPORT_SYSTEMATIC_SWEEP_PLAN.md
- ARRAY_SUPPORT_SUCCESSOR_SWEEP_20260430.md
- BROAD_SCALAR_EXPLORATION_2026-05-09.md

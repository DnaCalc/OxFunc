# Smart-Fuzzer Run Artifact Contract

Status: `contract_draft`

This contract defines the first stable shape for smart-fuzzer run artifacts. It
keeps ordinary passing cases compact while preserving enough state to reproduce,
deduplicate, and promote interesting failures through the normal OxFunc
surfaces.

Owning workset: `docs/worksets/W088_SMART_FUZZER_DIFFERENTIAL_EXPLORATION.md`
Owning bead: `oxf-u738.4`

## Goals

1. Make high-volume exploration cheap to store and scan.
2. Keep per-case prose out of ordinary passing rows.
3. Preserve deterministic replay inputs for generated cases.
4. Keep Excel observations tied to version, channel, compatibility, runner, and
   git metadata.
5. Promote only mismatches, unstable cases, harness blockers, and curated
   representative samples into heavier artifacts.

## Run Layout

A generated run should write under `smart-fuzzer/runs/<run_id>/`.

Required files:

```text
manifest.json
telemetry.jsonl
rollup.json
```

Conditional files and directories:

```text
representative_samples.jsonl
cases/
outcomes/
comparisons/
failure_packets/
minimized/
logs/
```

The run directory is ignored by git by default. Promotion candidates must be
copied or converted into tracked `smart-fuzzer/corpus/`, `docs/bugs/`, or
function-lane evidence surfaces as appropriate.

## Excel Comparator Plumbing Field

Every comparator run that compares Excel `Value2` outcomes against OxFunc
under a bit-exact policy must record how numeric inputs were passed to
Excel. The shape lives in `manifest.json` under
`environment.excel_input_plumbing`:

```json
"excel_input_plumbing": "cell_value2"   // exact f64 round-trip via Range.Value2
```

or:

```json
"excel_input_plumbing": "formula_literal_text"  // legacy path; encoding-drift class applies
```

Rows produced by the legacy path must NOT be used to close any
bit-exact `BUG-FUNC-*` exactness stream. See
`smart-fuzzer/planning/EXCEL_RUNNER_PLUMBING_NOTE.md` for the rule and
the runner inventory.

## Manifest

`manifest.json` is one JSON document per run.

Required fields:

```json
{
  "schema_version": "oxfunc.smart_fuzzer.run_manifest.v0",
  "run_id": "20260428T120000Z-pmt-ppmt-pilot",
  "created_utc": "2026-04-28T12:00:00Z",
  "git_revision": "",
  "worktree_dirty": true,
  "runner": {
    "runner_id": "smart-fuzzer-runner",
    "runner_version": "0.0.0",
    "command_line": []
  },
  "scope": {
    "function_ids": ["FUNC.PMT", "FUNC.PPMT"],
    "generator_ids": ["financial_payment_residuals.v0"],
    "excel_budget_cases": 0,
    "local_budget_cases": 0
  },
  "environment": {
    "host_os": "",
    "rust_profile": "",
    "excel_available": false,
    "excel_version": null,
    "excel_build": null,
    "excel_channel": null,
    "workbook_compatibility": "default",
    "locale_profile": "en-US",
    "excel_input_plumbing": "cell_value2"
  },
  "inputs": {
    "metadata_snapshot_refs": [],
    "source_index_digest": null,
    "seed": 0
  }
}
```

## Telemetry

`telemetry.jsonl` contains one compact row per evaluated case per comparison
surface. It is the default retained artifact for passing cases.

The row shape is defined in `CASE_SCHEMA_V0.md`. Rows should contain digests,
bucket labels, and outcome classes, not expanded value payloads unless the value
itself is already small and useful for deduplication.

Required retention properties:

1. A passing telemetry row must be sufficient to count coverage buckets.
2. A passing telemetry row must be sufficient to identify the generator and
   seed that created the case.
3. A passing telemetry row should not carry prose, screenshots, workbook files,
   or copied source snippets.
4. A passing telemetry row may be discarded after its aggregate counters have
   been folded into `rollup.json`, unless it is sampled.

## Rollup

`rollup.json` summarizes the run without requiring every telemetry row to stay
available forever.

Required fields:

```json
{
  "schema_version": "oxfunc.smart_fuzzer.rollup.v0",
  "run_id": "20260428T120000Z-pmt-ppmt-pilot",
  "case_counts": {
    "generated": 0,
    "local_evaluated": 0,
    "excel_evaluated": 0,
    "matches": 0,
    "mismatches": 0,
    "unstable": 0,
    "blocked": 0,
    "invalid_generator_output": 0
  },
  "by_function": {},
  "by_generator": {},
  "by_coverage_bucket": {},
  "throughput": {
    "local_cases_per_second": null,
    "excel_cases_per_second": null,
    "excel_batch_size": null
  },
  "promotion_candidates": []
}
```

Rollups may aggregate multiple comparisons for the same generated invocation.
They must keep seam or harness blockers separate from function-semantic
mismatches.

## Representative Samples

`representative_samples.jsonl` is optional. It stores a small bounded sample of
passing rows for coverage explanation and regression seed selection.

Default cap:

```text
max(100, min(5000, floor(total_matching_cases * 0.001)))
```

Sampling should prefer diversity across:

1. function id,
2. generator id,
3. arity shape,
4. value-kind signature,
5. coverage bucket,
6. outcome class.

Representative samples are not conformance evidence by themselves. They are
inputs for future deterministic replay or regression promotion.

## Heavy Artifacts

Create expanded case, outcome, comparison, and failure-packet files only for:

1. local-vs-Excel mismatches,
2. local surface disagreement before Excel evaluation,
3. nondeterministic or unstable outcomes,
4. blocked harness or seam observations,
5. reduced reproducers,
6. bounded representative samples explicitly selected for replay.

Heavy artifacts should use these paths:

```text
cases/<case_id>.json
outcomes/<case_id>.<surface>.json
comparisons/<case_id>.<lhs_surface>_vs_<rhs_surface>.json
failure_packets/<case_id>.json
minimized/<case_id>.json
```

Workbook files, screenshots, and raw Excel automation logs should be placed in
`logs/` and referenced from failure packets only when they add replay or
diagnostic value.

## Promotion Rules

Promotion out of `smart-fuzzer/runs/` requires:

1. Excel version, channel, build, locale, and workbook compatibility metadata.
2. A reduced or intentionally unreduced replay case with the reason recorded.
3. Classification as one of:
   - `function_semantic`,
   - `adapter_or_seam`,
   - `excel_harness`,
   - `generator_invalid`,
   - `needs_triage`.
4. A fresh comparison against the current reference runner.
5. A target destination:
   - `docs/bugs/` for function-semantic bug intake,
   - `smart-fuzzer/corpus/` for candidate replay corpus,
   - `docs/function-lane/` for curated function evidence,
   - handoff packet when OxFml-facing behavior is implicated.

For W088 pilot work, PMT and PPMT mismatches are expected first-class triage
targets. The former POWER stale-claim check was freshly confirmed and closed
under W078 on 2026-04-29; future POWER mismatches should be recorded as new
signals.

## Retention Policy

Default local retention:

1. Keep `manifest.json`, `rollup.json`, and promoted heavy artifacts.
2. Keep `telemetry.jsonl` while the run is actively guiding exploration.
3. Compress or delete ordinary passing telemetry after rollup aggregation when
   disk pressure or run count makes that useful.
4. Keep failure packets and minimized reproducers until they are promoted or
   explicitly rejected with a recorded reason.
5. Do not commit raw run directories unless a workset explicitly narrows and
   approves the artifact set.

## Validation Checklist

Before runner code writes this contract, validate:

1. JSON schema versions are present on every artifact.
2. `run_id` is stable across all artifacts in a run.
3. `case_id` is stable across telemetry, outcomes, comparisons, and failure
   packets.
4. Harness blockers do not increment function-semantic mismatch counts.
5. Ordinary passing rows can be aggregated without reading heavy artifacts.
6. A failure packet can be replayed or explains why replay is currently blocked.

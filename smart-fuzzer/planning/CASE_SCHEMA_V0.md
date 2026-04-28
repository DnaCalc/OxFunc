# Candidate Case Schema V0

Status: `schema_draft`

This is a planning schema for generated smart-fuzzer cases. It is not yet a
stable replay or evidence schema.

The schema is intentionally split between compact high-volume telemetry and
full failure packets. Most passing cases should not carry prose or expanded
per-case documentation.

## Case Record

```json
{
  "schema_version": "oxfunc.smart_fuzzer.case.v0",
  "case_id": "SFZ-00000001",
  "parent_case_ids": [],
  "generator": {
    "generator_id": "scalar_numeric_edges.v0",
    "seed": 123456789,
    "strategy": "edge_band_mutation",
    "source_seed_ref": "docs/function-lane/W45_WAVEA_OPERATOR_ARITHMETIC_SCENARIO_MANIFEST_SEED.csv#..."
  },
  "target": {
    "function_id": "FUNC.PMT",
    "canonical_surface_name": "PMT",
    "entry_kind": "built_in_function",
    "arity_min": 3,
    "arity_max": 5,
    "metadata_refs": [
      "docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv#FUNC.PMT"
    ]
  },
  "invocation": {
    "formula_text": "=PMT(0.05/12,360,200000)",
    "caller_locus": {
      "sheet_id": "Sheet1",
      "row": 1,
      "col": 1
    },
    "arguments": [
      {
        "position": 1,
        "surface_kind": "expression_scalar",
        "value_kind": "number",
        "payload": "0.05/12"
      },
      {
        "position": 2,
        "surface_kind": "literal_scalar",
        "value_kind": "number",
        "payload": "360"
      },
      {
        "position": 3,
        "surface_kind": "literal_scalar",
        "value_kind": "number",
        "payload": "200000"
      }
    ],
    "cell_fixture": []
  },
  "context": {
    "locale_profile": "en-US",
    "workbook_compatibility": "default",
    "calculation_mode": "automatic_controlled",
    "host_capabilities": [],
    "now_serial": null,
    "random_seed_value": null
  },
  "surfaces_requested": [
    "oxfunc_direct",
    "oxfml_adapter",
    "excel_formula"
  ],
  "comparison_policy": {
    "policy_id": "worksheet_error_exact.v0",
    "numeric_policy": "not_numeric",
    "array_policy": "shape_and_cellwise",
    "display_policy": "capture_only"
  }
}
```

## Compact Telemetry Row

Ordinary passing cases can be retained as JSONL or columnar rows shaped like:

```json
{
  "schema_version": "oxfunc.smart_fuzzer.telemetry.v0",
  "case_id": "SFZ-00000001",
  "run_id": "local-run-id",
  "function_id": "FUNC.PMT",
  "generator_id": "financial_payment_residuals.v0",
  "seed": 123456789,
  "invocation_digest": "sha256:...",
  "formula_text": "=PMT(0.05/12,360,200000)",
  "coverage_buckets": [
    "arity:range3_5",
    "args:number,number,number",
    "domain:ordinary_payment",
    "outcome:number"
  ],
  "local_outcome_digest": "sha256:...",
  "excel_outcome_digest": "sha256:...",
  "comparison_result": "match"
}
```

This row is not a promotion artifact. It exists to guide future exploration,
deduplicate reruns, and support rollup statistics.

## Outcome Record

```json
{
  "schema_version": "oxfunc.smart_fuzzer.outcome.v0",
  "case_id": "SFZ-00000001",
  "run_id": "local-run-id",
  "surface": "excel_formula",
  "execution_status": "observed",
  "semantic_status": "worksheet_error",
  "value_payload": {
    "number_value2": "-1073.643246..."
  },
  "shape": {
    "kind": "scalar",
    "rows": 1,
    "cols": 1
  },
  "display_payload": {
    "text": "($1,073.64)",
    "value2": "-1073.643246...",
    "formula2": "=PMT(0.05/12,360,200000)"
  },
  "environment": {
    "excel_version": "16.0",
    "excel_build": "",
    "excel_channel": "",
    "workbook_compatibility": "default",
    "git_revision": "",
    "runner_version": "smart-fuzzer-excel-runner/0.0"
  }
}
```

## Comparison Record

```json
{
  "schema_version": "oxfunc.smart_fuzzer.comparison.v0",
  "case_id": "SFZ-00000001",
  "run_id": "local-run-id",
  "lhs_surface": "oxfunc_direct",
  "rhs_surface": "excel_formula",
  "comparison_result": "mismatch",
  "mismatch_kind": "error_code_mismatch",
  "scope_classification": "function_semantic",
  "minimization_state": "unminimized",
  "promotion_state": "candidate_only"
}
```

## Failure Packet Delta

Only mismatches, unstable rows, blocked harness findings, and reduced
reproducers need the full expanded packet:

```json
{
  "schema_version": "oxfunc.smart_fuzzer.failure_packet.v0",
  "case_id": "SFZ-00000001",
  "summary": "PPMT pilot candidate returned a different numeric value after fresh Excel comparison",
  "case_record_ref": "cases/SFZ-00000001.json",
  "outcome_refs": [
    "outcomes/SFZ-00000001.oxfunc_direct.json",
    "outcomes/SFZ-00000001.excel_formula.json"
  ],
  "comparison_ref": "comparisons/SFZ-00000001.json",
  "reduction_lineage": [],
  "owner_classification": "OxFunc-owned bug",
  "scope_classification": "function_semantic",
  "promotion_state": "candidate_only"
}
```

## Required Invariants

1. A formula string must be generated from structured invocation fields.
2. Omitted, missing, blank, empty text, and worksheet errors must remain
   distinct.
3. Array shape and element order must be explicit.
4. Reference-like arguments must state `ReferenceKind` and target text.
5. A blocked seam or harness result must not be recorded as a function mismatch.
6. Every Excel outcome must carry version/channel/compat metadata before it can
   be considered for promotion.

# W15 Replay Diff and Explain Shapes V1

Status: `provisional`
Owner lane: `OxFunc`
Workset relation: `W018`
Worked packet: `W15`

## 1. Purpose
Define the expected diff and explain object shapes for the first future live replay-adapter run over the `W15` worked packet.

This note makes the output target explicit for:
1. typed diff records,
2. explain records,
3. mismatch, severity, and predicate vocabulary binding,
4. seam-limitation-aware classification.

It is a target-shape note only.
It is not evidence that these objects have already been emitted.

## 2. Foundation Vocabulary Binding
Pinned shared ids for this worked example:
1. mismatch kinds:
   - `mm.run.presence`
   - `mm.scenario.presence`
   - `mm.result.state`
   - `mm.view.value`
   - `mm.evidence.binding`
   - `mm.sidecar.payload`
2. severity classes:
   - `sev.semantic`
   - `sev.coverage`
   - `sev.instrumentation`
   - `sev.informational`
3. predicate kinds:
   - `pred.diff.mismatch_present`
   - `pred.invariant.failed`
   - `pred.evidence.claim_failed`

Local interpretation rule:
1. the registry ids coordinate cross-lane replay tooling,
2. OxFunc still owns the meaning of whether a given W15 difference is semantic, seam-limited, or coverage-only.

## 3. Diff Object Target
The first live `W15` adapter run should be able to emit a `ReplayDiff`-shaped record like:

```json
{
  "diff_id": "w15.default_vs_compat.row.CELL-WIDTH-001",
  "left_ref": "bundle://oxfunc.w15.packet.default_and_compat.v1/runs/w15.default",
  "right_ref": "bundle://oxfunc.w15.packet.default_and_compat.v1/runs/w15.compat_template",
  "comparison_scope": {
    "packet_id": "W15",
    "row_id": "CELL-WIDTH-001",
    "view_id": "manifest_row_result_view"
  },
  "mismatch_kind": "mm.view.value",
  "mismatch_path": "views/manifest_row_result_view/CELL-WIDTH-001/observed_value_text",
  "severity": "sev.semantic",
  "explanation_hint": "why_row_observed",
  "supporting_refs": [
    "evidence:W15-CELL-HOST-PRE-20260315",
    "artifact:.tmp/w15-cell-host-pre-results.csv",
    "artifact:.tmp/w15-cell-host-pre-results-compat.csv"
  ]
}
```

Required fields for emitted `W15` diffs:
1. `diff_id`
2. `left_ref`
3. `right_ref`
4. `comparison_scope`
5. `mismatch_kind`
6. `mismatch_path`
7. `severity`
8. `explanation_hint`
9. `supporting_refs`

## 4. Explain Object Target
The first live `W15` adapter run should be able to emit a `ReplayExplainRecord`-shaped record like:

```json
{
  "query_id": "explain.w15.bridge_parity.CELL-FILENAME-001",
  "query_kind": "why_bridge_parity",
  "scope_ref": {
    "packet_id": "W15",
    "row_id": "CELL-FILENAME-001",
    "run_id": "w15.default"
  },
  "supporting_refs": [
    "evidence:W15-XLL-BRIDGE-20260315",
    "artifact:.tmp/w15-xll-bridge-results.csv",
    "artifact:docs/function-lane/W15_EXECUTION_RECORD.md"
  ],
  "explanation_text": "The bridge row is parity-clean in the current baseline; provider-backed ox_CELL output matches native CELL for this explicit-reference lane.",
  "confidence_class": "provisional"
}
```

Required fields for emitted `W15` explain records:
1. `query_id`
2. `query_kind`
3. `scope_ref`
4. `supporting_refs`
5. `explanation_text`
6. `confidence_class`

## 5. Worked Classification Cases
### 5.1 Semantic mismatch case
Use when:
1. a replayed `W15` row changes observed value truth,
2. a compatibility descriptor is lost and causes a false row comparison,
3. evidence ids are rebound to the wrong row outcomes.

Expected shape:
1. `mismatch_kind`:
   - usually `mm.view.value` or `mm.result.state`
2. `severity`:
   - `sev.semantic`
3. `explanation_hint`:
   - `why_row_observed`

### 5.2 Projection-gap case
Use when:
1. the adapter failed to carry a source field,
2. a sidecar ref is absent from the bundle,
3. a declared view cannot cite a required source artifact.

Expected shape:
1. `mismatch_kind`:
   - usually `mm.evidence.binding` or `mm.sidecar.payload`
2. `severity`:
   - `sev.coverage` or `sev.instrumentation`
3. `explanation_hint`:
   - `why_projection_gap`

### 5.3 Seam-limited case
Use when:
1. a bridge-facing difference is qualified by the XLL seam-limit note,
2. the local semantic target remains intact,
3. the mismatch must remain visible without being reclassified as semantic truth drift.

Expected shape:
1. `mismatch_kind`:
   - usually `mm.view.value` or `mm.result.state`
2. `severity`:
   - usually `sev.instrumentation`
3. supporting refs must include:
   - `docs/function-lane/XLL_VERIFICATION_SEAM_LIMITATIONS.md`
4. explanation query:
   - `why_seam_limited`

## 6. Minimum W15 Explain Query Mapping
The first live run should support:
1. `why_row_observed`
   - cite row id, run id, result sidecar ref, and evidence id
2. `why_host_query`
   - cite the host-query seam note and relevant W15 evidence id
3. `why_bridge_parity`
   - cite the bridge sidecar and the relevant W15 bridge evidence id
4. `why_seam_limited`
   - cite the XLL limitation note plus the affected row and evidence id
5. `why_projection_gap`
   - cite the missing field or source path and classify it explicitly as adapter loss, not function semantic drift

## 7. Negative Example Shapes
### 7.1 Projection-gap explain

```json
{
  "query_id": "explain.w15.projection_gap.missing_sidecar_ref",
  "query_kind": "why_projection_gap",
  "scope_ref": {
    "packet_id": "W15",
    "view_id": "evidence_binding_view"
  },
  "supporting_refs": [
    "evidence:W15-INFO-PRE-20260315",
    "artifact:docs/function-lane/W15_REPLAY_ADAPTER_CONFORMANCE_CHECKLIST_V1.md"
  ],
  "explanation_text": "The bundle omitted one required result-sidecar reference. This is an adapter projection gap, not a semantic mismatch in the W15 packet itself.",
  "confidence_class": "provisional"
}
```

### 7.2 Seam-limited explain

```json
{
  "query_id": "explain.w15.seam_limited.bridge_error_shape",
  "query_kind": "why_seam_limited",
  "scope_ref": {
    "packet_id": "W15",
    "row_id": "XLL-BRIDGE-ERR-001",
    "run_id": "w15.default"
  },
  "supporting_refs": [
    "evidence:W15-XLL-BRIDGE-20260315",
    "artifact:docs/function-lane/XLL_VERIFICATION_SEAM_LIMITATIONS.md"
  ],
  "explanation_text": "The observed bridge difference is classified under the XLL verification seam limitations and does not by itself establish a core semantic mismatch for CELL/INFO.",
  "confidence_class": "provisional"
}
```

## 8. Acceptance Rule For Future Use
The first live adapter run over `W15` should be checked against this note and [W15_REPLAY_ADAPTER_CONFORMANCE_CHECKLIST_V1.md](/C:/Work/DnaCalc/OxFunc/docs/function-lane/W15_REPLAY_ADAPTER_CONFORMANCE_CHECKLIST_V1.md) together.

Pass rule:
1. emitted diffs and explains are structurally complete,
2. they use pinned registry ids,
3. they preserve evidence ids and sidecar refs,
4. they keep seam-limited cases visible without misclassifying them as semantic gaps.

## 9. Non-Claim Rule
This note still does not justify:
1. a `cap.C0` through `cap.C3` claim by itself,
2. any `cap.C4.distill_valid` claim,
3. any `cap.C5.pack_valid` claim.

# W15 Replay Adapter Conformance Checklist V1

Status: `provisional`
Owner lane: `OxFunc`
Workset relation: `W018`
Worked packet: `W15`

## 1. Purpose
Define the first explicit conformance checklist for a future live OxFunc packet-adapter run against the `W15` worked packet.

This checklist defines:
1. which fields must be present in the emitted bundle,
2. what counts as `cap.C0.ingest_valid` through `cap.C3.explain_valid` for this worked packet,
3. which failures count as projection gaps versus semantic gaps versus seam limitations.

This checklist is not itself a proving artifact.
It is the acceptance target for the first future adapter emission and replay run.

## 2. Scope
In scope:
1. `W15_INFO_PRE_SCENARIO_MANIFEST_SEED.csv`
2. `W15_CELL_HOST_PRE_SCENARIO_MANIFEST_SEED.csv`
3. `W15_XLL_BRIDGE_SCENARIO_MANIFEST_SEED.csv`
4. their paired result CSV sidecars,
5. the packet summary/evidence/limitation anchors already named in [W15_REPLAY_BUNDLE_SKELETON_V1.md](/C:/Work/DnaCalc/OxFunc/docs/function-lane/W15_REPLAY_BUNDLE_SKELETON_V1.md).

Out of scope:
1. witness distillation,
2. pack promotion,
3. broad multi-packet comparison beyond the `W15` worked example.

## 3. Required Source Preservation
The future emitted bundle must preserve:
1. packet/workset identity: `W15`
2. manifest identity for all three source manifests
3. run identity for both workbook descriptors:
   - `default`
   - `compat_template`
4. evidence ids:
   - `W15-INFO-PRE-20260315`
   - `W15-CELL-HOST-PRE-20260315`
   - `W15-XLL-BRIDGE-20260315`
5. XLL limitation reference path:
   - `docs/function-lane/XLL_VERIFICATION_SEAM_LIMITATIONS.md`
6. source sidecar refs for all six result CSV files
7. compatibility descriptor and run label per replayed row
8. Excel/environment metadata when present in the source packet.

## 4. Bundle-Field Presence Checklist
The emitted bundle must contain:

### 4.1 Top-level bundle manifest
1. `bundle_schema_id`
2. `bundle_schema_version`
3. `bundle_id`
4. `source_lanes`
5. `created_by`
6. `normalizer_version`
7. `artifact_layout_version`
8. `source_inventory_ref`

### 4.2 Run manifests
For both `w15.default` and `w15.compat_template`:
1. `run_id`
2. `lane_id`
3. `run_kind`
4. `profile_id`
5. `profile_version`
6. `config_fingerprint_ref`
7. `selection_ref`
8. `result_state_counts`
9. `source_artifact_roots`

### 4.3 Scenario manifests
For `W15.INFO.packet`, `W15.CELL.packet`, and `W15.XLL.packet`:
1. `scenario_id`
2. `scenario_kind`
3. `description`
4. `tags`
5. `pack_tags`
6. `generator_ref`
7. `evidence_ids`

### 4.4 Views
The emitted bundle must contain or materialize:
1. `manifest_row_result_view`
2. `run_summary_view`
3. `analysis_summary_view`
4. `evidence_binding_view`
5. `limitation_view`

## 5. Capability-Level Acceptance Criteria
### 5.1 `cap.C0.ingest_valid`
Pass only if:
1. the adapter ingests all three source manifests and all six result sidecars without silent loss,
2. the emitted bundle validates against the declared bundle skeleton fields,
3. any missing optional source field is surfaced explicitly as a projection gap, not silently dropped.

Fail as projection gap if:
1. a source path cannot be represented,
2. a source field exists locally but is omitted from the bundle without an explicit loss marker,
3. a local source schema id is ambiguous or missing.

### 5.2 `cap.C1.replay_valid`
Pass only if:
1. the bundle can replay the supported `W15` packet rows deterministically,
2. both run labels remain distinct,
3. both workbook descriptors remain distinct,
4. supported versus unsupported replay states are surfaced explicitly.

Fail as replay-invalid if:
1. row ids cannot be reconstructed,
2. deterministic rerun changes run identity or row identity,
3. supported rows are silently downgraded to unsupported.

### 5.3 `cap.C2.diff_valid`
Pass only if:
1. the adapter can compare the `default` and `compat_template` packet runs without losing row identity,
2. typed mismatch classes are emitted using pinned registry vocabulary,
3. seam-limited XLL bridge differences can be distinguished from semantic differences.

Fail as diff-invalid if:
1. mismatch classes are ad hoc or unpinned,
2. row-level diff scope is lost,
3. XLL seam limitations are forced into semantic mismatch classes by default.

### 5.4 `cap.C3.explain_valid`
Pass only if:
1. explain can cite row ids, evidence ids, and source refs,
2. explain can answer the minimum worked-example query set from the W15 bundle skeleton,
3. explain can expose known gaps explicitly.

Fail as explain-invalid if:
1. explanation text cannot cite source refs,
2. evidence ids are omitted,
3. known explain gaps are silently hidden.

## 6. Classification Rules
### 6.1 Projection gap
Use when the adapter cannot faithfully carry a source field or artifact into the normalized bundle.

Examples:
1. a local source schema field has no emitted normalized carrier,
2. a source sidecar ref is missing from bundle output,
3. an optional local field is intentionally unsupported but correctly declared as unsupported.

Recommended severity:
1. usually `sev.coverage` or `sev.instrumentation`, not `sev.semantic`.

### 6.2 Semantic gap
Use only when the replayed or compared result changes OxFunc semantic truth for the worked packet.

Examples:
1. a row result is changed or misbound,
2. a compatibility descriptor is lost and causes a false semantic comparison,
3. evidence ids are rebound to the wrong row outcomes.

Recommended severity:
1. `sev.semantic`.

### 6.3 Seam limitation
Use when the packet is semantically clean locally, but a verification surface such as XLL is known to be limited.

Examples:
1. a bridge mismatch that the W15 evidence and seam-limit note classify as an XLL limitation,
2. a host-bridge artifact omission that does not alter the core semantic target.

Rule:
1. seam limitations must remain visible in `limitation_view`,
2. they must not be silently collapsed into semantic success or semantic failure.

## 7. Minimum Explain Query Set
The future adapter run must answer these queries for `W15`:
1. `why_row_observed`
2. `why_host_query`
3. `why_bridge_parity`
4. `why_seam_limited`

Minimum answer content:
1. row or packet id,
2. run id and compatibility descriptor where applicable,
3. supporting evidence id refs,
4. source sidecar refs,
5. limitation ref if classification depends on seam limits.

## 8. Negative Tests Required For First Live Run
At minimum, the first live adapter run should include:
1. one projection-gap test
   - omit or simulate loss of one non-required field and verify the adapter surfaces a projection-gap state explicitly
2. one seam-limited diff test
   - compare a bridge-facing row and ensure the explanation cites the XLL limitation path rather than defaulting to semantic failure
3. one explain-gap test
   - request an unsupported explain surface and verify the gap is surfaced explicitly

## 9. Non-Claim Rule
Even if a future live run satisfies this checklist:
1. that would only justify evidence toward `cap.C0` through `cap.C3`,
2. it would still not justify `cap.C4.distill_valid`,
3. and it would not justify `cap.C5.pack_valid`.

## 10. Next Use
This checklist should be used immediately after the first future local packet-adapter run for `W15` to decide:
1. whether the emitted bundle is good enough to count as the first real OxFunc replay-adapter conformance artifact,
2. which remaining failures are adapter projection gaps,
3. and whether the repo can honestly move from doctrinal baseline to exercised replay support.

Companion output-shape target:
1. [W15_REPLAY_DIFF_EXPLAIN_SHAPES_V1.md](/C:/Work/DnaCalc/OxFunc/docs/function-lane/W15_REPLAY_DIFF_EXPLAIN_SHAPES_V1.md) defines the expected diff and explain object families for the same worked packet.

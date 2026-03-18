# W15 Replay Bundle Skeleton V1

Status: `provisional`
Owner lane: `OxFunc`
Workset: `W15`
Replay adapter relation: first concrete normalized-bundle target for `W018`

## 1. Purpose
Define the expected normalized Replay bundle skeleton for the `W15` `CELL` / `INFO` packet.

This is:
1. a target shape for the future OxFunc packet-adapter run,
2. a concrete worked example binding local OxFunc packet conventions to Replay appliance concepts,
3. not a claim that the bundle has already been emitted by a live adapter.

## 2. Source Packet Inputs
Source packet elements:
1. manifests:
   - `docs/function-lane/W15_INFO_PRE_SCENARIO_MANIFEST_SEED.csv`
   - `docs/function-lane/W15_CELL_HOST_PRE_SCENARIO_MANIFEST_SEED.csv`
   - `docs/function-lane/W15_XLL_BRIDGE_SCENARIO_MANIFEST_SEED.csv`
2. result sidecars:
   - `.tmp/w15-info-pre-results.csv`
   - `.tmp/w15-info-pre-results-compat.csv`
   - `.tmp/w15-cell-host-pre-results.csv`
   - `.tmp/w15-cell-host-pre-results-compat.csv`
   - `.tmp/w15-xll-bridge-results.csv`
   - `.tmp/w15-xll-bridge-results-compat.csv`
3. summary/evidence/limitation anchors:
   - `docs/function-lane/W15_EXECUTION_RECORD.md`
   - `docs/function-lane/FUNCTION_LANE_EVIDENCE_ID_REGISTRY.md`
   - `docs/function-lane/XLL_VERIFICATION_SEAM_LIMITATIONS.md`
   - `docs/worksets/W015_CELL_AND_INFO_HOST_QUERY_FUNCTIONS.md`

## 3. Normalized Bundle Skeleton
Illustrative shape:

```json
{
  "bundle_manifest": {
    "bundle_schema_id": "dna-replay-bundle/v1",
    "bundle_schema_version": "1",
    "bundle_id": "oxfunc.w15.packet.default_and_compat.v1",
    "source_lanes": ["oxfunc"],
    "created_by": "oxfunc.replay.packet_adapter",
    "normalizer_version": "1.0.0-draft",
    "artifact_layout_version": "1",
    "source_inventory_ref": "indexes/source_inventory.csv"
  },
  "run_manifests": [
    {
      "run_id": "w15.default",
      "lane_id": "oxfunc",
      "run_kind": "empirical_packet",
      "profile_id": "w15.host_query.packet",
      "profile_version": "1",
      "config_fingerprint_ref": "sidecars/config/w15.default.json",
      "selection_ref": "docs/function-lane/W15_INFO_PRE_SCENARIO_MANIFEST_SEED.csv",
      "result_state_counts": "indexes/result_state_counts.default.json",
      "source_artifact_roots": [
        "docs/function-lane/W15_INFO_PRE_SCENARIO_MANIFEST_SEED.csv",
        "docs/function-lane/W15_CELL_HOST_PRE_SCENARIO_MANIFEST_SEED.csv",
        "docs/function-lane/W15_XLL_BRIDGE_SCENARIO_MANIFEST_SEED.csv",
        ".tmp/w15-info-pre-results.csv",
        ".tmp/w15-cell-host-pre-results.csv",
        ".tmp/w15-xll-bridge-results.csv"
      ]
    },
    {
      "run_id": "w15.compat_template",
      "lane_id": "oxfunc",
      "run_kind": "empirical_packet",
      "profile_id": "w15.host_query.packet",
      "profile_version": "1",
      "config_fingerprint_ref": "sidecars/config/w15.compat_template.json",
      "selection_ref": "docs/function-lane/W15_INFO_PRE_SCENARIO_MANIFEST_SEED.csv",
      "result_state_counts": "indexes/result_state_counts.compat_template.json",
      "source_artifact_roots": [
        "docs/function-lane/W15_INFO_PRE_SCENARIO_MANIFEST_SEED.csv",
        "docs/function-lane/W15_CELL_HOST_PRE_SCENARIO_MANIFEST_SEED.csv",
        "docs/function-lane/W15_XLL_BRIDGE_SCENARIO_MANIFEST_SEED.csv",
        ".tmp/w15-info-pre-results-compat.csv",
        ".tmp/w15-cell-host-pre-results-compat.csv",
        ".tmp/w15-xll-bridge-results-compat.csv"
      ]
    }
  ],
  "scenario_manifests": [
    {
      "scenario_id": "W15.INFO.packet",
      "scenario_kind": "manifest_packet",
      "description": "INFO host-query packet",
      "tags": ["w15", "info", "host_query"],
      "pack_tags": ["packet", "row_first"],
      "generator_ref": "docs/function-lane/W15_INFO_PRE_SCENARIO_MANIFEST_SEED.csv",
      "evidence_ids": ["W15-INFO-PRE-20260315"],
      "clause_ids": []
    },
    {
      "scenario_id": "W15.CELL.packet",
      "scenario_kind": "manifest_packet",
      "description": "CELL host-sensitive packet",
      "tags": ["w15", "cell", "host_query"],
      "pack_tags": ["packet", "row_first"],
      "generator_ref": "docs/function-lane/W15_CELL_HOST_PRE_SCENARIO_MANIFEST_SEED.csv",
      "evidence_ids": ["W15-CELL-HOST-PRE-20260315"],
      "clause_ids": []
    },
    {
      "scenario_id": "W15.XLL.packet",
      "scenario_kind": "manifest_packet",
      "description": "CELL/INFO XLL bridge parity packet",
      "tags": ["w15", "xll", "bridge_parity"],
      "pack_tags": ["packet", "row_first", "xll_surface"],
      "generator_ref": "docs/function-lane/W15_XLL_BRIDGE_SCENARIO_MANIFEST_SEED.csv",
      "evidence_ids": ["W15-XLL-BRIDGE-20260315"],
      "clause_ids": []
    }
  ],
  "views": [
    "manifest_row_result_view",
    "run_summary_view",
    "analysis_summary_view",
    "evidence_binding_view",
    "limitation_view"
  ],
  "adapter_capability_ref": "docs/function-lane/OXFUNC_REPLAY_ADAPTER_CAPABILITY_MANIFEST_V1.json",
  "registry_refs": [
    {
      "registry_family": "capability_level",
      "registry_version": "foundation-handoff-20260315-pass-01"
    },
    {
      "registry_family": "predicate_kind",
      "registry_version": "foundation-handoff-20260315-pass-01"
    },
    {
      "registry_family": "mismatch_kind",
      "registry_version": "foundation-handoff-20260315-pass-01"
    },
    {
      "registry_family": "witness_lifecycle_state",
      "registry_version": "foundation-handoff-20260315-pass-01"
    }
  ],
  "sidecar_refs": [
    "docs/function-lane/W15_EXECUTION_RECORD.md",
    "docs/function-lane/XLL_VERIFICATION_SEAM_LIMITATIONS.md",
    "docs/function-lane/FUNCTION_LANE_EVIDENCE_ID_REGISTRY.md"
  ]
}
```

## 4. Expected View Content
Expected `manifest_row_result_view` content:
1. row id,
2. packet id,
3. run id,
4. run label,
5. compatibility descriptor,
6. formula or query text under test,
7. observed status and observed value text,
8. source sidecar ref,
9. evidence-id refs,
10. limitation refs where applicable.

Expected `run_summary_view` content:
1. packet/workset id,
2. run id,
3. locale/environment metadata,
4. workbook-compatibility descriptor,
5. summary row counts,
6. mismatch or expected-failure counts.

Expected `analysis_summary_view` content:
1. packet-level statement that `CELL` and `INFO` require a typed host-query seam,
2. packet-level statement that W15 native and XLL parity are green for the admitted slice,
3. packet-level statement that XLL seam limits remain separately classified.

Expected `evidence_binding_view` content:
1. `W15-INFO-PRE-20260315`
2. `W15-CELL-HOST-PRE-20260315`
3. `W15-XLL-BRIDGE-20260315`
4. supporting artifact refs from the local evidence registry.

Expected `limitation_view` content:
1. references into `docs/function-lane/XLL_VERIFICATION_SEAM_LIMITATIONS.md`,
2. explicit distinction between semantic target and XLL verification-surface qualification.

## 5. Explain Queries This Skeleton Must Support
Minimum explain queries for a future adapter run:
1. `why_row_observed`
   - explain one packet row outcome with row id, run label, and source sidecar refs.
2. `why_host_query`
   - explain why a `CELL` or `INFO` lane is host-query classified rather than pure-local.
3. `why_bridge_parity`
   - explain why one XLL bridge row is parity-clean or parity-mismatched.
4. `why_seam_limited`
   - explain why a mismatch is classified as a seam limitation instead of a semantic mismatch when the limitation record applies.

## 6. Limitation and Non-Claim Rules
Rules:
1. this skeleton is a documentation target, not a emitted bundle artifact,
2. it does not prove `cap.C0` through `cap.C3` by itself,
3. it does not imply a live `DNA ReCalc` import has happened,
4. it does not authorize any fake event-stream decomposition,
5. it does not support a `cap.C4` or `cap.C5` claim.

## 7. Promotion Use
This skeleton should be used as:
1. the target shape for the first local OxFunc packet-adapter import test,
2. the comparison basis for checking that a future live adapter output preserved evidence ids, compatibility descriptors, and limitation refs,
3. a worked example for later packet-first witness-distillation planning under `W019`.

Companion acceptance target:
1. [W15_REPLAY_ADAPTER_CONFORMANCE_CHECKLIST_V1.md](/C:/Work/DnaCalc/OxFunc/docs/function-lane/W15_REPLAY_ADAPTER_CONFORMANCE_CHECKLIST_V1.md) defines the field-presence, replay, diff, explain, projection-gap, and seam-limit checks for the first live adapter run.

# OxFunc Replay Bundle Layout and Index V1

Status: `provisional`
Owner lane: `OxFunc`
Workset: `W20`

## 1. Purpose
Define the first explicit on-disk layout and index contract for emitted OxFunc replay bundles.

This note adapts the Foundation canonical replay bundle layout to OxFunc's packet-first and row-first model.

## 2. Layout Principles
The emitted bundle layout must:
1. preserve source packet identity and source artifact refs,
2. keep normalized views distinct from source sidecars,
3. keep indexes cheap to scan and easy to regenerate,
4. reserve reduction and witness areas without implying `cap.C4` or `cap.C5`,
5. avoid any event-stream-first bias that would distort OxFunc packet semantics.

## 3. Canonical OxFunc Bundle Root
Bundle root target:

```text
<bundle-root>/
  bundle_manifest.json
  adapter_capabilities/oxfunc.json
  run_manifests/
    w15.default.json
    w15.compat_template.json
  scenario_manifests/
    W15.INFO.packet.json
    W15.CELL.packet.json
    W15.XLL.packet.json
  views/
    manifest_row_result_view/
      w15.default.csv
      w15.compat_template.csv
    run_summary_view/
      w15.default.json
      w15.compat_template.json
    analysis_summary_view/
      W15.json
    evidence_binding_view/
      W15.csv
    limitation_view/
      W15.csv
    source_inventory_view/
      W15.csv
  diff/
    expected/
    emitted/
  explain/
    expected/
    emitted/
  sidecars/
    source/
    normalized/
  indexes/
    source_inventory.csv
    scenario_index.csv
    run_index.csv
    evidence_index.csv
    limitation_index.csv
    diff_index.csv
    explain_index.csv
    witness_index.csv
    reduction_index.csv
  registries/
    capability_level.json
    predicate_kind.json
    mismatch_kind.json
    reduction_status.json
    witness_lifecycle_state.json
  reductions/
  witnesses/
```

## 4. Directory Rules
### 4.1 Root-level files
1. `bundle_manifest.json` is mandatory.
2. `adapter_capabilities/oxfunc.json` is mandatory and should mirror the local capability manifest.

### 4.2 `run_manifests/`
One JSON file per emitted run.

Current worked-example rule:
1. `w15.default.json`
2. `w15.compat_template.json`

### 4.3 `scenario_manifests/`
One JSON file per packet/scenario family, not per internal micro-event.

Current worked-example rule:
1. `W15.INFO.packet.json`
2. `W15.CELL.packet.json`
3. `W15.XLL.packet.json`

### 4.4 `views/`
Views are normalized outputs, not source artifacts.

View file-shape rule:
1. row-dense views should use CSV where that preserves packet readability,
2. summary views should use JSON when structure is hierarchical,
3. emitted file names should carry the run id or packet id explicitly.

### 4.5 `sidecars/`
`sidecars/source/`:
1. source manifests,
2. source result CSVs,
3. source execution records and limitation notes when copied into the bundle root.

`sidecars/normalized/`:
1. adapter-generated auxiliary JSON or CSV material not elevated to a first-class view or index.

Sidecar rule:
1. sidecars may be copied, linked, or referenced by stable relative path,
2. but the bundle indexes must make their resolution explicit.

### 4.6 `indexes/`
Indexes are required for cheap lookup across packet-first bundles.

Required current baseline indexes:
1. `source_inventory.csv`
2. `scenario_index.csv`
3. `run_index.csv`
4. `evidence_index.csv`
5. `limitation_index.csv`
6. `diff_index.csv`
7. `explain_index.csv`

Reserved current baseline indexes:
1. `witness_index.csv`
2. `reduction_index.csv`

### 4.7 `registries/`
The bundle must pin the registry snapshots it depends on.

Current baseline:
1. `capability_level.json`
2. `predicate_kind.json`
3. `mismatch_kind.json`
4. `reduction_status.json`
5. `witness_lifecycle_state.json`

### 4.8 `diff/` and `explain/`
Use a split between:
1. `expected/`
2. `emitted/`

Reason:
1. current OxFunc replay rollout already defines expected target shapes for `W15`,
2. the first live adapter run should be able to compare emitted outputs against those targets cleanly.

### 4.9 `reductions/` and `witnesses/`
Reserved now, not active now.

Rule:
1. their presence in the layout does not imply `cap.C4` or `cap.C5`.

## 5. Index File Contracts
### 5.1 `source_inventory.csv`
Required columns:
1. `source_ref_id`
2. `source_schema_id`
3. `bundle_relative_path`
4. `source_role`
5. `packet_id`
6. `run_scope`
7. `notes`

### 5.2 `scenario_index.csv`
Required columns:
1. `scenario_id`
2. `scenario_kind`
3. `packet_id`
4. `generator_ref`
5. `evidence_ids`

### 5.3 `run_index.csv`
Required columns:
1. `run_id`
2. `packet_id`
3. `run_label`
4. `compat_descriptor`
5. `profile_id`
6. `source_manifest_refs`

### 5.4 `evidence_index.csv`
Required columns:
1. `evidence_id`
2. `packet_id`
3. `scenario_id`
4. `run_scope`
5. `artifact_ref`
6. `notes`

### 5.5 `limitation_index.csv`
Required columns:
1. `limitation_ref_id`
2. `packet_id`
3. `scope_ref`
4. `limitation_artifact_ref`
5. `limitation_class`
6. `notes`

### 5.6 `diff_index.csv`
Required columns:
1. `diff_id`
2. `packet_id`
3. `left_run_id`
4. `right_run_id`
5. `mismatch_kind`
6. `severity`
7. `artifact_ref`

### 5.7 `explain_index.csv`
Required columns:
1. `query_id`
2. `packet_id`
3. `query_kind`
4. `scope_ref`
5. `artifact_ref`
6. `notes`

## 6. W15 Worked Example Layout Target
Concrete target root:

```text
.tmp/replay-bundles/oxfunc-w15-v1/
```

Worked-example rule:
1. keep the first emitted bundle under `.tmp/` unless later promotion doctrine says otherwise,
2. do not write into canonical docs directories from the live adapter run,
3. use the index files above as the first machine-readable audit surface.

## 7. Sidecar Resolution Rules
Resolution priority:
1. bundle-relative copied sidecar path
2. bundle-relative stable reference path
3. explicit external source ref marked as external

Rule:
1. no sidecar ref may be left as prose only,
2. any unresolved sidecar must be surfaced in indexes as a projection gap.

## 8. Non-Claim Rule
This note does not:
1. emit a real bundle,
2. validate a real bundle,
3. prove `cap.C0` through `cap.C3`,
4. authorize `cap.C4` or `cap.C5`.

## 9. Next Use
`W021` should use this layout as the on-disk target for the first live `W15` adapter run and record any deviations explicitly as:
1. projection gaps,
2. seam limitations,
3. or real semantic/adaptation errors.

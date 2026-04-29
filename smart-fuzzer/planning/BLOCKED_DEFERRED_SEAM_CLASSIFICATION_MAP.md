# Blocked And Deferred Seam Classification Map

Status: `planning_artifact_ready`

Owning bead: `oxf-1avj.5`

Owning workset:
`docs/worksets/W089_SMART_FUZZER_SWEEPING_INVOCATION_SPACE_EXPLORATION.md`

This note defines how W089 separates function mismatches from missing context,
blocked host seams, and deferred provider lanes.

## 1. Builder

Default command:

```powershell
powershell -ExecutionPolicy Bypass -File smart-fuzzer\tools\Build-SweepPlanningArtifacts.ps1
```

Output:

```text
smart-fuzzer/cache/blocked-seam-map-v0.json
```

## 2. Lane Classes

Blocked/deferred lane classes:

1. `current_version_deferred_inventory`
2. `catalog_only_metadata_gap`
3. `arity_metadata_gap`
4. `special_interface_context_required`
5. `host_interaction_context_required`
6. `external_provider_context_required`
7. `locale_profile_context_required`
8. `cube_provider_context_required`
9. `volatile_context_control_required`

These lane classes are coverage facts. They are not function-semantic failures.

## 3. Classification Rules

1. Rows with provider, cube, RTD, or external-provider requirements are
   `context_provider_blocked` unless the run manifest names a live fixture.
2. Locale-sensitive rows are `context_provider_blocked` or
   `oxfml_seam_blocked` unless a locale profile is declared.
3. Unknown arity is a metadata/generator blocker before high-volume generation.
4. Volatile rows require explicit recalc controls and repeatability policy.
5. Special interface rows require a seam-specific harness before comparison
   results can be treated as function evidence.

## 4. Required Reporting

Every run rollup must include:

1. count by blocked lane,
2. count by surface,
3. count by category,
4. skipped Excel quota due to blocked lane,
5. fixture requirement for each untested blocked lane.

## 5. Current Status Axes

1. `execution_state`: `blocked_seam_map_ready`
2. `scope_completeness`: `scope_partial`
3. `target_completeness`: `target_partial`
4. `integration_completeness`: `partial`
5. `open_lanes`: execution approval, mismatch triage protocol

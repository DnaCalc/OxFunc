# Generator Matrix And Typed Mutator Plan

Status: `planning_artifact_ready`

Owning bead: `oxf-1avj.2`

Owning workset:
`docs/worksets/W089_SMART_FUZZER_SWEEPING_INVOCATION_SPACE_EXPLORATION.md`

This note defines how W089 turns the dimension inventory into generation
families. It is still planning infrastructure: it creates a matrix and budgets,
but it does not generate or evaluate fuzzer cases.

## 1. Builder

Default command:

```powershell
powershell -ExecutionPolicy Bypass -File smart-fuzzer\tools\Build-SweepPlanningArtifacts.ps1
```

Primary output:

```text
smart-fuzzer/cache/generator-matrix-v0.json
```

The builder also emits planning JSON for the local dry-run budget, Excel
candidate budget, blocked seam map, and roadmap trace template so that the
remaining W089 planning beads share the same inventory source.

## 2. Matrix Inputs

The matrix consumes `smart-fuzzer/cache/dimension-inventory-v0.json`. If that
cache is absent, the builder rebuilds it with
`smart-fuzzer/tools/Build-DimensionInventory.ps1`.

Each surface row contributes:

1. arity shape and arity probe tags,
2. universal value-kind probes,
3. profiled focus probes,
4. numeric, text, array, and reference axes,
5. context and execution seam axes,
6. known-deviation tags,
7. blocked/deferred lanes,
8. risk and selection tags.

## 3. Mandatory Basis

Every non-blocked surface gets a small basis rather than a full Cartesian
product:

1. at-min and at-max arity probes,
2. omitted and explicit-missing probes where optional arity exists,
3. a representative scalar number/text/logical/error/blank/missing set,
4. one single-cell-array contrast,
5. one execution seam from each available seam family,
6. one bit-exact comparison policy row.

Unknown-arity rows get `metadata_gap_arity_probe` and remain eligible for
metadata-repair or catalog-review sampling before high-volume generation.

## 4. Mutator Families

The matrix assigns typed mutators by row features:

1. `arity_edge_mutator`
2. `optional_missing_mutator`
3. `variadic_budget_mutator`
4. `scalar_value_kind_mutator`
5. `numeric_band_mutator`
6. `numeric_boundary_mutator`
7. `text_band_mutator`
8. `array_shape_mutator`
9. `reference_fixture_mutator`
10. `context_fixture_mutator`
11. `execution_seam_mutator`
12. `lookup_family_mutator`
13. `financial_solver_mutator`
14. `date_time_mutator`
15. `text_search_slice_mutator`
16. `statistical_distribution_mutator`
17. `operator_form_mutator`
18. `known_deviation_reference_mutator`
19. `blocked_lane_sentinel_mutator`

Later implementation can split these tags into concrete Rust generators. W089
only pins the matrix and budgets.

## 5. Sampling Shape

Sampling is staged:

1. mandatory basis for every surface,
2. pairwise combinations across arity, value kind, array/reference, context,
   and execution seam,
3. strength-3 combinations for high-risk functions and known-bug-adjacent
   families,
4. family-specific mutators for financial, statistical, lookup/reference,
   operator, date/time, and text families,
5. local outcome-diversity feedback before any Excel spend.

The matrix deliberately avoids a full Cartesian product. The generator should
record skipped dimensions and reason codes in coverage rollups.

## 6. Output Contract

`generator-matrix-v0.json` contains:

1. `schema_version`,
2. `inventory_ref`,
3. `summary`,
4. `rows[]`.

Each row contains:

1. `surface_id`,
2. `canonical_surface_name`,
3. `category`,
4. `arity_shape`,
5. `mandatory_basis_tags`,
6. `mutator_tags`,
7. `selection_reason_tags`,
8. `known_deviation_tags`,
9. `blocked_or_deferred_lanes`,
10. `local_case_budget`,
11. `excel_candidate_quota`.

Budgets are planning quotas. They are not execution evidence.

## 7. Current Status Axes

1. `execution_state`: `generator_matrix_ready`
2. `scope_completeness`: `scope_partial`
3. `target_completeness`: `target_partial`
4. `integration_completeness`: `partial`
5. `open_lanes`: local dry-run plan, Excel candidate budget, blocked seam
   classification, execution approval, mismatch triage protocol

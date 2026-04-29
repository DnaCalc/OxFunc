# Dimension Inventory And Coverage Taxonomy

Status: `schema_ready`

Owning bead: `oxf-1avj.1`

Owning workset:
`docs/worksets/W089_SMART_FUZZER_SWEEPING_INVOCATION_SPACE_EXPLORATION.md`

This note defines the first machine-readable inventory contract for broad
smart-fuzzer exploration. It turns the W089 axis plan into a function-by-function
matrix that later generator, local-evaluation, and Excel-candidate beads can
consume.

The inventory is not semantic authority and is not a run result. It is a
derived planning artifact.

## 1. Builder

Default command:

```powershell
powershell -ExecutionPolicy Bypass -File smart-fuzzer\tools\Build-DimensionInventory.ps1
```

Default output:

```text
smart-fuzzer/cache/dimension-inventory-v0.json
```

The cache output is local/derived. Rebuild it when the library-context snapshot,
bug register, or function-lane inventories move.

## 2. Inputs

The builder consumes:

1. `docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv`
2. `docs/function-lane/FUNCTION_CATALOG_CURRENT_BASELINE_LOCAL.csv`
3. `docs/bugs/BUG_STREAM_REGISTER.csv`
4. `docs/function-lane/W50_DEFERRED_CURRENT_VERSION_INVENTORY.csv`
5. `docs/function-lane/W51_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_INVENTORY.csv`
6. `docs/function-lane/*DEFERRED*INVENTORY*.csv`
7. `docs/function-lane/*SCENARIO_MANIFEST_SEED.csv`
8. `crates/oxfunc_core/src/functions/*.rs`

Missing optional inventories should produce empty reference lists, not function
semantic claims.

## 3. Inventory Row Shape

Each `surfaces[]` row has this shape:

1. `surface_id`
2. `canonical_surface_name`
3. `category`
4. `metadata`
5. `function_surface`
6. `arity`
7. `value_type_axes`
8. `numeric_axes`
9. `text_axes`
10. `array_axes`
11. `reference_axes`
12. `context_axes`
13. `execution_seams`
14. `comparison_policy`
15. `coverage_counter_dimensions`
16. `known_deviation_tags`
17. `blocked_or_deferred_lanes`
18. `risk_and_selection_tags`
19. `refs`

Rows should be compact. They identify dimensions to sample and counters to
aggregate; they are not per-case evidence.

## 4. Arity Taxonomy

Arity shape values:

1. `unknown_arity`
2. `exact`
3. `optional_suffix_range`
4. `variadic_known_min`

Arity probe tags:

1. `argc_below_min`
2. `argc_at_min`
3. `argc_at_max`
4. `argc_above_max`
5. `omitted_optional_suffix`
6. `explicit_missing_optional`
7. `empty_argument_optional`
8. `variadic_budget_low`
9. `variadic_budget_mid`
10. `variadic_budget_high`
11. `metadata_gap_arity_probe`

The first generator bead should use these tags to create concrete call shapes
without expanding to a full Cartesian product.

## 5. Value-Type Taxonomy

Universal scalar/control probes:

1. `scalar_number`
2. `scalar_text`
3. `scalar_logical`
4. `scalar_error`
5. `blank_cell`
6. `empty_cell`
7. `missing_arg`

Profiled focus probes:

1. `array_literal`
2. `array_spill`
3. `reference_single_cell`
4. `reference_area`
5. `reference_multi_area`
6. `callable_or_lambda`
7. `presentation_or_rich_value`
8. `provider_context_value`

The inventory deliberately separates universal invalid/edge controls from
profiled focus probes. Later generators should choose between valid-path
exploration and invalid-control exploration explicitly.

## 6. Numeric Taxonomy

Standard numeric bands:

1. `positive_zero`
2. `negative_zero`
3. `tiny_magnitude`
4. `small_integer`
5. `large_integer`
6. `two_to_53_adjacent`
7. `fraction_near_integer`
8. `half_fraction`
9. `ordinary_fraction`
10. `large_finite`
11. `overflow_neighborhood`
12. `underflow_neighborhood`
13. `date_serial_low`
14. `date_serial_leap_boundary`
15. `date_serial_high`
16. `rate_near_zero`
17. `rate_high`
18. `solver_seed_sensitive`

NaN and infinity are local seam probes only when representable. They are not
Excel input parity claims.

## 7. Text Taxonomy

Standard text bands:

1. `empty_string`
2. `whitespace_only`
3. `numeric_looking_text`
4. `date_looking_text`
5. `boolean_looking_text`
6. `error_looking_text`
7. `case_variant`
8. `unicode_sample`
9. `normalization_sensitive`
10. `wildcard_chars`
11. `regex_like_chars`
12. `delimiter_heavy`
13. `long_string`

Locale-specific text parsing is a context axis, not a default pass tolerance.

## 8. Array And Reference Taxonomy

Array shape bands:

1. `scalar_control`
2. `single_cell_array`
3. `row_vector`
4. `column_vector`
5. `small_matrix`
6. `mixed_type_matrix`
7. `contains_errors`
8. `contains_blanks`
9. `shape_mismatch_pair`
10. `spill_size_edge_sample`
11. `grid_limit_sample`

Reference bands:

1. `single_cell`
2. `rectangular_area`
3. `same_sheet_multi_area`
4. `cross_sheet_reference`
5. `whole_row`
6. `whole_column`
7. `spill_anchor`
8. `structured_reference`
9. `external_reference_blocked`
10. `reference_vs_array_literal_contrast`

## 9. Context And Seam Taxonomy

Context bands:

1. `excel_version_channel`
2. `workbook_compatibility_version`
3. `date_system`
4. `locale_profile`
5. `caller_location`
6. `calculation_mode`
7. `volatile_recalc`
8. `host_provider_capability`
9. `worksheet_neighborhood`

Execution seam bands:

1. `direct_oxfunc_value`
2. `oxfml_prepared_call`
3. `excel_formula_text`
4. `xll_bridge_future`
5. `provider_host_future`

Blocked or deferred lanes must stay visible in coverage rollups. They should
not be counted as function mismatches.

## 10. Comparison Taxonomy

Current OxFunc parity policy is bit-exact typed equality. The smart-fuzzer must
not classify approximate numeric agreement as a pass.

Comparison classes:

1. `exact_typed_bit_match`
2. `known_expected_deviation`
3. `unexpected_mismatch`
4. `excel_harness_blocked`
5. `oxfml_seam_blocked`
6. `context_provider_blocked`
7. `invalid_generator_case`
8. `unstable_or_non_reproducible`

Expected PMT/PPMT/IPMT financial-payment exactness drift is a known deviation
class and a useful reference mismatch lane. It is not a repair target under
W089.

The former POWER/OP_POWER stale-claim check was freshly confirmed and closed
under W078 on 2026-04-29. Future POWER mismatches should be triaged as new
signals rather than assumed continuations of BUG-FUNC-005.

## 11. Coverage Counter Dimensions

Every later run rollup should be able to aggregate by:

1. `surface_id`
2. `canonical_surface_name`
3. `category`
4. `metadata_status`
5. `special_interface_kind`
6. `runtime_boundary_kind`
7. `arity_shape`
8. `arity_probe`
9. `value_kind_vector`
10. `numeric_band`
11. `text_band`
12. `array_shape_band`
13. `reference_band`
14. `context_band`
15. `execution_seam`
16. `local_outcome_class`
17. `excel_comparison_class`
18. `known_deviation_tag`
19. `blocked_or_deferred_lane`
20. `selection_reason`

This is the durable pass telemetry surface. Per-case prose should remain
reserved for failures, unstable rows, blocked harness cases, and reduced
reproducers.

## 12. Initial Snapshot Observations

The current library-context snapshot contains `534` rows. Snapshot profile
counts observed during this bead:

1. `metadata_status`: `217` extracted, `302` curated, `14` catalog-only, `1`
   doc-modeled.
2. `special_interface_kind`: `513` ordinary rows, with the remaining rows in
   callable, presentation, width-conversion, host, provider, registration, and
   operator lanes.
3. `arg_preparation_profile`: `349` `ValuesOnlyPreAdapter`, `57`
   `RefsVisibleInAdapter`, `128` blank/unknown.
4. `coercion_lift_profile`: `291` `Custom`, `38`
   `UnaryNumericScalarOrArrayElementwise`, `33`
   `AggregateDirectAndRangeDualPolicy`, `28` `None`, `14`
   `UnaryNumericScalarOnly`, `2` `LookupMatchProfile`, `128` blank/unknown.

These are inventory facts, not completion claims.

## 13. Current Status Axes

1. `execution_state`: `inventory_schema_ready`
2. `scope_completeness`: `scope_partial`
3. `target_completeness`: `target_partial`
4. `integration_completeness`: `partial`
5. `open_lanes`: generator matrix, local dry-run plan, Excel candidate budget,
   blocked seam classification, execution approval, mismatch triage protocol

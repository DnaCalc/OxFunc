# Function Slice Contract (Prelim) - SUBTOTAL() / AGGREGATE()

## 1. Purpose
Pin the admitted current-baseline OxFunc slice for the row-visibility aggregate family.

## 2. Admitted Slice
1. `SUBTOTAL` over current reference-baseline function numbers `1..11` and `101..111`,
2. `AGGREGATE` over current reference-baseline function numbers `1..19`,
3. reference-form row visibility and nested-aggregate suppression,
4. `AGGREGATE` option handling for:
   - nested aggregate suppression,
   - manual hidden-row suppression,
   - filtered-row suppression,
   - error suppression.

## 3. OxFunc / Host Seam
1. `arg_preparation_profile`: `refs_visible_in_adapter`
2. required typed host callback:
   - `query_aggregate_reference_context(reference)`
3. the callback returns shape-aligned per-cell facts:
   - `row_hidden_manual`
   - `row_filtered_out`
   - `nested_subtotal_or_aggregate`
4. OxFunc uses those facts only to filter the resolved reference payload, then delegates to the ordinary aggregate/statistical kernels already present in core.

## 4. Current-Baseline Findings
1. `SUBTOTAL` always ignores filtered-out rows.
2. `SUBTOTAL` always ignores nested `SUBTOTAL` / `AGGREGATE` cells.
3. `SUBTOTAL(1..11, ...)` includes manually hidden rows.
4. `SUBTOTAL(101..111, ...)` excludes manually hidden rows.
5. on the admitted reference-form slice, `AGGREGATE` options `0..3` ignore nested `SUBTOTAL` / `AGGREGATE` cells, while options `4..7` keep nested aggregate results and only split hidden/filter/error handling.

## 5. XLL Seam Note
1. the typed callback surface is pinned in OxFunc core.
2. the generated XLL bridge does not yet supply this callback surface end-to-end.
3. current closure reading therefore rests on:
   - core/runtime tests with mock host context,
   - native Excel replay artifacts,
   - and explicit seam documentation,
   not on complete XLL bridge parity.

## 6. Status
1. runtime_status: `evidenced`
2. seam_status: `typed_host_query_pinned`
3. closure_reading: admitted current-baseline OxFunc side is closure-grade for `SUBTOTAL` / `AGGREGATE` in this packet.

# Interesting Functions Initial Classification

## Purpose
Provide a complete, first-pass class-axis mapping for all currently tagged interesting functions (tiers 3/4/5).

## Source
1. `research/runs/20260228-130325-excel-compat-spec-index-pass-01/outputs/function_interest_index.csv`
2. Current function-definition class axes in:
   - `EXCEL_FUNCTION_DEFINITION_PRELIM_SPEC.md`
   - `EXCEL_FUNCTION_DEFINITION_PRELIM_CONFORMANCE.csv`

## Artifact
1. `INTERESTING_FUNCTIONS_INITIAL_CLASSIFICATION.csv`

## Coverage Snapshot
1. Total rows: `71`
2. Tier distribution:
   - tier 3 (`medium_interest`): `23`
   - tier 4 (`high_interest`): `43`
   - tier 5 (`critical_interest`): `5`

## Notes
1. This is intentionally heuristic and marked for interactive refinement.
2. Classification confidence is encoded per row (`low`/`medium`) and should not be treated as final policy.
3. Priority review targets:
   - `INDIRECT`, `OFFSET` (context-dependence vs non-determinism),
   - `RTD` lifecycle and invalidation,
   - `XLOOKUP` reference-output behavior,
   - `XMATCH` adapter parity completion (wildcard/binary/approximate implementation),
   - volatile family precision (`volatile_full` vs `volatile_contextual`).
4. Confirmed interesting seam behavior so far:
   - `SUM` is argument-structure-sensitive at the function boundary: `SUM("2",TRUE)` follows direct-scalar coercion and yields `3`, while `SUM({"2",TRUE})` and `SUM(range-with-text-and-logical)` follow array-scan policy and yield `0`. OxFunc now models that distinction explicitly and treats source-erased arrays as an `opaque_array_value` fallback that still uses scan policy.
   - `INDEX` is reference-return sensitive in two separate ways: explicit blank `row_num` / `col_num` positions mean `0`, and `area_num` selection happens before row/column slicing, including same-sheet multi-area references.
   - `INDIRECT` has a sharp omission seam on `a1_style`: omitting the second argument defaults to `TRUE`, but explicitly leaving it blank behaves like `FALSE` and switches interpretation to R1C1.
   - `SEQUENCE` is not just a shape producer: the current baseline requires full row-major payload materialization, including omitted middle-argument defaults for `rows`, `columns`, `start`, and `step`.

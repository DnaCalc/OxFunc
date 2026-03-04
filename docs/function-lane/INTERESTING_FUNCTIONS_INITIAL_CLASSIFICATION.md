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
   - volatile family precision (`volatile_full` vs `volatile_contextual`).

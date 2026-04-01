# Function Slice - Database Family Current-Baseline Promotion Contract (Prelim)

Status: `active`
Owner lane: `OxFunc`
Workset: `W065`

## 1. Purpose
Promote the already-evidenced W23 database-family slice into the ordinary-backlog closure program and the published current-baseline snapshot.

## 2. Covered Surface
1. `DAVERAGE`
2. `DCOUNT`
3. `DCOUNTA`
4. `DGET`
5. `DMAX`
6. `DMIN`
7. `DPRODUCT`
8. `DSTDEV`
9. `DSTDEVP`
10. `DSUM`
11. `DVAR`
12. `DVARP`

## 3. Admitted Current-Baseline Slice
1. the database argument is a rectangular header-plus-records range.
2. the field argument is admitted as a header text or 1-based column index.
3. the criteria range is admitted as a rectangular header-plus-criteria grid.
4. duplicate criteria headers and multi-row criteria `OR` semantics are in scope.
5. text-prefix matching and simple comparison operators (`=`, `>`, `<`, `>=`, `<=`, `<>`) are in scope.
6. omitted field handling for `DCOUNT` is in scope.

## 4. Main Rules Carried Into W065
1. database headers are matched against the field argument and criteria headers.
2. a criteria row is an `AND` conjunction across populated criteria cells.
3. multiple criteria rows are `OR`.
4. duplicate criteria headers in the same row are also `OR`.
5. plain text criteria on the admitted slice use Excel-style prefix matching.
6. `DGET` returns `#NUM!` when more than one record matches and `#VALUE!` when no record matches.
7. `DCOUNT` with omitted field counts records that match the criteria.
8. sample vs population formulas for `DSTDEV` / `DSTDEVP` / `DVAR` / `DVARP` follow the ordinary statistical distinction over the matched records.

## 5. Runtime / Formal Anchors
Runtime anchors:
1. `crates/oxfunc_core/src/functions/database_family.rs`
2. `crates/oxfunc_core/src/functions/surface_dispatch.rs`

Formal anchors:
1. `formal/lean/OxFunc/Functions/DatabaseFamily.lean`

Native replay anchors:
1. `docs/function-lane/W65_SCENARIO_MANIFEST_SEED.csv`
2. `tools/w65-probe/run-w65-database-baseline.ps1`
3. `.tmp/w65-database-results.csv`

Provenance anchors:
1. `docs/function-lane/FUNCTION_SLICE_DATABASE_FAMILY_CONTRACT_PRELIM.md`
2. `docs/function-lane/W23_DATABASE_SCENARIO_MANIFEST_SEED.csv`
3. `docs/function-lane/W23_EXECUTION_RECORD.md`

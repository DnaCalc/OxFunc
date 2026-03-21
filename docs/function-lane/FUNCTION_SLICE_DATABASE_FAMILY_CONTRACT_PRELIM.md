# Function Slice - Database Family Contract (Prelim)

Status: `provisional`
Workset: `W23`

## 1. Purpose
1. define the admitted current-baseline semantic slice for the `D*` database family,
2. separate record/header/criteria-grid semantics from the host-sensitive residuals that also live in `W23`,
3. pin the bounded current-baseline slice with real Rust, Lean, and native Excel evidence.

## 2. In-Scope Members
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

## 3. Admitted Slice
1. the database argument is a rectangular header-plus-records range.
2. the field argument is admitted as a header text or 1-based column index.
3. the criteria range is admitted as a rectangular header-plus-criteria grid.
4. duplicate criteria headers and multi-row criteria `OR` semantics are in scope.
5. text-prefix matching and simple comparison operators (`=`, `>`, `<`, `>=`, `<=`, `<>`) are in scope.
6. omitted field handling for `DCOUNT` is in scope.

## 4. Main Rules Pinned In This Packet
1. database headers are matched against the field argument and criteria headers.
2. a criteria row is an `AND` conjunction across populated criteria cells.
3. multiple criteria rows are `OR`.
4. duplicate criteria headers in the same row are also `OR`.
5. plain text criteria on the admitted slice use Excel-style prefix matching.
6. `DGET` returns `#NUM!` when more than one record matches and `#VALUE!` when no record matches.
7. `DCOUNT` with omitted field counts records that match the criteria.
8. sample vs population formulas for `DSTDEV` / `DSTDEVP` / `DVAR` / `DVARP` follow the ordinary statistical distinction over the matched records.

## 5. Out Of Scope
1. criteria formulas evaluated in full worksheet context.
2. locale-sensitive parsing beyond the seeded current-baseline text/numeric lanes.
3. workbook-version sweep and historical-version sweep.
4. host-sensitive functions elsewhere in `W23`.

## 6. Boundary Notes
1. this family is not a host-query seam in the same sense as `GETPIVOTDATA`, `ISFORMULA`, or `HYPERLINK`.
2. once prepared references resolve to database and criteria grids, the admitted current-baseline slice is OxFunc-owned.
3. the family remains separate from ordinary scalar/range packets because record/header/criteria geometry is its own semantic substrate.

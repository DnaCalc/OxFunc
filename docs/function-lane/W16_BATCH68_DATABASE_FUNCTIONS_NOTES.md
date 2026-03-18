# W16 Batch 68 - Database Functions

Status: `in_progress-self-contained`
Workset: `W16`
Evidence ID: `W16-BATCH68-DATABASE-FUNCTIONS-20260316`

## Scope
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

## Shape
1. This batch is a bounded reference-visible database family over a local record-selection engine.
2. The engine parses a database table from a 2-D range, resolves the field by header text or 1-based index, and applies a criteria table with Excel-style OR-of-rows / AND-of-columns semantics.
3. Plain text criteria are treated as case-insensitive prefix matches, explicit operators (`=`, `<>`, `<`, `<=`, `>`, `>=`) are supported, and wildcard matching is supported on equality/inequality text criteria.
4. `DCOUNT` and `DCOUNTA` admit the bounded omitted-field slice and count matching records when the field argument is omitted.
5. `DGET` is bounded to the ordinary single-record extraction slice: one match returns the field value, zero matches return `#VALUE!`, and multiple matches return `#NUM!`.

## Executable Coverage
1. Rust unit tests pin seeded lanes for prefix-text criteria, duplicate-header range criteria, omitted-field `DCOUNT`, numeric aggregates, variance/stddev aggregates, and `DGET` uniqueness behavior.
2. The Lean file is metadata-only and mirrors the shared `FunctionMeta` profile for the family.

## Open Lanes
1. Formula-criteria rows, blank-header criteria columns, and other advanced-filter-style criteria constructs are out of scope in this bounded family.
2. Criteria-header miss behavior is bounded to `#VALUE!` in this slice rather than a broader Excel characterization packet.
3. Shared dispatch/catalog/root-import wiring remains separate work because it is outside the owned-file scope.
4. Successor ownership:
   - this family is now extracted to `W023` because its record/criteria-grid semantics are better treated as a separate host/metadata/database packet than as an ordinary bounded pure-family residual.

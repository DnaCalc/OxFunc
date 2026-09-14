# BUGREP-FUNC-027: Referenced blank cell publishes #VALUE! in scalar numeric contexts

## Intake
- **Report id**: `BUGREP-FUNC-027`
- **Filed**: 2026-09-14
- **Source channel**: other (program investigation survey + campaign bead)
- **Reporter/source**: `Foundation/notes/PROGRAM_INVESTIGATION_2026-09-10.md` (§1 "`=A1+1` with blank `A1` returns `#VALUE!` where Excel returns `1`, and a unit test pins the wrong behaviour"); bead `oxf-xvt5.1` (W110-1 / campaign W4)
- **Reported against ref**: `5166ffd`
- **Reported against kind**: commit
- **Reported against note**: `main` HEAD when the bead was claimed; the survey's claim was made by code reading and was reproduced here for the first time by executing the truth table.
- **Canonical bug id**: `BUG-FUNC-049`
- **Status**: triaged

## Observed Symptom
Any formula that reads a blank cell where exactly one number is expected publishes
`#VALUE!` instead of treating the blank as `0`: `=A1+1`, `=A1*2`, `=A1-1`, `=A1/2`,
`=A1^2`, `=A1%`, `=-A1`, `=ABS(A1)`, `=ROUND(A1,2)`, `=NOT(A1)`, and the blank slot of a
lifted `=A1:A3+1`. `=2/A1` publishes `#VALUE!` instead of `#DIV/0!`. Aggregates, text
contexts, comparisons, `IF`, `TEXT`, `ISBLANK`/`ISNUMBER` and unary plus were already right.

## Reproduction
1. `cargo test -p oxfunc_core --offline --lib blank_cell_coercion_truth_table` at `5166ffd`
   with the new truth-table module added (`crates/oxfunc_core/src/functions/blank_cell_coercion_truth_table.rs`),
   which drives `eval_surface_value_call` with a `ReferenceSystemProvider` resolving `A1` to
   `CoreValue::Empty`.
2. Expected: all 32 rows publish the Excel value (see the module header for provenance).
3. Actual at `5166ffd`: 12 of 32 rows diverge — 10 scalar-numeric rows (`#VALUE!` where
   `1`, `0`, `0`, `0`, `0`, `-1`, `0`, `#DIV/0!`, `0`, `0` are expected), `=NOT(A1)`
   (`#VALUE!` for `TRUE`), and `=A1:A3+1` (`{#VALUE!;3;4}` for `{1;3;4}`).

## Initial Ownership Read
- **Initial classification**: OxFunc-owned bug
- **Reason**: The prepared-scalar numeric funnel `functions::adapters::coerce_prepared_to_number`
  mapped `CoreValue::Empty` to `CoercionError::EmptyCell`, which every scalar adapter's error
  mapper publishes as `#VALUE!`. No OxFml/OxCalc involvement: the dispatch-level reproduction
  fails inside OxFunc alone.

## Links
1. `crates/oxfunc_core/src/functions/blank_cell_coercion_truth_table.rs`
2. `docs/bugs/streams/BUG-FUNC-049_blank_cell_scalar_numeric_coercion.md`
3. bead `oxf-xvt5.1` (epic `oxf-xvt5`, W110)

## Triage Notes
Linked to canonical stream `BUG-FUNC-049` on filing. The survey's narrower guess (only the
`ABS`/`ROUND`-style single-argument path) was too small: the binary operators and the unary
minus/percent executor were on the same funnel.

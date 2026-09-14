# BUG-FUNC-049: Blank cell in a scalar numeric context publishes #VALUE! instead of 0

## Summary
- **Bug id**: `BUG-FUNC-049`
- **Opened**: 2026-09-14
- **Status**: `validated_local`
- **Owner workset**: `W110` (bead `oxf-xvt5.1`, campaign W4 blank-cell coercion)

## Source Refs
- **Reported against ref**: `5166ffd`
- **Reproduced on ref**: `5166ffd` (the truth table below, executed before any production change)
- **Introduced in ref**: unknown (the funnel has mapped `Empty` to `EmptyCell` since it was written; the pinning unit test `binary_numeric_surface_lifts_scalar_array_elementwise` asserted the `#VALUE!`)
- **Fixed in ref**: the W110-1 commit that lands this stream (see `git log --grep oxf-xvt5.1`)
- **Ref notes**: The program survey (`Foundation/notes/PROGRAM_INVESTIGATION_2026-09-10.md` §1)
  named the symptom by code reading; this stream is the first executed reproduction.

## Ownership And Root Cause
- **Ownership class**: OxFunc-owned bug
- **Root cause class**: initial_impl_gap
- **Root cause summary**: `functions::adapters::coerce_prepared_to_number` — the funnel every
  scalar adapter reads a single number through (binary operators via
  `binary_numeric::eval_binary_numeric_scalars` / `map_binary_numeric_item`, the unary executor
  via `apply_unary_numeric_scalar_prepared`, `ABS`, `ROUND`, `NOT`, the k/index/mode
  arguments of the lookup and statistical surfaces) — mapped `CoreValue::Empty` to
  `CoercionError::EmptyCell`, and every scalar error mapper publishes that as `#VALUE!`.
  `EmptyCell` is a legitimate distinct signal for the aggregate lane, but the scalar lane has
  no use for it: Excel reads a referenced blank as `0` wherever one number is expected.
  The unary-numeric `_calc_surface` twin and DROP/TAKE's `parse_integer_calc` read the
  low-level `coerce_calc_scalar_to_number` directly and had the same gap.

## Why Did We Get This Wrong?
- **Spec already correct and code was wrong?**: yes (the repo's own W16 probe records pinned
  "blank-reference coercion to zero" for `DOLLARDE`/`DOLLARFR` and `DAY`/`MONTH`/`YEAR`/`DAYS`
  in 2026-03; the operators never got the same treatment)
- **Spec vague or missing?**: partly — Microsoft's operator page documents the conversion
  algebra and the `#VALUE!` for unconvertible text but does not spell out the blank case
- **Code once correct and later regressed?**: no
- **Likely introduced in ref**: unknown
- **Explanation**: `operator_arithmetic_family.rs` documents "blank -> 0" for unary plus, and
  unary plus alone had a hand-written `CoreValue::Empty => 0` arm; every other scalar surface
  trusted the shared funnel, which had the aggregate lane's semantics. A unit test then pinned
  the wrong answer, so the whole scalar lane read as "tested" while diverging from Excel.

## Reproduction
1. `cargo test -p oxfunc_core --offline --lib blank_cell_coercion_truth_table`
   (`crates/oxfunc_core/src/functions/blank_cell_coercion_truth_table.rs`: 32 rows through
   `eval_surface_value_call` with `A1 -> CoreValue::Empty`, `A2 = 2`, `A3 = 3`; the `A1:A3`
   area is served both dense and sparse).
2. Expected: every row publishes the Excel value.
3. Actual before the fix — 12 of 32 rows diverged:
   - `=A1+1` `#VALUE!` (want `1`) · `=A1*2` (want `0`) · `=-A1` (want `0`) · `=ABS(A1)`
     (want `0`) · `=ROUND(A1,2)` (want `0`) · `=A1-1` (want `-1`) · `=A1/2` (want `0`) ·
     `=2/A1` `#VALUE!` (want `#DIV/0!`) · `=A1^2` (want `0`) · `=A1%` (want `0`)
   - `=NOT(A1)` `#VALUE!` (want `TRUE`)
   - `=A1:A3+1` `{#VALUE!;3;4}` (want `{1;3;4}`)
   Already correct before the fix (20 rows): `=+A1` `0`; `=A1+""` `#VALUE!`; `=A1&"x"` `"x"`;
   `=LEN(A1)` `0`; `=TEXT(A1,"0")` `"0"`; `=A1=0` `TRUE`; `=A1=""` `TRUE`; `=IF(A1,1,2)` `2`;
   `=SUM(A1)` `0`; `=SUM(A1,1)` `1`; `=COUNT(A1)` `0`; `=COUNTA(A1)` `0`; `=AVERAGE(A1)`
   `#DIV/0!`; `=AVERAGE(A1,4)` `4`; `=MAX(A1)` `0`; `=MAX(A1,-3)` `-3`; `=PRODUCT(A1,5)` `5`;
   `=ISBLANK(A1)` `TRUE`; `=ISNUMBER(A1)` `FALSE`; `=SUM(A1:A3)` `5` (dense and sparse).

## Spec And Contract Relationship
- **Spec references**:
  1. `docs/function-lane/FUNCTION_LANE_EVIDENCE_ID_REGISTRY.md` rows
     `W16-BATCH37-DOLLAR-FRACTION-20260316` and `W16-BATCH38-DATE-PARTS-20260316`
     (native-Excel COM probes: "blank-reference coercion to zero")
  2. Microsoft, "Calculation operators and precedence in Excel"
     (<https://support.microsoft.com/en-us/office/48be406d-4975-4d31-b2b8-7af9e0e2878a>) —
     conversion algebra; unconvertible text is `#VALUE!` (the `=A1+""` boundary)
  3. Microsoft function pages for the aggregate rule (AVERAGE, COUNT, COUNTA, MAX, PRODUCT:
     "empty cells … are ignored" / "does not count empty cells") and the IS functions page —
     all fetched and checked 2026-09-14; exact URLs in the truth-table module header
- **Spec state at intake**: correct_and_not_implemented (for the scalar lane)
- **Notes**: The scalar-zero rule and the aggregate-ignore rule are different Excel
  behaviours. The fix keeps them on different helpers so neither can leak into the other.

## Investigation Log
1. 2026-09-14: Baseline floor at `5166ffd` (`cargo test -p oxfunc_core --offline --no-fail-fast`):
   1574 lib tests pass, three pre-existing reds not related to this stream —
   `finite_combinatoric_witnesses_match_excel_bits` (1-ULP, `discrete_dist_family.rs`),
   `oxfml_seam_integration::oxfunc_function_corpus_passes_through_adapter` (FN-ACOS-01 /
   FN-COTH-01 last-bit corpus rows after the W109 ACOS/COTH kernel commits), and
   `unary_numeric_equivalence_law::law3_overflowing_kernel_declares_non_pass_policy`
   (`FUNC.TANH` NaN at 710 under a PASS policy after the W109 TANH commit).
2. 2026-09-14: Truth table written first and run: 12 of 32 rows red (list above).
3. 2026-09-14: Consumer census of `coerce_prepared_to_number` (~300 sites, ~70 files) —
   every site is a scalar argument position; no caller iterates a data range and skips on
   `Err`. `CoercionError::EmptyCell` is matched only by error→`#VALUE!` mappers and by
   `coercion::aggregate_scan_sum` (no production callers). The production aggregate policies
   (`functions::aggregate_common`) ignore blanks by matching `CoreValue::Empty` before any
   coercion, so they never depend on the funnel.
4. 2026-09-14: Fix landed at the scalar layer (see Fix Plan); full floor re-run: only the same
   three pre-existing reds remain; the truth table is 8/8 green (32/32 rows).

## Similar-Risk Scan
### Adjacent families to check
1. Reference-arg numeric positions that bypass the funnel: `INDEX` row/col
   (`coercion::coerce_arg_to_number`), `OFFSET` (own `Empty` arm), `INDIRECT` (own arm),
   `MATCH`/`XMATCH` lookup value (own `EmptyCell` → `#N/A` mapping).
2. Optional-argument positions where a blank *reference* and an *omitted* argument may
   differ in Excel (`LOG` base, `WEEKDAY` return_type, `SEQUENCE` defaults, `RANDARRAY`).
3. The Lean substrate: `formal/lean/OxFunc/CoercionPrimitives.lean` (`coerceToNumber
   .emptyCell = error`) and the 61 per-function Lean models that read it directly.

### Check method
1. Extend the truth table with those positions once each row has a live-Excel answer
   (oracle probe through the COM runner); never pin a guessed value.
2. Add a `coerceScalarToNumber` to the Lean primitives mirroring
   `coerce_scalar_calc_value_to_number` and route the scalar adapter models through it.

### Results
1. Not changed in this stream: `INDEX`'s reference-arg path (`coerce_arg_to_number` still
   yields an error for a blank row/col reference) — no oracle answer in hand, so it is left as
   found and filed.
2. The Lean substrate still models the OLD scalar behaviour (`emptyCell → error`); the Rust
   scalar lane and the Lean scalar adapter models are now out of alignment on blank input.

### Follow-on Openings
1. `oxf-xvt5.4` — Lean alignment of the scalar blank rule (`coerceScalarToNumber` + the 61
   consumer models).
2. `oxf-xvt5.5` — `FunctionMeta` blank-handling axis (Step 3 of the bead, judged too wide for
   this stream).
3. `oxf-xvt5.6` — reference-arg and optional-arg blank positions off the funnel, needing
   oracle answers (`INDEX`, `OFFSET`, `INDIRECT`, `MATCH`/`XMATCH`, `SEQUENCE`, `RANDARRAY`,
   `WEEKDAY`, `LOG`, `DROP`/`TAKE`).
4. `oxf-xvt5.7` — live-Excel sign-off of the 32 truth-table rows; promotes this stream to
   `closed_signed_off`.
5. `oxf-xvt5.8` — the three pre-existing W109-lane red tests found at the baseline
   (catalogued, not fixed here).

## Fix Plan
1. `crates/oxfunc_core/src/coercion.rs`: new `coerce_scalar_calc_value_to_number`
   (`Empty → Ok(0.0)`, everything else delegated to `coerce_calc_scalar_to_number`, so
   `Missing` stays `MissingArg`). The low-level helpers and `aggregate_scan_sum` are untouched;
   their existing unit tests (`empty_cell_is_distinct_error`,
   `coerce_calc_scalar_to_number_*`) still pin the distinct `EmptyCell` signal, and a new test
   pins that the scalar rule does not leak into them.
2. `crates/oxfunc_core/src/functions/adapters.rs`: `coerce_prepared_to_number` routes
   `Missing | Empty` through the scalar helper (one declared rule, no second copy).
3. `crates/oxfunc_core/src/functions/unary_numeric.rs` (`eval_unary_numeric_calc_surface`,
   `map_unary_numeric_calc_item`) and
   `crates/oxfunc_core/src/functions/dynamic_array_reshape_family.rs` (`parse_integer_calc`):
   the `_calc` twins read the same scalar helper, so the prep-helper equivalence tests hold
   on blank input.
4. `crates/oxfunc_core/src/functions/binary_numeric.rs`:
   `binary_numeric_surface_lifts_scalar_array_elementwise` corrected — it pinned `#VALUE!`
   for a blank array slot; it now pins `0`.
5. No public signature changed; no sibling repo touched.

## Validation
1. `cargo test -p oxfunc_core --offline --lib blank_cell_coercion_truth_table` → 8 passed
   (32/32 rows, dense and sparse area providers).
2. `cargo test -p oxfunc_core --offline --no-fail-fast` → lib 1584 passed / 1 failed
   (pre-existing `finite_combinatoric_witnesses_match_excel_bits`), every integration binary
   green except the two pre-existing W109-lane reds named in the Investigation Log; no new
   failure anywhere.
3. `rustfmt --check` clean on every touched file; no clippy warning originates from a changed
   line (the touched files carry pre-existing warnings at untouched lines).
4. Live-Excel sign-off of the 32 rows: NOT performed in this stream (no oracle host in the
   session). Status therefore stays `validated_local`, not `closed_signed_off`.

## Linked Reports
1. `BUGREP-FUNC-027`

## Evidence
1. `crates/oxfunc_core/src/functions/blank_cell_coercion_truth_table.rs` (32 rows, module
   header records the provenance of every expected value)
2. `crates/oxfunc_core/src/coercion.rs` tests
   `scalar_context_reads_blank_as_zero_and_delegates_everything_else` and
   `scalar_zero_rule_does_not_leak_into_the_aggregate_facing_helpers`
3. bead `oxf-xvt5.1` description (reproduction record appended 2026-09-14)

## Closure Checklist
- [x] fix landed or non-OxFunc ownership recorded
- [x] validation recorded (local; live-Excel sign-off outstanding)
- [x] root cause recorded
- [x] similar-risk scan recorded
- [ ] spec/matrix/contract updated if required (Lean substrate alignment filed as a follow-up bead)
- [x] handoff filed if required (none required: no evaluator-facing or FEC/F3E clause changed)
- [x] linked reports updated

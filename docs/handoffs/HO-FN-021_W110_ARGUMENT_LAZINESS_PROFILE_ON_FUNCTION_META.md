# HO-FN-021: W110-4 `argument_laziness_profile` On `FunctionMeta` (reply to OxFml `HANDOFF-OXFUNC-007`)

Status: `filed`
Direction: `OxFunc -> OxFml`
Source repo/workset: `OxFunc/W110` (bead `oxf-xvt5.13`)
Replies to: `../OxFml/docs/handoffs/HANDOFF-OXFUNC-007_W079_LAZY_ARGUMENT_PROFILE_ON_FUNCTION_META.md`
(OxFml bead `fml-kt8.1`; registered in OxFunc as `HANDOFF-OXFUNC-007-W079`, acknowledged 2026-09-15 —
the `-W079` suffix only disambiguates from the DnaOneCalc `HANDOFF-OXFUNC-007` row OxFunc already carried)
Filed date: `2026-09-15`

## What Landed In OxFunc

1. `crate::function::ArgumentLazinessProfile` — a closed `Copy`/`Eq` enum in the W105 axis
   mould, carried on `FunctionMeta` as `argument_laziness_profile` and defaulted through
   `FunctionMeta::DEFAULTS_BASE` / `FunctionMeta::DEFAULT_ARGUMENT_LAZINESS_PROFILE`, so no
   existing `function_spec!` literal changed. Variants (the handoff's names, kept):

   | Variant | Functions | Shape |
   |---|---|---|
   | `Eager` | everything else, incl. `AND`/`OR`/`XOR` and every UDF | every argument evaluated before the call |
   | `BranchOnCondition` | `IF` | arg 0 decides; exactly one remaining branch |
   | `ConditionValuePairs` | `IFS` | `(condition, value)` pairs left to right; conditions until the first TRUE, that pair's value only |
   | `IndexedChoice` | `CHOOSE` | arg 0 is a 1-based index; the selected value only |
   | `MatchedCase` | `SWITCH` | arg 0 compared to candidates at odd positions; the result after the first match, or the trailing default, only |
   | `FallbackOnError` | `IFERROR`, `IFNA` | arg 0 always; arg 1 only when arg 0 is an error the function catches |

   Accessors: `is_lazy()` (non-`Eager`), `discriminator_index()` (`Some(0)` for every lazy shape,
   `None` for `Eager`).

2. **One deliberate difference from the handoff's sketch.** The sketch had
   `MatchedCase { has_trailing_default: bool }`. Whether a `SWITCH` call carries a trailing default
   is a property of the CALL's argument count (`SWITCH(expr, v1, r1, …, [default])`: a default is
   present iff the count after `expr` is odd), not of the function, so a static per-function meta
   cannot carry it. `MatchedCase` is payload-free; the per-call arity rule stays where it is today
   (OxFunc's `SWITCH` surface and OxFml's `evaluate_lazy_selector_call`).

3. **The dispatch-target query** — `FunctionCallTarget::argument_laziness_profile()` (additive
   method on the type OxFml already resolves in `CompiledFunctionCallTarget::from_surface_name`).
   `FunctionCallTarget::function_meta().argument_laziness_profile` reads the same field.

4. **Registry projection** — `FunctionSpecAxesMetadata` gains `argument_laziness_profile: String`
   (keys `eager` / `branch_on_condition` / `condition_value_pairs` / `indexed_choice` /
   `matched_case` / `fallback_on_error`); `render_registry_metadata_csv(...)` appends an
   `argument_laziness_profile` column after `real_result_policy` (19 columns); the version token
   is `function_spec_axes_metadata.v2` (the contract's own "projecting a further axis bumps it"
   rule). `FunctionSpecAxesMetadata::default_axes()` carries `eager`, so a UDF registration is
   eager. **No public signature OxFml calls today changed**: `RegistryFunctionMeta` was NOT given a
   new field (OxFml's `language_service_tests.rs:1536` and `runtime_consumer_facade_tests.rs:4023`
   construct it literally); `FunctionSpecAxesMetadata` is only constructed downstream through
   `default_axes()` / `from_meta`, which is why the projection went there.

5. **Tests.** `functions::argument_laziness_golden` (7 tests: the declared shape of each of the six
   selectors and of `AND`/`OR`/`XOR`/`SUM` matches the documented Excel behaviour, with the
   Microsoft function-reference pages and the retained live-Excel rows cited in the module header;
   the lazy set is exactly the six; every lazy shape discriminates on argument 0; the dispatch
   target exposes the axis by surface name; `LET`/`LAMBDA`/`_XLFN.SINGLE` resolve to no OxFunc
   target and carry no meta) and `registry::tests::argument_laziness_axis_agrees_with_selector_branch_over_the_catalog`
   (the handoff's cross-check: over the whole catalog, `ErrorCollapseProfile::SelectorBranch` and a
   non-`Eager` laziness profile coincide, the registry projection mirrors the meta, exactly six lazy
   functions). The `function_meta_golden.txt` snapshot was regenerated (every line gains the new
   field; only the six selectors carry a non-`Eager` value).

6. **Contract doc.** `docs/function-lane/OXFUNC_KERNEL_METADATA_AND_ADMISSION_PROFILE_CONTRACT.md`
   §4.5 records the axis, its keys, the appended column and the `v2` token.

## Live-Excel Re-Confirmation (COM probe, Excel 16.0 build 20326, 2026-09-15)

Retained locally as `.tmp/w110-argument-laziness-probe-results.csv` (the `.tmp/` convention the
other probe records use). Untaken branches / later candidates are not evaluated:
`=IF(TRUE,1,1/0)` -> `1`; `=IF(FALSE,1/0,2)` -> `2`; `=IFS(TRUE,1,TRUE,1/0)` -> `1`;
`=IFS(TRUE,1,1/0,2)` -> `1`; `=IFS(FALSE,1/0,TRUE,2)` -> `2`; `=CHOOSE(1,"a",1/0)` -> `a`;
`=CHOOSE(2,1/0,"b")` -> `b`; `=SWITCH(2,1,1/0,2,3)` -> `3`; `=SWITCH(1,1,7,1/0,3)` -> `7`;
`=SWITCH(3,1,1/0,2,1/0,9)` -> `9`; `=IFERROR(1,1/0)` -> `1`; `=IFERROR(1/0,5)` -> `5`;
`=IFNA(1,1/0)` -> `1`; `=IFNA(NA(),2)` -> `2`; `=IFNA(1/0,2)` -> `#DIV/0!`.
The logical folds are eager: `=OR(TRUE,1/0)`, `=OR(1/0,TRUE)`, `=AND(FALSE,1/0)`,
`=AND(1/0,FALSE)`, `=XOR(TRUE,1/0)`, `=XOR(1/0,TRUE)` all `#DIV/0!`; `=OR(TRUE,NA())` -> `#N/A`.

## A Finding OxFml Should Know About (not part of this handoff's ask)

OxFunc's `OR` and `AND` kernels return on the first deciding value before scanning the remaining
arguments, so `FUNC.OR(TRUE, #DIV/0!)` publishes `TRUE` and `FUNC.AND(FALSE, #DIV/0!)` publishes
`FALSE` where Excel publishes `#DIV/0!` (`XOR` is right). Filed as OxFunc bead `oxf-xvt5.14`
and catalog row G1-01. This does not change the axis (Excel is eager; the kernel is what is
wrong), and nothing in OxFml needs to change for it — OxFml already evaluates every argument of
an `Eager` function.

## What OxFml Can Now Do (the handoff's own plan)

1. Replace the `IF` / `IFERROR` / `IFS` / `CHOOSE` / `SWITCH` / `IFNA` arms of
   `CompiledFunctionSpecialForm::from_surface_name` with
   `function_call_target.argument_laziness_profile()`, keying `evaluate_if_call` on
   `BranchOnCondition`, `evaluate_iferror_call` on `FallbackOnError`, and
   `evaluate_lazy_selector_call` on `ConditionValuePairs` / `IndexedChoice` / `MatchedCase` (and,
   if OxFml prefers one path for both fallback forms, `FallbackOnError` too — the axis does not
   distinguish `IFERROR` from `IFNA`; which errors are caught is probed through the function's own
   dispatch, as the handoff already delegates).
2. Keep `LET` / `LAMBDA` / `_XLFN.SINGLE` as evaluator-owned special forms — they are not on the
   axis and never will be.
3. Keep the prepared-call trace vocabulary unchanged.
4. Retire the "interim, name-keyed" note on `CompiledFunctionSpecialForm` and mark this handoff
   acknowledged in OxFml's register.

## Acceptance Evidence (OxFunc, 2026-09-15)

`cargo test -p oxfunc_core --offline --no-fail-fast`: lib 1592 passed / 1 failed — the failure is
the pre-existing 1-ULP `finite_combinatoric_witnesses_match_excel_bits` (W109 lane, catalogued in
`oxf-xvt5.8`); the two other pre-existing catalogued reds (`oxfml_seam_integration::oxfunc_function_corpus_passes_through_adapter`,
`unary_numeric_equivalence_law::law3_overflowing_kernel_declares_non_pass_policy`) are unchanged
from the baseline taken before any edit (lib 1584/1 -> 1592/1: the eight new tests). `rustfmt
--edition 2024 --check` clean on every touched file; no clippy finding on any changed line (the
crate's pre-existing clippy reds are `oxf-xvt5.11`). `cargo check -p oxfunc_core --all-targets`
compiles `oxfml_core` (OxFunc's dev-dependency) against the change.

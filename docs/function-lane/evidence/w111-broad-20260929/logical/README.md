# Logical-fold text evidence

This packet records the AND/OR/XOR direct-text rule and its distinction from
array/reference text. It is evidence for G1-02 (`oxf-xvt5.15`), not a
whole-function parity claim.

Status: `execution_state=in_progress`, `scope_completeness=scope_partial`,
`target_completeness=target_partial`, `integration_completeness=partial`.
Open lanes: canonical bead/catalog reconciliation by the campaign owner;
whole-function semantic coverage beyond the text rule; orthogonal locale and
Excel-version sweeps.

## Captures

- `cases.jsonl` and `excel.jsonl`: 540 prepared-value cases captured by the
  campaign owner on 2026-09-29, 180 each for AND, OR, XOR. The retained
  `rollup.json` reports 540 exact typed matches with Excel 16.0 build 20430,
  workbook Compatibility Version 2. Inputs and results in this corpus are
  logical, text, errors, and arrays of those values; no floating-point tolerance
  enters the comparison. Generate the cases with
  `smart-fuzzer/tools/w111/gen_logical_text_20260929.py` and run them through
  `smart-fuzzer/tools/Run-ArraySupportTranche.ps1`.
- `spelling.csv`: 75 rows from a fresh run of
  `tools/w110-probe/run-w110-logical-text-spelling-probe.ps1`. It additionally
  records computed text, actual cell references, Unicode look-alikes, and
  repeated coerced spellings. `ISTEXT(E1)` / `ISTEXT(E2)` confirm the fixture
  cells contain text. The script records Excel version/build and capture time;
  it does not record workbook Compatibility Version.
- `precedence.csv`: 111 rows from a fresh run of
  `tools/w110-probe/run-w110-and-or-error-precedence-probe.ps1`, including
  argument-order error propagation before/after a deciding logical value.
  Its workbook Compatibility Version is also not recorded by that script.
- `manifest.json`: provenance for the 540-case runner capture, including the
  source revision and original run-directory locations. The retained files
  here are copies of those artifacts.

## Observed rule and alignment

Direct scalar text contributes a logical only when it is exactly TRUE or FALSE
under ASCII case folding. Whitespace is not trimmed. Numeric text and all other
direct text are ignored. Text from an array or reference is ignored even when
it spells TRUE or FALSE. If no logical/number contributed, the result is
`#VALUE!`. Ignored text does not hide a worksheet error; the first error in
argument order surfaces, including after a deciding AND/OR value.

The production Rust rule already existed in commit `6a015c7`, in
`coercion::parse_excel_logical_text` and
`functions::aggregate_common::and_argument_truth`; this packet does not change
that runtime code. It supplies previously missing dispatch-level replay tests
and corrects the stale Lean AND/OR direct-text arms. The new shared Lean
`LogicalFold` substrate uses the same ASCII-only text rule, and `XorFn` now has
an executable parity fold over prepared arguments. Generic Lean theorems show
that direct text cannot itself raise an error, array/reference text is ignored,
and worksheet errors propagate; concrete cases exercise the three folds.

`crates/oxfunc_core/src/functions/argument_laziness_golden.rs` binds every
retained JSON case to its captured Excel outcome by case ID and checks it
through `eval_surface_value_call`. It rejects duplicate IDs, missing outcomes,
formula/function mismatches, and changed per-function row counts. Separate
checks use a real reference-provider interface for E1/E2 and contrast that
origin with prepared scalar text returned by expressions. These tests exercise
OxFunc's prepared-value seam; expression evaluation remains OxFml's concern.

## Validation and limits

- `cargo test -p oxfunc_core --offline argument_laziness_golden -- --nocapture`:
  11 passed, including all 540 retained cases and the reference/computed-text
  contrasts.
- `lake build` in `formal/lean`: 494 jobs passed, including `LogicalFold`,
  `AndFn`, `OrFn`, and `XorFn`.
- `cargo test -p oxfunc_core --offline`: 1,629 library tests passed, four
  ignored, one failure in the already-catalogued
  `finite_combinatoric_witnesses_match_excel_bits` (bead `oxf-xvt5.8`), with
  `0.6846054400000001` versus `0.68460544`. That numeric kernel is outside
  this text-rule repair and was not changed here. This run stopped before
  integration targets.
- `cargo test -p oxfunc_core --offline --no-fail-fast`: all targets exercised;
  the same three failures recorded in `oxf-xvt5.8` remain: the library
  combinatoric witness above, `oxfunc_function_corpus_passes_through_adapter`,
  and `law3_overflowing_kernel_declares_non_pass_policy`. No logical-fold test
  failed. The seam-corpus diagnostics additionally show CHOOSE/SWITCH/IFS
  prepared-structure expectations differing from omitted unselected arguments;
  this packet does not change those evaluator fixtures. The full local log is
  `.tmp/w111-logical-core-all-targets-validation-20260929.txt` (not a retained
  oracle artifact).

No function metadata, argument-admission declaration, or FEC/F3E interface is
changed. Existing direct/reference origin semantics are exercised. No XLL host
path is exercised by this COM plus prepared-value packet, and it does not claim
XLL equivalence. The Lean number carrier remains the existing rational
prepared-value abstraction; this packet addresses text/origin/error behavior.

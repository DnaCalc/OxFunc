# HO-FN-018 - Explicit @ Operand Parser Follow-Up

Direction: `OxFunc -> OxFml`
Source repo/workset: `OxFunc/W100`
Target repo/workset: `OxFml formula parser/admission follow-up`
Filed: `2026-06-15`
Status: `filed`

## Purpose

Record the remaining W100 OxFunc-side seam blocker after the local `INDEX`
value-context fallback cleared the function-corpus mismatch.

`cargo test -p oxfunc_core --test oxfml_seam_integration` now reports
`37 passed; 1 failed`. The remaining failing test is
`w050_seam_scenarios_pass_from_oxfunc_side`, and all failing rows are explicit
`@` formulas rejected by syntax diagnostics before OxFunc function semantics are
entered.

## Observed Failing Rows

1. `B01`: `=@A1:A3` rejects with unexpected trailing token `Colon`.
2. `B02`: `=@A1:C1` rejects with unexpected trailing token `Colon`.
3. `B04`: `=@SEQUENCE(3)` rejects with unexpected trailing token `LParen`.
4. `B06`: `=@A1:B2` rejects with unexpected trailing token `Colon`.
5. `F04`: `=@OFFSET(A1,1,0,3,1)` rejects with unexpected trailing token `LParen`.

## Local Trace

The trace points to the OxFml parser path for a leading explicit `@`. The parser
special-case for `@` followed by an identifier consumes only the immediate member
token and returns a prefix expression before range and postfix parsing have a
chance to consume `:...` or `(...)`.

Expected parser behavior for the W100 admitted slice is that explicit `@`
wraps the full operand expression:

1. ranges such as `A1:A3`,
2. same-row/same-column ranges such as `A1:C1`,
3. dynamic-array function calls such as `SEQUENCE(3)`,
4. reference-returning function calls such as `OFFSET(...)`.

If parsing reached binding/evaluation, the existing implicit-intersection path
would be able to exercise the OxFunc-side seam. The present failure occurs
before that boundary.

## Requested OxFml Follow-Up

1. Adjust explicit-`@` parsing so the operand is parsed through the ordinary
   range/postfix/call expression path rather than stopping after a single
   identifier token.
2. Preserve existing `@` semantics for structured-reference member forms if
   that special case is still needed.
3. Rerun the W100-compatible fixture rows listed above from the OxFml side and
   the OxFunc side.
4. Reply with either an acknowledgement/landing reference or a narrower parser
   blocker if any row needs a different admission contract.

## OxFunc State

OxFunc blocker bead: `oxf-acdw.1.1`.

Validation after the local `INDEX` fallback:

1. `cargo test -p oxfunc_core functions::index::tests::` passes 18/18.
2. `cargo test -p oxfunc_core --test oxfml_seam_integration` reports
   `37 passed; 1 failed`; `oxfunc_function_corpus_passes_through_adapter`
   passes, and only the explicit-`@` W050 rows remain.

This handoff does not mark W100 integration confidence as green. W100 remains
`target_partial` / `integration_completeness: partial` until the explicit-`@`
rows parse and replay successfully, or a narrower downstream blocker is filed.

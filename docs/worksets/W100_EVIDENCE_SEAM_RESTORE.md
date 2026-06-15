# W100 Evidence Seam Restore

Status: `in_progress`

## Purpose

Restore confidence in the OxFunc/OxFml evidence seam after the June 2026 review
identified broken or stale cross-repo validation paths.

This packet records the OxFunc-local W100 checkpoint state. `.beads/` remains
the live blocker surface.

## Canonical Surfaces

1. `.beads/` epic `oxf-acdw`
2. `.beads/` task `oxf-acdw.1`
3. `.beads/` blocker `oxf-acdw.1.1`
4. `docs/handoffs/HO-FN-018_explicit_at_operand_parser_followup.md`
5. `docs/upstream/NOTES_FOR_OXFML.md`
6. `crates/oxfunc_core/tests/oxfml_seam_integration.rs`
7. `crates/oxfunc_core/tests/fixtures/w050_oxfunc_admitted_fixture_cases.json`
8. `crates/oxfunc_core/tests/fixtures/oxfunc_adapter_function_corpus.json`

## Current Checkpoint

2026-06-15:

1. `cargo test -p oxfunc_core --test oxfml_seam_integration` improved from
   `36 passed; 2 failed` to `37 passed; 1 failed`.
2. The `FN-INDEX-02` `=INDEX(A1:B3,2,2)` mismatch is no longer failing after
   the OxFunc-side value-context fallback for providers that can enumerate
   reference values but do not support `ReferenceSystemOperation::Transform`.
3. The remaining W100 failure is explicit `@` formula parsing/admission in
   OxFml before OxFunc semantics are entered.
4. `HO-FN-018` is filed and registered for the explicit-`@` operand parser
   follow-up.

## Validation Evidence

1. `cargo test -p oxfunc_core functions::index::tests::` passed 18/18.
2. `cargo test -p oxfunc_core --test oxfml_seam_integration` reports
   `37 passed; 1 failed`.
3. Full `cargo test -p oxfunc_core` was attempted after the targeted fix:
   core lib passed `1399 passed; 0 failed; 1 ignored`; non-W100 integration
   suites passed; the package still fails on the explicit-`@` W050 rows.

## Open Lanes

1. `oxf-acdw.1.1`: explicit `@` formulas rejected by OxFml syntax diagnostics.
2. OxFml acknowledgement or narrower downstream blocker for `HO-FN-018`.
3. W100 gate replay after the downstream parser/admission lane moves.

## Doctrine Axes

scope_completeness: `scope_partial`
target_completeness: `target_partial`
integration_completeness: `partial`
open_lanes: `[explicit_at_parser_operand, HO-FN-018_acknowledgement, post_fix_W100_replay]`

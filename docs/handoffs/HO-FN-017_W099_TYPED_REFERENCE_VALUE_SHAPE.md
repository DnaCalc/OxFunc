# HO-FN-017 - W099 Typed Reference Value Shape

handoff_id: `HO-FN-017`

direction: `OxFunc->OxFml`

source_workset: `W099`

status: `filed`

filed_date: `2026-06-01`

## Summary

W099-002 changes `ReferenceLike` from a universal `kind`/`target` carrier into a typed reference payload with explicit `system`, `identity`, and `display` fields. The old `kind` and `target` fields remain as W099 migration-only mirrors for legacy call sites, but direct struct literals outside `oxfunc_value_types` must move to constructors so the typed payload is populated.

## OxFml Impact

OxFml evaluator code that creates `ReferenceLike` values should use `ReferenceLike::new(kind, target)` for textual Excel-grid references until a later W099 bead exposes provider-native typed construction at the evaluator boundary. This preserves the current textual behavior while ensuring `system`, `identity`, and `display` are populated consistently.

The W099-002 local validation found and repaired the immediate OxFml compile surface in:

1. `crates/oxfml_core/src/eval/mod.rs`
2. `crates/oxfml_core/src/host/mod.rs`

## Open Follow-Through

1. Replace `ReferenceLike::new` textual construction with provider/reference-system constructors when W099-003 and later provider beads land.
2. Keep OxFml's structured-reference display strings as temporary textual identities until the reference-system provider can carry opaque or composite identities directly.
3. Do not treat this handoff as cross-repo seam completion; it is a compatibility landing record for the first typed-value foundation bead.

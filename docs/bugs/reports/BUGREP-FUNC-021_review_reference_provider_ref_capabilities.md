# BUGREP-FUNC-021: Review finding on ReferenceSystemProvider &T capabilities

## Intake
- **Report id**: `BUGREP-FUNC-021`
- **Filed**: 2026-06-15
- **Source channel**: local review follow-up
- **Reporter/source**: June 2026 OxFunc cleanup review
- **Reported against ref**: `w100-w102-cleanup-pass working tree`
- **Reported against kind**: unknown
- **Reported against note**: static review of resolver trait forwarding
- **Canonical bug id**: `BUG-FUNC-036`
- **Status**: triaged

## Observed Symptom
The blanket `ReferenceSystemProvider for &T` impl forwarded most methods but
not `capabilities()`, so reference-bound providers could silently fall back to
the permissive default capability profile.

## Reproduction
1. See `BUG-FUNC-036`.
2. Expected: `&T` and `&&T` expose the inner provider capabilities.
3. Actual before the working-tree patch: capability checks used
   `permissive_local()`.

## Initial Ownership Read
- **Initial classification**: OxFunc-owned bug
- **Reason**: the bug is in OxFunc resolver trait forwarding.

## Links
1. `docs/bugs/streams/BUG-FUNC-036_reference_provider_ref_wrapper_drops_capabilities.md`
2. `crates/oxfunc_core/src/resolver.rs`

## Triage Notes
No Excel replay is required; this is a deterministic Rust dispatch gap.

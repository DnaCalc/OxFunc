# BUG-FUNC-036: Blanket &T ReferenceSystemProvider impl silently drops capabilities()

## Summary
- **Bug id**: `BUG-FUNC-036`
- **Opened**: `2026-06-11`
- **Status**: `closed` (fix landed in `7a0003f` on main; static fix, no Excel evidence required)
- **Owner workset**: `W102A` (no-probe structural fixes)

## Source Refs
- **Reported against ref**: working tree at review `2026-06-10`
- **Reproduced on ref**: code review; no fuzzer lane required (static structural finding)
- **Introduced in ref**: unknown; present since the blanket `&T` impl was added
- **Fixed in ref**: `7a0003f` (landed on main 2026-06-18; full lib suite 1399 passed)
- **Ref notes**: static analysis; no Excel differential needed (pure Rust trait dispatch).

## Ownership And Root Cause
- **Ownership class**: `OxFunc-owned bug`
- **Root cause class**: `spec_mismatch`
- **Root cause summary**: The blanket impl `ReferenceSystemProvider for &T` forwarded
  `dereference`, `enumerate_values`, `resolve_text`, `facts`, `transform_reference`,
  `compose_references`, and `caller_context` to `(**self)`, but omitted `capabilities()`.
  As a result any call site holding a `&ConcreteProvider` (rather than `ConcreteProvider`
  directly) fell through to the trait default `permissive_local()`, masking the concrete
  provider's actual capability policy. The primary exposure path is the INDIRECT dispatch
  arm: it builds a `FunctionExecutionContextRef` over an already-reference-bound provider,
  so `resolve_eval_value`'s `ensure_reference_resolution_allowed` gate checked
  `permissive_local()` instead of the host's settings. A host disabling
  `allow_eval_time_deref` or `allow_three_d_refs` had those restrictions silently bypassed;
  a host enabling `allow_external_refs` had external references wrongly denied.

## Reproduction
The bug is structural and deterministic. The following function always returns the default
`permissive_local()` regardless of what the inner provider implements:

```rust
fn call_via_ref<T: ReferenceSystemProvider>(p: &T) -> ReferenceSystemCapabilities {
    p.capabilities()  // before fix: always permissive_local()
}
```

## Fix Plan
Added `fn capabilities(&self) -> ReferenceSystemCapabilities { (**self).capabilities() }`
to the blanket `impl<T: ReferenceSystemProvider + ?Sized> ReferenceSystemProvider for &T`
block in `crates/oxfunc_core/src/resolver.rs`. The `&&T` case is covered transitively
because `&&T` implements `&T` which now delegates correctly.

## Validation
- Rust unit test `resolver::tests::ref_wrapper_forwards_capabilities_single_and_double_indirection`:
  asserts `&MockResolver` and `&&MockResolver` both return the concrete restrictive
  capabilities; asserts `resolve_eval_value` denies `ThreeD` through `&T` when
  `allow_three_d_refs = false`; asserts `resolve_eval_value` returns
  `EvalTimeDerefNotAllowed` through `&T` when `allow_eval_time_deref = false`.
- Full resolver test suite: `20 passed; 0 failed` (`cargo test -p oxfunc_core resolver`).

## Similar-Risk Scan
- The `Box<T>` case is not affected: `Box<T>` derefs to `T` directly, not through the
  `&T` blanket impl, so calls dispatch to the concrete `T` implementation.
- No other blanket impls in resolver.rs omit method forwarding; the `NullReferenceSystemProvider`
  does not override `capabilities()` and therefore inherits `permissive_local()`, which is
  the correct and intended policy for the null provider.
- No `Arc<T>` or `Rc<T>` blanket impls exist; if added they will need the same fix.

## Closure Checklist
- [x] fix landed (commit `7a0003f`, on main)
- [x] validation recorded
- [x] root cause recorded
- [x] similar-risk scan recorded
- [ ] spec/matrix/contract updated if required
- [ ] handoff filed if required
- [x] linked reports updated

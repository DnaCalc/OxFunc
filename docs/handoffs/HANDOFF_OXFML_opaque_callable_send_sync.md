# HO-FN-020 - `OpaqueCallable: Send + Sync` and the `send-values` feature (W110-2 / campaign W3.1)

Direction: `OxFunc -> OxFml` (primary; four source lines), with a note each for `OxCalc` and `DnaTreeCalc`
Source repo/workset: `OxFunc/W110` (bead `oxf-xvt5.2`)
Target repo/workset: `OxFml/crates/oxfml_core/src/eval/mod.rs` callable construction sites
Filed: `2026-09-15`
Status: `filed`

## What OxFunc now ships

`crates/oxfunc_value_types` (re-exported as `oxfunc_core::value::*`) has a cargo
feature `send-values`, **off by default**, and `oxfunc_core` passes it through
(`oxfunc_core/send-values`). The feature changes exactly three things:

```rust
// crates/oxfunc_value_types/src/lib.rs
#[cfg(feature = "send-values")]      pub type Shared<T> = std::sync::Arc<T>;
#[cfg(not(feature = "send-values"))] pub type Shared<T> = std::rc::Rc<T>;

pub struct CalcValue     { pub core: CoreValue, pub rich: Option<Shared<RichValue>> }   // was Option<Rc<RichValue>>
pub struct CallableValue { pub arity, pub summary, pub handle: Shared<dyn OpaqueCallable> } // was Rc<dyn OpaqueCallable>

#[cfg(feature = "send-values")]      pub trait OpaqueCallable: Debug + Send + Sync + 'static { fn as_any(&self) -> &dyn Any; }
#[cfg(not(feature = "send-values"))] pub trait OpaqueCallable: Debug + 'static               { fn as_any(&self) -> &dyn Any; }
```

With the feature off, `Shared<T>` **is** `Rc<T>` and nothing downstream changes:
`cargo check --offline` in OxFml, OxCalc and the six DnaTreeCalc crates that
consume the engine passed unchanged on 2026-09-14/15 (see "Evidence").

With the feature on, `oxfunc_value_types::send_values_audit` proves at compile
time (`const _: () = assert_send_sync::<T>()`) that `CalcValue`, `CoreValue`,
`CalcArray`, `ReferenceLike`, `RichValue`, `RichObjectValue`, `PresentationValue`,
`ErrorMetadataValue` and `CallableValue` are all `Send + Sync`; a run-time test
moves a callable-carrying `CalcValue` to another thread and compares handle
identity after the join.

## What blocks the feature from defaulting on: four lines in OxFml, nothing else

The bead asked whether OxFml's LAMBDA closures capture evaluation state that is
`!Send`. **They do not.** Both `OpaqueCallable` implementers in OxFml carry plain
data:

| Implementer (`eval/mod.rs`) | Fields | `Send + Sync`? |
|---|---|---|
| `OxFmlCallableBinding` (`:535`) | `String`, `String`, `CallableDefinedNameBinding` (`BoundExpr` + `BTreeMap<String, DefinedNameBinding>` + strings), `Vec<CallableCapturedRef>` | **yes** once `CalcValue` is |
| `OxFmlRuntimeCallableHandle` (`:566`) | `callable_token: String` | **yes** |

The live closure state (`HelperBinding::Lambda { params: Rc<[LambdaParam]>,
body: Rc<CompiledExpr>, closure: HelperBindingFrame }` with its
`Rc<Vec<RefCell<..>>>` slots) stays inside the evaluator's frame; the value only
carries the token that the evaluator resolves. That design is what makes the
bound hold — nothing needs to change in it.

Compiler proof, not reading: `cargo test -p oxfunc_core --features send-values`
compiles `oxfml_core` (a dev-dependency) with the feature unified on and fails
with **exactly three E0308 errors and no E0277**:

```
error[E0308]: mismatched types  --> OxFml\crates\oxfml_core\src\eval\mod.rs:548:17
   expected `Arc<dyn OpaqueCallable>`, found `Rc<OxFmlCallableBinding>`
error[E0308]: mismatched types  --> OxFml\crates\oxfml_core\src\eval\mod.rs:1061:21
   expected `Arc<dyn OpaqueCallable>`, found `Rc<OxFmlRuntimeCallableHandle>`
error[E0308]: mismatched types  --> OxFml\crates\oxfml_core\src\eval\mod.rs:1088:21
   expected `Arc<dyn OpaqueCallable>`, found `Rc<OxFmlRuntimeCallableHandle>`
error: could not compile `oxfml_core` (lib) due to 3 previous errors
```

rustc reports an E0277 (`cannot be sent between threads safely`) at an `impl`
site whose type fails a `Send + Sync` supertrait even when E0308s are present
elsewhere (verified with a scratch probe on 2026-09-14), so the absence of E0277
at `:535` and `:566` is the evidence that the bound holds.

### The change OxFml has to make (four substitutions, compatible with both feature states)

Replace `Rc::new(` with `oxfunc_core::value::Shared::new(` at the `handle:`
field of each `OxCallableValue` literal — the alias resolves to `Rc` today and to
`Arc` under the feature, so OxFml then compiles in both states and OxFunc can flip
the default whenever the program wants it:

| File | Line (2026-09-14) | Today | After |
|---|---|---|---|
| `crates/oxfml_core/src/eval/mod.rs` | 548 | `handle: Rc::new(OxFmlCallableBinding { .. })` | `handle: Shared::new(OxFmlCallableBinding { .. })` |
| `crates/oxfml_core/src/eval/mod.rs` | 1061 | `handle: Rc::new(OxFmlRuntimeCallableHandle { .. })` | `handle: Shared::new(OxFmlRuntimeCallableHandle { .. })` |
| `crates/oxfml_core/src/eval/mod.rs` | 1088 | `handle: Rc::new(OxFmlRuntimeCallableHandle { .. })` | `handle: Shared::new(OxFmlRuntimeCallableHandle { .. })` |
| `crates/oxfml_core/tests/authored_input_tests.rs` | 290 | `handle: std::rc::Rc::new(TestCallable)` | `handle: oxfunc_core::value::Shared::new(TestCallable)` |

`Rc` stays in use at every other `eval/mod.rs` site (`HelperBinding`,
`HelperBindingFrame`, `EvaluationFrameState`, compiled-body caches) — those are
evaluator internals, not value payloads, and are out of this handoff's scope.

Verified on a scratch copy of `oxfml_core` (scratchpad, real repo untouched)
with exactly those four substitutions and `oxfunc_core = { .., features =
["send-values"] }`:

- `cargo check --offline --all-targets` — `Finished` (lib + all test targets).
- `cargo test --offline` — 33 targets, **316 passed, 3 failed**; the 3 are
  `oxfunc_catalog_snapshot_export_tests::w044_export_*`, which open OxFunc's W044
  export CSV by a path relative to the crate directory and cannot find it from
  the scratchpad copy (`Io(NotFound)`); they are unrelated to the feature.

## OxCalc: nothing to change; the Send audit can assert the value tables

`OxCalc/src/oxcalc-core/src/grid/machine.rs` `concurrency_prep_send_audit`
(`#[cfg(test)]`, ~`:28296-28350`) today asserts the structural coordinator types
`Send` and **documents** that `GridCalcRefWorkbook`, `GridCalcRefSheet` and
`GridOptimizedValuation` are not, because of `Rc` inside `CalcValue`. Against a
scratch copy of `oxcalc-core` wired to the patched OxFml copy with `send-values`
on:

- `cargo check --offline --all-targets` — `Finished`; OxCalc's own code has no
  `Rc`-typed use of `CalcValue::rich` or `CallableValue::handle`
  (its only `Rc` is `TreecalcInvocationHostSlot`, an engine-internal slot).
- Adding `const _: () = assert_send::<GridCalcRefWorkbook>();`,
  `..::<GridCalcRefSheet>()`, `..::<GridOptimizedValuation>()` and
  `..::<oxfunc_core::value::CalcValue>()` to that audit module compiles; a
  control line `assert_send::<std::rc::Rc<u8>>()` in the same module fails with
  E0277 as it must, and is the only error. So once OxFml lands the four lines and
  the feature is on, the audit's "recorded upstream blocker" paragraph can become
  four live assertions.
- `OxCalc/Cargo.toml` allows `clippy::arc_with_non_send_sync` "until the W053
  Rc->Arc value-model migration"; that allowance becomes removable at the same
  time.

## DnaTreeCalc: nothing to change for the feature; `!Send` sessions have a second cause

`cargo check --offline` on the six consumer crates (`dnacalc-host-core`,
`dnacalc-formula-ux-core`, `dnacalc-extension-host-core`, `dnatreecalc-host`,
`dnacalc-bench-host`, `dnacalc-conform`) passed in the default state; the three
`Rc` uses the bead counted in DnaTreeCalc are `Rc<RefCell<..>>` browser-closure
holders (`browser_file_io.rs`, `home_shell.rs`, `grid_canvas.rs`), not value
handles. Not verified in the ON state (would need the whole workspace copied to a
scratch root; OxCalc's clean result plus zero value-handle sites make a further
blocker unlikely, but that is an inference, not a compile).

Note for the DnaTreeCalc owner: `dnacalc-host-core/src/lib.rs` (~`:870`) records
that `OxCalcDocumentContext` is `!Send + !Sync` for **two** reasons — the
`Rc<RichValue>` this feature removes, and a `NodeRef<Owned, ..>` handle in the
workspace-state map that is `!Sync` on its own. Turning the feature on does not
by itself make the sessions `Send`; that second cause is DnaTreeCalc's.

## Exit condition for this handoff

1. OxFml lands the four `Shared::new` substitutions (compiles in both states).
2. OxFunc flips `send-values` on by default (or the program decides to keep it
   opt-in) — a one-line `Cargo.toml` change in `oxfunc_value_types` plus the
   passthrough default in `oxfunc_core`.
3. OxCalc replaces the audit's negative paragraph with the four assertions.

Until 1 lands, the OxFunc side of `oxf-xvt5.2` is reported as *landed behind a
feature, default off; downstream handoff open* — not as `CalcValue: Send`.

## Evidence (run 2026-09-14/15, OxFunc working tree at the oxf-xvt5.2 commit)

- `cargo test -p oxfunc_value_types --offline` — 26 passed.
- `cargo test -p oxfunc_value_types --offline --features send-values` — 27
  passed (adds `send_values_audit::tests::calc_value_with_callable_handle_crosses_a_thread_boundary`).
- `cargo test -p oxfunc_value_types -p oxfunc_core --offline --no-fail-fast` —
  lib 1584 passed / 1 failed (pre-existing `finite_combinatoric_witnesses_match_excel_bits`),
  plus the two other pre-existing reds catalogued in `oxf-xvt5.8`; identical to
  the baseline taken before any edit.
- `cargo check -p oxfunc_core --offline --features send-values` — `Finished`.
- `cargo check --offline` in `../OxFml`, `../OxCalc`, and the six DnaTreeCalc
  consumers — `Finished` (default state). The whole-workspace DnaTreeCalc check
  stops only at `dnacalc-bench-desktop`'s `tauri::generate_context!` (missing
  `target/onecalc-preview` frontend dir), which is environmental.
- Arc clone cost on the evaluation hot path: **not measured**, and this handoff
  makes no claim about it either way; the default-off feature keeps the `Rc`
  path in production until someone measures.

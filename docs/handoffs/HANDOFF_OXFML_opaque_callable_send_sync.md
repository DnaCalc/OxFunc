# HO-FN-020 - `OpaqueCallable: Send + Sync` and the `send-values` feature (W110-2 / campaign W3.1)

Direction: `OxFunc -> OxFml` (primary; four source lines), with a note each for `OxCalc` and `DnaTreeCalc`
Source repo/workset: `OxFunc/W110` (bead `oxf-xvt5.2`)
Target repo/workset: `OxFml/crates/oxfml_core/src/eval/mod.rs` callable construction sites
Filed: `2026-09-15`
Status: `acknowledged` — landed on both sides 2026-09-15 (OxFml `fml-kt8.10`, commit `1f19bf7`; OxFunc default flip `oxf-xvt5.10`). See "Default flip" at the end.

## What OxFunc now ships

`crates/oxfunc_value_types` (re-exported as `oxfunc_core::value::*`) has a cargo
feature `send-values`, **off by default as filed; on by default since
2026-09-15** (see "Default flip"), and `oxfunc_core` passes it through
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

## Default flip (2026-09-15, `oxf-xvt5.10`)

### What landed on the OxFml side

OxFml bead `fml-kt8.10` (commit `1f19bf7`, closed in `29b009d`) applied the
four substitutions. Read on 2026-09-15 in `../OxFml` at that commit: the
`handle:` field at `eval/mod.rs:548`, `:1061`, `:1088` (and a further
test-side literal at `:7434`) and `tests/authored_input_tests.rs:290` build
through `oxfunc_core::value::Shared::new`; no `Rc::new(` handle site remains
in OxFml, OxCalc, or DnaTreeCalc (grep over `OxFml/crates`, `OxCalc/src`,
`DnaTreeCalc/src` for `handle: Rc::new`, `Rc<dyn OpaqueCallable>`,
`Rc<RichValue>`: the only hits are two comment lines in OxCalc's
`grid/machine.rs` Send audit). OxFml also added an opt-in passthrough feature
`oxfml_core/send-values` and a `#[cfg(all(test, feature = "send-values"))]`
module `eval::send_values_audit`; making that audit run under OxFml's default
test command is OxFml's `fml-kt8.17`.

### What changed in OxFunc

`crates/oxfunc_value_types/Cargo.toml` and `crates/oxfunc_core/Cargo.toml`
now declare `default = ["send-values"]`. Nothing else changed shape: the
`Shared<T>` alias, the `OpaqueCallable` supertraits, and `send_values_audit`
are the same `cfg` seam as filed, so every consumer that builds `oxfunc_core`
with default features now gets `Shared<T> = Arc<T>`, `OpaqueCallable: Send +
Sync`, and the compile-time `CalcValue: Send + Sync` proof, without editing
its own manifest.

### The `Rc` arm is kept for one release; retirement condition

`--no-default-features` on `oxfunc_value_types` / `oxfunc_core` still selects
the `Rc` arm (verified: `cargo check -p oxfunc_value_types -p oxfunc_core
--lib --no-default-features --offline` Finished; `cargo test -p
oxfunc_value_types --no-default-features --offline` 26 passed). It is kept so
a consumer that still needs a thread-bound value model has a named opt-out
while it migrates, not because any consumer is known to use it.

The `cfg` seam and the `Rc` arm are retired — `Shared<T> = Arc<T>`
unconditionally, `OpaqueCallable: Send + Sync` unconditionally,
`send_values_audit` unconditional, feature `send-values` deleted from both
manifests and from `oxfml_core`'s passthrough — when **all** of these hold:

1. OxFml `fml-kt8.17` has landed (its audit runs under the default test
   command and its `send-values` passthrough feature is gone or a no-op).
2. OxCalc has replaced the negative paragraph in
   `grid/machine.rs` `concurrency_prep_send_audit` with the four live
   `assert_send` lines named below and dropped the
   `clippy::arc_with_non_send_sync` allowance in `OxCalc/Cargo.toml`.
3. One `cargo check --offline` each in `../OxFml`, `../OxCalc`, and the six
   DnaTreeCalc consumer crates has passed with default features **and** no
   manifest in those repos names `oxfunc_core`/`oxfunc_value_types` with
   `default-features = false` or `--no-default-features` in any script
   (grep the manifests and `scripts/` directories at retirement time).

The retirement is a separate OxFunc bead, `oxf-xvt5.12` (child of
`oxf-xvt5`), not part of the flip commit.

### Evidence for the flip (run 2026-09-15 in the OxFunc working tree, dirty with the owner's W109 WIP, unchanged by this bead)

- Baseline before any edit, default OFF:
  `cargo test -p oxfunc_value_types -p oxfunc_core --offline --no-fail-fast`
  — value_types **26 passed**; core lib **1584 passed / 1 failed**
  (`finite_combinatoric_witnesses_match_excel_bits`, the pre-existing 1-ULP
  red); `oxfml_seam_integration` 37 / 1
  (`oxfunc_function_corpus_passes_through_adapter`);
  `unary_numeric_equivalence_law` 5 / 1
  (`law3_overflowing_kernel_declares_non_pass_policy`); every other target
  green. All three reds are the ones catalogued on `oxf-xvt5.2` /
  `oxf-xvt5.8`.
- After the flip, default ON, same command — value_types **27 passed** (adds
  `send_values_audit::tests::calc_value_with_callable_handle_crosses_a_thread_boundary`);
  core lib **1584 passed / 1 failed**; the same two integration reds; nothing
  else changed. `oxfml_core` compiled as `oxfunc_core`'s dev-dependency with
  the feature unified ON — the in-repo confirmation that HO-FN-020 landed.
- Opt-out arm: `cargo check -p oxfunc_value_types -p oxfunc_core --lib
  --no-default-features --offline` Finished; `cargo test -p
  oxfunc_value_types --no-default-features --offline` 26 passed.
- `rustfmt --edition 2024 --check crates/oxfunc_value_types/src/lib.rs` clean;
  `cargo clippy -p oxfunc_value_types --offline` no findings (the one
  `--all-targets` warning at a test line is pre-existing and untouched);
  `oxfunc_core` lib clippy reds are the pre-existing W109/`oxf-xvt5.11` set,
  none on a line this bead changed.
- Downstream, read-only, one cargo at a time, default state (= ON), each run
  only after `tasklist` showed no `cargo.exe`/`rustc.exe` anywhere:
  - `../OxFml`: `cargo check --offline` Finished; `cargo check --offline
    --all-targets` Finished.
  - `../OxCalc`: `cargo check --offline` Finished and `cargo check --offline
    --all-targets` Finished, every crate Fresh — OxCalc's own agent had
    already rebuilt the workspace against the flipped manifests
    (`target/debug/.fingerprint/oxfunc_value_types-*/lib-oxfunc_value_types.json`
    records `features: ["default", "send-values"]`, and
    `--message-format=json` reports `oxfunc_value_types` and `oxfunc_core`
    resolved with `['default', 'send-values']`, `fresh: true`). That run is
    the live check the note above anticipated; it did not need any OxCalc
    source change.
  - `../DnaTreeCalc`: `cargo check --offline -p dnacalc-host-core -p
    dnacalc-formula-ux-core -p dnacalc-extension-host-core -p
    dnatreecalc-host -p dnacalc-bench-host -p dnacalc-conform` Finished
    (35.6 s), `oxfunc_value_types`/`oxfunc_core` resolved with
    `['default', 'send-values']`.
  - `../OxXlPlay` (a direct `oxfunc_value_types` consumer the bead did not
    list; no `Rc`/`OpaqueCallable` use): `cargo check --offline` Finished.

### Notes for the sibling repos now that the default is on

- **OxCalc**: the four `assert_send::<GridCalcRefWorkbook>()`,
  `::<GridCalcRefSheet>()`, `::<GridOptimizedValuation>()`,
  `::<oxfunc_core::value::CalcValue>()` lines in
  `grid/machine.rs` `concurrency_prep_send_audit` now compile under default
  features (verified on a scratch copy 2026-09-15 with the feature on; the
  default flip makes that the ordinary build). The
  `clippy::arc_with_non_send_sync` allowance in `OxCalc/Cargo.toml` can go.
- **DnaTreeCalc**: nothing to change for the feature. `OxCalcDocumentContext`
  stays `!Sync` for its own `NodeRef<Owned, ..>` reason (see above).
- **Arc clone cost on the evaluation hot path**: still **not measured**; this
  flip makes no claim about it. If the program wants a number, OxFml's
  `fml-kt8.17` names the W075 perf fixture as the instrument.


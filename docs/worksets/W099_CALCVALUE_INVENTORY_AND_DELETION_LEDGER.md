# W099 CalcValue Inventory And Deletion Ledger

Status: `inventory_ready_for_W099_002`
Bead: `oxf-im4m.1`

## 1. Purpose

This artifact is the W099 start map for the CalcValue code rework. It records the
initial source scan, current owner decisions, deletion routes, and first code
batch boundary before broad call-site migration begins.

This is not a completion record. W099 remains `scope_partial`, `target_partial`,
and `integration_partial` until the terminal source audit proves that no
unowned legacy value carrier, old reference-provider path, bridge, shim,
side-car, or migration adapter remains.

Machine-readable occurrence ledger:
1. `docs/worksets/W099_CALCVALUE_OCCURRENCE_LEDGER.csv`
2. one row per scanned surface/file pair, plus a zero-count row for `HOST_REF_`;
3. each row carries match count, final owner, deletion batch, deletion route, and
   audit notes;
4. every counted occurrence in a row inherits that row's deletion route for
   W099 planning purposes, with W099 implementation beads responsible for
   shrinking the row counts as their substrate migrates.

Reviewed inbound observations:
1. `../OxFml/docs/upstream/NOTES_FOR_OXFUNC.md` was reviewed at W099 start.
2. Relevant live constraints are callable-slot carriage, prepared argument/result
   distinctions, direct-array versus reference-visible argument preservation,
   rich return-surface preservation for `IMAGE` / `HYPERLINK`, and current
   reference/multi-area seam expectations.
3. W099 supersedes the older `ReferenceKind` / `ReferenceLike.target` endpoint
   with the W098 typed reference payload and `ReferenceSystemProvider`, but must
   preserve the observable distinctions OxFml depends on during the migration.

## 2. Initial Scan

Scan command:

```powershell
rg -n "\b(EvalValue|CallArgValue|PreparedArgValue|EvalArray|ArrayCellValue|ExtendedValue|LambdaValue|ReferenceLike|ReferenceKind|ReferenceResolver|ReferenceTextResolver|resolve_eval_value|resolve_reference_values|HOST_REF_)\b|\.target\b" crates -g "*.rs"
```

Count command:

```powershell
$terms = 'EvalValue','CallArgValue','PreparedArgValue','EvalArray','ArrayCellValue','ExtendedValue','LambdaValue','ReferenceLike','ReferenceKind','ReferenceResolver','ReferenceTextResolver','resolve_eval_value','resolve_reference_values','HOST_REF_'
foreach ($t in $terms) {
  $m = rg -n --glob '*.rs' $t crates 2>$null
  $lines = @($m).Count
  $files = @($m | ForEach-Object { ($_ -split ':',2)[0] } | Sort-Object -Unique).Count
  "${t},${lines},${files}"
}
$m2 = rg -n --glob '*.rs' '\.target\b' crates 2>$null
"ReferenceLike.target_or_any_dot_target,$(@($m2).Count),$(@($m2 | ForEach-Object { ($_ -split ':',2)[0] } | Sort-Object -Unique).Count)"
```

Initial counts:

| Legacy surface | Matches | Files |
| --- | ---: | ---: |
| `EvalValue` | 5244 | 252 |
| `CallArgValue` | 3056 | 239 |
| `PreparedArgValue` | 1332 | 105 |
| `EvalArray` | 878 | 102 |
| `ArrayCellValue` | 3004 | 103 |
| `ExtendedValue` | 30 | 6 |
| `LambdaValue` | 55 | 8 |
| `ReferenceLike` | 536 | 135 |
| `ReferenceKind` | 296 | 68 |
| `ReferenceResolver` | 1280 | 233 |
| `ReferenceTextResolver` | 21 | 4 |
| `resolve_eval_value` | 49 | 17 |
| `resolve_reference_values` | 29 | 13 |
| `HOST_REF_` | 0 | 0 |
| `.target` | 201 | 130 |

Occurrence-ledger row counts:

| Surface | Ledger rows | Matches |
| --- | ---: | ---: |
| `ArrayCellValue` | 103 | 3004 |
| `CallArgValue` | 239 | 3056 |
| `EvalArray` | 102 | 878 |
| `EvalValue` | 252 | 5244 |
| `ExtendedValue` | 6 | 30 |
| `HOST_REF_` | 1 | 0 |
| `LambdaValue` | 8 | 55 |
| `PreparedArgValue` | 105 | 1332 |
| `ReferenceKind` | 68 | 296 |
| `ReferenceLike` | 135 | 536 |
| `ReferenceLike.target_or_any_dot_target` | 130 | 201 |
| `ReferenceResolver` | 233 | 1280 |
| `ReferenceTextResolver` | 4 | 21 |
| `resolve_eval_value` | 17 | 49 |
| `resolve_reference_values` | 13 | 29 |

Definition anchors:

| Surface | Definition |
| --- | --- |
| `ReferenceKind` | `crates/oxfunc_value_types/src/lib.rs` |
| `ReferenceLike` | `crates/oxfunc_value_types/src/lib.rs` |
| `ArrayCellValue` | `crates/oxfunc_value_types/src/lib.rs` |
| `EvalArray` | `crates/oxfunc_value_types/src/lib.rs` |
| `LambdaValue` | `crates/oxfunc_value_types/src/lib.rs` |
| `EvalValue` | `crates/oxfunc_value_types/src/lib.rs` |
| `CallArgValue` | `crates/oxfunc_value_types/src/lib.rs` |
| `ExtendedValue` | `crates/oxfunc_value_types/src/lib.rs` |
| `PreparedArgValue` | `crates/oxfunc_core/src/functions/adapters.rs` |
| `FunctionExecutionContextBundle` | `crates/oxfunc_core/src/function_call.rs` |
| `FunctionExecutionContextRef` | `crates/oxfunc_core/src/function_call.rs` |
| `ReferenceTextResolver` | `crates/oxfunc_core/src/resolver.rs` |
| `ReferenceResolver` | `crates/oxfunc_core/src/resolver.rs` |

## 3. Churn Concentration

Largest hit clusters from the first scan:

| Surface | Primary files |
| --- | --- |
| `EvalValue` | `surface_dispatch.rs`, `xlookup.rs`, `callable_helpers.rs`, `dynamic_array_reshape_family.rs`, `criteria_family.rs`, `index.rs`, `text_scalar_misc.rs`, `adapters.rs` |
| `CallArgValue` | `surface_dispatch.rs`, `xlookup.rs`, `index.rs`, `match_fn.rs`, `subtotal_aggregate_family.rs`, `operator_compare_concat_family.rs`, `sumproduct_family.rs`, `criteria_family.rs` |
| `PreparedArgValue` | `callable_helpers.rs`, `adapters.rs`, `xmatch.rs`, `dynamic_array_reshape_family.rs`, `aggregate_common.rs`, `surface_dispatch.rs`, `group_pivot_common.rs`, `choose_ifs_family.rs` |
| `EvalArray` | `surface_dispatch.rs`, `callable_helpers.rs`, `xlookup.rs`, `dynamic_array_reshape_family.rs`, `index.rs`, `pivotby_fn.rs`, `criteria_family.rs`, `groupby_fn.rs` |
| `ArrayCellValue` | `surface_dispatch.rs`, `dynamic_array_reshape_family.rs`, `xlookup.rs`, `callable_helpers.rs`, `criteria_family.rs`, `pivotby_fn.rs`, `index.rs`, `groupby_fn.rs` |
| `ExtendedValue` | `image_fn.rs`, `surface_dispatch.rs`, `hyperlink_fn.rs`, `now_fn.rs`, `today_fn.rs`, `oxfunc_value_types/src/lib.rs` |
| `LambdaValue` | `callable_helpers.rs`, `surface_dispatch.rs`, `groupby_fn.rs`, `pivotby_fn.rs`, `function_call.rs`, `callable_stage1_prepared.rs`, `group_pivot_common.rs`, `oxfunc_value_types/src/lib.rs` |
| `ReferenceResolver` | `surface_dispatch.rs`, `complex_family.rs`, `chi_f_t_family.rs`, `financial_time_value_family.rs`, `database_family.rs`, `dynamic_array_reshape_family.rs`, `function_call.rs`, `statistical_tests_family.rs` |
| `ReferenceTextResolver` | `function_call.rs`, `indirect.rs`, `surface_dispatch.rs`, `resolver.rs` |
| `ReferenceLike` / `ReferenceKind` | `resolver.rs`, `operator_reference_family.rs`, `index.rs`, `adapters.rs`, `offset.rs`, `op_spill_ref.rs`, `op_implicit_intersection.rs`, structured-table tests |

## 4. Owner And Deletion Routes

| Current surface | Final owner | Deletion route |
| --- | --- | --- |
| `EvalValue` | `CalcValue` / `CoreValue` | W099-008 and W099-012 retire dispatch and kernel return/input uses; W099-015 deletes the type. |
| `CallArgValue` | `CalcValue` | W099-005 maps all boundary arguments to `CalcValue`, including `CoreValue::Missing`, `CoreValue::Empty`, and `CoreValue::Reference`; W099-015 deletes the type. |
| `PreparedArgValue` | `CalcValue` plus transient local preparation facts | W099-007 removes it as a value carrier. Any retained preparation fact must be non-value metadata scoped to preparation. |
| `EvalArray` | `CalcArray` | W099-006 ports array storage, row-major iteration, spill helpers, and dynamic-array helpers; W099-015 deletes the type. |
| `ArrayCellValue` | `CalcValue` cells inside `CalcArray` | W099-006 ports empty/error-cell policy to `CalcValue.core`; W099-015 deletes the type. |
| `ExtendedValue` | `CalcValue { core, rich }` | W099-010 folds object, presentation, and error metadata into `RichValue`; W099-015 deletes the type. |
| `LambdaValue` | `RichValue::Callable(CallableValue)` | W099-011 removes OxFunc kernel dependence; W099-013 removes OxFml native `EvalValue::Lambda`; W099-015 deletes the type. |
| `ReferenceLike { kind, target }` | typed `ReferenceLike { system, identity, display }` | W099-002 replaces the native payload; compatibility textual constructors get explicit deletion owners. |
| `ReferenceKind` | textual-reference compatibility/fact vocabulary only | W099-002 moves it out of native identity. W099-009 deletes function-kernel reliance on it where provider facts should be used. |
| `ReferenceResolver` | `ReferenceSystemProvider` | W099-003 adds provider and adapters; W099-009 migrates reference-sensitive functions; W099-015 deletes the old trait. |
| `ReferenceTextResolver` | `ReferenceSystemProvider::resolve_text` | W099-003 adapts `INDIRECT` and tests; W099-015 deletes the old trait. |
| `resolve_eval_value` | provider dereference or `CalcValue` projection helper | W099-004/W099-009 move callers to `ReferenceSystemProvider` and `CalcValue.core`. |
| `resolve_reference_values` | provider enumeration | W099-003 introduces enumeration request/result types; W099-009 migrates sparse/aggregate consumers. |
| `.target` | textual compatibility display/input only | W099-002 stops exposing universal `.target`; W099-009 removes function logic that treats display text as identity. |
| `HOST_REF_*` | no active OxFunc-local residue found | W099-014 remains the downstream OxCalc/OxFml integration audit lane for runtime host identities. |

## 5. First Code Batch Boundary

The next code batch is W099-002/W099-003 foundation shape. It must be small
enough to review and must not start the broad `EvalValue` or `CallArgValue`
sweep.

Required first edits:
1. replace the native `ReferenceLike { kind, target }` payload in
   `oxfunc_value_types` with typed `system`, `identity`, and optional display
   metadata;
2. keep textual/multi-area constructors only as compatibility constructors with
   explicit deletion ownership;
3. preserve representation-level `CalcArray<Vec<CalcValue>>` support for
   missing, empty, nested arrays, rich values, callables, and references;
4. keep `CallableValue` equality as `Rc::ptr_eq(handle)` plus arity;
5. add `ReferenceSystemProvider` request/result/error scaffolding in
   `oxfunc_core::resolver`;
6. add a provider slot to `FunctionExecutionContextBundle` /
   `FunctionExecutionContext`, initially alongside old resolver slots as
   compatibility-only inputs;
7. add focused value-crate and core tests for textual identity, opaque identity,
   display-as-non-identity, provider dereference, provider text resolution, and
   provider enumeration shape.

Batch guardrails:
1. no `type EvalValue = CalcValue` alias;
2. no new final argument wrapper around `CalcValue`;
3. no broad dispatch or kernel sweep before the provider foundation compiles;
4. no behavior-changing parity fix mixed into the foundation batch unless it is
   isolated and routed through the bug stream;
5. adapters introduced in this batch must say which later W099 bead deletes them.

## 6. Immediate Open Lanes

1. W099-002 must decide final Rust names for `ReferenceSystemId`,
   `ReferenceIdentity`, `ReferenceDisplay`, `TextualReferenceIdentity`,
   `ReferenceHandle`, and composite reference identity.
2. W099-003 must decide provider result names for describe, dereference,
   enumerate, facts, resolve text, and transform/compose without freezing a
   concrete host implementation in OxFunc.
3. OxFml callable follow-through remains a later cross-repo code lane and is
   not satisfied by the current legacy `EvalValue::Lambda -> CalcValue`
   fallback conversion.

## 7. Status Axes

execution_state: `complete`

scope_completeness: `scope_partial`

target_completeness: `target_partial`

integration_completeness: `partial`

open_lanes:
1. typed reference payload foundation,
2. `ReferenceSystemProvider` foundation,
3. call-boundary migration,
4. array/preparation/dispatch/kernel/rich/callable migration,
5. OxFml callable follow-through,
6. OxCalc reference-system integration,
7. legacy type/provider/adapter deletion audit,
8. final cross-repo validation.

## 8. W099-001 Closure Review

Pre-Closure Verification Checklist:

| # | Check | Result |
|---|-------|--------|
| 1 | Function contract rows complete and promoted for all in-scope functions? | Yes - not in W099-001 scope; no function contract rows were changed. |
| 2 | Lean obligations for each slice class satisfied or explicitly aligned per formalization strategy? | Yes - not in W099-001 scope; no function slice claim is made. |
| 3 | Rust implementation and required tests pass for all in-scope functions? | Yes - W099-001 is inventory-only; source scan and workset checks passed. |
| 4 | At least one deterministic replay artifact exists per in-scope function behavior? | Yes - not in W099-001 scope; no function behavior claim is made. |
| 5 | Evidence links complete and reproducible? | Yes - scan commands and generated CSV ledger are recorded. |
| 6 | Version scope explicit on both axes? | Yes - not in W099-001 scope; no Excel behavior/version claim is made. |
| 7 | Public-doc vs empirical discrepancies recorded and resolved in favor of empirical Excel behavior? | Yes - not in W099-001 scope; no empirical behavior discrepancy is handled. |
| 8 | XLL verification-seam limitations documented where material? | Yes - not material to this inventory bead. |
| 9 | Cross-repo impact assessed and handoff filed if boundary/evaluator-facing clauses affected? | Yes - inbound OxFml observations were reviewed; no handoff is filed because this bead only inventories the planned refactor. |
| 10 | No known semantic gap remains in declared scope? | Yes - declared W099-001 scope is the inventory/deletion ledger, not semantic migration. |
| 11 | Completion language audit passed? | Yes - this artifact does not claim W099 terminal completion or function implementation. |
| 12 | `docs/IN_PROGRESS_FEATURE_WORKLIST.md` updated? | Yes - `IP-25` added for W099. |
| 13 | Execution-state blocker surface updated? | Yes - bead `oxf-im4m.1` is the live execution surface. |

Completion Claim Self-Audit:

1. Scope re-read: passed. W099-001 asked for scan/count/classification/deletion routing before implementation; the CSV plus narrative ledger provide that.
2. Gate criteria re-read: passed. W099-002 can start from the recorded ledger and first-batch boundary.
3. Silent scope reduction check: passed. This bead did not narrow W099; it leaves all W099 migration lanes open.
4. "Looks done but is not" pattern check: passed. No scaffold or compatibility adapter is reported as implementation, no tests are used to claim migrated semantics, and no downstream handoff is treated as closure.
5. Included result: passed. This section records the checklist and self-audit for the W099-001 closure claim only.

Fresh-eyes review:

1. Issue found and corrected: `IP-25` was added to the feature map body but was initially missing from the current-reading list.
2. Issue found and corrected: generated CSV file paths initially used Windows backslashes; the ledger was regenerated with forward slashes.
3. Remaining risk: `.target` is intentionally a conservative regex bucket and may include non-reference fields. W099-002/W099-009 must refine those rows as code migrates.

Validation:

1. `git diff --check`: passed.
2. `scripts/check-worksets.ps1`: passed.
3. CSV consistency scan: passed; the occurrence ledger matches the live `rg` source scan and has no missing owner, deletion batch, or deletion route fields.

## 9. W099-002 Value-Type Foundation Record

execution_state: `complete`

scope_completeness: `scope_complete`

target_completeness: `target_complete`

integration_completeness: `partial`

open_lanes:
1. W099-003 provider foundation has not landed.
2. W099-009/W099-015 must delete the migration-only `ReferenceLike.kind` and `ReferenceLike.target` mirrors after downstream callers stop reading them.
3. OxFml has a filed handoff, `HO-FN-017`; its constructor-use repairs are a compatibility landing, not cross-repo seam completion.

Planned scope:
1. Establish the first typed `ReferenceLike` shape with `system`, `identity`, and `display`.
2. Keep `CalcArray` on `CalcValue`.
3. Preserve `CallableValue` equality semantics by opaque handle identity plus arity.
4. Avoid a final-looking `EvalValue = CalcValue` alias.
5. Mark migration-only reference mirrors for later deletion.

Evidence:
1. `CalcArray` already stores `Vec<CalcValue>` and remains unchanged.
2. `CallableValue` equality remains handle id plus arity.
3. `ReferenceLike` now carries `ReferenceSystemId`, `ReferenceIdentity`, and optional `ReferenceDisplay`.
4. `ReferenceLike.kind` and `ReferenceLike.target` remain only as W099 migration mirrors with an inline deletion-owner comment.
5. `HO-FN-017` records the OxFml evaluator-facing impact.

Pre-Closure Verification Checklist:

| # | Check | Result |
|---|-------|--------|
| 1 | Function contract rows complete and promoted for all in-scope functions? | Yes - not in W099-002 scope; no function contract rows were changed. |
| 2 | Lean obligations for each slice class satisfied or explicitly aligned per formalization strategy? | Yes - not in W099-002 scope; no function slice claim is made. |
| 3 | Rust implementation and required tests pass for all in-scope functions? | Yes - value-type tests, core lib tests, and core test compilation passed. |
| 4 | At least one deterministic replay artifact exists per in-scope function behavior? | Yes - not in W099-002 scope; no Excel function behavior claim is made. |
| 5 | Evidence links complete and reproducible? | Yes - commands are listed in this record. |
| 6 | Version scope explicit on both axes? | Yes - not material to this value-type foundation bead; no Excel version behavior claim is made. |
| 7 | Public-doc vs empirical discrepancies recorded and resolved in favor of empirical Excel behavior? | Yes - no public-doc/empirical behavior discrepancy is handled in this bead. |
| 8 | XLL verification-seam limitations documented where material? | Yes - not material to this value-type foundation bead. |
| 9 | Cross-repo impact assessed and handoff filed if boundary/evaluator-facing clauses affected? | Yes - `HO-FN-017` filed for OxFml evaluator `ReferenceLike` construction. |
| 10 | No known semantic gap remains in declared scope? | Yes - declared W099-002 scope is the value-type foundation shape and migration-only constructor landing. |
| 11 | Completion language audit passed? | Yes - this record claims only W099-002 bead closure, not W099 terminal migration or function semantic completion. |
| 12 | `docs/IN_PROGRESS_FEATURE_WORKLIST.md` updated? | Yes - W099 is already represented by `IP-25`; no new feature-map row was required for this bead. |
| 13 | Execution-state blocker surface updated? | Yes - bead `oxf-im4m.2` is the live execution surface and is closed with this evidence. |

Completion Claim Self-Audit:

1. Scope re-read: passed. The bead asked for value-type foundation shape, not provider behavior or full call-boundary migration.
2. Gate criteria re-read: passed. All W099-002 acceptance checks were addressed directly or verified as already true.
3. Silent scope reduction check: passed. The `kind`/`target` mirrors are explicitly marked migration-only and routed to later deletion beads rather than treated as final design.
4. "Looks done but is not" pattern check: passed. Constructor migration is compatibility work only; provider integration and mirror deletion remain open lanes.
5. Included result: passed. This section records checklist, self-audit, evidence, and remaining integration lanes for the W099-002 closure claim.

Fresh-eyes review:

1. Issue found and corrected: `cargo check --lib` initially missed test and OxFml construction sites; a full `cargo test --lib --no-run` exposed the missing `ReferenceLike` fields, and 155 legacy literals plus one shorthand helper were moved to `ReferenceLike::new`.
2. Issue found and corrected: full rustfmt introduced unrelated formatting churn in non-reference files; those diffs were removed from the OxFunc commit.
3. Issue found and corrected: the OxFml evaluator compile surface required matching constructor repairs; the impact is recorded in `HO-FN-017`.
4. Remaining tension: structured, opaque, and composite references still have textual migration mirrors for legacy callers. This is intentional W099 scaffolding and remains owned by later deletion/provider beads.

Validation:

1. `cargo test --manifest-path crates/oxfunc_value_types/Cargo.toml`: passed, 18 tests.
2. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib --no-run`: passed.
3. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib`: passed, 1320 passed, 1 ignored.

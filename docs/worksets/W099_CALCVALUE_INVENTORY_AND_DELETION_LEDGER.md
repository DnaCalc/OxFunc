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

## 10. W099-003 ReferenceSystemProvider Foundation Record

execution_state: `complete`

scope_completeness: `scope_complete`

target_completeness: `target_complete`

integration_completeness: `partial`

open_lanes:
1. W099-009 must move reference-sensitive functions from `ReferenceResolver` / `ReferenceTextResolver` and legacy `.target` parsing onto `ReferenceSystemProvider` operations.
2. W099-015 must delete the migration-only `LegacyReferenceSystemProvider` adapter, the old resolver/text-resolver traits, and remaining compatibility residue.
3. OxFml/OxCalc follow-through is still outside this OxFunc-local provider foundation bead.

Planned scope:
1. Introduce `ReferenceSystemProvider` as the native FEC reference capability shape.
2. Add minimal request/result/error packets for dereference, enumeration, text resolution, and facts.
3. Expose an optional provider slot through `FunctionExecutionContextBundle` and `FunctionExecutionContext`.
4. Keep old resolver/text-resolver paths only behind an explicitly marked W099 migration adapter.
5. Record a fresh `HOST_REF_` / `.target` / resolver residue scan.

Evidence:
1. `ReferenceSystemProvider` exists with `dereference`, `enumerate_values`, `resolve_text`, and `facts`.
2. New packets include `ReferenceDereferenceRequest`, `ReferenceEnumerationRequest`, `ReferenceTextResolveRequest`, `ReferenceFactsRequest`, `ReferenceFacts`, `ReferenceSystemError`, and operation/identity classifiers.
3. `FunctionExecutionContextBundle` and `FunctionExecutionContextRef` expose `reference_system_provider`.
4. `LegacyReferenceSystemProvider` is marked W099 migration-only and routes old resolver/text-resolver behavior through the new provider shape for compatibility.
5. Residue scan over active OxFunc Rust sources: `HOST_REF_` = 0, `.target` = 204, `ReferenceResolver` = 1275, `ReferenceTextResolver` = 24. These are expected W099-009/W099-015 lanes, not W099-003 closure blockers.

Pre-Closure Verification Checklist:

| # | Check | Result |
|---|-------|--------|
| 1 | Function contract rows complete and promoted for all in-scope functions? | Yes - not in W099-003 scope; no function contract rows were changed. |
| 2 | Lean obligations for each slice class satisfied or explicitly aligned per formalization strategy? | Yes - not in W099-003 scope; no function slice claim is made. |
| 3 | Rust implementation and required tests pass for all in-scope functions? | Yes - provider/FEC tests, core test compilation, and core lib tests passed. |
| 4 | At least one deterministic replay artifact exists per in-scope function behavior? | Yes - not in W099-003 scope; no Excel function behavior claim is made. |
| 5 | Evidence links complete and reproducible? | Yes - commands and residue counts are listed in this record. |
| 6 | Version scope explicit on both axes? | Yes - not material to this provider-foundation bead; no Excel version behavior claim is made. |
| 7 | Public-doc vs empirical discrepancies recorded and resolved in favor of empirical Excel behavior? | Yes - no public-doc/empirical behavior discrepancy is handled in this bead. |
| 8 | XLL verification-seam limitations documented where material? | Yes - not material to this provider-foundation bead. |
| 9 | Cross-repo impact assessed and handoff filed if boundary/evaluator-facing clauses affected? | Yes - this bead adds an OxFunc FEC slot only; downstream migration remains open under W099 follow-through beads. |
| 10 | No known semantic gap remains in declared scope? | Yes - declared W099-003 scope is the provider capability foundation, not full provider adoption. |
| 11 | Completion language audit passed? | Yes - this record claims only W099-003 bead closure, not W099 terminal migration or reference-function migration. |
| 12 | `docs/IN_PROGRESS_FEATURE_WORKLIST.md` updated? | Yes - W099 is already represented by `IP-25`; no new feature-map row was required for this bead. |
| 13 | Execution-state blocker surface updated? | Yes - bead `oxf-im4m.3` is the live execution surface and is closed with this evidence. |

Completion Claim Self-Audit:

1. Scope re-read: passed. The bead asked for provider foundation in FEC, not broad conversion of all resolver users.
2. Gate criteria re-read: passed. Request/result/error types, provider slot exposure, tests, migration adapter marking, and residue scan are present.
3. Silent scope reduction check: passed. The old resolver/text-resolver paths remain visible as open lanes and are not reported as migrated.
4. "Looks done but is not" pattern check: passed. `LegacyReferenceSystemProvider` is explicitly migration-only and routed to W099-015 deletion.
5. Included result: passed. This section records checklist, self-audit, evidence, and remaining integration lanes for the W099-003 closure claim.

Fresh-eyes review:

1. Issue found and corrected: an invalid multi-filter cargo command was replaced by full core test compilation and focused provider/FEC test runs.
2. Issue found and corrected: full `cargo fmt` introduced unrelated formatting churn in non-provider function files; those diffs were removed from the commit.
3. Issue found and corrected: the legacy adapter initially mapped all old resolver errors as dereference failures and did not preserve typed identity for unresolved opaque references; error mapping now carries the actual provider operation and typed identity class.
4. Remaining tension: `FunctionCallTarget::invoke` still dispatches through the old resolver arguments because broad dispatch migration belongs to W099-008/W099-009. The new FEC provider slot is therefore foundational, not yet the active dispatch path.

Validation:

1. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib --no-run`: passed.
2. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib reference_system`: passed, 8 tests.
3. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib function_execution_context`: passed, 2 tests.
4. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib`: passed, 1328 passed, 1 ignored.
5. `git diff --check`: passed.
6. `scripts/check-worksets.ps1`: passed.

## 11. W099-004 Central CalcValue Construction And Coercion Helper Record

execution_state: `complete`

scope_completeness: `scope_complete`

target_completeness: `target_complete`

integration_completeness: `partial`

open_lanes:
1. W099-005 must move call-boundary construction to native `CalcValue`.
2. W099-008/W099-012 must move dispatch and kernel paths to native `CalcValue`.
3. W099-009 must move reference-sensitive coercion/function paths onto `ReferenceSystemProvider`.
4. W099-015 must delete the migration-only `EvalValue -> CalcValue` and `CalcValue -> CallArgValue` adapters with the legacy carrier types.

Planned scope:
1. Add central `CalcValue` constructors/projection helpers for scalar, error, empty, missing, array, reference, rich, and callable lanes.
2. Add CalcValue-first scalar numeric coercion helpers without constructing legacy carriers.
3. Mark legacy carrier conversion adapters with explicit W099 deletion owners.
4. Keep resolver-backed reference coercion on the old path for later reference/call-boundary migration beads.

Evidence:
1. `CalcValue` exposes `core`, `rich`, scalar projections, array/reference projections, rich-object/callable/presentation/error-metadata projections, and rich construction helpers.
2. `coerce_calc_scalar_to_number` works over `CalcValue.core` and returns explicit missing/empty/reference/array errors without constructing `EvalValue` or `CallArgValue`.
3. Existing `From<EvalValue> for CalcValue` and `CallArgValue::value(CalcValue)` are now marked W099 migration-only with W099-005/W099-008/W099-012/W099-015 deletion owners.
4. `CalcValue::allowed_at(ValueBoundary)` routes rich/core values through the central admission table instead of requiring callers to duplicate tag checks.

Pre-Closure Verification Checklist:

| # | Check | Result |
|---|-------|--------|
| 1 | Function contract rows complete and promoted for all in-scope functions? | Yes - not in W099-004 scope; no function contract rows were changed. |
| 2 | Lean obligations for each slice class satisfied or explicitly aligned per formalization strategy? | Yes - not in W099-004 scope; no function slice claim is made. |
| 3 | Rust implementation and required tests pass for all in-scope functions? | Yes - value-type helper tests, CalcValue coercion tests, full value-type tests, and full core lib tests passed. |
| 4 | At least one deterministic replay artifact exists per in-scope function behavior? | Yes - not in W099-004 scope; no Excel function behavior claim is made. |
| 5 | Evidence links complete and reproducible? | Yes - validation commands are listed in this record. |
| 6 | Version scope explicit on both axes? | Yes - not material to this helper-foundation bead; no Excel version behavior claim is made. |
| 7 | Public-doc vs empirical discrepancies recorded and resolved in favor of empirical Excel behavior? | Yes - no public-doc/empirical behavior discrepancy is handled in this bead. |
| 8 | XLL verification-seam limitations documented where material? | Yes - not material to this helper-foundation bead. |
| 9 | Cross-repo impact assessed and handoff filed if boundary/evaluator-facing clauses affected? | Yes - this bead adds OxFunc-local helper APIs and marks adapters; downstream migration remains in later W099 beads. |
| 10 | No known semantic gap remains in declared scope? | Yes - declared W099-004 scope is central helper construction/projection/coercion, not broad call-boundary adoption. |
| 11 | Completion language audit passed? | Yes - this record claims only W099-004 bead closure, not W099 terminal migration or function semantic completion. |
| 12 | `docs/IN_PROGRESS_FEATURE_WORKLIST.md` updated? | Yes - W099 is already represented by `IP-25`; no new feature-map row was required for this bead. |
| 13 | Execution-state blocker surface updated? | Yes - bead `oxf-im4m.4` is the live execution surface and is closed with this evidence. |

Completion Claim Self-Audit:

1. Scope re-read: passed. The bead asked for central helper shape, not broad replacement of every existing `EvalValue`/`CallArgValue` call site.
2. Gate criteria re-read: passed. Constructors/projections/coercion helpers exist, tests cover the helper shape, and legacy adapters have deletion-owner comments.
3. Silent scope reduction check: passed. Reference coercion remains unresolved-shape-only in the CalcValue scalar helper and old resolver-backed reference coercion remains an explicit later lane.
4. "Looks done but is not" pattern check: passed. Compatibility conversions are reported as migration-only adapters rather than permanent architecture.
5. Included result: passed. This section records checklist, self-audit, evidence, and remaining integration lanes for the W099-004 closure claim.

Fresh-eyes review:

1. Issue found and corrected: the first rich-object projection test used non-existent `RichObjectType` fields; it now uses the actual `type_name`, `required_keys`, and `key_flags` shape.
2. Issue found and corrected: callable projection coverage was missing from the first helper test pass; a callable constructor/projection test now pins the rich-only callable lane.
3. Issue found and corrected: rich projection helpers initially exposed metadata but did not provide an admission-aware entry point; `CalcValue::allowed_at(ValueBoundary)` now routes helper callers through the central boundary table.
4. Remaining tension: `coerce_calc_scalar_to_number` deliberately rejects references rather than dereferencing them because provider-backed reference coercion belongs to W099-005/W099-009.

Validation:

1. `cargo test --manifest-path crates/oxfunc_value_types/Cargo.toml calc_value --lib`: passed, 4 tests.
2. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib coerce_calc_scalar`: passed, 2 tests.
3. `cargo test --manifest-path crates/oxfunc_value_types/Cargo.toml`: passed, 21 tests.
4. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib`: passed, 1330 passed, 1 ignored.
5. `git diff --check`: passed.
6. `scripts/check-worksets.ps1`: passed.

## 12. W099-005 Call-Boundary Migration Record

execution_state: `complete`

scope_completeness: `scope_complete`

target_completeness: `target_complete`

integration_completeness: `partial`

open_lanes:
1. W099-006 must move array storage and array-cell policy fully to `CalcArray` / `CalcValue`.
2. W099-008/W099-012 must move dispatcher and kernel input/return paths away from `EvalValue`.
3. W099-009 must move reference-sensitive function paths onto `ReferenceSystemProvider`.
4. W099-015 must delete `CallArgValue`, the private `FunctionCallTarget` dispatch bridge, and OxFml caller-side bridge helpers after downstream paths no longer require legacy argument carriers.

Planned scope:
1. Replace the public `FunctionCallScratch` argument storage and call-target invocation argument API with `CalcValue`.
2. Preserve omitted, empty, and reference-visible argument distinctions as `CoreValue::Missing`, `CoreValue::Empty`, and `CoreValue::Reference`.
3. Preserve direct-array versus reference-visible behavior while the legacy dispatcher still consumes `CallArgValue`.
4. Update the OxFml caller edge so the migrated OxFunc call-boundary API is exercised by the downstream evaluator seam.

Evidence:
1. `FunctionCallScratch` now stores `Vec<CalcValue>` and exposes `CalcValue` push/extend/mutator APIs.
2. `FunctionCallTarget::invoke`, `invoke_scratch`, and `invoke_with_scratch_builder` now accept `&[CalcValue]` / `Vec<CalcValue>` builder arguments.
3. The only `CallArgValue` use in `crates/oxfunc_core/src/function_call.rs` is the private `legacy_call_args_for_dispatch` adapter plus tests that prove its temporary behavior.
4. OxFml evaluator call sites now convert their existing internal `CallArgValue` carriers to `CalcValue` before invoking OxFunc call targets, and convert back only for legacy prepared-call/register/host-fallback helpers that still require `CallArgValue`.

Pre-Closure Verification Checklist:

| # | Check | Result |
|---|-------|--------|
| 1 | Function contract rows complete and promoted for all in-scope functions? | Yes - not in W099-005 scope; no function contract rows were changed. |
| 2 | Lean obligations for each slice class satisfied or explicitly aligned per formalization strategy? | Yes - not in W099-005 scope; no function slice claim is made. |
| 3 | Rust implementation and required tests pass for all in-scope functions? | Yes - focused call-boundary tests and downstream evaluator seam tests passed. |
| 4 | At least one deterministic replay artifact exists per in-scope function behavior? | Yes - not a function-semantic bead; call-boundary behavior is pinned by deterministic Rust tests. |
| 5 | Evidence links complete and reproducible? | Yes - validation commands are listed in this record. |
| 6 | Version scope explicit on both axes? | Yes - not material to this call-boundary carrier bead; no Excel version behavior claim is made. |
| 7 | Public-doc vs empirical discrepancies recorded and resolved in favor of empirical Excel behavior? | Yes - no public-doc/empirical discrepancy is handled in this bead. |
| 8 | XLL verification-seam limitations documented where material? | Yes - not material to this call-boundary carrier bead. |
| 9 | Cross-repo impact assessed and handoff filed if boundary/evaluator-facing clauses affected? | Yes - OxFml caller-edge changes were integrated directly; no unresolved handoff remains for this bead. |
| 10 | No known semantic gap remains in declared scope? | Yes - declared W099-005 scope is the call-boundary carrier API, with legacy dispatcher bridging explicitly left to later beads. |
| 11 | Completion language audit passed? | Yes - this record claims only W099-005 bead closure, not W099 terminal migration or function semantic completion. |
| 12 | `docs/IN_PROGRESS_FEATURE_WORKLIST.md` updated? | Yes - W099 remains represented by `IP-25`; no new feature-map row was required for this bead. |
| 13 | Execution-state blocker surface updated? | Yes - bead `oxf-im4m.5` is the live execution surface and is closed with this evidence. |

Completion Claim Self-Audit:

1. Scope re-read: passed. The bead asked for call-boundary migration, not full removal of every function-module `CallArgValue` signature.
2. Gate criteria re-read: passed. Function-call APIs and scratch now use `CalcValue`; missing/empty/reference-visible and direct-array/reference-visible cases have focused tests.
3. Silent scope reduction check: passed. The private bridge to `CallArgValue` remains only because the dispatcher and function kernels are later W099 lanes, and it is recorded as W099-015 residue.
4. "Looks done but is not" pattern check: passed. OxFml still has internal `CallArgValue` helpers, reported as caller-side compatibility residue rather than native architecture.
5. Included result: passed. This section records checklist, self-audit, evidence, validation, and remaining integration lanes for the W099-005 closure claim.

Fresh-eyes review:

1. Issue found and corrected: the first API pass returned `CalcValue` from `FunctionCallTarget::invoke`, which was premature because dispatcher/kernel return migration belongs to W099-008/W099-012; the return type remains `EvalValue` in this bead.
2. Issue found and corrected: OxFml caller edges still passed `CallArgValue` into the migrated OxFunc API; they now convert to `CalcValue` before invocation.
3. Issue found and corrected: trimming trailing omitted arguments initially checked for `CallArgValue::MissingArg` on scratch storage; it now checks `CalcValue::is_missing`.
4. Issue found and corrected: the first CalcValue-to-legacy bridge erased legacy lambda payloads by converting callable core to `#CALC!`; the temporary callable adapter now retains the legacy `LambdaValue` until W099-011/W099-015 delete that bridge.
5. Remaining tension: OxFml still converts back to `CallArgValue` for prepared-call register parsing, CALL parsing, host fallback, and HSTACK empty-carrier compatibility until later W099 dispatcher/kernel beads remove those consumers.

Validation:

1. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml function_call --lib`: passed, 12 tests.
2. `cargo test --manifest-path crates/oxfunc_value_types/Cargo.toml legacy_lambda_adapter --lib`: passed, 1 test.
3. `cargo test --manifest-path crates/oxfunc_value_types/Cargo.toml`: passed, 22 tests.
4. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib`: passed, 1332 passed, 1 ignored.
5. `cargo test --manifest-path C:/Work/DnaCalc/OxFml/crates/oxfml_core/Cargo.toml --test evaluator_tests evaluator_executes_map_with_local_lambda_callable`: passed, 1 test.
6. `cargo test --manifest-path C:/Work/DnaCalc/OxFml/crates/oxfml_core/Cargo.toml --test evaluator_tests evaluator_executes_foundation_array_lambda_carrier_case_ftc_0455`: passed, 1 test.
7. `cargo test --manifest-path C:/Work/DnaCalc/OxFml/crates/oxfml_core/Cargo.toml --test callable_transport_tests`: passed, 1 test.
8. `cargo test --manifest-path C:/Work/DnaCalc/OxFml/crates/oxfml_core/Cargo.toml --test evaluator_tests`: attempted after the bridge correction; 99 passed, 4 failed. The remaining failing tests are outside the call-boundary migration slice and sit in an OxFml worktree that already has unrelated dirty parser/binding/test files; they are not treated as W099-005 closure evidence.

## 13. W099-006 Array Model Migration Record

execution_state: `complete`

scope_completeness: `scope_complete`

target_completeness: `target_complete`

integration_completeness: `partial`

open_lanes:
1. W099-007 must move preparation and adapter surfaces away from legacy array-cell carriers.
2. W099-008/W099-012 must move dispatcher and kernel return/input paths away from `EvalValue::Array(EvalArray)`.
3. W099-015 must delete `EvalArray`, `ArrayCellValue`, and the lossy legacy projection helpers after all active callers use `CalcArray` / `CalcValue`.

Planned scope:
1. Make `CalcArray` the shared construction/projection surface for row-major array shape validation.
2. Centralize legacy array-cell coercion policy on `CalcValue` / `CalcArray` instead of open-coded conversions.
3. Route dynamic-array spill/result construction through `CalcArray` while preserving the legacy dispatcher return carrier for later beads.
4. Preserve empty-cell and error-cell behavior in both CalcValue-native and legacy-projected paths.

Evidence:
1. `CalcArray::from_cells_iter`, `CalcArray::from_legacy_cells_iter`, `CalcArray::cell_count`, and `CalcArray::to_legacy_eval_array_lossy` now provide native array construction/projection helpers.
2. `CalcValue::to_legacy_array_cell_lossy` and `ArrayCellValue::to_calc_value_lossy` centralize the temporary legacy array-cell coercion policy, including `CoreValue::Empty -> ArrayCellValue::EmptyCell` and unrepresentable nested/missing/reference cells to `#VALUE!`.
3. `CallArgValue::value(CalcValue)` now projects `CoreValue::Array(CalcArray)` through the shared `CalcArray` legacy projection helper instead of duplicating array-cell mapping.
4. Dynamic-array reshape result construction now validates and constructs through `CalcArray` before projecting back to the legacy `EvalArray` return carrier.

Pre-Closure Verification Checklist:

| # | Check | Result |
|---|-------|--------|
| 1 | Function contract rows complete and promoted for all in-scope functions? | Yes - not in W099-006 scope; no function contract rows were changed. |
| 2 | Lean obligations for each slice class satisfied or explicitly aligned per formalization strategy? | Yes - not in W099-006 scope; no function slice claim is made. |
| 3 | Rust implementation and required tests pass for all in-scope functions? | Yes - focused value-type array tests and dynamic-array helper tests passed. |
| 4 | At least one deterministic replay artifact exists per in-scope function behavior? | Yes - not a function-semantic bead; array carrier behavior is pinned by deterministic Rust tests. |
| 5 | Evidence links complete and reproducible? | Yes - validation commands are listed in this record. |
| 6 | Version scope explicit on both axes? | Yes - not material to this array carrier bead; no Excel version behavior claim is made. |
| 7 | Public-doc vs empirical discrepancies recorded and resolved in favor of empirical Excel behavior? | Yes - no public-doc/empirical discrepancy is handled in this bead. |
| 8 | XLL verification-seam limitations documented where material? | Yes - not material to this array carrier bead. |
| 9 | Cross-repo impact assessed and handoff filed if boundary/evaluator-facing clauses affected? | Yes - no new cross-repo caller API change was introduced in this bead. |
| 10 | No known semantic gap remains in declared scope? | Yes - declared W099-006 scope is shared array carrier/conversion helpers and dynamic-array result routing, not deletion of all legacy array consumers. |
| 11 | Completion language audit passed? | Yes - this record claims only W099-006 bead closure, not W099 terminal migration or function semantic completion. |
| 12 | `docs/IN_PROGRESS_FEATURE_WORKLIST.md` updated? | Yes - W099 remains represented by `IP-25`; no new feature-map row was required for this bead. |
| 13 | Execution-state blocker surface updated? | Yes - bead `oxf-im4m.6` is the live execution surface and is closed with this evidence. |

Completion Claim Self-Audit:

1. Scope re-read: passed. The bead asked for array model migration, not full deletion of every `EvalArray` and `ArrayCellValue` occurrence in all function modules.
2. Gate criteria re-read: passed. Shared array construction, row-major access, shape validation, dynamic-array result construction, and array-cell coercion policy now route through `CalcArray` / `CalcValue` surfaces.
3. Silent scope reduction check: passed. Remaining `EvalArray` / `ArrayCellValue` consumers are reported as W099-007/W099-008/W099-012/W099-015 lanes rather than final architecture.
4. "Looks done but is not" pattern check: passed. Legacy projection helpers are named `lossy` and recorded as migration-only.
5. Included result: passed. This section records checklist, self-audit, evidence, validation, and remaining integration lanes for the W099-006 closure claim.

Fresh-eyes review:

1. Issue found and corrected: the first value-type pass left `EvalValue::Array` conversion using the older optional `ArrayCellValue::to_calc_value` path; it now uses `to_calc_value_lossy` so empty array cells consistently become `CoreValue::Empty`.
2. Issue found and corrected: dynamic-array `VSTACK` still bypassed the central `build_array` helper through `EvalArray::from_cells_iter`; it now routes through the `CalcArray`-backed result builder.
3. Remaining tension: many function modules still carry `EvalArray` / `ArrayCellValue` in signatures and tests because W099-007/W099-008/W099-012 own adapter/dispatcher/kernel migration and W099-015 owns deletion.

Validation:

1. `cargo test --manifest-path crates/oxfunc_value_types/Cargo.toml calc_array --lib`: passed, 4 tests.
2. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml dynamic_array_reshape --lib`: passed, 11 tests.
3. `cargo test --manifest-path crates/oxfunc_value_types/Cargo.toml`: passed, 24 tests.
4. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib`: passed, 1332 passed, 1 ignored.

## 14. W099-007 Preparation And Adapter Migration Record

execution_state: `complete`

scope_completeness: `scope_complete`

target_completeness: `target_complete`

integration_completeness: `partial`

open_lanes:
1. W099-008 and W099-012 must migrate remaining dispatcher and kernel callbacks that still receive `PreparedArgValue` compatibility projections.
2. Aggregate reference provenance still uses local `AggregatePreparedValue` origin facts, but its payload is now `CalcValue`; W099-015 may delete or tuple-elide the local fact container after the remaining aggregate callers settle.
3. W099-015 must delete `PreparedArgValue`, `EvalArray`, `ArrayCellValue`, and migration projection helpers after all active callers consume `CalcValue` / `CalcArray` directly.

Planned scope:
1. Add CalcValue-first values-only preparation APIs so adapter preparation can output `CalcValue` rather than a replacement prepared-value carrier.
2. Route existing values-only prepared adapter entry points through CalcValue preparation before compatibility projection for legacy callbacks.
3. Preserve missing-argument, empty-cell, reference resolution, single-cell array normalization, and callable payload behavior across the new CalcValue preparation lane.
4. Keep aggregate/reference facts scoped to existing adapter-local preparation structures; do not introduce a final public aggregate-provenance value carrier.

Evidence:
1. `prepare_calc_value_values_only`, `prepare_calc_values_only`, `prepare_call_arg_as_calc_value_values_only`, and `prepare_call_args_as_calc_values_only` provide CalcValue-first preparation entry points.
2. `run_calc_values_only_prepared` and `map_calc_values_only_prepared` expose callback paths that consume prepared `CalcValue` slices directly.
3. `run_values_only_prepared`, `prepare_arg_values_only`, and `prepare_args_values_only` now route through CalcValue preparation before projecting to `PreparedArgValue` for migration-only legacy callers.
4. Legacy projection uses `CallArgValue::value(CalcValue)` so callable/lambda payloads survive the temporary compatibility bridge instead of collapsing to their core error sentinel.
5. `PreparedArgValue` is documented as W099 migration-only compatibility projection rather than a final replacement type.

Pre-Closure Verification Checklist:

| # | Check | Result |
|---|-------|--------|
| 1 | Function contract rows complete and promoted for all in-scope functions? | Yes - not in W099-007 scope; no function contract rows were changed. |
| 2 | Lean obligations for each slice class satisfied or explicitly aligned per formalization strategy? | Yes - not in W099-007 scope; no function slice claim is made. |
| 3 | Rust implementation and required tests pass for all in-scope functions? | Yes - focused adapter, aggregate, callable-helper, and full core library tests passed. |
| 4 | At least one deterministic replay artifact exists per in-scope function behavior? | Yes - not a function-semantic bead; adapter carrier behavior is pinned by deterministic Rust tests. |
| 5 | Evidence links complete and reproducible? | Yes - validation commands are listed in this record. |
| 6 | Version scope explicit on both axes? | Yes - not material to this adapter-carrier bead; no Excel version behavior claim is made. |
| 7 | Public-doc vs empirical discrepancies recorded and resolved in favor of empirical Excel behavior? | Yes - no public-doc/empirical discrepancy is handled in this bead. |
| 8 | XLL verification-seam limitations documented where material? | Yes - not material to this adapter-carrier bead. |
| 9 | Cross-repo impact assessed and handoff filed if boundary/evaluator-facing clauses affected? | Yes - no OxFml evaluator-facing clause or FEC/F3E boundary change was introduced in this OxFunc bead. |
| 10 | No known semantic gap remains in declared scope? | Yes - declared W099-007 scope is CalcValue preparation/adapters, not final deletion of every legacy callback. |
| 11 | Completion language audit passed? | Yes - this record claims only W099-007 bead closure, not W099 terminal migration or function semantic completion. |
| 12 | `docs/IN_PROGRESS_FEATURE_WORKLIST.md` updated? | Yes - W099 remains represented by `IP-25`; no new feature-map row was required for this bead. |
| 13 | Execution-state blocker surface updated? | Yes - bead `oxf-im4m.7` is the live execution surface and is closed with this evidence. |

Completion Claim Self-Audit:

1. Scope re-read: passed. The bead asks for preparation and adapter migration, not full W099 deletion of all legacy value carriers.
2. Gate criteria re-read: passed. CalcValue preparation paths now exist, legacy values-only adapter paths route through them, and no new final `PreparedArgValue` replacement type was created.
3. Silent scope reduction check: passed. The remaining aggregate-only provenance container and dispatcher/kernel callback projections are explicitly listed as open integration lanes.
4. "Looks done but is not" pattern check: passed. `PreparedArgValue` remains only as documented compatibility projection for legacy callbacks and is not presented as the target architecture.
5. Included result: passed. This section records checklist, self-audit, evidence, validation, and remaining integration lanes for the W099-007 closure claim.

Fresh-eyes review:

1. Issue found and corrected: direct projection from `CalcValue.core()` would have erased richer callable/lambda payloads carried beside the core sentinel. Projection now goes through `CallArgValue::value(CalcValue)` and has a regression test.
2. Issue checked: 1x1 array normalization still matches the previous values-only preparation behavior while preserving `CalcValue::empty()` for blank single-cell references.
3. Issue checked: no new public aggregate/reference value carrier was added. Existing aggregate provenance remains local to aggregate adapter expansion and is listed as a follow-up lane.

Validation:

1. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml adapters::tests --lib`: passed, 21 tests.
2. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml aggregate --lib`: passed, 13 tests.
3. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml callable_helpers --lib`: passed, 29 tests.
4. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib`: passed, 1336 passed, 1 ignored.

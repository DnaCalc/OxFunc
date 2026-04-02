# W47 Execution Record - Typed Context And Query Bundle Freeze

Status: `complete`
Workset: `W047`

## 1. Purpose
Freeze the first shared typed context/query bundle for the already-covered seam-heavy OxFunc scope so OxFml can wire the covered functions without inventing side channels.

## 2. Packet Outputs
Artifacts produced or updated in this packet:
1. `docs/HISTORY.md`
2. `docs/worksets/W049_RUNTIME_LIBRARY_CONTEXT_PROVIDER_CONSUMER_MODEL.md`
3. `docs/function-lane/W47_TYPED_CONTEXT_QUERY_DEPENDENCY_MAP.csv`
4. `docs/function-lane/W47_EXECUTION_RECORD.md`
5. `docs/function-lane/W47_OXFML_CONSUMER_RECONCILIATION.md`
6. `docs/function-lane/W47_CONSUMER_MISMATCH_LEDGER.csv`
7. `docs/upstream/NOTES_FOR_OXFML.md`

## 3. Current Result
The current first-freeze bundle is now explicit and no longer only implied by packet-local notes.

Pinned shared bundle members:
1. `ReferenceResolver`
2. `NowProvider`
3. `TodayProvider`
4. `RandomProvider`
5. `LocaleFormatContext`
6. `HostInfoProvider`
7. `RtdProvider`
8. `RegisteredExternalProvider`

Pinned current reading:
1. the bundle remains capability-scoped and typed,
2. the current OxFunc query names and result partitions are archived freeze provenance and are summarized by the retained `W049` runtime model,
3. OxFunc keeps query classification and worksheet result/error projection,
4. OxFml / host keeps live workbook/application/environment/provider truth.

## 4. Main Findings
1. The existing OxFunc-side query surface is already small enough to freeze honestly; no new mega-provider or raw workbook object is needed.
2. `HostInfoProvider` remains the right shared facade for:
   - `CELL`
   - `INFO`
   - `ISFORMULA`
   - `FORMULATEXT`
   - `SHEET`
   - `SHEETS`
   - `SUBTOTAL`
   - `AGGREGATE`
   - `ASC`
   - `DBCS`
   - `JIS`
   - `TRANSLATE`
3. `LocaleFormatContext` remains the right first-pass shared locale/profile carrier for omitted-default `NUMBERVALUE`.
4. `RtdProvider` remains separate rather than being folded into `HostInfoProvider`, because `RTD` is best modeled as prepared request plus typed current-outcome resolution.
5. Time/random provider traits remain narrow and separate:
   - `NowProvider`
   - `TodayProvider`
   - `RandomProvider`
6. `RegisteredExternalProvider` should also remain separate rather than being folded into `HostInfoProvider`, because registration/catalog runtime is not workbook metadata and `W046` now has a real typed runtime seam for `CALL` / `REGISTER.ID`.

## 5. Verification Basis
This packet reuses exercised evidence from the closed function packets rather than creating new function semantics.

Verified packet bases:
1. `W15_EXECUTION_RECORD.md`
2. `W34_EXECUTION_RECORD.md`
3. `W35_EXECUTION_RECORD.md`
4. `W36_EXECUTION_RECORD.md`
5. `W40_EXECUTION_RECORD.md`
6. `W43_EXECUTION_RECORD.md`
7. `W46_EXECUTION_RECORD.md`

Code surfaces reviewed:
1. `crates/oxfunc_core/src/resolver.rs`
2. `crates/oxfunc_core/src/locale_format.rs`
3. `crates/oxfunc_core/src/host_info.rs`
4. `crates/oxfunc_core/src/functions/rtd_fn.rs`
5. `crates/oxfunc_core/src/functions/call_register_id_family.rs`
6. `crates/oxfunc_core/src/functions/now_fn.rs`
7. `crates/oxfunc_core/src/functions/today_fn.rs`
8. `crates/oxfunc_core/src/functions/rand_fn.rs`

## 6. Completeness Axes
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`
5. open_lanes:
   - later witness-bearing runtime consumption remains follow-on work under `W069`, not an open current-phase freeze gap

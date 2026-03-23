# Function Slice - Typed Context And Query Bundle Contract (Prelim)

Workset: `W047`

## 1. Purpose
Freeze the first shared typed context/query bundle for the already-covered seam-heavy OxFunc scope.

The goal of this packet is not to invent a new abstraction layer. It is to pin the exact current OxFunc-side callback and context floor that the already-covered functions now depend on, so OxFml can consume the covered scope without side channels.

## 2. Bundle Members
The current first-freeze candidate is:
1. `ReferenceResolver`
2. time/random providers:
   - `NowProvider`
   - `TodayProvider`
   - `RandomProvider`
3. `LocaleFormatContext`
4. `HostInfoProvider`
5. `RtdProvider`
6. `RegisteredExternalProvider`

## 3. Current Shared Reading
1. the bundle stays capability-scoped and typed,
2. OxFunc keeps Excel semantic classification, worksheet-visible projection, and error policy,
3. OxFml / host provides the live workbook/application/environment/provider facts through typed queries or typed provider traits,
4. the current query names and result partitions are the first freeze candidate,
5. no pre-freeze merge or split is needed unless a concrete OxFml consumer mismatch appears.

## 4. Exact Current Bundle Surface
### 4.1 `ReferenceResolver`
Current trait:
1. `capabilities() -> ResolverCapabilities`
2. `resolve_reference(reference: ReferenceLike) -> Result<EvalValue, RefResolutionError>`
3. `caller_context() -> Option<CallerContext>`

### 4.2 Time / Random Provider Surface
Pinned current traits:
1. `NowProvider::now_serial() -> f64`
2. `TodayProvider::today_serial() -> f64`
3. `RandomProvider::random_unit() -> f64`

### 4.3 `LocaleFormatContext`
Pinned current fields:
1. `profile`
2. `date_system`
3. `parser`
4. `formatter`

Current emphasized profile fields for the covered seam-heavy slice:
1. `decimal_separator`
2. `thousands_separator`
3. `list_separator`
4. `currency_symbol`
5. `date_separator`
6. `time_separator`
7. `currency_decimals`

### 4.4 `HostInfoProvider`
Pinned current query families:
1. `query_cell_info(query: CellInfoQuery, reference: Option<ReferenceLike>) -> Result<EvalValue, HostInfoError>`
2. `query_info(query: InfoQuery) -> Result<EvalValue, HostInfoError>`
3. `query_formula_text(reference: ReferenceLike) -> Result<EvalValue, HostInfoError>`
4. `query_sheet_index(spec: SheetIdentitySpec) -> Result<EvalValue, HostInfoError>`
5. `query_sheet_count(spec: SheetCountSpec) -> Result<EvalValue, HostInfoError>`
6. `query_aggregate_reference_context(reference: ReferenceLike) -> Result<AggregateReferenceContext, HostInfoError>`
7. `query_width_conversion_mode(function: WidthConversionFunction) -> Result<WidthConversionMode, HostInfoError>`
8. `query_translate(request: TranslateRequest) -> Result<TranslateProviderResult, HostInfoError>`

### 4.5 `RtdProvider`
Pinned current request:
1. `RtdRequest`
   - `prog_id`
   - `server_name`
   - `topic_strings`

Pinned current result:
1. `RtdProviderResult::Value(EvalValue)`
2. `RtdProviderResult::NoValueYet`
3. `RtdProviderResult::CapabilityDenied`
4. `RtdProviderResult::ConnectionFailed`
5. `RtdProviderResult::ProviderError(WorksheetErrorCode)`

### 4.6 `RegisteredExternalProvider`
Pinned current typed surface:
1. `resolve_register_id(request: RegisterIdRequest) -> Result<RegisteredExternalDescriptor, RegisteredExternalProviderError>`
2. `lookup_registered_external(register_id: f64) -> Result<RegisteredExternalDescriptor, RegisteredExternalProviderError>`
3. `invoke_registered_external(descriptor: RegisteredExternalDescriptor, args: [CallArgValue]) -> Result<EvalValue, RegisteredExternalProviderError>`

Pinned current request/descriptor families:
1. `RegisterIdRequest`
   - `library_name`
   - `procedure`
   - optional `declared_type_text`
2. `RegisteredExternalDescriptor`
   - `stable_registration_id`
   - `register_id`
   - `origin_kind`
   - `display_name`
   - `library_name`
   - `procedure`
   - optional `declared_type_text`

## 5. Covered Function Dependencies
The current already-covered seam-heavy rows depend on the bundle as follows:
1. `CELL`
   - `ReferenceResolver`
   - `HostInfoProvider.query_cell_info`
2. `INFO`
   - `HostInfoProvider.query_info`
3. `ISFORMULA`
   - `HostInfoProvider.query_cell_info(CellInfoQuery::IsFormula, Some(reference))`
4. `FORMULATEXT`
   - `HostInfoProvider.query_formula_text`
5. `SHEET`
   - `HostInfoProvider.query_sheet_index`
6. `SHEETS`
   - `HostInfoProvider.query_sheet_count`
7. `SUBTOTAL`
   - `ReferenceResolver`
   - `HostInfoProvider.query_aggregate_reference_context`
8. `AGGREGATE`
   - `ReferenceResolver`
   - `HostInfoProvider.query_aggregate_reference_context`
9. `ASC` / `DBCS` / `JIS`
   - `HostInfoProvider.query_width_conversion_mode`
10. `NUMBERVALUE`
   - `LocaleFormatContext` for omitted-default separator lanes
11. `TRANSLATE`
   - `HostInfoProvider.query_translate`
12. `RTD`
   - `RtdProvider`
13. `NOW`
   - `NowProvider`
14. `TODAY`
   - `TodayProvider`
15. `RAND`
   - `RandomProvider`
16. `REGISTER.ID`
   - `RegisteredExternalProvider.resolve_register_id`
17. `CALL`
   - `RegisteredExternalProvider.lookup_registered_external`
   - `RegisteredExternalProvider.resolve_register_id`
   - `RegisteredExternalProvider.invoke_registered_external`

## 6. Ownership Split
OxFunc owns:
1. argument admission and coercion,
2. query-kind classification,
3. worksheet-visible result/error projection,
4. local deterministic subcases such as:
   - `TRANSLATE` same-language passthrough,
   - `ADDRESS` rendering,
   - width-conversion kernels once mode is known.

OxFml / host owns:
1. live workbook/application/environment/provider truth,
2. stored formula text retrieval,
3. sheet topology truth,
4. aggregate-region visibility and nested-aggregate context truth,
5. width-conversion profile truth,
6. translation provider invocation,
7. RTD lifecycle and current topic result resolution.
8. registered-external lookup, handle allocation, and actual external invocation.

## 7. First-Freeze Candidate Reading
The current first shared freeze candidate is:
1. keep the current query/result names,
2. keep the current typed result partitions,
3. keep the bundle capability-scoped rather than collapsing it into raw host objects,
4. only merge, split, or rename if concrete consumer modeling proves the current shape insufficient.

## 8. Evidence Posture
This packet does not claim new function evidence by itself.

It freezes one shared bundle from already exercised packet-local evidence in:
1. `docs/function-lane/CELL_INFO_HOST_QUERY_SEAM_PRELIM.md`
2. `docs/function-lane/FUNCTION_SLICE_REFERENCE_METADATA_AND_FORMULA_VISIBILITY_CONTRACT_PRELIM.md`
3. `docs/function-lane/FUNCTION_SLICE_ISFORMULA_CONTRACT_PRELIM.md`
4. `docs/function-lane/FUNCTION_SLICE_WIDTH_CONVERSION_HOST_PROFILE_CONTRACT_PRELIM.md`
5. `docs/function-lane/FUNCTION_SLICE_NUMBERVALUE_LOCALE_DEFAULT_CONTRACT_PRELIM.md`
6. `docs/function-lane/FUNCTION_SLICE_TRANSLATE_PROVIDER_LANGUAGE_CONTRACT_PRELIM.md`
7. `docs/function-lane/FUNCTION_SLICE_RTD_CONTRACT_PRELIM.md`
8. `docs/function-lane/FUNCTION_SLICE_NOW_CONTRACT_PRELIM.md`
9. `docs/function-lane/FUNCTION_SLICE_RAND_CONTRACT_PRELIM.md`
10. `docs/function-lane/FUNCTION_SLICE_CALL_REGISTER_ID_UDF_REGISTRATION_SEAM_PRELIM.md`
11. execution records:
   - `W15_EXECUTION_RECORD.md`
   - `W34_EXECUTION_RECORD.md`
   - `W35_EXECUTION_RECORD.md`
   - `W36_EXECUTION_RECORD.md`
   - `W40_EXECUTION_RECORD.md`
   - `W43_EXECUTION_RECORD.md`
   - `W46_EXECUTION_RECORD.md`

## 9. Artifact Bindings
1. workset: `docs/worksets/W047_TYPED_CONTEXT_AND_QUERY_BUNDLE_FREEZE.md`
2. dependency map: `docs/function-lane/W47_TYPED_CONTEXT_QUERY_DEPENDENCY_MAP.csv`
3. execution record: `docs/function-lane/W47_EXECUTION_RECORD.md`
4. core traits:
   - `crates/oxfunc_core/src/resolver.rs`
   - `crates/oxfunc_core/src/locale_format.rs`
   - `crates/oxfunc_core/src/host_info.rs`
   - `crates/oxfunc_core/src/functions/rtd_fn.rs`
   - `crates/oxfunc_core/src/functions/call_register_id_family.rs`
   - `crates/oxfunc_core/src/functions/now_fn.rs`
   - `crates/oxfunc_core/src/functions/today_fn.rs`
   - `crates/oxfunc_core/src/functions/rand_fn.rs`

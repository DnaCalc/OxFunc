# OxFml / OxFunc Shared Interface Freeze Candidate V1 Delta

Status: `acknowledged_alignment`
Owner lane: `OxFunc`
Date: 2026-03-31

## 1. Purpose
Record the current section-by-section comparison between:
1. OxFunc candidate: `docs/function-lane/OXFML_OXFUNC_SHARED_INTERFACE_FREEZE_CANDIDATE_V1.md`
2. OxFml mirrored packet: `../OxFml/docs/spec/formula-language/OXFML_OXFUNC_SHARED_INTERFACE_FREEZE_CANDIDATE_V1.md`

This note exists to record the local alignment pass that led to OxFml acknowledgment of `HO-FN-004`.

## 2. Overall Read
Current overall read:
1. the two packets are substantively aligned on the shared packet/carrier model,
2. the remaining differences are now mostly framing, scope presentation, and local-context detail,
3. no new broad callable, return-surface, or library-context model disagreement is visible in this comparison pass,
4. OxFml's mirrored packet now treats the archived freeze-promotion handoff behind `OxFunc_V1` as acceptable current shared freeze wording for the narrowed seam families.

## 3. Section Comparison

### 3.1 Purpose And Scope
Status: `minor wording delta`

Aligned substance:
1. both packets treat the artifact as a current-phase shared freeze candidate rather than a product API,
2. both packets treat the scope as the narrowed seam families rather than the whole catalog.

Remaining difference:
1. the OxFunc note also carries local acknowledgement-scope context for:
   - the explicit interesting `W051` backlog,
   - the excluded `185` ordinary backlog rows,
   - the excluded `17` deferred rows.
2. the OxFml note omits those W051/W050 counts and stays purely on the shared seam family set.

Promotion reading:
1. this is not a model disagreement,
2. the W051/W050 exclusion framing can remain OxFunc-local context rather than shared freeze text if desired.

### 3.2 Typed Context / Query Bundle
Status: `aligned after narrowing`

Shared packet:
1. `ReferenceResolver`
2. `CellInfo`
3. `Info`
4. `Rtd`
5. `Image`

Shared working rules:
1. capability-scoped and typed
2. `TypedContextQueryFamily::Image` is the first freeze name
3. preserved reference identity remains explicit where required
4. broader query families remain outside the current freeze candidate

Promotion reading:
1. this section is ready for shared freeze wording.

### 3.3 Returned Value Surface
Status: `aligned`

Shared split:
1. `OrdinaryValue`
2. `ValueWithPresentation`
3. `RichValue`
4. `TypedHostProviderOutcome`

Shared consequences:
1. `HYPERLINK` preserves `ValueWithPresentation`
2. `IMAGE` preserves `RichValue` with `_webimage`
3. published worksheet fallback remains separate from the semantic return carrier

Promotion reading:
1. this section is ready for shared freeze wording.

### 3.4 Runtime Library Context Consumer Shape
Status: `aligned after explicit `LibraryContextSnapshotRef` addition`

Shared runtime direction:
1. `LibraryContextProvider`
2. immutable `LibraryContextSnapshot`
3. `LibraryContextSnapshotRef`
4. runtime-only consumer shape preferred over direct CSV mirroring

Shared working rules:
1. pinned CSV export remains a stabilization/test-pinning artifact
2. snapshot identity and generation remain explicit
3. richer export-description fields may remain outside the runtime-only shared minimum where explicit mapping preserves meaning

Promotion reading:
1. this section is ready for shared freeze wording.

### 3.5 Minimum Callable Carrier
Status: `aligned`

Shared minimum carrier:
1. opaque callable identity/token
2. `origin_kind`
3. `capture_mode`
4. `arity_shape`
5. `invocation_contract_ref`

Shared working rules:
1. typed invocation over opaque callable identity is the preferred boundary
2. no extra explicit invocation-model field is currently needed
3. parameter/capture/body detail remains provenance/replay detail for the current phase
4. defined-name callable preservation is first-pass callable freeze pressure, not a later extension

Promotion reading:
1. this section is ready for shared freeze wording.

### 3.6 Registered External Runtime Packet Family
Status: `aligned with minor presentation delta`

Shared direct packet set:
1. `RegisterIdRequest`
2. `RegisteredExternalDescriptor`
3. `RegisteredExternalCallRequest`
4. `RegisteredExternalTarget`
5. `RegisteredExternalProvider`

Shared working rules:
1. `RegisteredExternalProvider` remains separate from `HostInfoProvider`
2. descriptor-driven dereference and general type coercion remain OxFunc-owned
3. the seven-field `RegisteredExternalDescriptor` is the shared minimum field set
4. OxFml mutation/controller packets remain OxFml-owned for the current phase
5. bind-visible registration/unregister yields new snapshot generation plus bind invalidation
6. `CALL` / `REGISTER.ID`-only descriptor mutation yields targeted reevaluation by default

Remaining difference:
1. the OxFml packet explicitly lists the adjacent OxFml funnel packet family,
2. the OxFunc note now mirrors that list, so the remaining difference is only presentation, not substance.

Promotion reading:
1. this section is ready for shared freeze wording.

### 3.7 Boundary Non-Claims / Remaining Out-Of-Model Work
Status: `presentation delta`

OxFml packet emphasizes:
1. boundary non-claims
2. current promotion gap

OxFunc packet emphasizes:
1. excluded W051/W050 scope context
2. remaining non-interface work after freeze acceptance

Promotion reading:
1. these are complementary rather than conflicting,
2. the shared freeze text can take the OxFml-style non-claims,
3. the OxFunc remaining-work list can stay local packet context in `W051`.

## 4. Recommended Promotion Text
Recommended shared promotion text:
1. “The current-phase shared interface freeze candidate covers the narrowed seam families only: typed context/query bundle, returned-value surface, runtime library-context consumer shape, minimum callable carrier, and the registered-external `CALL` / `REGISTER.ID` packet family.”
2. “The packet and carrier set in this candidate is converged enough to promote from note-level convergence into explicit shared seam-freeze text.”
3. “Later widening is mismatch-driven rather than theory-driven.”

## 5. Current Remaining Gap
The remaining gap is now:
1. explicit promotion of the acknowledged shared freeze wording into the owner packets and seam ledgers,
2. any future widening or correction only if a concrete mismatch appears later.

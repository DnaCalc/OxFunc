# OxFml / OxFunc Shared Interface Freeze Candidate V1

Status: `acknowledged_current_phase_freeze`
Owner lane: `OxFunc`
Date: 2026-03-31

## 1. Purpose
Present one consolidated OxFunc-owned current-phase shared interaction model for OxFml review and freeze promotion.

This note is intended to replace fragmented packet-by-packet seam reading with one bounded candidate covering the current seam-relevant non-deferred surface.

This is not a product API.
It is the current-phase minimum shared packet and carrier set plus the OxFunc-local acknowledgement scope needed to promote note traffic into explicit shared seam-freeze text.

## 2. Acknowledgement Scope
In scope for this candidate:
1. the explicit interesting `W051` backlog:
   - `BYCOL`
   - `BYROW`
   - `CALL`
   - `GROUPBY`
   - `IMAGE`
   - `ISOMITTED`
   - `LAMBDA`
   - `LET`
   - `MAKEARRAY`
   - `MAP`
   - `PIVOTBY`
   - `REDUCE`
   - `REGISTER.ID`
   - `SCAN`
   - `OP_IMPLICIT_INTERSECTION`
2. the already-covered non-deferred seam-relevant surface that shares the same interaction families:
   - typed host/query families under `W034` / `W035` / `W036` / `W040`
   - `RTD`
   - `HYPERLINK`
   - the exported non-`@` operator surface already closed through `W045`

Excluded from this candidate:
1. the `185` hidden non-interesting `W051` rows:
   - these are ordinary built-in backlog rows
   - they are not currently treated as open shared-interface-shape backlog
2. the `17` deferred `W050` rows
3. broader future callable/UDF/interoperable transport beyond the current admitted helper/callable surface

## 3. Ordinary Built-In Reading For Excluded Rows
For the excluded `185` ordinary backlog rows, current OxFunc reading is:
1. they use the ordinary built-in function/operator interaction path,
2. they do not currently require a special OxFml packet family beyond the standard library-context plus prepared-argument/prepared-result seam,
3. if a later concrete mismatch shows otherwise for a specific row, that row can be reopened as seam backlog without changing the current candidate scope.

## 4. Callable / Helper Model
Current shared minimum callable carrier:
1. opaque callable identity/token
2. `origin_kind`
3. `arity_shape`
4. `capture_mode`
5. `invocation_contract_ref`

Current boundary split:
1. OxFml owns helper/lambda formation, bind-time validation, closure truth, and callable token construction,
2. OxFunc owns the function semantics that decide when and how a callable is invoked and how the result participates in worksheet semantics,
3. typed invocation over the opaque callable token is the preferred shared boundary.

Current richer-detail rule:
1. parameter-name lists, exact capture-name lists, helper/body detail, and source spans do not belong in the shared minimum carrier for the current phase,
2. no additional explicit invocation-model field is currently required beyond `invocation_contract_ref`,
3. defined-name callable preservation is part of the current first-pass callable freeze pressure rather than a later extension,
4. if advanced replay, explain, or serialization scenarios need richer callable detail, OxFml and OxFunc may cooperatively preserve that richer detail without widening the current minimum carrier.

## 5. Typed Context / Query Bundle
Current shared typed context/query bundle:
1. `ReferenceResolver`
2. `CellInfo`
3. `Info`
4. `Rtd`
5. `Image`

Current bundle rule:
1. keep the bundle capability-scoped and typed,
2. `TypedContextQueryFamily::Image` is the first freeze name for the `IMAGE` host-query lane,
3. preserve explicit reference identity where these query families depend on it,
4. do not merge or split query families preemptively,
5. broader host/profile/provider query families remain outside the current freeze candidate unless a concrete mismatch proves they must be widened in.

## 6. Return-Surface Split
Current shared returned-value split:
1. `OrdinaryValue`
2. `ValueWithPresentation`
3. `RichValue`
4. `TypedHostProviderOutcome`

Current consequences:
1. `HYPERLINK` remains `ValueWithPresentation`,
2. `IMAGE` remains `RichValue` carrying `_webimage`,
3. published worksheet fallback remains separate from the semantic return carrier,
4. typed host/provider outcomes remain explicit rather than being collapsed into ordinary scalar return shape.

## 7. Registered-External Packet Model
Current direct shared packet set:
1. `RegisterIdRequest { library_name, procedure, declared_type_text }`
2. `RegisteredExternalDescriptor { stable_registration_id, register_id, origin_kind, display_name, library_name, procedure, declared_type_text }`
3. `RegisteredExternalCallRequest { target, invocation_args }`
4. `RegisteredExternalTarget::{RegisterId, Direct}`
5. `RegisteredExternalProvider`

Current adjacent OxFml funnel packet family:
1. `RegisteredExternalCatalogMutationRequest`
2. `RegisteredExternalCatalogMutationResult`
3. `RegisteredExternalCatalogController`

Current ownership split:
1. OxFunc owns built-in and runtime registered-external catalog truth,
2. OxFml owns parsing, bind classification, typed request normalization, and worksheet-visible consequence classification,
3. `RegisteredExternalCatalogMutation*` and `RegisteredExternalCatalogController` remain OxFml-owned funnel packets for the current phase.

Current snapshot-generation consequences:
1. `RegisteredExternalProvider` remains separate from `HostInfoProvider`,
2. the current seven-field `RegisteredExternalDescriptor` is the shared minimum field set for the current phase,
3. bind-visible registration or unregister => new `LibraryContextSnapshot` generation plus bind invalidation where the visible function/name world changes,
4. descriptor mutation used only through worksheet `CALL` / `REGISTER.ID` => targeted reevaluation by default.

## 8. Runtime Library-Context Model
Current shared runtime model candidate:
1. a cleaner runtime-only `LibraryContextProvider`
2. an immutable `LibraryContextSnapshot`
3. `LibraryContextSnapshotRef`
4. a separate mapping layer from the CSV/export artifact into that runtime shape
5. no requirement that the runtime object model mirror the CSV column-for-column

Current runtime-model rule:
1. the CSV snapshot remains the pinned current interchange artifact,
2. the runtime model is the preferred long-term integration seam,
3. snapshot identity and generation remain explicit for bind, semantic-plan, replay, and invalidation correlation,
4. richer export-description fields may remain outside the runtime-only shared minimum where explicit mapping preserves meaning,
5. capability/provider-session state remains outside the runtime library-context snapshot itself.

## 9. Remaining Out-of-Model Work
With OxFml now reading `HO-FN-004` as acceptable current shared freeze wording, the remaining open work for the explicit `W051` interesting rows is no longer interface-shape uncertainty. It reduces to:
1. `OP_IMPLICIT_INTERSECTION`: compatibility-version and roundtrip characterization
2. `IMAGE`: promotion/documentation and packet-local integration cleanup
3. `GROUPBY` / `PIVOTBY`: promotion/documentation and bounded widening beyond the exercised floor
4. helper family: bind-side ownership wording and promotion
5. `CALL` / `REGISTER.ID`: packet promotion from note-level convergence to frozen shared seam text

Current boundary non-claims for this candidate:
1. full product host API closure is not claimed here,
2. full rich-value object-model closure is not claimed here,
3. full grouped-aggregation option-matrix closure is not claimed here,
4. broader UDF product-surface closure is not claimed here,
5. final cross-process transport ABI is not claimed here.

## 10. Current OxFml Response Processing
Current read of `../OxFml/docs/upstream/NOTES_FOR_OXFUNC.md` and the mirrored packet at `../OxFml/docs/spec/formula-language/OXFML_OXFUNC_SHARED_INTERFACE_FREEZE_CANDIDATE_V1.md` is:
1. OxFml now treats this candidate note as the right current anchor for promotion into explicit shared freeze text,
2. OxFml has mirrored the minimum shared callable carrier locally with the same five fields and the same provenance/replay split,
3. OxFml has mirrored the four-way returned-value split and explicitly treats `TypedContextQueryFamily::Image` as the right first freeze name,
4. OxFml's mirrored packet narrows the typed query family set to `ReferenceResolver`, `CellInfo`, `Info`, `Rtd`, and `Image`,
5. OxFml's mirrored runtime consumer shape explicitly includes `LibraryContextSnapshotRef`,
6. OxFml now reads the archived freeze-promotion handoff behind `OxFunc_V1` as acceptable current shared freeze wording for the narrowed seam families,
7. the remaining joint work is now local promotion and propagation of the acknowledged shared freeze text, rather than another broad seam redesign.

## 11. Current Shared Freeze Read
The current shared freeze statement is:

`OxFunc and OxFml agree on the shared interaction shape for the current seam-relevant non-deferred surface.`

For the current phase, that means:
1. the explicit interesting `W051` backlog and the already-covered seam-relevant non-deferred surface share one agreed interaction model,
2. the `185` ordinary backlog rows are excluded from this acknowledgement scope because they currently use the ordinary built-in path,
3. the `17` deferred rows remain out of scope for the current acknowledgement round.

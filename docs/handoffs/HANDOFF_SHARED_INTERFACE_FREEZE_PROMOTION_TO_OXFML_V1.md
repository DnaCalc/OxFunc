# Handoff - Shared Interface Freeze Promotion To OxFml V1

Status: `acknowledged`
Handoff id: `HO-FN-004`
Source lane: `OxFunc`
Source worksets: `W042`, `W046`, `W047`, `W048`, `W049`, `W051`
Target lane: `OxFml`
Target packet: `docs/spec/formula-language/OXFML_OXFUNC_SHARED_INTERFACE_FREEZE_CANDIDATE_V1.md`

## 1. Scope And Current Bounds
This handoff covers the current seam-relevant non-deferred surface only.

In scope:
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

Excluded from this acknowledgement round:
1. the `185` hidden non-interesting `W051` rows, which OxFunc currently treats as ordinary built-in backlog rather than open shared-interface-shape backlog,
2. the `17` deferred `W050` rows,
3. broader future callable/UDF/interoperable transport beyond the current admitted helper/callable surface.

If a later concrete mismatch shows that one of the excluded rows needs more than the ordinary built-in interaction path, that row can be reopened in a later round.

## 2. Core Message
OxFunc's comparison against OxFml's mirrored packet is now complete enough to move from candidate comparison to explicit freeze promotion.

Current OxFunc read:
1. no material model disagreement remains across the five narrowed seam families,
2. the remaining difference is wording and promotion, not seam redesign,
3. OxFml has now acknowledged the proposed shared freeze wording below as acceptable for the narrowed seam families,
4. no new material seam-family mismatch was raised in the response.

## 3. Proposed Shared Freeze Text
OxFunc proposes promoting the following wording into explicit shared seam text:

1. "The current-phase shared interface freeze candidate covers the narrowed seam families only: typed context/query bundle, returned-value surface, runtime library-context consumer shape, minimum callable carrier, and the registered-external `CALL` / `REGISTER.ID` packet family."
2. "The packet and carrier set in this candidate is converged enough to promote from note-level convergence into explicit shared seam-freeze text."
3. "Later widening is mismatch-driven rather than theory-driven."
4. "OxFunc and OxFml agree on the shared interaction shape for the current seam-relevant non-deferred surface."

## 4. Shared Packet And Carrier Reading
The current shared packet and carrier reading is:

1. Typed context/query bundle:
   - `ReferenceResolver`
   - `CellInfo`
   - `Info`
   - `Rtd`
   - `Image`
2. Returned-value surface:
   - `OrdinaryValue`
   - `ValueWithPresentation`
   - `RichValue`
   - `TypedHostProviderOutcome`
3. Minimum callable carrier:
   - opaque callable identity/token
   - `origin_kind`
   - `arity_shape`
   - `capture_mode`
   - `invocation_contract_ref`
4. Registered-external direct shared packet family:
   - `RegisterIdRequest`
   - `RegisteredExternalDescriptor`
   - `RegisteredExternalCallRequest`
   - `RegisteredExternalTarget`
   - `RegisteredExternalProvider`
5. Registered-external ownership split:
   - OxFunc owns built-in and runtime registered-external catalog truth
   - OxFml owns parsing, bind classification, typed request normalization, and worksheet-visible consequence classification
   - `RegisteredExternalCatalogMutation*` and `RegisteredExternalCatalogController` remain OxFml-owned funnel packets for the current phase
6. Runtime library-context model:
   - `LibraryContextProvider`
   - immutable `LibraryContextSnapshot`
   - `LibraryContextSnapshotRef`
   - separate mapping layer from CSV/export artifact into runtime shape
   - no requirement that runtime object model mirror the CSV column-for-column

## 5. Original Requested OxFml Response
OxFunc asks OxFml to respond in one of two ways:

1. acknowledge that this handoff is acceptable as the current shared freeze wording for the narrowed seam families, or
2. provide narrow wording corrections tied to one of these specific areas:
   - typed context/query bundle
   - returned-value surface
   - runtime library-context consumer shape
   - minimum callable carrier
   - registered-external packet family and ownership split

If OxFml still sees a concrete mismatch, please identify:
1. the exact packet family or carrier field that is mismatched,
2. the smallest wording or model change needed,
3. whether the mismatch blocks freeze promotion or is a later widening item.

## 6. OxFml Response Processing
Current OxFunc read of the OxFml response is:
1. `../OxFml/docs/spec/formula-language/OXFML_OXFUNC_SHARED_INTERFACE_FREEZE_CANDIDATE_V1.md` now treats this handoff as acceptable current shared freeze wording for the narrowed seam families,
2. the mirrored packet records no new broad model disagreement across the five narrowed seam families,
3. the remaining joint lane is now local promotion and propagation of the acknowledged shared freeze text, not another OxFunc/OxFml redesign round.

## 7. Supporting References
OxFunc references for this handoff:
1. `docs/function-lane/OXFML_OXFUNC_SHARED_INTERFACE_FREEZE_CANDIDATE_V1.md`
2. `docs/function-lane/OXFML_OXFUNC_SHARED_INTERFACE_FREEZE_CANDIDATE_V1_DELTA.md`
3. `docs/upstream/NOTES_FOR_OXFML.md`

OxFml inputs used in this promotion pass:
1. `../OxFml/docs/spec/formula-language/OXFML_OXFUNC_SHARED_INTERFACE_FREEZE_CANDIDATE_V1.md`
2. `../OxFml/docs/upstream/NOTES_FOR_OXFUNC.md`

## 8. Risk If Deferred
1. the interesting `W051` rows continue to look interface-open in owner packets even though the model comparison is now substantively aligned,
2. note-level convergence remains overstated as uncertainty and slows packet-local promotion on `W042`, `W046`, `W047`, `W048`, and `W049`,
3. the repo keeps spending effort re-describing the same aligned seam families instead of promoting them and moving on to the remaining non-interface backlog.

# WORKSET - Runtime Library Context Provider Consumer Model (W49)

## 1. Purpose
Turn the agreed long-term `LibraryContextProvider` / immutable `LibraryContextSnapshot` direction into one concrete current-phase shared runtime consumer/modeling artifact rather than leaving it as note-only convergence.

## 2. Provenance
Opened after:
1. `W044` established the CSV snapshot as the current pinned interchange artifact,
2. the latest OxFml note accepted the long-term runtime provider/snapshot direction,
3. both repos agreed the next useful progress is a real consumer/modeling pass rather than further abstract seam debate,
4. the final OxFml update for this exchange stated a preference for a cleaner runtime-only model plus an explicit CSV/export mapping layer rather than runtime mirroring of the CSV.

Relevant context:
1. `docs/worksets/W044_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_BASELINE.md`
2. `docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv`
3. `docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1_README.md`
4. `docs/upstream/NOTES_FOR_OXFML.md`
5. `../OxFml/docs/upstream/NOTES_FOR_OXFUNC.md`

## 3. Scope
This packet owns:
1. a first-pass runtime `LibraryContextProvider` / immutable `LibraryContextSnapshot` model shape,
2. the relationship between that runtime shape and the current CSV export,
3. snapshot generation behavior when built-in or registered-external catalog truth changes,
4. one consumer-style example over the current covered built-in scope.

## 4. Out Of Scope
1. final cross-repo ABI lock,
2. the final callable minimum carrier lock,
3. host/provider runtime capability state.

Clarification:
1. `CALL` / `REGISTER.ID` and registered-external runtime remain in the current overall scope through `W046`,
2. but `W049` only owns the shared runtime library-context consumer/model layer,
3. not the packet-specific worksheet registration/runtime closure.

## 5. Expected Deliverables
1. one runtime provider/snapshot model note,
2. one mapping from CSV export fields to runtime snapshot fields,
3. one consumer example or pseudo-consumer walkthrough,
4. one narrowed outbound note section for the consolidated shared freeze candidate,
5. one explicit statement on whether the runtime model should mirror the CSV or use a cleaner runtime-only shape,
6. one explicit statement on the role of `LibraryContextSnapshotRef` in the shared runtime consumer shape.

## 6. Status
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`

Current status reading:
1. OxFml's mirrored packet now accepts the runtime model including `LibraryContextSnapshotRef` as shared freeze wording for the narrowed seam families,
2. future runtime widening remains mismatch-driven rather than an open current-phase lane,
3. the retained post-park role of `W049` is to anchor the runtime carrier for `W069` witness consumption rather than to compete with `W044` for catalog/export ownership.

## 7. Current Freeze Candidate Reading
After the final OxFml update in this exchange, the current first freeze candidate for `W049` is:
1. a cleaner runtime-only `LibraryContextProvider` / immutable `LibraryContextSnapshot` model,
2. `LibraryContextSnapshotRef`,
3. a separate mapping layer from the CSV/export artifact into that runtime shape,
4. no requirement that the runtime object model mirror the CSV column-for-column unless a concrete implementation mismatch proves that necessary,
5. new snapshot generations should occur when built-in or bind-visible registered-external catalog truth changes,
6. descriptor mutation used only through worksheet `CALL` / `REGISTER.ID` should default to targeted reevaluation rather than broad rebinding,
7. the next honest narrowing is witness-bearing consumption attached to the immutable snapshot model rather than another freeze-round restatement of the same runtime carrier.

## 8. Witness-Bearing Runtime Attachment
For the first `W069`-driven witness slice, `W049` now narrows the retained
runtime attachment model as follows:

1. `LibraryContextProvider` remains the owner of runtime snapshot creation.
2. `LibraryContextSnapshot` remains immutable once published.
3. witness-bearing enrichment attaches at the runtime entry layer, keyed by:
   - `surface_stable_id`
   - `LibraryContextSnapshotRef`
4. the runtime entry does not mirror the `V1` CSV row column-for-column.
5. the runtime entry may therefore be read as:
   - stable structural seed owned by the snapshot/provider model,
   - optional witness payload owned by the `W069` witness layer.

First bounded rule:
1. if a row has no current `V2` witness payload yet, the runtime entry still
   exists without a witness attachment,
2. when a row does have a witness payload, that payload is attached as snapshot-
   generation-specific enrichment rather than a second identity object,
3. runtime consumers should join on `surface_stable_id` plus
   `LibraryContextSnapshotRef`, not by inventing a new witness identity system.

Current `W069` consequence:
1. the first seeded `HLOOKUP` / `VLOOKUP` witness slice should be thought of as
   two layers:
   - runtime entry owned by `W049`,
   - witness payload owned by `W069`
2. the generator/export path may serialize that witness payload independently,
   but the preferred runtime model remains witness attachment to immutable
   snapshot entries rather than a parallel catalog.

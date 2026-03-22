# WORKSET - Runtime Library Context Provider Consumer Model (W49)

## 1. Purpose
Turn the agreed long-term `LibraryContextProvider` / immutable `LibraryContextSnapshot` direction into one concrete first-pass runtime consumer/modeling artifact rather than leaving it as note-only convergence.

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
2. full registered-external runtime support,
3. the final callable minimum carrier lock,
4. host/provider runtime capability state.

## 5. Expected Deliverables
1. one runtime provider/snapshot model note,
2. one mapping from CSV export fields to runtime snapshot fields,
3. one consumer example or pseudo-consumer walkthrough,
4. one narrowed outbound note section for the final OxFunc response in this exchange,
5. one explicit statement on whether the runtime model should mirror the CSV or use a cleaner runtime-only shape.

## 6. Initial Status
1. execution_state: `planned`
2. scope_completeness: `scope_partial`
3. target_completeness: `target_partial`
4. integration_completeness: `partial`
5. open_lanes:
   - no runtime provider/snapshot model artifact exists yet
   - no field-by-field CSV-to-runtime mapping exists yet
   - no consumer example is pinned yet

## 7. Current Freeze Candidate Reading
After the final OxFml update in this exchange, the current first freeze candidate for `W049` is:
1. a cleaner runtime-only `LibraryContextProvider` / immutable `LibraryContextSnapshot` model,
2. a separate mapping layer from the CSV/export artifact into that runtime shape,
3. no requirement that the runtime object model mirror the CSV column-for-column unless a concrete implementation mismatch proves that necessary.

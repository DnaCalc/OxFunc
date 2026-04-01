# WORKSET - Typed Context And Query Bundle Freeze (W47)

## 1. Purpose
Package and promote the first shared typed context/query bundle for the already-covered seam-heavy OxFunc scope so OxFml can wire the completed functions without inventing side channels.

## 2. Provenance
Opened after:
1. `W044` established the first real downstream library-context snapshot export,
2. `W034` / `W035` / `W036`, `W040`, and `W043` closed the current OxFunc-side host/profile/provider/query seams,
3. the latest OxFml note explicitly accepted the current first-freeze working rule and identified the typed context/query bundle as the next lock lane,
4. the final OxFml update for this exchange accepted the current OxFunc query names and result partitioning as the first freeze candidate, subject only to concrete consumer mismatches.

Relevant context:
1. `docs/upstream/NOTES_FOR_OXFML.md`
2. `../OxFml/docs/upstream/NOTES_FOR_OXFUNC.md`
3. `docs/worksets/W044_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_BASELINE.md`
4. `docs/worksets/W043_RTD_COM_ACTIVATION_AND_TOPIC_LIFECYCLE_SEAM.md`
5. `docs/worksets/W040_REFERENCE_METADATA_AND_FORMULA_VISIBILITY_FUNCTIONS.md`
6. `docs/worksets/W034_DEFERRED_WIDTH_CONVERSION_HOST_PROFILE_CAPABILITY_BASELINE.md`
7. `docs/worksets/W035_DEFERRED_NUMBERVALUE_LOCALE_DEFAULT_PROFILE_BASELINE.md`
8. `docs/worksets/W036_DEFERRED_PROVIDER_LANGUAGE_CAPABILITY_BASELINE.md`

## 3. Scope
This packet owns the bounded shared typed context/query bundle for the current-phase freeze candidate:
1. `ReferenceResolver`
2. `CellInfo`
3. `Info`
4. `Rtd`
5. `Image`

Current working split:
1. preserved reference identity remains explicit where these query families depend on it,
2. `CellInfo` and `Info` cover the admitted host-observing worksheet slice for the current freeze candidate,
3. `Rtd` remains a typed provider lane rather than a generic host query,
4. `Image` is the first freeze name for the `IMAGE` host-query lane,
5. broader host/profile/provider query families remain outside this current-phase packet freeze candidate unless a later concrete mismatch proves they must be widened back in.

## 4. Out Of Scope
1. final callable carrier lock,
2. broader host/profile/provider query families beyond the current mirrored freeze candidate,
3. broader provider/subscription generalization beyond the already-covered scope.

Clarification:
1. `IMAGE` and `@` remain in the current overall program scope but are not owned by this packet,
2. `CALL` / `REGISTER.ID` stay primarily owned by `W046`,
3. however the shared `RegisteredExternalProvider` bundle member belongs here because OxFml needs it in the first frozen typed runtime bundle,
4. the primary owners remain:
   - `W046` for `CALL` / `REGISTER.ID` provenance and future widening,
   - `W023` provenance for `IMAGE`,
   - `W014` for implicit intersection / `@`.

## 5. Expected Deliverables
1. one shared typed context/query bundle note with explicit query names and result types,
2. one local reconciliation artifact showing which completed function families depend on which bundle members,
3. any OxFunc-local naming or shape adjustments needed to make the shared bundle honest,
4. one narrowed outbound note section for the final OxFunc response in this exchange,
5. one explicit statement of which current query/result names are treated as the first freeze candidate.

## 6. Initial Status
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`

Current status reading:
1. OxFml's mirrored packet now accepts the current typed bundle as shared freeze wording for the narrowed seam families,
2. future widening remains mismatch-driven rather than an open current-phase lane.

## 7. Current Freeze Candidate Reading
After the final OxFml update in this exchange, the current first freeze candidate for `W047` is:
1. keep the typed bundle at:
   - `ReferenceResolver`
   - `CellInfo`
   - `Info`
   - `Rtd`
   - `Image`
2. keep the bundle capability-scoped and typed,
3. preserve explicit reference identity where query meaning depends on it,
4. only change the shape if a concrete OxFml consumer mismatch appears,
5. treat this as the current shared bundle model for the consolidated freeze-candidate note rather than as an exploratory sketch.

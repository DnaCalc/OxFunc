# WORKSET - Typed Context And Query Bundle Freeze (W47)

## 1. Purpose
Lock the first shared typed context/query bundle for the already-covered seam-heavy OxFunc scope so OxFml can wire the completed functions without inventing side channels.

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
This packet owns the bounded shared context/query bundle for the already-covered scope:
1. `ReferenceResolver`
2. host time/random inputs already assumed by covered functions
3. `LocaleFormatContext`
4. `HostInfoProvider` queries currently needed for:
   - `CELL` / `INFO`
   - `ISFORMULA`
   - `FORMULATEXT`
   - `SHEET` / `SHEETS`
   - `SUBTOTAL` / `AGGREGATE`
   - `ASC` / `DBCS` / `JIS`
   - `NUMBERVALUE`
   - `TRANSLATE`
5. `RtdProvider` request/result shape for `RTD`

## 4. Out Of Scope
1. final callable carrier lock,
2. final registration/runtime for `CALL` / `REGISTER.ID`,
3. rich-value publication for `IMAGE`,
4. implicit intersection / `@`,
5. broader provider/subscription generalization beyond the already-covered scope.

## 5. Expected Deliverables
1. one shared typed context/query bundle note with explicit query names and result types,
2. one local reconciliation artifact showing which completed function families depend on which bundle members,
3. any OxFunc-local naming or shape adjustments needed to make the shared bundle honest,
4. one narrowed outbound note section for the final OxFunc response in this exchange,
5. one explicit statement of which current query/result names are treated as the first freeze candidate.

## 6. Initial Status
1. execution_state: `planned`
2. scope_completeness: `scope_partial`
3. target_completeness: `target_partial`
4. integration_completeness: `partial`
5. open_lanes:
   - no packet-specific shared bundle artifact exists yet
   - no machine-readable dependency map exists yet
   - current query/result naming is spread across several packet-local contract notes
   - no explicit freeze-candidate list is pinned yet for the current query/result names

## 7. Current Freeze Candidate Reading
After the final OxFml update in this exchange, the current first freeze candidate for `W047` is:
1. keep the bundle capability-scoped and typed,
2. start from the current OxFunc query names and result partitions,
3. do not merge or split query families preemptively,
4. only change the shape if a concrete OxFml consumer mismatch appears.

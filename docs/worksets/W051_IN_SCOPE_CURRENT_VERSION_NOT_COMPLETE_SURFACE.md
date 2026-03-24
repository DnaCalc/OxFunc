# WORKSET - In-Scope Current-Version Not-Complete Surface (W51)

## 1. Purpose
Centralize the Excel function and operator rows that are still in scope for the current OxFunc version target and are not yet fully complete.

This packet exists to stop older family packets from acting as the active outstanding-scope list.

## 2. Provenance
This packet consolidates active current-version backlog ownership from:
1. `W014_IMPLICIT_INTERSECTION_OPERATOR.md`
2. `W023_DEFERRED_HOST_METADATA_AND_DATABASE_FUNCTIONS.md`
3. `W025_DEFERRED_MISC_ADDIN_AND_DYNAMIC_ARRAY_OUTLIERS.md`
4. `W038_FUNCTIONAL_LAMBDA_AND_HELPER_FAMILY.md`
5. `W045_NON_AT_OPERATOR_UNIVERSE_CLOSURE_PASS.md`
6. `W046_CALL_AND_REGISTER_ID_UDF_REGISTRATION_SEAM.md`
7. latent catalog gaps visible through `W044`

## 3. Scope
Machine-readable inventory:
1. `docs/function-lane/W51_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_INVENTORY.csv`

Current total:
1. `21` function rows.
2. `4` operator rows.
3. `25` total rows.

Functions:
1. `BYCOL`
2. `BYROW`
3. `CALL`
4. `COLUMNS`
5. `GROUPBY`
6. `HYPERLINK`
7. `IMAGE`
8. `ISOMITTED`
9. `LAMBDA`
10. `LET`
11. `MAKEARRAY`
12. `MAP`
13. `PIVOTBY`
14. `RANDARRAY`
15. `RANDBETWEEN`
16. `REDUCE`
17. `REGISTER.ID`
18. `ROWS`
19. `SCAN`
20. `TRIMRANGE`
21. `VALUETOTEXT`

Operators:
1. `OP_IMPLICIT_INTERSECTION` (`@`, legacy alias `SINGLE`)
2. `OP_TRIM_REF_LEADING`
3. `OP_TRIM_REF_TRAILING`
4. `OP_TRIM_REF_BOTH`

## 4. Current-Version Rule
For the current version target:
1. every row not listed in `W050` and not already complete must appear here,
2. trim-reference operators are treated as still-open current-version work even though `W045` closed a bounded structural slice,
3. `PIVOTBY` is not complete and belongs here,
4. `RANDARRAY` is not deferred for current-version tracking and belongs here.

## 5. Ownership Rule
1. `W51` is the canonical current-version not-complete list.
2. Older packets remain provenance/evidence owners and, where applicable, execution owners for their family-specific work.
3. New latent gaps should be added here immediately, then extracted into narrower execution packets as needed.

## 6. Status
1. execution_state: `in_progress`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`
5. open_lanes:
   - execution still lives across the provenance packets and future follow-on packets

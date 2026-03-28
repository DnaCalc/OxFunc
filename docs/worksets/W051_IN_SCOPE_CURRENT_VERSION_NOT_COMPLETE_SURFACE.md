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
1. `14` function rows.
2. `1` operator rows.
3. `15` total rows.

Completed and removed from this inventory (moved to function-phase-complete):
- `COLUMNS`, `RANDARRAY`, `RANDBETWEEN`, `ROWS`, `TRIMRANGE`, `VALUETOTEXT` (6 functions)
- `OP_TRIM_REF_LEADING`, `OP_TRIM_REF_TRAILING`, `OP_TRIM_REF_BOTH` (3 operators, verified against W045 structural slice)

Runtime-partial (remain in inventory):
- `GROUPBY`, `PIVOTBY` (2 functions, OxFunc now has callable-backed runtime kernels and bounded OxFml adapter coverage on the admitted current-baseline slice; the remaining gap is wider completion promotion/documentation rather than first adapter proof)

Important current reading:
- some rows remain here because the cross-repo/current-surface packet is not yet fully closed, not because OxFunc still lacks a real runtime kernel.
- that narrower reading now applies to:
  - `OP_IMPLICIT_INTERSECTION`
  - the callable-helper family rows from `W038`
  - `CALL` / `REGISTER.ID`
- the rows that still represent a genuinely open current-surface boundary are now mainly:
  - `IMAGE` first-freeze/promotion work after the newly exercised local OxFml lane
  - the registered-external packet-freeze/admission/snapshot lane under `W046`
  - the broader promotion/documentation lane for `GROUPBY` / `PIVOTBY`

Functions:
1. `BYCOL`
2. `BYROW`
3. `CALL`
4. `GROUPBY`
5. `IMAGE`
6. `ISOMITTED`
7. `LAMBDA`
8. `LET`
9. `MAKEARRAY`
10. `MAP`
11. `PIVOTBY`
12. `REDUCE`
13. `REGISTER.ID`
14. `SCAN`

Operators:
1. `OP_IMPLICIT_INTERSECTION` (`@`, legacy alias `SINGLE`)

## 4. Current-Version Rule
For the current version target:
1. every row not listed in `W050` and not already complete must appear here,
2. `GROUPBY` and `PIVOTBY` are no longer W038-blocked scaffolds; they remain here because their current-surface promotion packet is still open,
3. `HYPERLINK` is now treated as complete on the OxFunc side and therefore removed from `W051`; host publication application remains above OxFunc rather than an OxFunc function gap,
4. `ROWS`, `COLUMNS`, `RANDBETWEEN`, `VALUETOTEXT`, `RANDARRAY`, `TRIMRANGE` are now function-phase-complete and removed,
5. trim-reference operators (`OP_TRIM_REF_*`) are verified against W045 structural slice and removed.

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
   - several rows now have real OxFunc runtime/formal/evidence closure on the admitted slice, but remain in `W051` until the surrounding seam or promotion packet is explicitly closed

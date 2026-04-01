# OxFunc Surface Admission And Labeling Policy

Status: `active`
Date: 2026-04-01

Supersession note:
1. For current row counts, backlog membership, and the consumer-facing admission report, this document is superseded by `docs/worksets/W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md`.
2. For backlog execution ownership, use `W051` plus `docs/function-lane/W58_HIDDEN_ORDINARY_BACKLOG_NORMALIZED.csv`; `docs/function-lane/W51_HIDDEN_NON_DEFERRED_BACKLOG_FIRST_PASS.csv` remains provenance for the original hidden snapshot-entry discovery.
3. The labeling vocabulary below remains active unless it conflicts with `W051`.
4. For the current OxFunc-owned shared-interface freeze candidate over the seam-relevant non-deferred surface, use `docs/function-lane/OXFML_OXFUNC_SHARED_INTERFACE_FREEZE_CANDIDATE_V1.md`.

## 1. Purpose
Define how downstream consumers, primarily `DNA OneCalc`, should interpret and label the three admission categories in the OxFunc function surface:
1. `function-phase-complete` rows,
2. `W050` deferred rows,
3. `W051` in-scope-not-complete rows.

This document also provides the honest current status of specific seam-heavy rows and defines labeling rules for help, completion, product UI, and scenario metadata.

## 2. Admission Categories

### 2.1 Function-Phase-Complete
Definition (from `CHARTER.md` Section 7.4 and `OPERATIONS.md` Section 3):
1. reference-baseline semantics are characterized with high confidence,
2. the function/evaluation seam is understood and documented,
3. the Rust implementation is thorough and tested,
4. the Lean/formal work required by the formalization strategy for the function's primary semantic substrate has been attended to and aligned,
5. no known function-semantic gap remains in current-phase scope.

Downstream reading:
1. the function's core semantics are implemented and tested against the current reference Excel baseline,
2. locale/version sweeps may still be pending as orthogonal validation phases,
3. XLL verification-seam limitations may still exist and are documented separately,
4. the function is admitted in the runtime dispatch and snapshot export surfaces.

Current consumer-facing report from `W051`:
1. `534` published rows total:
   - `511` functions,
   - `23` operators.
2. `374` rows are currently usable on a first-pass consumer read:
   - `374` supported,
   - `0` preview.
3. `17` rows are deferred through `W050`.
4. `143` hidden snapshot entries are non-deferred current-version backlog on the consumer-facing published-catalog reading.
5. the ordinary-backlog execution program now operates on `150` normalized function rows after `W060`.
6. the exact first-pass stale-row set still lives in `docs/function-lane/W44_DOCUMENTED_COMPLETE_SNAPSHOT_STALE_INVENTORY.csv` as refresh provenance, but those rows are no longer stale in the current published snapshot export.

### 2.2 W050 Deferred (Current-Version Excluded)
Definition (from `docs/worksets/W050_DEFERRED_CURRENT_VERSION_SURFACE.md`):
1. explicitly excluded from the current OxFunc completion target,
2. the exclusion is intentional and tracked, not an accidental omission.

Current members: `17` functions (see `W50_DEFERRED_CURRENT_VERSION_INVENTORY.csv`).

Categories of deferral:
1. external data provider dependency (`CUBE*`, `STOCKHISTORY`, `WEBSERVICE`, `DETECTLANGUAGE`, `COPILOT`),
2. provider-language seam (`TRANSLATE`),
3. XML/encoding surface with external dependency (`FILTERXML`, `ENCODEURL`),
4. pivot-topology dependency (`GETPIVOTDATA`),
5. stored-metadata surface (`PHONETIC`),
6. add-in-owned surface (`EUROCONVERT`).

Downstream reading:
1. these functions exist in the snapshot export catalog but must not be treated as supported or evaluable,
2. they should appear in downstream catalogs only with explicit deferred/unsupported labeling,
3. their snapshot export rows carry `catalog_only` metadata status for most members.

### 2.3 W051 In-Scope Not-Complete
Definition (from `docs/worksets/W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md`):
1. in scope for the current OxFunc version target,
2. not yet fully complete,
3. many have real OxFunc runtime/formal/evidence work on their admitted slice, but remain listed because the surrounding seam, promotion, or documentation packet is still open.

Current working members: `150` functions + `0` operators = `150` total normalized execution rows.
Current split:
1. `0` explicit preview-cluster rows,
2. `143` hidden non-deferred snapshot entries now centralized through `W51_HIDDEN_NON_DEFERRED_BACKLOG_CURRENT.csv`,
3. `150` normalized execution rows now centralized through `W51_NORMALIZED_ORDINARY_BACKLOG_CURRENT.csv`.

Important current narrowing:
1. there is no remaining explicit preview-cluster row in `W051`,
2. `OP_IMPLICIT_INTERSECTION` is now complete for declared current-phase OxFunc scope under `W014`,
3. `GROUPBY` and `PIVOTBY` are now promoted out of `W051` through `W055`,
4. the callable-helper family, `IMAGE`, and `CALL` / `REGISTER.ID` are now promoted out of `W051` after shared-freeze acknowledgment and packet-local closure.

Downstream reading:
1. the remaining `143` hidden snapshot entries are real non-deferred current-version backlog and should not be silently treated as supported just because they were omitted from the older narrow `W051` packet reading,
2. those `143` entries correspond to `150` machine-clean execution rows for closure planning,
3. those rows are ordinary backlog, not the current shared-interface acknowledgement scope for OxFunc ↔ OxFml freeze work,
4. `W051`, not this policy note, is the authoritative owner for current row membership and counts.

## 3. Seam-Heavy Row Honest Status

Completed current-phase promotions:
1. `OP_IMPLICIT_INTERSECTION` is now complete for declared current-phase OxFunc scope under `W014`; current-baseline native replay pins both explicit `@` storage behavior and `_xlfn.SINGLE(...)` normalization onto the same semantic surface, while OxFml keeps the legacy-single alias upstream rather than requiring a second OxFunc runtime path.
2. `GROUPBY` and `PIVOTBY` are now complete for declared current-phase OxFunc scope under `W055`; the supported reading now reflects a real grouped-aggregation runtime plus OxFml `W053` adapter floor rather than preview-only packet status.
3. `IMAGE` is now complete for declared current-phase OxFunc scope under `W023`; keep its non-ordinary `_webimage` rich-value carrier explicit.
4. `CALL` and `REGISTER.ID` are now complete for declared current-phase OxFunc scope under `W046`; wider admission-matrix widening is follow-on evidence, not a blocker.
5. the `W038` callable-helper family (`LET`, `LAMBDA`, `ISOMITTED`, `MAP`, `REDUCE`, `SCAN`, `BYROW`, `BYCOL`, `MAKEARRAY`) is now complete for declared current-phase scope after shared callable-freeze promotion.

## 4. Downstream Labeling Rules

### 4.1 Admission Category Labels
Downstream consumers should assign one of the following admission-category labels to each row:

| Admission Category | Snapshot Export Reading | Label for Downstream |
|-------------------|----------------------|---------------------|
| function-phase-complete | Row present, not in W050 or W051, `metadata_status` is `function_meta_extracted` or `function_meta_curated` | `supported` |
| W051 with real runtime | Row present, listed in W051, W051 notes confirm real kernel | `preview` |
| W051 without real runtime | Row present, listed in W051, W051 notes indicate open boundary | `experimental` |
| W050 deferred | Row present, listed in W050 | `deferred` |
| W051 hidden backlog | Row present, tracked by W051 hidden-backlog appendix, still `catalog_only` in the export | `catalog_only` |

Current warning:
1. the old simple reading "`catalog_only` and absent from W050/W051 means standalone catalog-only backlog" is superseded by `W051`;
2. snapshot status alone is still not authoritative for backlog truth; `W051` remains the owner for current row membership.

### 4.2 Labeling By Surface

#### Help Surface
| Admission Category | Help Behavior |
|-------------------|---------------|
| `supported` | Show full available help from snapshot fields. No qualification needed for core semantics. May note XLL seam limits if material. |
| `preview` | Show available help with visible `[Preview]` qualifier. Note that the function has real runtime support on the admitted slice but current-surface closure is not yet complete. |
| `experimental` | Show available help with visible `[Experimental]` qualifier. Note specific gap kind from Section 3. |
| `deferred` | Show function name and category only. Display `[Not available in current version]` with deferral reason from W050. |
| `catalog_only` | Show function name only. Display `[Catalog entry only - not yet characterized]`. |

#### Completion Surface
| Admission Category | Completion Behavior |
|-------------------|---------------------|
| `supported` | Include in completion list. No special decoration. |
| `preview` | Include in completion list with `[Preview]` badge. |
| `experimental` | Include in completion list with `[Experimental]` badge. |
| `deferred` | Include in completion list with `[Deferred]` badge and lower sort priority. |
| `catalog_only` | Include in completion list with `[Catalog Only]` badge and lowest sort priority. |

#### Product UI Surface
| Admission Category | UI Behavior |
|-------------------|-------------|
| `supported` | Normal display. |
| `preview` | Visible preview indicator in function browser and result display. |
| `experimental` | Visible experimental indicator. Link to `interface_contract_ref` if available. |
| `deferred` | Grayed or separated in function browser. Evaluation should produce a clear host-level message rather than a silent failure. |
| `catalog_only` | Same as deferred. |

#### Scenario Metadata Surface
| Admission Category | Scenario Metadata Behavior |
|-------------------|-----------------------------|
| `supported` | Record `admission_category: supported` in scenario run metadata. |
| `preview` | Record `admission_category: preview` and `preview_reason` in scenario run metadata. |
| `experimental` | Record `admission_category: experimental` and `experimental_gap_kind` in scenario run metadata. |
| `deferred` | Record `admission_category: deferred` and `deferral_reason` in scenario run metadata. Flag scenario as using a deferred function. |
| `catalog_only` | Record `admission_category: catalog_only`. Flag scenario as using an uncharacterized function. |

### 4.3 Labeling Implementation Rule
1. OneCalc must derive the admission category by joining the snapshot export against the current W050 and W051 inventories, not by reading the snapshot export alone.
2. The old narrow join against only `W50_DEFERRED_CURRENT_VERSION_INVENTORY.csv` and `W51_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_INVENTORY.csv` is no longer sufficient by itself; current non-deferred outstanding-row truth also requires `docs/worksets/W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md`, `docs/function-lane/W51_HIDDEN_NON_DEFERRED_BACKLOG_CURRENT.csv`, and `docs/function-lane/W51_NORMALIZED_ORDINARY_BACKLOG_CURRENT.csv`. The older first-pass `W51_HIDDEN_NON_DEFERRED_BACKLOG_FIRST_PASS.csv` remains provenance only.
3. Any row present in the snapshot export and explicitly classified by `W051` as part of the preview cluster should be labeled `preview` or `experimental` based on the W051 notes.
4. Any row present in `W050` should be labeled `deferred`.
5. Any row present in the W051 hidden-backlog appendix should be labeled `catalog_only` until promoted.
6. The first-pass `W44` stale inventory remains useful as provenance for the refresh set, but those rows now belong in the supported bucket in the current export as well.
7. This policy should be treated as subordinate to `W051` for current row membership and backlog truth.

## 5. What This Policy Does Not Cover
1. OxFml-owned formula admission and diagnostic semantics,
2. OxReplay-owned replay surface labeling,
3. host-level UI interaction design,
4. locale/version sweep completion status (orthogonal validation phase),
5. XLL verification-seam limitation details (see `XLL_VERIFICATION_SEAM_LIMITATIONS.md`).

## 6. Authoritative Upstream References
1. `CHARTER.md` Section 7.4 - completeness reporting semantics
2. `OPERATIONS.md` Section 3 - operating principles, `function-phase-complete` definition
3. `OPERATIONS.md` Section 11 - report-back completeness contract
4. `docs/worksets/W050_DEFERRED_CURRENT_VERSION_SURFACE.md`
5. `docs/worksets/W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md`
6. `docs/function-lane/W50_DEFERRED_CURRENT_VERSION_INVENTORY.csv`
7. `docs/function-lane/W51_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_INVENTORY.csv`
8. `docs/function-lane/W51_HIDDEN_NON_DEFERRED_BACKLOG_CURRENT.csv`
9. `docs/function-lane/W51_NORMALIZED_ORDINARY_BACKLOG_CURRENT.csv`
10. `docs/function-lane/W51_HIDDEN_NON_DEFERRED_BACKLOG_FIRST_PASS.csv`
11. `docs/function-lane/OXFUNC_DOWNSTREAM_METADATA_AND_HELP_CONTRACT.md`
12. `docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1_README.md`

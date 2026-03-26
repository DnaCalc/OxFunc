# IN_PROGRESS_FEATURE_WORKLIST.md — OxFunc

Canonical repo-level register of feature areas that are in-progress under workset completion doctrine.

Status: active.
Last updated: 2026-03-26.

## Status Vocabulary

- `in-progress`: partial implementation exists, parity/completeness not yet achieved.
- `blocked`: in-progress with active blocker (see CURRENT_BLOCKERS.md).
- `planned`: explicitly accepted into scope, no shipped work yet.

## Active Feature Register

### IP-01: Function Catalog Expansion

- **Status**: in-progress
- **Current floor**: 40+ functions at `function-phase-complete` across W001-W015 and follow-on closure packets, with the standalone `SUMIF` gap now closed through `W052` and the low-order `ASINH` / `PV` / `FV` / `PMT` publication residuals now closed through `W053`.
- **Remaining gaps**: current-version backlog tracking is now centralized in `W050` and `W051`. `W050` owns deferred-current-version rows (`W041` family plus `EUROCONVERT`), while `W051` owns in-scope not-complete rows (`W014`, residual `W023`, `W038`, `W046`, trim-reference operators, and latent gaps such as `TRIMRANGE`, `GROUPBY`, `PIVOTBY`, `ROWS`, `COLUMNS`, `RANDBETWEEN`, `VALUETOTEXT`, and `RANDARRAY`).
- **Why still open**: `W016` is closed, `W022` closes the criteria-family residual, `W024` is reconciled, `W025` is resolved as a classification packet, `W026` is resolved as a characterization-and-extraction packet, `W027` is packet-complete for its declared scope, `W028` corrected the local canonical catalog to `511` names, `W029` is complete as a benchmark/classification packet, `W030` and `W031` are now closed as seam-definition packets, `W032` repaired the reopened finance packet, `W033` closes the newly promoted information-predicate and forecast-compatibility packet, `W034` / `W035` / `W036` now close the locale/profile/provider residual seam packets, `W037` closes the remaining large-root `XIRR` publication lane, `W040` closes the reference-metadata family, and `W045` closes the current non-`@` operator universe.
- **Canonical owner**: aggregate current-version tracking now lives in `W050` / `W051`; family provenance and execution ownership remain with the narrower packets (`W014`, `W023`, `W038`, `W041`, `W045`, `W046`, `W025`).

### IP-02: Locale and Version Sweeps

- **Status**: planned
- **Current floor**: dual-axis version tracking infrastructure in place; no systematic sweep execution yet.
- **Remaining gaps**: locale-sensitive coercion and formatting behavior across Excel app versions/channels and workbook Compatibility Versions.
- **Why still open**: orthogonal validation phase; functions declared `function-phase-complete` under reference baseline only.
- **Canonical owner**: workset TBD (orthogonal validation phase).

### IP-03: UDF Surface Contract (VBA/JS/Automation)

- **Status**: planned
- **Current floor**: XLL surface contract partially exercised through W009/W011; VBA/JS/Automation boundaries not yet characterized.
- **Remaining gaps**: VBA UDF call semantics, JavaScript API UDF boundary, Automation-facing function invocation semantics.
- **Why still open**: chartered in CHARTER.md Section 4 item 5; not yet targeted by a workset.
- **Canonical owner**: workset TBD.

### IP-04: Formalization Deepening

- **Status**: in-progress
- **Current floor**: Lean substrate-level executable models and bindings for admitted slices per formalization strategy.
- **Remaining gaps**: deeper proof obligations beyond substrate alignment; property and metamorphic proof coverage for complex function families.
- **Why still open**: formalization strategy permits substrate-level work for current phase; deeper obligations are tracked but deferred.
- **Canonical owner**: ongoing across worksets.

### IP-05: XLL Seam Hardening

- **Status**: in-progress
- **Current floor**: XLL add-in bridge exercised through W009/W011; registration flags and basic invocation evidence collected.
- **Remaining gaps**: comprehensive seam limitation catalog; adversarial seam tests; seam-level vs function-semantic status separation in all verification records.
- **Why still open**: seam limitations are documented but not yet systematically hardened across all function families.
- **Canonical owner**: W009/W011 continuation + future worksets.

### IP-06: OxFml/FEC/F3E Interface Refinement

- **Status**: in-progress
- **Current floor**: interface constraints documented in `docs/upstream/NOTES_FOR_OXFML.md`; provisional sketches for provenance carriers and boundary contracts.
- **Remaining gaps**: first shared typed context/query bundle, first shared return-surface split, runtime provider/snapshot consumer model, finalized upstream provenance vocabulary, reference-identity carrier, prepared-call contract, evaluation-mode contract.
- **Why still open**: the latest OxFml note now accepts the first-freeze working rule and the current `W044` callable-row split for one round, but the next seam locks still need explicit packet owners and shared artifacts. Callable field-lock follow-up remains deferred to `W042`; the next agreed seam-hardening owners are `W047`, `W048`, and `W049`.
- **Canonical owner**: cross-repo; tracked via upstream observation ledger, with OxFunc-local seam hardening in `W042`, `W047`, `W048`, and `W049`.
- **Immediate follow-on after seam freeze**: continue current-scope completion through `W014` (`@`), `W046` (`CALL` / `REGISTER.ID`), and `W023` residual `IMAGE` / `HYPERLINK` publication work rather than treating them as deferred out-of-scope.

### IP-07: Implicit Intersection and Scalarization Semantics

- **Status**: in-progress
- **Current floor**: provisional canonicalization row `FDEF-018`; OxFml formula-language rules preserve `@` parse acceptance; W14 now records a dedicated OxFunc investigation slice and upstream handoff packet.
- **Remaining gaps**: precise caller-context scalarization semantics, spill-anchor/reference-result provenance, compatibility-version mapping for `@` vs `SINGLE`/`_xlfn.SINGLE`, runtime implementation, Lean executable model, and deterministic replay artifacts.
- **Why still open**: current evidence proves syntax and migration pressure, but not a fully implemented scalarization seam across OxFunc, OxFml, and FEC/F3E.
- **Canonical owner**: `W014`.
- **Scope note**: this remains in current scope; it is difficult, not deferred out-of-scope.

### IP-08: Replay Appliance Packet Adapter Rollout

- **Status**: in-progress
- **Current floor**: `W020` and `W021` now have a first live local proving artifact under `.tmp/replay-bundles/oxfunc-w15-v1/`, with `W21_EXECUTION_RECORD.md` and the emitted bundle validation/replay/diff/explain sidecars evidencing local `cap.C0` through `cap.C3` for the `W15` worked packet.
- **Remaining gaps**: live `DNA ReCalc` import against an OxFunc packet bundle, replay-valid reduced packet or row witnesses, a second packet proving the adapter is not `W15`-specific, and any future pack-grade promotion evidence.
- **Why still open**: the local adapter surface is now real and exercised, but `cap.C4` / `cap.C5` remain explicitly non-claimed and the cross-lane replay-host path is still unproven.
- **Canonical owner**: `W018` through `W021`.

### IP-09: Function Name Localization Library

- **Status**: in-progress
- **Current floor**: `W28` now has a reproducible official-support harvest, `40` published `hreflang` alternates, a `20,360`-row localized-name seed, a `509`-name current English harvest, and a reconciliation artifact against the older `500`-row catalog freeze.
- **Remaining gaps**: version-marker extraction, normative variation matching against `MS-OE376`, normalization of localized function names against canonical OxFunc ids, and eventual library-context export for OxFml parse/bind use.
- **Why still open**: `W28` completed its declared discovery-and-seed scope, but the actual localization library and normative reconciliation work remain follow-on packets.
- **Canonical owner**: `W028`.

### IP-10: Library-Context Snapshot Export

- **Status**: in-progress
- **Current floor**: OxFunc now exposes a first explicit snapshot artifact in `docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv` with identity/version semantics, first-pass function and operator rows, metadata profiles, and reading guidance in `OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1_README.md`.
- **Remaining gaps**: refinement of field coverage, richer per-entry semantic/gating refs, broader operator coverage beyond the currently exported universe, and a pinned runtime consumer/model beyond the CSV interchange artifact.
- **Why still open**: the first-pass snapshot is now real and usable, and OxFml has accepted the current first-freeze working rule, but the next step is a concrete runtime provider/snapshot consumer model rather than more note-only agreement.
- **Canonical owner**: `W044` for the export artifact, with follow-on consumer/model work in `W049`.

# IN_PROGRESS_FEATURE_WORKLIST.md — OxFunc

Canonical repo-level register of feature areas that are in-progress under workset completion doctrine.

Status: active.
Last updated: 2026-04-01.

Supersession note:
- For current catalog-truth counts and non-deferred backlog membership, `IP-01` is superseded by `docs/worksets/W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md`.
- For the current shared-interface freeze candidate over the seam-relevant non-deferred surface, use `docs/function-lane/OXFML_OXFUNC_SHARED_INTERFACE_FREEZE_CANDIDATE_V1.md`.

## Status Vocabulary

- `in-progress`: partial implementation exists, parity/completeness not yet achieved.
- `blocked`: in-progress with active blocker (see CURRENT_BLOCKERS.md).
- `planned`: explicitly accepted into scope, no shipped work yet.

## Active Feature Register

### IP-01: Function Catalog Expansion

- **Status**: in-progress
- **Current consumer-facing report**: `W051` now records `534` published rows (`511` functions, `23` operators), with `374` currently usable rows (`374` supported, `0` preview), `17` deferred rows in `W050`, and `143` hidden non-deferred backlog snapshot entries. After `W060`, the same ordinary backlog now stands at `150` normalized execution rows across `W061` through `W068`.
- **Remaining gaps**: current-version backlog tracking is now centralized in `W050` and `W051`. `W050` owns the `17` deferred-current-version rows (`W041` family plus `TRANSLATE` and `EUROCONVERT`), while `W051` now owns the remaining non-deferred outstanding rows: the centralized ordinary backlog, now reduced by `W060` to `150` execution rows across `W061` through `W068`.
- **Current narrowing**: `W014`, `W023`, `W038`, `W046`, and `W055` are now complete for declared current-phase scope and no longer contribute rows to `W051`.
- **Why still open**: the repo now has one main residual semantic backlog class (`W051`), plus broader `W044` field-normalization work. The exact `114` documented-complete snapshot-stale rows have now been refreshed into the published catalog artifact, `W058` removed grouped-row ambiguity from the ordinary execution backlog, and `W059` / `W060` have now closed the first `42` rows of that program.
- **Canonical owner**: aggregate current-version tracking now lives in `W050` / `W051`; family provenance and execution ownership remain with the narrower packets (`W014`, `W023`, `W038`, `W041`, `W045`, `W046`, `W025`), and snapshot/export alignment remains coupled to `W044`.

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
- **Remaining gaps**: deeper proof obligations beyond substrate alignment; property and metamorphic proof coverage for complex function families; and the explicit missing-function-id formalization backlog now pinned in `W054`.
- **Why still open**: formalization strategy permits substrate-level work for current phase; deeper obligations are tracked but deferred, and `W054` now owns the first explicit Rust-vs-Lean function-id gap reconciliation pass.
- **Canonical owner**: `W054` for explicit missing-function-id reconciliation, with deeper proof work ongoing across function packets.

### IP-05: XLL Seam Hardening

- **Status**: in-progress
- **Current floor**: XLL add-in bridge exercised through W009/W011; registration flags and basic invocation evidence collected.
- **Remaining gaps**: comprehensive seam limitation catalog; adversarial seam tests; seam-level vs function-semantic status separation in all verification records.
- **Why still open**: seam limitations are documented but not yet systematically hardened across all function families.
- **Canonical owner**: W009/W011 continuation + future worksets.

### IP-06: OxFml/FEC/F3E Interface Refinement

- **Status**: in-progress
- **Current floor**: the narrowed seam families now have a consolidated OxFunc candidate, a completed OxFunc-side comparison delta, and an OxFml-acknowledged `HO-FN-004` promotion handoff.
- **Current freeze candidate**: the current OxFunc-owned consolidated candidate now lives in `docs/function-lane/OXFML_OXFUNC_SHARED_INTERFACE_FREEZE_CANDIDATE_V1.md`.
- **Current outbound promotion packet**: `docs/handoffs/HANDOFF_SHARED_INTERFACE_FREEZE_PROMOTION_TO_OXFML_V1.md`.
- **Remaining gaps**: explicit promotion and propagation of the acknowledged shared freeze text across the local owner packets and any downstream coordinator-facing consumers, plus any later concrete mismatch-driven corrections.
- **Why still open**: OxFml now reads the handoff as acceptable shared freeze wording for the narrowed seam families, but `W046` and the row-level interesting backlog still retain packet-local promotion and downstream propagation work after the cross-repo freeze acknowledgment.
- **Canonical owner**: cross-repo; tracked via the upstream observation ledger and `HO-FN-004`, with the current OxFunc-side freeze candidate assembled from `W042`, `W046`, `W047`, `W048`, and `W049`.
- **Immediate follow-on after freeze promotion**: continue current-scope completion through the hidden ordinary backlog execution program; `W059` and `W060` are now complete, leaving `W061` through `W068`, while `W023`, `W038`, `W046`, and `W055` are now promoted out for declared current-phase scope.

### IP-07: Implicit Intersection and Scalarization Semantics

- **Status**: in-progress
- **Current floor**: canonicalization row `FDEF-018`; native Excel replay for seeded `@` lanes plus current-baseline `_xlfn.SINGLE(...)` normalization; Rust runtime in `op_implicit_intersection.rs`; Lean binding in `ImplicitIntersection.lean`; and OxFml adapter/evaluator/semantic-plan evidence for explicit `@` and legacy-single compatibility semantics.
- **Remaining gaps**: broader pre-dynamic-array serialization/roundtrip sweeps and structured-reference/table-context interaction outside the admitted current-phase slice.
- **Why still open**: the OxFunc-side current-baseline target is complete; only orthogonal future validation/interop lanes remain.
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
- **Current floor**: OxFunc now exposes a first explicit snapshot artifact in `docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv` with identity/version semantics, first-pass function and operator rows, metadata profiles, and reading guidance in `OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1_README.md`. Downstream metadata and help contract is now documented in `OXFUNC_DOWNSTREAM_METADATA_AND_HELP_CONTRACT.md`, and surface admission and labeling policy for downstream consumers is documented in `OXFUNC_SURFACE_ADMISSION_AND_LABELING_POLICY.md`.
- **Remaining gaps**: refinement of field coverage, richer per-entry semantic/gating refs, broader operator coverage beyond the currently exported universe, a pinned runtime consumer/model beyond the CSV interchange artifact, and structured help/signature payload population (help prose, argument names/descriptions, and signature display strings are not yet available from OxFunc).
- **Why still open**: the first-pass snapshot is now real and usable, OxFml has accepted the current first-freeze working rule, and downstream contract/labeling docs are now explicit, but the next step is a concrete runtime provider/snapshot consumer model and structured help payload population rather than more note-only agreement.
- **Canonical owner**: `W044` for the export artifact, with follow-on consumer/model work in `W049`.

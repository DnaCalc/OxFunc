# IN_PROGRESS_FEATURE_WORKLIST.md — OxFunc

Canonical repo-level register of feature areas that are in-progress under workset completion doctrine.

Status: active.
Last updated: 2026-04-01.

Supersession note:
- For current catalog-truth counts and non-deferred backlog membership, `IP-01` is superseded by `docs/worksets/W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md`.
- For the current shared-interface freeze candidate over the seam-relevant non-deferred surface, use `docs/function-lane/OXFML_OXFUNC_SHARED_INTERFACE_FREEZE_CANDIDATE_V1.md`.

## Status Vocabulary

- `in-progress`: partial implementation exists, parity/completeness not yet achieved.
- `blocked`: in-progress with an active blocker recorded in `.beads/`; `CURRENT_BLOCKERS.md` is exceptional prose-only if used at all.
- `planned`: explicitly accepted into scope, no shipped work yet.

## Active Feature Register

### IP-01: Function Catalog Expansion

- **Status**: in-progress
- **Current consumer-facing report**: `W051` now records `534` published rows (`511` functions, `23` operators), with `517` currently usable rows (`517` supported, `0` preview), `17` deferred rows in `W050`, and `0` hidden non-deferred backlog snapshot entries. After `W068`, the ordinary non-deferred backlog is fully drained (`0` normalized execution rows).
- **Remaining gaps**: current-version backlog tracking remains centralized in `W050` and `W051`. `W050` owns the `17` deferred-current-version rows (`W041` family plus `TRANSLATE` and `EUROCONVERT`), while `W051` is now complete for declared non-deferred current-version scope and therefore empty.
- **Current narrowing**: `W014`, `W023`, `W038`, `W046`, and `W055` are now complete for declared current-phase scope and no longer contribute rows to `W051`.
- **Why still open**: current-version closure still intentionally excludes the `17` deferred `W050` rows, and broader `W044` field-normalization work remains beyond the current first-pass export. The exact `114` documented-complete snapshot-stale rows have now been refreshed into the published catalog artifact, `W058` removed grouped-row ambiguity from the ordinary execution backlog, and `W059` / `W060` / `W061` / `W062` / `W063` / `W064` / `W065` / `W066` / `W067` / `W068` have now closed the full `192`-row ordinary execution program.
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
- **Current parking reading**: `W054` is now complete for the explicit Rust-vs-Lean function-id reconciliation of the non-deferred parked surface; no active missing-id or modern-alias mismatch remains for those rows.
- **Remaining gaps**: deeper proof obligations beyond substrate alignment; property and metamorphic proof coverage for complex function families; and any future formalization follow-on for the intentionally deferred `W050` set.
- **Why still open**: current-phase doctrine is satisfied by substrate-level executable models, bindings, and alignment for the parked non-deferred surface, but deeper proof work remains a separate long-run lane.
- **Canonical owner**: `W054` as the closed provenance packet for current non-deferred id reconciliation, with future proof-deepening work distributed across later function packets.

### IP-05: XLL Seam Hardening

- **Status**: in-progress
- **Current floor**: XLL add-in bridge exercised through W009/W011; registration flags and basic invocation evidence collected.
- **Remaining gaps**: comprehensive seam limitation catalog; adversarial seam tests; seam-level vs function-semantic status separation in all verification records.
- **Why still open**: seam limitations are documented but not yet systematically hardened across all function families.
- **Canonical owner**: W009/W011 continuation + future worksets.

### IP-06: OxFml/FEC/F3E Interface Refinement

- **Status**: planned
- **Current floor**: the narrowed seam families now have an OxFml-acknowledged shared freeze reading, propagated through the local owner packets and downstream summary docs for the parked current-phase surface.
- **Current freeze candidate**: the current OxFunc-owned consolidated candidate now lives in `docs/function-lane/OXFML_OXFUNC_SHARED_INTERFACE_FREEZE_CANDIDATE_V1.md`.
- **Current outbound promotion packet**: `docs/handoffs/HANDOFF_SHARED_INTERFACE_FREEZE_PROMOTION_TO_OXFML_V1.md`.
- **Remaining gaps**: only future mismatch-driven corrections or later post-parking seam redesign work beyond the current shared freeze wording.
- **Why still open**: there is no active current-phase blocker left in this lane, but the cross-repo seam remains an evolvable program surface rather than a forever-frozen ABI.
- **Canonical owner**: cross-repo; tracked via the upstream observation ledger and `HO-FN-004`, with the current OxFunc-side freeze candidate assembled from `W042`, `W046`, `W047`, `W048`, and `W049`.
- **Current parking reading**: the hidden ordinary backlog execution program is fully drained through `W068`; the remaining current-version exclusions are the intentional `W050` deferred set.

### IP-07: Implicit Intersection and Scalarization Semantics

- **Status**: planned
- **Current floor**: canonicalization row `FDEF-018`; native Excel replay for seeded `@` lanes plus current-baseline `_xlfn.SINGLE(...)` normalization; Rust runtime in `op_implicit_intersection.rs`; Lean binding in `ImplicitIntersection.lean`; and OxFml adapter/evaluator/semantic-plan evidence for explicit `@` and legacy-single compatibility semantics.
- **Remaining gaps**: broader pre-dynamic-array serialization/roundtrip sweeps and structured-reference/table-context interaction outside the admitted current-phase slice.
- **Why still open**: the OxFunc-side current-baseline target is complete; only orthogonal future validation and interop sweeps remain.
- **Canonical owner**: `W014`.
- **Scope note**: no active current-phase blocker remains; any future work here is validation/deepening rather than baseline closure.

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
- **Remaining gaps**: richer per-entry semantic/gating refs, broader operator coverage beyond the currently exported universe, runtime witness attachment beyond the CSV interchange artifact, and structured help/signature/semantic-witness payload population.
- **Why still open**: the first-pass snapshot is now real and usable, but the highest-leverage next step is no longer just more profile columns. It is the `W069` `Semantic Witness Snapshot V2` direction: structured help, signature, evidence, and formal-reference payloads attached to the library-context model.
- **Canonical owner**: `W044` for the `V1` export artifact, `W049` for the runtime provider/snapshot model, and `W069` for the `V2` semantic witness plan.

### IP-11: Execution Doctrine Migration And Active Tree Reduction

- **Status**: in-progress
- **Current floor**: OxFunc has a parked non-deferred baseline, a clean active branch tip, a stable recovery tag anchor in `OxFunc_V1`, and a mature but document-heavy workset-plus-evidence execution style.
- **Current phase**: `W070` Phase A, Phase B, Phase C, and Phase D are complete; Phase E active-tree reduction is next.
- **Remaining gaps**: too much historical execution/provenance material still remains in the active tree, transitional bridge surfaces still need to narrow or retire, and the first real post-migration workset execution under beads still remains even though the live bead workspace and remaining `W070` graph are now fully encoded in `.beads/`.
- **Why still open**: doctrine and bead bootstrap are now in place, but the active-tree reduction waves and the first real post-migration execution still remain.
- **Canonical owner**: `W070`.

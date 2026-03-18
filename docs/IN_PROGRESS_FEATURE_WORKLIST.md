# IN_PROGRESS_FEATURE_WORKLIST.md — OxFunc

Canonical repo-level register of feature areas that are in-progress under workset completion doctrine.

Status: active.
Last updated: 2026-03-18.

## Status Vocabulary

- `in-progress`: partial implementation exists, parity/completeness not yet achieved.
- `blocked`: in-progress with active blocker (see CURRENT_BLOCKERS.md).
- `planned`: explicitly accepted into scope, no shipped work yet.

## Active Feature Register

### IP-01: Function Catalog Expansion

- **Status**: in-progress
- **Current floor**: 40 functions at `function-phase-complete` across W001-W015, with the criteria-family shape batch (`COUNTIF`, `COUNTIFS`, `SUMIFS`, `AVERAGEIF`, `AVERAGEIFS`, `MAXIFS`, `MINIFS`) now reconciled out of `W17` through `W022`.
- **Remaining gaps**: the host/metadata/database successor packet `W023`, extracted successor packets `W025` / `W026` / `W027`, the remaining interesting-function universe, and operator-as-function (`OP_*`) semantics for undeclared operators.
- **Why still open**: `W016` is closed, `W022` closes the criteria-family residual, and `W024` is now reconciled; the remaining low-interest work now sits in the successor extraction packets rather than the ordinary mega-batch itself.
- **Canonical owner**: ongoing across worksets, currently `W023` / `W025` / `W026` / `W027` for the residual low-interest universe and `W014` for deferred `@`.

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
- **Remaining gaps**: finalized upstream provenance vocabulary; reference-identity carrier; prepared-call contract; evaluation-mode contract.
- **Why still open**: upstream interface is actively evolving; OxFunc observations inform but do not control OxFml design.
- **Canonical owner**: cross-repo; tracked via upstream observation ledger.

### IP-07: Implicit Intersection and Scalarization Semantics

- **Status**: in-progress
- **Current floor**: provisional canonicalization row `FDEF-018`; OxFml formula-language rules preserve `@` parse acceptance; W14 now records a dedicated OxFunc investigation slice and upstream handoff packet.
- **Remaining gaps**: precise caller-context scalarization semantics, spill-anchor/reference-result provenance, compatibility-version mapping for `@` vs `SINGLE`/`_xlfn.SINGLE`, runtime implementation, Lean executable model, and deterministic replay artifacts.
- **Why still open**: current evidence proves syntax and migration pressure, but not a fully implemented scalarization seam across OxFunc, OxFml, and FEC/F3E.
- **Canonical owner**: `W014`.

### IP-08: Replay Appliance Packet Adapter Rollout

- **Status**: in-progress
- **Current floor**: `W020` and `W021` now have a first live local proving artifact under `.tmp/replay-bundles/oxfunc-w15-v1/`, with `W21_EXECUTION_RECORD.md` and the emitted bundle validation/replay/diff/explain sidecars evidencing local `cap.C0` through `cap.C3` for the `W15` worked packet.
- **Remaining gaps**: live `DNA ReCalc` import against an OxFunc packet bundle, replay-valid reduced packet or row witnesses, a second packet proving the adapter is not `W15`-specific, and any future pack-grade promotion evidence.
- **Why still open**: the local adapter surface is now real and exercised, but `cap.C4` / `cap.C5` remain explicitly non-claimed and the cross-lane replay-host path is still unproven.
- **Canonical owner**: `W018` through `W021`.

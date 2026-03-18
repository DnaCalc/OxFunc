# OxFunc Replay Appliance Packet Adapter V1

Status: `provisional`
Owner lane: `OxFunc`
Adapter id: `oxfunc.replay.packet_adapter`
Adapter version: `1.0.0-draft`

## 1. Scope and Non-Goals
This note defines the OxFunc-local packet adapter contract for Replay appliance rollout.

In scope:
1. packet-first and row-first projection of existing OxFunc empirical artifacts into Replay bundle form,
2. preservation of OxFunc semantic authority, evidence ids, correlation links, boundary invariants, and seam limitations,
3. conservative capability claims for ingest, replay, diff, and explain over current packet artifacts,
4. packet-first witness-distillation scaffolding with explicit non-claim language.

Non-goals:
1. inventing a fake internal event stream for OxFunc,
2. replacing OxFunc semantic contracts with Replay bundle prose,
3. claiming reduced-witness replay validity without local proving artifacts,
4. claiming pack-grade promotion support in this pass,
5. erasing XLL or host-limitation distinctions by flattening them into semantic mismatches.

## 2. Authority Split and Conflict Handling
Authority split:
1. OxFunc remains authoritative for function and operator semantic meaning, evidence meaning, boundary invariants, correlation-ledger meaning, and XLL limitation classification.
2. The Foundation replay handoff from `2026-03-15` is authoritative for cross-lane replay governance, shared capability vocabulary, shared registry families, witness lifecycle vocabulary, and Replay bundle host expectations.
3. `DNA ReCalc` is a Logistics-layer replay host surface, not a new OxFunc semantic authority.

Explicit adaptation rule:
1. Foundation architecture defines a shared `ReplayEvent` object and generalized event families.
2. OxFunc does not currently produce an honest fine-grained semantic event stream.
3. Therefore the OxFunc adapter treats packet rows, analysis summaries, evidence bindings, invariant declarations, and limitation records as the primary witnesses.
4. Any normalized event family emitted for OxFunc is a derived projection view over those packet witnesses, not a source-semantic truth source.

Conflict note:
1. If generic Replay wording suggests event-stream-first treatment, OxFunc adapts that wording to packet-first and row-first semantics.
2. This is an adaptation of rollout wording, not a redefinition of OxFunc function semantics.

## 3. Packet-First and Row-First Replay Model
Primary OxFunc replay units:
1. packet/workset replay run,
2. manifest row cluster where one workset uses several related manifests,
3. individual manifest row,
4. execution-record summary,
5. evidence-binding record,
6. invariant declaration,
7. limitation marker,
8. source-native sidecar artifact partition.

Source-native inputs remain first-class:
1. scenario manifest CSVs,
2. output result CSVs,
3. execution records,
4. function-slice contract notes,
5. evidence-id registry rows,
6. correlation-ledger rows,
7. seam-limitation notes.

Packet replay rule:
1. replay validity for OxFunc means the packet can be re-run or re-import so that packet rows, run labels, compatibility descriptors, locale/environment metadata, and required sidecar references remain auditable.
2. replay does not require a stepwise simulation trace if row results and packet summaries are the honest witness.

## 4. Preserved Metadata
Every OxFunc replay bundle projection must preserve or reference:
1. `workset_id`,
2. `packet_id` or source manifest id,
3. `scenario_id` or row id,
4. `function_id` or slice id where applicable,
5. `evidence_id` refs,
6. correlation-ledger refs,
7. run label,
8. compatibility descriptor,
9. locale profile,
10. environment metadata including Excel build/channel where relevant,
11. source manifest path,
12. output artifact refs,
13. execution-record refs,
14. invariant refs,
15. limitation refs,
16. verification-surface classification,
17. semantic-target status.

Compatibility and environment rule:
1. locale, Excel build/channel, and workbook-compatibility descriptors are not optional decoration.
2. They are part of the replay identity for packet evidence and must remain visible in bundle-normalized views.

## 5. Normalized Views and Event Families Without Packet Distortion
Required normalized views:
1. `manifest_row_result_view`
2. `run_summary_view`
3. `analysis_summary_view`
4. `evidence_binding_view`
5. `correlation_binding_view`
6. `invariant_view`
7. `limitation_view`
8. `source_inventory_view`

Allowed derived event families:
1. `packet.started`
2. `packet.completed`
3. `row.observed`
4. `row.mismatched`
5. `analysis.completed`
6. `evidence.bound`
7. `invariant.bound`
8. `limitation.noted`

Projection rule:
1. these event families are normalized convenience views only,
2. they must be reproducible from packet rows or packet summaries,
3. they must not imply hidden internal evaluator steps that OxFunc did not capture,
4. if a source packet does not justify an event, the adapter must emit no event rather than fabricate one.

## 6. Boundary-Invariant Incorporation
Boundary invariants remain OxFunc-owned.

Replay bundle incorporation must preserve:
1. invariant id,
2. statement,
3. covered boundaries,
4. scenario ids,
5. expected observation,
6. status,
7. supporting evidence ids,
8. related limitation refs where applicable.

Required boundary families remain:
1. formula evaluation,
2. interop ingress,
3. reference reuse,
4. persistence,
5. interchange,
6. optional UDF/XLL.

Invariant failure rule:
1. invariant failures should map to shared predicate vocabulary such as `pred.invariant.failed`,
2. but the invariant statement and its semantic meaning stay OxFunc-local and must be cited from OxFunc docs or packet records.

## 7. Adapter Capability Target and Known Limits
Highest capability honestly claimed in this pass:
1. `cap.C0.ingest_valid`
2. `cap.C1.replay_valid`
3. `cap.C2.diff_valid`
4. `cap.C3.explain_valid`

Current non-claims:
1. `cap.C4.distill_valid` is scaffolded only in packet-first form and is not claimed complete.
2. `cap.C5.pack_valid` is not claimed.

Current proving path:
1. existing manifest-driven packet artifacts already define stable source inputs,
2. existing execution records and evidence ids already define row/result and summary surfaces,
3. current W15 packet artifacts give a concrete packet replay import example with run labels, compatibility descriptors, and host-limitation distinctions,
4. `tools/replay-adapter/run-w15-replay-adapter-baseline.ps1` now emits a live local `W15` replay bundle under `.tmp/replay-bundles/oxfunc-w15-v1/`,
5. `docs/function-lane/W21_EXECUTION_RECORD.md` records the first local proving artifact for `cap.C0` through `cap.C3`.

Known limits:
1. no fake internal event stream will be emitted for OxFunc,
2. XLL verification seam limits must remain classified as seam limits unless OxFunc explicitly promotes them to semantic failures,
3. reduced packet or row witnesses are not yet proven replay-valid,
4. no pack-grade export or witness-promotion claim is made,
5. schema ids for OxFunc source packets are still local-only adapter ids in this pass.

## 8. Registry Version Pins
Foundation-shared registries pinned in this pass:
1. `capability_level` -> snapshot `foundation-handoff-20260315-pass-01`
2. `predicate_kind` -> snapshot `foundation-handoff-20260315-pass-01`
3. `mismatch_kind` -> snapshot `foundation-handoff-20260315-pass-01`
4. `witness_lifecycle_state` -> snapshot `foundation-handoff-20260315-pass-01`
5. `reduction_status` -> snapshot `foundation-handoff-20260315-pass-01`

Local-only OxFunc ids used in this pass must be prefixed `oxfunc.local.`.

Current local-only ids:
1. `oxfunc.local.packet_manifest.csv.v1`
2. `oxfunc.local.packet_results.csv.v1`
3. `oxfunc.local.execution_record.md.v1`
4. `oxfunc.local.evidence_registry.table.v1`
5. `oxfunc.local.correlation_ledger.csv.v1`
6. `oxfunc.local.limitation_note.md.v1`

## 9. Witness Lifecycle and Quarantine Usage Rules
Current packet-adapter rule set:
1. source packets and source sidecars remain the primary local evidence assets,
2. any future reduced witness must carry a lifecycle record and a reduction manifest ref,
3. explanatory-only reduced witnesses may support explain surfaces but are not pack-eligible,
4. quarantined witnesses remain indexable and explainable but are not promotion-eligible,
5. superseded reduced witnesses must retain traceability back to the source packet, source evidence ids, and the replacing witness id.

Current OxFunc rollout position:
1. W018 only establishes the adapter binding and capability baseline.
2. W019 defines packet-first reduction units, lifecycle expectations, quarantine rules, and supersession policy.
3. No reduced witness is promoted in this pass.

## 10. Open Gaps and Evidence Requirements
Open gaps:
1. no locally proven reduced packet witness yet exists for OxFunc,
2. no conformance artifact yet proves packet-first `cap.C4.distill_valid`,
3. no `cap.C5.pack_valid` evidence exists,
4. local source schema ids remain adapter-local pending shared `OxReplay` ingestion conventions,
5. no live `DNA ReCalc` import run has yet been exercised against an OxFunc packet bundle in this repo.

Evidence required next:
1. one external replay-host import, preferably `DNA ReCalc`, over the emitted `W15` bundle,
2. one reduced packet or row witness that replays validly before any `cap.C4.distill_valid` claim is made,
3. one follow-up packet beyond `W15` proving the local adapter surface is not packet-specific.

## 11. Worked Example - W15 Packet Binding
`W15` is the first concrete packet example for this adapter baseline.

Source packet elements:
1. manifests:
   - `docs/function-lane/W15_INFO_PRE_SCENARIO_MANIFEST_SEED.csv`
   - `docs/function-lane/W15_CELL_HOST_PRE_SCENARIO_MANIFEST_SEED.csv`
   - `docs/function-lane/W15_XLL_BRIDGE_SCENARIO_MANIFEST_SEED.csv`
2. result sidecars:
   - `.tmp/w15-info-pre-results.csv`
   - `.tmp/w15-info-pre-results-compat.csv`
   - `.tmp/w15-cell-host-pre-results.csv`
   - `.tmp/w15-cell-host-pre-results-compat.csv`
   - `.tmp/w15-xll-bridge-results.csv`
   - `.tmp/w15-xll-bridge-results-compat.csv`
3. execution/evidence/limitation anchors:
   - `docs/function-lane/W15_EXECUTION_RECORD.md`
   - `docs/function-lane/FUNCTION_LANE_EVIDENCE_ID_REGISTRY.md`
   - `docs/function-lane/XLL_VERIFICATION_SEAM_LIMITATIONS.md`

Projection intent:
1. the three manifests remain packet-definition inputs under `oxfunc.local.packet_manifest.csv.v1`,
2. the six result CSVs remain packet-result sidecars under `oxfunc.local.packet_results.csv.v1`,
3. `W15_EXECUTION_RECORD.md` provides packet summary and explain-facing narrative,
4. the three W15 evidence ids remain the stable local evidence-binding spine,
5. XLL limitation references remain limitation metadata, not semantic-failure defaults.

Explain expectation for the worked example:
1. replay explain should be able to answer why a `CELL` lane is host-query classified,
2. why a bridge row is parity-clean or parity-mismatched,
3. and why a mismatch is classified as seam-limited rather than semantically divergent when the limitation record says so.

Concrete target artifact:
1. `docs/function-lane/W15_REPLAY_BUNDLE_SKELETON_V1.md` now defines the expected normalized bundle skeleton for this first worked example.
2. `docs/function-lane/W15_REPLAY_ADAPTER_CONFORMANCE_CHECKLIST_V1.md` now defines the first live-run acceptance checklist for the same packet.
3. `docs/function-lane/W15_REPLAY_DIFF_EXPLAIN_SHAPES_V1.md` now defines the expected diff/explain output objects for the same packet.
4. `.tmp/replay-bundles/oxfunc-w15-v1/` is now the first emitted local proving artifact for the same packet.

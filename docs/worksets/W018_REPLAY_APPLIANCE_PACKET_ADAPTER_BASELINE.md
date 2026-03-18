# WORKSET - TUX1000 Replay Appliance Packet Adapter Baseline (W18)

## 1. Purpose
Incorporate the Foundation Replay appliance handoff into OxFunc without weakening OxFunc-local semantic or evidence doctrine.

Primary intent:
1. define the OxFunc packet adapter role for Replay bundle ingestion, replay, diff, and explain,
2. pin the authority split between OxFunc semantics and Foundation replay governance,
3. publish a machine-readable capability manifest with conservative capability claims,
4. map OxFunc packet artifacts into normalized views without fabricating a fake event stream,
5. prepare the packet-first basis that later witness-distillation work can build on.

## 2. Position and Dependencies
Program position:
1. post-`W015` current-baseline packet and host-query replay packet maturity,
2. post-`W016` bulk non-interesting breadth closure and packet reconciliation discipline,
3. parallel to `W017` residual function hardening, but not owned by the residual function packet.

Dependencies:
1. `W015` for concrete packet replay artifacts with dual-run labels, compatibility descriptors, and seam-limitation distinctions,
2. `W016` for evidence-id discipline at bulk packet scale and explicit packet reconciliation doctrine,
3. Foundation replay handoff package `..\\Foundation\\research\\runs\\20260315-215019-replay-appliance-authoritative-pass-01\\outputs\\`.

Blocking relationship:
1. `W018` blocks `W019`.

## 3. Scope
In scope:
1. packet-adapter contract note,
2. machine-readable capability manifest,
3. local registry and canonical-doc incorporation,
4. packet-view and explain-surface mapping,
5. conservative capability claim baseline through `cap.C3.explain_valid`.

Out of scope:
1. fake event-stream semantics,
2. claiming packet-distillation closure,
3. claiming reduced-witness replay validity,
4. claiming pack-grade export or promotion.

## 4. Working Thesis
OxFunc already has an honest replay shape:
1. manifest-defined packets,
2. row-level observed results,
3. execution-record summaries,
4. evidence-id and correlation-ledger bindings,
5. boundary-invariant declarations,
6. XLL or host-limitation qualifications.

The Replay appliance should therefore consume and normalize that packet shape.
It should not force OxFunc into an invented event-stream-first model.

## 5. Deliverables
1. `docs/function-lane/OXFUNC_REPLAY_APPLIANCE_PACKET_ADAPTER_V1.md`
2. `docs/function-lane/OXFUNC_REPLAY_ADAPTER_CAPABILITY_MANIFEST_V1.json`
3. canonical-doc updates for replay bundle, capability, invariant, and lifecycle binding,
4. registry pin notes for Foundation shared vocabulary and OxFunc local-only source-schema ids,
5. workset sequencing updates showing the replay rollout path after `W017`.

## 5A. First Concrete Packet Binding
The first concrete packet exemplar for `W018` is `W15`.

Why `W15` is the right first exemplar:
1. it already has deterministic manifest-driven packet inputs,
2. it already has dual-run labels and compatibility descriptors,
3. it already has explicit evidence ids,
4. it already has a clear XLL seam-limit distinction,
5. it already exercises both native and bridge-facing packet witnesses.

Required `W15` replay binding surfaces:
1. source manifests and result CSVs bind to local replay source-schema ids,
2. packet rows map into `manifest_row_result_view`,
3. dual-run workbook-lane summaries map into `run_summary_view`,
4. evidence ids and XLL limitation refs remain attached to explain output,
5. no fake internal evaluator event stream is introduced.

Concrete target artifact:
1. `docs/function-lane/W15_REPLAY_BUNDLE_SKELETON_V1.md` defines the expected first normalized bundle shape for this exemplar packet.
2. `docs/function-lane/W15_REPLAY_ADAPTER_CONFORMANCE_CHECKLIST_V1.md` defines what the first live adapter emission must prove for `cap.C0` through `cap.C3`.
3. `docs/function-lane/W15_REPLAY_DIFF_EXPLAIN_SHAPES_V1.md` defines the expected typed diff and explain outputs for that same first live run.

## 6. Gate Model
### G1 - Authority Split Closure
Pass when:
1. OxFunc semantic/evidence authority remains explicit,
2. Foundation replay-governance authority remains explicit,
3. any event-model wording conflict is adapted rather than copied blindly.

### G2 - Adapter Contract Closure
Pass when:
1. packet-first and row-first replay units are explicit,
2. preserved metadata is explicit,
3. normalized views and derived event families are explicit,
4. fake internal event streams remain prohibited.

### G3 - Capability Manifest Closure
Pass when:
1. the adapter manifest exists,
2. capability claims are machine-readable,
3. no claim exceeds `cap.C3.explain_valid`,
4. `cap.C4` and `cap.C5` remain non-claimed.

### G4 - Canonical Incorporation Closure
Pass when:
1. the function-lane README and related canonical docs reference the replay packet adapter baseline,
2. evidence-id discipline, formal alignment notes, invariant checklist fields, and runtime-requirements notes incorporate the replay rollout language,
3. workset sequencing reflects the replay rollout path.

## 7. Status
Execution state:
1. `in_progress`

Claim confidence:
1. `provisional`

Assurance maturity:
1. `not-yet-exercised`

## 8. Completeness Axes
1. `scope_completeness`: `scope_partial`
2. `target_completeness`: `target_partial`
3. `integration_completeness`: `partial`
4. `open_lanes`:
   - no live OxFunc packet has yet been exercised through a Replay bundle import in this repo,
   - `cap.C4.distill_valid` remains scaffolded only,
   - `cap.C5.pack_valid` remains out of scope for this pass.

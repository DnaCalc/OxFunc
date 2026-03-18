# WORKSET - TUX1000 W15 First Live Replay Adapter Run Baseline (W21)

## 1. Purpose
Execute the first live local OxFunc packet-adapter run against the `W15` worked packet and judge it against the predeclared skeleton, conformance checklist, and diff/explain shapes.

Primary intent:
1. emit the first real OxFunc replay bundle,
2. validate ingest/replay/diff/explain against `W15`,
3. produce the first honest proving artifact for local replay capability claims.

## 2. Position and Dependencies
Program position:
1. immediate successor to `W020`,
2. first exercised replay-adapter workset,
3. first candidate proving path for `cap.C0.ingest_valid` through `cap.C3.explain_valid`.

Dependencies:
1. `W018` packet-adapter baseline,
2. `W019` witness-lifecycle baseline,
3. `W020` emitted bundle layout/index baseline,
4. `W15_REPLAY_BUNDLE_SKELETON_V1.md`,
5. `W15_REPLAY_ADAPTER_CONFORMANCE_CHECKLIST_V1.md`,
6. `W15_REPLAY_DIFF_EXPLAIN_SHAPES_V1.md`.

## 3. Scope
In scope:
1. one live adapter emission for the `W15` packet,
2. one bundle validation pass,
3. one deterministic replay rerun,
4. one typed diff run,
5. one explain run over the minimum W15 query set,
6. explicit classification of any failures as projection-gap, seam-limited, or semantic-gap.

Out of scope:
1. witness distillation,
2. pack export,
3. multi-packet generalization beyond `W15`.

## 4. Working Thesis
The first honest replay capability claim for OxFunc should come from one concrete packet, not from generalized prose.

`W15` is the best first proving packet because:
1. it already has deterministic dual-run manifests,
2. it already has evidence ids,
3. it already has XLL seam limitations explicitly classified,
4. it already has a worked-example bundle target.

## 5. Deliverables
1. first live emitted OxFunc replay bundle for `W15`,
2. first bundle validation artifact,
3. first replay/diff/explain run artifacts,
4. first replay-adapter execution record tied to the `W15` packet,
5. capability-claim assessment against `cap.C0` through `cap.C3`.

Current local artifacts:
1. `tools/replay-adapter/run-w15-replay-adapter-baseline.ps1`
2. `.tmp/replay-bundles/oxfunc-w15-v1/`
3. `docs/function-lane/W21_EXECUTION_RECORD.md`

## 6. Gate Model
### G1 - Bundle Emission Closure
Pass when:
1. the emitted bundle matches the `W15` skeleton and `W020` layout sufficiently for validation,
2. projection gaps are surfaced explicitly.

### G2 - Replay and Diff Closure
Pass when:
1. deterministic rerun succeeds for the supported `W15` scope,
2. typed mismatch classes are emitted with pinned registry ids,
3. seam-limited rows remain distinguishable from semantic mismatches.

### G3 - Explain Closure
Pass when:
1. the minimum W15 explain query set succeeds or reports explicit explain gaps,
2. each explain result cites supporting refs correctly.

### G4 - Capability Assessment Closure
Pass when:
1. `cap.C0` through `cap.C3` are either evidenced or explicitly not yet evidenced,
2. no overclaim is made for `cap.C4` or `cap.C5`.

## 7. Status
Execution state:
1. `complete`

Claim confidence:
1. `provisional`

Assurance maturity:
1. `exercised-locally`

## 8. Completeness Axes
1. `scope_completeness`: `scope_complete`
2. `target_completeness`: `target_complete`
3. `integration_completeness`: `integrated`
4. `open_lanes`:
   - no `DNA ReCalc` import has been exercised yet against the emitted bundle,
   - no reduced witness has been proven replay-valid,
   - no pack-grade export/promotion flow exists yet.

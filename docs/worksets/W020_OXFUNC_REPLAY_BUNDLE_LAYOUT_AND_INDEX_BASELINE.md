# WORKSET - TUX1000 OxFunc Replay Bundle Layout and Index Baseline (W20)

## 1. Purpose
Define the first explicit OxFunc-local filesystem layout and index contract for emitted replay bundles.

Primary intent:
1. turn the current `W15` bundle skeleton from an in-memory shape into an on-disk target layout,
2. define where manifests, views, indexes, sidecars, registry refs, and capability refs should land,
3. make the first future local adapter run auditable and repeatable without ad hoc path invention.

## 2. Position and Dependencies
Program position:
1. immediate successor to `W019`,
2. doctrinal/logistics-layer work that is now exercised by the first local emitted bundle,
3. prerequisite for the first live packet-adapter run against a concrete filesystem target.

Dependencies:
1. `W018` packet-adapter baseline,
2. `W019` witness-lifecycle baseline,
3. `W15_REPLAY_BUNDLE_SKELETON_V1.md`,
4. `W15_REPLAY_ADAPTER_CONFORMANCE_CHECKLIST_V1.md`,
5. `W15_REPLAY_DIFF_EXPLAIN_SHAPES_V1.md`.

## 3. Scope
In scope:
1. bundle-root directory shape for OxFunc packet-first replay bundles,
2. canonical index file set,
3. sidecar-ref resolution rules,
4. local path and naming rules for first emitted bundles,
5. W15 worked-example layout target.

Out of scope:
1. live adapter emission,
2. proving `cap.C0` through `cap.C3`,
3. pack export layout,
4. reduced-witness layout promotion beyond placeholder reservations.

## 4. Working Thesis
OxFunc needs one honest emitted-bundle layout before any local adapter run can count as replay evidence.

That layout should:
1. preserve packet-first and row-first semantics,
2. keep source artifacts and normalized views separately visible,
3. keep evidence ids, limitation refs, and registry snapshots easy to index,
4. reserve space for later reductions without implying `cap.C4` today.

## 5. Deliverables
1. bundle-layout and index baseline note under `docs/function-lane/`,
2. worked-example layout target for `W15`,
3. workset sequence updates showing `W020` as the layout/index bridge between doctrine and first exercised run.

Current local artifacts:
1. `docs/function-lane/OXFUNC_REPLAY_BUNDLE_LAYOUT_AND_INDEX_V1.md`
2. `docs/function-lane/W15_REPLAY_BUNDLE_SKELETON_V1.md`
3. `docs/function-lane/W15_REPLAY_ADAPTER_CONFORMANCE_CHECKLIST_V1.md`
4. `docs/function-lane/W15_REPLAY_DIFF_EXPLAIN_SHAPES_V1.md`
5. `.tmp/replay-bundles/oxfunc-w15-v1/`
6. `docs/function-lane/W21_EXECUTION_RECORD.md`

## 6. Gate Model
### G1 - Layout Closure
Pass when:
1. bundle-root directories and required files are explicit,
2. view and index file naming is explicit,
3. sidecar resolution rules are explicit.

### G2 - Worked Example Closure
Pass when:
1. `W15` has an explicit emitted-bundle layout target,
2. index entries required for row, evidence, limitation, and run lookup are explicit.

### G3 - Non-Overclaim Closure
Pass when:
1. no live adapter execution is implied,
2. no capability claim beyond documentation baseline is made,
3. reduction and pack areas remain explicitly reserved only.

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
   - none in declared `W020` scope after the first emitted `W15` bundle validated the layout and index contract locally.

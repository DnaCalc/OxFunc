# W21 W15 First Live Replay Run Contract V1

Status: `provisional`
Owner lane: `OxFunc`
Workset: `W21`

## 1. Purpose
Define the exact execution contract for the first live local replay-adapter run over the `W15` packet.

This note is the run contract for `W21`.

## 2. Expected Inputs
Required source inputs:
1. `docs/function-lane/W15_INFO_PRE_SCENARIO_MANIFEST_SEED.csv`
2. `docs/function-lane/W15_CELL_HOST_PRE_SCENARIO_MANIFEST_SEED.csv`
3. `docs/function-lane/W15_XLL_BRIDGE_SCENARIO_MANIFEST_SEED.csv`
4. `.tmp/w15-info-pre-results.csv`
5. `.tmp/w15-info-pre-results-compat.csv`
6. `.tmp/w15-cell-host-pre-results.csv`
7. `.tmp/w15-cell-host-pre-results-compat.csv`
8. `.tmp/w15-xll-bridge-results.csv`
9. `.tmp/w15-xll-bridge-results-compat.csv`
10. `docs/function-lane/W15_EXECUTION_RECORD.md`
11. `docs/function-lane/XLL_VERIFICATION_SEAM_LIMITATIONS.md`

Required target docs:
1. `docs/function-lane/W15_REPLAY_BUNDLE_SKELETON_V1.md`
2. `docs/function-lane/W15_REPLAY_ADAPTER_CONFORMANCE_CHECKLIST_V1.md`
3. `docs/function-lane/W15_REPLAY_DIFF_EXPLAIN_SHAPES_V1.md`
4. `docs/function-lane/OXFUNC_REPLAY_BUNDLE_LAYOUT_AND_INDEX_V1.md`

## 3. Expected Output Root
Target output root:
1. `.tmp/replay-bundles/oxfunc-w15-v1/`

## 4. Required Phases
The first live run must perform:
1. ingest
2. emit normalized bundle
3. validate emitted bundle
4. replay supported rows deterministically
5. diff `default` vs `compat_template`
6. explain the minimum W15 query set

## 5. Minimum Output Artifacts
Required outputs:
1. emitted bundle root under `.tmp/replay-bundles/oxfunc-w15-v1/`
2. bundle validation result
3. replay result artifact
4. diff artifact
5. explain artifact set
6. execution record for the live run

## 6. Acceptance Reference Set
The run is judged against:
1. `W15_REPLAY_BUNDLE_SKELETON_V1.md`
2. `W15_REPLAY_ADAPTER_CONFORMANCE_CHECKLIST_V1.md`
3. `W15_REPLAY_DIFF_EXPLAIN_SHAPES_V1.md`
4. `OXFUNC_REPLAY_BUNDLE_LAYOUT_AND_INDEX_V1.md`

## 7. Current Blocking Reality
Current repo state now exposes:
1. a local packet-adapter implementation,
2. a bundle emitter,
3. a bundle validator,
4. a replay/diff/explain runner for the `W15` worked packet.

Implemented entrypoint:
1. `tools/replay-adapter/run-w15-replay-adapter-baseline.ps1`

Primary proving artifact:
1. `.tmp/replay-bundles/oxfunc-w15-v1/`

## 8. Failure Classification Rule
If `W21` is attempted before the missing substrate exists:
1. missing implementation surface is a blocker, not a failed capability claim,
2. no synthetic or hand-written bundle should be misreported as a live adapter output,
3. manual files may exist only as target artifacts, not as proving artifacts.

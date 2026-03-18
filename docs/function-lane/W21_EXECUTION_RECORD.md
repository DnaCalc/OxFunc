# W21 Execution Record

Status: `complete-provisional`
Workset: `W21`
Bundle root: `.tmp/replay-bundles/oxfunc-w15-v1`

## 1. Purpose
Record the first live local OxFunc replay-adapter run over the `W15` worked packet.

## 2. Scope
1. emit the first real local OxFunc replay bundle for `W15`,
2. validate the emitted bundle against the predeclared layout and field expectations,
3. replay the packet rows deterministically from the emitted bundle views,
4. emit typed diff and explain artifacts,
5. assess local evidence for `cap.C0.ingest_valid` through `cap.C3.explain_valid`.

## 3. Completeness Axes
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`
5. open_lanes:
   - no `DNA ReCalc` import has been exercised yet against the emitted bundle,
   - no reduced witness has been proven replay-valid,
   - no pack-grade export/promotion flow exists yet.

## 4. Executed Scope
Execution date:
1. `2026-03-17`

Primary tool entrypoint:
1. `tools/replay-adapter/run-w15-replay-adapter-baseline.ps1`

Primary source inputs:
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

Primary emitted outputs:
1. `.tmp/replay-bundles/oxfunc-w15-v1/bundle_manifest.json`
2. `.tmp/replay-bundles/oxfunc-w15-v1/views/manifest_row_result_view/w15.default.csv`
3. `.tmp/replay-bundles/oxfunc-w15-v1/views/manifest_row_result_view/w15.compat_template.csv`
4. `.tmp/replay-bundles/oxfunc-w15-v1/diff/emitted/w15.default_vs_compat.json`
5. `.tmp/replay-bundles/oxfunc-w15-v1/explain/emitted/w15.explain.json`
6. `.tmp/replay-bundles/oxfunc-w15-v1/sidecars/normalized/w15.validation.json`
7. `.tmp/replay-bundles/oxfunc-w15-v1/sidecars/normalized/w15.replay_result.json`
8. `.tmp/replay-bundles/oxfunc-w15-v1/sidecars/normalized/w15.capability_assessment.json`

## 5. Findings
1. the emitted bundle matched the declared `W20` filesystem layout and required file set for the current baseline,
2. packet-row replay from the emitted views was deterministic: `expected_row_count=140`, `observed_row_count=140`, `replay_valid=true`,
3. local capability evidence is now present for:
   - `cap.C0.ingest_valid`
   - `cap.C1.replay_valid`
   - `cap.C2.diff_valid`
   - `cap.C3.explain_valid`
4. the current run still does not justify:
   - `cap.C4.distill_valid`
   - `cap.C5.pack_valid`
5. no projection gaps were surfaced in the baseline emitted bundle,
6. emitted diff rows were limited to workbook-identity-sensitive `CELL("filename")` lanes and their bridge analogues across `default` versus `compat_template`,
7. those current diff rows are classified as `sev.informational`, not as core semantic failures and not as XLL seam failures.

## 6. Capability Assessment
From `.tmp/replay-bundles/oxfunc-w15-v1/sidecars/normalized/w15.capability_assessment.json`:
1. `cap.C0.ingest_valid` -> `true`
2. `cap.C1.replay_valid` -> `true`
3. `cap.C2.diff_valid` -> `true`
4. `cap.C3.explain_valid` -> `true`
5. `cap.C4.distill_valid` -> `false`
6. `cap.C5.pack_valid` -> `false`

## 7. Diff Classification Notes
Current emitted diff rows:
1. `W15CELL-001`
2. `W15CELL-009E`
3. `W15CELL-010`
4. `W15XLL-011`
5. `W15XLL-019E`
6. `W15XLL-020`

Interpretation:
1. these rows differ because the `default` and `compat_template` runs use different saved workbook identities and file formats,
2. the row truth is still preserved within each run label,
3. this is a run-surface/environment distinction and is currently classified as `sev.informational`.

## 8. Standing
1. `W20` now has an exercised emitted-bundle witness rather than a target-only layout note,
2. `W21` is no longer blocked by missing adapter implementation,
3. OxFunc now has one real local replay-bundle proving artifact for packet-first replay support,
4. the next step toward `W019` witness work is not more bundle doctrine; it is one packet-first reduced witness that remains replay-valid.

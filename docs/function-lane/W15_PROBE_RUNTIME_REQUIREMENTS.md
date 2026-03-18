# W15 Probe Runtime Requirements

Status: `provisional`
Workset: `W15`

## 1. Purpose
Define empirical replay requirements for the `CELL` / `INFO` host-query workset.

## 2. Scenario Packs
1. `docs/function-lane/W15_INFO_PRE_SCENARIO_MANIFEST_SEED.csv`
2. `docs/function-lane/W15_CELL_HOST_PRE_SCENARIO_MANIFEST_SEED.csv`

## 3. Required Runtime Lanes
1. formula recalculation path for `INFO(type_text)` environment/workbook queries.
2. saved-workbook path for `CELL("filename", ...)`.
3. explicit formatting/alignment/protection setup path for host-sensitive `CELL` lanes.
4. output capture for both displayed text and `Value2`.

## 4. Expected Output Artifacts
1. `.tmp/w15-info-pre-results.csv`
2. `.tmp/w15-info-pre-results-compat.csv`
3. `.tmp/w15-cell-host-pre-results.csv`
4. `.tmp/w15-cell-host-pre-results-compat.csv`
5. `.tmp/w15-xll-bridge-results.csv`
6. `.tmp/w15-xll-bridge-results-compat.csv`

## 5. Replay Commands
1. `powershell -File tools/w15-probe/run-w15-suite.ps1 -OutDir .tmp`
2. `powershell -File tools/w15-probe/run-w15-info-preprobe.ps1 -Manifest docs/function-lane/W15_INFO_PRE_SCENARIO_MANIFEST_SEED.csv -Out .tmp/w15-info-pre-results.csv -RunLabel default`
3. `powershell -File tools/w15-probe/run-w15-cell-host-preprobe.ps1 -Manifest docs/function-lane/W15_CELL_HOST_PRE_SCENARIO_MANIFEST_SEED.csv -Out .tmp/w15-cell-host-pre-results.csv -RunLabel default`
4. `powershell -File tools/w15-probe/run-w15-xll-bridge.ps1 -Manifest docs/function-lane/W15_XLL_BRIDGE_SCENARIO_MANIFEST_SEED.csv -Out .tmp/w15-xll-bridge-results.csv -BuildIfMissing -RunLabel default`

## 6. Classification Rules
1. `INFO` and `CELL` rows must now be replayed on both workbook lanes: `default` and `compat_template`.
2. every output row must carry `run_label` and `compat_descriptor` so workbook-mode behavior remains auditable.
3. `CELL` host rows exist to pin which option families require a host-query provider rather than a pure resolver-only path.
4. these W15 probe packs provide seam-definition evidence plus current-baseline parity evidence; upstream acknowledgment is still tracked separately.

## 7. Replay Appliance / DNA ReCalc Import Path
Planned import path under the replay packet adapter baseline:
1. `dnarecalc ingest --lane oxfunc --source docs/function-lane/W15_INFO_PRE_SCENARIO_MANIFEST_SEED.csv`
2. `dnarecalc ingest --lane oxfunc --source docs/function-lane/W15_CELL_HOST_PRE_SCENARIO_MANIFEST_SEED.csv`
3. `dnarecalc normalize --input <w15-packet-root> --out <bundle-root>`
4. `dnarecalc replay --bundle <bundle-root> --scenario <row-id-or-packet-id>`

Import rule:
1. the packet adapter must treat W15 manifests and result CSV sidecars as first-class packet witnesses,
2. it must not fabricate an internal evaluator event stream,
3. it must preserve host-query seam limitations and dual-run workbook-lane identity.

## 8. Normalized Output Expectations
Normalized output for W15-style packets should preserve:
1. packet/workset identity,
2. row id or scenario id,
3. `run_label`,
4. `compat_descriptor`,
5. locale/environment metadata,
6. evidence-id refs,
7. execution-record refs,
8. XLL or host-limitation refs where applicable.

Expected normalized views:
1. `manifest_row_result_view`
2. `run_summary_view`
3. `analysis_summary_view`
4. `evidence_binding_view`
5. `limitation_view`

## 9. Explain and Future Distill Eligibility
Explain expectation:
1. W15 packets should support row- and packet-level explain surfaces that cite row ids, evidence ids, and limitation refs.

Future distill note:
1. W15 packets are candidates for future packet-first reduced-witness experiments under `W019`,
2. but no reduced witness is claimed replay-valid yet,
3. and any future reduced witness must preserve workbook-lane identity and host-limitation classification.

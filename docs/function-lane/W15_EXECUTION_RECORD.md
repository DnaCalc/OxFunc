# W15 Execution Record

Status: `complete-provisional`
Workset: `W15`
Evidence IDs: `W15-INFO-PRE-20260315`; `W15-CELL-HOST-PRE-20260315`; `W15-XLL-BRIDGE-20260315`

## 1. Purpose
Track the initial execution state for the `CELL` / `INFO` host-query workset.

## 2. Scope
1. define the typed host-query seam for `CELL` and `INFO`,
2. carry forward the deferred `CELL` closure work from `W12`,
3. create and replay the initial `INFO` empirical baseline,
4. prepare runtime/formal artifacts for later admitted `CELL` / `INFO` implementation slices and integrate them into the XLL bridge.

## 3. Completeness Axes
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`
5. open_lanes:
   - broader locale/alternate-version sweeps remain orthogonal validation-phase work.
   - XLL verification-seam limitations remain tracked separately where they qualify worksheet-parity claims.

## 4. Executed Scope
Execution date:
1. `2026-03-15`

Artifacts created or updated:
1. `docs/worksets/W015_CELL_AND_INFO_HOST_QUERY_FUNCTIONS.md`
2. `docs/function-lane/FUNCTION_SLICE_INFO_CONTRACT_PRELIM.md`
3. `docs/function-lane/CELL_INFO_HOST_QUERY_SEAM_PRELIM.md`
4. `docs/function-lane/W15_INFO_PRE_SCENARIO_MANIFEST_SEED.csv`
5. `tools/w15-probe/run-w15-info-preprobe.ps1`
6. `crates/oxfunc_core/src/host_info.rs`
7. `crates/oxfunc_core/src/functions/info_fn.rs`
8. `formal/lean/OxFunc/HostInfoSeam.lean`
9. `formal/lean/OxFunc/Functions/Info.lean`
10. `docs/handoffs/HANDOFF_W015_CELL_INFO_HOST_QUERY_TO_OXFML.md`

## 5. Empirical Findings
Replayed `INFO(type_text)` on the local Excel host for both `default` and `compat_template` workbook descriptors:
1. `directory`
2. `numfile`
3. `origin`
4. `osversion`
5. `recalc`
6. `release`
7. `system`
8. `memavail`
9. `memused`
10. `totmem`

Observed outcomes:
1. `directory` -> non-empty directory text
2. `numfile` -> `1`
3. `origin` -> `$A:$A$1`
4. `osversion` -> `Windows (64-bit) NT 10.00`
5. `recalc` -> `Automatic`
6. `release` -> `16.0`
7. `system` -> `pcdos`
8. `memavail` / `memused` / `totmem` -> `#N/A`
9. broadened `CELL` host-sensitive and active-selection lanes on both workbook descriptors:
   - `filename` -> saved-workbook path ending in `[cell-host-probe.xlsx]Sheet1`
   - `format` -> `F2`
   - `color` -> `1`
   - `parentheses` -> `1`
   - `prefix` -> `'`
   - `protect` -> `1`
   - `width` -> `20`, with native `INDEX(CELL("width",...),2) -> FALSE` and `COLUMNS(CELL("width",...)) -> 2`
   - omitted-reference active-selection lanes now pinned across the admitted set:
     - `row`, `address`, `col`, `contents`, `type`
     - `filename`, `format`, `color`, `prefix`, `protect`, `width`, `parentheses`
10. XLL bridge parity on both workbook descriptors:
   - `ox_INFO(...)` matched native `INFO(...)` for all ten seeded lanes
   - `ox_CELL(...)` matched native `CELL(...)` for explicit-reference lanes:
     - `filename`, `format`, `color`, `parentheses`, `prefix`, `protect`, `width`
   - `ox_CELL(...)` matched native `CELL(...)` for omitted-reference active-selection lanes:
     - `row`, `address`, `col`, `contents`, `type`
     - `filename`, `format`, `color`, `prefix`, `protect`, `width`, `parentheses`
   - `ox_CELL(...)` matched native `CELL(...)` for cross-sheet explicit-reference `filename` and `format`
11. generated ordinary `ox_CELL` / `ox_INFO` exports worked without the `#` macro-type suffix in this provider-backed path, even though the manual legacy `GET.*` probe wrappers still require `#`

## 6. Verification Runs
1. `powershell -File tools/w15-probe/run-w15-info-preprobe.ps1 -Manifest docs/function-lane/W15_INFO_PRE_SCENARIO_MANIFEST_SEED.csv -Out .tmp/w15-info-pre-results.csv`
2. `powershell -File tools/w15-probe/run-w15-cell-host-preprobe.ps1 -Manifest docs/function-lane/W15_CELL_HOST_PRE_SCENARIO_MANIFEST_SEED.csv -Out .tmp/w15-cell-host-pre-results.csv`
3. `powershell -File tools/w15-probe/run-w15-suite.ps1 -OutDir .tmp`
4. `cargo fmt --manifest-path crates/oxfunc_core/Cargo.toml`
5. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml`
6. `cargo check --manifest-path tools/xll-addin/oxfunc_xll/Cargo.toml`
7. `powershell -File tools/xll-addin/build-oxfunc-xll.ps1 -Profile release`
8. `lake build` (from `formal/lean`)

## 7. Replay Appliance Worked Packet Example
This workset is the first concrete packet exemplar for the OxFunc replay adapter baseline.

Candidate source-schema binding for `W15`:
1. `docs/function-lane/W15_INFO_PRE_SCENARIO_MANIFEST_SEED.csv` -> `oxfunc.local.packet_manifest.csv.v1`
2. `docs/function-lane/W15_CELL_HOST_PRE_SCENARIO_MANIFEST_SEED.csv` -> `oxfunc.local.packet_manifest.csv.v1`
3. `docs/function-lane/W15_XLL_BRIDGE_SCENARIO_MANIFEST_SEED.csv` -> `oxfunc.local.packet_manifest.csv.v1`
4. `.tmp/w15-info-pre-results.csv`, `.tmp/w15-info-pre-results-compat.csv`, `.tmp/w15-cell-host-pre-results.csv`, `.tmp/w15-cell-host-pre-results-compat.csv`, `.tmp/w15-xll-bridge-results.csv`, `.tmp/w15-xll-bridge-results-compat.csv` -> `oxfunc.local.packet_results.csv.v1`
5. `docs/function-lane/W15_EXECUTION_RECORD.md` -> `oxfunc.local.execution_record.md.v1`
6. `docs/function-lane/FUNCTION_LANE_EVIDENCE_ID_REGISTRY.md` rows `W15-INFO-PRE-20260315`, `W15-CELL-HOST-PRE-20260315`, `W15-XLL-BRIDGE-20260315` -> `oxfunc.local.evidence_registry.table.v1`
7. `docs/function-lane/XLL_VERIFICATION_SEAM_LIMITATIONS.md` -> `oxfunc.local.limitation_note.md.v1`

Candidate normalized replay views for `W15`:
1. `manifest_row_result_view` for the individual `INFO`, `CELL`, and bridge parity rows
2. `run_summary_view` for `default` and `compat_template` workbook-lane summaries
3. `analysis_summary_view` for the packet-level host-query and parity conclusions
4. `evidence_binding_view` for `W15-INFO-PRE-20260315`, `W15-CELL-HOST-PRE-20260315`, and `W15-XLL-BRIDGE-20260315`
5. `limitation_view` for the XLL seam qualification carried from `XLL_VERIFICATION_SEAM_LIMITATIONS.md`

Candidate explain surfaces for `W15`:
1. why a given `INFO(type_text)` row differs by run label, if it ever does
2. why a given `CELL(info_type)` lane is classified as host-query rather than pure-local
3. why an XLL parity mismatch would count as a seam limitation rather than a core semantic mismatch when that classification is warranted

Current replay-binding rule:
1. the packet adapter may project `row.observed`, `analysis.completed`, `evidence.bound`, and `limitation.noted` views for `W15`,
2. but it must not fabricate hidden evaluator-step events between manifest rows and observed outputs.

## 8. Standing
1. seam-definition work is now explicit and evidence-backed,
2. `INFO` is confirmed to be a typed host-query function for the seeded lanes,
3. `INFO` is now integrated through the XLL bridge and matches native `INFO(...)` on both `default` and `compat_template` workbook descriptors,
4. broadened `CELL` lanes are pinned by dedicated native and XLL parity manifests, including the raw two-item `width` artifact, cross-sheet refs, and omitted-reference active-selection behavior across the admitted info-type set,
5. the local typed host-query seam is now implemented in Rust and mirrored in Lean substrate bindings,
6. `CELL` and `INFO` now share a standardized provider seam rather than ad hoc workbook inspection logic,
7. no known local semantic gap remains in the admitted current-baseline `CELL` / `INFO` slice,
8. OxFml has now acknowledged `HO-FN-002` in both its upstream note and handoff register, so the cross-repo seam gate is closed for the declared `W015` scope.

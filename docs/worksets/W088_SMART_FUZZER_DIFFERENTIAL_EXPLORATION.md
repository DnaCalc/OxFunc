# W088 Smart-Fuzzer Differential Exploration Pilot

Status: `in_progress`

## 1. Purpose

Establish the first OxFunc smart-fuzzer execution lane for differential
exploration against Excel without turning generated pass cases into heavyweight
documentation.

The pilot owns the first practical harness loop:

1. compact telemetry and failure-packet artifact discipline,
2. Excel batch-throughput measurement,
3. static metadata and risk indexing,
4. typed candidate generation for bounded pilot families,
5. local Rust/OxFml evaluation before Excel spend,
6. typed comparison and mismatch classification,
7. reduced failure promotion through the ordinary bug stream.

This workset does not claim full catalog exploration or function semantic
closure. It establishes the execution substrate and proves the loop on bounded
pilot surfaces.

## 2. Depends On

1. `W070` for bead-based execution discipline.
2. `W072` for bug intake, root-cause, and regression-stream protocol.
3. `W044` for the current library-context snapshot export input.
4. `W049` for the runtime provider/snapshot direction.

## 3. Parent Doctrine And Spec Surfaces

1. `CHARTER.md`
2. `OPERATIONS.md`
3. `docs/BEADS.md`
4. `docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1_README.md`
5. `docs/function-lane/OXFUNC_SURFACE_ADMISSION_AND_LABELING_POLICY.md`
6. `docs/bugs/README.md`
7. `smart-fuzzer/README.md`
8. `smart-fuzzer/planning/SMART_FUZZER_DESIGN.md`
9. `smart-fuzzer/planning/CASE_SCHEMA_V0.md`

## 4. Upstream Dependencies

1. `OxFml` prepared-argument and evaluator-seam behavior.
2. `../OxFml/docs/upstream/NOTES_FOR_OXFUNC.md` for current seam floor.

## 5. Scope

In scope:

1. define compact telemetry and failure-packet formats for the pilot,
2. implement or script an Excel batch-throughput benchmark,
3. emit run metadata with Excel version/channel, workbook compatibility, runner
   version, manifest hash, git revision, and tree state,
4. build a derived static index from existing OxFunc metadata and known bug
   surfaces,
5. generate bounded candidate cases for live known-risk pilot families and
   stale-claim review rows,
6. run local OxFunc/OxFml evaluation before Excel comparison where feasible,
7. compare typed outcomes with function-family-aware policies,
8. minimize or preserve mismatch candidates with reduction lineage,
9. route confirmed mismatches through `docs/bugs/` rather than this workset.

Out of scope for this pilot:

1. full 500-function catalog exploration,
2. locale and alternate Excel-version sweeps beyond captured metadata,
3. broad XLL seam fuzzing,
4. provider/cube/RTD live-provider parity,
5. promotion of sampled pass rows as function closure evidence,
6. replacement of existing function-lane scenario manifests or bug streams.

## 6. Pilot Families

Initial pilots should focus on current live known-risk classes and stale-claim
review rows:

1. PMT / PPMT residual financial-payment lanes,
2. numeric comparison tolerance and exact-match contrast lanes,
3. lookup-family array-valued lookup inputs,
4. text scalar and text-slice array-lift behavior,
5. omitted optional arguments in dynamic-array functions,
6. `COUNTBLANK` range-only behavior,
7. reference and aggregate preparation, including same-sheet multi-area lanes,
8. stale bug-stream sanity checks, including any remaining `POWER` claim,
   which must be freshly confirmed before it is treated as a current bug.

## 7. Artifact Economy Rule

The expected pass-to-fail ratio is high.

Rules:

1. ordinary passing cases are compact telemetry, not per-case prose artifacts,
2. coverage rollups and throughput summaries are first-class outputs,
3. full packets are reserved for mismatches, unstable outcomes, blocked harness
   rows, and reduced reproducers,
4. only reduced or otherwise durable failures move into canonical bug, evidence,
   or regression surfaces.

## 8. Initial Epic Lanes

1. artifact economy and schema discipline,
2. Excel throughput benchmark,
3. static metadata/risk index,
4. bounded pilot generator,
5. local evaluator and Excel comparator,
6. mismatch minimization and promotion path.

## 9. Closure Condition

This pilot reaches its terminal gate only when:

1. a reproducible throughput benchmark exists and records phase counters,
2. compact telemetry and failure-packet formats are exercised by at least one
   pilot run,
3. the static index consumes the current library-context snapshot and known bug
   surfaces,
4. at least one bounded pilot family runs through local evaluation, Excel
   comparison, and typed comparison,
5. any confirmed mismatch is either minimized and routed through `docs/bugs/`,
   or explicitly classified as seam/harness blocked,
6. pass statistics are reported as coverage telemetry rather than completion
   evidence.

## 10. Reporting Contract

All W088 reports must include:

1. `execution_state`,
2. `scope_completeness`,
3. `target_completeness`,
4. `integration_completeness`,
5. explicit `open_lanes` while any axis remains partial.

Use `in_progress` when uncertain.

## 11. Execution Notes

### 2026-04-28 Pilot Infrastructure Pass
1. Added the Excel throughput runner
   `smart-fuzzer/tools/Run-ExcelThroughputBenchmark.ps1`.
2. Produced ignored local run `smart-fuzzer/runs/w088-excel-throughput-baseline/`:
   - generated cases: `6100`
   - Excel evaluated: `6100`
   - blocked: `0`
   - measured formula write/calculate/extract throughput: about `60.9k`
     cases/second for the 100/1000/5000 batch mix
3. Added the PMT/PPMT pilot comparator
   `smart-fuzzer/tools/Run-PmtPpmtPilot.ps1` and standalone local evaluator
   `smart-fuzzer/tools/pmt_ppmt_local_eval/`.
4. Produced ignored local run `smart-fuzzer/runs/w088-pmt-ppmt-pilot/`:
   - generated cases: `28`
   - local evaluated: `28`
   - Excel evaluated: `28`
   - exact matches: `7`
   - numeric bit mismatches: `21`
   - blocked: `0`
5. Routed confirmed PMT/PPMT exactness drift into:
   - `docs/bugs/reports/BUGREP-FUNC-019_smart_fuzzer_pmt_ppmt_exactness_drift.md`
   - `docs/bugs/streams/BUG-FUNC-015_pmt_ppmt_annuity_exactness_drift.md`
   - follow-up bead `oxf-fckb`

Status axes after this pass:

1. `scope_completeness`: `scope_partial`
2. `target_completeness`: `target_partial`
3. `integration_completeness`: `partial`
4. `open_lanes`: broader catalog exploration, mismatch minimization, adjacent
   financial-family scan, stale `POWER` review, and follow-up PMT/PPMT repair

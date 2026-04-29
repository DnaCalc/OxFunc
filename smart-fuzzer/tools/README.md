# Smart-Fuzzer Tools

Status: `tooling_sandbox`

Tracked tools in this directory are reproducible helpers for W088 and W089
exploration, plus W090 array-support sweep planning.
Generated outputs should normally go to `smart-fuzzer/cache/` or
`smart-fuzzer/runs/`, both ignored by default.

## Build-DimensionInventory.ps1

Builds the W089 function-by-function dimension inventory for sweep planning. It
derives arity, value-kind, numeric/text, array, reference, context, execution
seam, bit-exact comparison-policy, known-deviation, blocked/deferred, and
coverage-counter axes from the current library-context snapshot plus related
registers.

```powershell
powershell -ExecutionPolicy Bypass -File smart-fuzzer\tools\Build-DimensionInventory.ps1
```

Default output:

```text
smart-fuzzer/cache/dimension-inventory-v0.json
```

The inventory is not semantic authority and does not run the fuzzer. It is the
input map for later generator and budget beads.

## Build-SweepPlanningArtifacts.ps1

Builds the remaining W089 planning artifacts from the dimension inventory. If
the inventory cache is absent, this script rebuilds it first. It does not
generate fuzzer cases, run local evaluation, run Excel, or compare outcomes.

```powershell
powershell -ExecutionPolicy Bypass -File smart-fuzzer\tools\Build-SweepPlanningArtifacts.ps1
```

Default outputs:

```text
smart-fuzzer/cache/generator-matrix-v0.json
smart-fuzzer/cache/local-dry-run-budget-v0.json
smart-fuzzer/cache/excel-candidate-budget-v0.json
smart-fuzzer/cache/blocked-seam-map-v0.json
smart-fuzzer/cache/roadmap-trace-template-v0.json
```

These outputs are derived planning cache files for W089. They are not
comparison evidence.

## Build-ArraySupportSweepPlan.ps1

Builds the W090 array-support candidate inventory, first-tranche plan, compact
replay matrix, and generated highlights from the W089 dimension inventory plus
source-code risk signals. It does not run local evaluation, run Excel, or
compare outcomes.

```powershell
powershell -ExecutionPolicy Bypass -File smart-fuzzer\tools\Build-ArraySupportSweepPlan.ps1 -RefreshInventory
```

Default outputs:

```text
smart-fuzzer/cache/array-support-candidate-inventory-v0.json
smart-fuzzer/cache/array-support-first-tranche-v0.json
smart-fuzzer/cache/array-support-replay-matrix-v0.json
smart-fuzzer/cache/array-support-highlights-v0.md
```

The first generated tranche is
`w090-tranche-a-math-scalar-numeric-array-lift`. Cache rows are exploration
inputs only; pass rows from later execution remain aggregate telemetry, and
unexpected mismatches must be promoted through `BUG-FUNC-*` or narrower repair
beads.

## Build-StaticRiskIndex.ps1

Builds a derived function risk index for exploration ordering. It consumes:

1. `docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv`,
2. `docs/function-lane/FUNCTION_CATALOG_CURRENT_BASELINE_LOCAL.csv`,
3. `docs/bugs/BUG_STREAM_REGISTER.csv`,
4. `docs/function-lane/*SCENARIO_MANIFEST_SEED.csv`,
5. `docs/function-lane/*DEFERRED*INVENTORY*.csv`,
6. `crates/oxfunc_core/src/functions/*.rs`.

Default output:

```powershell
powershell -ExecutionPolicy Bypass -File smart-fuzzer\tools\Build-StaticRiskIndex.ps1
```

The default index path is `smart-fuzzer/cache/static-risk-index.json`. The
index is not semantic authority; it is a disposable exploration-ordering input.

## Run-ExcelThroughputBenchmark.ps1

Runs a COM-driven Excel batch benchmark and writes artifact-contract files under
`smart-fuzzer/runs/<run_id>/`.

```powershell
powershell -ExecutionPolicy Bypass -File smart-fuzzer\tools\Run-ExcelThroughputBenchmark.ps1
```

Custom batch sizes can be passed as a comma-separated string:

```powershell
powershell -ExecutionPolicy Bypass -File smart-fuzzer\tools\Run-ExcelThroughputBenchmark.ps1 `
  -RunId local-excel-throughput-smoke -BatchSizes "100,1000,5000"
```

The benchmark records cold start, formula write time, calculation time, result
extraction time, Excel version/build, workbook compatibility where COM exposes
it, git revision, runner version, and a manifest hash. If Excel automation is
not available, it writes a blocked telemetry row instead of treating the result
as a function mismatch.

## Run-PmtPpmtPilot.ps1

Runs the first OxFunc-vs-Excel pilot comparator over a bounded PMT/PPMT case
set. The script writes compact case, outcome, comparison, telemetry, manifest,
and rollup artifacts under `smart-fuzzer/runs/<run_id>/`; full per-case packets
are written only for mismatches.

```powershell
powershell -ExecutionPolicy Bypass -File smart-fuzzer\tools\Run-PmtPpmtPilot.ps1 `
  -RunId local-pmt-ppmt-pilot
```

The local side is evaluated through the standalone Rust helper in
`smart-fuzzer/tools/pmt_ppmt_local_eval/`, which calls the public
`oxfunc_core` value surface without adding files to the main workspace.

## Run-ExpandedFinanceExploration.ps1

Runs the larger financial-neighborhood exploration lane. The Rust explorer
generates and locally evaluates a high-volume PMT/PPMT/IPMT case set, then the
PowerShell wrapper evaluates selected candidates in Excel and compares typed
outcomes.

```powershell
powershell -ExecutionPolicy Bypass -File smart-fuzzer\tools\Run-ExpandedFinanceExploration.ps1 `
  -RunId local-expanded-finance-10m -CaseCount 10000000
```

The expected PMT/PPMT/IPMT non-zero-rate exactness drift is classified as
`expected_known_financial_exactness_drift`. Unexpected mismatches are written as
failure packets.

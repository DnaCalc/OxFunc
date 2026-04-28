# Smart-Fuzzer Tools

Status: `tooling_sandbox`

Tracked tools in this directory are reproducible helpers for W088 exploration.
Generated outputs should normally go to `smart-fuzzer/cache/` or
`smart-fuzzer/runs/`, both ignored by default.

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

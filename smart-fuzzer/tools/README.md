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

## Build-ScenarioSeedExecutableCases.ps1

Builds a W089 executable case set from existing function-lane scenario manifests
by extracting literal-argument calls for non-blocked, non-known-deviation
surfaces. It is intentionally conservative: rows that need references, nested
formula evaluation, providers, formula binding, or other unavailable fixtures
are recorded as skipped rather than forced through the pure value evaluator.

```powershell
powershell -ExecutionPolicy Bypass -File smart-fuzzer\tools\Build-ScenarioSeedExecutableCases.ps1
```

Default output:

```text
smart-fuzzer/cache/scenario-seed-executable-cases-v0.json
```

The output can be executed by the generic case-set path in
`Run-ArraySupportTranche.ps1`:

```powershell
powershell -ExecutionPolicy Bypass -File smart-fuzzer\tools\Run-ArraySupportTranche.ps1 `
  -CaseSetPath smart-fuzzer\cache\scenario-seed-executable-cases-v0.json
```

The builder enforces the published `arity_min` / `arity_max` metadata from the
dimension inventory. Manifest calls outside those bounds are not part of the
default pure OxFunc comparison universe; keep them for a dedicated OxFml
admission-negative lane rather than sending them to the Excel comparison
runner.

## Build-AxisWitnessCaseSet.ps1

Builds a W089 axis-witness case set. Each runnable axis is represented by a
control/variant pair where Excel should observe a different result after one
invocation-space dimension changes; each individual call is then compared
between direct OxFunc value evaluation and Excel `Formula2`.

Here `runnable` means runnable in the current OxFunc-accessible comparison
region: direct OxFunc value calls, simple typed fixtures, and a matching Excel
`Formula2` evaluation. The generated case set intentionally does not force
broader DNA Calc axes through this runner when they need OxFml, XLL, provider,
locale, workbook, spill-neighborhood, structured-reference, or rich-value
harnesses.

```powershell
powershell -ExecutionPolicy Bypass -File smart-fuzzer\tools\Build-AxisWitnessCaseSet.ps1
```

Default output:

```text
smart-fuzzer/cache/axis-witness-case-set-v0.json
```

Execute it through the generic case-set path:

```powershell
powershell -ExecutionPolicy Bypass -File smart-fuzzer\tools\Run-ArraySupportTranche.ps1 `
  -CaseSetPath smart-fuzzer\cache\axis-witness-case-set-v0.json
```

The case-set metadata also records broader DNA Calc axis witnesses that need
separate fixtures or harnesses, such as workbook compatibility, locale,
volatile/statistical comparators, OxFml prepared calls, XLL/provider seams,
cross-sheet and structured references, callable values, and rich-value returns.
Those rows are coverage facts, not OxFunc mismatches.

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

## Run-ArraySupportTranche.ps1

Runs the W090 first executable array-support tranche against OxFunc and Excel.
The runner reads `array-support-first-tranche-v0.json`, materializes the formula
seeds into typed local arguments, captures full Excel spill ranges, compares
typed array/scalar digests with no tolerance, and writes compact telemetry plus
failure packets only for non-pass classifications.

```powershell
powershell -ExecutionPolicy Bypass -File smart-fuzzer\tools\Run-ArraySupportTranche.ps1
```

Default outputs:

```text
smart-fuzzer/runs/<run_id>/cases/cases.jsonl
smart-fuzzer/runs/<run_id>/outcomes/local.jsonl
smart-fuzzer/runs/<run_id>/outcomes/excel.jsonl
smart-fuzzer/runs/<run_id>/comparisons/comparisons.jsonl
smart-fuzzer/runs/<run_id>/failure_packets/*.json
smart-fuzzer/runs/<run_id>/rollup.json
smart-fuzzer/runs/<run_id>/roadmap_trace.md
```

The comparison policy is exact typed equality with bit-exact numeric digests.
Pass-heavy rows stay compact; full packets are intentionally reserved for
failures and harness blockers.

`Run-ArraySupportTranche.ps1` can also run generated successor case sets:

```powershell
powershell -ExecutionPolicy Bypass -File smart-fuzzer\tools\Run-ArraySupportTranche.ps1 `
  -CaseSetPath smart-fuzzer\cache\array-support-successor-executable-tranches-v0.json
```

To execute one generated tranche from the case set:

```powershell
powershell -ExecutionPolicy Bypass -File smart-fuzzer\tools\Run-ArraySupportTranche.ps1 `
  -CaseSetPath smart-fuzzer\cache\array-support-successor-executable-tranches-v0.json `
  -CaseSetTrancheId w090-successor-statistical-functions
```

## Build-ArraySupportExecutableTranches.ps1

Builds executable W090 successor case sets from the array-support candidate
inventory plus existing scenario manifests. It extracts parseable scalar
function-call seeds, turns one scalar argument at a time into a duplicate inline
array, and emits local typed arguments for the generic array tranche evaluator.
Rows without a parseable manifest seed are recorded as skipped telemetry rather
than silently treated as reviewed.

```powershell
powershell -ExecutionPolicy Bypass -File smart-fuzzer\tools\Build-ArraySupportExecutableTranches.ps1
```

Default output:

```text
smart-fuzzer/cache/array-support-successor-executable-tranches-v0.json
```

The output is an execution input only. It does not assert that skipped rows were
reviewed, and it does not replace family-specific replay design where a later
tranche needs references, host context, or richer array values.

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

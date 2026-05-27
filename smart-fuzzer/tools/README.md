# Smart-Fuzzer Tools

Status: `tooling_sandbox`

Tracked tools in this directory are reproducible helpers for the OxFunc smart-fuzzer (W088, W089, W090, W092, W097). Generated outputs go to `smart-fuzzer/cache/` or `smart-fuzzer/runs/`, both ignored by default.

## Shared module: `CellRefBatch.psm1`

All comparator runners import `CellRefBatch.psm1`. It exposes:

1. `Invoke-ExcelCellRefBatch` — drive Excel through cell `Value2` plumbing (the bit-exact path).
2. `Test-FormulaTextIsBitExactSafe` — refuse to run a case whose `formula_text` embeds a numeric literal with more than `15` significant digits (Excel's parser is not correctly-rounded past that point; the value must instead come from a cell fixture).
3. `Get-StandardSeverityClass` — the canonical CHARTER §4.1 severity vocabulary, used by every runner. Emits one of `match | structural_mismatch | numeric_drift_1ulp | numeric_drift_gt1ulp | harness_blocked_local | harness_blocked_excel | generator_invalid`, plus optional sub-tags including `excel_imprecision_witness` (local exact integer, Excel `±1` ULP off — still an OxFunc bug, the tag only records the repair direction).
4. `Get-F64BitsHex`, `ConvertTo-ExcelOutcome`, `Get-UlpDistance` — typed-outcome helpers.

The vocabulary, severity grading, and `excel_imprecision_witness` sub-tag policy are anchored in:

- `CHARTER.md` §4.1 Parity Target And Bug-Severity Grading.
- `smart-fuzzer/planning/SMART_FUZZER_DESIGN.md` §1 Goal / §1.1 Bug-Severity Grading.
- `smart-fuzzer/planning/EXCEL_RUNNER_PLUMBING_NOTE.md` (binding plumbing rule and the literal-text-vs-cell-ref witness).

### Quality-bar tests

Two self-tests verify the module without touching Excel:

```powershell
powershell -ExecutionPolicy Bypass -File smart-fuzzer\tools\Test-CellRefBatchHelpers.ps1
powershell -ExecutionPolicy Bypass -File smart-fuzzer\tools\Test-UnsafeLiteralGuard.ps1
```

The first exercises `Test-FormulaTextIsBitExactSafe` (13 cases — short literals pass, long literals are rejected) and `Get-StandardSeverityClass` (18 cases — exact match, signed-zero collapse, kind drift, error-code drift, logical drift, `1` ULP vs `>1` ULP numeric drift, `excel_imprecision_witness` sub-tag, missing-outcome harness blocks, array same-digest / shape-drift / element-drift). The second verifies the guard fires end-to-end and that `Run-ArraySupportTranche.ps1` actually invokes the safety helpers.

## Excel comparator plumbing rule (binding)

A comparator runner that drives Excel via COM and claims **bit-exact typed equality** must pass numeric inputs to Excel through cell `Range.Value2`, not through formula literal text. Excel's formula parser is not correctly-rounded for long decimal literals, so the literal-text path silently rounds inputs to a neighbouring `f64` and the comparator then sees a kernel "drift" that is entirely the harness's fault.

The shared module's `Test-FormulaTextIsBitExactSafe` enforces this on every case: any literal with more than `15` significant digits causes the case to be flagged `generator_invalid` (structural-mismatch class) before Excel is touched. Long-decimal inputs must instead come from a `cell_fixture` entry whose value is written via `Range.Value2`.

`smart-fuzzer/planning/EXCEL_RUNNER_PLUMBING_NOTE.md` carries the empirical witness and the full historical context.

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

The generic case-set runner for W090 and W092 tranches. Reads a case set, runs the Rust local evaluator (`array_tranche_local_eval`), drives Excel via COM with bit-exact plumbing, compares typed array/scalar digests with no tolerance, and writes compact telemetry plus failure packets only for non-pass classifications.

```powershell
powershell -ExecutionPolicy Bypass -File smart-fuzzer\tools\Run-ArraySupportTranche.ps1 `
  -RunId my-run -CaseSetPath smart-fuzzer\cache\axis-witness-case-set-v0.json `
  -CaseSetTrancheId w089-axis-witness-sweep-v0
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

Each comparison row carries both fields:

1. `classification` — the operational label (`exact_typed_bit_match`, `known_residual`, `adapter_or_seam_mismatch`, `unexpected_mismatch`, …). This is the legacy run-level overlay.
2. `severity_class` — the CHARTER §4.1 underlying bug severity (`match | structural_mismatch | numeric_drift_1ulp | numeric_drift_gt1ulp | harness_blocked_local | harness_blocked_excel | generator_invalid`). Independent of run-level overlays — a `known_residual` row still surfaces as a numeric drift bug in `severity_class`.

The rollup aggregates both:

```json
{
  "by_classification": { "exact_typed_bit_match": 34, "known_residual": 24 },
  "by_severity_class":  { "match": 34, "numeric_drift_gt1ulp": 21, "numeric_drift_1ulp": 3 },
  "by_severity_sub_tag": { }
}
```

The comparison policy is exact typed equality with bit-exact numeric digests. Pass-heavy rows stay compact; full packets are intentionally reserved for failures and harness blockers.

Plumbing safety: the runner refuses to evaluate a case whose `formula_text` contains a numeric literal with more than `15` significant digits — those cases are marked `generator_invalid` (structural class) before Excel is touched. Long-decimal inputs must come from `cell_fixture` entries which write via `Range.Value2`.

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

## Run-BroadScalarExploration.ps1

Runs the broad single-arg/two-arg numeric scalar smart-fuzzer cycle. The
local Rust explorer
`smart-fuzzer/tools/pmt_ppmt_local_eval/src/bin/broad_scalar_explorer.rs`
walks `~50` math, transcendental, gamma/erf, hyperbolic, power, mod, round,
log, and combinatoric scalar functions across per-family numeric bands.
Subnormal magnitudes and non-finite values are excluded because the Excel
formula literal parser rejects them.

```powershell
& "smart-fuzzer\tools\Run-BroadScalarExploration.ps1" `
  -RunId broad-scalar-cycle-NNN -CaseCount 1500000 -Seed 17 -CandidateLimit 600
```

The PowerShell wrapper batches the selected candidates into a single Excel
COM `Range.Formula2` write, reads `Range.Value2` plus `ERROR.TYPE(...)`,
and classifies the comparison as `exact_typed_bit_match`,
`expected_formula_literal_encoding_drift`, `known_residual_numeric_drift`,
`unexpected_mismatch`, `unexpected_kind_drift`, or
`unexpected_error_code_drift`. Failure packets are written for unexpected
classes only.

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

# Category-B Evaluation-Class Probe Plan

Status: `planning_artifact_ready`

Owning workset: `docs/worksets/W104_INVOCATION_TEST_CATEGORY_SPLIT_AND_CONTEXT_SENSITIVE_CATALOG.md`
Owning decision: `docs/decisions/ODR-FN-002-invocation-test-category-split.md`

## Scope

Category 2 of [ODR-FN-002](../../docs/decisions/ODR-FN-002-invocation-test-category-split.md):
context-free invocations OxFunc can drive directly against Excel as simple formulas.
This plan adds the **evaluation-class** axis (kind / coercion / error shape) alongside
the bit-exactness sweeps the smart-fuzzer already runs.

It does **not** cover Category-1 context-sensitive rows — those are published to
`smart-fuzzer/corpus/context_sensitive_catalog/` and are a separate smart-fuzzer lane:
a future runner that drives the downstream OxCalc → OxFml → OxFunc stack as its
evaluation engine. Both categories are smart-fuzzer scope; this plan is the Category-2
half, and the Category-1 downstream-driven runner is a later lane seeded by that catalog.

## Reuse, do not rebuild

The runnable Excel comparison harness already exists and is the execution target for
this lane. No new runner plumbing is introduced:

1. `smart-fuzzer/tools/Run-ArraySupportTranche.ps1` — generic case-set runner: runs the
   Rust local evaluator, drives Excel via COM with bit-exact plumbing, compares typed
   digests with no tolerance, writes telemetry + failure packets + rollup.
2. `smart-fuzzer/tools/CellRefBatch.psm1` — Excel COM driver (`Range.Value2` numeric
   plumbing, `Formula2` write, `ERROR.TYPE` capture) plus `Get-StandardSeverityClass`,
   `Get-F64BitsHex`, `Get-UlpDistance`, `Test-FormulaTextIsBitExactSafe`.
3. `smart-fuzzer/tools/pmt_ppmt_local_eval/` (`array_tranche_local_eval` binary) — the
   OxFunc local side via the public `oxfunc_core` value surface.

The new artifact is a generator, `smart-fuzzer/tools/Build-EvaluationClassProbes.ps1`,
emitting the existing `oxfunc.smart_fuzzer.scenario_seed_case_set.v0` case-set format.

## Evaluation-class axes

Both interests named in ODR-FN-002 are first-class; both are bugs when they diverge.

### A. Coercion (structural — CHARTER §4.1)

1. logical→number: `=1+TRUE`, `=SUM(TRUE,TRUE)`, `=TRUE*3`, `=N(TRUE)` — and the
   array-literal contrast `=SUM({1,TRUE})` where Excel does **not** coerce the inline
   logical.
2. text→number: `="2"+3`, `=SUM("2",TRUE)`, `=PRODUCT("2","3")` — and the array
   contrast `=SUM({"2",3})`.
3. blank handling: `=A1+1` with A1 blank, `=SUM(A1,2)` with A1 blank.
4. logical/text in comparison and unary context: `=--TRUE`, `=-"2"`.

The point is the *coercion policy*, which differs by argument surface (direct scalar
vs inline array element vs range cell). The smart-fuzzer's `arg_preparation_profile`
already distinguishes these surfaces; this lane makes the coercion outcome the
comparison target.

### B. Error shape and code (structural — CHARTER §4.1)

1. error generation: `=1/0` (#DIV/0!), `=SQRT(-1)` (#NUM!), `=ABS("x")` (#VALUE!),
   `=NA()` (#N/A), `=CHOOSE(0,1)` (#VALUE!).
2. error propagation and precedence: `=#N/A+1`, `=SUM(1/0,2)`, `=IFERROR(1/0,99)`,
   leftmost-error precedence in multi-error expressions.
3. error round-trip via `ERROR.TYPE` (already captured by the runner) so the *code*,
   not just the error-ness, is compared.

### C. Kind and shape

1. returned-value kind: `=IF(TRUE,1,"a")` vs `=IF(FALSE,1,"a")`, `=T(1)`, `=N("x")`.
2. scalar→array lift of scalar functions: `=ABS({-1,-2})`, `=SQRT({4,9})`.
3. 1x1 array vs scalar publication (the `BUG-FUNC-026` / `HO-FN-010` shape seam).

## Comparison and promotion

The runner already emits `severity_class` from `Get-StandardSeverityClass`:
`match | structural_mismatch | numeric_drift_1ulp | numeric_drift_gt1ulp |
harness_blocked_local | harness_blocked_excel | generator_invalid`.

For this lane:

1. A kind / coercion / error-code divergence is `structural_mismatch` — top priority,
   promoted as a structural `BUG-FUNC-*` on discovery.
2. A numeric value divergence on a probe that should be exact is `numeric_drift_*` —
   promoted with a bit witness, repair direction toward Excel (`excel_imprecision_witness`
   sub-tag where OxFunc is analytic-exact and Excel is `±1` ULP off).
3. Pass rows stay compact telemetry, not individually durable evidence.

## Plumbing safety

All seeded probes use short, bit-exact-safe literals (CHARTER §4.1 plumbing rule). Any
numeric input needing more than 15 significant digits must come from a `cell_fixture`
written via `Range.Value2`, never from formula literal text — the generator and runner
both enforce this via `Test-FormulaTextIsBitExactSafe`.

## Next bead

Execute `Build-EvaluationClassProbes.ps1` output through `Run-ArraySupportTranche.ps1`
on an Excel-capable host; triage mismatches per the promotion rules above. Execution
requires Excel automation and produces run telemetry under `smart-fuzzer/runs/`, which
is gitignored.

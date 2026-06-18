# W104 Invocation Test Category Split And Context-Sensitive Catalog

Status: `in_progress`

## Purpose

Operationalize [ODR-FN-002](../decisions/ODR-FN-002-invocation-test-category-split.md):
split every in-scope invocation under test into Category 1 (context-sensitive →
publish a catalog, evaluate downstream OxCalc→OxFml→OxFunc, do not fake context
locally) and Category 2 (context-free → evaluate locally via the smart-fuzzer for
both evaluation-class behavior and bit-exactness).

Both categories are inside the smart-fuzzer's testing scope; they differ only in
runner. Category 2 runs today on the existing local-Rust + Excel-COM harness. Category 1
will run on a future smart-fuzzer runner that drives the downstream OxCalc→OxFml→OxFunc
stack as its evaluation engine — that runner is a later lane, and the catalog is its seed
corpus. AFL-style expansion applies to both categories.

`.beads/` owns live readiness and blockers.

## Canonical Surfaces

1. `docs/decisions/ODR-FN-002-invocation-test-category-split.md` (the decision)
2. `.beads/` epic for W104 and its task lanes
3. `smart-fuzzer/corpus/context_sensitive_catalog/README.md` (Category-1 catalog index)
4. `smart-fuzzer/corpus/context_sensitive_catalog/catalog-v0.json` (Category-1 catalog data)
5. `smart-fuzzer/planning/CATEGORY_B_EVALUATION_CLASS_PROBE_PLAN.md` (Category-2 lane)
6. `smart-fuzzer/tools/Build-EvaluationClassProbes.ps1` (Category-2 generator)
7. `smart-fuzzer/README.md` "Current Scope Boundary" (pre-existing boundary this promotes)
8. `smart-fuzzer/planning/BLOCKED_DEFERRED_SEAM_CLASSIFICATION_MAP.md` (Category-1 source lanes)

## Current Checkpoint

2026-06-18:

1. ODR-FN-002 written and indexed in `docs/decisions/README.md`.
2. Category-1 catalog established under `smart-fuzzer/corpus/context_sensitive_catalog/`
   and seeded with the explicit-`@` implicit-intersection rows (the live `HO-FN-018`
   consumer), `INDEX`/`OFFSET` reference form, `INDIRECT`/`CELL`/`OFFSET` host-context
   rows, structured/cross-sheet/spill-anchor references, and a formula-binding example.
   These are published for downstream evaluation and are not evaluated in OxFunc.
3. Category-2 runnable Excel comparison scaffolding confirmed already present:
   `Run-ArraySupportTranche.ps1` (generic case-set runner) + `CellRefBatch.psm1`
   (Excel COM, `Range.Value2` bit-exact plumbing) + `pmt_ppmt_local_eval`. No new
   runner plumbing is required; the increment is a new generator.
4. Category-2 evaluation-class probe lane planned and seeded by
   `Build-EvaluationClassProbes.ps1`, which emits the existing
   `oxfunc.smart_fuzzer.scenario_seed_case_set.v0` format consumed by the runner.
5. First local run of the Category-2 seed (OxFunc side only, via
   `array_tranche_local_eval`) executed all probes `ok` and immediately surfaced one
   classification finding: `VALUE("2")` returns `#VALUE!` locally because `value_fn.rs`
   parses text through a `LocaleFormatContext` the local harness does not supply (W082
   locale seam). `VALUE` is therefore locale-context-sensitive — Category 1, not 2 — and
   was moved to the catalog (`CSC-0015`, seam `locale_context`) and removed from the
   Category-2 seed so it does not read as a false Excel mismatch. This is the
   classifier working as intended on its first pass.

## Validation Evidence

1. `smart-fuzzer/corpus/context_sensitive_catalog/catalog-v0.json` parses as JSON
   (15 entries across 8 seam classes).
2. `Build-EvaluationClassProbes.ps1` runs and emits a valid case set with the same
   schema string and field shape as `Build-TypedArgProbes.ps1` (25 cases, 14 surfaces).
3. The Category-2 seed was run through the OxFunc local side
   (`cargo run --bin array_tranche_local_eval -- --cases ... --out ...`): 25/25
   `execution_status: ok`, outcome kinds number=12, error=7, logical=2, text=2,
   array=2 — covering kind, coercion, error-shape, and array-lift axes.
4. Local-vs-Excel comparison of the Category-2 set is a follow-up bead; the Excel oracle
   half requires a host with Excel automation and is recorded as run telemetry under
   `smart-fuzzer/runs/` (gitignored), not as part of this checkpoint.

## Open Lanes

1. Category-2: execute `Build-EvaluationClassProbes.ps1` output through
   `Run-ArraySupportTranche.ps1` on an Excel-capable host and triage any
   structural (kind/coercion/error-shape) or numeric-drift mismatch through
   `docs/bugs/`.
2. Category-1: keep the catalog growing from the deferred seam map; hand the catalog
   to the OxCalc→OxFml→OxFunc downstream path as the evaluation owner.
3. Category-1: reconcile the catalog with the existing
   `crates/oxfunc_core/tests/fixtures/w050_oxfunc_deferred_fixture_register.json` so the
   two do not drift into competing registers.

## Doctrine Axes

scope_completeness: `scope_partial`
target_completeness: `target_partial`
integration_completeness: `partial`
open_lanes: `[cat2_probe_execution, cat1_catalog_growth, cat1_downstream_handoff, cat1_deferred_register_reconciliation]`

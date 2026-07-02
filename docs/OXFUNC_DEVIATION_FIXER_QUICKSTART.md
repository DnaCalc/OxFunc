# OxFunc Deviation Fixer Quick-Start

Status: `operational_reference`
Last updated: `2026-07-02`

One page for taking on an open OxFunc↔Excel deviation as fast as possible: where
the worklist lives, how to reproduce a row, how the two test lanes work, the exact
commands, and what counts as sign-off. This is a convenience index — the canonical
authorities are the catalog, `CHARTER.md` §4.1, and the bug streams it points to.

## The parity target (one sentence)

Every OxFunc-vs-Excel discrepancy is a bug; the repair direction is to match
Excel **bit-for-bit**, including where Excel itself is imprecise (a ±1 ULP
"OxFunc-exact, Excel-off" row is still a bug — you move toward Excel, not toward
analytic correctness). Never accept a divergence. See `CHARTER.md` §4.1.

## Where the worklist lives

- **[OXFUNC_EXCEL_DISCREPANCY_CATALOG.md](OXFUNC_EXCEL_DISCREPANCY_CATALOG.md)** —
  the single open worklist for **Category 2** (context-free, locally-evaluable)
  discrepancies. Each row carries function(s), severity, maturity, and its
  `BUG-FUNC-###` / `oxf-*` id. Start here.
- **Category 1** (context-sensitive: locale, workbook mode, OxFml prepared calls,
  XLL, providers, cross-sheet refs, spill) is **not** in the catalog — it is a seed
  corpus under `smart-fuzzer/corpus/context_sensitive_catalog/`, awaiting the
  downstream OxCalc→OxFml→OxFunc runner ([ODR-FN-002](decisions/ODR-FN-002-invocation-test-category-split.md), W104).
- **"OxFunc-exact but Excel-wrong"** cases live in
  [EXCEL_MATH_DEVIATION_CATALOG.md](EXCEL_MATH_DEVIATION_CATALOG.md), not the
  discrepancy catalog — but they are still tracked as bugs whose repair is to match
  Excel.

## Severity & maturity vocabulary

Severity: `STR` structural (wrong kind/error/shape/array) · `NUM-L` >~2 ULP ·
`NUM-S` ≤~2 ULP. Structural is top priority, fix on discovery; numeric drift is
continuous-triage (magnitude sets priority, but 1 ULP is still a bug).

Maturity: `M0` noted → `M1` tested (minimized reproducer) → `M2` repair-tried →
`M3` fixed-unsigned (locally green, awaiting live-Excel sign-off) → `HO` downstream.

The machine vocabulary emitted by the comparator is
`match | structural_mismatch | numeric_drift_1ulp | numeric_drift_gt1ulp |
harness_blocked_local | harness_blocked_excel | generator_invalid`, plus the
sub-tag `excel_imprecision_witness`. It lives in `CHARTER.md` §4.1 and is
implemented in `smart-fuzzer/tools/CellRefBatch.psm1` (`Get-StandardSeverityClass`).

## The two test lanes

### Lane A — deterministic in-crate conformance (`cargo test`, no Excel)

Excel-expected values are **pinned as raw IEEE-754 bit-patterns** inline in each
kernel's module (e.g. `assert_eq!(acoth_kernel(1_000_000.0).unwrap().to_bits(),
0x3eb0_c6f7_a0b5_f3b3)` in `crates/oxfunc_core/src/functions/acoth.rs`). Bit-equality
against a pinned witness = bit-exact Excel parity for that input. This lane also
guards structure and algebra, not just numbers:

- `function_spec!` macro + `FunctionMeta` declaration — `crates/oxfunc_core/src/function.rs`
- Golden meta snapshot — `crates/oxfunc_core/tests/function_meta_golden.rs`
  vs `crates/oxfunc_core/tests/fixtures/function_meta_golden.txt`
- Catalog/registry conformance (arity ↔ signature ↔ help) — `crates/oxfunc_core/tests/catalog_conformance.rs`
- Cross-surface algebraic laws (kernel/scalar/array-lift agree) — `crates/oxfunc_core/tests/unary_numeric_equivalence_law.rs`

```bash
cargo test -p oxfunc_core                       # everything (unit + inline witnesses + integration)
cargo test -p oxfunc_core --lib                 # in-crate unit + inline kernel bit-witnesses only
cargo test -p oxfunc_core --test catalog_conformance
cargo test -p oxfunc_core --test function_meta_golden
cargo test -p oxfunc_core --test unary_numeric_equivalence_law
cargo test -p oxfunc_core <name-substring> -- --nocapture   # filter to one function's tests
```

If you change a `FunctionMeta` field, the golden snapshot
(`function_meta_golden.txt`) must be updated in the same commit or the equality
test fails.

### Lane B — smart-fuzzer differential harness vs **live** Excel (Windows + Excel, COM)

This is where new deviations and ULP magnitudes are actually measured against the
oracle. Pipeline: generate typed cases → cheap local `oxfunc_core` eval → rank →
batch to a live `Excel.Application` via COM → typed bit-exact compare → minimize →
promote to `docs/bugs/` then a catalog row.

Requires Windows with Excel installed; runs are fully scripted/headless (no UI).

```powershell
# No Excel needed — module self-tests + regenerate the per-surface status map:
powershell -ExecutionPolicy Bypass -File smart-fuzzer\tools\Test-CellRefBatchHelpers.ps1
powershell -ExecutionPolicy Bypass -File smart-fuzzer\tools\Test-UnsafeLiteralGuard.ps1
powershell -ExecutionPolicy Bypass -File smart-fuzzer\tools\Build-FunctionStatusMap.ps1

# Live Excel comparison (generic case-set runner, cell-ref plumbing):
powershell -ExecutionPolicy Bypass -File smart-fuzzer\tools\Run-ArraySupportTranche.ps1 `
  -RunId my-repro-run -CaseSetPath smart-fuzzer\cache\axis-witness-case-set-v0.json
```

Outputs land in `smart-fuzzer/runs/<run_id>/` (gitignored): `comparisons.jsonl`,
`failure_packets/`, `rollup.json`, and `manifest.json` (records Excel
version/channel, git revision, and the input-plumbing mode).

### The binding plumbing rule (read before trusting any magnitude)

A comparator that claims bit-exact equality **must** pass numeric inputs to Excel
through cell `Range.Value2`, **never** through formula literal text. Excel's parser
is not correctly-rounded past ~15 digits, so a long literal silently evaluates on a
neighbouring f64 and the "drift" you see is the harness's fault, not the kernel's
(`=ABS(-140920.05717469757655635)` → `…01ee` via literal vs `…0202` via cell-ref,
~20 ULP of pure harness error). Full rule + witness:
`smart-fuzzer/planning/EXCEL_RUNNER_PLUMBING_NOTE.md`.

Consequence: `Run-BroadScalarExploration.ps1`, `Run-ExpandedFinanceExploration.ps1`,
and `Run-PmtPpmtPilot.ps1` still use literal-text plumbing and **cannot close an
exactness stream**. Several older magnitudes (BUG-FUNC-015/-021/-024/-025) predate
the cell-ref resweep — re-measure through a cell-ref run before treating a number
as the regression floor. See `smart-fuzzer/planning/KNOWN_MISMATCH_RESWEEP_PLAN.md`.

## Fixer path for one deviation

1. **Find the row** in [OXFUNC_EXCEL_DISCREPANCY_CATALOG.md](OXFUNC_EXCEL_DISCREPANCY_CATALOG.md);
   note the `BUG-FUNC-###` and `oxf-*` id.
2. **Read the stream** `docs/bugs/streams/BUG-FUNC-###_*.md` (repro, root cause,
   validation commands, evidence) and `br show oxf-xxxx`.
3. **Reproduce locally** against the kernel in
   `crates/oxfunc_core/src/functions/…` and its inline bit-witness; iterate with
   `cargo test -p oxfunc_core <name> -- --nocapture`.
4. **Confirm current status** via `smart-fuzzer/planning/FUNCTION_STATUS_MAP.md`.
   If the magnitude is pre-resweep, re-measure through a cell-ref Excel run first.
5. **Bit-exact match against live Excel = sign-off.** Update/close the
   `BUG-FUNC-###` stream and `oxf-*` bead, and **remove the row from the catalog**
   (fixed items do not stay in the tracker). Record any transferable insight in
   [OXFUNC_FIX_LEARNING_LOG.md](OXFUNC_FIX_LEARNING_LOG.md).

## File map

| What | Where |
|---|---|
| Open Category-2 worklist | `docs/OXFUNC_EXCEL_DISCREPANCY_CATALOG.md` |
| Bug streams (root cause, evidence, validation) | `docs/bugs/streams/BUG-FUNC-*.md` |
| Bug reports (intake) | `docs/bugs/reports/BUGREP-FUNC-*.md`, `docs/bugs/BUG_*_REGISTER.csv` |
| Live task/bead state | `.beads/` via `br` (`br show oxf-xxxx`, `br list --status open`) |
| Kernels + inline bit-witnesses | `crates/oxfunc_core/src/functions/*.rs` |
| In-crate conformance tests | `crates/oxfunc_core/tests/*.rs` |
| Per-surface status map | `smart-fuzzer/planning/FUNCTION_STATUS_MAP.md` (rebuild: `Build-FunctionStatusMap.ps1`) |
| Excel comparator module | `smart-fuzzer/tools/CellRefBatch.psm1` |
| Comparator runners | `smart-fuzzer/tools/Run-*.ps1` (+ `smart-fuzzer/tools/README.md`) |
| Plumbing rule + resweep plan | `smart-fuzzer/planning/EXCEL_RUNNER_PLUMBING_NOTE.md`, `KNOWN_MISMATCH_RESWEEP_PLAN.md` |
| Severity / category policy | `CHARTER.md` §4.1, `docs/decisions/ODR-FN-002-invocation-test-category-split.md` |

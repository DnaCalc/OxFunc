# W097 R-B — Shared `CellRefBatch.psm1` paired before/after

Status: `tranche_complete`

Owning workset: `docs/worksets/W097_BIT_EXACT_RESWEEP_OF_KNOWN_MISMATCHES.md`
Owning bead: `oxf-ic1h.2`

## 1. What this record is

Bead `oxf-ic1h.2` requires that the cell-ref helper be lifted out of
`Run-BroadScalarExploration.ps1` into a shared module
`smart-fuzzer/tools/CellRefBatch.psm1`, and that at least one
comparator runner consumes the shared module without behavior change vs
the inline helper, recorded as a paired before/after run.

The shared module landed at `smart-fuzzer/tools/CellRefBatch.psm1` and
exports:

- `Invoke-ExcelCellRefBatch -Candidates <object[]>`
- `Get-F64BitsHex -Value <double>`
- `ConvertTo-ExcelOutcome -Value <object> -ErrorType <object>`
- `Get-UlpDistance -A <double> -B <double>`

Candidate shape: `@{ function_name; args; expected? }`. Each `args`
entry is either a scalar `double` or a matrix block `@{ kind="matrix";
rows; cols; values }`. Matrix-bearing candidates may pin which
element of an array result becomes the bit-exact comparison target via
`@{ result_index = @(r, c) }` (1-based, default `@(1, 1)`); the
formula is wrapped in `INDEX(<call>, r, c)` so array-returning
functions like `MINVERSE` do not spill into the error-type companion
cell.

`Run-BroadScalarExploration.ps1` was refactored to import the module
and replace its inline helper. No semantic changes.

## 2. Paired before/after runs

Reference (inline helper, pre-refactor):

- `smart-fuzzer/runs/broad-scalar-cycle-010-cellref/`
  - `1,000,000` cases, seed `17`, `600` Excel-sampled
  - matches `468`, unexpected `132`, blocked `0`

Validation (shared module, post-refactor, same seed and case count):

- `smart-fuzzer/runs/W097-R-B-validation-cellref-module/`
  - `1,000,000` cases, seed `17`, `600` Excel-sampled
  - matches `468`, unexpected `132`, blocked `0`

Per-row `excel_outcome.bits_hex` (or `digest_payload` for non-number
rows) was compared across all `600` rows: **`0` diffs**.

## 3. Doctrine

The module is now the canonical cell-ref Excel comparator helper.
`Run-PmtPpmtPilot.ps1`, `Run-ExpandedFinanceExploration.ps1`,
`Run-ArraySupportTranche.ps1` and any new comparator runners should
consume `CellRefBatch.psm1` rather than carrying their own inline
helper. Tranches R-C through R-H build directly on top of this module.

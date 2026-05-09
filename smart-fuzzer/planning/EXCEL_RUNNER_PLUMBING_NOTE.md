# Excel Runner Plumbing Note

Status: `plumbing_doctrine`
Owning workset: `W092`

This note records a comparator-side plumbing rule that affects every script
in this repo that compares OxFunc against live Excel through COM. It is
binding for new runners; existing runners should adopt it on their next
non-trivial revision.

## The Rule

A comparator runner that wants to claim **bit-exact typed comparison**
between OxFunc and Excel **must not** pass numeric inputs through formula
literal text. It must write each numeric input to a worksheet cell as a
raw IEEE-754 double via `Range.Value2 = <double>` (or a 2D `object[,]`
batch of doubles) and reference that cell from the formula.

## Why

Excel's formula parser converts a decimal text literal into a `f64`. The
parser is correctly-rounded for short literals but is not always
correctly-rounded for the longer literals a fuzzer-generated `{value:.17}`
or `{value:.17E}` produces, especially when the integer part already
consumes much of the f64 significand budget. The result is that
`=COS(literal_text_for_v)` may compute `COS(v')` for some `v' ≠ v` that is
one or more ULPs away from `v`. The comparator then sees the local
OxFunc result for `v` and Excel's result for `v'`, and reports a numeric
drift that is entirely caused by the comparator harness, not by an
algorithmic difference in either side.

Empirical confirmation lives in
`smart-fuzzer/runs/broad-scalar-cycle-003`. With the literal-text path,
`~214` of `600` Excel-sampled rows landed in the
`expected_formula_literal_encoding_drift` bucket (relative tolerance
`1e-12 * scale`). A focused re-probe on one of those witnesses
(`=ABS(-140920.05717469757655635)`) showed:

1. `Range.Value2 = v` then `=ABS(B1)` → `0x410133c075180202` (matches
   OxFunc-local exactly).
2. `=ABS(-140920.05717469757655635)` literal → `0x410133c0751801ee`
   (differs by `~20` ULPs because Excel parsed a neighbouring f64 from
   the text).

The cell-ref path is bit-exact for every double tested, including
`0.1`, `0.30000000000000004`, max normal (`1.79e308`), and the smallest
positive normal (`2.2250738585072014e-308`).

## Implications

1. The `expected_formula_literal_encoding_drift` classification is a
   harness-induced artefact. Once cell-ref plumbing is in place, the
   class disappears, and the rows that used to absorb into it become
   either `exact_typed_bit_match` or `unexpected_mismatch` worth
   investigating.
2. Some rows currently classified as encoding drift are actually
   genuine sub-ULP imprecision in Excel kernels (the witness
   `=COMBIN(23,10) → 1144066.0000000002` is one such row — the inputs
   `23` and `10` have no encoding ambiguity). These should be reclassified.
3. Some rows currently in `BUG-FUNC-021`, `BUG-FUNC-014`,
   `BUG-FUNC-015`, `BUG-FUNC-024`, and `BUG-FUNC-025` may shrink (the
   measured drift was partly an encoding artefact) or grow (the measured
   drift was understated because the encoding path round-tripped to a
   value where the kernel happens to be exact). The numeric magnitudes
   on those streams need to be re-measured under exact-input plumbing
   before the streams are closed.

## Inventory: comparator runners

The runners below all currently drive Excel via COM in this repo. The
"plumbing" column records whether each one passes numeric arguments
through formula literal text or through cell `Value2`. The "follow-up"
column records whether the script needs a refactor to use cell-ref
plumbing.

### Active smart-fuzzer comparators

| Runner | Plumbing | Follow-up |
| --- | --- | --- |
| `smart-fuzzer/tools/Run-BroadScalarExploration.ps1` | literal-text | refactor (cycle generator for BUG-FUNC-027) |
| `smart-fuzzer/tools/Run-ExpandedFinanceExploration.ps1` | literal-text | refactor (PMT/PPMT/IPMT exactness, BUG-FUNC-015) |
| `smart-fuzzer/tools/Run-PmtPpmtPilot.ps1` | literal-text | refactor (PMT/PPMT pilot, BUG-FUNC-015) |
| `smart-fuzzer/tools/Run-ArraySupportTranche.ps1` | mixed: cell `Value2` for fixtures, formula text contains numeric args otherwise | partial refactor for inline-array literals with non-integer cells (BUG-FUNC-021, BUG-FUNC-024, BUG-FUNC-025) |

### Active smart-fuzzer non-comparator scripts

| Runner | Purpose | Follow-up |
| --- | --- | --- |
| `smart-fuzzer/tools/Run-ExcelThroughputBenchmark.ps1` | throughput, not comparison | none — fixed shapes, encoding-drift not relevant |
| `smart-fuzzer/tools/Build-*.ps1` | builders, not runners | none — builders emit `cell_fixture` entries which already encourage cell-ref binding downstream |

### Workset probe scripts

The `tools/w*-probe/` and `tools/*-probe/` directories together hold
~68 PowerShell scripts that drove historical workset baselines. They
fall into three subclasses:

1. Some already use cell-ref binding (e.g.
   `tools/w37-probe/run-w37-xirr-large-root-baseline.ps1` writes flow
   amounts through `Range.Value2`), and their evidence is reliable.
2. Most use literal-text formulas with hand-crafted, short numeric
   values where 17-digit print is round-trip-safe; their evidence is
   approximately reliable but should not be the basis for **declaring a
   bit-exact regression floor**.
3. A few use long literal numerics in workset evidence rows
   (`tools/w24-probe/run-w24-batch11-financial-time-value-baseline.ps1`
   et al.). Those rows are at risk; if a workset's witness floor
   depends on bit-exact equality there, the witness should be
   re-replayed under cell-ref plumbing.

These probes are mostly frozen historical baselines. They do **not** need
a routine refactor; instead, they need to be considered "not bit-exact
authoritative" unless the evidence has been replayed through cell-ref
plumbing or the values involved are short literals.

## Adoption Plan

1. Land the cell-ref plumbing in `Run-BroadScalarExploration.ps1` first,
   replay one cycle, confirm the encoding-drift bucket collapses.
2. Lift the cell-ref helper into a small shared PowerShell module so the
   other comparator runners (`Run-PmtPpmtPilot.ps1`,
   `Run-ExpandedFinanceExploration.ps1`,
   `Run-ArraySupportTranche.ps1`) can adopt it without copy-paste.
3. Re-run `BUG-FUNC-021`, `BUG-FUNC-014`, `BUG-FUNC-015`,
   `BUG-FUNC-024`, `BUG-FUNC-025`, and the closed-but-historically-bit-exact
   streams (`BUG-FUNC-005`, `BUG-FUNC-013`) under the new plumbing to
   re-measure each residual exactness gap. See
   `KNOWN_MISMATCH_RESWEEP_PLAN.md`.
4. New comparator runners and worksets must default to the cell-ref
   plumbing.

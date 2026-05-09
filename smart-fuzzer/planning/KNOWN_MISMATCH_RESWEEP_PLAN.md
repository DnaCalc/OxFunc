# Known-Mismatch Re-Sweep Plan

Status: `plan_draft`

Owning workset proposal: `W097_BIT_EXACT_RESWEEP_OF_KNOWN_MISMATCHES`
Owning bug stream: this plan supports the re-replay rows demanded by
`BUG-FUNC-027` Section "2026-05-09 Plumbing Caveat", and feeds back into
`BUG-FUNC-014`, `BUG-FUNC-015`, `BUG-FUNC-021`, `BUG-FUNC-024`, and
`BUG-FUNC-025`.

## 1. Motivation

The 2026-05-09 cell-ref plumbing finding (see
`smart-fuzzer/planning/EXCEL_RUNNER_PLUMBING_NOTE.md`) showed that
OxFunc-vs-Excel comparisons run under formula-literal-text plumbing
contain a hidden harness artefact: Excel's formula parser maps a
17-digit decimal literal to a neighbouring `f64`, which can introduce up
to a few ULPs of input-side drift before either side's kernel runs. The
existing `expected_formula_literal_encoding_drift` triage class was
absorbing that artefact under a `1e-12 * scale` tolerance.

A direct re-replay of cycle-003's seed under cell-ref plumbing
(`broad-scalar-cycle-010-cellref`) showed:

1. about `60%` of the encoding-drift rows resolve to bit-exact match —
   confirming they were harness artefacts.
2. about `40%` of the encoding-drift rows resolve to genuine
   `unexpected_mismatch` — newly-surfaced kernel drifts the tolerance
   was hiding.
3. integer-valued kernels in OxFunc (`COMBIN`, `PERMUT`, `PERMUTATIONA`,
   `FACTDOUBLE`) produced bit-exact integers, while Excel was 1 ULP off
   in the same row — i.e. OxFunc is *more* accurate than Excel for some
   currently-open BUG-FUNC bands.

The implication for closed-and-handed-off BUG-FUNC streams: their
"closed" status was given when the comparison harness was less sharp.
Some of those streams may have been over-spec'd for repair, others
under-spec'd. Both directions are interesting.

## 2. Surfaces To Re-Sweep

The table below lists every BUG-FUNC stream whose closure or repair
direction depended on a numeric exactness comparison against Excel.
Each row gets a planned re-replay tranche under cell-ref plumbing.

| Bug stream | Status today | Surface to re-replay | Plumbing risk |
| --- | --- | --- | --- |
| `BUG-FUNC-005` POWER zero-to-zero | `closed` | `POWER(0, 0)` and adjacency; closed via empirical Excel pin | low — values are exact integers/zeros |
| `BUG-FUNC-013` Normal distribution accuracy | `closed` | `NORM.DIST`, `NORM.INV`, `NORMSDIST`, `NORMSINV` over published witness rows | medium — fractional probability/quantile inputs |
| `BUG-FUNC-014` XIRR solver-precision | `closed` | XIRR cashflow vectors and rate roots | medium — flow-amount cell inputs already use Value2; rate-bind sensitivity remains |
| `BUG-FUNC-015` PMT/PPMT/IPMT exactness | `validated_local` | non-zero-rate annuity rows | medium — rate / nper / pv have wide bands |
| `BUG-FUNC-021` W090 statistical exactness | `open` | `BETADIST/BETAINV`, `CHIDIST/CHIINV`, `FDIST/FINV`, `GAMMADIST/GAMMAINV`, `HYPGEOMDIST`, `NEGBINOMDIST`, `NORMSDIST/NORMSINV`, `TDIST/TINV`, `PERCENTRANK`, `CONFIDENCE.T`, `Z.TEST`, `KURT`, `SKEW`, `SKEW.P` | high — fractional shape parameters and tail probabilities |
| `BUG-FUNC-023` non-statistical and matrix shape drift | `open` (split) | residual after split into `024`/`025` | low — see split |
| `BUG-FUNC-024` BESSELY exactness | `open` | `BESSELY(2.5, 1)` and adjacency band | medium — fractional argument sensitivity |
| `BUG-FUNC-025` MINVERSE matrix exactness | `open` | `MINVERSE({1,2;3,4})` and adjacent matrices | medium — element-wise inputs through array literal text |
| `BUG-FUNC-027` broad scalar findings | `open` | CLASS-C* subclasses (GAMMA neg-non-int, MOD drift, trig moderate-large, ATANH near-boundary, ACOTH/ACOSH near 1) | high — these were measured under literal-text plumbing |

## 3. Tranche Order

Order picks shortest expected fixture work first, then escalates.

1. **Tranche R-A**: re-replay `BUG-FUNC-027` CLASS-C* under the existing
   broad-scalar runner (already done as cycle-010-cellref; expand to
   cycles 011..015 with multiple seeds for breadth). Output: revised
   ULP magnitudes for each subclass, classification of which subclass
   shrinks vs. grows.
2. **Tranche R-B**: lift the cell-ref helper from
   `Run-BroadScalarExploration.ps1` into a small shared PowerShell
   module (`smart-fuzzer/tools/CellRefBatch.psm1`) so other comparators
   can adopt without copy-paste. Spec the module API so it accepts a
   list of `{ function_name, args[], expected? }` and returns typed
   outcomes.
3. **Tranche R-C**: re-replay `BUG-FUNC-015` (PMT/PPMT/IPMT). Refactor
   `Run-PmtPpmtPilot.ps1` and `Run-ExpandedFinanceExploration.ps1` to
   use the shared helper. Re-run the existing 28-case PMT/PPMT pilot
   surface plus a fresh `~1M`-case finance broad seed; compare ULP
   distribution to the prior `BUG-FUNC-015` evidence.
4. **Tranche R-D**: re-replay `BUG-FUNC-021` statistical distributions.
   Build a dedicated `stat_distribution_explorer.rs` along the same
   shape as `broad_scalar_explorer.rs` but covering the listed family
   with thoughtful per-distribution argument bands (degrees of freedom,
   shape parameters, tail/cumulative flag, x/p domain). Run through
   the cell-ref harness. Output: per-distribution ULP histograms
   replacing the current "approximate" `BUG-FUNC-021` row counts.
5. **Tranche R-E**: re-replay `BUG-FUNC-024` BESSELY. Fixed band of
   `(x, n)` pairs around the witness `(2.5, 1)` with both directions
   sampled.
6. **Tranche R-F**: re-replay `BUG-FUNC-025` MINVERSE. Build a small
   matrix generator that writes matrix elements via `Value2` to a
   block of cells and references them with a range argument like
   `=MINVERSE(C1:D2)`. This is a step beyond the scalar plumbing: array
   inputs through cells. Output: bit-exact ULP measurement of MINVERSE
   over `2x2`, `3x3`, `4x4` random and structured matrices.
7. **Tranche R-G**: re-replay `BUG-FUNC-013` and `BUG-FUNC-014`.
   These are closed; the goal here is to confirm closure under the
   sharper plumbing or surface a regression / under-spec.
8. **Tranche R-H**: re-replay `BUG-FUNC-005` POWER zero-to-zero. Lowest
   risk; validates the closure remains tight under the new plumbing.

## 4. Expected Information Gain

1. **Real ULP magnitudes** for each open exactness class — the
   `BUG-FUNC-021` repair direction in Section 4 of that stream needs
   per-subfamily kernel-by-kernel work, and a fresh ULP histogram is
   the right input.
2. **Possible BUG-FUNC closures** for rows that were "drifting" only
   through the harness; if the cell-ref re-replay shows bit-exact
   match, the row is no longer a kernel issue.
3. **Possible inverse findings**: rows where Excel is the one drifting
   from the analytic answer (the COMBIN witness from cycle-010 is the
   first example). These need their own classification; under the
   no-tolerance comparison policy, "OxFunc is correct, Excel is 1 ULP
   off" is recorded as a `known_excel_imprecision_witness` rather than
   an OxFunc bug.
4. **Better priors for new exploration** — the encoding-drift bucket
   was suppressing real signal. Under cell-ref plumbing, future cycles
   should have a tighter signal-to-noise ratio in the unexpected
   mismatch channel.

## 5. Doctrine Boundaries

This plan is a re-measurement initiative, not a re-fix initiative. No
existing repair landings are reverted. No closed BUG-FUNC stream is
reopened automatically. After each tranche:

1. If the cell-ref re-replay confirms the original repair was correct,
   the stream's "Closure Checklist" gets a row added recording the
   re-replay reference and the revised ULP magnitude.
2. If the re-replay surfaces a regression or a new sub-class, a
   distinct successor `BUG-FUNC-NNN` stream is opened. The original
   stream is not reopened in place unless the regression is direct.
3. If the re-replay shows the original repair over-fitted to a harness
   artefact (less likely but possible), a successor stream is opened
   to record the corrected expectation.

## 6. Workset Proposal

Open `W097_BIT_EXACT_RESWEEP_OF_KNOWN_MISMATCHES` with a single epic
bead and per-tranche children. The epic carries the cross-tranche
status axes; each child closes when its tranche's run record and ULP
histogram are filed under `smart-fuzzer/runs/`.

Dependencies:

1. R-A done (this is partially done by `cycle-010-cellref`).
2. R-B before R-C/R-D/R-E/R-F (the shared helper).
3. R-G and R-H can run in parallel with R-C/R-D once R-B lands.

Sequencing rule: Tranche R-B is the bottleneck because the rest of the
plan depends on the shared helper. Estimated effort for R-B is small
(half-day at most) and produces a reusable artefact.

## 7. Status Axes

- `scope_completeness`: `scope_partial` until every tranche has a
  recorded re-replay run.
- `target_completeness`: `target_partial`.
- `integration_completeness`: `partial`.
- `open_lanes`: tranches R-A through R-H, plus the cell-ref helper
  module landing.

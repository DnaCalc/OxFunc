# W097 Bit-Exact Re-Sweep Of Known Mismatches

Status: `complete`

## Completion Note (2026-05-10)

All seven tranche children of epic `oxf-ic1h` are tranche-complete.
Per-tranche run records and per-stream evidence appendices:

| Tranche | Bead       | Streams covered            | Run record |
| ------- | ---------- | -------------------------- | ---------- |
| `R-A`   | `oxf-ic1h.1` | `BUG-FUNC-027` CLASS-C*    | `smart-fuzzer/planning/W097-R-A-broad-scalar-cell-ref-resweep.md` |
| `R-B`   | `oxf-ic1h.2` | (helper module landing)    | `smart-fuzzer/planning/W097-R-B-cellref-module-paired.md` |
| `R-C`   | `oxf-ic1h.3` | `BUG-FUNC-015`             | `smart-fuzzer/planning/W097-R-C-pmt-ppmt-ipmt-cell-ref-resweep.md` |
| `R-D`   | `oxf-ic1h.4` | `BUG-FUNC-021`             | `smart-fuzzer/planning/W097-R-D-stat-distribution-cell-ref-resweep.md` |
| `R-E`   | `oxf-ic1h.5` | `BUG-FUNC-024`             | `smart-fuzzer/planning/W097-R-E-bessely-cell-ref-resweep.md` |
| `R-F`   | `oxf-ic1h.6` | `BUG-FUNC-025`             | `smart-fuzzer/planning/W097-R-F-minverse-cell-ref-resweep.md` |
| `R-G/H` | `oxf-ic1h.7` | `BUG-FUNC-005`, `-013`, `-014` | `smart-fuzzer/planning/W097-R-GH-closed-streams-cell-ref-resweep.md` |

Helper module landed: `smart-fuzzer/tools/CellRefBatch.psm1`. Comparator
runners refactored to consume it: `Run-BroadScalarExploration.ps1`,
`Run-PmtPpmtPilot.ps1`, `Run-ExpandedFinanceExploration.ps1`. New
runners that consume it directly: `Run-StatDistributionExploration.ps1`,
`Run-BesselyResweep.ps1`, `Run-MinverseResweep.ps1`,
`Run-ClosedStreamResweep.ps1`. New local Rust binaries:
`stat_distribution_explorer`, `matrix_local_eval`.

Refactor of `Run-ArraySupportTranche.ps1` (the formula-literal portion
for non-integer cells) is not landed in this workset — that runner
already mixes cell-`Value2` for fixtures with formula-text for inline
arguments, and the comparator surfaces it drives are now better served
by the per-stream re-sweep runners landed here. A separate workset
should pick up the residual formula-literal cleanup if any of those
surfaces is reopened.

Findings summary across the seven tranches:

1. `BUG-FUNC-005`, `BUG-FUNC-014` closures: confirmed bit-exact
   under cell-ref. No follow-up.
2. `BUG-FUNC-013` closure: direct witnesses confirmed bit-exact.
   Two helper-adjacent rows (`GAUSS(1)`, `PHI(0)`) drift `1-2` ULP
   under the sharper plumbing — recorded as a follow-up candidate
   without reopening this stream in place.
3. `BUG-FUNC-015` (PMT/PPMT/IPMT): pilot bit-for-bit identical to
   literal-text reference. 1M-case finance broad-seed re-replay
   produced per-function ULP histograms that tighten the recorded
   magnitudes; two PPMT high-rate / huge-PV rows escalated from
   "expected drift" to a kind-drift sub-class (PPMT returns `#NUM!`
   where Excel returns a finite denormal).
4. `BUG-FUNC-021` (statistical distributions): per-distribution ULP
   histograms across `20` distributions, `800` Excel-sampled
   candidates. `SKEW.P` and `PERCENTRANK` have no measurable drift
   and can be dropped from scope. New `HYPGEOMDIST` domain sub-class
   (`#NUM!` locally where Excel returns a finite probability) added
   as a follow-up sub-bullet of the existing fix plan.
5. `BUG-FUNC-024` (BESSELY): witness bit-for-bit reproduced; per-`n`
   ULP histogram across an `(x, n)` band confirms the diagnosis
   that the BESSELY drift surface is broad and large
   (`10^6..10^12` ULP across most of the surface), supporting the
   "substrate/algorithm only; do not patch this witness" repair
   direction.
6. `BUG-FUNC-025` (MINVERSE): witness bit-for-bit reproduced; per-cell
   ULP measurements across `45` matrices (2x2 / 3x3 / 4x4) establish
   a per-kind drift floor for future repair regression-validation.
7. `BUG-FUNC-027` (broad scalar CLASS-C*): five fresh-seed cycles
   (`011..015`) plus the reference `010` re-classified the
   subclasses. CLASS-C1 (GAMMA) shrinks ~100x; CLASS-C2 (MOD)
   persists; CLASS-C3 (trig) grows; CLASS-C4 (ATANH) stable;
   CLASS-C5 (ACOTH/ACOSH) broadens. New CLASS-C3.h hyperbolic-
   overflow saturation sub-class surfaced (COTH NaN-vs-saturated).
   New "OxFunc-more-accurate-than-Excel" pattern surfaced for
   integer combinatorics (COMBIN/COMBINA exact integer locally vs
   `±1` ULP in Excel).

## 1. Purpose

Re-replay every known OxFunc-vs-Excel exactness mismatch surface under the
new cell-ref Excel comparator plumbing so that the recorded ULP magnitude
of each open and closed `BUG-FUNC-*` exactness stream reflects bit-exact
input plumbing rather than the legacy formula-literal-text harness.

## 2. Problem Statement

The 2026-05-09 broad-scalar smart-fuzzer cycle uncovered that comparator
runs which pass numeric arguments through formula text introduce a hidden
harness artefact: Excel's formula parser is not always correctly-rounded
for long decimal literals, so a comparator can report a "kernel drift"
that is in fact only an input-side `f64` neighbour mismatch. The full
rule and the empirical witness live in
`smart-fuzzer/planning/EXCEL_RUNNER_PLUMBING_NOTE.md`.

A direct re-replay of one cycle under cell-ref plumbing
(`broad-scalar-cycle-010-cellref`) showed that:

1. about `60%` of the rows formerly classified as `expected_formula_literal_encoding_drift`
   were genuine harness artefact and now resolve to bit-exact match,
2. about `40%` of those rows were genuine kernel drift hidden under the
   loose tolerance and now resolve to `unexpected_mismatch`,
3. some BUG-FUNC bands actually have OxFunc *more* accurate than Excel
   (for example `=COMBIN(23,10)` returns the exact integer in OxFunc and
   a `1 ULP` offset in Excel).

Every existing `BUG-FUNC-*` stream that turned on bit-exact numeric
comparison was characterised under the legacy plumbing; their measured
magnitudes need to be re-measured before any open stream is closed and
before any closed stream is taken as a durable regression floor.

## 3. Scope

In scope:

1. cell-ref helper module landing for shared use across comparator
   runners,
2. cell-ref refactor of `Run-PmtPpmtPilot.ps1`,
   `Run-ExpandedFinanceExploration.ps1`, and the formula-literal portion
   of `Run-ArraySupportTranche.ps1`,
3. tranche-by-tranche re-replay of `BUG-FUNC-005`, `BUG-FUNC-013`,
   `BUG-FUNC-014`, `BUG-FUNC-015`, `BUG-FUNC-021`, `BUG-FUNC-024`,
   `BUG-FUNC-025`, and `BUG-FUNC-027` CLASS-C* subclasses,
4. revised ULP magnitudes recorded in each stream's evidence section,
5. opening of successor streams when the re-replay surfaces new
   subclasses or shows the original repair under-spec'd or over-spec'd,
6. recording any "OxFunc more accurate than Excel" rows as a new
   `known_excel_imprecision_witness` classification under the
   no-tolerance comparison policy.

Out of scope:

1. reverting any landed repair commit,
2. automatic re-opening of closed streams,
3. moving OxFunc kernel work for any subclass into this workset
   (kernel repairs continue to belong to the originating bug stream
   and its repair workset),
4. changing the no-tolerance comparison policy,
5. the literal-vs-cell-ref reconciliation for non-comparator probe
   scripts in `tools/w*-probe/` — those are frozen historical evidence
   and can be marked "not bit-exact authoritative" in their respective
   workset records when needed, without retrofit.

## 4. OxFunc-Local Surface

W097 work lands in the smart-fuzzer infrastructure and in evidence
records of existing bug streams. It does not modify any
`oxfunc_core::functions::*` kernel.

The workset:

1. lifts the cell-ref helper from `Run-BroadScalarExploration.ps1` into
   a shared `smart-fuzzer/tools/CellRefBatch.psm1` module,
2. updates the four affected comparator runners
   (`Run-PmtPpmtPilot.ps1`, `Run-ExpandedFinanceExploration.ps1`,
   `Run-ArraySupportTranche.ps1`, `Run-BroadScalarExploration.ps1`)
   to consume the shared module,
3. adds a per-tranche replay run record under `smart-fuzzer/runs/`,
4. amends each affected `BUG-FUNC-*` stream with a "Cell-ref re-replay"
   evidence section recording the revised ULP histogram,
5. opens any successor `BUG-FUNC-*` stream that the re-replay justifies.

The full tranche order, dependencies, and per-tranche success criteria
live in `smart-fuzzer/planning/KNOWN_MISMATCH_RESWEEP_PLAN.md`.

## 5. Bead Layout

Allocated epic and child beads in `.beads/`:

1. epic `oxf-ic1h`: W097 cell-ref re-sweep of known mismatches
2. child `oxf-ic1h.1` (R-A): BUG-FUNC-027 CLASS-C* re-replay (partially done; independent of R-B)
3. child `oxf-ic1h.2` (R-B): cell-ref helper module landing (`smart-fuzzer/tools/CellRefBatch.psm1`); priority 1 bottleneck for R-C/R-D/R-E/R-F/R-G/R-H
4. child `oxf-ic1h.3` (R-C): PMT/PPMT/IPMT re-replay (covers BUG-FUNC-015); blocks-on R-B
5. child `oxf-ic1h.4` (R-D): statistical distribution re-replay (covers BUG-FUNC-021); blocks-on R-B
6. child `oxf-ic1h.5` (R-E): BESSELY re-replay (covers BUG-FUNC-024); blocks-on R-B
7. child `oxf-ic1h.6` (R-F): MINVERSE matrix re-replay (covers BUG-FUNC-025); blocks-on R-B
8. child `oxf-ic1h.7` (R-G/R-H): closed-stream re-replays (covers BUG-FUNC-013, BUG-FUNC-014, BUG-FUNC-005); blocks-on R-B

## 6. Status Axes

- `scope_completeness`: `scope_complete` for the W097 re-measurement scope.
  Every tranche has a recorded re-replay run and updated stream evidence
  section. `Run-ArraySupportTranche.ps1` partial refactor is deferred outside
  W097 because that runner is no longer the active evidence path for these
  stream surfaces.
- `target_completeness`: `target_complete` for the declared W097
  re-measurement target. Each affected `BUG-FUNC-*` stream has a
  "Cell-Ref Re-Replay" evidence section appended.
- `integration_completeness`: `integrated` for the W097 tooling and evidence
  surface. Kernel repair was explicitly outside W097 scope.
- `open_lanes`: none for the declared W097 re-measurement target. Follow-up
  candidates are filed per the Completion Note finding bullets.

## 7. Doctrine Notes

This is a re-measurement initiative under the
"Anti-Premature-Completion Doctrine" of `AGENTS.md`. No completion claim
is implied for kernel correctness. The successful product is a corrected
characterisation of the existing bug-stream surfaces, not a corrected
kernel.

If a re-replay shows that an open stream's actual ULP magnitudes are
materially different from the recorded ones, the stream's repair
direction may need to change. That re-direction work belongs to the
originating workset (for example `W090` for `BUG-FUNC-021`), not to
W097.

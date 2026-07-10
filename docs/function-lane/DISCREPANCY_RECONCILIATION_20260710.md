# Discrepancy Reconciliation — 2026-07-10

Status: `catalog_reconciled_current_baseline`

## Declared scope

Reconcile the canonical Category-2 discrepancy catalog against the current Rust
tree, existing bug/handoff records, and fresh live-Excel evidence. This is a
tracker-maintenance and verification slice, not a claim that OxFunc has reached
global Excel parity or that W108 has passed its open gate.

Reference environment for the fresh probes:

- Excel application: `16.0`, build `20131`
- workbook Compatibility Version: `2`
- numeric input path: worksheet cell references populated through `Range.Value2`
- equality policy: typed outcome plus exact binary64 bits, with no tolerance

## Reconciliation decisions

### Trigonometric family remains open

The large-input domain rule and the numeric kernel are separate facts. OxFunc and
Excel both return `#NUM!` at `|x| >= 2^27`, but the current Rust implementation
still uses the platform `sin`/`cos`/`tan` kernels and differs from Excel at
argument-dependent inputs below that boundary.

| Witness | OxFunc vs Excel |
|---|---:|
| `COS(49.214601836)` | 1 ULP |
| `COS(149.214601836)` | 1 ULP |
| `TAN(797601.58)` | 719 ULP |
| `SIN(961281.44)` | 49 ULP |
| `SIN(100000)` | 230 ULP |
| `TAN(100000)` | 230 ULP |
| `COT(100000)` | 351 ULP |
| `CSC(100000)` | 351 ULP |
| `SIN(134217727)` | 5,664 ULP |
| `SIN(134217728)` | exact typed `#NUM!` match |

Some sampled family members are exact at the same argument (for example
`COS(100000)` and `SEC(100000)`), so the catalog now describes an
argument-dependent reduction/publication residual rather than implying that
every call differs. Reproducer: `smart-fuzzer/tools/Run-TrigCatalogRecheck.ps1`.

### YIELDDISC stale row retired

`YIELDDISC(44013,44562,95,100,0)` is bit-exact in a fresh three-way replay:

| Engine | Result bits |
|---|---|
| OxFunc | `0x3fa1f7047dc11f70` |
| ExcelFinancialFunctions F# | `0x3fa1f7047dc11f70` |
| Excel | `0x3fa1f7047dc11f70` |

The rate-first formula repair had already landed and the in-crate test already
pinned the Excel target. The old catalog row and bead `oxf-pzav` were stale
tracker state, not a current discrepancy.

### Matrix `1x1` rows reclassified to Category 1

`MINVERSE(5)` and `MMULT(5,2)` preserve `1x1` arrays internally. Nested `TYPE`
evidence establishes that the scalar appearance occurs at final worksheet
publication, outside the context-free OxFunc comparison boundary. The MMULT row
was removed from Category 2; both witnesses now appear as Category-1
`publication_shape` entries `CSC-0024` and `CSC-0025`. Handoff `HO-FN-010` and
bead `oxf-i45e` remain open until the downstream publication/comparator seam is
acknowledged and integrated.

### Financial spot-check interpretation

The same fresh three-way replay prevents exact witnesses from prematurely
clearing argument-dependent rows:

- IRR: three witnesses exact, but `IRR({-100,50,60})` remains 80 ULP and the
  mixed five-flow witness remains 14,096 ULP; row stays open.
- NPER: the three-way witness is exact, but the distinct in-crate `NPER-0000`
  witness remains 1 ULP; row stays open.
- YIELD: 19 ULP; YIELDMAT: 1 ULP; both stay open.
- TBILLYIELD: the current three-way witness is exact; the recorded
  settlement-specific 1-ULP lane remains open pending recapture of that exact
  input rather than being cleared from one clean witness.

## Current canonical status

- Category-2 rows after the follow-up closure sweep: `23` (`G3=7`, `G4=5`,
  `G5=1`, `G6=10`)
- `scope_completeness`: `scope_partial`
- `target_completeness`: `target_partial`
- `integration_completeness`: `partial`
- `open_lanes`: G3 special/statistical exactness; G4 elementary/trig and
  conversion exactness; G5 MINVERSE multi-cell exactness; G6 annuity,
  day-count, solver, and financial rounding exactness (TBILLYIELD retired);
  Category-1 downstream
  publication seam `HO-FN-010`; alternate Excel versions/channels and locale
  sweeps.

## Pre-closure checklist for the retired YIELDDISC tracker item

The checklist applies only to the YIELDDISC discrepancy row/bead, not W108 or
global parity.

| # | Result | Evidence |
|---|---|---|
| 1 | Yes | Existing YIELDDISC contract/surface unchanged; this slice reconciles a landed formula repair. |
| 2 | Yes | No new semantic/formal slice introduced by tracker reconciliation. |
| 3 | Yes | The focused in-crate bit-exact test and catalog/tooling validations pass. |
| 4 | Yes | Fresh deterministic three-way witness recorded above. |
| 5 | Yes | Reproducer is `tools/g6-threeway/run-g6-threeway.ps1 -Only yielddisc.witness`. |
| 6 | Yes | Excel build and workbook Compatibility Version are explicit above. |
| 7 | Yes | Empirical Excel bits are the asserted target. |
| 8 | Yes | Cell-ref COM verification bypasses the XLL seam; no XLL qualification is material. |
| 9 | Yes | Formula numeric order is OxFunc-local; no FEC/F3E or evaluator-facing change. |
| 10 | Yes | No known YIELDDISC semantic gap remains in this declared current-baseline witness scope. |
| 11 | Yes | Claims are restricted to retiring this stale tracker item. |
| 12 | Yes | Canonical catalog/worklist surfaces are reconciled. |
| 13 | Yes | Bead `oxf-pzav` is retired with the live evidence; downstream bead `oxf-i45e` remains open. |

Completion-claim self-audit for that same narrow item: scope re-read `pass`;
gate criteria re-read `pass`; silent scope reduction check `pass`; scaffolding,
test, contract, Lean, and handoff-pattern check `pass`. The broader catalog and
W108 remain explicitly partial as reported above.

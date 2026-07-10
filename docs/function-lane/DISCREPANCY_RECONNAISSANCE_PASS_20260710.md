# Bounded Discrepancy Reconnaissance Pass — 2026-07-10

Status: `bounded_pass_recorded`; broader discrepancy work remains `in_progress`.

Follow-up closure audit: TBILLYIELD was subsequently repaired and removed after
a `2156/2156` exact sweep; ATANH's apparent two-witness x87-LN candidate failed
an expanded 368-case sweep and was reverted. See
[`CANDIDATE_CLOSURE_SWEEP_20260710.md`](CANDIDATE_CLOSURE_SWEEP_20260710.md).

## Scope and clean-room boundary

This pass gives every one of the 24 open Category-2 catalog rows the same bounded
treatment:

1. two deterministic representative cases;
2. exact binary64 local-vs-live-Excel comparison through `Range.Value2` cell refs;
3. a current implementation-path sketch;
4. one or more plausible Excel calculation paths;
5. a small set of discriminating probes for the next search.

The pass uses only source code owned by this repository, public algorithms and
libraries already admitted by project doctrine, published historical floating-point
information, and reproducible black-box Excel behavior. It does not inspect,
decompile, disassemble, or otherwise derive information from proprietary Excel
binaries.

Reference environment: Excel `16.0` build `20131`, workbook Compatibility Version
`2`, x86-64 host. Equality is typed bit equality with no tolerance.

## Reproducible artifacts

- Cases: `smart-fuzzer/corpus/discrepancy-recon/catalog-row-recon-v0.json`
- Runner: `smart-fuzzer/tools/Run-DiscrepancyCatalogRecon.ps1`
- Exact result ledger: `docs/function-lane/DISCREPANCY_RECON_RESULTS_20260710.csv`
- Calculation-path search map: `docs/function-lane/DISCREPANCY_CALCULATION_MAP.csv`
- Decomposition microprobes: `docs/function-lane/EXCEL_DECOMPOSITION_MICROPROBES_20260710.csv`
- Local detailed run: `smart-fuzzer/runs/catalog-row-recon-20260710-004/`

Re-run:

```powershell
.\smart-fuzzer\tools\Run-DiscrepancyCatalogRecon.ps1 `
  -RunId catalog-row-recon-current
```

## Per-row result

| Row | Two-case result | Bounded inference / next search |
|---|---|---|
| G3-01 distributions | BETAINV 2 ULP; CHIINV 3 ULP | Small stable drift. Search piecewise inverse schedules and x87 stores around tail transforms before replacing whole CDF kernels. |
| G3-02 GAMMA reflection | 182 ULP near -1; 9 ULP at -1.5 | Magnitude grows near a pole, consistent with reflection staging and trig/pi reduction. Recurrence-linked probes around negative integers are high-value. |
| G3-03 linear regression | FORECAST 5 ULP on exact line; perturbed case 2 ULP | Excel returns exact 10 for the exact line and matches both centered-slope and intercept-plus-slope worksheet decompositions. A centered covariance kernel is a stronger hypothesis than a generic low-bit matrix fix. |
| G3-04 GROWTH | 11 and 13 ULP | Excel GROWTH differs from worksheet log-domain regression and from direct EXP/LN or POWER on the geometric control. It appears to use a dedicated historical regression/publication path. |
| G3-05 chi-square tests | 8 and 7 ULP | Alias results track together. Split statistic accumulation from gamma-tail evaluation using permutations with the same mathematical statistic. |
| G3-06 F tests | 1 ULP on both aliases | Likely a variance-accumulator or beta-tail store boundary. Translation/reversal probes can distinguish them cheaply. |
| G3-07 GAUSS/PHI | GAUSS 2 ULP; PHI 1 ULP | PHI(0) isolates constant/publication rounding. GAUSS cancellation suggests a distinct direct kernel or CDF-minus-half staging. |
| G4-01 trig | TAN 719 ULP; SIN just below guard 5,664 ULP | Strong older-reduction signal. Search x87 FSIN/FCOS/FPTAN and FPREM1/extended-pi fingerprints; domain guard already matches. |
| G4-02 ATANH | bounded candidate matched two, expanded closure failed | The half-log-ratio graph matched `297/368` but regressed `71`; it was reverted. Current odd-symmetric platform path matches `235/368`; row remains open. |
| G4-03 ACOTH | 1 ULP on both | Excel ACOTH differs from Excel ATANH(1/x) and both obvious worksheet log identities. The shared-ATANH hypothesis is eliminated; search a distinct x87/log1p graph. |
| G4-04 combinatorial | COMBIN 1 ULP; PERMUT 1 ULP | Excel deliberately misses a representable integer in opposite directions across the two cases. This points to historical product/division or gamma paths, not an exact integer recurrence. |
| G4-05 CONVERT | m→ft 1 ULP; in→m exact | Unit-specific stored constants/direction matter. Infer Excel factor bits from power-of-two inputs and reciprocal direction. |
| G5-01 MINVERSE | two 2x2 cells each 1 ULP | Fingerprint small integer matrices against symbolic 2x2, Gauss-Jordan, and scaled-LU variants with x87 row stores. |
| G6-01 annuity | PMT witness exact; PPMT 1 ULP | Confirms the remaining witness is schedule/recurrence-sensitive, not a blanket PMT miss. Use stored-PMT component identities and the Phase-E `em` isolation. |
| G6-02 ACCRINT | base case exact; triple-edge 1 ULP | The previously unnamed residual is now pinned. Sum-day-numerators-then-divide is the first expression-tree candidate for constant bases. |
| G6-03 YIELD | catalog 19 ULP; par case 6 ULP | The par case should have a simple economic root yet still misses, strengthening the solver-iterate/publication hypothesis. |
| G6-04 ODDFYIELD | 311,909 and 307,444 ULP | Similar large drift at discount and par prices points decisively to inversion schedule/stopping, with the forward price kernel already aligned. |
| G6-05 RATE | mortgage 586 ULP; one-period identity 72 ULP | Excel does not publish the exact mathematical 0.1 in the identity case. Search for a returned intermediate iterate rather than nearest-root polishing. |
| G6-06 IRR | 80 and 14,096 ULP | Root slope and cashflow geometry change the gap sharply. Search NPV evaluation order jointly with solver schedule; residual minimization alone is not Excel's rule. |
| G6-07 CUMPRINC | full schedule exact; half schedule 1 ULP | Boundary-dependent accumulation. Compare sum of published PPMT rows, direct balance difference, and continuous recurrence. |
| G6-08 NPER | NPER-0000 1 ULP; control exact | Narrow log/store-boundary lane. Enumerate x87 LN/FYL2XP1 staging around numerator, denominator, and final division. |
| G6-09 YIELDMAT | basis 1 at 1 ULP; basis 0 at 2 ULP | Closed-form expression-tree search over integer day counts is tractable; test sum-before-divide and x87 continuous variants. |
| G6-10 TBILLYIELD | bounded controls exact; follow-up repaired and signed off | Expanded pre-repair matrix found 308 one-ULP cases; grouping `(360/days)` closed all `2156/2156`, so the row is retired. |
| G6-11 XNPV | fractional-year case 16 ULP; exact-year control exact | Excel XNPV is bit-identical to a direct worksheet POWER-per-term decomposition on the witness. Reproduced `power_kernel` per term is now a concrete repair candidate. |

## Calculation-path search vocabulary

The calculation map deliberately uses a small common vocabulary so future search
tools can enumerate paths instead of accumulating bespoke experiments:

1. `strict-f64`: store after every source-level operation.
2. `x87-continuous`: PC=64 extended temporaries retained across an expression.
3. `x87-block-store`: x87 primitives inside a block, one binary64 store at the
   named boundary.
4. `x87-transcendental`: known `FYL2X`, `FYL2XP1`, `F2XM1`, `FSIN`, `FCOS`,
   `FPTAN`, or `FPREM1` candidate.
5. `association-tree`: multiply/divide/add association and reciprocal staging.
6. `accumulator`: forward, reverse, pairwise, compensated, or x87-extended sum.
7. `solver-schedule`: Newton/secant/bisection hybrid, fixed iterations, stop
   predicate, and publication choice (last iterate, endpoint, midpoint, or
   minimum-residual float).
8. `table-constant`: direction-specific binary64 or extended unit/polynomial
   constants.

These are hypotheses, not claims about Excel implementation. A path is promoted
only after it predicts multiple independent live-Excel witnesses.

## Highest-value next experiments

1. XNPV: substitute the reproduced worksheet POWER/x87 path per term and score
   forward/reverse sums. This is the clearest reuse of the ExcelExp breakthrough.
2. GROWTH: compute one log-domain prediction and apply the reproduced x87 EXP once.
3. ATANH/ACOTH: enumerate four compact x87 log/reciprocal store patterns.
4. Trig: build an x87 FSIN/FCOS/FPTAN microprobe below the `2^27` guard and compare
   remainder fingerprints, without inspecting Excel code.
5. Solver cluster: make iteration traces first-class data for YIELD, ODDFYIELD,
   RATE, and IRR, then search schedule/publication combinations across all eight
   witnesses simultaneously.

## Status axes

- `scope_completeness`: `scope_complete` for this bounded 24-row reconnaissance
  packet; every row has two cases and a calculation map.
- `target_completeness`: `target_partial`; no row is claimed repaired by this pass.
- `integration_completeness`: `integrated` for the case corpus, runner, result
  ledger, and calculation map in OxFunc; downstream Category-1 work is unchanged.
- `open_lanes`: 23 Category-2 rows pending repair or retirement evidence;
  alternative Excel application versions/channels;
  locale/version sweeps; downstream publication handoff `HO-FN-010`.

## Pre-closure verification for this bounded packet

This checklist closes only the declared 24-row reconnaissance packet. It does
not close the discrepancy rows, W108, or global Excel parity.

| # | Result | Packet evidence |
|---|---|---|
| 1 | Yes | Function contracts are unchanged; stable row ids and calculation maps cover the declared evidence scope. |
| 2 | Yes | No new formal semantic slice is claimed. ATANH's existing Lean model covers domain/surface behavior; numeric-bit substrate remains outside that admitted model and is explicitly recorded. |
| 3 | Yes | Full `oxfunc_core --lib` suite passes, including new ATANH and XNPV-candidate tests. |
| 4 | Yes | Two deterministic cases exist for every row; decomposition probes are separately reproducible. |
| 5 | Yes | Case, runner, exact result, calculation-map, and decomposition artifacts are linked above. |
| 6 | Yes | Excel build, x86-64 host axis, and Compatibility Version are explicit. |
| 7 | Yes | Empirical Excel bits govern; mathematical identities are used only as discriminators. |
| 8 | Yes | Verification is direct COM/cell-ref and does not make an XLL-seam claim. |
| 9 | Yes | No FEC/F3E or evaluator-facing contract changed; XNPV production routing was deliberately not promoted. |
| 10 | Yes | No evidence-coverage gap remains in the declared bounded packet; function-semantic gaps remain openly listed. |
| 11 | Yes | Completion language is restricted to the reconnaissance packet. Rows remain open/M3 as appropriate. |
| 12 | Yes | Worklist, W108, catalog, and owning bug-stream records are updated. |
| 13 | Yes | No new execution blocker arose; existing downstream handoff `HO-FN-010` remains explicit. |

Completion-claim self-audit: scope re-read `pass`; gate criteria re-read `pass`;
silent scope reduction check `pass`; scaffolding/test/contract/Lean/handoff pattern
check `pass`. The bounded per-row timebox was preserved: two primary witnesses
per row, with deeper decomposition only where cross-row identities produced an
immediate discriminator.

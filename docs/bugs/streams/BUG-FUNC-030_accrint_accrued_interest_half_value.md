# BUG-FUNC-030: ACCRINT returns half of Excel's accrued interest

## Summary
- **Bug id**: `BUG-FUNC-030`
- **Opened**: `2026-05-28`
- **Status**: `closed_signed_off`
- **Owner workset**: `W090` intake; final G6-02 exactness closure under `W109`

## Source Refs
- **Reported against ref**: working tree at run `typed-arg-001`
- **Reproduced on ref**: runs `typed-arg-001`, `typed-arg-002`, W109
  b39/b40/b42/b43, and the frozen ACCRINT publication held-out
- **Introduced in ref**: `unknown`
- **Fixed in ref**: `cd1f9fe`
- **Ref notes**: The original half-value defect was repaired in the June W090
  tranche. Commit `cd1f9fe` closes the successor G6-02 publication residual
  against Excel `16.0` build `20228` x64, workbook Compatibility Version `2`,
  using NoCache `cell_value2_bulk` exact-bit evidence.

## Ownership And Root Cause
- **Ownership class**: `OxFunc-owned bug`
- **Root cause class**: `computation_defect`
- **Root cause summary** (diagnosed 2026-05-28): the bug is in the
  `settlement <= first_interest` (odd-first-stub) branch of `accrint_kernel`
  (`crates/oxfunc_core/src/functions/bond_core_family.rs`). It returns
  `coup * dd(issue, settlement) / dd(issue, first)` with `coup = par*rate/freq`.
  The denominator `dd(issue, first)` is the **entire** issue→first-interest
  span, which can be **more than one** quasi-coupon period. For the witness
  (issue 2020-01-01, first 2021-01-01, freq 2 → a 1-year, 2-period stub;
  settlement 2020-07-01) this gives `25 * 180/360 = 12.5`, whereas Excel sums
  over quasi-coupon periods: settlement is exactly one full quasi-period after
  issue, so `par*(rate/freq)*1 = 25`. The single linear interpolation is only
  correct when issue→first is exactly one period; it mishandles multi-quasi-
  coupon-period first stubs.
- **Correct algorithm**: MS ACCRINT first-stub formula
  `par * (rate/freq) * Σ_i (A_i / NL_i)` over the quasi-coupon periods (defined
  backward from `first_interest` by `12/freq` months) that the issue→settlement
  span touches, with day-counts per `basis`.
- **Lane (re-triaged 2026-05-28)**: `needs-analysis`, not the localized
  code-fix originally assumed. Requires implementing the quasi-coupon-period
  summation and verifying against an Excel matrix across basis conventions and
  1-period vs multi-period stubs (and the partial-end-period case). `ACCRINTM`
  matched, so the defect is specific to periodic `ACCRINT`.

## Reproduction
```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File smart-fuzzer\tools\Build-TypedArgProbes.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File smart-fuzzer\tools\Run-ArraySupportTranche.ps1 `
  -RunId typed-arg-001 `
  -CaseSetPath smart-fuzzer\cache\typed-arg-probes-v0.json
```

| Formula | OxFunc local | Excel |
| --- | --- | --- |
| `=ACCRINT(43831,44197,44013,0.05,1000,2,0)` | `number:12.5` (`0x4029000000000000`) | `number:25` (`0x4039000000000000`) |

Inputs: issue `43831` (2020-01-01), first_interest `44197`, settlement
`44013` (2020-07-01), rate `0.05`, par `1000`, frequency `2`, basis `0`.
Half-year accrual at 5% on par 1000 ≈ `25` (Excel). OxFunc returns `12.5`.

## Fix (2026-06-20)
Landed. `accrint_kernel` was rewritten from the single issue->first linear
interpolation to the MS quasi-coupon-period summation
(`crates/oxfunc_core/src/functions/bond_core_family.rs`): the accrual span is
walked over quasi-coupon periods (each `12/freq` months, anchored on
first_interest via `addm(first, k*m)` so end-of-month clamping never drifts); a
full period contributes one coupon, a partial period contributes
`accrued_days / normal_length`, and the whole coupon-fraction sum is scaled by
`par·rate/freq` once. `calc_method` was corrected to Excel's empirical
behaviour — TRUE accrues from issue, FALSE from one quasi-coupon period before
first_interest (signed, so a settlement before that start is negative).

## Validation (live Excel 16.0 build 20026, 2026-06-20)
A 15-case matrix over both branches, all five bases, freq 1/2/4, calc TRUE/FALSE,
end-of-month dates, and 1-period vs multi-period stubs:

- `13/15` exact typed bit matches, including the reported witness
  `=ACCRINT(43831,44197,44013,0.05,1000,2,0)` → `25` (was `12.5`); the
  multi-period stub forward case (old kernel under-counted by 90); and
  `calc_method=FALSE` (verified against a 6-case calc-method battery).
- `2/15` residual: `S5` `act/360` is 1 ULP (`137.5` vs Excel `137.50000000000003`);
  `S4` `act/act` (basis 1/3) is `~0.07%` off because Excel's normal-period-length
  for a *later* coupon period in a multi-coupon span deviates from the actual
  period length when that period crosses a leap February (isolated single
  periods use the actual length and match). This is a distinct, pre-existing
  `act/act` convention residual (the old kernel was equally off there), now
  tracked on the catalog ACCRINT row; not the half-value defect this stream opened.

Regression: `bond_core_family::tests::accrint_slices` updated to pin the witness
(`25`), `calc TRUE == FALSE` for a regular first coupon, and `TRUE > FALSE` for a
long first coupon. Full `oxfunc_core` lib suite green (1417 passed).

## Fix (2026-06-21) — act/act leap-February residual closed
The forward per-period loop measured each leap-crossing period by its *own* actual length
(182 days), but Excel normalises the settlement-side fraction by the **canonical** last
coupon period length `CoupDays(first - 1 period, first)` (184 days) — a single length, so a
leap-crossing period is never measured by its actual length. `accrint_kernel` was rewritten
as a faithful port of ExcelFinancialFunctions `accrInt` (bonds.fs):
- **settlement ≤ first** (odd first coupon): backward from `pcd = first - 1 period`, whole
  periods counting as `int(calc_method)`, settlement tail normalised by the canonical length,
  the issue period by its own length.
- **settlement > first** (a regime F#'s public API rejects, but Excel computes): a forward
  accrual from the accrual start, whole periods = 1 and the final partial by the canonical
  length. OxFunc now matches Excel here where F# throws.

Helpers ported: `change_month_flag`, `find_pcd_ncd_accr`, `diff360_us` (both 30/360 modes),
`days_between_num`/`days_between_denum`, `actual_coup_days_accr`/`coup_days_accr`.

## Validation (live Excel 16.0 b20026, G6 three-way harness, 2026-06-21)
Bit-exact across a **24-case sweep**: all five bases; leap-crossing act/act, act/365, act/360
partials; settlement before *and* after first_interest; quarterly/annual/semiannual; EOM
dates; deep multi-period; issue mid-period. **Residual: 1 ULP** on a single `us30360`
triple-edge (issue mid-period AND settlement past first_interest) — an operation-order artifact
(constant-length bases want sum-then-divide), reclassified NUM-S on the catalog G6 row; not
accepted. Regression `bond_core_family::tests::accrint_leap_february_and_settlement_after_first_bit_exact`
pins the act/act, act/365 leap partials and the settlement-after-first case.

## Final G6-02 Publication Residual Closure (2026-08-09)

The W109 b39/b40/b42 identification established the two accrual-fraction
programs but left 13 one-ULP publication rows. The final clean-room graph is:

```text
coupon = (par * rate) / frequency        # ordinary binary64, then stored
a      = identified accrual fraction     # ordinary binary64, then stored
result = RN53(RN64(coupon * a))          # excel_x87_mul(coupon, a)
```

The earlier statement that ACCRINT was entirely a plain-SSE2 body is therefore
too broad. Its day-count, branch, fraction, and coupon arithmetic remain the
identified ordinary-binary64 graph; only the final product crosses the legacy
x87 PC64 multiply followed by a binary64 result store. Applying x87 staging to
earlier operations is neither required nor authorized by this result.

The exact publication repair and focused pins landed in `cd1f9fe`.

### Exact replay evidence

| Corpus | Exact replay |
|---|---:|
| b39 identification lattice | `25,410/25,410` |
| b40 held-out lattice | `51,420/51,420` |
| b42 held-out lattice | `68,790/68,790` |
| b43 rate ladder, recaptured on build 20228 | `780/780` |
| frozen publication held-out, fresh build 20228 | `450/450` |
| **Combined** | **`146,850/146,850`** |

Current-reference sign-off provenance for b43 and the frozen held-out is Excel
`16.0` build `20228`, 64-bit, workbook Compatibility Version `2`, Windows x64,
`Run-W109BulkBatch.ps1 -NoCache`, and `cell_value2_bulk` input plumbing.

1. Recaptured b43 answer SHA-256:
   `CE2CB4B34FD46DEE40DDCD4724769471F255BA2ECE81D957546BA079D4CDF847`.
2. Fresh 450-row held-out answer SHA-256:
   `D0A6F58585AAE4E8C5727FD0EE5E792B686E970FF1B721B6D1CBD257C301B58C`.
3. The focused exact publication pin test passed `1/1`.
4. The production and selected candidate both replay b43 `780/780` and the
   frozen held-out `450/450`; companion id/argument/metadata predictions were
   validated before promotion.
5. Post-patch full core validation passed `1519` library tests with `0` failures
   and `4` ignored tests; every integration and doc-test target passed.
6. The preceding W109 formal build passed `492` Lean jobs. `cd1f9fe` changes no
   Lean definition or formal substrate; the existing BondCoreFamily binding and
   the amended contract carry the scoped alignment required by the executable-
   semantic-model strategy.

## Similar-Risk Scan
- `ACCRINTM` matched on equivalent inputs (not affected).
- Other coupon-period functions (`COUPDAYBS`, `COUPDAYS`, `COUPDAYSNC`,
  `COUPNCD`, `COUPNUM`, `COUPPCD`) matched bit-exactly in the same run, so
  the period/frequency machinery they share is not uniformly broken — the
  defect is local to `ACCRINT`.

## Evidence
1. `smart-fuzzer/tools/Build-TypedArgProbes.ps1`
2. ignored run artifacts under `smart-fuzzer/runs/typed-arg-001/`
3. `smart-fuzzer/planning/UNPOKED_SURFACE_COMPLETION_SWEEP_FINDINGS_2026-05-28.md` §4.2
4. `docs/function-lane/W109_G6_BOND_SCHEDULE_IDENTIFICATION_20260720.md`
5. `smart-fuzzer/tools/calc_graph_racer/src/bin/race_accrint_publication.rs`
6. `smart-fuzzer/tools/calc_graph_racer/src/bin/generate_accrint_publication_heldout.rs`
7. `smart-fuzzer/work/w109/G6-b2b3/answers-b43-accrint-build20228-20260809.json`
8. `smart-fuzzer/work/w109/G6-b2b3/answers-accrint-publication-heldout-20260809.json`

## Closure Checklist
- [x] fix landed or non-OxFunc ownership recorded (`cd1f9fe` for the final G6-02 residual)
- [x] validation recorded (`146,850/146,850` combined exact replay)
- [x] root cause recorded
- [x] similar-risk scan recorded
- [x] spec/matrix/contract updated if required
- [x] cross-repo impact assessed; handoff not required because no FEC/F3E or evaluator-facing clause changed
- [x] linked report, catalog, calculation map, workset, worklist, and register surfaces updated
- [x] closed bead retained closed and successor residual evidence attached by comment

## Closure Verification (2026-08-09)

Status axes for the declared `BUG-FUNC-030` / G6-02 current-reference
discrepancy slice:

- `execution_state: complete`
- `scope_completeness: scope_complete`
- `target_completeness: target_complete`
- `integration_completeness: integrated`
- `open_lanes: []` within this bug slice. Other bond/financial catalog rows,
  alternate application/channel/CPU profiles, locale sweeps, and the wider W109
  campaign remain outside this scoped closure; no bond-family, function-phase,
  workset, or global completion claim is made.

Pre-Closure Verification Checklist (`OPERATIONS.md` Section 12):

1. contract rows complete/promoted for the slice: yes; the ACCRINT final-
   publication clause and FDEF-057 are aligned at
   `provisional_w109_aligned`. The wider bond family remains provisional.
2. Lean/formal alignment satisfied: yes; the existing BondCoreFamily function
   binding remains applicable, the shared numeric helper is not duplicated in
   Lean per the formalization strategy, and the preceding `492`-job Lean build
   passed. `cd1f9fe` changes no formal artifact or substrate.
3. Rust implementation and required tests pass: yes; the focused pin is `1/1`,
   the full core run is `1519` passed, `0` failed, `4` ignored, and all
   integration/doc-test targets passed.
4. deterministic replay artifact exists: yes; five retained corpora replay
   `146,850/146,850` exact.
5. evidence links complete and reproducible: yes; the racer/generator, artifact
   paths, hashes, counts, and landed ref are recorded above.
6. both version axes explicit: yes; Excel 16.0 build 20228 x64 and workbook
   Compatibility Version 2 are the sign-off profile.
7. public-doc/empirical divergence handled in favor of Excel: yes; the
   calc_method behavior and final x87 publication follow black-box Excel rather
   than the simpler documented algebra.
8. XLL seam limitation documented where material: yes; it is not material to
   this direct worksheet-oracle/core-kernel slice.
9. cross-repo impact assessed: yes; no FEC/F3E boundary or evaluator-facing
   clause changed, so no handoff is required.
10. no known semantic gap remains in the declared G6-02 slice: yes.
11. completion-language audit passed: yes; closure is scoped to BUG-FUNC-030 /
    G6-02 and explicitly excludes the wider family and campaign.
12. in-progress worklist updated: yes; it records the landed ACCRINT repair and
    retains every wider open lane.
13. execution-state surface updated: yes; already-closed bead `oxf-bx1u` was
    not reopened, and a successor residual-closure comment records `cd1f9fe`
    and the exact evidence.

Completion Claim Self-Audit (`OPERATIONS.md` Section 14):

1. scope re-read: pass; only the periodic ACCRINT half-value lineage and its
   successor G6-02 calculation/publication residual are claimed.
2. gate criteria re-read: pass; exact graph, landed ref, focused/full tests,
   current-reference NoCache sign-off, contract alignment, and retained replay
   evidence are present.
3. silent scope reduction: pass; both calc_method paths, all bases/frequencies,
   pre/post-first-interest regimes, and discriminator/held-out classes remain
   represented in the retained corpora.
4. looks-done-but-is-not patterns: pass; no stub, compile-only path,
   unsupported proof claim, or unacknowledged handoff supports closure.
5. result: pass for the declared `BUG-FUNC-030` / G6-02 slice.

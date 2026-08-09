# BUG-FUNC-015: PMT/PPMT annuity exactness drift versus Excel

## Summary
- **Bug id**: `BUG-FUNC-015`
- **Opened**: `2026-04-28`
- **Status**: `investigating` (open — W108 partial repair landed; exact parity not landed)
- **Owner workset**: `W108` (absorbs the earlier W088/W103 lane)

## Re-Confirmation (2026-06-20) — not signed off
During the M3 sign-off sweep this stream was found mislabeled `M3` ("fixed-unsigned") in
the discrepancy catalog. There is **no landed fix** (`Fixed in ref: not yet fixed`; the
"fix landed" closure box is unchecked). The divergence was re-confirmed bit-for-bit against
the current OxFunc surface output and **live Excel 16.0 build 20026**:

| Formula | OxFunc | Excel | ULP |
|---------|--------|-------|-----|
| `=PMT(0.05/12,360,200000)` | `0xc090c692af15f632` | `0xc090c692af15f63a` | `8` |
| `=PPMT(0.05/12,1,360,200000)` | `0xc06e09eace0506e4` | `0xc06e09eace050723` | `63` |

Catalog maturity corrected `M3 → M1`. Remains a KED known-residual held for the W103
PMT-family substrate campaign.

## Source Refs
- **Reported against ref**: `d864c1bf0c1ba29e20f8858f0b5851f94352d88f`
- **Reproduced on ref**: `d864c1bf0c1ba29e20f8858f0b5851f94352d88f`
- **Introduced in ref**: `unknown`
- **Fixed in ref**: `not yet fixed`
- **Ref notes**: W088 smart-fuzzer pilot replayed local OxFunc value-surface
  calls against live Excel COM `Value2` on 2026-04-28.

## Ownership And Root Cause
- **Ownership class**: `OxFunc-owned bug`
- **Root cause class**: `initial_impl_gap`
- **Root cause summary**: the landed quotient-first discount structure is the
  right high-level PMT composition, but Excel's private `|tau|<1` annuity
  helper and the timing-1 association/publication path remain unidentified.
  Current evidence rules out several bounded graph and correction families; it
  does not prove irreducibility or authorize a fitted approximation.

## Why Did We Get This Wrong?
- **Spec already correct and code was wrong?**: `partial`
- **Spec vague or missing?**: `yes`
- **Code once correct and later regressed?**: `unknown`
- **Likely introduced in ref**: `unknown`
- **Explanation**: prior financial time-value evidence admitted representative
  numeric rows but did not pin the exact Excel publication behavior across
  enough non-zero-rate PMT/PPMT lanes. The smart-fuzzer pilot widened that
  evidence and found the drift is systematic rather than isolated to one
  witness row.

## Reproduction
1. Run:
   - `powershell -NoProfile -ExecutionPolicy Bypass -File smart-fuzzer\tools\Run-PmtPpmtPilot.ps1 -RunId w088-pmt-ppmt-pilot`
2. Current summary:
   - generated cases: `28`
   - local evaluated: `28`
   - Excel evaluated: `28`
   - exact matches: `7`
   - numeric bit mismatches: `21`
   - blocked: `0`
3. Known witness rows:
   - `PMT(0.05/12,360,200000)` local bits `0xc090c692af15f632`,
     Excel bits `0xc090c692af15f63a`
   - `PPMT(0.05/12,1,360,200000)` local bits `0xc06e09eace0506e4`,
     Excel bits `0xc06e09eace050723`

## Spec And Contract Relationship
- **Spec references**:
  1. `docs/function-lane/FUNCTION_SLICE_FINANCIAL_TIME_VALUE_FAMILY_CONTRACT_PRELIM.md`
  2. `docs/function-lane/W24_BATCH11_FINANCIAL_TIME_VALUE_EXECUTION_RECORD.md`
  3. `docs/function-lane/W24_BATCH11_FINANCIAL_TIME_VALUE_SCENARIO_MANIFEST_SEED.csv`
- **Spec state at intake**: `incomplete exactness characterization`
- **Notes**: existing admitted rows remain useful, but the PMT/PPMT exact
  publication lane is reopened for current-baseline Excel parity.

## Investigation Log
1. 2026-04-28: W088 added a PMT/PPMT pilot comparator that generates compact
   case JSONL, local outcomes, Excel outcomes, comparison telemetry, and
   full failure packets only for mismatches.
2. 2026-04-28: first pilot run exposed a harness display-width false positive;
   the Excel side was corrected to use a batched `ERROR.TYPE` companion column
   for typed error detection.
3. 2026-04-28: corrected run confirmed 21 numeric bit-level mismatches and 7
   exact matches, with zero-rate and invalid-period lanes matching exactly.
4. 2026-04-28: expanded smart-fuzzer run
   `expanded-finance-10m-20260428` generated and locally evaluated
   10,000,000 PMT/PPMT/IPMT-neighborhood cases, then sampled 640 cases against
   Excel. The sample produced 536 exact matches, 102 expected known
   financial-exactness or formula-literal encoding deviations, and 2 additional
   high-rate/long-horizon `PPMT` samples where local returned `#NUM!` while
   Excel returned a tiny numeric value or zero. These are recorded as adjacent
   evidence for the same blocked financial-payment exactness lane pending
   later investigation.
5. 2026-05-10: W097 R-C cell-ref re-replay. Both runners refactored to
   import `smart-fuzzer/tools/CellRefBatch.psm1` and pass numeric inputs
   via `Range.Value2`. See "Cell-Ref Re-Replay" section below for the
   per-function ULP histograms and the unexpected-mismatch escalation.

## Similar-Risk Scan
### Adjacent families to check
1. `IPMT`
2. `CUMIPMT`
3. `CUMPRINC`
4. `RATE` rows that depend on `PMT` inputs

### Check method
1. Extend the W088 pilot generator over adjacent financial time-value rows.
2. Keep exact `Value2` bit comparison and compact pass telemetry.
3. Promote only confirmed mismatches to failure packets and bug streams.

### Results
1. The current pilot confirms `PMT` and `PPMT` only.
2. Adjacent-family review remains open; do not infer adjacent parity from this
   pilot.

### Follow-on Openings
1. Bead: PMT/PPMT exactness repair/review opened from W088.

## Fix Plan
1. Characterize Excel's non-zero-rate PMT/PPMT publication rule over a wider
   matrix.
2. Decide whether the issue is in the shared annuity kernel, PPMT composition,
   or final publication/rounding policy.
3. Add focused exact-bit regression coverage for the confirmed witness set.
4. Re-run the PMT/PPMT pilot and adjacent-family scan before narrowing the bug.

## Validation
1. `cargo check --manifest-path smart-fuzzer/tools/pmt_ppmt_local_eval/Cargo.toml`
2. W088 pilot run `w088-pmt-ppmt-pilot`

## Linked Reports
1. `BUGREP-FUNC-019`

## Cell-Ref Re-Replay (W097 R-C, 2026-05-10)

Both runners (`Run-PmtPpmtPilot.ps1`, `Run-ExpandedFinanceExploration.ps1`)
were refactored to consume the shared `CellRefBatch.psm1` so numeric
inputs reach Excel via `Range.Value2` rather than formula-literal text.
See `smart-fuzzer/planning/W097-R-C-pmt-ppmt-ipmt-cell-ref-resweep.md` for
the full tranche record.

### Pilot 28-case (cell-ref `Run-PmtPpmtPilot.ps1`)

Run: `smart-fuzzer/runs/W097-R-C-pmt-ppmt-pilot-cellref/`. Match/
mismatch counts identical to the literal-text `w088-pmt-ppmt-pilot`
(`7` matches, `21` mismatches), and Excel-side `bits_hex` differences
between literal-text and cell-ref are zero across all 28 rows. The
pilot's short numeric literals round-trip correctly through Excel's
parser, so the recorded BUG-FUNC-015 magnitudes for this surface are
confirmed without revision.

### Finance broad seed 1M-case (cell-ref `Run-ExpandedFinanceExploration.ps1`)

Run: `smart-fuzzer/runs/W097-R-C-expanded-finance-1m-cellref/`.

| Metric                    | `expanded-finance-10m-20260428` (literal-text) | R-C (cell-ref) |
| ------------------------- | ---------------------------------------------: | -------------: |
| Excel sampled             |                                          `640` |          `800` |
| Match rate                |                                          `84%` |          `88%` |
| Known PMT-family drift    |                                          `102` |           `92` |
| Unexpected mismatches     |                                            `2` |            `2` |

Per-function ULP histogram of the cell-ref `known_residual_pmt_family_kernel_drift`:

| Function | rows | min ULP | median ULP | max ULP                |
| -------- | ---: | ------: | ---------: | ---------------------: |
| `PMT`    | `19` |     `0` |        `4` | `5.1E10`               |
| `IPMT`   | `22` |     `0` |      `832` | `4.2E16`               |
| `PPMT`   | `51` |     `0` |      `282` | `1.4E19` (saturating)  |

The PMT median of `4` ULP confirms a small-magnitude drift floor that
the literal-text run absorbed under `1e-12 * scale` tolerance. The
IPMT and PPMT distributions are bimodal: a tight cluster near
`0..1000` ULP and a long tail to `~10^16+` ULP for high-rate /
long-horizon / huge-PV combinations.

The two unexpected mismatches escalated from "expected drift" to a
true kind-drift class because OxFunc returns `#NUM!` while Excel
returns a finite tiny denormal:

- `=PPMT(0.94202241811931720, 1147, 1600, 677560705614.16699...)` →
  local `error:Num`, excel `-8.66E-120` (`0xa7365da0faa805b4`)
- `=PPMT(0.65754274790347489, 475, 1992, 629739.80507821717765182)` →
  local `error:Num`, excel `0` (`0x0000000000000000`)

These are the same shape as the two adjacent witnesses already noted
in Investigation Log item 4. They join the BUG-FUNC-015 repair scope
as a kind-drift sub-class (PPMT high-rate / long-horizon / huge-PV
should not raise `#NUM!` when Excel returns a finite value).

## Evidence
1. `smart-fuzzer/tools/Run-PmtPpmtPilot.ps1`
2. `smart-fuzzer/tools/Run-ExpandedFinanceExploration.ps1`
3. `smart-fuzzer/tools/CellRefBatch.psm1`
4. `smart-fuzzer/tools/pmt_ppmt_local_eval/`
5. ignored local run artifacts under `smart-fuzzer/runs/w088-pmt-ppmt-pilot/`
6. ignored local run artifacts under `smart-fuzzer/runs/expanded-finance-10m-20260428/`
7. W092 reference replay:
   `smart-fuzzer/runs/w092-axis-known-reference-cycle-001/` records the PMT
   reference pair as `known_expected_deviation` under the axis-witness harness.
8. W097 R-C cell-ref re-replay tranche record:
   `smart-fuzzer/planning/W097-R-C-pmt-ppmt-ipmt-cell-ref-resweep.md`
9. W097 R-C cell-ref pilot run:
   `smart-fuzzer/runs/W097-R-C-pmt-ppmt-pilot-cellref/`
10. W097 R-C cell-ref finance broad seed run:
    `smart-fuzzer/runs/W097-R-C-expanded-finance-1m-cellref/`

## 2026-07-05 W108 Phase-E Reconciliation (validated 2026-07-10)

Further clean-room research under `C:\Temp\ExcelExpFunction` supplied `5,319`
live-Excel family rows on Excel 16.0 build 20131. Inputs were transferred through
`Range.Value2` and captured with exact binary64 input/output bits.

The new evidence supersedes two earlier interpretations while preserving the
landed partial repair:

1. Excel PMT uses the discount arrangement already present in the current
   OxFunc kernel:
   `em=expm1(-n*log1p(r)); v=1+em; pmt=(pv+fv*v)*r/em`.
2. The Phase-C OpenOffice/LibreOffice forward-form conclusion is rejected by
   large-`t` rows where Excel collapses `v=1+em` to zero while the forward form
   remains nonzero.
3. The historical Kahan/Goldberg `log1p` and `expm1` identities, built on the
   W108 x87 `EXP`/`LN` substrate, are the best tested primitive model.
4. The best candidate scores `2285/4040` PMT rows exact and `92.9%` within
   `1` ULP. On `553` `nper=1` isolation rows, PMT's final division is exact
   whenever the candidate `em` is exact, localizing the remaining last-bit gap
   to `em=expm1(-log1p(rate))`.
5. The candidate is not promoted into Rust: it has not yet demonstrated a
   non-regressing advantage over current OxFunc on one common repo-owned corpus,
   and approximately `43%` of the adversarial PMT rows still differ by a last
   bit.

Validation rerun on 2026-07-10:

- `python validate_reference.py research/data/ground_truth_all.json research/data/disc2_results.json`
  -> x87 EXP/LN `294/294` exact.
- `python validate_power_reference.py` -> POWER `315/315` exact.
- `python finlab/final_sweep.py` -> PMT candidate `2285/4040` exact,
  `92.9%` within `1` ULP.
- `python finlab/score_family.py` -> public-source recurrence model `244/855`
  exact on the adjacent family; this is model telemetry, not an OxFunc pass rate.

Canonical synthesis:
`docs/EXCEL_FINANCIAL_ANNUITY_SPEC_AND_FINDINGS.md`.
Compact exact-bit witness seed:
`docs/function-lane/W108_ANNUITY_PHASE_E_WITNESS_SEED.csv`.

Current open sub-lanes:

1. repo-owned replay of all Phase-E exact inputs against current OxFunc,
2. exact partial-extended store/rounding placement in `log1p`/`expm1`,
3. per-function IPMT/PPMT/CUM op-order and accumulation,
4. alternate CPU, Excel channel, and workbook Compatibility Version validation.

## 2026-07-03 W108 Root-Cause Resolution (partially superseded by Phase E)

The historical notes below explain the partial repair landed on `ecbcd60`, but
their pure-double and forward-chain interpretations are superseded by the July 4
x87 result and the July 5 Phase-E annuity evidence recorded later in this file.

A deep multi-agent investigation (live Excel 16.0 b20131, 64-bit; 25 targeted
probes + 855-cell factor grid + intermediate-precision discriminators) fully
root-caused this drift and the adjacent family. Repair is scoped under
`docs/worksets/W108_EXCEL_NUMERIC_CORE_AND_FINANCIAL_POWER_EXACTNESS.md`
(epic `oxf-wpzw`, beads `oxf-wpzw.1/.2/.3`).

Findings:

1. **FV/PV already match Excel bit-exact** (`powi`/`powf` factor + `(F-1)/r`
   term). The drift is NOT a shared growth factor.
2. **PMT uses a different substrate than FV/PV**: `exp(n*log1p(r))` factor +
   `expm1`-based term, not the `powi` kernel. OxFunc's PMT wrongly reuses the FV
   `powi` kernel -> catastrophic error on the common loan regime (up to `5.5e8`
   ULP; `PMT(1e-9,120,1e5)`). Switching PMT to the exp/expm1 chain collapses this
   to <=2 ULP and closes the `PMT(0.05/12,360,200000)` witness bit-exact.
3. **64-bit Excel is pure IEEE-754 double** (SSE2; no x87, no FMA, no wide
   intermediates) — proven directly (`=A1*A1-B1`, dot-product, `=A1^2-B1` all
   publish `0x0`). The fix is fully portable; the earlier x87-tail hypothesis is
   retracted.
4. **Excel's `exp`/`log` are correctly-rounded, not UCRT** (Rust `f64::exp` =
   UCRT; UCRT `log1p` misrounds ~21%). Matching Excel across the full input space
   requires a correctly-rounded `exp`/`expm1`/`log1p`/`log` (CR vs Intel SVML fork
   resolved in W108-A).
5. **NPER solved**: `CR_log(ratio) / log(1+r)` (numerator correctly-rounded,
   denominator plain `ln(1+r)` NOT `log1p`).
6. **PPMT/IPMT/CUMPRINC use a dedicated internal principal path** (running-balance
   / geometric-factor), NOT `PMT - IPMT` — proven: built-in `PPMT(...,1,...)`
   `...0723` vs standalone `PMT-IPMT` `...0724` vs `CUMPRINC(1,1)` `...0722`.

Repair path (portable, pure-double): explicit Excel elementary-primitives core
(`exp`/`expm1`/`log1p`/`log` + `powi`/`powf`) validated against a batch oracle
corpus, then re-express PMT/PPMT/IPMT/CUMPRINC/CUMIPMT/NPER (and fix the related
`POWER` 377-ULP surface anomaly) explicitly on those primitives. Isolation lanes
used: `FV(r,n,0,-1,0)` = Excel's internal `(1+r)^n`; `FV(r,n,-1,0,type)` = the
annuity term.

## Closure Checklist
- [ ] local fix implemented
- [ ] validation recorded
- [x] root cause recorded
- [x] similar-risk scan recorded
- [ ] spec/matrix/contract updated if required
- [x] linked reports updated
- [ ] handoff filed if required
- [ ] fix landed or non-OxFunc ownership recorded

## 2026-08-09 W109 Intermediate And Timing Discrimination

This checkpoint supersedes the July-24 claims that the residual was “not
black-box-closable”, “needed provenance”, or had reached a proved boundary.
Those claims exceeded the evidence. The clean-room result is bounded-negative
only for the explicitly searched leaves, operators, precision/store schedules,
and graph sizes. A reproducing Excel program exists; larger graphs and newly
designed discriminators remain actionable. The long EXT6 run is still active,
with its durable log last reporting shard `191/400`.

The current-build evidence uses Excel `16.0` build `20228`, 64-bit, workbook
Compatibility Version `2`, `Range.Value2` bulk input/output, and `NoCache` where
fresh capture was required.

### Private-helper intermediate audit

1. Across `234` power-of-two-rate and `90` independent general-rate rows, the
   ordinary binary64 Kahan representative scores `224/324`; the per-operation
   x87-PC64-to-binary64 spill representative scores `226/324`.
2. Exact representability of `n*log1p(rate)` is strongly associated with misses
   in the power-of-two and pooled adjusted analyses, but the independent
   general-rate adjusted result does not support treating it as a universal
   branch predicate. It remains a probe stratifier, not an identified mechanism.
3. A TwoProduct low-word repair cannot close the gap: `239/324` rows have a zero
   exact low word, leaving `93` existing x87-helper misses immutable and an
   absolute low-only ceiling of `231/324`.
4. All `60` tested smooth degree-`0..8` quotient, denominator, and joint
   smooth-plus-low interval systems require positive numerical LP widening;
   exact rational Farkas certificates establish infeasibility for `51/60`.
   The other nine remain numerical negative evidence only. Least-squares gains
   fail to transfer to the independent general-rate cohort, so no coefficient
   vector is promoted.

### Timing-factor metamers

The `fv=0`, power-of-two-rate metamer is a strong local identity: stored
reciprocal multiplication matches all `832/832` paired type-0/type-1 rows,
while true division matches `773/832`. This cancels the opaque helper only in
that algebraically special lane and is not a global timing-order proof.

A separately frozen general-rate discriminator captured `15` contexts, each
with a `16`-value consecutive-PV ladder at both timing values (`480/480` calls).
The best tested subtractive family,
`(q - q*store(rate/(1+rate)))*rate`, scores `378/480` (`239/240` type 0,
`139/240` type 1), but explains only `1/15` contexts exactly. PC64-continuous
divide and reciprocal forms are observational twins at `360/480`; every stored
divide/reciprocal and before/after-rate alternative scores lower. The remaining
type-1 helper/association path therefore contaminates any standalone tail
inference, and no timing graph is identified or eligible for production.

### Current decision

`BUG-FUNC-015` remains `investigating`. No production or formal change is
authorized by this checkpoint. The next useful probe is a frozen
helper-association discriminator for the type-1 lane, followed by larger-graph
search and per-function PPMT/IPMT/CUM recurrence/publication identification.
The canonical evidence and artifact hashes are recorded in
`docs/function-lane/W109_PMT_INTERMEDIATE_AND_TIMING_DISCRIMINATION_20260809.md`.

Status axes:

1. `scope_completeness`: `scope_partial`
2. `target_completeness`: `target_partial`
3. `integration_completeness`: `partial`
4. `open_lanes`: private `|tau|<1` helper graph; type-1 helper association and
   timing-factor staging; PPMT/IPMT/CUM recurrence/publication graphs; unfinished
   EXT6 and larger-graph search; alternate Excel-version/Compatibility-Version
   validation; production/formal integration after exact identification.

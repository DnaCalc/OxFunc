# BUG-FUNC-015: PMT/PPMT annuity exactness drift versus Excel

## Summary
- **Bug id**: `BUG-FUNC-015`
- **Opened**: `2026-04-28`
- **Status**: `validated_local`
- **Owner workset**: `W088`

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
- **Root cause summary**: non-zero-rate `PMT` and `PPMT` publication currently
  follows the local annuity formulas closely but not Excel's exact `Value2`
  result bits across the pilot matrix.

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

## Evidence
1. `smart-fuzzer/tools/Run-PmtPpmtPilot.ps1`
2. `smart-fuzzer/tools/pmt_ppmt_local_eval/`
3. ignored local run artifacts under `smart-fuzzer/runs/w088-pmt-ppmt-pilot/`

## Closure Checklist
- [ ] local fix implemented
- [ ] validation recorded
- [x] root cause recorded
- [x] similar-risk scan recorded
- [ ] spec/matrix/contract updated if required
- [x] linked reports updated
- [ ] handoff filed if required
- [ ] fix landed or non-OxFunc ownership recorded

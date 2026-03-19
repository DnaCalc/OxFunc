# WORKSET - Ordinary Functions Mega-Batch Execution Plan (W24)

## 1. Purpose
Execute the remaining ordinary low-interest residuals as one uninterrupted mega-batch without weakening OxFunc closure discipline.

This packet is for the post-`W023` ordinary residuals only:
1. bounded semantic-hardening families,
2. locale/profile-sensitive but still ordinary function families,
3. financially or statistically heavy but still non-host/non-metadata functions.

It is not for:
1. host/query/visibility functions,
2. cell-metadata functions,
3. database-grid functions,
4. deferred operator/seam work such as `@`.

## 2. Dependencies and Ownership
Dependencies:
1. `W017` residual inventory freeze,
2. `W022` criteria-family closure,
3. `W023` extraction of host/metadata/database functions.

Machine-readable checklist:
1. `docs/function-lane/W24_ORDINARY_FUNCTIONS_MEGA_BATCH_CHECKLIST.csv`

Current target:
1. `87` ordinary residual functions from `W17`.

## 3. Execution Principle
The batch runs continuously down a fixed checklist.

Rules:
1. work down the checklist order without pausing for narrative checkpoints,
2. finish the full output/test/evidence bundle for one family before marking any row in that family complete,
3. if a family hits a real blocker, file `BLK-FN-*`, mark the family blocked, and continue immediately with the next non-blocked family,
4. do not silently narrow scope,
5. do not claim completion for any function or family until the normal OxFunc closure surfaces exist.

## 4. Mandatory Output Contract Per Family
Each family completed inside `W24` must still produce the same closure surfaces used elsewhere in the repo.

Minimum required outputs:
1. one family note:
   - `docs/function-lane/W24_BATCH##_..._NOTES.md`
2. one seeded native replay manifest:
   - `docs/function-lane/W24_BATCH##_..._SCENARIO_MANIFEST_SEED.csv`
3. one runtime-requirements note:
   - `docs/function-lane/W24_BATCH##_..._RUNTIME_REQUIREMENTS.md`
4. one family execution record or a clearly delimited section inside a `W24` master execution record,
5. Rust runtime implementation/tests,
6. Lean substrate/binding update aligned to the family substrate,
7. evidence-registry row,
8. `.tmp` replay artifact for the native baseline,
9. XLL-facing verification only where the family surface makes that materially relevant,
10. conformance/contract promotion when the family closes.

## 5. Mandatory Verification Per Family
For each family, the minimum verification set is:
1. native replay runner against the seeded manifest,
2. targeted Rust tests for that family,
3. `cargo check --manifest-path tools/xll-addin/oxfunc_xll/Cargo.toml` when shared dispatch/export surfaces changed,
4. `lake build`,
5. `cargo fmt --manifest-path crates/oxfunc_core/Cargo.toml`.

When a family changes shared core behavior materially, run:
1. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml` for the affected family or the whole crate, whichever is honest for the touched surface.

## 6. Reduction Rules
This is a mega-batch, but it is not a giant undifferentiated blob.

The allowed reduction unit is:
1. family,
2. or tightly-coupled alias family.

The forbidden reduction unit is:
1. “all 87 functions at once with one combined evidence claim”.

So the batch is continuous in execution order, but closure still happens family-by-family inside the batch.

## 7. Checklist Tick Rule
A function row in `W24_ORDINARY_FUNCTIONS_MEGA_BATCH_CHECKLIST.csv` may be marked `done` only when:
1. its owning family note exists,
2. its family replay artifact exists and is green,
3. its Rust implementation/tests are green,
4. its Lean alignment/binding state is updated,
5. its contract/conformance surface is updated,
6. no known semantic gap remains for the declared current-baseline slice.

Allowed row states:
1. `planned`
2. `in_progress`
3. `blocked`
4. `done`
5. `extracted` (only if new evidence proves the row is not honestly an ordinary function after all)

## 8. Wave Order
The checklist order is grouped into waves that maximize throughput while reducing context switching.

### Wave 1 - Small Ordinary Hardening
1. `DATEVALUE`, `TIMEVALUE`, `DAYS360`, `DATEDIF`
2. `SWITCH`
3. `TEXTAFTER`, `TEXTBEFORE`
4. `ARRAYTOTEXT`, `TEXTSPLIT`

### Wave 2 - Statistical and Distribution Helpers
1. `CONFIDENCE.T`, `Z.TEST`
2. `ERF`, `ERF.PRECISE`, `ERFC`, `ERFC.PRECISE`, `GAMMA`, `GAMMALN`, `GAMMALN.PRECISE`, `WEIBULL`, `WEIBULL.DIST`
3. `CHISQ.TEST`, `CHITEST`, `F.TEST`, `FTEST`, `T.TEST`, `TTEST`, `ZTEST`

### Wave 3 - Lookup, Frequency, and Regression
1. `LOOKUP`, `FREQUENCY`, `PROB`, `MODE.MULT`
2. `GROWTH`, `TREND`, `LINEST`, `LOGEST`

### Wave 4 - Locale and Text/Profile Functions
1. `ASC`, `DBCS`, `JIS`
2. `NUMBERVALUE`, `REGEXEXTRACT`, `REGEXREPLACE`, `REGEXTEST`, `TRANSLATE`

### Wave 5 - Financial Core Time-Value
1. `PV`, `FV`, `PMT`, `NPER`, `NPV`, `RATE`, `IPMT`, `PPMT`, `ISPMT`, `MIRR`, `FVSCHEDULE`, `PDURATION`, `RRI`, `NOMINAL`, `EFFECT`
2. `IRR`, `XNPV`, `XIRR`

### Wave 6 - Coupon, Bond, and Odd-Bond
1. `COUPDAYBS`, `COUPDAYS`, `COUPDAYSNC`, `COUPNCD`, `COUPNUM`, `COUPPCD`
2. `ACCRINT`, `ACCRINTM`, `DURATION`, `MDURATION`, `PRICE`, `PRICEMAT`, `YIELD`, `YIELDDISC`, `YIELDMAT`
3. `ODDFPRICE`, `ODDFYIELD`, `ODDLPRICE`, `ODDLYIELD`
4. `AMORDEGRC`, `AMORLINC`

### Wave 7 - Misc Ordinary Residuals
1. `BAHTTEXT`, `CONVERT`, `EUROCONVERT`, `PERCENTOF`, `RANDARRAY`

## 9. Blocker Policy
When a family is blocked:
1. record the blocker in `CURRENT_BLOCKERS.md`,
2. mark the affected rows `blocked`,
3. continue with the next family in checklist order,
4. only extract to a new successor workset if new evidence shows the family does not belong in the ordinary batch.

## 10. Completion Rule
`W24` closes only when every checklist row is either:
1. `done`, or
2. `extracted` with explicit successor ownership and rationale.

No row may remain silently open.

`W17` may only be reduced/closed after the `W24` checklist has been reconciled back into the residual inventory.

## 11. Status
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`
5. open_lanes:
   - none in declared `W24` scope after reconciliation to `W24_SCOPE_RECONCILIATION.csv`.

# BUGREP-FUNC-006: Local Excel probe on numeric comparison family split

## Intake
- **Report id**: `BUGREP-FUNC-006`
- **Filed**: 2026-04-08
- **Source channel**: local test
- **Reporter/source**: live Excel COM probe on current workstation
- **Reported against ref**: `7989fafaef703f15f2bfbdded323c03345da1072`
- **Reported against kind**: `commit`
- **Reported against note**: Exact working ref pinned with `git rev-parse HEAD` at intake time.
- **Canonical bug id**: `BUG-FUNC-004`
- **Status**: `triaged`

## Observed Symptom
Local Excel probing showed that near-equality tolerance is not limited to the
ordinary operators named in the inbound handoff. `COUNTIF` / `SUMIF`, database
criteria matching, and `SWITCH` share the tolerant lane, while `MATCH`,
`XMATCH`, and `DELTA` exact-match paths remain exact on the tested scenarios.

## Reproduction
1. Evaluate the following against live Excel:
   - `=COUNTIF(A1:A1,0.1+0.2)` with `A1=0.3`
   - `=SUMIF(A1:A1,0.1+0.2,B1:B1)` with `A1=0.3`, `B1=7`
   - `=DCOUNT(D1:E2,"V",G1:G2)` / `=DSUM(...)` with numeric criteria
   - `=SWITCH(0.1+0.2,0.3,1,2)`
   - `=MATCH(0.1+0.2,{0.3},0)`, `=XMATCH(0.1+0.2,{0.3},0)`,
     `=DELTA(0.1+0.2,0.3)`
2. Expected result: family behavior split must be recorded explicitly.
3. Actual result: tolerant in operator/criteria/database/`SWITCH`; exact in
   `MATCH` / `XMATCH` / `DELTA` exact-match paths.

## Initial Ownership Read
- **Initial classification**: `OxFunc-owned bug`
- **Reason**: the tolerant families are all implemented in OxFunc-local helpers
  today, and the local evidence/contract packet had not pinned this family
  split before.

## Links
1. `docs/function-lane/FLOATING_POINT_SCENARIO_MANIFEST_SEED.csv`
2. `docs/function-lane/FLOATING_POINT_EXECUTION_RECORD.md`
3. `crates/oxfunc_core/src/functions/criteria_family.rs`
4. `crates/oxfunc_core/src/functions/database_family.rs`
5. `crates/oxfunc_core/src/functions/misc_switch_info_family.rs`
6. `crates/oxfunc_core/src/functions/match_fn.rs`
7. `crates/oxfunc_core/src/functions/xmatch.rs`
8. `crates/oxfunc_core/src/functions/delta_fn.rs`

## Triage Notes
This report widened `BUG-FUNC-004` beyond the original handoff wording and is
the reason `W077` reopens criteria/database/SWITCH rows in `W051`.

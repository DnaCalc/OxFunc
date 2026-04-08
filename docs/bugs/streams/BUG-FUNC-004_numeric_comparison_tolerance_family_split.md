# BUG-FUNC-004: Numeric comparison tolerance missing from tolerant families

## Summary
- **Bug id**: `BUG-FUNC-004`
- **Opened**: 2026-04-08
- **Status**: `validated_local`
- **Owner workset**: `W077`

## Source Refs
- **Reported against ref**: `7989fafaef703f15f2bfbdded323c03345da1072`
- **Reproduced on ref**: `7989fafaef703f15f2bfbdded323c03345da1072`
- **Introduced in ref**: `unknown`
- **Fixed in ref**: `not yet fixed`
- **Ref notes**: Intake pinned the current working ref with `git rev-parse HEAD`.
  The local bug was reproduced against a live Excel instance and then corrected
  in the working tree, but no landed commit ref exists yet.

## Ownership And Root Cause
- **Ownership class**: `OxFunc-owned bug`
- **Root cause class**: `test_gap`
- **Root cause summary**: OxFunc reused exact IEEE-style numeric comparison
  helpers across ordinary operators, criteria/database selection, and `SWITCH`
  without a dedicated empirical lane for Excel near-equality semantics. Earlier
  W45 / W22 / W23 / W65 evidence packets never exercised the stronger
  arithmetic-generated boundary cases, so the first local correction used a
  round-to-nearest 15-digit surrogate that still diverged from Excel on the
  boundary rows.

## Why Did We Get This Wrong?
- **Spec already correct and code was wrong?**: `yes`
- **Spec vague or missing?**: `no`
- **Code once correct and later regressed?**: `no`
- **Likely introduced in ref**: `unknown`
- **Explanation**: The target contract is empirical Excel parity, but the local
  compare helpers stayed on exact-double semantics because the admitted native
  evidence sets did not include near-equality comparison rows. That made the
  code and the evidence packet both too weak for the promoted completion claim.

## Reproduction
1. Live Excel on 2026-04-08 observed:
   - `=0.1+0.2=0.3 -> TRUE`
   - `=0.1+0.2<>0.3 -> FALSE`
   - `=0.1+0.2<0.3 -> FALSE`
   - `=0.1+0.2<=0.3 -> TRUE`
   - `=0.1+0.2>0.3 -> FALSE`
   - `=0.1+0.2>=0.3 -> TRUE`
   - `=COUNTIF(A1:A1,0.1+0.2) -> 1` with `A1=0.3`
   - `=SUMIF(A1:A1,0.1+0.2,B1:B1) -> 7` with `A1=0.3`, `B1=7`
   - `=DCOUNT(...) -> 1`, `=DSUM(...) -> 0.3` on matching numeric criteria
   - `=SWITCH(0.1+0.2,0.3,1,2) -> 1`
   - `=((123456789012345*10)+5)/1E25=((123456789012345*10)+4)/1E25 -> TRUE`
   - `=((123456789012345*10)+5)/1E25<>((123456789012345*10)+4)/1E25 -> FALSE`
   - `=((123456789012345*10)+5)/1E25>((123456789012345*10)+4)/1E25 -> FALSE`
   - `=((123456789012345*10)+5)/1E25<=((123456789012345*10)+4)/1E25 -> TRUE`
   - `=COUNTIF(C2:C2,((123456789012345*10)+5)/1E25) -> 1` with `C2=((...)+4)/1E25`
   - `=SWITCH(((123456789012345*10)+5)/1E25,((123456789012345*10)+4)/1E25,1,2) -> 1`
2. Contrast exact-match Excel observations:
   - `=MATCH(0.1+0.2,{0.3},0) -> #N/A`
   - `=XMATCH(0.1+0.2,{0.3},0) -> #N/A`
   - `=DELTA(0.1+0.2,0.3) -> 0`
   - `=MATCH(((123456789012345*10)+5)/1E25,B1:B1,0) -> #N/A` with
     `B1=((123456789012345*10)+4)/1E25`
   - `=XMATCH(((123456789012345*10)+5)/1E25,B1:B1,0) -> #N/A`
   - `=DELTA(((123456789012345*10)+5)/1E25,B1) -> 0`
3. Actual pre-fix OxFunc behavior:
   - operators, criteria/database, and `SWITCH` all used exact-double numeric
     equality/order helpers and therefore diverged on the tolerant lane,
   - the first local helper correction still diverged on the arithmetic-
     generated boundary pair because it rounded to 15 significant digits rather
     than matching the observed truncation-style lane.

## Spec And Contract Relationship
- **Spec references**:
  1. `docs/function-lane/FUNCTION_SLICE_OPERATOR_COMPARE_CONCAT_FAMILY_CONTRACT_PRELIM.md`
  2. `docs/function-lane/FUNCTION_SLICE_CRITERIA_FAMILY_CONTRACT_PRELIM.md`
  3. `docs/function-lane/FUNCTION_SLICE_DATABASE_FAMILY_CONTRACT_PRELIM.md`
  4. `docs/function-lane/FLOATING_POINT_BEHAVIOR_RESEARCH_NOTES.md`
- **Spec state at intake**: `correct_and_not_implemented`
- **Notes**: the problem was not a missing target; it was a missing empirical
  comparison lane in the local evidence packets and a resulting implementation
  gap in the shared numeric helpers.

## Investigation Log
1. 2026-04-08: processed `HANDOFF-OXFUNC-003` and pinned the working ref.
2. 2026-04-08: re-ran live Excel for `IF("",1,2)` and `IFS("",1,TRUE,2)`,
   confirming that both return `#VALUE!` and therefore do not open a local
   OxFunc bug.
3. 2026-04-08: confirmed `operator_compare_concat_family.rs`,
   `criteria_family.rs`, `database_family.rs`, and `misc_switch_info_family.rs`
   all used exact-double numeric comparison helpers.
4. 2026-04-08: confirmed `MATCH`, `XMATCH`, and `DELTA` exact-match paths stay
   exact on the tested near-equality cases.
5. 2026-04-08: opened bounded owner `W077`.
6. 2026-04-08: added a shared Excel-style numeric comparison helper and routed
   ordinary operator compare, criteria/database numeric criteria, and `SWITCH`
   through it while leaving exact-match contrast families untouched.
7. 2026-04-08: added focused Rust tests plus widened native probe manifests for
   the tolerant family and the exact-match contrast cases.
8. 2026-04-08: boundary review against live Excel showed the first helper model
   still diverged on the arithmetic-generated pair
   `((123456789012345*10)+5)/1E25` versus `((123456789012345*10)+4)/1E25`.
9. 2026-04-08: corrected the helper to the empirically pinned truncation-style
   15-significant-digit lane, widened the local criteria/database coverage to
   the touched `*IFS` and additional database members, and re-ran the native
   probes successfully.

## Similar-Risk Scan
### Adjacent families to check
1. `COUNTIF`, `COUNTIFS`, `SUMIF`, `SUMIFS`, `AVERAGEIF`, `AVERAGEIFS`,
   `MAXIFS`, `MINIFS`
2. `DAVERAGE`, `DCOUNT`, `DCOUNTA`, `DGET`, `DMAX`, `DMIN`, `DPRODUCT`,
   `DSTDEV`, `DSTDEVP`, `DSUM`, `DVAR`, `DVARP`
3. `SWITCH`
4. `MATCH`, `XMATCH`, `DELTA`
5. `IF` / `IFS` text-condition coercion

### Check method
1. searched local numeric comparison helpers and family-specific equality paths,
2. ran live Excel probes for operators, criteria, database criteria, `SWITCH`,
   `MATCH`, `XMATCH`, `DELTA`, `IF`, and `IFS`,
3. added focused Rust tests to lock both the tolerant families and the exact
   contrast families,
4. widened W45 / floating-point empirical manifests for replayable evidence.

### Results
1. ordinary operators are tolerant on the tested near-equality cases.
2. criteria/database numeric criteria are tolerant on the tested near-equality
   cases and therefore reopen those previously promoted families.
3. `SWITCH` exact-match selection is tolerant on the tested near-equality
   cases.
4. the stronger arithmetic-generated boundary pair confirms that the tolerant
   helper must follow the truncation-style 15-significant-digit lane rather than
   round-to-nearest.
5. `MATCH`, `XMATCH`, and `DELTA` exact-match paths remain exact on the tested
   cases and therefore stay out of the tolerant helper.
6. `IF` / `IFS` empty-text condition handling already matches Excel and remains
   a closed-no-action report rather than part of this canonical bug.

### Follow-on Openings
1. `W077`
2. `HO-FN-008`

## Fix Plan
1. keep one shared Excel-style numeric comparison helper for ordinary compare,
   criteria/database numeric criteria, and `SWITCH`, now corrected to the
   empirically pinned truncation-style 15-significant-digit lane,
2. preserve exact-match behavior for `MATCH`, `XMATCH`, and `DELTA`,
3. widen native probe coverage and floating-point documentation to the stronger
   arithmetic-generated boundary rows,
4. reopen current-gap truth surfaces for every family that previously overclaimed
   completion on exact-double semantics,
5. file the OxFunc -> OxFml reply packet with the corrected family split.

## Validation
1. `cargo fmt --manifest-path crates/oxfunc_core/Cargo.toml`
2. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib excel_numeric_compare -- --nocapture`
3. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib operator_compare_concat_family -- --nocapture`
4. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib criteria_family -- --nocapture`
5. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib database_family -- --nocapture`
6. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib misc_switch_info_family -- --nocapture`
7. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib match_fn -- --nocapture`
8. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib xmatch -- --nocapture`
9. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib delta_fn -- --nocapture`
10. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib surface_dispatch -- --nocapture`
11. `powershell -ExecutionPolicy Bypass -File tools/w45-probe/run-w45-waveb-operator-compare-concat-baseline.ps1`
12. `powershell -ExecutionPolicy Bypass -File tools/fp-probe/run-fp-excel-baseline.ps1 -Manifest docs/function-lane/FLOATING_POINT_SCENARIO_MANIFEST_SEED.csv -Out .tmp/fp-results-excel-e.csv -Lanes FP-E`

## Linked Reports
1. `BUGREP-FUNC-005`
2. `BUGREP-FUNC-006`

## Evidence
1. `../OxFml/docs/handoffs/HANDOFF-OXFUNC-003_CORPUS_IF_EMPTY_TEXT_AND_FLOAT_COMPARE.md`
2. `crates/oxfunc_core/src/functions/operator_compare_concat_family.rs`
3. `crates/oxfunc_core/src/functions/criteria_family.rs`
4. `crates/oxfunc_core/src/functions/database_family.rs`
5. `crates/oxfunc_core/src/functions/misc_switch_info_family.rs`
6. `crates/oxfunc_core/src/functions/match_fn.rs`
7. `crates/oxfunc_core/src/functions/xmatch.rs`
8. `crates/oxfunc_core/src/functions/delta_fn.rs`
9. `docs/function-lane/FLOATING_POINT_SCENARIO_MANIFEST_SEED.csv`

## Closure Checklist
- [ ] fix landed or non-OxFunc ownership recorded
- [x] validation recorded
- [x] root cause recorded
- [x] similar-risk scan recorded
- [x] spec/matrix/contract updated if required
- [x] handoff filed if required
- [x] linked reports updated

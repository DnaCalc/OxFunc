# BUG-FUNC-014: XIRR solver-precision drift versus Excel

## Summary
- **Bug id**: `BUG-FUNC-014`
- **Opened**: `2026-04-10`
- **Status**: `validated_local`
- **Owner workset**: `W087`

## Source Refs
- **Reported against ref**: `2e818f03a71ba393690275a7fb437ddd9a6bf760`
- **Reproduced on ref**: `2e818f03a71ba393690275a7fb437ddd9a6bf760`
- **Introduced in ref**: `unknown`
- **Fixed in ref**: `not yet fixed`
- **Ref notes**: direct local replay on 2026-04-10 confirmed a bounded `XIRR`
  precision drift on the current OxFunc ref, while live Excel `Value2` replay
  on the same date pinned the current baseline target.

## Ownership And Root Cause
- **Ownership class**: `OxFunc-owned bug`
- **Root cause class**: `initial_impl_gap`
- **Root cause summary**: the admitted `XIRR` evidence did not pin this exact
  current-baseline witness at Excel `Value2` precision, and the local
  multi-cashflow positive-root path used a tighter secant/Newton solve instead
  of Excel's guess-sensitive bracketed publication behavior.

## Why Did We Get This Wrong?
- **Spec already correct and code was wrong?**: `yes`
- **Spec vague or missing?**: `no`
- **Code once correct and later regressed?**: `no`
- **Likely introduced in ref**: `unknown`
- **Explanation**: the current `cashflow_rate_family.rs` iterative path uses a
  bounded tolerance/iteration policy that is close to Excel on the pinned
  witness but not identical at final solve precision. The earlier evidence
  floor was not strong enough to expose that residual drift.

## Reproduction
1. Direct local replay on 2026-04-10:
   - `XIRR({-10000,2750,4250,3250,2750},{44927,45108,45292,45473,45658})`
     -> `0.24449183218286558`
2. Live Excel `Value2` replay on 2026-04-10:
   - the same formula -> `0.24449183344840997`

## Spec And Contract Relationship
- **Spec references**:
  1. `docs/function-lane/W37_EXECUTION_RECORD.md`
  2. `docs/function-lane/FUNCTION_SLICE_CASHFLOW_RATE_FAMILY_CONTRACT_PRELIM.md`
  3. `docs/function-lane/W51_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_INVENTORY.csv`
- **Spec state at intake**: `correct but incomplete`
- **Notes**: `XIRR` was previously refreshed out of stale snapshot state, but
  this bounded exact witness reopens a narrower solver-precision lane rather
  than claiming a broad family regression.

## Investigation Log
1. 2026-04-10: user asked to confirm whether several small-variation
   differences were real function issues or display/comparison-policy issues.
2. 2026-04-10: direct local replay confirmed the exact current OxFunc `XIRR`
   witness value `0.24449183218286558`.
3. 2026-04-10: live Excel `Value2` replay confirmed the current-baseline target
   `0.24449183344840997`.
4. 2026-04-10: code review located the bounded iterative solve path in
   `cashflow_rate_family.rs` and separated this lane from the earlier `RATE`
   omitted-guess convergence bug.
5. 2026-04-11: expanded live Excel replay showed the reopened witness is
   guess-sensitive on positive guesses (`0.01`, `0.1`, `0.5`, `1.0`) and that
   Excel's returned `Value2` is a bracket midpoint publication observable, not
   the tightest root of the current `XNPV` equation.
6. 2026-04-11: the local general multi-cashflow positive-root path was moved to
   an Excel-like bracket-and-bisection publication policy while preserving the
   earlier two-cashflow `W37` lane and negative-root/negative-guess behavior.
7. 2026-04-11: widened adjacent replay showed the new publication rule matches
   the pinned witness and some neighboring guess/cashflow rows exactly, while
   the packet stayed open pending exact characterization of the wider matrix.
8. 2026-04-11: adjacent `IRR` replay on the same cashflow shape showed only
   tiny `1e-13`-scale guess-sensitive differences and did not justify widening
   `W087` beyond `XIRR`.
9. 2026-04-11: brute-force fitting over the widened `XIRR` matrix showed that
   no single simple current-midpoint `step`, bracket-width, or residual
   threshold explains all observed Excel returned values. The remaining drift
   is therefore narrowed to the exact publication branch shape rather than a
   single missing epsilon tweak.
10. 2026-04-11: exact live Excel `Value2` replay on the widened adjacent matrix
    confirmed that the target values are exact bisection midpoints on the same
    bracket path as the current OxFunc implementation. That rules out bracket
    construction as the remaining mismatch and narrows the open problem to the
    midpoint-publication rule.
11. 2026-04-11: the Microsoft-documented `0.000001 percent` check was tested as
    an `r +/- 1e-8` bracket criterion. It is necessary but not sufficient:
    both some correct Excel values and some still-wrong current OxFunc values
    satisfy that check, so exact parity still requires a stronger empirical
    publication rule.
12. 2026-04-11: further characterization ruled out two additional simple
    publication heuristics:
    - first midpoint whose 15-significant-digit decimal truncation or rounding
      stabilizes,
    - first midpoint whose local Newton correction `|f/f'|` falls below a
      single fixed threshold.
    Both fail on the widened exact `Value2` matrix.
13. 2026-04-11: focused local re-verification reran the exact `XIRR` unit
    tests and the `w87_xirr_adjacent_probe` example against the widened witness
    matrix now pinned in
    `docs/function-lane/W24_BATCH12_CASHFLOW_RATE_SCENARIO_MANIFEST_SEED.csv`.
    The current working-tree implementation matches the full bounded matrix
    locally, so the earlier "still misses some adjacent rows" read no longer
    reproduces on the current tree.

## Similar-Risk Scan
### Adjacent families to check
1. same financial-iteration family:
   - `IRR`
   - `RATE`
   - `XNPV`
2. adjacent `XIRR` input-shape lanes:
   - explicit guess handling
   - two-cashflow special path
   - date spacing / sign-pattern sensitivity

### Check method
1. direct local replay on the pinned current-baseline witness
2. live Excel `Value2` replay on the same witness
3. local code review for shared iterative solver controls and branch paths
4. live Excel replay across explicit positive guesses on the same witness
5. live Excel replay on adjacent multi-cashflow positive-root rows
6. direct local replay on adjacent `IRR` rows with matching cashflow shape
7. local brute-force fitting against candidate midpoint `step`, bracket-width,
   and residual-threshold stop rules

### Results
1. `XIRR` is a real bounded local solver-precision gap against live Excel
   `Value2`, not a display-policy-only difference.
2. `RATE` is an adjacent iterative-family neighbor but is already on its own
   bounded packet (`W081`) with a different failure shape.
3. widened adjacent replay and focused local re-verification now show the
   current working-tree implementation matches the bounded `XIRR`
   multi-cashflow positive-root publication family pinned for `W087`.
4. `IRR` on a matching multi-cashflow vector shows only tiny `1e-13`-scale
   guess-sensitive differences and does not reopen from this packet.
5. the reopened multi-cashflow positive-root lane is guess-sensitive in Excel,
   while the earlier two-cashflow positive-root publication lane remains a
   separate special case already owned by `W37`.
6. the widened matrix rules out a single simple midpoint `step`, width,
   residual, or `r +/- 1e-8` documentation threshold as a complete explanatory
   rule for Excel's publication behavior, even though the current working-tree
   implementation now matches the bounded witness set locally.
7. the widened matrix also rules out simple decimal-stabilization and single
   Newton-correction-threshold publication rules.

### Follow-on Openings
1. `W087`

## Fix Plan
1. keep the bounded widened `XIRR` witness matrix pinned in repo artifacts
2. keep focused regression coverage on the exact witness set
3. reconcile `W051` and related truth surfaces honestly
4. promote the working-tree correction onto a landed ref
5. rerun the broader repo verification lanes once the unrelated OxFml seam test
   compile break is resolved

## Validation
1. focused local Rust tests for `cashflow_rate_family`
2. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib functions::cashflow_rate_family::tests::xirr_simple_positive_root_matches_exact_excel_publication_witnesses -- --exact --nocapture`
3. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib functions::cashflow_rate_family::tests::xirr_two_cashflow_positive_root_matches_excel_guess_matrix -- --exact --nocapture`
4. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib functions::cashflow_rate_family::tests::xirr_general_positive_root_matches_excel_guess_matrix -- --exact --nocapture`
5. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib functions::cashflow_rate_family::tests::xirr_general_positive_root_matches_widened_excel_value2_matrix -- --exact --nocapture`
6. `cargo run --manifest-path crates/oxfunc_core/Cargo.toml --example w87_xirr_adjacent_probe`
7. live Excel `Value2` replay for the pinned widened witness matrix
8. `powershell -ExecutionPolicy Bypass -File scripts/check-worksets.ps1`

## Linked Reports
1. `BUGREP-FUNC-018`

## Evidence
1. `crates/oxfunc_core/src/functions/cashflow_rate_family.rs`
2. `docs/function-lane/W37_EXECUTION_RECORD.md`
3. `docs/worksets/W087_XIRR_SOLVER_PRECISION_RECONCILIATION.md`

## Closure Checklist
- [x] local fix implemented
- [x] validation recorded
- [x] root cause recorded
- [x] similar-risk scan recorded
- [x] spec/matrix/contract updated if required
- [x] linked reports updated
- [ ] handoff filed if required

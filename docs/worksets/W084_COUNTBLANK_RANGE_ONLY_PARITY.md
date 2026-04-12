# WORKSET - COUNTBLANK Range-Only Parity (W084)

## 1. Purpose
Own the bounded local repair where `COUNTBLANK` currently over-admits
array-valued substitutes even though live Excel accepts true ranges and rejects
array-valued substitutes with `#VALUE!`.

## 2. Why This Packet Exists
Live Excel replay on 2026-04-10 narrowed the contract:
1. `=LET(d,{"";1},COUNTBLANK(d)) -> #VALUE!`,
2. `=COUNTBLANK(A1:A3) -> 2` when `A1=""`, `A2=1`, `A3` empty,
3. contrast controls show this is not a blanket aggregate-family change:
   - `COUNT({"2",TRUE}) -> 0`
   - `COUNTA({"";1}) -> 2`
   - `ROWS({1;2;3}) -> 3`
   - `COLUMNS({1,2,3}) -> 3`
4. the reported local ref still expands direct aggregate arguments in
   `countblank_fn.rs`, so the repo needed a bounded owner for the parity repair
   and truth-surface reconciliation.

## 3. Provenance
1. user direction on 2026-04-10
2. live Excel replay on 2026-04-10
3. `docs/bugs/reports/BUGREP-FUNC-015_countblank_rejects_array_valued_substitutes_in_excel.md`
4. `docs/bugs/streams/BUG-FUNC-011_countblank_range_only_parity_gap.md`
5. `crates/oxfunc_core/src/functions/countblank_fn.rs`

## 4. Scope
In scope:
1. record the `COUNTBLANK` parity gap as a canonical bug stream and bounded
   workset,
2. tighten local `COUNTBLANK` surface behavior so array-valued substitutes
   reject with `#VALUE!`,
3. preserve current range semantics for counting empty cells and `""`,
4. add focused regression coverage for the direct-array and true-range lanes,
5. reconcile `W051` and adjacent truth surfaces honestly.

Out of scope:
1. blanket aggregate-family changes to `COUNT` or `COUNTA`,
2. narrowing `ROWS` or `COLUMNS`,
3. reopening `AREAS`, `ISFORMULA`, `FORMULATEXT`, `SUBTOTAL`, or `AGGREGATE`
   unless a direct local mismatch is later found,
4. OxFml parser/binder/display work above the OxFunc function surface.

## 5. Initial Epic Lanes
1. bug intake and owner registration
2. `COUNTBLANK` range-only repair
3. focused validation
4. W51/workset truth reconciliation
5. bounded adjacent policy review framing

## 6. Closure Condition
`W084` is complete for declared scope only when:
1. direct array-valued `COUNTBLANK` inputs reject locally with `#VALUE!`,
2. true range inputs still count empty cells and `""` correctly,
3. focused validation is recorded,
4. `W051` and bug/workset surfaces no longer overclaim `COUNTBLANK`.

## 7. Current Reading
1. execution_state: `in_progress`
2. scope_completeness: `scope_partial`
3. target_completeness: `target_partial`
4. integration_completeness: `integrated`
5. open_lanes:
   - landed-ref promotion for the working-tree `COUNTBLANK` correction
   - honest `W051`/bug-surface promotion on a committed ref
   - bounded adjacent policy review to ensure the fix stays function-local

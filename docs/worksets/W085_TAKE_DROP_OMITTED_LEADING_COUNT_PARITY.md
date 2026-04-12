# WORKSET - TAKE/DROP Omitted Leading-Count Parity (W085)

## 1. Purpose
Own the bounded local repair where `TAKE` and `DROP` currently fail when the
leading row-count argument is syntactically omitted and the third argument is
used to slice columns.

## 2. Why This Packet Exists
`HANDOFF-OXFUNC-004` correctly narrowed the remaining failure after the OxFml
helper-invocation fix:
1. live Excel on 2026-04-10 pinned:
   - `TAKE(array,,n)` as “all rows, slice columns”
   - `DROP(array,,n)` as “all rows, drop columns”
2. direct local OxFunc replay on 2026-04-10 showed on the reported ref:
   - `TAKE(array, MissingArg, 1) -> Preparation(MissingArg)`
   - `DROP(array, MissingArg, 1) -> Preparation(MissingArg)`
3. the earlier `W39` seeded rows covered explicit counts only and therefore
   overclaimed the family closure for omitted-leading-count forms.

## 3. Provenance
1. `../OxFml/docs/handoffs/HANDOFF-OXFUNC-004_TAKE_OMITTED_ROW_COUNT_IN_COMPLEX_LAMBDA.md`
2. direct live Excel replay on 2026-04-10
3. direct local OxFunc replay on 2026-04-10
4. `docs/bugs/streams/BUG-FUNC-012_take_drop_omitted_leading_count_parity_gap.md`
5. `docs/function-lane/W39_EXECUTION_RECORD.md`

## 4. Scope
In scope:
1. record the reopened `TAKE` / `DROP` omission/defaulting gap as a canonical
   bug stream and bounded owner workset,
2. normalize omitted leading row-count handling so the third argument can slice
   columns while all rows are retained,
3. add focused regression tests for the exact omitted-leading-count shape,
4. add replayable W39 witness rows for omitted-leading-count `TAKE` / `DROP`,
5. reconcile `W39`, `W51`, and workset truth honestly.

Out of scope:
1. helper-binding fixes in OxFml already closed upstream,
2. wider reshape-family cleanup beyond directly replayed omitted-leading-count
   lanes,
3. presentation/publication changes above the ordinary OxFunc array result.

## 5. Initial Epic Lanes
1. handoff intake and ownership registration
2. omitted-leading-count repair
3. focused validation
4. W39/W51 truth reconciliation
5. bounded adjacent reshape review framing

## 6. Closure Condition
`W085` is complete for declared scope only when:
1. `TAKE(...,,n)` and `TAKE(...,,-n)` default omitted row count to all rows
   locally,
2. `DROP(...,,n)` and `DROP(...,,-n)` do the same,
3. focused validation is recorded,
4. `W39` and `W51` no longer overclaim `TAKE` / `DROP`.

## 7. Current Reading
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`
5. open_lanes:
   - none

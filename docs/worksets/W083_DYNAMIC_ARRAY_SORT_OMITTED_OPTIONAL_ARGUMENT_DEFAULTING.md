# WORKSET - Dynamic-Array Sort Omitted Optional-Argument Defaulting (W083)

## 1. Purpose
Own the bounded OxFunc-side repair for the reopened sort-family lane where
optional `SORT` / `SORTBY` controls fail when they are syntactically omitted
and therefore arrive on the prepared surface as `MissingArg`.

## 2. Why This Packet Exists
The W39 packet admitted explicit `SORT` / `SORTBY` seeded rows but did not
exercise omission/defaulting through the prepared surface:
1. direct local replay on 2026-04-10 showed `SORT({2;3;7;5},,-1)` fails with
   `Preparation(MissingArg)` and therefore publishes `#VALUE!`,
2. the explicit equivalent `SORT({2;3;7;5},1,-1)` succeeds, so the local gap is
   omitted optional-argument handling rather than sorting semantics,
3. adjacent helper review showed `SORTBY(..., by_array,)` shares the same local
   omitted sort-order path,
4. `W51` and the dynamic-array reshape truth surfaces therefore need a bounded
   current owner rather than stale W39 overclaim.

## 3. Provenance
1. user follow-on on 2026-04-10
2. direct local OxFunc replay on 2026-04-10
3. `docs/bugs/streams/BUG-FUNC-010_dynamic_array_sort_family_omitted_optional_argument_defaulting_gap.md`
4. `docs/function-lane/FUNCTION_SLICE_DYNAMIC_ARRAY_SHAPING_AND_RESHAPING_FAMILY_CONTRACT_PRELIM.md`
5. `docs/function-lane/W39_EXECUTION_RECORD.md`

## 4. Scope
In scope:
1. record the reopened `SORT` intake and its adjacent `SORTBY` helper risk as a
   canonical bug stream and bounded owner workset,
2. normalize omitted optional sort-family controls so `MissingArg` and
   `EmptyCell` default the same way as absent optional arguments,
3. add focused regression coverage for the exact prepared-surface omission
   shape,
4. add a replayable W39 witness row for `SORT({2;3;7;5},,-1)`,
5. reconcile `W051`, W39, and workset truth honestly.

Out of scope:
1. broad dynamic-array reshape optional-default cleanup beyond the sort family,
2. parser/binder changes in OxFml,
3. publication/display adaptation above the ordinary OxFunc array result,
4. claiming that all optional-middle-argument dynamic-array functions have now
   been reviewed.

## 5. Initial Epic Lanes
1. bug intake and ownership registration
2. sort-family omitted-default repair
3. focused validation
4. W39/W51 truth reconciliation
5. adjacent reshape review framing

## 6. Closure Condition
`W083` is complete for declared scope only when:
1. `SORT({2;3;7;5},,-1)` no longer fails locally and matches the explicit
   defaulted control lane,
2. adjacent `SORTBY(..., by_array,)` omission handling is aligned on the same
   helper path,
3. focused validation is recorded,
4. `W051` and the bug/workset surfaces no longer overclaim `SORT` / `SORTBY`.

## 7. Current Reading
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`
5. open_lanes:
   - none

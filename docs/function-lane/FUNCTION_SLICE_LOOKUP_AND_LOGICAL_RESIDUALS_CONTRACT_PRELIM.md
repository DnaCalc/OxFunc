# Function Slice - Lookup And Logical Residuals Contract (Prelim)

Status: `active`
Owner lane: `OxFunc`
Workset: `W068`

## 1. Purpose
Define the current-phase contract for the `W068` final ordinary residual wave: `HLOOKUP`, `IFS`, and `VLOOKUP`.

## 2. Covered Surface
1. `HLOOKUP`
2. `IFS`
3. `VLOOKUP`

## 3. `IFS` Contract
1. `IFS` uses the refs-visible adapter seam because condition/result pairs are scanned lazily rather than flattened eagerly.
2. `IFS` scans `(logical_test, value_if_true)` pairs left-to-right and returns the first branch whose test coerces to true.
3. `IFS` does not force later condition or result branches after an earlier test succeeds.
4. `IFS` treats direct text conditions, including numeric text such as `"2"`, as `#VALUE!`.
5. `IFS` treats empty and missing conditions as false on the admitted current baseline.
6. `IFS` returns `#N/A` when every test is false.
7. `IFS` rejects odd argument counts as a pair-structure error surfaced as `#VALUE!`.

## 4. `HLOOKUP` / `VLOOKUP` Contract
1. `HLOOKUP` and `VLOOKUP` use the refs-visible adapter seam because the table argument preserves reference/array structure through local lookup selection.
2. `HLOOKUP` and `VLOOKUP` publish current-baseline metadata as `lookupMatch` kernels with workbook-state host interaction and `refOnly` dependency profiles.
3. The row/column index argument is numerically coerced, truncated toward zero, and then validated:
   - below `1` -> `#VALUE!`
   - beyond the selected table dimension -> `#REF!`
4. Omitted `range_lookup` defaults to approximate matching.
5. Exact-match lanes (`range_lookup = FALSE`) follow the current-baseline lookup surface, including wildcard support on the seeded text lanes.
6. Approximate-match lanes return `#N/A` when the lookup value falls below the first candidate key.
7. Exact-match misses also return `#N/A`.
8. Returned empty cells publish as numeric zero on the admitted current baseline through the existing cell-to-value projection rule.
9. The current baseline also spills `HLOOKUP` and `VLOOKUP` when
   `lookup_value` is itself an array, preserving the lookup-value shape and
   applying the existing scalar selection semantics per element.

## 5. Runtime / Formal Anchors
Runtime anchors:
1. `crates/oxfunc_core/src/functions/choose_ifs_family.rs`
2. `crates/oxfunc_core/src/functions/vhlookup_family.rs`
3. `crates/oxfunc_core/src/functions/surface_dispatch.rs`

Formal anchors:
1. `formal/lean/OxFunc/Functions/ChooseIfsFamily.lean`
2. `formal/lean/OxFunc/Functions/VhlookupFamily.lean`

Native replay anchors:
1. `docs/function-lane/W68_SCENARIO_MANIFEST_SEED.csv`
2. `tools/w68-probe/run-w68-lookup-logical-baseline.ps1`
3. `.tmp/w68-lookup-logical-results.csv`

Provenance anchors:
1. `docs/function-lane/W16_BATCH43_CHOOSE_IFS_NOTES.md`

## 6. Current-Phase Note
1. `IFS` enters `W068` with existing `W16` empirical/formal grounding and is repinned here through a packet-local replay bundle.
2. `HLOOKUP` and `VLOOKUP` enter `W068` with existing runtime plus Lean metadata grounding and are repinned here through their first dedicated ordinary-backlog closure replay bundle.
3. Live Excel replay on 2026-04-08 later reopened this family for
   array-valued `lookup_value` spill behavior; the local correction is landed on
   `5d54d7f4ab2cdde6458272292d15ae1b317a0fef`. Fresh Excel replay on
   2026-04-29 pinned the adjacent `XLOOKUP` multi-needle return shape, and the
   local `XLOOKUP` correction is validated in the working tree pending
   landed-ref promotion under `BUG-FUNC-006` / `W079`.

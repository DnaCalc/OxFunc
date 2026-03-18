# W16 Batch 43 - CHOOSE and IFS Helper Family

Status: `in_progress-provisional`
Workset: `W16`
Evidence ID: `W16-BATCH43-CHOOSE-IFS-20260316`

## Scope
1. `CHOOSE`
2. `IFS`

## Native Excel Baseline
Probe artifact:
1. `.tmp/w16-batch43-choose-ifs-probe.csv`

## Current Baseline Semantics
1. `CHOOSE` uses a 1-based `index_num` over the trailing choice arguments.
2. `CHOOSE` coerces `index_num` numerically and truncates toward zero before the range check.
3. `CHOOSE` returns `#VALUE!` when the truncated index is less than `1`, greater than the available choice count, or otherwise fails numeric coercion.
4. `CHOOSE` forces only the selected branch; unselected later branches are not prepared.
5. `IFS` scans `(logical_test, value_if_true)` pairs left-to-right and returns the first branch whose test coerces to TRUE.
6. `IFS` treats direct text conditions, including numeric text like `"2"`, as `#VALUE!` rather than truthy numeric coercions in the current baseline.
7. `IFS` returns `#N/A` when every test is false.
8. `IFS` does not force later pairs once an earlier test succeeds.
9. `IFS` rejects odd argument counts as a pair-structure error, surfaced as `#VALUE!`.

## Pinned Lanes For This Helper Slice
1. `CHOOSE(2,10,20,30) -> 20`
2. `CHOOSE(2.9,10,20,30) -> 20`
3. `CHOOSE(0.9,10,20,30) -> #VALUE!`
4. `CHOOSE(4,10,20,30) -> #VALUE!`
5. `CHOOSE("2","a","b") -> "b"`
6. `IFS(FALSE,1,TRUE,2) -> 2`
7. `IFS("2","hit") -> #VALUE!`
8. `IFS(FALSE,1,0,2) -> #N/A`
9. `IFS(TRUE,1,TRUE,1/0) -> 1`
10. `IFS(1/0,1,TRUE,2) -> #DIV/0!`

## Current Implementation Notes
1. The Rust helper module is in `crates/oxfunc_core/src/functions/choose_ifs_family.rs` and is now wired through the shared dispatch/export surfaces.
2. The family uses `refsVisibleInAdapter` metadata because both functions select lazily among branch-like arguments rather than flattening the whole call through a values-only adapter.
3. Selected branch materialization currently follows the existing lazy-helper convention used elsewhere in OxFunc for blank vs missing prepared values:
   - blank selected result becomes numeric zero
   - missing selected result becomes `#VALUE!`
4. Focused unit tests now run through the normal crate test suite.

## Formal Support
1. `formal/lean/OxFunc/Functions/ChooseIfsFamily.lean` records the shared metadata plus executable short-circuit models for:
   - 1-based `CHOOSE` selection over deferred branches
   - left-to-right `IFS` pair scanning with `#N/A` on no match
2. The Lean slice is now imported from the root `OxFunc.lean` surface.

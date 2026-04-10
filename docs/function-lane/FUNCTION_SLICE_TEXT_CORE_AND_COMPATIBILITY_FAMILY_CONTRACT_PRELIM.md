# Function Slice - Text Core And Compatibility Family Contract (Prelim)

Status: `active`
Owner lane: `OxFunc`
Workset: `W066`

## 1. Purpose
Promote the already-evidenced text core and compatibility slice into the ordinary-backlog closure program and the published current-baseline snapshot.

## 2. Covered Surface
1. `CODE`
2. `CONCATENATE`
3. `FIND`
4. `FINDB`
5. `LEFT`
6. `LEFTB`
7. `LEN`
8. `LENB`
9. `LOWER`
10. `MID`
11. `MIDB`
12. `PROPER`
13. `REPLACE`
14. `REPLACEB`
15. `REPT`
16. `RIGHT`
17. `RIGHTB`
18. `SEARCH`
19. `SEARCHB`
20. `SUBSTITUTE`
21. `TRIM`
22. `UNICODE`
23. `UPPER`

## 3. Admitted Current-Baseline Slice
1. all rows use the ordinary values-only preparation seam.
2. text length and indexing are characterized against UTF-16-unit behavior for the current Excel baseline.
3. `LEFT` and `RIGHT` preserve whole surrogate pairs at the one-character boundary observed in the current baseline.
4. `MID`, `FIND`, `SEARCH`, `REPLACE`, and the `*B` delegates operate over one-based UTF-16-unit positions on the admitted baseline.
5. `SEARCH` wildcard semantics are in scope for the current ASCII-seeded baseline (`*`, `?`, `~`).
6. the `*B` compatibility rows are admitted as current-baseline delegates to the non-`B` text functions rather than a separate historical DBCS-byte-semantic claim.
7. `UNICODE` returns the first Unicode scalar, not merely the first raw UTF-16 unit, and rejects invalid leading surrogate structures.
8. `TRIM` collapses ASCII spaces but preserves non-breaking spaces on the admitted slice.
9. `LEFT`, `RIGHT`, and `MID` spill over a single array-valued count/start
   input on the current baseline, and the current-baseline `LEFTB`, `RIGHTB`,
   and `MIDB` delegates inherit the same working-tree correction.
10. `CHAR`, `CODE`, `LOWER`, `UPPER`, `TRIM`, and `REPT` spill over a single
    array-valued admitted argument on the current baseline for the lanes pinned
    under `W080`.
11. `TEXTAFTER` and `TEXTBEFORE` spill over a single array-valued `text` or
    `instance_num` argument on the current baseline for the lanes pinned under
    `W080`; delimiter-array widening is not admitted from the current evidence.

## 4. Main Rules Carried Into W066
1. `CODE` rejects empty text with `#VALUE!` and reads only the first UTF-16 code unit.
2. `LOWER` and `UPPER` textify logicals on the ordinary values-only seam.
3. `REPT` truncates `number_times` toward zero, rejects negatives with `#VALUE!`, and enforces Excel's `32767` UTF-16-unit result ceiling.
4. `CONCATENATE` remains scalar-only and rejects multi-cell ranges with `#VALUE!`.
5. `LEN` counts UTF-16 code units on the current baseline.
6. `LEFT`, `RIGHT`, and `MID` truncate numeric count/position arguments toward zero before domain checks.
7. `PROPER` follows the current baseline word-boundary behavior where non-letter separators, including apostrophes and digits, restart capitalization.
8. `SUBSTITUTE` leaves the source text unchanged when `old_text` is empty and rejects `instance_num < 1` with `#VALUE!`.
9. `FIND` is case-sensitive and treats wildcard characters literally.
10. `SEARCH` is case-insensitive on the current ASCII-seeded baseline and honors `*`, `?`, and `~`.
11. `FINDB`, `LEFTB`, `LENB`, `MIDB`, `REPLACEB`, `RIGHTB`, and `SEARCHB` match the current Unicode-baseline delegate posture documented in `W16_BATCH73`.
12. `CHAR`, `CODE`, `LOWER`, `UPPER`, and `TRIM` now have current-baseline
    evidence for spill over a single array-valued primary argument.
13. `REPT` now has current-baseline evidence for spill over one array-valued
    text or `number_times` argument.
14. `TEXTAFTER` and `TEXTBEFORE` now have current-baseline evidence for spill
    over one array-valued `text` or `instance_num` argument, but delimiter-array
    widening remains outside the admitted slice.
15. The broader function-array-support question remains open beyond the seeded
    text-slice and first bounded text-scalar/text-delimiter lanes and is now
    explicitly owned under `BUG-FUNC-007` / `BUG-FUNC-008` / `W080` rather than
    assumed closed.

## 5. Runtime / Formal Anchors
Runtime anchors:
1. `crates/oxfunc_core/src/functions/text_scalar_misc.rs`
2. `crates/oxfunc_core/src/functions/concat_family.rs`
3. `crates/oxfunc_core/src/functions/text_slice_family.rs`
4. `crates/oxfunc_core/src/functions/text_search_replace_family.rs`
5. `crates/oxfunc_core/src/functions/text_b_compat_family.rs`
6. `crates/oxfunc_core/src/functions/text_unicode_fn.rs`
7. `crates/oxfunc_core/src/functions/surface_dispatch.rs`

Formal anchors:
1. `formal/lean/OxFunc/Functions/CodeFn.lean`
2. `formal/lean/OxFunc/Functions/LowerFn.lean`
3. `formal/lean/OxFunc/Functions/UpperFn.lean`
4. `formal/lean/OxFunc/Functions/TrimFn.lean`
5. `formal/lean/OxFunc/Functions/ReptFn.lean`
6. `formal/lean/OxFunc/Functions/ConcatFamily.lean`
7. `formal/lean/OxFunc/Functions/TextSliceFamily.lean`
8. `formal/lean/OxFunc/Functions/TextSearchReplaceFamily.lean`
9. `formal/lean/OxFunc/Functions/TextBCompatFamily.lean`
10. `formal/lean/OxFunc/Functions/TextUnicodeFn.lean`

Native replay anchors:
1. `docs/function-lane/W66_SCENARIO_MANIFEST_SEED.csv`
2. `tools/w66-probe/run-w66-text-core-compat-baseline.ps1`
3. `.tmp/w66-text-core-compat-results.csv`

Provenance anchors:
1. `docs/function-lane/W16_BATCH31_TEXT_SCALAR_MISC_NOTES.md`
2. `docs/function-lane/W16_BATCH35_TEXT_UNICODE_NOTES.md`
3. `docs/function-lane/W16_BATCH36_TEXT_SLICE_NOTES.md`
4. `docs/function-lane/W16_BATCH40_HELPER_CONCAT_NOTES.md`
5. `docs/function-lane/W16_BATCH42_TEXT_SEARCH_REPLACE_NOTES.md`
6. `docs/function-lane/W16_BATCH73_TEXT_B_COMPAT_NOTES.md`

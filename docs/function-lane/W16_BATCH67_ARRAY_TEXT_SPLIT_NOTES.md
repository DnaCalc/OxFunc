# W16 Batch 67 - Array Text Split Family

Status: `packet-evidenced`
Workset: `W16`

## Scope
1. `ARRAYTOTEXT`
2. `TEXTSPLIT`

## Feasibility Split
1. `ARRAYTOTEXT` is the cleaner half of this pair in a self-contained pass because it only needs scalar/array materialization plus deterministic rendering.
2. `TEXTSPLIT` is feasible for the primary scalar baseline, but the full Excel surface is broader because it publishes dynamic arrays and has deeper delimiter, collation, and spill-host lanes.

## Current Implementation Notes
1. This family now participates in shared dispatch, export-spec, and root Lean import surfaces; the earlier self-contained wording is obsolete.
2. `ARRAYTOTEXT` currently covers:
   - scalar inputs promoted to `1x1` arrays
   - concise mode `0`
   - strict mode `1`
   - row-major concise rendering
   - strict rendering with braces, row separators, and quoted text cells
3. `TEXTSPLIT` currently covers the admitted scalar baseline:
   - column delimiter only
   - row + column delimiter split
   - multi-delimiter arrays
   - `ignore_empty`
   - ASCII-only case-insensitive `match_mode = 1`
   - default `#N/A` padding and explicit `pad_with`
4. The current `TEXTSPLIT` slice is intentionally narrower than full Excel parity. `W24` Batch 04 now evidences the admitted slice through scalar `ARRAYTOTEXT(TEXTSPLIT(...),1)` witnesses while still leaving broader spill-publication and richer collation lanes outside the closure claim.

## Seeded Unit Lanes
1. `ARRAYTOTEXT({TRUE,#VALUE!;"Hello",2},0) -> "TRUE, #VALUE!, Hello, 2"`
2. `ARRAYTOTEXT({TRUE,#VALUE!;"Hello",2},1) -> "{TRUE,#VALUE!;""Hello"",2}"`
3. `TEXTSPLIT("Dakota Lennon Sanchez"," ") -> {"Dakota","Lennon","Sanchez"}`
4. `TEXTSPLIT("1,2,3;4,5",",",";") -> 2x3 with default #N/A padding`
5. `TEXTSPLIT("Do. Or do not. There is no try. -Anonymous",{".","-"},,TRUE) -> 1x5`


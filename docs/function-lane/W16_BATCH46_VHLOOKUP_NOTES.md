# W16 Batch 46 - VLOOKUP / HLOOKUP Family

Status: `in_progress-provisional`
Workset: `W16`
Evidence ID: `W16-BATCH46-VHLOOKUP-20260316`

## Scope
1. `VLOOKUP`
2. `HLOOKUP`

## Native Excel Baseline
Probe artifacts:
1. `.tmp/w16-batch46-vhlookup-probe.csv`
2. `.tmp/probe_vhlookup_edges.ps1`

Pinned lanes:
1. `VLOOKUP(2,{1,10;2,20;3,30},2,FALSE) -> 20`
2. `VLOOKUP(2.9,{1,10;2,20;3,30},2,TRUE) -> 20`
3. `VLOOKUP(0.5,{1,10;2,20;3,30},2,TRUE) -> #N/A`
4. `VLOOKUP(4,{1,10;2,20;3,30},2,FALSE) -> #N/A`
5. `VLOOKUP(2,{1,10;2,20;3,30},0,FALSE) -> #VALUE!`
6. `VLOOKUP("b*",{"abc",1;"bcd",2},2,FALSE) -> 2`
7. `HLOOKUP(2,{1,2,3;10,20,30},2,FALSE) -> 20`
8. `HLOOKUP(2.9,{1,2,3;10,20,30},2,TRUE) -> 20`
9. `HLOOKUP(0.5,{1,2,3;10,20,30},2,TRUE) -> #N/A`
10. `HLOOKUP(4,{1,2,3;10,20,30},2,FALSE) -> #N/A`
11. `HLOOKUP(2,{1,2,3;10,20,30},0,FALSE) -> #VALUE!`
12. `HLOOKUP("b*",{"abc","bcd";1,2},2,FALSE) -> 2`

## Current Implementation Notes
1. The slice reuses the existing `MATCH` surface for exact and approximate lookup over the first column or first row of the supplied table.
2. `col_index_num` / `row_index_num` truncates toward zero.
3. Result indexes less than `1` map to `#VALUE!`; indexes beyond the table width or height map to `#REF!`.
4. Approximate lookup below the first key maps to `#N/A`.
5. Exact `FALSE` lookup inherits the current `MATCH` wildcard behavior for text needles.
6. Returned true blank cells remain numeric `0`; text-empty cells remain text-empty.

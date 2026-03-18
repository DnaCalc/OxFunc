# Handoff - W015 CELL and INFO Host-Query Seam to OxFml/FEC

Handoff id: `HO-FN-002`  
Direction: `OxFunc -> OxFml`  
Status: `acknowledged`  
Source workset: `W015`

## 1. Purpose
Record the current OxFunc seam requirement for `CELL` and `INFO`.

The key point is not “OxFunc needs more parse tree”.
The key point is “OxFunc needs typed host facts for information functions”.

## 2. Current OxFunc Position
`CELL` and `INFO` should be modeled as:
1. OxFunc-owned query semantics
2. over typed upstream host-query facilities

not as:
1. pure local kernels
2. arbitrary stringly evaluator callbacks
3. workbook-object access embedded directly in OxFunc

## 3. Requested Upstream Support
Please leave room for a typed capability surface that can answer:
1. cell metadata queries keyed by preserved reference identity
2. workbook information queries
3. application/environment information queries

Candidate logical shape:
1. `CellInfoQuery`
2. `InfoQuery`
3. typed provider/facility methods rather than parse-tree or workbook-object leakage

## 4. Why This Matters
Observed W15 baseline on `2026-03-15`:
1. `INFO("directory")`, `numfile`, `origin`, `osversion`, `recalc`, `release`, and `system` return concrete host/workbook facts
2. `INFO("memavail")`, `memused`, and `totmem` return `#N/A` on the current host

So these functions cannot be completed honestly by OxFunc without upstream host facts.

## 5. Function Split
`CELL` is mixed:
1. local/reference-based:
   - `address`
   - `row`
   - `col`
   - `contents`
   - `type`
2. host-query:
   - `filename`
   - `format`
   - `color`
   - `prefix`
   - `protect`
   - `width`

`INFO` is mostly host-query.

## 6. Current Local Artifacts
1. `docs/worksets/W015_CELL_AND_INFO_HOST_QUERY_FUNCTIONS.md`
2. `docs/function-lane/CELL_INFO_HOST_QUERY_SEAM_PRELIM.md`
3. `docs/function-lane/FUNCTION_SLICE_INFO_CONTRACT_PRELIM.md`
4. `crates/oxfunc_core/src/host_info.rs`
5. `crates/oxfunc_core/src/functions/info_fn.rs`
6. `formal/lean/OxFunc/HostInfoSeam.lean`
7. `formal/lean/OxFunc/Functions/Info.lean`

## 6.1 Updated Local State
Local W15 replay is now green for the admitted current-baseline slice:
1. dual-run workbook replay (`default` + `compat_template`) is captured in `.tmp/w15-info-pre-results*.csv`, `.tmp/w15-cell-host-pre-results*.csv`, and `.tmp/w15-xll-bridge-results*.csv`
2. generated `ox_INFO(...)` matches native `INFO(...)` for all ten seeded lanes on both workbook descriptors
3. generated `ox_CELL(...)` matches native `CELL(...)` for:
   - explicit-reference host lanes,
   - cross-sheet `filename` / `format`,
   - omitted-reference active-selection lanes across the admitted info-type set,
   - the native two-item `width` artifact (`INDEX(...,2)` and `COLUMNS(...)`)

## 7. Ask
Please acknowledge whether the upstream interface can carry a typed host-query capability for these functions, or whether OxFml/FEC wants a different transport shape that still preserves the same semantic split.

## 8. Upstream Acknowledgment
OxFml has now acknowledged this handoff on `2026-03-15` in:
1. `../OxFml/docs/upstream/NOTES_FOR_OXFUNC.md`
2. `../OxFml/docs/handoffs/HANDOFF_REGISTER.csv`

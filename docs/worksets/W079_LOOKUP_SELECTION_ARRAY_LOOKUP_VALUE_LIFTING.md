# WORKSET - Lookup-Family Array Lookup-Value Lifting (W079)

## 1. Purpose
Own the bounded OxFunc-side correction for the lookup-family lane where live
Excel spills array-valued `lookup_value` inputs for `XMATCH`, `MATCH`,
`VLOOKUP`, `HLOOKUP`, and adjacent `XLOOKUP`, while the current local surface
rejects or mishandles those lanes.

## 2. Why This Packet Exists
The earlier lookup-family packets closed on scalar lookup-value evidence and did
not pin dynamic-array needle behavior:
1. live Excel on 2026-04-08 confirmed `XMATCH({1,2,3},...)` and
   `MATCH({1,2,3},...,0)` spill one result per lookup-value element,
2. the composed `FILTER + ISNUMBER + XMATCH` set-intersection formula returns
   `6` in live Excel,
3. live Excel also confirmed `VLOOKUP({1,2,3},...)` and `HLOOKUP({1,2,3},...)`
   spill one result per lookup-value element,
4. local `XMATCH` and `MATCH` both still prepared `lookup_value` as scalar-only
   and local `VLOOKUP` / `HLOOKUP` still assumed the shared match result was
   scalar-only, so the family surfaced `#VALUE!` or equivalent scalar-only
   failure instead of elementwise `#N/A` / index results.
5. fresh Excel replay on 2026-04-29 confirmed adjacent `XLOOKUP` preserves the
   array-valued `lookup_value` shape, applies top-left fallback values per
   missing needle, and scalarizes matrix return selections to the first cell of
   the selected row or column for multi-needle lookup.

## 3. Provenance
1. user-reported downstream corpus issue on 2026-04-08
2. live Excel COM replay on 2026-04-08
3. `docs/bugs/streams/BUG-FUNC-006_lookup_selection_array_lookup_value_lifting_gap.md`
4. `docs/function-lane/XMATCH_EXECUTION_RECORD.md`
5. `docs/function-lane/W10_EXECUTION_RECORD.md`
6. `docs/function-lane/FUNCTION_SLICE_LOOKUP_AND_LOGICAL_RESIDUALS_CONTRACT_PRELIM.md`

## 4. Scope
In scope:
1. record the inbound XMATCH issue and the widened local lookup-family probe as
   bug reports plus one canonical bug stream,
2. correct local `XMATCH` and `MATCH` so array-valued `lookup_value` spills
   per-element results using the existing scalar semantics for each element,
3. correct local `VLOOKUP` and `HLOOKUP` so array-valued `lookup_value` spills
   per-element selected-cell results using the existing scalar semantics for
   each element,
4. add focused Rust and surface-dispatch tests for the spilled lookup-family
   lanes,
5. reopen stale current-gap and execution-record truth for `XMATCH`, `MATCH`,
   `VLOOKUP`, `HLOOKUP`, and adjacent `XLOOKUP` risk,
6. correct the adjacent `XLOOKUP` return-shape follow-on after fresh replay
   confirms the exact multi-needle selection rule.

Out of scope:
1. new XLL export-policy widening beyond the existing seam limits,
2. broader locale/channel sweeps beyond the current installed Excel baseline,
3. OxFml parse/bind changes unless a new seam blocker is found.

## 5. Initial Epic Lanes
1. bug intake and family ownership registration
2. live Excel replay and adjacent-family scan
3. local `XMATCH` / `MATCH` / `VLOOKUP` / `HLOOKUP` / `XLOOKUP` surface correction
4. focused validation
5. current-gap and execution-record truth reconciliation

## 6. Closure Condition
`W079` is complete for declared scope only when:
1. `XMATCH`, `MATCH`, `VLOOKUP`, `HLOOKUP`, and adjacent `XLOOKUP` all spill
   array-valued `lookup_value` lanes locally,
2. focused validation is recorded,
3. current-gap and execution records no longer overclaim the lookup family,
4. adjacent `XLOOKUP` landed-ref promotion state is recorded explicitly rather
   than hidden behind stale closure language.

## 7. Current Reading
1. execution_state: `closed`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`
5. open_lanes: none
6. landed lanes:
   - `XMATCH`, `MATCH`, `VLOOKUP`, and `HLOOKUP` array-valued
     `lookup_value` corrections are landed on
     `5d54d7f4ab2cdde6458272292d15ae1b317a0fef`
   - `XLOOKUP` array-valued `lookup_value` correction is landed on
     `b1faa5e8f08cd534601dc57bf79a9fed3ff26972`
   - focused validation replayed on 2026-04-29:
     `xmatch_surface`, `match_fn`, `vhlookup_family`, `xlookup`, and
     `surface_dispatch`
7. XLOOKUP shape note:
   - fresh Excel COM replay on 2026-04-29 confirmed `XLOOKUP` preserves
     array-valued `lookup_value` shape, uses top-left `if_not_found` fallback
     values per missing needle, and scalarizes matrix return selections to the
     first cell of the selected row or column for multi-needle lookup

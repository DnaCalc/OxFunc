# WORKSET - Function Array Support Review (W080)

## 1. Purpose
Own the bounded OxFunc-side seed for systematic function array-support review,
starting with the immediate text-slice correction for `LEFT`, `RIGHT`, and
`MID`, then carrying that learning into an explicit review program for other
ordinary functions whose scalar parameters may spill under the current Excel
baseline.

## 2. Why This Packet Exists
Recent local bug work reopened multiple functions because array-valued scalar
inputs were never probed explicitly:
1. `MATCH`, `XMATCH`, `VLOOKUP`, and `HLOOKUP` all turned out to spill over an
   array-valued `lookup_value`,
2. live Excel on 2026-04-08 also confirmed
   `MID("MISSISSIPPI",SEQUENCE(11),1)`, `LEFT("MISSISSIPPI",SEQUENCE(3))`, and
   `RIGHT("MISSISSIPPI",SEQUENCE(3))` spill rather than reject,
3. OxFunc currently has no bounded owner for a systematic array-support review
   across the broader supported surface, so each discovery would otherwise land
   as an isolated surprise bug.

## 3. Provenance
1. user follow-on review on 2026-04-08
2. live Excel COM replay on 2026-04-08
3. `docs/bugs/streams/BUG-FUNC-007_text_slice_array_position_and_count_spill_gap.md`
4. `docs/function-lane/FUNCTION_SLICE_TEXT_CORE_AND_COMPATIBILITY_FAMILY_CONTRACT_PRELIM.md`
5. `docs/worksets/W079_LOOKUP_SELECTION_ARRAY_LOOKUP_VALUE_LIFTING.md`

## 4. Scope
In scope:
1. record the text-slice family spill finding as a canonical bug stream,
2. correct the immediate seed rows `LEFT` / `LEFTB`, `MID` / `MIDB`, and
   `RIGHT` / `RIGHTB`,
3. reopen current-gap and contract truth so those rows are not overclaimed,
4. define the bounded review owner for broader function array-support review,
5. capture the next-family scan shape explicitly rather than leaving it as chat
   intent.

Out of scope:
1. a full 517-function array-support implementation sweep in one pass,
2. speculative automation over the entire function surface,
3. cross-repo handoff unless a later review packet identifies a seam issue,
4. unrelated function-semantic changes outside the immediate seeded families.

## 5. Initial Epic Lanes
1. text-slice bug intake and ownership registration
2. immediate `LEFT` / `RIGHT` / `MID` runtime correction
3. focused validation
4. current-gap and contract reconciliation
5. broader array-support review framing and next-batch sequencing

## 6. Closure Condition
`W080` is complete for declared scope only when:
1. the immediate text-slice seed correction is validated locally,
2. `W051` and the bug/workset surfaces no longer overclaim those rows,
3. the broader function-array-support review has an explicit bounded owner and
   a concrete next-batch review shape,
4. no claim is made that the full supported surface has already been reviewed.

## 7. Current Reading
1. execution_state: `in_progress`
2. scope_completeness: `scope_partial`
3. target_completeness: `target_partial`
4. integration_completeness: `integrated`
5. open_lanes:
   - landed-ref promotion for the local `LEFT` / `RIGHT` / `MID` spill correction
   - systematic family-by-family review beyond the initial text-slice seed

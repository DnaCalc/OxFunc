# WORKSET - Function Array Support Review (W080)

## 1. Purpose
Own the bounded OxFunc-side seed for systematic function array-support review:
the immediate text-slice correction for `LEFT`, `RIGHT`, and `MID`, plus two
bounded adjacent text-family follow-on batches. The broader supported-surface
array sweep is owned by successor workset `W090`.

## 2. Why This Packet Exists
Recent local bug work reopened multiple functions because array-valued scalar
inputs were never probed explicitly:
1. `MATCH`, `XMATCH`, `VLOOKUP`, and `HLOOKUP` all turned out to spill over an
   array-valued `lookup_value`,
2. live Excel on 2026-04-08 also confirmed
   `MID("MISSISSIPPI",SEQUENCE(11),1)`, `LEFT("MISSISSIPPI",SEQUENCE(3))`, and
   `RIGHT("MISSISSIPPI",SEQUENCE(3))` spill rather than reject,
3. the initial seed needed a bounded successor owner for systematic
   array-support review across the broader supported surface, so each discovery
   would not land as an isolated surprise bug,
4. live Excel COM replay on 2026-04-09 widened the first bounded follow-on
   batch to `CHAR`, `CODE`, `LOWER`, `UPPER`, `TRIM`, `REPT`, `TEXTAFTER`, and
   `TEXTBEFORE`.

## 3. Provenance
1. user follow-on review on 2026-04-08
2. live Excel COM replay on 2026-04-08
3. `docs/bugs/streams/BUG-FUNC-007_text_slice_array_position_and_count_spill_gap.md`
4. `docs/function-lane/FUNCTION_SLICE_TEXT_CORE_AND_COMPATIBILITY_FAMILY_CONTRACT_PRELIM.md`
5. `docs/worksets/W079_LOOKUP_SELECTION_ARRAY_LOOKUP_VALUE_LIFTING.md`
6. `docs/bugs/streams/BUG-FUNC-008_text_scalar_and_delimiter_array_support_gap.md`
7. `docs/worksets/W090_FUNCTION_ARRAY_SUPPORT_SYSTEMATIC_SWEEP.md`

## 4. Scope
In scope:
1. record the text-slice family spill finding as a canonical bug stream,
2. correct the immediate seed rows `LEFT` / `LEFTB`, `MID` / `MIDB`, and
   `RIGHT` / `RIGHTB`,
3. execute the first and second bounded follow-on batches for adjacent text
   rows,
4. reopen current-gap and contract truth so those rows are not overclaimed,
5. define successor `W090` as the bounded owner for broader function
   array-support review,
6. capture the next-family scan shape explicitly through that successor rather
   than leaving it as chat intent.

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
6. first bounded text-scalar and text-delimiter review batch
7. second bounded text search/replace review batch

## 6. Closure Condition
`W080` is complete for declared scope only when:
1. the immediate text-slice seed correction is validated locally,
2. `W051` and the bug/workset surfaces no longer overclaim those rows,
3. at least one bounded post-text-slice batch is replayed and reconciled
   honestly,
4. the broader function-array-support review has an explicit bounded owner and
   a concrete next-batch review shape,
5. no claim is made that the full supported surface has already been reviewed.

## 7. Successor Sweep Owner
`W090` owns the broader systematic family-by-family array-support sweep after
this seed packet. `W080` records the seed corrections and bounded follow-on
batches only; it does not claim that the full supported surface has been
reviewed.

## 8. First Bounded Batch (2026-04-09)
1. confirmed live Excel spill lanes:
   - `CHAR(SEQUENCE(3)+64)`
   - `CODE({"A","B"})`
   - `LOWER({"A","B"})`
   - `UPPER({"a","b"})`
   - `TRIM({"  a  "," b "})`
   - `REPT("x",SEQUENCE(3))`
   - `REPT({"a","b"},2)`
   - `TEXTAFTER("a-b-c","-",SEQUENCE(3))`
   - `TEXTBEFORE("a-b-c","-",SEQUENCE(3))`
   - `TEXTAFTER({"a-b","c-d"},"-")`
   - `TEXTBEFORE({"a-b","c-d"},"-")`
2. non-lift note from the same replay:
   - `TEXTAFTER("a-b,c-d",{"-",","})` did not open an obvious spill lane in the
     simple probe, so delimiter-array widening remains out of scope.
3. first-batch bug owner:
   - `BUG-FUNC-008`

## 9. Second Bounded Batch (2026-04-29)
1. confirmed live Excel spill lanes:
   - `FIND({"a","b"},"abc")`
   - `FIND("a",{"abc","bca"})`
   - `FIND("a","abc",SEQUENCE(3))`
   - `SEARCH({"a","b"},"abc")`
   - `SEARCH("A",{"abc","bca"})`
   - `REPLACE({"abc","def"},2,1,"Z")`
   - `REPLACE("abc",SEQUENCE(3),1,"Z")`
   - `REPLACE("abc",2,SEQUENCE(3),"Z")`
   - `REPLACE("abc",2,1,{"X","Y"})`
   - `PROPER({"hello world","o'brien"})`
   - `SUBSTITUTE({"foo bar","bar foo"},"foo","x")`
   - `SUBSTITUTE("foo bar foo",{"foo","bar"},"x")`
   - `SUBSTITUTE("foo bar foo","foo",{"x","y"})`
   - `SUBSTITUTE("foo foo","foo","x",SEQUENCE(3))`
2. second-batch bug owner:
   - `BUG-FUNC-016`
3. local correction:
   - implemented bounded single-array lift for the confirmed scalar positions
     in `text_search_replace_family`
   - `FINDB`, `SEARCHB`, and `REPLACEB` inherit the correction through their
     current-baseline delegates

## 10. Current Reading
1. execution_state: `closed`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`
5. open_lanes:
   - none for the declared `W080` seed scope
6. successor lanes:
   - `W090` owns broader systematic family-by-family review beyond the initial
     seeded batches
7. landed lanes:
   - `LEFT` / `LEFTB`, `MID` / `MIDB`, and `RIGHT` / `RIGHTB` text-slice
     correction landed on `5d54d7f4ab2cdde6458272292d15ae1b317a0fef`
   - `CHAR`, `CODE`, `LOWER`, `UPPER`, `TRIM`, `REPT`, `TEXTAFTER`, and
     `TEXTBEFORE` batch-A correction landed on
     `2e818f03a71ba393690275a7fb437ddd9a6bf760`
   - `FIND` / `FINDB`, `SEARCH` / `SEARCHB`, `REPLACE` / `REPLACEB`,
     `PROPER`, and `SUBSTITUTE` batch-B correction landed on
     `b1faa5e8f08cd534601dc57bf79a9fed3ff26972`
   - focused validation replayed on 2026-04-29:
     `text_slice_family`, `text_scalar_misc`, `text_delim_family`, and
     `surface_dispatch`

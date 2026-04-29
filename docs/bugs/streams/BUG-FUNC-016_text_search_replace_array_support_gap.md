# BUG-FUNC-016: Text search/replace scalar parameter array-support gap

## Summary
- **Bug id**: `BUG-FUNC-016`
- **Opened**: 2026-04-29
- **Status**: `closed`
- **Owner workset**: `W080`

## Source Refs
- **Reported against ref**: `3700e434de7983dfccc9cefa7f136de45e35ea2c`
- **Reproduced on ref**: `3700e434de7983dfccc9cefa7f136de45e35ea2c`
- **Introduced in ref**: `unknown`
- **Fixed in ref**: `b1faa5e8f08cd534601dc57bf79a9fed3ff26972`
- **Ref notes**: W080 next-batch review found this on the current local read.
  Fresh Excel COM replay on 2026-04-29 against Excel 16.0 build 19929 showed
  the tested single-array scalar-parameter lanes spill. The local correction is
  landed on `b1faa5e8f08cd534601dc57bf79a9fed3ff26972`.

## Ownership And Root Cause
- **Ownership class**: `OxFunc-owned bug`
- **Root cause class**: `test_gap`
- **Root cause summary**: the earlier W066 text search/replace packet admitted
  scalar semantics for `FIND`, `SEARCH`, `REPLACE`, `PROPER`, and `SUBSTITUTE`,
  but did not probe array-valued scalar parameters. The shared
  `text_search_replace_family` surface therefore kept scalar-only prepared
  coercion while adjacent W080 batches had already proven Excel spills similar
  text scalar lanes.

## Reproduction
Fresh Excel COM replay on 2026-04-29 observed:
1. `ARRAYTOTEXT(FIND({"a","b"},"abc"),1) -> {1,2}`
2. `ARRAYTOTEXT(FIND("a",{"abc","bca"}),1) -> {1,3}`
3. `ARRAYTOTEXT(FIND("a","abc",SEQUENCE(3)),1) -> {1;#VALUE!;#VALUE!}`
4. `ARRAYTOTEXT(SEARCH({"a","b"},"abc"),1) -> {1,2}`
5. `ARRAYTOTEXT(SEARCH("A",{"abc","bca"}),1) -> {1,3}`
6. `ARRAYTOTEXT(REPLACE("abc",SEQUENCE(3),1,"Z"),1) -> {"Zbc";"aZc";"abZ"}`
7. `ARRAYTOTEXT(PROPER({"hello world","o'brien"}),1) -> {"Hello World","O'Brien"}`
8. `ARRAYTOTEXT(SUBSTITUTE("foo foo","foo","x",SEQUENCE(3)),1) -> {"x foo";"foo x";"foo foo"}`

Pre-fix local structural behavior:
1. `eval_find_surface` and `eval_search_surface` scalar-coerced
   `find_text`, `within_text`, and `start_num`.
2. `eval_replace_surface` scalar-coerced `old_text`, `start_num`,
   `num_chars`, and `new_text`.
3. `eval_proper_surface` and `eval_substitute_surface` scalar-coerced their
   text and optional instance arguments.
4. array-valued arguments therefore surfaced `#VALUE!` instead of preserving
   the input array shape and applying scalar semantics per element.

## Similar-Risk Scan
The issue is the bounded W080 second text batch. `FINDB`, `SEARCHB`, and
`REPLACEB` delegate to the corrected Unicode-baseline implementations, so the
same repair covers those current-baseline compatibility rows. Multiple-array
broadcast behavior remains outside this bounded pass and should not be inferred
from the single-array evidence.

## Fix Plan
1. add the same bounded single-array lift pattern used by the first W080 batch
   to `text_search_replace_family`,
2. allow one array-valued argument in empirically confirmed scalar positions,
3. preserve existing scalar behavior for each element, including per-element
   `#VALUE!` publication,
4. add focused module tests for all probed lanes.

## Validation
1. 2026-04-29 fresh Excel COM replay for the rows above
2. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib text_search_replace_family -- --nocapture`
   - 13 passed
3. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib text_b_compat_family -- --nocapture`
   - 3 passed
4. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib surface_dispatch -- --nocapture`
   - 75 passed

## Evidence
1. `crates/oxfunc_core/src/functions/text_search_replace_family.rs`
2. `crates/oxfunc_core/src/functions/text_b_compat_family.rs`
3. `docs/function-lane/W66_SCENARIO_MANIFEST_SEED.csv`
4. `docs/worksets/W080_FUNCTION_ARRAY_SUPPORT_REVIEW.md`

## Closure Checklist
- [x] fix landed or non-OxFunc ownership recorded
- [x] validation recorded
- [x] root cause recorded
- [x] similar-risk scan recorded
- [x] spec/matrix/contract updated if required
- [x] handoff filed if required

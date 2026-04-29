# BUG-FUNC-008: Text scalar and delimiter array-support gap

## Summary
- **Bug id**: `BUG-FUNC-008`
- **Opened**: 2026-04-09
- **Status**: `closed`
- **Owner workset**: `W080`

## Source Refs
- **Reported against ref**: `5d54d7f4ab2cdde6458272292d15ae1b317a0fef`
- **Reproduced on ref**: `5d54d7f4ab2cdde6458272292d15ae1b317a0fef`
- **Introduced in ref**: `unknown`
- **Fixed in ref**: `2e818f03a71ba393690275a7fb437ddd9a6bf760`
- **Ref notes**: intake pinned the current committed local ref on 2026-04-09.
  Live Excel COM replay on 2026-04-09 widened the `W080` review beyond the
  earlier text-slice seed and showed that several ordinary text-core and
  delimiter rows spill on the current baseline where the local values-only seam
  was still scalar-only. The bounded batch-A correction is landed on the fixed
  ref above; broader W066 review remains open separately under `W080`.

## Ownership And Root Cause
- **Ownership class**: `OxFunc-owned bug`
- **Root cause class**: `test_gap`
- **Root cause summary**: the earlier `W066` text-core packet exercised scalar
  UTF-16, domain, and compatibility lanes but omitted dynamic-array admission
  for the ordinary unary text rows, `REPT`, and the `TEXTAFTER` /
  `TEXTBEFORE` scalar-parameter lanes.

## Why Did We Get This Wrong?
- **Spec already correct and code was wrong?**: `yes`
- **Spec vague or missing?**: `yes`
- **Code once correct and later regressed?**: `no`
- **Likely introduced in ref**: `unknown`
- **Explanation**: the local contract text treated the text-core packet as
  scalar unless a function was already known to be array-producing. Live Excel
  replay now shows that multiple ordinary text functions spill over a single
  array-valued argument on the current baseline. The local runtime did not
  regress from a previously correct state; the evidence set simply never pinned
  those lanes.

## Reproduction
1. Live Excel on 2026-04-09 observed:
   - `=CHAR(SEQUENCE(3)+64) -> {A;B;C}`
   - `=CODE({"A","B"}) -> {65,66}`
   - `=LOWER({"A","B"}) -> {"a","b"}`
   - `=UPPER({"a","b"}) -> {"A","B"}`
   - `=TRIM({"  a  "," b "}) -> {"a","b"}`
   - `=REPT("x",SEQUENCE(3)) -> {"x";"xx";"xxx"}`
   - `=REPT({"a","b"},2) -> {"aa","bb"}`
   - `=TEXTAFTER("a-b-c","-",SEQUENCE(3)) -> {"b-c";"c";#N/A}`
   - `=TEXTBEFORE("a-b-c","-",SEQUENCE(3)) -> {"a";"a-b";#N/A}`
   - `=TEXTAFTER({"a-b","c-d"},"-") -> {"b","d"}`
   - `=TEXTBEFORE({"a-b","c-d"},"-") -> {"a","c"}`
2. Actual pre-fix OxFunc structural behavior:
   - `text_scalar_misc.rs` kept `CHAR`, `CODE`, `LOWER`, `UPPER`, `TRIM`, and
     `REPT` on scalar-only prepared coercion paths,
   - `text_delim_family.rs` kept `TEXTAFTER` and `TEXTBEFORE` on scalar-only
     prepared coercion for all arguments,
   - array-valued arguments therefore surfaced `#VALUE!` instead of spill
     results on the current baseline.
3. Local structural cause:
   - these functions used `prepare_args_values_only(...)` plus scalar
     `coerce_prepared_to_*` calls without any bounded single-array lift.

## Spec And Contract Relationship
- **Spec references**:
  1. `docs/function-lane/FUNCTION_SLICE_TEXT_CORE_AND_COMPATIBILITY_FAMILY_CONTRACT_PRELIM.md`
  2. `docs/function-lane/W66_SCENARIO_MANIFEST_SEED.csv`
- **Spec state at intake**: `vague`
- **Notes**: the existing text-core contract already admitted scalar semantics
  for these rows but did not explicitly state whether the current baseline
  spills over a single array-valued argument. This bug is closed on the landed
  batch-A correction ref while the broader review stays explicit under `W080`.

## Investigation Log
1. 2026-04-09: selected the first bounded `W080` batch from adjacent ordinary
   text functions whose runtime still used scalar-only values-only coercion.
2. 2026-04-09: live Excel replay confirmed spill lanes for `CHAR`, `CODE`,
   `LOWER`, `UPPER`, `TRIM`, `REPT`, `TEXTAFTER`, and `TEXTBEFORE`.
3. 2026-04-09: live Excel replay also showed `TEXTAFTER` /
   `TEXTBEFORE` delimiter-array input does not open an obvious spill lane in the
   simple probe, so the local correction was kept intentionally narrower.
4. 2026-04-09: corrected the local runtime to lift over a single array-valued
   argument in the observed positions only:
   - unary text/number rows over argument 1,
   - `REPT` over either text or count when exactly one argument is array-valued,
   - `TEXTAFTER` / `TEXTBEFORE` over text or `instance_num`.
5. 2026-04-09: added focused unit and surface-dispatch coverage for the new
   spill lanes and refreshed `W080` / `W051` truth surfaces.
6. 2026-04-29: promoted the batch-A correction to landed-ref status on
   `2e818f03a71ba393690275a7fb437ddd9a6bf760`; focused tests were replayed and
   current-gap truth no longer carries these fixed rows.

## Similar-Risk Scan
### Adjacent families to check
1. `CHAR`, `CODE`, `LOWER`, `UPPER`, `TRIM`
2. `REPT`
3. `TEXTAFTER`, `TEXTBEFORE`
4. `FIND`, `SEARCH`, `REPLACE`, `PROPER`, `SUBSTITUTE`
5. any remaining ordinary text rows in `W066` that still rely on scalar-only
   prepared coercion

### Check method
1. live Excel COM replay on 2026-04-09 using bounded array formulas,
2. local code review of scalar-only values-only coercion sites in
   `text_scalar_misc.rs` and `text_delim_family.rs`,
3. focused Rust/unit/surface-dispatch validation for the confirmed lanes.

### Results
1. `CHAR`, `CODE`, `LOWER`, `UPPER`, `TRIM`, `REPT`, `TEXTAFTER`, and
   `TEXTBEFORE` are corrected together on landed ref
   `2e818f03a71ba393690275a7fb437ddd9a6bf760`.
2. `TEXTAFTER` / `TEXTBEFORE` delimiter-array behavior is not widened from this
   packet because the simple probe did not establish a spill lane there.
3. `FIND`, `SEARCH`, `REPLACE`, `PROPER`, and `SUBSTITUTE` remain open for a
   later bounded `W080` review batch rather than being guessed from analogy.

### Follow-on Openings
1. `W080`

## Fix Plan
1. add bounded single-array lift to the ordinary unary text rows in
   `text_scalar_misc.rs`,
2. widen `REPT` over one array-valued text or count argument,
3. widen `TEXTAFTER` / `TEXTBEFORE` over one array-valued text or
   `instance_num` argument only,
4. add focused unit and surface-dispatch tests for the confirmed lanes,
5. keep the broader text-family review explicit under `W080` rather than
   claiming the full packet is now characterized.

## Validation
1. live Excel COM replay on 2026-04-09 for the batch-A formulas listed above
2. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib text_scalar_misc -- --nocapture`
3. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib text_delim_family -- --nocapture`
4. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib surface_dispatch -- --nocapture`
5. 2026-04-29 replayed focused local validation:
   - `text_scalar_misc`: 11 passed
   - `text_delim_family`: 9 passed
   - `surface_dispatch`: 75 passed

## Linked Reports
1. `BUGREP-FUNC-012`

## Evidence
1. `crates/oxfunc_core/src/functions/text_scalar_misc.rs`
2. `crates/oxfunc_core/src/functions/text_delim_family.rs`
3. `crates/oxfunc_core/src/functions/surface_dispatch.rs`
4. `docs/function-lane/FUNCTION_SLICE_TEXT_CORE_AND_COMPATIBILITY_FAMILY_CONTRACT_PRELIM.md`
5. `docs/worksets/W080_FUNCTION_ARRAY_SUPPORT_REVIEW.md`

## Closure Checklist
- [x] fix landed or non-OxFunc ownership recorded
- [x] validation recorded
- [x] root cause recorded
- [x] similar-risk scan recorded
- [x] spec/matrix/contract updated if required
- [x] linked reports updated
- [x] handoff filed if required

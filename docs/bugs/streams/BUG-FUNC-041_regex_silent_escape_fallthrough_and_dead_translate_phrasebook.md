# BUG-FUNC-041: Regex unrecognized escapes silently match literal letters; dead translate phrasebook

## Summary
- **Bug id**: `BUG-FUNC-041`
- **Opened**: `2026-06-11`
- **Status**: `open` (fix landed in `7a0003f` but OVER-REJECTS; live Excel escape battery 2026-06-18 — see below)
- **Owner workset**: `W102A` (no-probe structural fixes)

## Live Excel Verification (2026-06-18) — admitted-slice diverges
A 40-escape `REGEXTEST("a1.B z","\X")` battery vs Excel 16.0 build 20026 found **18
divergences**, all "OxFunc rejects what Excel admits":
- anchors `\b \B \A \Z \z` — Excel admits (TRUE), OxFunc `#VALUE!`
- whitespace `\n \t \r \f \v` — Excel admits (FALSE), OxFunc `#VALUE!`
- escaped metacharacters `\( \) \| \^ \$ \/` — Excel admits (FALSE), OxFunc `#VALUE!`
  (yet OxFunc admits `\* \+ \? \[`, so it is internally inconsistent)
- `\h \e` — Excel admits, OxFunc `#VALUE!`

Both correctly reject unknown letter escapes (`\q \k \m \g \p \c \x \y \j \o`). The
W102A fix swapped "silently match literal" for over-rejection; the admitted escape set
must expand to Excel's. Tracked as bead `oxf-fyhi`. Stays open.

## Source Refs
- **Reported against ref**: review pass 2026-06-10 (digest `functions-text-lookup.md` F9)
- **Reproduced on ref**: code inspection; no fuzzer run required (code-path determinism)
- **Introduced in ref**: unknown (predates current bug-stream history)
- **Fixed in ref**: `not yet fixed` (working-tree patch present; checkpoint not landed)

## Ownership And Root Cause

### Part A — Silent escape fallthrough (the bug)
- **Ownership class**: `OxFunc-owned bug`
- **Root cause class**: `spec_mismatch`
- **Root cause summary**: `parse_escape_atom` mapped any unrecognized escape sequence to
  `RegexAtom::Literal(other)`, so `\n` matched the letter `'n'`, `\t` matched `'t'`,
  `\b` matched `'b'`, and `\x` matched `'x'`. Similarly `parse_class_piece` (inside
  character classes `[…]`) had the same fallthrough arm `other =>
  Ok(RegexClassPiece::Literal(other))`. These produce silently wrong TRUE/FALSE results
  even within the admitted regex slice rather than surfacing a `#VALUE!` error. The
  correct behavior inside the admitted slice is to reject unrecognized escapes with `#VALUE!`
  so the caller is aware the pattern is not usable with this engine.

### Part B - Dead translate phrasebook (dead code)
- **Ownership class**: `OxFunc-owned bug` for Part A; Part B is local cleanup
  under the same W102A code-review finding, not a separate canonical defect class.
- **Root cause class**: `spec_mismatch` for Part A; Part B is code hygiene cleanup.
- **Root cause summary**: `phrasebook()`, `translate_kernel()`, `normalize_phrase()`, and
  `normalize_lang_code()` in `number_regex_translate_family.rs` were never called by
  `eval_translate_surface` — that surface delegates entirely to `HostInfoProvider::query_translate`.
  The functions were a prototype stub that was superseded when the provider-delegation model
  landed. They had no callers outside the file (confirmed by `rg phrasebook|translate_kernel`).

## Reproduction

**Part A** — before the fix, the following produced `Ok(true)` where `#VALUE!` is correct:
```
regextest_kernel(&txt("n"), &txt("\\n"), true)   // \n silently matched 'n'
regextest_kernel(&txt("t"), &txt("\\t"), true)   // \t silently matched 't'
regextest_kernel(&txt("b"), &txt("\\b"), true)   // \b silently matched 'b'
regextest_kernel(&txt("n"), &txt("[\\n]"), true) // same inside class
```

**Part B** — `translate_kernel` had `pub` visibility but zero external call sites:
```
rg "translate_kernel|phrasebook" crates/ -- returns only the definitions in the source file
```

## Fix Plan

### Part A
In `parse_escape_atom`: replaced the catch-all `other => RegexAtom::Literal(other)` arm with
an explicit whitelist of supported literal-escape punctuation (`\\ \. \* \+ \? \[ \]`) and a
final `_ => return Err(WorksheetErrorCode::Value)` arm for everything else. The six shorthand
atoms (`\d \D \w \W \s \S`) are unchanged.

In `parse_class_piece`: same change — added `\\ \. \* \+ \? \[ \] \- \^` as explicit
literal-escape cases (the extra two cover characters with special meaning inside `[…]`),
kept the `D | W | S` explicit error, and replaced the remaining `other` fallthrough with
`_ => Err(WorksheetErrorCode::Value)`.

### Part B
The current working-tree patch removes `normalize_phrase`, `normalize_lang_code`,
`phrasebook`, and `translate_kernel` from `number_regex_translate_family.rs`.
No callers existed; no tests referenced them.

## Validation
- New regression tests added:
  - `regextest_unrecognized_escape_newline_is_value_error` — `\n` → `#VALUE!`
  - `regextest_unrecognized_escape_tab_is_value_error` — `\t` → `#VALUE!`
  - `regextest_unrecognized_escape_word_boundary_is_value_error` — `\b` → `#VALUE!`
  - `regextest_unrecognized_escape_hex_is_value_error` — `\x` → `#VALUE!`
  - `regextest_unrecognized_escape_in_class_is_value_error` — `[\n]` → `#VALUE!`
  - `regextest_admitted_literal_escapes_still_work` — `\.` and `\\` continue to work
- Full `oxfunc_core` regex filter: 24 tests, 0 failures.

## Similar-Risk Scan
- The `parse_class_piece` fallthrough for `D | W | S` (negated shorthands inside `[…]`)
  was already an explicit `#VALUE!`; that behavior is correct and unchanged.
- The loud `#VALUE!` rejections for groups `()`, alternation `|`, anchors `^ $`,
  and braces `{}` in `parse_regex_pattern` are intentional slice boundaries; not touched.
- No other escape-parsing paths exist in this file.

## Closure Checklist
- [ ] fix landed or non-OxFunc ownership recorded (working-tree patch present; checkpoint not landed)
- [x] validation recorded
- [x] root cause recorded
- [x] similar-risk scan recorded
- [ ] OxFunc-local admitted escape contract updated under W102B
      (`FUNCTION_SLICE_REGEX_TRIAD_CONTRACT_PRELIM.md` does not enumerate the
      admitted escape set)
- [ ] handoff filed if W102B evidence shows an OxFml-facing contract change is required
- [x] linked reports updated

# BUG-FUNC-041: Regex unrecognized escapes silently match literal letters; dead translate phrasebook

## Summary
- **Bug id**: `BUG-FUNC-041`
- **Opened**: `2026-06-11`
- **Status**: `open` (W102A fix landed in `7a0003f` but over-rejected; local repair
  on `2026-06-26` expands the admitted escape set and is live-Excel signed off;
  checkpoint/landing still pending)
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
W102A fix swapped "silently match literal" for over-rejection. The `2026-06-26`
working-tree repair expands the admitted escape set locally and is now live-Excel
signed off; bead `oxf-fyhi` stays open until checkpoint/landing.

## Live Excel Verification (2026-06-26) — local repair matches
The repaired working tree was replayed against desktop Excel via COM using the same
40-case escape battery. Excel environment: version `16.0`, build `20026`, Windows
64-bit. Result: `40/40` exact typed matches, `0` mismatches.

Artifacts:
- `docs/bugs/evidence/BUG-FUNC-041_REGEX_ESCAPE_SIGNOFF_20260626.md`
- `smart-fuzzer/runs/bug-func-041-regex-escape-signoff-20260626/rollup.json`
- `smart-fuzzer/runs/bug-func-041-regex-escape-signoff-20260626/cases/regex-escape-battery.jsonl`
- `smart-fuzzer/runs/bug-func-041-regex-escape-signoff-20260626/outcomes/local.jsonl`
- `smart-fuzzer/runs/bug-func-041-regex-escape-signoff-20260626/outcomes/excel.jsonl`
- `smart-fuzzer/runs/bug-func-041-regex-escape-signoff-20260626/comparisons/comparisons.jsonl`

## Source Refs
- **Reported against ref**: review pass 2026-06-10 (digest `functions-text-lookup.md` F9)
- **Reproduced on ref**: code inspection; no fuzzer run required (code-path determinism)
- **Introduced in ref**: unknown (predates current bug-stream history)
- **Fixed in ref**: `not yet landed` (working-tree repair is live-Excel signed off; checkpoint/landing not landed)

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

## Repair State

### Part A
The original W102A repair correctly removed silent literal fallthrough but made the admitted
escape set too narrow. The current working-tree repair expands `parse_escape_atom` to the
Excel-observed escape split:

- admitted shorthand classes: `\d \D \w \W \s \S`
- admitted zero-width assertions: `\A \Z \z \b \B`
- admitted character escapes: `\n \t \r \f \v \e \h`
- admitted escaped literal metacharacters: `\. \* \+ \? \[ \] \( \) \| \^ \$ \/ \\`
- rejected unknown letter escapes remain rejected with `#VALUE!`: `\q \k \m \g \p \c \x \y \j \o`

The matcher now has explicit zero-width assertion atoms so anchors/boundaries are not modeled as
literals, and quantified zero-width assertions are rejected to avoid implying a richer local regex
engine than the current slice supports.

`parse_class_piece` now admits the character escapes and escaped literals that are meaningful
inside character classes, while unknown class escapes still return `#VALUE!`.

### Part B
The dead `TRANSLATE` phrasebook cleanup remains part of the prior W102A checkpoint and is not
changed by the 2026-06-26 regex escape repair.

## Validation
- New regression tests added:
  - `regextest_admits_excel_escape_battery` — the Excel-observed admitted/rejected split
    for the 40-escape battery baseline.
  - `regextest_unknown_letter_escapes_remain_value_errors` — unknown letter escapes still
    return `#VALUE!`.
  - `regextest_control_escapes_match_control_characters` — `\n \t \r \f \v \e \h`.
  - `regextest_zero_width_assertions_compose_with_literals` — `\A`, `\z`, `\b`, and `\B`.
  - `regextest_admitted_escapes_work_in_classes` — admitted class escapes.
  - `regextest_unrecognized_escape_in_class_is_value_error` — unknown class escapes still
    return `#VALUE!`.
- Full `oxfunc_core` regex filter on 2026-06-26: 24 tests, 0 failures.
- Live Excel sign-off on 2026-06-26: 40-case escape battery, 40 exact typed
  matches, 0 mismatches.

## Similar-Risk Scan
- The `parse_class_piece` fallthrough for `D | W | S` (negated shorthands inside `[…]`)
  was already an explicit `#VALUE!`; that behavior is correct and unchanged.
- The loud `#VALUE!` rejections for unescaped groups `()`, alternation `|`, anchors `^ $`,
  and braces `{}` in `parse_regex_pattern` remain intentional slice boundaries. Escaped
  grouping/alternation/anchor characters are now admitted only as literals.
- No other escape-parsing paths exist in this file.

## Closure Checklist
- [ ] fix landed or non-OxFunc ownership recorded (working-tree repair present and live-Excel signed off; checkpoint/landing not landed)
- [x] validation recorded
- [x] root cause recorded
- [x] similar-risk scan recorded
- [x] OxFunc-local admitted escape contract updated under W102B
- [x] cross-repo handoff assessed: not required for this pure OxFunc parser-slice repair
- [x] linked reports updated

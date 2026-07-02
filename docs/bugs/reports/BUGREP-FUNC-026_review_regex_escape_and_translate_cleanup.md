# BUGREP-FUNC-026: Review finding on regex escapes and translate cleanup

## Intake
- **Report id**: `BUGREP-FUNC-026`
- **Filed**: 2026-06-15
- **Source channel**: local review follow-up
- **Reporter/source**: June 2026 OxFunc cleanup review
- **Reported against ref**: `w100-w102-cleanup-pass working tree`
- **Reported against kind**: unknown
- **Reported against note**: review digest `functions-text-lookup.md` F9
- **Canonical bug id**: `BUG-FUNC-041`
- **Status**: triaged

## Observed Symptom
The admitted regex parser silently treated unrecognized escapes as literal
letters, and the same file contained a dead translate phrasebook superseded by
host-provider delegation.

## Reproduction
1. See `BUG-FUNC-041`.
2. Expected: unrecognized regex escapes return `#VALUE!`.
3. Actual before the working-tree patch: `\n`, `\t`, `\b`, and similar patterns
   could match literal letters.

## Initial Ownership Read
- **Initial classification**: OxFunc-owned bug
- **Reason**: the escape behavior is inside OxFunc regex parsing.

## Links
1. `docs/bugs/streams/BUG-FUNC-041_regex_silent_escape_fallthrough_and_dead_translate_phrasebook.md`
2. `crates/oxfunc_core/src/functions/number_regex_translate_family.rs`

## Triage Notes
The regex admitted-slice contract now enumerates supported escapes via
`BUG-FUNC-041-REGEX-ESCAPES-20260626`. The local repair is live-Excel signed off
against the 40-case escape battery; the remaining lane is checkpoint/landing.

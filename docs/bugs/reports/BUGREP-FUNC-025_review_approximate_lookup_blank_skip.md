# BUGREP-FUNC-025: Review finding on approximate lookup blank skipping

## Intake
- **Report id**: `BUGREP-FUNC-025`
- **Filed**: 2026-06-15
- **Source channel**: local review follow-up
- **Reporter/source**: June 2026 OxFunc cleanup review
- **Reported against ref**: `w100-w102-cleanup-pass working tree`
- **Reported against kind**: unknown
- **Reported against note**: review digest `functions-text-lookup.md` F4
- **Canonical bug id**: `BUG-FUNC-040`
- **Status**: triaged

## Observed Symptom
Approximate/binary lookup paths aborted on blank cells instead of skipping them,
breaking canonical approximate-match idioms.

## Reproduction
1. See `BUG-FUNC-040`.
2. Expected: blanks are skipped in approximate and binary lookup candidate scans.
3. Actual before the working-tree patch: `#N/A`.

## Initial Ownership Read
- **Initial classification**: OxFunc-owned bug
- **Reason**: the bug is in OxFunc lookup candidate collection.

## Links
1. `docs/bugs/streams/BUG-FUNC-040_approximate_match_blank_skipping_gap.md`
2. `crates/oxfunc_core/src/functions/match_fn.rs`
3. `crates/oxfunc_core/src/functions/xmatch.rs`
4. `crates/oxfunc_core/src/functions/vhlookup_family.rs`

## Triage Notes
Error-cell and cross-type approximate behavior remain separate probe lanes.

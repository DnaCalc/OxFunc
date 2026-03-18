# W16 Batch 36 - Text Slice Family

Status: `in_progress-provisional`
Workset: `W16`
Evidence ID: `W16-BATCH36-TEXT-SLICE-20260316`

## Scope
1. `LEN`
2. `LEFT`
3. `RIGHT`
4. `MID`

Out of current batch scope:
1. `LENB`
2. `LEFTB`
3. `RIGHTB`
4. `MIDB`

## Deterministic Baseline Reuse
Reused deterministic source/evidence anchors:
1. `docs/function-lane/STRING_BEHAVIOR_RESEARCH_NOTES.md`
2. `docs/function-lane/STRING_SCENARIO_MANIFEST_SEED.csv`
3. `docs/function-lane/STRING_EXECUTION_RECORD.md`
4. `docs/function-lane/TEXT_FUNCTION_EMPIRICAL_EXPANSION_NOTES.md`
5. `.tmp/w16-batch35-36-text-probe.csv`
6. `.tmp/w16-batch36-text-slice-emoji-probe.csv`

Pinned reused lanes:
1. `LEN(UNICHAR(128512)) -> 2` in the current baseline (`STR8-012`).
2. `LEN("e"&UNICHAR(769)) -> 2` for combining-sequence text (`STR8-015`).
3. `LEFT(UNICHAR(128512),1)` and `RIGHT(UNICHAR(128512),1)` both return the full surrogate pair in the current baseline (`.tmp/w16-batch35-36-text-probe.csv`).
4. `MID(UNICHAR(128512),2,1)` returns the trailing low-surrogate code unit and `UNICODE(MID(UNICHAR(128512),2,1)) -> 56832` in the current baseline (`.tmp/w16-batch36-text-slice-emoji-probe.csv`).
5. overflow-truncated emoji text can leave a dangling high-surrogate tail and `LEN(RIGHT(A1,1)) -> 1` in that state (`STR8-045`).

## Current Implementation Notes
1. all four functions use the ordinary values-only preparation seam.
2. `LEN` counts raw UTF-16 code units and `MID` slices by one-based UTF-16 code-unit offsets, so surrogate-pair and dangling-tail states remain observable instead of being normalized away.
3. `LEFT` and `RIGHT` preserve a complete leading or trailing surrogate pair when a single-character slice lands exactly on that pair boundary.
4. `LEFT` and `RIGHT` default `num_chars` to `1` only when the second argument is omitted entirely.
5. `LEFT`, `RIGHT`, and `MID` truncate numeric position/count arguments toward zero before domain checks.
6. `MID` uses one-based indexing, returns `#VALUE!` for `start_num < 1` or `num_chars < 0`, and returns empty text when the start position is past the end of the string.
7. `LEFT` and `RIGHT` clamp oversized positive counts to the full text length.

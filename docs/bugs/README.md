# OxFunc Bug Tracking

This directory holds the canonical local bug-intake and regression-family tracking
scaffolding for OxFunc.

Project-wide exactness residuals are summarized in
[`../KNOWN_EXACTNESS_DEVIATIONS.md`](../KNOWN_EXACTNESS_DEVIATIONS.md). The bug
streams in this directory remain the detailed ownership and evidence records.

It separates:
1. bug reports
   - every incoming report or observed defect record
2. bug streams
   - the canonical known-bug lane that owns investigation, root-cause analysis,
     similar-risk scanning, validation, and closure

## Identity Model

### Bug Reports
Individual reports use:
- `BUGREP-FUNC-NNN`

A report captures:
1. who reported the problem,
2. what exact repo ref it was reported against,
3. the initial symptom and reproduction context,
4. the canonical bug stream linkage once triaged.

### Bug Streams
Canonical known-bug lanes use:
- `BUG-FUNC-NNN`

A stream captures:
1. the canonical problem statement,
2. exact affected refs,
3. reproduction state,
4. ownership classification,
5. root-cause classification and explanation,
6. introduced/fixed refs where known,
7. similar-risk scan results,
8. links to all known related reports.

## Files
1. `BUG_REPORT_REGISTER.csv`
   - one row per incoming report
2. `BUG_STREAM_REGISTER.csv`
   - one row per canonical bug stream
3. `BUG_REPORT_TEMPLATE.md`
   - report note template
4. `BUG_STREAM_TEMPLATE.md`
   - canonical stream template
5. `reports/`
   - individual bug report notes
6. `streams/`
   - canonical bug stream notes

## Working Rules
1. Every incoming bug gets a `BUGREP-FUNC-*` record even if it is immediately recognized as a duplicate.
2. Every non-trivial unique bug gets or links to a canonical `BUG-FUNC-*` stream.
3. Duplicate reports are not discarded; they are linked to the canonical bug stream through the report register and report note.
4. Worksets remain the bounded implementation owner for the bug, and `.beads/` remains the live execution-state and blocker surface.
5. A canonical bug stream is not closed until:
   - the fix landed or non-OxFunc ownership was recorded,
   - local validation was recorded,
   - root-cause analysis was recorded,
   - similar-risk scanning was recorded,
   - any required spec/matrix/contract or handoff update was recorded,
   - linked reports were updated.

## Source-Ref Rule
Every report and stream must record the exact source ref against which the defect
was observed:
1. preferred: released version/tag,
2. fallback: exact git commit SHA,
3. if neither is known, record `unknown` and say why.

Current bootstrap ref:
- `9e9c573a46d97e248a0373938fb53dcac916fac2`

# BUG-FUNC-041 Regex Escape Excel Sign-Off - 2026-06-26

Status: `evidence_record`
Bug: `BUG-FUNC-041`
Bead: `oxf-fyhi`
Evidence ID: `BUG-FUNC-041-REGEX-ESCAPES-20260626`
Run ID: `bug-func-041-regex-escape-signoff-20260626`

## Purpose

Replay the repaired working-tree regex escape parser/matcher against live Excel for
the 40-case `REGEXTEST("a1.B z","\X")` escape battery that previously exposed
W102A over-rejection.

## Inputs

1. Case source: `smart-fuzzer/cache/regex-escape-battery.jsonl`
2. Local evaluator: `smart-fuzzer/tools/pmt_ppmt_local_eval` binary
   `array_tranche_local_eval`
3. Excel oracle: desktop Excel COM automation

## Excel Environment

1. Excel version: `16.0`
2. Excel build: `20026`
3. Operating system reported by Excel: `Windows (64-bit) NT 10.00`

## Result

1. total_cases: `40`
2. exact_typed_matches: `40`
3. unexpected_mismatches: `0`
4. mismatch_case_ids: `[]`

The repaired local outcomes matched Excel's typed TRUE/FALSE/error results across:

1. admitted shorthand classes,
2. admitted zero-width assertions,
3. admitted whitespace/control escapes,
4. admitted escaped metacharacter literals,
5. continued `#VALUE!` rejection for unknown letter escapes.

## Local Run Artifacts

The detailed local run is under an ignored smart-fuzzer run directory:

1. `smart-fuzzer/runs/bug-func-041-regex-escape-signoff-20260626/rollup.json`
2. `smart-fuzzer/runs/bug-func-041-regex-escape-signoff-20260626/cases/regex-escape-battery.jsonl`
3. `smart-fuzzer/runs/bug-func-041-regex-escape-signoff-20260626/outcomes/local.jsonl`
4. `smart-fuzzer/runs/bug-func-041-regex-escape-signoff-20260626/outcomes/excel.jsonl`
5. `smart-fuzzer/runs/bug-func-041-regex-escape-signoff-20260626/comparisons/comparisons.jsonl`

## Status Axes

scope_completeness: `scope_complete`
target_completeness: `target_complete`
integration_completeness: `partial`
open_lanes: `[BUG-FUNC-041_checkpoint_landing]`

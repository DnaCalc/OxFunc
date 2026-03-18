# W16 Batch 24 - Base Conversion and Roman Parsing

Status: `in_progress-provisional`
Workset: `W16`
Evidence ID: `W16-BATCH24-BASE-ARABIC-20260315`

## Scope
1. `BASE`
2. `ARABIC`

## Native Excel Baseline
Probe artifact:
1. `.tmp/w16-batch24-base-arabic-probe.csv`

Pinned lanes:
1. `BASE(31,16) -> "1F"`
2. `BASE(31,16,4) -> "001F"`
3. `BASE(31.9,16,4.8) -> "001F"`
4. `BASE(-1,16) -> #NUM!`
5. `BASE(31,1) -> #NUM!`
6. `ARABIC("LVII") -> 57`
7. `ARABIC("mcmxii") -> 1912`
8. `ARABIC("") -> 0`
9. `ARABIC("ABC") -> #VALUE!`
10. `ARABIC("IV") -> 4`

## Current Implementation Notes
1. `BASE` currently uses truncated numeric arguments, radix bounds `2..36`, and left-pads with ASCII `0` up to the truncated minimum-length request.
2. `ARABIC` currently accepts lowercase Roman text by uppercasing first, returns `0` on the empty string lane, and rejects non-Roman characters with `#VALUE!`.
3. This batch remains provisional inside `W016`; broader Roman-notation admissibility beyond the pinned rows can be widened later if new evidence demands it.

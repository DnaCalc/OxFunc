# W16 Batch 25 - Single Mode

Status: `in_progress-provisional`
Workset: `W16`
Evidence ID: `W16-BATCH25-MODE-SNGL-20260315`

## Scope
1. `MODE.SNGL`

## Native Excel Baseline
Probe artifact:
1. `.tmp/w16-batch25-mode-sngl-probe.csv`

Pinned lanes:
1. `MODE.SNGL(1,2,2,3) -> 2`
2. `MODE.SNGL(1,2,3) -> #N/A`
3. `MODE.SNGL(TRUE,"2") -> #N/A`
4. `MODE.SNGL(G1:G2) -> #N/A`
5. `MODE.SNGL(G1:G3) -> #N/A`
6. `MODE.SNGL(2,2,3,3,4) -> 2`
7. `MODE.SNGL("x") -> #N/A`

## Current Implementation Notes
1. Only numeric survivors participate in the mode count.
2. Direct and reference-derived text/logicals are ignored rather than coerced.
3. Errors propagate through the surface path.
4. Ties pick the smallest repeated numeric value in the current baseline.

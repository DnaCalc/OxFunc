# W16 Batch 49 - Switch

Status: `in_progress-provisional`
Workset: `W16`
Evidence ID: `W16-BATCH49-SWITCH-20260316`

## Scope
1. `SWITCH`

## Native Excel Baseline
Probe artifacts:
1. `.tmp/probe-switch.ps1`
2. `.tmp/probe-switch2.ps1`
3. `.tmp/probe-switch3.ps1`

Pinned lanes:
1. `SWITCH(2,1,"a",2,"b") -> "b"`
2. `SWITCH(2,1,"a",3,"c","d") -> "d"`
3. `SWITCH(2,1,"a") -> #N/A`
4. `SWITCH(2,2,TRUE) -> TRUE`
5. `SWITCH(TRUE,1,"a",TRUE,"b") -> "b"`
6. `SWITCH("a","A",1,"a",2) -> 1`
7. `SWITCH("2",2,1,"2",2) -> 2`
8. Later candidates and results are not forced after an earlier match, but an earlier candidate error still propagates.

## Current Implementation Notes
1. Candidate comparison is exact by type except for text, which follows the current ASCII case-insensitive baseline.
2. Result expressions are selected lazily in the same style as `IF` and `CHOOSE`.
3. A missing match without default returns `#N/A`.

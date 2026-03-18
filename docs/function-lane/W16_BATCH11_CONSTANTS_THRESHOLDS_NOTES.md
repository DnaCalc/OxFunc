# W16 Batch 11 - Constants and Threshold Functions

Status: `in_progress-provisional`
Workset: `W16`
Evidence ID: `W16-BATCH11-CONSTANTS-THRESHOLDS-20260315`

## Scope
1. `TRUE`
2. `FALSE`
3. `NA`
4. `DELTA`
5. `GESTEP`
6. `TRUNC`

## Native Excel Baseline
Probe artifact:
1. `.tmp/w16-batch11-constants-thresholds-probe.csv`

Pinned lanes:
1. `TRUE() -> TRUE`
2. `FALSE() -> FALSE`
3. `NA() -> #N/A`
4. `DELTA(5,5) -> 1`
5. `DELTA(5,4) -> 0`
6. `GESTEP(5) -> 1`
7. `GESTEP(4,5) -> 0`
8. `TRUNC(8.9) -> 8`
9. `TRUNC(-8.9) -> -8`
10. `TRUNC(3.14159,3) -> 3.141`
11. `TRUNC(314.159,-10) -> 0`

# W16 Batch 29 - Log and Directional Rounding Functions

Status: `in_progress-provisional`
Workset: `W16`
Evidence ID: `W16-BATCH29-LOG-ROUNDING-20260315`

## Scope
1. `LOG`
2. `ROUNDDOWN`
3. `ROUNDUP`

## Native Excel Baseline
Probe artifact:
1. `.tmp/w16-batch29-log-rounding-probe.csv`

Pinned lanes:
1. `LOG(8,2) -> 3`
2. `LOG(8) -> 0.903089986991944`
3. `LOG(0,10) -> #NUM!`
4. `LOG(8,1) -> #DIV/0!`
5. `LOG(-1,10) -> #NUM!`
6. `LOG(8,-2) -> #NUM!`
7. `LOG("8",2) -> 3`
8. `LOG(TRUE,10) -> 0`
9. `ROUNDDOWN(3.14159,3) -> 3.141`
10. `ROUNDDOWN(-3.14159,3) -> -3.141`
11. `ROUNDDOWN(314.159,-2) -> 300`
12. `ROUNDDOWN(1.5,0.9) -> 1`
13. `ROUNDUP(3.14159,3) -> 3.142`
14. `ROUNDUP(-3.14159,3) -> -3.142`
15. `ROUNDUP(314.159,-2) -> 400`
16. `ROUNDUP(1.5,0.9) -> 2`

## Current Implementation Notes
1. `LOG` defaults the base to `10` when omitted.
2. `LOG` uses ordinary values-only numeric coercion for both arguments, so direct text numeric and logical values are admitted through the standard scalar coercion path.
3. `LOG(number,1)` is a special `#DIV/0!` lane rather than `#NUM!`.
4. `ROUNDDOWN` rounds toward zero at the requested digit position.
5. `ROUNDUP` rounds away from zero at the requested digit position.
6. Both directional rounding functions truncate `num_digits` toward zero before applying the rounding policy.

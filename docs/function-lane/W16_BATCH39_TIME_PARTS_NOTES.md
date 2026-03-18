# W16 Batch 39 - Time Serial Parts

Status: `in_progress-provisional`
Workset: `W16`
Evidence ID: `W16-BATCH39-TIME-PARTS-20260316`

## Scope
1. `HOUR`
2. `MINUTE`
3. `SECOND`
4. `TIME`

## Native Excel Baseline
Probe artifacts:
1. `.tmp/w16-batch39-time-parts-probe.csv`
2. `.tmp/w16-batch39-time-parts-edge-probe.csv`

Pinned lanes:
1. `HOUR(0) -> 0`
2. `MINUTE(0) -> 0`
3. `SECOND(0) -> 0`
4. `HOUR(1.9) -> 21`
5. `MINUTE(1.9) -> 36`
6. `SECOND(1.9) -> 0`
7. `HOUR(-0.1) -> #NUM!`
8. `TIME(1,2,3) -> 0.0430902777777778`
9. `TIME(27,0,0) -> 0.125`
10. `TIME(0,120,0) -> 0.0833333333333333`
11. `TIME(0,0,120) -> 0.00138888888888889`
12. `TIME(TRUE,2,3) -> 0.0430902777777778`
13. `TIME("1","2","3") -> 0.0430902777777778`
14. `TIME("x",2,3) -> #VALUE!`
15. `TIME(-1,60,0) -> 0`
16. `TIME(-1,0,1) -> #NUM!`
17. `TIME(32767,0,0) -> 0.291666666666742`
18. `TIME(32768,0,0) -> #NUM!`
19. `TIME(,2,3) -> 0.00142361111111111`

## Current Implementation Notes
1. `HOUR`, `MINUTE`, and `SECOND` extract from the fractional-day portion of the serial after rejecting negative or non-finite inputs with `#NUM!`.
2. `TIME` truncates each component toward zero, accepts numeric text and logicals through the ordinary numeric coercion path, and treats omitted/blank components as zero.
3. Component magnitudes above `32767` yield `#NUM!`.
4. Negative intermediate components are accepted only when the normalized total time remains non-negative, which is required for rows like `TIME(-1,60,0) -> 0`.
5. Positive totals are reduced modulo one day for the returned serial fraction.

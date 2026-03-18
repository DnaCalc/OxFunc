# W16 Batch 37 - Dollar Fraction Conversion Family

Status: `in_progress-provisional`
Workset: `W16`
Evidence ID: `W16-BATCH37-DOLLAR-FRACTION-20260316`

## Scope
1. `DOLLARDE`
2. `DOLLARFR`

## Native Excel Baseline
Probe artifacts:
1. `.tmp/w16-batch37-dollar-fraction-probe.csv`
2. `.tmp/w16-batch37-dollar-fraction-ref-probe.csv`

Pinned lanes:
1. `DOLLARDE(1.02,16) -> 1.125`
2. `DOLLARFR(1.125,16) -> 1.02`
3. `DOLLARDE(1.02,8) -> 1.025`
4. `DOLLARFR(1.125,8) -> 1.1`
5. `DOLLARDE(1.02,16.9) -> 1.125`
6. `DOLLARFR(1.125,16.9) -> 1.02`
7. `DOLLARDE(1.02,0.9) -> #DIV/0!`
8. `DOLLARFR(1.125,0.9) -> #DIV/0!`
9. `DOLLARDE(1.02,-0.1) -> #NUM!`
10. `DOLLARFR(1.125,-0.1) -> #NUM!`
11. `DOLLARDE(TRUE,16) -> #VALUE!`
12. `DOLLARFR(TRUE,16) -> #VALUE!`
13. `DOLLARDE("1.02",16) -> 1.125`
14. `DOLLARFR("1.125",16) -> 1.02`
15. `DOLLARDE(,16) -> #N/A`
16. `DOLLARFR(,16) -> #N/A`
17. `DOLLARDE(A1,A2) -> 0` when `A1` is blank and `A2=16`
18. `DOLLARFR(A3,A1) -> #DIV/0!` when `A3=1.125` and `A1` is blank
19. `DOLLARDE(1.01,32) -> 1.03125`
20. `DOLLARFR(-1.125,16) -> -1.02`

## Current Implementation Notes
1. Both functions use values-only preparation with a custom numeric coercion policy: numeric text is accepted, blank cells become `0`, omitted required args return worksheet `#N/A`, and logical arguments are rejected with `#VALUE!`.
2. The denominator argument is truncated toward zero before the core conversion math is applied.
3. Any negative denominator yields `#NUM!`; any truncated zero denominator yields `#DIV/0!`.
4. `DOLLARDE` converts the post-decimal digits of the quoted fractional price into a decimal part using the digit width of the denominator.
5. `DOLLARFR` performs the inverse transform on the fractional part while preserving the whole-part sign behavior observed in native Excel.
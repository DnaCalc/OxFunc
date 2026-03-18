# W16 Batch 34 - Roman Numeral Rendering

Status: `in_progress-provisional`
Workset: `W16`
Evidence ID: `W16-BATCH34-ROMAN-20260316`

## Scope
1. `ROMAN`

## Native Excel Baseline
Probe artifact:
1. `.tmp/w16-batch33-roman-decimal-probe.csv`

Pinned lanes:
1. `ROMAN(499) -> "CDXCIX"`
2. `ROMAN(499,1) -> "LDVLIV"`
3. `ROMAN(499,2) -> "XDIX"`
4. `ROMAN(499,3) -> "VDIV"`
5. `ROMAN(499,4) -> "ID"`
6. `ROMAN(0) -> ""`
7. `ROMAN(-1) -> #VALUE!`
8. `ROMAN(3999) -> "MMMCMXCIX"`
9. `ROMAN(4000) -> #VALUE!`
10. `ROMAN("499") -> "CDXCIX"`
11. `ROMAN(499,5) -> #VALUE!`
12. `ROMAN(499,-1) -> #VALUE!`
13. `ROMAN(499.9) -> "CDXCIX"`
14. `ROMAN(499,1.9) -> "LDVLIV"`

## Current Implementation Notes
1. `ROMAN` is wired as a deterministic values-only custom surface with optional second-argument form control and no new OxFml/FEC seam demand.
2. The current kernel follows the empirically pinned Excel simplification tiers `0..4`, including the sharper late-tier collapses `499 -> VDIV -> ID` and `999 -> VMIV -> IM`.
3. Blank or omitted first-argument lanes currently follow the observed Excel result `""`, while blank or omitted second-argument lanes default to classic form `0`.
4. Logical second-argument lanes are pinned to observed Excel behavior: `TRUE -> classic` and `FALSE -> most simplified`.
5. This batch remains provisional inside `W016`; broader Roman admissibility and alternate-version sweeps remain outside the admitted current-baseline slice.

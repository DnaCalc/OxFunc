# W16 Batch 52 - Complex Number Text Family

Status: `in_progress-provisional`
Workset: `W16`
Evidence ID: `W16-BATCH52-COMPLEX-FAMILY-20260316`

## Scope
1. `COMPLEX`
2. `IMABS`
3. `IMAGINARY`
4. `IMARGUMENT`
5. `IMCONJUGATE`
6. `IMCOS`
7. `IMCOSH`
8. `IMCOT`
9. `IMCSC`
10. `IMCSCH`
11. `IMDIV`
12. `IMEXP`
13. `IMLN`
14. `IMLOG10`
15. `IMLOG2`
16. `IMPOWER`
17. `IMPRODUCT`
18. `IMREAL`
19. `IMSEC`
20. `IMSECH`
21. `IMSIN`
22. `IMSINH`
23. `IMSQRT`
24. `IMSUB`
25. `IMSUM`
26. `IMTAN`

## Native Excel Baseline
Probe artifacts:
1. `.tmp/probe_complex_family.ps1`
2. `.tmp/probe_complex_edges2.ps1`
3. `.tmp/probe_complex_suffix.ps1`
4. `.tmp/probe_complex_suffix2.ps1`
5. `.tmp/probe_complex_blank.ps1`
6. `.tmp/probe_complex_textnums.ps1`
7. `.tmp/probe_complex_cases3.ps1`

Pinned lanes:
1. `COMPLEX(3,4) -> "3+4i"`
2. `COMPLEX(0,1) -> "i"`; `COMPLEX(0,-1) -> "-i"`; `COMPLEX(3,0) -> "3"`
3. `COMPLEX(3,4,"j") -> "3+4j"`; invalid suffix such as `"x"` maps to `#VALUE!`
4. `COMPLEX("3","4")` admits text numerics, while logical inputs map to `#VALUE!`
5. `IMABS("3+4i") -> 5`; `IMREAL("3+4i") -> 3`; `IMAGINARY("3+4i") -> 4`
6. `IMARGUMENT("3+4i") -> 0.927295218001612`; `IMARGUMENT(0) -> #DIV/0!`
7. `IMCONJUGATE("3+4i") -> "3-4i"`
8. `IMSUM("3+4i","1-2i") -> "4+2i"`; `IMSUB(...) -> "2+6i"`; `IMPRODUCT(...) -> "11-2i"`; `IMDIV(...) -> "-1+2i"`
9. `IMSQRT("3+4i") -> "2+i"`; `IMPOWER("3+4i",2) -> "-7+24i"`; `IMPOWER("3+4i",0.5) -> "2+i"`
10. `IMEXP`, `IMLN`, `IMLOG10`, `IMLOG2`, `IMSIN`, `IMCOS`, `IMTAN`, `IMSINH`, `IMCOSH`, `IMSEC`, `IMSECH`, `IMCSC`, `IMCSCH`, `IMCOT` all match the seeded `"3+4i"` baseline rows in unit tests
11. Mixed suffixes such as `IMSUM("i","j")` map to `#VALUE!`; unary complex-text outputs preserve `j` when the operand uses `j`
12. Invalid complex text such as `"foo"` maps to `#NUM!`; `IMDIV(...,0)`, `IMLN(0)`, `IMLOG10(0)`, and `IMLOG2(0)` map to `#NUM!`
13. Blank/missing scalar operands behave as zero in the seeded admitted lanes, e.g. blank input to `IMABS` gives `0`

## Current Implementation Notes
1. The Rust family now owns the kernels plus the shared-surface-ready eval functions and per-function `FunctionMeta` constants used by the integrated dispatch path.
2. Parsing admits Excel-style real text, pure imaginary text (`i`, `-i`, `4i`, `4j`), and mixed text such as `3+4i`, `3+j`, and `3-j`.
3. Only lowercase `i` and `j` suffixes are admitted; uppercase and mixed-suffix operand sets are rejected.
4. Formatting preserves Excel-style omission of the `1` coefficient before `i` or `j`, drops the suffix for purely real results, and snaps near-integer floating outputs to stable integer text.
5. Error mapping in the admitted slice is: invalid complex text `#NUM!`, invalid suffix/logical misuse `#VALUE!`, zero argument for `IMARGUMENT` `#DIV/0!`, and logarithm/division poles `#NUM!`.
6. Shared dispatch, XLL export catalog generation, and Lean root imports are now wired for the admitted slice.

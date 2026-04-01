# Function Slice - Complex Number Family Contract (Prelim)

Status: `active`
Owner lane: `OxFunc`
Workset: `W060`

## 1. Purpose
Define the current-phase contract for the `W060` complex-number family.

## 2. Covered Surface
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

## 3. Contract
1. the family uses the ordinary values-only pre-adapter seam with custom kernels and text/value coercion rules,
2. parsing admits Excel-style real numerics, pure imaginary forms such as `i`, `-i`, `4i`, `4j`, and mixed forms such as `3+4i`, `3+j`, and `3-j`,
3. only lowercase `i` and `j` suffixes are admitted; invalid suffix requests in `COMPLEX` return `#VALUE!`,
4. unary text-result functions preserve the operand suffix choice when the input uses `j`,
5. mixed-suffix arithmetic sets such as `IMSUM("i","j")` return `#VALUE!`,
6. invalid complex text such as `"foo"` returns `#NUM!`,
7. `IMARGUMENT(0)` returns `#DIV/0!`,
8. division and logarithm poles such as `IMDIV(...,0)`, `IMLN(0)`, `IMLOG10(0)`, and `IMLOG2(0)` return `#NUM!`,
9. formatting preserves Excel-style omission of the `1` coefficient before `i` or `j`, drops the suffix for purely real results, and snaps near-integer floating outputs to stable integer text,
10. `IMPOWER` admits fractional exponents on the current baseline and publishes the principal branch value.

## 4. Runtime / Formal Anchors
Runtime anchors:
1. `crates/oxfunc_core/src/functions/complex_family.rs`
2. `crates/oxfunc_core/src/functions/surface_dispatch.rs`
3. `tools/xll-addin/oxfunc_xll/export_specs.csv`

Formal anchors:
1. `formal/lean/OxFunc/Functions/ComplexFamily.lean`

Native replay anchors:
1. `docs/function-lane/W60_SCENARIO_MANIFEST_SEED.csv`
2. `tools/w60-probe/run-w60-complex-family-baseline.ps1`
3. `.tmp/w60-complex-family-results.csv`

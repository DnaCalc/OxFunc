# ATAN arithmetic graph, 2026-09-29

State: `in_progress`; `scope_completeness=scope_partial`,
`target_completeness=target_partial`, `integration_completeness=partial`.
Owner: `oxf-mwue.28.12`, BUG-FUNC-061. No whole-function promotion is made.

The current x86_64 candidate reproduces **181,495 numeric observations and 841
typed observations** exactly. This total includes discovery and refinement;
the last **112,498 numeric inputs** were generated only after freezing the
revised candidate, excluding all earlier input bits. They span normal binary64
exponents, ordinary magnitudes, the unit branch boundary, and neighbors of
mathematically chosen output midpoints. High-precision mathematics selects
inputs only; live Excel supplies every expected answer.

The reference is 64-bit Excel 16.0 build 20430, workbook Compatibility Version 2,
on the recorded Windows/x86_64 host. Update channel and alternate platforms are
not established by these captures. Numerical arguments enter through Value2
cells referenced by formulas; typed captures retain fixtures, formula readback,
input bits and exact result digests. Altered signed-zero/subnormal ingress from
the initial broad packet is withheld, rather than turned into kernel semantics.

## Observed graph and failed alternatives

For magnitude at most one, use public FPATAN(x,1). For greater magnitude, compute
FPATAN(1,abs(x)) directly, retain that extended result and the extended pi/2
constant through subtraction, then store binary64 and restore the sign. There
is no binary64 reciprocal or intermediate angle publication in this branch.

The first direct-FPATAN candidate matched all 1,224 discovery rows, but the first
independent numeric sample had two one-ULP differences among 29,573 inputs.
The 38,200-row refinement sample reproduced those differences and found more.
Typed repetition confirmed the same input bits and outcomes. Precision-control,
rounding-control and CPU-affinity probes did not justify blaming instability.
Stored reciprocal, stored-angle, binary64-pi, standard-library and half-angle
alternatives are retained as rejected graphs. The extended inverse-angle graph
matches those observations and the later independent sample.

| Evidence stage | Numeric rows | Typed rows | Current exact matches |
| --- | ---: | ---: | ---: |
| Initial discovery | 1,224 | 0 | 1,224 |
| First independent capture | 29,573 | 297 | 29,870 |
| Refinement and repetition | 38,200 | 544 | 38,744 |
| Independent revised-candidate capture | 112,498 | 0 | 112,498 |

These are observation counts, not a claim of unique inputs across repeated
controls. Frozen candidates, initial failed results, capture provenance and
graph comparisons remain separate artifacts.

## Replay and formal alignment

Four numerical integration tests replay every retained admitted result through
production dispatch; two typed tests replay the qualified typed banks. The
corresponding validation logs are retained in `validation/`. The Lean model
executes the magnitude branch and sign restoration with explicitly supplied
direct and extended-difference primitives. Its binding and boundary theorems
pass; it does not silently substitute Float.atan or binary64 pi for the observed
primitive. Public ISA operations and black-box Excel observations are the only
sources for this characterization; no Microsoft binary was inspected.

Open lanes: formal identity of the numerical primitives, non-x86_64 fallback and
alternate CPU/version validation, context-dependent numeric text preparation,
uncommon reference/evaluator carriers, and combined campaign verification. The
shared arctangent helper also has separately retained ACOT and imaginary-family
caller evidence; their counts are not included above.

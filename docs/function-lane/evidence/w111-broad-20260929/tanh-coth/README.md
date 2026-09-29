# TANH and COTH composition

`scope_completeness=scope_partial`, `target_completeness=target_partial`,
`integration_completeness=partial`.

Open lanes: observed SINH-dependent numerical inputs, new independently observed SINH-dependent
composition failures, broader function contexts and platform
coverage. No whole-function parity claim is made.

Public Excel captures are retained under the adjacent `hyperbolic/discovery`
and `hyperbolic/heldout` directories, with application and compatibility version
provenance. The research uses published arithmetic operations and black-box
results only. It does not inspect Excel binaries.

For absolute input below one, the observed TANH denominator uses the
cancellation-safe pair `expm1(x)` and `expm1(-x)`, rather than the independently
published COSH result. The candidate uses these steps:

1. Add the two expm1 results with RN64 arithmetic and a binary64 store.
2. Add two with RN64 arithmetic and a binary64 store, then divide by two.
3. Divide the existing SINH result by this denominator with RN64 arithmetic
   and a binary64 store.

For absolute input at least one, the denominator remains the existing COSH
result; the final division has the same RN64/store sequence. COTH retains its
existing staged reciprocal of the resulting TANH value. Existing zero errors
and saturation remain in their publication adapters.

The recorded research compares retained versus stored sums, EXP versus expm1
denominators, and final division variants. Ordinary published SINH/COSH division
matches 8,965/8,990 discovery and 22,414/23,102 subsequent TANH observations. The
refined composition matches 8,990/8,990 and 23,096/23,102. Its six remaining
inputs are the same five tiny values near 1.5e-16 and one value near -0.23367
that still differ in SINH. COTH matches 8,990/8,990 and 23,095/23,100, retaining
five of those shared numerical discrepancies.

`composition-graph-research.json` retains every best-candidate failure and the
prior production failures. `research-w111_tanh_lifetimes.rs` freezes the exact
research source. The production replay test runs all 64,182 observations and
pins the 11 mismatches explicitly; two focused tests pass, with 64,171 exact
outcomes. Inputs that would be changed by numeric Excel ingress are not present
in these qualified captures.

`candidate-freeze.json` and its source copies precede generation of the fresh
7,142-input packet for each function. It includes independent small inputs,
normal bit patterns, exponential near-one cases, neighboring boundary bits,
and ordinary/overflow controls. All six frozen production/dependency hashes
were verified unchanged before replay. TANH matches 7,090/7,142 and COTH matches
7,092/7,142. Every failed independent row is retained and pinned, rather than
reclassified as a match. TANH has 50 failures around 1.5e-16 and a signed pair
near 1e-8; COTH has 48 and the same signed pair. Direct SINH/COSH observations
at these inputs and their neighbors now confirm that every one of the 52 failed
TANH inputs is a SINH mismatch: the local denominator is exactly one and Excel
TANH equals Excel SINH exactly. The paired 244-case dependency captures and
local replay are retained as `dependency-*`. No input-specific correction is
introduced. `independent-replay.json` records the unchanged
production outcomes and alternative-graph counts. The additional replay test
now exercises 78,466 total observations: 78,353 exact and 113 explicit failures.

The shared executable Lean composition was aligned by the campaign coordinator;
its focused build passes (10 jobs). The EXP/expm1/SINH primitives
remain empirical dependencies, with the observed SINH residuals explicitly open.

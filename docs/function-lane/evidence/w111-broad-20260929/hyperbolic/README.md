# Hyperbolic arithmetic and publication observations

W111, BUG-FUNC-068, bead `oxf-mwue.28.18`. Reference: Excel 16.0 build 20430,
64-bit, Compatibility Version 2; channel unverified. All admitted input bits
were supplied through Value2. Only public interfaces and arithmetic hypotheses
are used.

The discovery packet has 8,990 observations per function. After candidate
freezing, a separately seeded packet supplies 23,100 fresh inputs per function
and 23,102 for TANH; prior input bits were excluded by the generator. Candidate
sources and capture manifests are retained beside the exact answers.

| Function | Discovery candidate | Fresh independent candidate | Remaining observation |
|---|---:|---:|---|
| COSH | 8,990 / 8,990 | 23,100 / 23,100 | No difference in these numeric banks |
| SINH | 8,990 / 8,990 | 23,094 / 23,100 | Six small-input arithmetic differences |
| CSCH | 8,990 / 8,990 | 23,094 / 23,100 | The same six inherited SINH differences |
| TANH | 8,965 / 8,990 | 22,414 / 23,102 | Small-input composition remains partial |
| COTH | 8,967 / 8,990 | 22,453 / 23,100 | Inherits TANH composition differences |

The COSH candidate adds separately published EXP values through an extended
precision operation followed by a binary64 store. SINH uses a similarly staged
EXP difference when the magnitude is at least one and retains the observed
expm1 pair below one. CSCH uses a staged reciprocal and publishes tiny results
as positive zero. These rules describe the candidate's observable graph, not
the internals of Excel. The failed fresh rows remain evidence for further
discriminating probes; no input-specific corrections are used.

TANH's raw ratio can be nonfinite after EXP overflow even for finite input.
Observed Excel outputs saturate to the input sign. The declared result policy
and numeric Q dispatcher now agree with the scalar and array paths; all six
unary equivalence laws pass. Its metadata fixture records that evidenced policy.
The policy repair does not resolve the small-input numerical differences.

The subsequent TANH denominator uses the expm1 pair below magnitude one, with
staged additions and division. It matches all 8,990 discovery cases and
23,096/23,102 prior independent cases; COTH matches 8,990 and 23,095/23,100.
A fresh frozen 7,142-row bank per function then matches 7,090 TANH and 7,092
COTH cases. All 52 new TANH failures equal the directly observed SINH result
with COSH exactly one. The 244 neighboring dependency controls match COSH
entirely and expose 124 SINH differences. Those failed banks remain explicit
discovery for the next refinement. See `../tanh-coth/` for exact lineage and
correlations; no input-specific correction is used.

`HyperbolicComposition.lean` binds the numerical primitives explicitly and
executes publication and composition rules. It is imported by each function's
formal module. Primitive identity and other-platform behavior are not proved by
those bindings. The focused root model build passed nine jobs, followed by a
20-job function-binding build. The changed COSH dependency also passes the three
retained SECH replay tests, including both independent cohorts.
The refined TANH graph and bindings subsequently pass ten focused Lean jobs.

Research commands and hypotheses live in
`crates/oxfunc_core/examples/w111_hyperbolic_graphs.rs`; the authoritative fresh
public-dispatch judgement is retained at
`../integer-publication/validation/w111-integer-hyperbolic-heldout-judgement.json`.
`artifact-manifest.json` records retained file hashes.

`scope_completeness=scope_partial`, `target_completeness=target_partial`,
`integration_completeness=partial`. Open lanes: stated small-input residuals,
additional typed/host coverage, numerical primitive alignment, combined
validation and any receiving-side declaration impact. A passing numeric bank
does not promote a whole function.

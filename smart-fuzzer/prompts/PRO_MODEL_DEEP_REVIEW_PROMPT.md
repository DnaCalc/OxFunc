# Pro Model Deep Review Prompt

Use this prompt when asking a stronger model for a deep design review of the
OxFunc smart-fuzzer concept. The output is advisory input only; it does not
become OxFunc doctrine without ordinary evidence and synthesis.

```text
You are reviewing a proposed smart-fuzzer for OxFunc, a clean-room Rust
implementation lane for Excel worksheet function semantics. OxFunc contains
hundreds of Excel functions, rich function metadata, a typed value universe,
function contracts, existing scenario manifests, bug streams, and limited Lean
checking. OxFml owns formula parse/bind and the single-node evaluator seam, and
OxFunc owns value/function semantics.

Clean-room rule: use only public documentation, published research, and
reproducible black-box Excel observations. Do not rely on proprietary internals
or reverse engineering.

Goal:
Design a smart-fuzzer that explores Excel function invocation space to find
cases where OxFunc/OxFml differ from Excel, while producing honest coverage and
confidence metrics. The invocation space includes roughly 500 worksheet
functions/operators, many arities, scalar values, arrays, references, omitted
arguments, blanks, errors, callable values, host/provider contexts, locale and
workbook compatibility axes, and function-specific quirks. Rust local evaluation
is fast; Excel comparison is much slower but can run in batches.

Important OxFunc doctrine:
1. sampled agreement is not semantic closure;
2. mismatches must become minimized replayable evidence;
3. function-semantic failures must be separated from seam, harness, host, and
   admission failures;
4. version axes must be captured: Excel app version/channel and workbook
   Compatibility Version;
5. omitted argument, missing argument, blank cell, empty text, error, scalar,
   direct array, opaque array, reference, and multi-area reference are distinct;
6. public docs may differ from empirical Excel behavior, and empirical behavior
   wins when reproducibly observed.

Current proposed architecture:
1. static indexer over function metadata, contracts, scenario manifests, bug
   streams, and Rust function source risk signals;
2. typed generator that produces structured invocation records and formula text;
3. fast local Rust/OxFml-adapter evaluator;
4. candidate prioritizer using local outcome novelty, static risk, known bug
   adjacency, stale-claim review signals, and coverage deficits;
5. batched Excel runner capturing Formula2, Value2, Text, spill shape,
   version/build/channel, workbook compatibility, runner version, manifest hash,
   and git revision;
6. typed comparator with family-specific numeric policies and exact error/shape
   comparisons;
7. minimizer that reduces formulas, arrays, fixtures, references, and numeric
   values while preserving the mismatch;
8. agent loop that suggests generator tactics and reviews mismatch clusters but
   never acts as source-of-truth.

Please provide a rigorous design critique and improvement plan. Address:
1. the best initial pilot surface and why;
2. a practical throughput benchmarking design for Excel batch evaluation;
3. how to allocate Excel budget between broad coverage, live PMT/PPMT risk,
   and stale-claim review rows that require fresh confirmation;
4. generator strategies by function family and value type;
5. static source-risk features that are useful without overfitting;
6. coverage metrics that are meaningful but do not overclaim;
7. comparator policies for numeric exactness, arrays, errors, rich values, and
   display/value splits;
8. minimization strategies for formulas with references, arrays, omitted args,
   and host/provider context;
9. how to separate OxFunc function bugs from OxFml seam bugs and Excel harness
   limitations;
10. artifact schemas and run metadata that should be captured from the start;
11. how agents should interact with generated evidence safely;
12. failure modes and design traps likely to waste time or create false
   confidence;
13. a staged implementation sequence with gate criteria, expressed by dependency
   and evidence readiness, not calendar dates.

Be concrete. Prefer mechanisms, schemas, ranking formulas, and examples over
general testing advice. Call out assumptions that need empirical validation.
```

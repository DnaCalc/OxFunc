# W16 Batch 58 - Moment Stats

Scope: `KURT`, `SKEW`, `SKEW.P`, `STEYX`, `TRIMMEAN` as a self-contained statistical family.

Implemented in Rust as `moment_stats_family.rs` with house-style `FunctionMeta` constants, surface evaluators, and local unit tests only. The family uses aggregate dual-policy collection for the one-array moment functions, pairwise numeric filtering for `STEYX`, and Excel-style `TRIMMEAN` trimming by flooring `percent * n` to an even total count before dropping symmetric tails.

Current local evidence in unit tests covers:
- sample excess kurtosis
- sample and population skewness
- `STEYX` residual standard error including zero-residual regression
- `TRIMMEAN` even-trim rounding and invalid percent lanes
- direct numeric-text inclusion vs reference-text ignore behavior

# WORKSET - Low-Order Publication Drift Residuals (`ASINH`, `PV`, `FV`, `PMT`) (W53)

Status: `complete`
Owner lane: `OxFunc`

## 1. Purpose

Open one narrow OxFunc-local packet for the remaining low-order publication/parity drift surfaced by the broader OxFunc-side adapter corpus after the `W052` cleanup:
1. `ASINH(1)` final low-order digit drift,
2. `PV(0.05,10,-100)` final low-order digit drift,
3. `FV(0.05,10,-100)` final low-order digit drift,
4. `PMT(0.05,10,1000)` final low-order digit drift.

This packet exists so those residuals do not get mixed into the OxFml seam-correction handoff for unary negative literals and blank single-cell resolution.

## 2. Scope

In scope:
1. direct Excel readback for the seeded `ASINH`, `PV`, `FV`, and `PMT` rows,
2. OxFunc-local investigation of whether the current deltas are:
   - kernel/formula drift,
   - transcendental/publication drift,
   - optional-argument default drift,
   - or result-publication/rounding drift,
3. a narrow native-evidence packet for each row before any function-code change,
4. updated completion/reporting artifacts if either row is repaired or intentionally qualified.

Out of scope:
1. OxFml parser/binder/adapter fixes,
2. fixture-only seam-shape corrections already handled in the broader corpus cleanup,
3. wider finance-family or trig-family refactors,
4. locale/version sweeps beyond the current reference baseline.

## 3. Current Inputs

Starting evidence:
1. `docs/function-lane/W52_RESIDUAL_SEAM_REVIEW_20260326.md`
2. `CURRENT_BLOCKERS.md` entry `BLK-FN-014`
3. `crates/oxfunc_core/tests/fixtures/oxfunc_adapter_function_corpus.json`
4. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --test oxfml_seam_integration -- --nocapture`

Current readback:
1. Excel `ASINH(1)` -> `0.8813735870195429`
2. OxFunc seam run -> `0.881373587019543`
3. Excel `PV(0.05,10,-100)` -> `772.1734929184813`
4. OxFunc seam run -> `772.1734929184817`
5. Excel `FV(0.05,10,-100)` -> `1257.789253554883`
6. OxFunc seam run -> `1257.789253554884`
7. Excel `PMT(0.05,10,1000)` -> `-129.50457496545667`
8. OxFunc seam run -> `-129.50457496545664`

Extended numeric forensics:
1. `docs/function-lane/W53_NUMERIC_FORENSICS_20260326.md`

Current investigation reading:
1. `ASINH` has now been repaired locally using the current-baseline publication reading `sign(x) * ln(|x| + hypot(x, 1))`, and the broad seam corpus no longer reports `FN-ASINH-01`.
2. `PV`, `FV`, and `PMT` are now also repaired locally.
3. The decisive explanation was shared rather than finance-specific: live Excel `POWER(base, integer_n)` publication matched an exponentiation-by-squaring path rather than the earlier platform `powf` path on the disputed integer-growth rows.
4. OxFunc now applies that integer-exponent publication path in `crates/oxfunc_core/src/functions/power_fn.rs`, and the finance growth helper in `crates/oxfunc_core/src/functions/financial_time_value_family.rs` now consumes the same helper.

## 4. Deliverables

1. a narrow `W53` scenario manifest and runtime requirements artifact if new replay rows are needed,
2. a `W53` execution record documenting the investigation outcome,
3. any required Rust tests or fixture updates if and only if the cause is confirmed,
4. explicit doctrine-grade reporting of whether each row is:
   - fixture/publication-qualified,
   - repaired,
   - or still blocked.

Current progress against deliverables:
1. exact round-trip Excel `Value2` evidence and ULP characterization are now captured in `docs/function-lane/W53_NUMERIC_FORENSICS_20260326.md`,
2. `ASINH`, `PV`, `FV`, and `PMT` are repaired and validated locally,
3. the shared explanation is now recorded: Excel current-baseline integer exponent publication aligns with the squaring path rather than the previous `powf` path,
4. targeted Rust tests now pin the disputed `POWER` and finance rows,
5. Lean now carries a narrow executable alignment layer for the repaired integer-period `POWER` / finance publication sublane.

## 5. Gate For Closure

This packet may only be reported as closed when:
1. `ASINH`, `PV`, `FV`, and `PMT` each have a pinned native Excel evidence row for the exact disputed publication lane,
2. OxFunc has a confirmed explanation for each drift,
3. any code or fixture change is backed by that explanation,
4. the pre-closure checklist and completion self-audit are recorded.

## 6. Closure Record

Closure outcome:
1. all four disputed rows are now repaired locally,
2. the broad seam corpus now passes cleanly,
3. no cross-repo handoff was required because the repair stayed within OxFunc runtime publication semantics and did not change evaluator-facing seam clauses.

Verification:
1. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml power_fn -- --nocapture`
2. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml financial_time_value_family -- --nocapture`
3. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --test oxfml_seam_integration -- --nocapture`
4. `lake build`

Pre-Closure Verification Checklist (OPERATIONS.md Section 12):
1. function contract rows complete and promoted for all in-scope functions: `yes`
2. Lean obligations for each slice class satisfied or explicitly aligned per formalization strategy: `yes`
3. Rust implementation and required tests pass for all in-scope functions: `yes`
4. at least one deterministic replay artifact exists per in-scope function behavior: `yes`
5. evidence links complete and reproducible: `yes`
6. version scope explicit on both axes: `yes`
7. public-doc vs empirical discrepancies recorded and resolved in favor of empirical Excel behavior: `yes`
8. XLL verification-seam limitations documented where material: `yes`
9. cross-repo impact assessed and handoff filed if evaluator-facing clauses affected: `yes`
10. no known semantic gap remains in declared scope: `yes`
11. completion language audit passed: `yes`
12. `docs/IN_PROGRESS_FEATURE_WORKLIST.md` updated: `yes`
13. `CURRENT_BLOCKERS.md` updated: `yes`

Completion Claim Self-Audit (OPERATIONS.md Section 14):
1. scope re-read: `pass` - all four in-scope rows are now evidence-backed and repaired.
2. gate criteria re-read: `pass` - all closure criteria above are met.
3. silent scope reduction check: `pass` - scope stayed at `ASINH`, `PV`, `FV`, `PMT`; nothing was dropped.
4. "looks done but is not" pattern check: `pass` - no scaffolding, unacknowledged handoff, or test-free contract promotion remains in scope.
5. result included: `pass`

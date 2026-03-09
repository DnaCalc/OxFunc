# WORKSET - TUX1000 Coercion and Reference-Resolution Primitives (W4)

## 1. Purpose
Formalize conversion/coercion primitives and the explicit `Ref -> EvalValue` seam between OxFunc semantics and FEC-provided context services.

This workset hardens the most critical cross-boundary adapter contract in the kickoff program.

## 2. Kickoff Position and Dependencies
Kickoff role:
1. W4 in `W000_KICKOFF_PROGRAM_W001_W006.md`.

Dependencies:
1. depends on W3 value-universe closure.

Downstream consumers:
1. W5 `ABS` adapter/kernel correctness.
2. W6 `XMATCH` contract and implementation consistency.

## 3. Scope
In scope:
1. primitive coercion algebra (identity, numeric/text/logical, array lift, error propagation).
2. reference normalization and dereference policy primitives.
3. formal seam alternatives for reference resolution.
4. selected baseline seam with explicit tradeoff record.

Out of scope:
1. host scheduler/lifecycle implementation details (FEC ownership).
2. broad function-family rollout.

## 4. Seam Alternatives (Required to Document)
1. capability-record model,
2. abstract oracle-function model,
3. effect/monadic model.

Selection rule:
1. maximize proof composability and runtime traceability,
2. preserve contract stability if implementation strategy changes.

## 5. Deliverables
1. `docs/function-lane/COERCION_AND_CONVERSION_PRELIM_SPEC.md`
2. `docs/function-lane/REF_RESOLUTION_SEAM_OPTIONS.md`
3. `docs/function-lane/COERCION_DECISION_TABLE.csv`
4. `docs/function-lane/COERCION_SCENARIO_MANIFEST_SEED.csv`
5. `docs/function-lane/COERCION_PROBE_RUNTIME_REQUIREMENTS.md`
6. `docs/function-lane/COERCION_EXECUTION_RECORD.md`
7. `tools/coercion-probe/results/COERCION_RESULTS_TEMPLATE.csv`
8. Lean coercion primitives + seam abstraction
9. Rust coercion primitives + resolver interface
10. conformance-row linkage updates (`FDEF-029` and affected rows)

## 6. Gate Model
### G1 - Primitive Enumeration Closure
Pass when:
1. primitive operations are fully enumerated and typed.

### G2 - Seam Contract Closure
Pass when:
1. alternatives and tradeoffs are documented,
2. one baseline seam is selected with rationale.

### G3 - Executable Closure
Pass when:
1. Lean and Rust scaffolds compile for the selected seam.
2. minimum coercion cases are test-backed.

### G4 - Integration Closure
Pass when:
1. W5 and W6 reference the selected seam without policy ambiguity.

### G5 - Empirical Closure
Pass when:
1. coercion scenario manifest and runtime requirements are published with explicit expected fields.
2. baseline Excel run is recorded in `COERCION_EXECUTION_RECORD.md` with version/channel/compat descriptors.
3. mismatch/drift reporting fields are present in emitted coercion results.

## 7. Status
Execution state:
1. `complete`.

Claim confidence:
1. `provisional` (global aggregate precedence and external-reference seed lane remain explicitly bounded).

Gate snapshot:
1. `G1` primitive enumeration closure: `closed-provisional`.
2. `G2` seam contract closure: `closed` (selected baseline: `capability_record_model`).
3. `G3` executable closure: `closed` (Rust + Lean builds/tests passing for W4 scaffolds).
4. `G4` integration closure: `closed-provisional` (W5/W6 now reference selected seam baseline).
5. `G5` empirical closure: `closed-provisional` (baseline run captured with explicit mismatch ledger).


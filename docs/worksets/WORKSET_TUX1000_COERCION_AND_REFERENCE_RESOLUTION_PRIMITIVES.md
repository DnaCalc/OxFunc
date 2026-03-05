# WORKSET - TUX1000 Coercion and Reference-Resolution Primitives (W4)

## 1. Purpose
Formalize conversion/coercion primitives and the explicit `Ref -> EvalValue` seam between OxFunc semantics and FEC-provided context services.

This workset hardens the most critical cross-boundary adapter contract in the kickoff program.

## 2. Kickoff Position and Dependencies
Kickoff role:
1. W4 in `WORKSET_TUX1000_KICKOFF_PROGRAM_W1_W6.md`.

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
4. Lean coercion primitives + seam abstraction
5. Rust coercion primitives + resolver interface
6. conformance-row linkage updates (`FDEF-029` and affected rows)

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

## 7. Status
Execution state:
1. `planned`.

Claim confidence:
1. `draft` (pending baseline seam selection).

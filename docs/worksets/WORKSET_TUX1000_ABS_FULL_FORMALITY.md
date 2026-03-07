# WORKSET - TUX1000 ABS Full Formality (W5)

## 1. Purpose
Deliver the first nontrivial scalar function closure (`ABS`) using the complete OxFunc artifact chain and kickoff substrate outputs.

`ABS` is the proving slice for:
1. value taxonomy usage,
2. coercion seam usage,
3. floating-point edge policy integration,
4. array-lift and boundary semantics.

## 2. Kickoff Position and Dependencies
Kickoff role:
1. W5 in `WORKSET_TUX1000_KICKOFF_PROGRAM_W1_W6.md`.

Dependencies:
1. depends on W2 floating-point characterization,
2. depends on W3 value-universe closure,
3. depends on W4 coercion/resolver seam closure.

## 3. Scope
In scope:
1. full `ABS` contract (admission/coercion/kernel/array/error/boundary lanes).
2. Lean formal model obligations for the declared `ABS` slice.
3. Rust implementation and tests mapped to contract/theorem IDs.
4. empirical differential checks for unresolved edge behavior.

Out of scope:
1. broader numeric family closure.

## 4. Required Behavior Lanes
1. admission lane,
2. coercion lane (consumes W4 `capability_record_model` seam baseline),
3. numeric kernel lane,
4. floating-point lane (`-0`, subnormal, NaN/infinity mapping where observable),
5. array-lift lane,
6. formula-vs-reference boundary lane.

## 5. Deliverables
1. `docs/function-lane/FUNCTION_SLICE_ABS_CONTRACT_PRELIM.md`
2. conformance row updates (`FDEF-030` and related rows)
3. Lean module + theorem inventory for `ABS`
4. Rust implementation/tests for `ABS`
5. empirical evidence bundle references and promoted findings where needed
6. correlation-ledger row for `FUNC.ABS`

## 6. Gate Model
### G1 - Contract Closure
Pass when:
1. `ABS` contract fields are complete with explicit unknowns.

### G2 - Formal Closure
Pass when:
1. Lean obligations for declared scope compile and pass.

### G3 - Runtime and Verification Closure
Pass when:
1. Rust implementation compiles,
2. required test matrix passes.

### G4 - Evidence Closure
Pass when:
1. unresolved edge lanes have replayable evidence.
2. dual-axis version scope is explicit.

### G5 - Promotion Closure
Pass when:
1. `ABS` claim status is set with explicit scope and maturity statement.

## 7. Status
Execution state:
1. `planned`.

Claim confidence:
1. `draft` (pre-implementation).

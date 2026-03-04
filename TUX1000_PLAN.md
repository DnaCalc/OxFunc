# OxFunc TUX-1000 Plan

## 1. Purpose
`TUX1000_PLAN.md` is the aspirational execution adjunct to `CHARTER.md`.

It sets an intentionally high bar for formal, reproducible function semantics execution in a domain that is historically ambiguous. It is sequence-based and gate-based, not calendar-based.

## 2. North-Star Objective
Convert Excel function compatibility from folklore into an industrial assurance pipeline:
1. explicit contracts per function/operator,
2. executable Rust semantics,
3. Lean-checked formal obligations,
4. differential evidence against Excel behavior,
5. promotion gates that prevent hand-wavy claims.

## 3. TUX-1000 Commitments
1. No unscoped compatibility claims.
2. No proof claims without corresponding implementation/test obligations.
3. No implementation claims without linked contract rows.
4. No validated status without replayable evidence.
5. No silent ambiguity: unresolved behavior is explicit and version-scoped.

## 4. Architecture of Execution
Every function/operator slice must traverse five synchronized artifacts:
1. Contract Artifact:
   - function row with semantic axes, FEC profile, and version scope.
2. Formal Artifact:
   - Lean model entry and theorem obligations.
3. Runtime Artifact:
   - Rust adapter/kernel implementation.
4. Verification Artifact:
   - tests (contract, differential, property/metamorphic as required).
5. Evidence Artifact:
   - source/evidence bindings with reproducible empirical payloads when needed.

## 5. Value-System Program (Foundational Workstream)
Before broad function closure, establish stable value semantics:
1. `ValueTag` algebra:
   - scalar, reference-like, array, error, blank, extended-value wrappers.
2. coercion contracts:
   - admission coercion, runtime coercion, domain rejection boundaries.
3. error algebra:
   - canonical worksheet error families and propagation rules.
4. adaptation contracts:
   - post-call adaptation including array-anchor and extended-value boundaries.

This is the reusable substrate for nearly all function proofs and runtime code.

## 6. Function Family Ladder
Execution is ordered by assurance leverage and dependency risk.

### 6.1 Stage T0 - Scaffold Integrity
1. establish row schema stability and function ids.
2. establish Lean/Rust skeletons and artifact linking.
3. establish status promotion mechanics.

Gate:
1. at least one full end-to-end slice completes all artifacts.

### 6.2 Stage T1 - Constant and Nullary Seed Family
Primary seed: `PI()`.

Gate:
1. `PI()` reaches validated status in declared profile scope.
2. slice template is reusable without ad-hoc edits.

### 6.3 Stage T2 - Pure Numeric Scalar Family
Examples: `SIN`, `COS`, `ASIN`.

Gate:
1. domain/error boundaries are formalized and tested.
2. refinement claims from Lean model to Rust behavior are established for selected seeds.

### 6.4 Stage T3 - Coercion-Heavy Aggregate Family
Examples: `SUM`, `SUMIF`-adjacent coercion lanes.

Gate:
1. direct-arg vs range-scan coercion policy is explicit and test-backed.
2. conflict lanes are version-scoped and evidence-backed.

### 6.5 Stage T4 - Reference/Context-Sensitive Family
Examples: `ROW`, `INDIRECT`, `OFFSET`, `XLOOKUP` return-shape lanes.

Gate:
1. FEC dependencies and trigger semantics are explicit.
2. context dependence is separated from non-determinism in row classifications.

### 6.6 Stage T5 - Volatile/Time/Random Family
Examples: `NOW`, `TODAY`, `RAND`, `RANDARRAY`.

Gate:
1. invalidation policy and output stability policy are separated and verified.
2. deterministic replay strategy for empirical evidence is explicit.

### 6.7 Stage T6 - External Provider Family
Examples: `RTD`, CUBE family.

Gate:
1. minimal lifecycle semantics are formalized.
2. replay packs cover connect/update/invalidate/disconnect scenarios.

### 6.8 Stage T7 - UDF Surface Contracts
Surfaces: XLL, VBA UDF, JS custom functions, Automation add-ins.

Gate:
1. argument/return mapping contracts and capability boundaries are explicit.
2. conformance rows include registration/signature/adapter semantics.

## 7. PI() First Demonstration Slice
The `PI()` slice is the mandatory proving ground for the full method.

Required artifacts:
1. contract row with final axis tags and FEC profile.
2. Lean semantics with theorems for:
   - totality,
   - determinism,
   - declared arity behavior.
3. Rust implementation with parity tests for the same obligations.
4. correlation record binding:
   - contract id,
   - Lean theorem ids,
   - Rust test ids,
   - evidence ids,
   - version axis metadata.

If this seed is not clean and reusable, broader ladder execution is blocked by policy.

## 8. Correlation Ledger Model
Every formalized function slice should have a machine-readable correlation record containing:
1. `function_id`
2. `contract_row_id`
3. `lean_module`
4. `lean_theorem_ids`
5. `rust_module`
6. `rust_test_ids`
7. `evidence_ids`
8. `excel_version_scope`
9. `compatibility_version_scope`
10. `status`

## 9. Quality Bar and Failure Policy
1. Any discovered divergence:
   - becomes a named replayable case,
   - receives a bounded classification (`spec gap`, `policy ambiguity`, `implementation defect`, `environmental variability`).
2. No downgrade of guarantees without explicit doc updates.
3. No profile promotion while mandatory slice-class obligations are unresolved.

## 10. Operating Posture
1. Keep slices small, complete, and composable.
2. Prefer one closed end-to-end proof chain over broad unverified coverage.
3. Expand coverage only when the method itself is stable.
4. Treat each validated slice as reusable infrastructure, not one-off output.

## 11. Relationship to Doctrine
1. `CHARTER.md` is normative for mission/scope/done criteria.
2. `OPERATIONS.md` is normative for lane execution doctrine.
3. This plan is aspirational and directional; it cannot override charter or operations doctrine.


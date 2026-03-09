# OPERATIONS.md - OxFunc Operations

## 1. Purpose
This document defines how OxFunc work is executed day-to-day: fast vertical slices, formal rigor, evidence-backed compatibility claims, and explicit promotion gates.

It adapts Foundation doctrine and the practical execution posture proven in sibling programs, but is specific to the function/value lane.

## 2. Precedence and Alignment
When guidance conflicts, precedence is:
1. `../Foundation/CHARTER.md`
2. `../Foundation/ARCHITECTURE_AND_REQUIREMENTS.md`
3. `../Foundation/OPERATIONS.md`
4. `CHARTER.md` in this repository
5. This document (`OPERATIONS.md`)
6. `TUX1000_PLAN.md` (aspirational execution plan; non-doctrinal where conflict exists).

Non-negotiable carry-over:
1. Clean-room evidence discipline.
2. Sequence-only planning doctrine (no date commitments in execution plans).
3. Dual-axis version tracking (Excel app version/channel + workbook Compatibility Version).

## 3. Operating Principles
1. Correctness with explicit semantics before optimization.
2. Compatibility claims require reproducible evidence.
3. No function is "done" until contract, implementation, tests, and evidence align.
4. Regressions are permanent assets (minimized replayable cases).
5. Prefer end-to-end vertical slices over broad speculative scaffolding.
6. Keep process lightweight, but never skip required traceability fields.
7. Report-back language must always qualify completeness by scope/target/integration axes.

## 4. Execution Model
OxFunc executes as coupled lanes for each function/operator slice.

### 4.1 Coupled Lanes
1. Contract lane:
   - function row semantics and policy status in `docs/function-lane/*`.
2. Formal lane:
   - Lean model obligations for value/function semantics.
3. Runtime lane:
   - Rust implementation for kernel and adapter behavior.
4. Verification lane:
   - contract tests, differential tests, property/metamorphic tests.
5. Evidence lane:
   - spec anchors and empirical findings with version-scope metadata.

### 4.2 Function Slice Lifecycle
For each function/operator `f`, use this lifecycle:
1. `define`:
   - assign stable id, signature/arity, class tags, FEC dependency profile.
2. `formalize`:
   - encode value-level and function-level semantics in Lean.
3. `implement`:
   - implement Rust kernel + adapter boundaries.
4. `verify`:
   - run local/CI test obligations and proof obligations for the slice class.
5. `correlate`:
   - bind expected behavior to source/evidence and Excel differential observations.
6. `promote`:
   - update status (`draft -> provisional -> validated`) only when gate criteria are met.

### 4.3 Mandatory Function Contract Fields
Every function/operator row must explicitly carry:
1. function id and title.
2. arity/admission contract.
3. pre-call coercion policy.
4. core semantic outcome model.
5. post-call adaptation policy.
6. determinism/volatility/host-interaction tags.
7. thread-safety classification (`safe_pure`, `host_serialized`, `not_thread_safe`).
8. `arg_preparation_profile` (`values_only_pre_adapter` or `refs_visible_in_adapter`).
9. `coercion_lift_profile` and `kernel_signature_class`.
10. adapter-level `fec_dependency_profile`, surface-level `surface_fec_dependency_profile`, and facility tags.
11. version scope:
   - Excel application version/channel scope.
   - workbook Compatibility Version scope.
12. evidence bindings (`spec`, `empirical`, policy decisions).

### 4.4 FEC/F3E Invocation Invariant
1. Function adapter/kernel execution must occur only after FEC-side admission has succeeded for the invocation context.
2. Seam-level rejections (`token`, `snapshot`, `capability`, or equivalent host admission outcomes) are classified as boundary outcomes, not function semantic failures.
3. Function semantic failures are only outcomes produced after successful admission and execution of the declared function pipeline.

## 5. Promotion Gates
Status transitions are mechanical, not narrative.

### 5.1 `draft -> provisional`
Requires:
1. complete row fields and explicit unknowns.
2. initial Lean/Rust skeleton wired to the slice.
3. minimum tests for known happy/error paths.
4. at least one admissible evidence anchor.

### 5.2 `provisional -> validated`
Requires:
1. required proof obligations for the slice class pass.
2. required Rust tests and differential probes pass.
3. empirical findings (where needed) are replayable and version-scoped.
4. unresolved behavior is explicitly bounded (not implicit).
5. all required requirement/evidence links are present.

## 6. Slice Classes and Required Obligation Depth
Required proof and test depth depends on slice class:
1. Class A: pure constant/no-arg functions.
   - totality, determinism, and arity invariants.
2. Class B: pure scalar numeric functions.
   - domain/error boundary, coercion boundary, refinement checks.
3. Class C: coercion-heavy aggregates.
   - range-scan vs direct-arg coercion matrix obligations.
4. Class D: context/reference-sensitive functions.
   - caller/reference/FEC facility obligations and trigger semantics.
5. Class E: external/provider-dependent functions.
   - lifecycle/state-machine and replay obligations.

## 7. PI() Starter Slice Rule (Normative Starter)
`PI()` is the first normative end-to-end slice for this repository.

Minimum obligations:
1. Contract row:
   - exact arity: 0.
   - class tags: deterministic, nonvolatile, host interaction `none`.
   - FEC profile: `none`.
2. Lean:
   - total evaluation theorem.
   - determinism theorem.
   - arity rejection/admission theorem for the declared policy surface.
3. Rust:
   - adapter + kernel implementation.
   - deterministic repeatability tests.
   - argument-count behavior tests.
4. Correlation:
   - Excel observation pack for `PI()` behavior under declared version axes.
   - linked evidence id(s) in conformance rows.

This template is the generalization seed for `SIN`, `SUM`, `ROW`, and later families.

## 8. Tooling and Run Hygiene
1. Use deterministic, replayable command lanes for proofs/tests/evidence extraction.
2. Store temporary generated files under repository-local `.tmp/` unless task-specific semantics require otherwise.
3. Behavior-critical tooling should follow Foundation language policy:
   - prefer stable .NET tooling for orchestration and empirical artifact emission.
4. Keep machine-readable artifacts first-class; narrative summaries are secondary.
5. Empirical scenario manifests must carry explicit expectation fields (`expected_status`, `expected_observable`) so intentional-failure cases are tracked as expected behavior, not ambiguous failures.
6. Probe runs must emit run-metadata sidecars (manifest hash, runner version, git revision, environment/version metadata) for replay integrity.
7. Probe suites should produce an analyzer report with mismatch and drift classification against prior baselines when available.
8. Cross-cutting worksets must instantiate a cross-boundary invariant checklist (formula, interop ingress, reference reuse, persistence/interchange, and any additional active boundaries).
9. Boundary evidence must report seam-level status and function semantic status separately so adversarial seam tests do not pollute function conformance failure counts.

## 9. Doctrine and Plan Change Workflow
1. Routine function-row and implementation updates do not require synthesis runs by default.
2. Doctrine-shifting changes (gate model, evidence policy, status semantics, profile rules) must include a logged synthesis/decision record.
3. `TUX1000_PLAN.md` is updated as execution intent evolves, but cannot override charter/doctrine.

## 10. Definition of Done (Per Function Slice)
A function slice is done for a declared profile only when:
1. contract row is complete and promoted to required status.
2. Lean obligations for the slice class pass.
3. Rust implementation and required tests pass.
4. evidence links are complete and reproducible.
5. version scope is explicit on both required axes.
6. known unknowns are explicit and policy-bounded.

## 11. Report-Back Completeness Contract
Every completion report (chat updates, execution records, workset closure notes, handoff summaries) must include:
1. `execution_state` (`planned|in_progress|blocked|complete`).
2. `scope_completeness` (`scope_complete|scope_partial`).
3. `target_completeness` (`target_complete|target_partial`).
4. `integration_completeness` (`integrated|partial`).
5. explicit `open_lanes` list when any completeness axis is partial.

Normative wording rules:
1. Use `complete for declared scope` when `scope_completeness=scope_complete` and any other completeness axis is partial.
2. Do not claim `fully complete` unless all three completeness axes are complete and evidence links are present.
3. If runtime export/dispatch admission is generated automatically at compile-time, still report whether each function is included in the source-of-truth export table for the claim.

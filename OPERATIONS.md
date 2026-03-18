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
3. No bounded-fit function implementations: partial semantic coverage is scaffolding/work-in-progress, not a completed function.
4. Where public documentation and empirical Excel behavior differ, implementation follows empirical Excel behavior and the divergence is logged explicitly.
5. In the current implementation phase, a function is `function-phase-complete` when contract, implementation, tests, evidence, and the Lean/formal work required by `docs/function-lane/FORMALIZATION_STRATEGY_EXECUTABLE_SEMANTIC_MODEL.md` align on full known semantics for the current reference Excel baseline. This required formal work is determined by the function's primary semantic substrate and admitted slice, and may be a substrate-level executable model, function binding, and alignment layer rather than a full duplicate Lean implementation. No known function-semantic gap may remain in current-phase scope.
6. Locale and alternate Excel-version sweeps are separate orthogonal validation phases unless explicitly declared in scope for the current workset.
6. Regressions are permanent assets (minimized replayable cases).
7. Prefer end-to-end vertical slices over broad speculative scaffolding, but do not misreport scaffolding as implementation closure.
8. Keep process lightweight, but never skip required traceability fields.
9. Report-back language must always qualify completeness by scope/target/integration axes.
10. XLL verification-seam limits must be documented centrally in the seam project and repeated in function verification records when they materially qualify a function claim.

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
7. `close`:
   - claim implementation closure only after full known Excel semantics are represented for the tracked version axes; bounded seed coverage remains work-in-progress.

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

### 8.1 Replay Appliance Rule
Replay bundles, adapter manifests, explain records, and witness lifecycle records are Logistics-layer artifacts.

Rules:
1. they are secondary evidence carriers and must not become a new OxFunc semantic authority,
2. packet-first and row-first artifacts are the honest default replay surface for OxFunc,
3. normalized event families may be projected for tooling convenience but must never fabricate a fake internal evaluator event stream,
4. XLL or host-bridge limitations must remain classified separately from core semantic failures inside replay diff/explain surfaces,
5. Replay adapter capability levels (`cap.C0` through `cap.C5`) are rollout claims about replay tooling support, not substitutes for function-semantic closure.

### 8.2 Silent AutoRun Protocol
When the user explicitly enables AutoRun for a named scope and sets an exit gate:
1. the agent must treat repository artifacts as the progress surface and chat output as suppressed,
2. no interim status, checkpoint, or partial-progress reply may be emitted before the exit gate is reached,
3. repeated user prompts such as `continue` are resume confirmations, not new report requests,
4. the only allowed pre-gate chat output is a blocker-only summary when every remaining in-scope path is blocked and the blocker ledger has been updated,
5. the first ordinary report after a silent AutoRun run must occur at the declared exit gate and must still satisfy the normal completeness-reporting rules.

## 9. Doctrine and Plan Change Workflow
1. Routine function-row and implementation updates do not require synthesis runs by default.
2. Doctrine-shifting changes (gate model, evidence policy, status semantics, profile rules) must include a logged synthesis/decision record.
3. `TUX1000_PLAN.md` is updated as execution intent evolves, but cannot override charter/doctrine.

## 10. Definition of Done (Per Function Slice)
A function slice is done for a declared profile only when:
1. contract row is complete and promoted to required status.
2. Lean obligations for the slice class, together with any substrate-level executable-model and alignment work required by the formalization strategy for the slice, pass or are otherwise satisfied explicitly for the function's admitted slice.
3. Rust implementation and required tests pass.
4. evidence links are complete and reproducible.
5. version scope is explicit on both required axes.
6. public-doc vs empirical discrepancies are recorded explicitly and resolved in favor of empirical Excel behavior.
7. no known semantic gap remains between OxFunc and the empirically determined Excel function behavior for the declared version scope.
8. any relevant XLL verification-seam limitation is explicitly documented in both seam-level and function-level verification records.
9. known unknowns are explicit and policy-bounded, except that function-semantic omissions are not acceptable for closure; only external XLL verification-seam limits may remain.

## 11. Report-Back Completeness Contract
Every completion report (chat updates, execution records, workset closure notes, handoff summaries) must include:
1. `execution_state` (`planned|in_progress|blocked|complete`).
2. `scope_completeness` (`scope_complete|scope_partial`).
3. `target_completeness` (`target_complete|target_partial`).
4. `integration_completeness` (`integrated|partial`).
5. explicit `open_lanes` list when any completeness axis is partial.

Normative wording rules:
1. Use `complete for declared scope` only when the declared function scope already represents full known Excel semantics for the tracked version axes and only integration or external-host limits remain partial.
2. Do not use `complete for declared scope` for semantically bounded function slices or function-batch worksets that still carry known Excel-semantic gaps; report those as `scope_partial`.
3. Do not claim `fully complete` unless all three completeness axes are complete and evidence links are present.
4. If runtime export/dispatch admission is generated automatically at compile-time, still report whether each function is included in the source-of-truth export table for the claim.
5. Use `function-phase-complete` for function slices that satisfy the current implementation-phase goal over the current reference Excel baseline, with no known function-semantic gap remaining, and with the Lean/formal work required by the formalization strategy for the function's primary semantic substrate attended to and aligned, even if later locale/version sweeps are still pending as orthogonal validation phases.

## 12. Pre-Closure Verification Checklist

Before claiming any workset or feature item as complete, answer each item yes or no.
All items must be "yes" for a completion claim. Any "no" means the item is `in_progress`.

| # | Check | Yes/No |
|---|-------|--------|
| 1 | Function contract rows complete and promoted for all in-scope functions? | |
| 2 | Lean obligations for each slice class satisfied or explicitly aligned per formalization strategy? | |
| 3 | Rust implementation and required tests pass for all in-scope functions? | |
| 4 | At least one deterministic replay artifact exists per in-scope function behavior? | |
| 5 | Evidence links complete and reproducible? | |
| 6 | Version scope explicit on both axes (Excel app version/channel + workbook Compatibility Version)? | |
| 7 | Public-doc vs empirical discrepancies recorded and resolved in favor of empirical Excel behavior? | |
| 8 | XLL verification-seam limitations documented in seam-level and function-level records where material? | |
| 9 | Cross-repo impact assessed and handoff filed if FEC/F3E boundary or evaluator-facing clauses affected? | |
| 10 | No known semantic gap remains in declared scope? | |
| 11 | Completion language audit passed (no premature "done"/"complete" per AGENTS.md anti-premature-completion rules)? | |
| 12 | `docs/IN_PROGRESS_FEATURE_WORKLIST.md` updated? | |
| 13 | `CURRENT_BLOCKERS.md` updated (new/resolved)? | |

## 13. Expanded Definition of Done

A workset or feature item is done for its declared scope only when all of the following hold:

1. **Existing Section 10 criteria**: all nine items in Section 10 are satisfied.
2. **Three-axis report**: completion report includes `scope_completeness`, `target_completeness`, `integration_completeness`, and `open_lanes` per AGENTS.md anti-premature-completion rules.
3. **Cross-repo impact**: impact on OxFml evaluator-facing clauses and FEC/F3E boundary is assessed; handoff packet filed if affected.
4. **Checklist attached**: Pre-Closure Verification Checklist (Section 12) is filled in and all items are "yes".

## 14. Completion Claim Self-Audit

Before submitting a completion claim, the agent must perform this self-audit and include the results.

### Step 1: Scope Re-Read
Re-read the workset scope declaration. For each in-scope item, verify that exercised implementation (not scaffolding) matches. Any missing item = `in_progress`.

### Step 2: Gate Criteria Re-Read
Re-read the workset gate criteria. All pass criteria must be met. Any unmet criterion = gate open.

### Step 3: Silent Scope Reduction Check
Compare the original scope declaration with what was actually delivered. Any unreported narrowing of scope is a doctrine violation. If scope was intentionally narrowed, it must be explicitly documented with rationale.

### Step 4: "Looks Done But Is Not" Pattern Check
Check for these patterns:
- Stubs or placeholder implementations reported as real.
- Insufficient test coverage masking untested paths.
- Contract text that does not match exercised implementation.
- Lean obligations claimed as satisfied without exercised evidence.
- Handoffs filed but not acknowledged by receiving repo.

### Step 5: Include Result
Include the self-audit result in the completion report with explicit pass/fail for each step.

## 15. Carried-Forward Operating Lessons

These five lessons are derived from observed execution failures in OxVba (86+ worksets) and OxFunc's own execution history (13 worksets, 38 function-phase-complete functions). They are not speculative — each addresses a real failure mode.

### Lesson 1: Scaffold Determinism Is a Gate
Scaffolding (stubs, empty traits, compile-only code) must produce deterministic outputs or be explicitly marked non-functional. Non-deterministic scaffolding that silently passes tests is a gate failure.
*Source: OxVba Lesson 1.*

### Lesson 2: Spec Drift Checks Run Alongside Implementation
Do not defer spec-vs-implementation consistency checks to a separate phase. Run them as part of each workset execution. Spec drift discovered late is expensive to reconcile.
*Source: OxVba Lesson 3.*

### Lesson 3: Final Validation Must Not Mutate Tracked Evidence
Validation runs must not modify the artifacts they are validating. Evidence mutation during validation invalidates the evidence chain.
*Source: OxVba Lesson 9.*

### Lesson 4: Guard Artifact Scope Before Commit
Before committing, verify that only intended artifacts are staged. Accidental inclusion of generated files, temporary outputs, or out-of-scope changes pollutes the evidence record.
*Source: OxVba Lesson 12.*

### Lesson 5: Partial Semantics Are Not Implementation
A function, protocol, or contract that covers a subset of its declared semantic space is work-in-progress, not an implementation. This applies even if the subset compiles, passes tests, and looks correct for the covered cases.
*Source: OxFunc doctrine decision (`docs/function-lane/DOCTRINE_DECISION_FULL_EMPIRICAL_FUNCTION_IDENTITY_20260309.md`).*

## 16. Upstream Observation Ledger Protocol

### 16.1 Purpose
Repos that interact with OxFunc discover interface and design constraints through their own implementation work. Those observations must flow through a structured channel so they inform design before contracts solidify.

This is distinct from handoff packets (registered in `docs/handoffs/HANDOFF_REGISTER.csv`), which propose specific normative text changes. Observation ledgers are standing documents that accumulate design feedback over time.

### 16.2 Outbound Observations (OxFunc -> OxFml)
When OxFunc implementation work reveals design constraints that affect OxFml, write them to `docs/upstream/NOTES_FOR_OXFML.md` following this structure:

1. **Purpose**: what the consuming repo needs to know and why.
2. **Core message**: the essential design constraint in 2-3 sentences.
3. **Current evidence**: specific examples with concrete scenarios.
4. **Interface implications**: what the receiving repo must preserve, avoid, or expose.
5. **Minimum invariants**: binary testable statements.
6. **Open questions**: explicit questions the receiving repo should answer.

### 16.3 Inbound Observations (OxFml -> OxFunc)
OxFunc must check for inbound observation ledgers from OxFml at the start of any interface or design workset. Known source location:

| Source repo | Ledger location | Relationship |
|-------------|----------------|--------------|
| OxFml | `../OxFml/docs/upstream/NOTES_FOR_OXFUNC.md` | Evaluator-facing interface constraints |

### 16.4 Format
Observation ledger entries follow the structure in Section 16.2. Each entry should be self-contained, version-scoped, and traceable to specific implementation evidence.

### 16.5 Lifecycle
1. Observation ledgers are living documents — updated as new evidence accumulates.
2. Entries are never silently removed; outdated observations are marked superseded with rationale.
3. When an observation is addressed by the receiving repo (through spec changes, interface decisions, or handoff packets), the originating entry is updated with a resolution reference.
4. Observation ledgers are not completion artifacts — they do not close worksets or satisfy gate criteria. They are design inputs.

### 16.6 Agent Obligation
Agents starting work on OxFunc interface or contract design must:
1. Check all listed inbound observation sources (Section 16.3).
2. Note any unresolved observations that are relevant to current scope.
3. Include a "reviewed inbound observations" line in the workset status report.
4. When a design decision addresses an inbound observation, reference the observation entry explicitly.

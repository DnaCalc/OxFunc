# OxFunc Charter

## 1. Mission
OxFunc is the function-semantics and implementation lane for DNA Calc worksheet compatibility.

Its mission is to define, formalize, implement, and verify worksheet value and function behavior so compatibility claims are explicit, auditable, and reproducible.

OxFunc converts function behavior from folklore into:
1. explicit formal semantics,
2. executable implementation contracts,
3. machine-checkable proof obligations,
4. reproducible conformance evidence.

## 2. Doctrine Alignment and Values
When guidance conflicts, precedence is:
1. `../Foundation/CHARTER.md` (program doctrine),
2. `../Foundation/ARCHITECTURE_AND_REQUIREMENTS.md` (architecture constraints),
3. `../Foundation/OPERATIONS.md` (program operations),
4. `CHARTER.md` in this repository (OxFunc lane scope and execution constraints under Foundation doctrine),
5. `OPERATIONS.md` in this repository,
6. `TUX1000_PLAN.md` in this repository (aspirational, non-overriding).

Values ordering for OxFunc decisions:
1. Correctness with explicit semantics.
2. Compatibility with worksheet-observable behavior.
3. Reproducibility and evidence provenance.
4. Throughput and automation velocity.
5. Presentation elegance.

Mandatory carry-over from Foundation doctrine:
1. clean-room evidence discipline,
2. coupled assurance stack (spec/model/proof/test/evidence),
3. sequence-only planning (no date-commitment planning),
4. profile-scoped claims with explicit version context,
5. regressions as replayable permanent assets.

Non-negotiable OxFunc-specific doctrine:
1. function implementation targets full semantic identity with Excel for the declared version axes,
2. bounded or seed-only semantic coverage is scaffolding, not implementation closure,
3. when public documentation and empirical Excel behavior differ, OxFunc records the discrepancy and implements the empirically observed behavior,
4. the only allowed limitation is in the XLL test/verification seam, where host-surface reproduction may be incomplete even though OxFunc runtime semantics must still target full Excel parity,
5. XLL verification-seam limitations must be documented centrally in seam artifacts and repeated in function verification records wherever those limitations materially qualify a function claim.
6. in the current implementation phase, a function may be reported as `function-phase-complete` when its reference-baseline semantics are characterized with high confidence, the function/evaluation seam is understood and documented, the Rust implementation is thorough and tested, the Lean description covers the intended slice, and no known function-semantic gap remains in current-phase scope.
7. locale and alternate Excel-version sweeps are orthogonal validation phases unless a workset explicitly declares them in scope; they do not by themselves prevent `function-phase-complete` status.

## 3. Clean-room Rule (Non-negotiable)
Allowed inputs:
1. public specifications/documentation,
2. published research,
3. reproducible black-box observation of Excel behavior.

Not allowed:
1. proprietary or restricted materials,
2. reverse engineering of internals,
3. decompilation/disassembly of Office binaries.

## 4. Scope
In scope:
1. OxFunc as the F3E value/function slice:
   - worksheet value type semantics,
   - function/operator semantics,
   - function-level FEC capability declarations.
2. Built-in function universe and operator-as-function (`OP_*`) semantics.
3. Function call boundary semantics:
   - admission,
   - coercion,
   - kernel behavior,
   - post-call adaptation.
4. Value-universe formalization:
   - scalar/error/array/reference-like/extended-value lanes,
   - versioned edge behavior and ambiguity handling.
5. UDF surface contract semantics (XLL/VBA/JS/Automation) at function boundary depth.
6. Lean-facing formal model slices and proof obligations for selected families.
7. Rust implementation of value/function kernels and adapters.
8. Empirical edge characterization and replay artifacts for unresolved/spec-thin lanes.
9. Dual-axis version behavior tracking:
   - Excel app version/channel,
   - workbook Compatibility Version.

Out of scope:
1. Formula grammar/parse/bind ownership (OxFml lane).
2. Full FEC scheduler/protocol/lifecycle ownership (Foundation model lane).
3. Workbook-level scheduling semantics and engine concurrency internals.
4. Power Query/M, DAX, MDX internals.
5. Full VBA runtime semantics.

## 5. Boundary Contract (FEC/F3E)
Normative OxFunc boundary commitments:
1. F3E owns value semantics.
2. FEC provides context capabilities and host lifecycle policy.
3. OxFunc defines function-facing declarations (`deterministic`, `volatile`, `host-interaction`, `fec_dependency_profile`, capability tags).
4. FEC consumes those declarations for invalidation/scheduling/publication policy.
5. Any seam ambiguity is logged as an explicit boundary decision, never silently absorbed.

Implementation-seam rule:
1. OxFunc contracts must remain compatible with the active Foundation FEC/F3E interaction model.
2. Supported interaction shapes may include either:
   - `CompileFormula -> DeclareDependencies -> Evaluate -> Publish/Render`, or
   - `prepare -> open_session/capability_view -> execute -> commit`.
3. In all supported shapes, function-library invocation occurs only after FEC admission and capability decision.

## 6. Required Artifact Stack
Every promoted function slice must carry synchronized artifacts:
1. Contract artifact (`docs/function-lane/*` rows/spec notes).
2. Formal artifact (Lean module + theorem inventory).
3. Runtime artifact (Rust kernel/adapter implementation).
4. Verification artifact (contract/differential/property tests as required).
5. Evidence artifact (spec and/or empirical source bindings with reproducible provenance).
6. Correlation artifact (machine-readable linkage across the five artifacts).

## 7. Status and Gate Semantics
OxFunc uses three orthogonal status planes.

### 7.1 Execution State
For worksets and slice execution flow:
1. `planned`
2. `in_progress`
3. `blocked`
4. `complete`

### 7.2 Contract Confidence State
For function-definition rows/claims:
1. `draft`
2. `provisional`
3. `validated`

### 7.3 Assurance Maturity State
Mapped to Foundation pack language:
1. `exercised`: OxFunc-local artifacts and checks pass.
2. `green-validated`: required Foundation-level packs/evidence are complete.

Rule:
1. OxFunc may mark a slice `validated` only with explicit scope and evidence.
2. Program-level profile-green claims require `green-validated` Foundation pack closure.

### 7.4 Completeness Reporting Semantics (Mandatory)
All report-back messages and execution records must separate completion claims across these axes:
1. `execution_state`:
   - planned/in_progress/blocked/complete.
2. `scope_completeness`:
   - `scope_complete`: all obligations for the declared slice/profile scope are done.
   - `scope_partial`: some declared-scope obligations remain open.
3. `target_completeness`:
   - `target_complete`: no declared out-of-scope semantic lanes remain for the target behavior set.
   - `target_partial`: one or more semantic lanes are explicitly bounded/deferred.
4. `integration_completeness`:
   - `integrated`: admitted in all declared runtime surfaces (for example core dispatch/export lanes) for the claim.
   - `partial`: implemented but not admitted in all declared runtime surfaces.

Function-phase status term:
1. `function-phase-complete`:
   - allowed for function slices that satisfy the current implementation-phase goal over the current reference Excel baseline.
   - requires no known function-semantic gap in current-phase scope.
   - does not imply that later locale/version validation sweeps are finished.

Language rule:
1. Do not use unqualified "done/complete" claims.
2. Use `complete for declared scope` only when the declared function scope already represents full known Excel semantics for the tracked version axes and only integration or external-host limits remain partial.
3. Do not use `complete for declared scope` for semantically bounded function slices or packets; those remain `scope_partial`.
4. Always list explicit open lanes when `target_completeness = target_partial` or `integration_completeness = partial`.
5. Use `function-phase-complete` for a function only when the current implementation-phase goal is satisfied and the only remaining work is orthogonal validation-phase expansion (for example locale/version sweeps) or external XLL seam hardening.

## 8. Kickoff Program and Dependency Intent
Current kickoff bundle is the ordered `TUX1000` workset chain (`W1..W7`):
1. `PI()` method seed,
2. floating-point behavior characterization,
3. value-universe closure,
4. coercion and reference-resolution seam closure,
5. `ABS` full-formality slice,
6. `XMATCH` deterministic-quirks closure,
7. string normalization/comparison characterization.

Dependency policy:
1. W3 depends on W2 outputs.
2. W4 depends on W3 closure.
3. W5 depends on W2 + W3 + W4.
4. W6 depends on W3 + W4 + W7 (and consumes W2 numeric-edge outcomes).
5. W3 may begin before W7 closure but must absorb W7 outputs before W3 validation closure.

## 9. Definition of Done
A function slice is done for declared scope only when all hold:
1. coverage: explicit id and complete contract fields.
2. traceability: contract/formal/runtime/test/evidence linkage is machine-readable.
3. formalization: required theorem obligations for its slice class pass.
4. runtime: Rust implementation and required tests pass.
5. evidence: source bindings and empirical findings (where needed) are replayable.
6. version context: both required axes are explicit.
7. semantic identity: no known semantic gap remains between OxFunc and empirically determined Excel behavior for the function over the declared version axes.
8. discrepancy handling: any public-doc vs empirical divergence is explicitly recorded and resolved in favor of empirical Excel behavior.
9. XLL limitation disclosure: any XLL verification-seam limitation relevant to the function claim is explicitly documented in both seam-level and function-level verification records.
10. boundaries: unresolved behavior is explicit and policy-bounded, except that function-semantic omissions are not treated as acceptable closure; only XLL verification-seam limits may remain external to the function implementation claim.
11. maturity: status and assurance maturity are clearly stated (`draft/provisional/validated` and `exercised/green-validated`).

Completeness claim rule:
1. Any "done" claim must include completeness qualifiers from section 7.4.
2. For function slices, prefer `function-phase-complete` over bare "done" when the current implementation-phase goal is satisfied but later orthogonal validation sweeps remain.

## 10. Non-goal Guardrails
1. Do not claim behavior proof beyond stated contract scope.
2. Do not hide uncertainty behind broad compatibility language.
3. Do not overfit to a single Excel build while presenting unbounded claims.
4. Do not let large-catalog closure block small complete end-to-end slices.

## 11. Relationship to Operating and Aspirational Docs
1. `OPERATIONS.md` in this repository is the lane-level execution doctrine.
2. `TUX1000_PLAN.md` is the aspirational execution adjunct.
3. Workset files under `docs/worksets/` define sequence-level execution packets under this charter.
4. Foundation conformance/model docs remain authoritative for cross-program protocol and evidence governance.

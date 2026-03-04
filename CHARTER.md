# OxFunc Charter

## 1. Mission
OxFunc is the function-semantics and implementation program for DNA Calc worksheet compatibility.

Its mission is to define, formalize, implement, and verify the Excel worksheet function universe so that compatibility is a mathematically grounded, auditable fact.

This side-quest exists to convert function behavior from "folklore plus probes" into:
1. explicit formal semantics,
2. executable conformance artifacts,
3. mechanically checked proofs for selected high-value function families,
4. production-grade Rust implementations with explicit correctness obligations.

In the Fire Horse year and agentic coding era, the ambition is direct:
1. move faster than ad-hoc reverse engineering,
2. keep clean-room discipline intact,
3. make correctness composable across models, engines, and evidence packs.

## 2. Values Ordering
When values conflict, higher-ranked values win.

1. Correctness with explicit semantics
2. Compatibility with worksheet-observable Excel behavior
3. Reproducibility and evidence provenance
4. Throughput/automation velocity
5. Elegance of formal presentation

## 3. Clean-room Rule (Non-negotiable)
This effort uses only:
1. public specifications/documentation,
2. published research,
3. reproducible black-box observation of Excel behavior.

Excluded:
1. proprietary/restricted material,
2. reverse engineering of Excel internals,
3. decompilation/disassembly of Office binaries.

## 4. Scope
In scope:
1. OxFunc as the F3E value/function slice:
   - owns worksheet value type semantics,
   - owns function/operator semantics and contracts,
   - declares only the FEC capability dependencies required by those semantics.
2. Built-in worksheet function universe (catalog + signatures + behavior classes).
3. Operator-as-function universe (`OP_*`) where operators are evaluable semantics.
4. UDF function-surface contracts:
   - XLL/C API,
   - VBA UDF,
   - JavaScript custom functions,
   - Automation add-in surface (lower depth allowed).
5. Function-level semantics dimensions:
   - argument admission vs runtime error boundary,
   - coercion and domain/error behavior,
   - array-lift/error propagation behavior,
   - determinism/volatility/host-interaction classes,
   - FEC dependency declarations.
6. Worksheet value type system and function-call boundary semantics (value vs extended value, admission/coercion/return adaptation).
7. Function behavior-class declarations used at the FEC boundary (`deterministic`, `volatile`, `host-interaction`, related tags), while FEC owns invalidation/scheduling policy execution.
8. Lean-facing formal model slices and proof obligations for selected families.
9. Rust implementations:
   - value type system runtime,
   - function implementations,
   - verification utilities and differential harnesses.
10. Proof-target mapping for concrete implementation sets (including external projects such as ExcelFinancialFunctions where relevant).
11. Empirical edge-case scanning and replay infrastructure for Excel-validation-backed spec closure.
12. Localization-aware behavior as an explicit axis (parse/format/coercion impacts), with staged depth.
13. Dual-axis version behavior tracking:
   - Excel application version/channel,
   - workbook Compatibility Version setting,
   for function availability and semantics.

Out of scope:
1. Full workbook scheduler semantics (belongs to core engine model).
2. Power Query/M, DAX, and MDX language internals.
3. Full VBA language runtime semantics (belongs to OxVBA).
4. Full localization-completeness closure in the first formalization wave (tracked, not ignored).
5. Formula language grammar/parse/bind ownership (belongs to OxFml lane).
6. Full FEC/F3E protocol and host-state lifecycle design (consumed from model docs; not owned here).

Boundary seam policy:
1. Some semantics are intentionally cross-cutting (for example volatility and context dependence).
2. OxFunc owns function-level declarations and contract meaning.
3. FEC owns host execution policy (invalidation/scheduling/publication behavior) that consumes those declarations.
4. Any ambiguous ownership discovered during implementation is logged and resolved explicitly, not silently absorbed.

## 5. Deliverables
1. Function universe registry:
   - stable function ids,
   - signature/arity contracts,
   - classification axes,
   - source/evidence bindings.
2. Function classification completeness set:
   - full built-in list,
   - interesting/non-interesting tiering,
   - dependency-profile tags (`FEC` capabilities).
3. Formal function semantics schema:
   - preconditions,
   - postconditions,
   - invariants,
   - explicit error/algebraic result model.
4. Formal worksheet value model:
   - value-tag algebra,
   - coercion/normalization contracts,
   - error/detail and array-lift semantics.
5. Function-adapter contract model:
   - parse/admission policy,
   - pre-call coercion,
   - core typed kernel,
   - post-call adaptation.
6. UDF surface conformance schema:
   - registration/signature mapping,
   - argument/return value class mapping,
   - caller/context and capability boundaries.
7. Lean proof lanes for selected profile slices.
8. Rust implementation crates/modules for value/runtime/function kernels.
9. Test scaffolding:
   - contract tests,
   - differential tests,
   - metamorphic/property tests,
   - regression corpus.
10. Empirical scaffolding for Excel validation:
   - scenario manifests,
   - replayable probes,
   - promoted finding records (`EMP-*` lineage).
11. Conformance pack bindings (spec + empirical) for every formalized claim.
12. Versioned behavior matrix for function semantics keyed by:
   - Excel application version/channel,
   - workbook Compatibility Version.

## 6. Proof Ambition
The goal is not "prove Excel." The goal is to prove that our implementations refine our formalized contracts for explicitly scoped behavior profiles.

Proof posture:
1. Kernel-first proofs:
   - prove pure typed kernels (for example trig and financial core equations/properties).
2. Adapter correctness proofs:
   - show coercion/admission/error wrappers preserve declared contracts.
3. Refinement proofs:
   - implementation result equals formal model result under declared preconditions.
4. Differential conformance evidence:
   - where public specs are thin, empirical evidence provides bounded target behavior that is explicitly version-scoped.
5. Rust-runtime consistency:
   - Rust function/value implementations are required to satisfy declared contracts and proof-side assumptions.

Financial-functions lane:
1. treat known external implementations (for example fsprojects/ExcelFinancialFunctions) as candidate comparative implementations,
2. formalize target contracts first,
3. prove equivalence/refinement for selected functions and tolerances where feasible.

## 7. Phased Execution
### Phase A: Universe Closure
1. Complete built-in catalog and UDF-surface taxonomy.
2. Freeze function ids and signature schema.
3. Record known unknowns and unresolved ambiguity lanes explicitly.
4. Establish Rust module scaffolding for values/functions and verification harness entry points.

### Phase B: Semantic Contracts
1. Write contract rows for non-interesting baseline families.
2. Separate admission/coercion/domain/array-lift concerns in each row.
3. Bind each row to source evidence and empirical probe obligations.
4. Define localization influence points (argument separators, decimal behavior, parse/coercion sensitivity) and mark deferred-depth rows.

### Phase C: Formal Core and Proof Scaffolding
1. Define Lean data model and contract encoding.
2. Add proof obligations for selected canonical functions (`SIN`, `SUM`, `ROW`) and one financial profile slice.
3. Connect contracts to executable reference implementations.
4. Connect Lean-side contracts to Rust implementation obligations and test harness assertions.

### Phase D: Conformance Coupling
1. Bind formal claims to conformance matrices and empirical finding ids.
2. Enforce profile-scoped status (`draft`, `provisional`, `validated`) with explicit promotion rules.
3. Require replayable evidence bundles for every claim that depends on empirical behavior.
4. Run Excel differential/empirical lanes for unresolved edge cases and feed results back into contract rows.

## 8. Definition of Done
This side-quest is done for a declared profile only when all are true:

1. Coverage:
   - every in-scope function and operator has a function id and contract row.
   - every in-scope function is classified and mapped to implementation/proof/test lanes.
2. Traceability:
   - every contract row maps to evidence ids and requirement ids.
   - every Rust implementation unit maps to its contract/proof/test obligations.
3. Formalization:
   - Lean formal model covers declared profile slice semantics for values/functions.
4. Proofs:
   - required proof obligations for declared profile slice pass.
5. Conformance:
   - required packs and empirical replay lanes pass for the same profile slice.
6. Boundaries:
   - unresolved behavior is explicit, version-scoped, and non-blocking by policy (not hidden).
7. Validation:
   - Excel differential evidence exists for all mandatory edge-case lanes in the declared slice.
8. Localization posture:
   - localization-sensitive lanes are either validated or explicitly deferred with bounded policy and follow-up hooks.
9. Version context:
   - each function claim states applicable Excel version/channel scope and Compatibility Version scope, or is explicitly marked unbounded/provisional.

## 9. Non-goal Guardrails
1. Do not stall on global completeness before proving anything.
2. Do not claim proof of behavior not captured in formal contracts.
3. Do not hide ambiguity by overfitting to one Excel build.

## 10. Relationship to Existing Foundation Artifacts
1. This charter governs the `OxFunc` lane.
2. OxFunc-owned mutable artifacts live under `docs/function-lane/` and are the primary editable function-definition working set.
3. `OxFml` is the sibling F3E formula-language lane (grammar/parse/bind and formula-level semantics outside function/value ownership).
4. FEC/F3E boundaries are defined in model docs; OxFunc consumes those boundaries and contributes function/value capability declarations.
5. Foundation remains the owner of external Excel reference/spec corpus artifacts consumed by OxFunc.
6. OxVBA remains a sibling program; this effort aligns with its clean-room and formal-assurance spirit, while focusing on worksheet function semantics rather than VBA language runtime.

## 11. Execution Doctrine and Aspirational Plan
1. `OPERATIONS.md` is the lane-level execution doctrine for OxFunc and is normative unless it conflicts with this charter or Foundation doctrine.
2. `TUX1000_PLAN.md` is the aspirational execution adjunct for ambitious sequencing and method hardening.
3. `TUX1000_PLAN.md` cannot override this charter; on conflict, this charter and Foundation doctrine win.


# Function Definition Discussion Register

## 1. Purpose
Drive interactive decisions needed to move function-definition rows from `draft/provisional` to stable conformance policy.

## 2. Core Discussion Topics

### D-001: Volatile vs Non-Deterministic
Question:
1. Should volatility be treated strictly as an invalidation/recalc trigger property, with non-determinism as an output-stability property?

Why this matters:
1. Prevents conflating recalc behavior with semantic randomness/time/external variability.
2. Affects `XLS-CF-FN-008`, replay contracts, and reason-code interpretation.

Decision output needed:
1. Final axis definitions and allowed combinations.
2. Minimum required probe set per combination.

### D-002: Host Interaction During Evaluation
Question:
1. What host-state dimensions are first-class in function semantics (`workbook`, `application`, `environment`, `external-provider`)?

Why this matters:
1. Defines reproducibility boundary for empirical findings.
2. Controls capability-gating and platform caveat policy (`XLS-CF-FN-007`, `XLS-CF-VP-003`).

Decision output needed:
1. Canonical host-interaction taxonomy.
2. Required metadata contract in evidence outputs.

### D-003: Invalidation Trigger Classes
Question:
1. Which trigger classes are mandatory for function definitions and conformance tests (`T-DEP`, `T-VOL`, `T-HOST`, `T-EXT`, `T-VERSION`)?

Why this matters:
1. Drives probe design and replay determinism.
2. Reduces ambiguity in counter-signal interpretation.

Decision output needed:
1. Final trigger-class vocabulary.
2. Mapping contract from function rows to trigger-class set.

### D-004: Aggregate Coercion Policy
Question:
1. How should we formalize direct-argument coercion vs range-scan coercion, and which is normative when signals conflict?

Why this matters:
1. Blocks closure of `XLS-CF-TV-008`.
2. Impacts many aggregate functions beyond `SUMIF`.

Decision output needed:
1. Policy matrix template for aggregate families.
2. Version-scoping rule when behavior diverges.

### D-005: Argument-Gap Semantics
Question:
1. What is the compatibility policy for missing-argument forms (for example `=SUM(A1,,B1)`)?

Why this matters:
1. Blocks closure of `XLS-CF-FL-010`.
2. Affects parser/evaluator boundary assumptions.

Decision output needed:
1. Accepted/rejected/mapped classes by function family.
2. Compatibility-mode behavior plan if divergence remains.

### D-006: Dynamic-Array Function Coupling
Question:
1. Which spill-related expectations are function-definition obligations versus non-function formula/format/table obligations?

Why this matters:
1. Needed to close `XLS-CF-FL-005`, `XLS-CF-TB-004`, `XLS-CF-FM-005` coherently.
2. Prevents misclassification of mismatch causes.

Decision output needed:
1. Function-vs-non-function boundary statement.
2. Ownership mapping for each spill-related requirement lane.

### D-007: Argument and Return Conversion Boundaries
Question:
1. Which coercions/adaptations are function-definition obligations vs evaluator/worksheet host obligations?
2. How do we encode pre-call coercion and post-call adaptation in conformance rows?

Why this matters:
1. Central to UDF parity goals.
2. Required to model array-return anchor adaptation without collapsing full spill semantics into this lane.

Decision output needed:
1. Canonical pre-call coercion policy schema.
2. Canonical post-call adaptation schema and spill-boundary statement.

### D-008: Operator-As-Function Coverage
Question:
1. Which operators are modeled as evaluable `OP_*` function rows?
2. Which parser tokens remain syntax-only (`SYN_*`) with no function row?

Why this matters:
1. Unifies operator and function semantics under one conformance framework.
2. Prevents semantic/grammar ambiguity for separator-like tokens.

Decision output needed:
1. Approved `OP_*` inventory (including trim-ref family and `@`).
2. Approved syntax-only inventory and locale token profile.

### D-009: UDF Surface Model (XLL/VBA/Automation/JS)
Question:
1. What minimum semantic contract do we require per UDF surface in this phase?
2. Which open questions must stay explicit (for example VBA mutation constraints, Range return semantics)?

Why this matters:
1. Function-definition scope includes host integration classes.
2. We need comparable axis tags across built-ins and UDF families.

Decision output needed:
1. UDF-surface-specific metadata contract.
2. Prioritized empirical/doc probes per surface.

### D-010: Compatibility-Version-Scoped Function Definitions
Question:
1. How should workbook-level compatibility version toggle function semantics and conformance expectations?

Why this matters:
1. Prevents false regressions when version-scoped behavior is intentional.
2. Adds required evaluation axis to probe matrices.

Decision output needed:
1. Compatibility-version policy schema per function/operator row.
2. Replay matrix contract including compatibility version.

### D-011: Non-Interesting Function UDF Parity Hypothesis
Question:
1. Is every non-interesting function implementable with full fidelity via UDF-style implementation under explicit coercion/reference policies?
2. Which counterexamples exist?

Why this matters:
1. This is a major simplification candidate for implementation and assurance planning.
2. It can drive axis/tag refinement from concrete differential implementation evidence.

Decision output needed:
1. Acceptance/rejection criteria for parity hypothesis.
2. Differential validation campaign plan and required evidence outputs.

### D-012: `INDIRECT` and Context-Dependence vs Non-Determinism
Question:
1. Should `INDIRECT` be classified as deterministic context-dependent, non-deterministic, or mixed by context?

Why this matters:
1. Distinguishes invalidation policy and host-context dependence from true non-determinism.
2. Influences function-class boundary quality.

Decision output needed:
1. Final determinism/host-interaction classification rule for context-sensitive reference functions.
2. Example matrix (`INDIRECT`, `OFFSET`, `XLOOKUP`) with rationale.

### D-013: RTD Topic Lifecycle Semantics
Question:
1. What is the canonical lifecycle model for topic connect/register/update/invalidate/disconnect at cell-rooted evaluation boundary?

Why this matters:
1. External invalidation pathway correctness depends on this.
2. Needed to align docs, empirical probes, and conformance rows.

Decision output needed:
1. Minimal state machine for RTD topic lifecycle.
2. Replayable scenario set and promotion criteria.

### D-014: XLL Registration Signature and Type-System Mapping
Question:
1. What exact mapping from Excel registration type strings (`pxTypeText`) to function-definition class axes do we adopt?
2. How do we model reference-capable argument/return types and memory ownership boundaries in conformance rows?

Why this matters:
1. Required for language-independent `.xll` implementation planning with realistic fidelity targets.
2. Determines whether non-interesting function UDF parity hypothesis is technically achievable per function.

Decision output needed:
1. Canonical registration/type mapping table.
2. Required metadata fields for argument kinds, return adaptation, and callback constraints.

### D-015: FEC Profile Taxonomy and Enforcement
Question:
1. Is the current `fec_dependency_profile` vocabulary sufficient (`none`, `ref_only`, `caller_context`, `time_provider`, `random_provider`, `external_provider`, `locale_profile`, `composite`)?
2. Which capabilities belong in FEC versus other policy layers?

Why this matters:
1. FEC is the declared boundary for host/context dependencies in formula and function semantics.
2. Missing or weak FEC taxonomy makes conformance replay non-portable and weakens clean-room traceability.

Decision output needed:
1. Final FEC profile vocabulary and stability policy.
2. Enforcement contract for "no undeclared FEC facility usage."

### D-016: Function/Profile Mapping and Test Obligations
Question:
1. What is the required mapping from each function/operator row to FEC profile and facility tags?
2. Which differential probe obligations are mandatory per FEC profile class?

Why this matters:
1. Prevents hand-wavy function classifications that cannot be empirically validated.
2. Connects function-definition rows to repeatable probe plans and evidence IDs.

Decision output needed:
1. Row-level field contract (`fec_dependency_profile`, `fec_facility_tags`).
2. Minimal probe matrix per profile class (`ref_only`, `time_provider`, `external_provider`, etc.).

### D-017: Function Admission vs Runtime Error Boundary
Question:
1. For a given function/signature, which invalid call shapes are formula-entry rejections versus accepted formulas that evaluate to runtime errors?
2. For array-lifted inputs, do mixed-type failures collapse to a single scalar error or produce elementwise error-bearing arrays?

Why this matters:
1. This is a core gap for non-interesting functions (`SIN`/`ASIN` canonical lane) and blocks stable parser/evaluator contracts.
2. It determines whether parser, binder, or evaluator owns each class of failure signal.

Decision output needed:
1. Canonical policy schema for admission/coercion/domain/array-lift outcomes per function family.
2. Seed-backed decision table for `SIN()`, `SIN("asd")`, `SIN({1,"asd",3})`, `ASIN(2)` with explicit build/version scoping.

## 3. Decision Log Template
For each discussion item:
1. `decision_id`
2. `topic_id`
3. `decision_text`
4. `applies_to_fdef_ids`
5. `affected_requirement_ids`
6. `evidence_basis`
7. `date_utc`
8. `owner`

## 4. Current Blocking Status
1. Non-function conformance closure is complete to current evidence boundaries.
2. Remaining blocker is function-definition policy finalization using the discussion topics above.

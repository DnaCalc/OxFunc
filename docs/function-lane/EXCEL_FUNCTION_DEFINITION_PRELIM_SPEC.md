# Excel Function Definition Preliminary Spec

## 1. Purpose
Define a preliminary, implementation-facing frame for Excel worksheet function semantics.

This document is intentionally not final:
1. it captures current decisions and unresolved policy choices,
2. it marks which non-function conformance lanes depend on these choices,
3. it prepares structured interactive review.

## 2. Scope
In scope:
1. Function semantic classes (pure, volatile, non-deterministic, host-interactive, external-source).
2. Invalidation/recalc trigger classes and observable consequences.
3. Function evaluation context dependencies (workbook/session/environment).
4. Function-declared Formula Evaluation Context (`FEC`) capability usage.
5. Argument/return coercion and adaptation framing.
6. Value vs extended-value boundary at function call/return interface.
7. UDF surface taxonomy and compatibility posture.
8. Traceability from function-policy rows to `XLS-CF-*` lanes.

Out of scope:
1. Full per-function final semantics table for all 500 functions.
2. Workbook scheduler internals beyond worksheet-observable effects.
3. Full spill layout mechanics (tracked in non-function formula/table lanes).

## 3. Preliminary Function Class System

### 3.1 Class Axes
Each function can carry multiple orthogonal tags:
1. `determinism_class`: `deterministic | pseudo_random | time_dependent | external_event_dependent`.
2. `volatility_class`: `nonvolatile | volatile_full | volatile_contextual | undecided`.
3. `host_interaction_class`: `none | workbook_state | application_state | environment_state | external_provider`.
4. `thread_safety_class`: `safe_pure | host_serialized | not_thread_safe`.
5. `arg_preparation_profile`: `values_only_pre_adapter | refs_visible_in_adapter`.
6. `coercion_lift_profile`: declarative coercion+array-lift adapter profile id.
7. `kernel_signature_class`: `nullary_const | num_to_num | nums_to_num | text_to_text | lookup_match | custom`.
8. `error_policy_class`: `strict_propagate | conditional_mask | branch_selective | custom`.
9. `compat_version_policy`: `stable_across_versions | version_scoped | unknown`.
10. `fec_dependency_profile`: function-adapter-level FEC profile.
11. `surface_fec_dependency_profile`: surface pipeline FEC profile (including pre-adapter preparation).
12. `compile_eval_class`: `const_foldable_when_closed | runtime_ref_dependent | runtime_context_dependent`.

### 3.2 Working Definitions (Preliminary)
1. Volatile:
   - Volatility is invalidation policy, not output determinism.
   - A volatile cell can be scheduled for recalculation without direct dependency input edits.
2. Non-deterministic:
   - Function output can vary between evaluations with same explicit inputs and same workbook state.
   - Non-determinism can arise from time/random/external-source dependencies.
3. Host-interactive:
   - Function semantics depend on host/application/session state not fully represented in cell inputs.
   - Includes platform capability and feature availability boundaries.
4. FEC dependency profile:
   - Declares which host-context facilities are required/allowed by function semantics.
   - See `../../../Foundation/reference/conformance/excel-worksheet-engine/model/EXCEL_FORMULA_EVALUATION_CONTEXT_FEC.md` for capability families and policy framing.
5. Thread safety:
   - `safe_pure`: function evaluation has no shared mutable host state dependence in declared scope.
   - `host_serialized`: function is safe only under host-serialized invocation policy.
   - `not_thread_safe`: function semantics rely on non-thread-safe state and cannot be safely concurrent.
6. Argument preparation profile:
   - `values_only_pre_adapter`: reference dereference/normalization occurs before function adapter.
   - `refs_visible_in_adapter`: function adapter receives reference-bearing arguments and controls dereference behavior.
7. Coercion/lift profile:
   - declarative profile identifier for scalar coercion and array mapping behavior.
8. Kernel signature class:
   - pure function core shape independent from preparation/coercion seam.

9. `volatile_full` vs `volatile_contextual`:
   - retained as unresolved terminology pending interactive policy finalization.
   - current provisional intent:
     - `volatile_full`: always participates in volatile invalidation cycle.
     - `volatile_contextual`: participates only under function/context-specific conditions.

10. Compile-time evaluability class:
   - `const_foldable_when_closed`: expression can be reduced at compile/prepare time when all arguments are constant-closed.
   - `runtime_ref_dependent`: requires runtime value fetch/resolution from references.
   - `runtime_context_dependent`: requires runtime host context (for example caller/time/external context), so deterministic compile-time reduction is not valid.

Illustrative examples (planning):
1. `SIN(4)` and `SIN(2*PI())` can be treated as constant-closed reductions if desired.
2. `SIN(A1)` is runtime reference-dependent.
3. `ROW()` and `NOW()` are runtime context-dependent.

### 3.3 Current High-Risk Class Anchors
1. `NOW`, `TODAY`: volatile + time-dependent.
2. `RAND`, `RANDARRAY`: volatile + pseudo-random.
3. `RTD`: external-event-dependent + external-provider.
4. `INDIRECT`, `OFFSET`: reference-structural functions with high dependency impact.
5. CUBE family (`CUBESET`, `CUBEVALUE`, etc.): external-provider class with deferred depth.

### 3.4 FEC Integration Contract (First Pass)
FEC is now a first-class contract axis for this lane.

Normative planning rule:
1. Every function/operator row shall declare both:
   - function-adapter `fec_dependency_profile`,
   - surface pipeline `surface_fec_dependency_profile`.
2. Where needed, row notes shall include explicit facility tags (for example `cap_reference_resolution`, `cap_time_provider`).
3. A function must not observe undeclared FEC facilities in conformance-positive implementations.

Working profile vocabulary:
1. `none`: no external context dependency.
2. `ref_only`: depends on reference resolution facilities only.
3. `caller_context`: depends on caller position/shape context.
4. `time_provider`: depends on host time/date source.
5. `random_provider`: depends on host pseudo-random source.
6. `external_provider`: depends on external topic/provider lifecycle.
7. `locale_profile`: depends on locale parsing/format profile.
8. `composite`: depends on multiple facility families.

Reference:
1. `../../../Foundation/reference/conformance/excel-worksheet-engine/model/EXCEL_FORMULA_EVALUATION_CONTEXT_FEC.md`.

### 3.4.1 FEC Admission and Failure Classification
1. Function-library execution is permitted only after FEC/F3E admission for the invocation context succeeds.
2. Seam-level admission outcomes (`Applied`/`Rejected*` or equivalent) are boundary-policy signals and must be tracked separately from function semantic result classification.
3. Function semantic conformance claims are evaluated on admitted executions; rejected admissions are only function failures when the function contract explicitly requires admission for that scenario.
4. Empirical manifests for adversarial seam tests must encode expected seam status so intentional rejections are treated as expected behavior.

### 3.5 Layered Function Pipeline Contract
Normative decomposition:
1. Kernel:
   - pure semantic core (`num_to_num`, etc.).
2. Coercion/lift adapter:
   - declarative conversion and array-map behavior.
3. Argument preparation adapter:
   - dereference/normalization policy according to `arg_preparation_profile`.

Design rule:
1. Non-interesting scalar families should default to:
   - `arg_preparation_profile=values_only_pre_adapter`,
   - kernel-focused proofs and tests.
2. Reference-sensitive/aggregate families may require:
   - `arg_preparation_profile=refs_visible_in_adapter`,
   - provenance-aware semantics (for example direct-arg vs range-scan behavior in `SUM`-like families).

## 4. Invalidation and Recalc Trigger Model (Preliminary)
Trigger classes:
1. `T-DEP`: dependency graph input changed.
2. `T-VOL`: volatility tick (recalc cycle trigger without direct precedent edit).
3. `T-HOST`: host/application state changed (mode/session/calc-state axes).
4. `T-EXT`: external provider/event update.
5. `T-VERSION`: build/channel/platform behavior drift.

Preliminary rule:
1. Function definition rows must declare expected trigger classes.
2. Conformance probes must isolate trigger class in scenario design where feasible.
3. Workbook compatibility version is part of trigger context when version-scoped behavior applies.

### 4.1 Volatility mechanics (provisional)
1. Working model: volatility leaves a recalculation eligibility marker on the calling cell.
2. Marker semantics are not equivalent to dirty-edit semantics; volatility can still place cell in future recalc candidate set.
3. UDF-triggered volatility controls (`xlfVolatile` / `Application.Volatile`) are treated as policy hooks that can modify marker behavior.
4. Exact mechanics remain an explicit policy topic and empirical target.

### 4.2 RTD lifecycle mechanics (provisional)
1. First RTD evaluation for a topic establishes a topic connection and topic->cell association at worksheet boundary.
2. External topic updates trigger targeted invalidation for associated cells.
3. Recalculation can either refresh topic value (if topic remains referenced) or disconnect lifecycle path (if no longer referenced).
4. This lifecycle is modeled as external invalidation semantics, distinct from volatile invalidation.

## 5. Argument and Return Conversion Boundary
### 5.1 Pre-call argument coercion
1. Arguments can be coerced before function invocation according to function signature and evaluator policy.
2. Coercion source is host/evaluator policy, not function implementation internals.
3. Reference-like arguments may be normalized/dereferenced in either:
   - pre-adapter preparation (`values_only_pre_adapter`), or
   - function adapter (`refs_visible_in_adapter`).

### 5.2 Post-call return adaptation
1. Function return values can be adapted by host after function execution.
2. Array returns can be adapted into dynamic-array anchor representation at the calling cell.
3. Spill-cell virtual value projection is tracked as related but primarily non-function-lane behavior.

### 5.3 Value vs extended value
1. `value`: primary scalar/reference/array semantic payload.
2. `extended_value`: value plus host metadata/structure used at worksheet boundary.
3. Candidate extended-value families to refine:
   - formatting-hint enriched value,
   - error with detail payload (`source`, `description`, etc.),
   - virtual value relative to anchor.

### 5.4 Formula admission vs runtime error boundary
1. Function contracts need two separate outcome surfaces:
   - formula-admission surface (parser accepts or rejects formula entry),
   - runtime-evaluation surface (accepted formula returns value/error/array result).
2. Required-argument omission can belong to admission policy for specific shapes (for example canonical seed `SIN()`).
3. Accepted formula calls can still produce runtime coercion/domain errors (for example canonical seeds `SIN("asd")`, `ASIN(2)`).
4. Array-lift behavior must be explicit:
   - mixed-type array inputs (for example `SIN({1,"asd",3})`) need a declared policy for scalar fail-fast vs elementwise result with internal errors.
5. Current public function references are too thin to close this lane alone; empirical evidence remains mandatory for final policy.

## 6. Operator Functions and Syntax Delimiters
1. Evaluable operators are represented as pseudo-functions (`OP_*`) in this lane.
2. Parse-only delimiters are not function rows.
3. Current split:
   - semantic/evaluable example: `OP_UNION_REF`, `OP_IMPLICIT_INTERSECTION`, `OP_SPILL_REF`.
   - parse-only example: `SYN_ARG_SEPARATOR` with locale token profile.
4. Trim references are modeled as one operator family:
   - `OP_TRIM_REF(mode=leading|trailing|both)`.

## 7. UDF Surface Taxonomy (Preliminary)
1. XLL UDFs:
   - registration/lifetime/signature model (SDK + `xlfRegister` + caller context).
   - includes volatility and execution-context flags.
   - working SDK digest for this lane:
     `../../../Foundation/reference/conformance/excel-worksheet-engine/functions/XLL_SDK_REGISTRATION_AND_TYPES_REFERENCE.md`.
2. VBA UDFs:
   - scope rules differ workbook module vs add-in context.
   - COM object interaction and mutation restrictions remain explicit open questions.
3. Automation Add-in UDFs:
   - COM registration and invocation model.
   - lower detail priority for current pass.
4. JavaScript custom functions:
   - async/external and custom data-type implications.
   - extended value returns and custom entity payloads in scope for compatibility classification.

## 8. Compatibility-Version Semantics
1. Function definitions may be workbook-compatibility-version scoped.
2. Conformance matrix must include compatibility-version axis in addition to build/channel/platform.
3. Version divergence is modeled explicitly; not treated as automatic regression.

## 9. Implicit Intersection (`@`) as Operator Function
1. Canonical semantic id: `OP_IMPLICIT_INTERSECTION`.
2. Legacy/interop alias context:
   - historical preview representation included `SINGLE(...)`,
   - compatibility serialization may include `_xlfn.SINGLE(...)` in pre-DA contexts.
3. Alias forms are compatibility representations, not separate modern semantic operators.
4. Behavioral summary (source-backed, provisional wording):
   - `@` enforces single-value extraction behavior where formulas would otherwise return arrays/ranges in dynamic-array Excel.
   - when opening legacy formulas, Excel can insert `@` to preserve historical implicit-intersection behavior.
   - behavior remains context-dependent on argument/reference shape and surrounding formula context.

## 10. Coupling Into Non-Function Lanes
Function-definition decisions directly affect:
1. `XLS-CF-TV-008` aggregate coercion policy boundary.
2. `XLS-CF-FL-010` argument-gap rationale and parser/evaluator policy.
3. `XLS-CF-FL-005`, `XLS-CF-TB-004`, `XLS-CF-FM-005` where dynamic-array function semantics influence spill expectations.
4. `XLS-CF-FL-006` external-reference behavior interpretation in host/open-state contexts.
5. `XLS-CF-FL-012` function-call admission vs runtime error boundary and array-lift error propagation policy.

## 11. Evidence Model for This Lane
Evidence classes:
1. `spec_anchor`: public formal/help references (`ECS-*`, `REFX-*`).
2. `empirical_anchor`: promoted empirical findings (`EMP-*`).
3. `policy_decision`: explicit interactive decision logs (to be introduced in this lane).

Promotion principle:
1. Function-policy rows remain `draft` or `provisional` until supported by spec and/or empirical anchors with explicit policy decisions.

## 12. Immediate Next Step
1. Use `EXCEL_FUNCTION_DEFINITION_DISCUSSION.md` to resolve open policy decisions.
2. Update `EXCEL_FUNCTION_DEFINITION_PRELIM_CONFORMANCE.csv` with confirmed decisions and FEC profile tags.
3. Run language-independent prompt pack for non-interesting-function `.xll` implementation planning and differential validation, including FEC dependency declarations in contract rows.

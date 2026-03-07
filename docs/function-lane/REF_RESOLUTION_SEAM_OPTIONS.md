# Ref Resolution Seam Options (W4)

Status: `active`
Workset: `W4`
Decision: `selected_baseline=capability_record_model`

## 1. Purpose
Document W4 alternatives for the `Ref -> EvalValue` seam and record selected baseline with rationale.

## 2. Option A - Capability-Record Model
Shape:
1. resolver interface declares explicit capability record,
2. function/evaluator calls resolver with normalized reference token,
3. resolver returns explicit value or explicit resolution error.

Strengths:
1. direct mapping to `fec_dependency_profile` and facility tags,
2. strong runtime traceability for denied/unsupported paths,
3. stable adapter contract for Rust implementation and differential probes.

Tradeoffs:
1. policy surface requires explicit capability bookkeeping.

## 3. Option B - Abstract Oracle-Function Model
Shape:
1. seam represented as pure abstract resolution function.

Strengths:
1. compact formal reasoning surface.

Tradeoffs:
1. weaker operational traceability for capability-denied paths,
2. less direct enforcement mapping to function-row FEC declarations.

## 4. Option C - Effect/Monadic Model
Shape:
1. seam modeled as effectful computation with explicit context/state.

Strengths:
1. maximal expressiveness for future async/provider-lifecycle paths.

Tradeoffs:
1. heavier complexity for kickoff scope,
2. slows closure for W4/W5/W6 baseline gates.

## 5. Selected Baseline
Selected:
1. Option A (`capability_record_model`).

Selection rationale:
1. best balance of proof composability and runtime traceability,
2. strongest alignment with FEC declaration enforcement,
3. lowest integration risk for near-term W5/W6 consumers.

## 6. Baseline Contract Requirements
1. resolver capabilities must be explicit and machine-readable.
2. denied capability and unresolved reference outcomes must be explicit typed errors.
3. normalization of reference tokens occurs before resolution.
4. external/open-state reference allowance is explicit capability state, not implicit behavior.

## 7. Deferred Notes
1. effect/monadic formulation remains a candidate for later external-provider-heavy slices.
2. oracle-function abstraction remains useful for theorem simplification views but is not the runtime-facing baseline contract.

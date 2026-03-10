# FEC/F3E Interface Review Prep (Function Model)

Status: `active`
Owner lane: `OxFunc`

## 1. Purpose
Prepare a compact working plan to:
1. refine the layered function model for large-sweep rollout,
2. review an incoming FEC/F3E interface design against that model.

## 2. Current Function Model Snapshot
Current intended decomposition:
1. kernel layer:
   - pure semantic core (`num_to_num`, `nums_to_num`, etc.).
2. coercion/lift adapter layer:
   - declarative conversion + array-map/error policy.
3. arg-preparation/deref layer:
   - declarative reference/normalization policy.

Current key fields:
1. `arg_preparation_profile` (`values_only_pre_adapter | refs_visible_in_adapter`)
2. `coercion_lift_profile`
3. `kernel_signature_class`
4. `fec_dependency_profile` (adapter level)
5. `surface_fec_dependency_profile` (pipeline level)

## 3. Refinement Plan (Small, Concrete)
1. Profile catalog:
   - introduce a machine-readable catalog of reusable adapter profiles.
   - target shape: one row per profile with coercion/lift/error semantics.
2. Provenance model for `refs_visible_in_adapter`:
   - define explicit origin tags (`direct_arg`, `range_scan`, `ref_scalar`, `spill_ref`).
   - require these tags for aggregate/reference-sensitive families.
3. Generated profile tests:
   - one reusable harness that runs profile obligations over function assignments.
4. Lean profile theorems:
   - reusable proofs per profile class; instantiate by function binding.
5. Policy lint:
   - enforce that `values_only_pre_adapter` functions do not consume resolver APIs directly.

## 4. Interface Review Checklist (for new FEC/F3E design)
When reviewing the new interface, answer explicitly:
1. Does the interface separate preparation from adapter execution cleanly?
2. Can FEC provide both modes:
   - values-only prepared args,
   - refs-visible args?
3. Is argument provenance representable and stable across boundaries?
4. Can dual FEC declarations be represented:
   - adapter-level dependency,
   - surface-level dependency?
5. Are capability contracts explicit for deref/locale/feature gates?
6. Can array/spill/reference forms be represented without ambiguity?
7. Is replay evidence capture possible with version + compatibility scope?
8. Does the shape preserve formalization seams for Lean and executable seams for Rust?
9. Does the interface guarantee that function library execution occurs only after explicit session/capability admission?
10. Can seam-level rejection statuses be recorded independently from function semantic outcomes?

## 5. Expected Outputs from Review
1. compatibility verdict:
   - `compatible`, `compatible_with_adaptations`, or `conflicting`.
2. impact matrix:
   - fields to add/change/remove in function contracts.
3. migration notes:
   - ABS impact,
   - SUM-family impact,
   - broader non-interesting-function rollout impact.
4. decision log entries for any doctrinally significant changes.
5. seam-failure accounting note:
   - how intentional/adversarial rejection paths are separated from function conformance failure counts.

## 6. Compact Context for Next Prompt
1. ABS is currently used as the reference implementation for layered decomposition.
2. W5 is `function-phase-complete`; W6 remains `planned`.
3. New thread-safety and layered adapter fields are present in current local model/docs.
4. Admission invariant is explicit: function-library execution runs only after seam admission success.
5. Boundary failure accounting is explicit: seam rejections are tracked separately from function semantic failures.
6. Next action is a targeted design review of incoming FEC/F3E interface proposals plus continued function-lane rollout.

# Coercion and Conversion Preliminary Spec (W4)

Status: `active`
Workset: `W4`
Conformance anchor: `FDEF-029`

## 1. Purpose
Define typed coercion primitives and function-boundary conversion policy used by OxFunc function contracts.

This spec formalizes:
1. pre-call coercion behavior,
2. call-boundary distinctions (`missing_arg`, `empty_cell`, reference-like),
3. array-lift coercion posture for scalar lift kernels,
4. aggregate direct-arg vs range-scan policy matrix surfaces.

## 2. Primitive Set (Baseline)
Required primitive operations:
1. `to_number`
2. `to_text`
3. `to_logical`
4. `propagate_error`
5. `ref_to_eval_value` (delegated seam primitive)
6. `array_lift_map` (for scalar lift kernels)

Typed source domains:
1. `EvalValue`
2. `CallArgValue`

Typed output domains:
1. scalar target (`number`, `text`, `logical`)
2. lifted array outputs (where function family permits lift)
3. explicit coercion error outcomes.

## 3. Boundary Rules (Baseline)
1. `missing_arg` is call-boundary only and never an eval-result tag.
2. `empty_cell` is distinct from `missing_arg`.
3. `ref_to_eval_value` is explicit and out-of-model for function kernels; function kernels consume already-resolved evaluable values unless a function explicitly declares reference-aware behavior.
4. coercion never silently discards worksheet errors; error values propagate unless function family declares mask/branch behavior.

## 4. Admission vs Runtime Rule
1. Worksheet cell-entry and workbook-open ingress are the normative admission surfaces.
2. Evaluate-family/automation entrypoints are tracked as contextual API behavior and do not redefine normative worksheet admission.
3. Admitted calls may still fail at runtime due to coercion/domain outcomes.

## 5. Array-Lift Baseline
Baseline policy:
1. scalar numeric lift families (`SIN`/`ASIN`/`ABS` style) use elementwise lift semantics when arrays are admitted.
2. mixed-type element failures are represented as element-level error-bearing outcomes in the lifted surface.
3. policy status is `provisional`; contradiction-trigger cases force explicit revision.

Out of scope for this rule:
1. aggregate families (`SUM` and peers),
2. non-lifting functions by contract.

## 6. Aggregate Conflict Posture
Global precedence between:
1. direct-argument coercion, and
2. range-scan coercion
remains intentionally unresolved at global level.

Rule:
1. use per-family explicit matrix rows (`COERCION_DECISION_TABLE.csv`),
2. no implicit fallback precedence.

## 7. Ref->Val Seam Contract Hook
1. `ref_to_eval_value` calls through the W4 selected seam contract.
2. the selected seam must enforce declared FEC capability usage.
3. unresolved references and capability-denied references are explicit outcomes.

## 8. Versioning and Revision Triggers
Version axes:
1. Excel app version/channel.
2. workbook Compatibility Version.

Revision triggers:
1. contradiction of array-lift elementwise assumption in declared scalar lift family scope,
2. contradiction of aggregate family matrix row behavior,
3. contradiction of admission/runtime boundary classification on worksheet surfaces.

## 9. Evidence and Empirical Linkage
Baseline scenario source:
1. `COERCION_SCENARIO_MANIFEST_SEED.csv`

Runtime and output contracts:
1. `COERCION_PROBE_RUNTIME_REQUIREMENTS.md`
2. `tools/coercion-probe/results/COERCION_RESULTS_TEMPLATE.csv`

Execution tracking:
1. `COERCION_EXECUTION_RECORD.md`

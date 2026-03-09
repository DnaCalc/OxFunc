# WORKSET - TUX1000 XLL Add-in Bridge (W9)

## 1. Purpose
Build a Rust-based `OxFunc64.xll` add-in that exposes OxFunc function-library behavior through Excel `.xll` exports without coupling core function semantics to XLL transport/runtime concerns.

This workset creates an adapter lane for:
1. XLL entrypoint and registration plumbing,
2. caller/shim mapping from XLL argument forms into OxFunc call surface,
3. comparative validation workbooks that run Excel-native functions and OxFunc-exported functions side by side.

## 2. Position and Dependencies
Program position:
1. post-kickoff extension workset (`W9`), following W1..W7 and W8.1.

Dependencies:
1. consumes W4 coercion and reference-preparation seam model.
2. consumes W5 layered adapter/kernel model (`ABS` as seed).
3. consumes current function metadata fields (`arg_preparation_profile`, `coercion_lift_profile`, `kernel_signature_class`, dual FEC profiles).

## 3. Scope
In scope:
1. Rust `OxFunc64.xll` artifact scaffold and build lane.
2. explicit XLL caller shim layer that maps XLL inputs to OxFunc call arguments.
3. profile-derived export generation across the function catalog from core `FunctionMeta` metadata.
4. registration experiments for signature classes:
   - general/reference-admitting registration (U-style) for provenance/deref path testing,
   - value-only registration (Q-style) where profile-admitted.
5. workbook-based differential validation packs comparing native formulas vs OxFunc add-in formulas.

Out of scope:
1. changing core function kernels to fit XLL transport details.
2. claiming production-ready add-in hardening/security lifecycle.

## 4. Architectural Rule
Separation is mandatory:
1. OxFunc core implementation remains reusable outside XLL.
2. XLL layer is an adapter around OxFunc, not a semantics owner.
3. registration/signature policy decisions must be declarative and traceable per exported function.

## 5. Deliverables
1. workset execution notes and status updates.
2. XLL bridge crate/module scaffold and build instructions for `OxFunc64.xll`.
3. caller shim contract doc:
   - XLL argument kind -> OxFunc argument mapping,
   - error/empty/missing/reference handling mapping.
4. generated export set from profile-derived rules (U/Q variants emitted mechanically).
5. registration manifest notes for U-style vs Q-style export decisions.
6. workbook validation pack:
   - paired native vs OxFunc formulas,
   - expected equivalence/known-divergence annotations,
   - replay instructions.
7. correlation/evidence link updates for exported seed functions.

## 6. Gate Model
### G1 - Bridge Scaffold Closure
Pass when:
1. `OxFunc64.xll` builds reproducibly from repository instructions.

### G2 - Shim Contract Closure
Pass when:
1. XLL-to-OxFunc argument mapping contract is documented and implemented for seed exports.

### G3 - Registration Closure
Pass when:
1. generated exports are registered in Excel and callable,
2. U-style vs Q-style registration posture is explicitly recorded by profile-derived export rows.

### G4 - Differential Workbook Closure
Pass when:
1. workbook/test-set pack runs native vs OxFunc side-by-side checks,
2. mismatches are classified (spec gap / shim defect / environment variability / expected policy divergence).

### G5 - Separation Closure
Pass when:
1. adapter-boundary checks confirm core OxFunc modules do not take XLL-specific dependencies.

## 7. Status
Execution state:
1. `complete`.

Claim confidence:
1. `provisional` (profile-derived export set built/registered; validation coverage remains seed-bounded).

Assurance maturity:
1. `exercised`.

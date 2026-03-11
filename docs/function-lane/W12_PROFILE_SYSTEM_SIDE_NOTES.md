# W12 Profile/System Side Notes

Status: `active`
Workset: `W12`

Purpose:
1. capture profile/classification/coercion/deref observations that arise during W12 execution,
2. preserve candidate profile-system refinements without blocking W12 closure.

## Notes Log

1. Aggregate argument-structure pressure remains open:
   - `AVERAGE`, `COUNT`, and `COUNTA` can close direct-argument seed behavior under `values_only_pre_adapter`,
   - but full direct-scalar versus array-like policy still needs more empirical closure, and any finer source-class split should only be introduced if evidence proves it matters.
2. Count-family split is now clearer:
   - `COUNT` and `COUNTA` can share a packet without sharing the same semantic kernel,
   - so aggregate families need explicit count-policy tags rather than being folded into one generic aggregate profile.
3. Reference-sensitive control lanes confirm `refs_visible_in_adapter` is necessary:
   - `IFERROR` needs lazy fallback preparation,
   - `OFFSET` and `CELL` need preserved reference identity.
4. Numeric helper candidates are becoming visible:
   - `ROUND`, `AND`, and `DATE` each expose small reusable coercion/kernel patterns,
   - but they are still distinct enough to keep under `custom` in the current seed.
5. Prepared-text helper reuse is now justified:
   - `TEXTJOIN`, `CLEAN`, and `EXACT` all consume the same bounded scalar-to-text path,
   - which is now centralized in `functions::adapters`.
6. Provider seam standardization is stronger after W12:
   - `TODAY` and `RAND` reuse the same provider-seam posture as `NOW`,
   - and they produce concrete follow-back candidates for W11 volatile mapping.
7. Bounded A1 utility support is now shared infrastructure:
   - `OFFSET` and `CELL` both needed stable A1 parse/format helpers,
   - so a small function-local reference-text utility layer is now justified before broader resolver work.
8. Dynamic-array shape-only modeling remains useful but bounded:
   - `HSTACK` can close admission and shape classification under the current `ArrayShape` model,
   - but payload fill/padding remains explicit target-partial follow-up.

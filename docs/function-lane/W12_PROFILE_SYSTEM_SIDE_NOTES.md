# W12 Profile/System Side Notes

Status: `active`
Workset: `W12`

Purpose:
1. capture profile/classification/coercion/deref observations that arise during W12 execution,
2. preserve candidate profile-system refinements without blocking W12 closure.

## Notes Log

1. Aggregate argument-structure pressure remains open:
   - `AVERAGE`, `COUNT`, and `COUNTA` are now pinned strongly enough to close on the simpler `direct_scalar` versus `array_like` distinction,
   - and the current baseline does not justify any richer source-class split for these aggregate kernels.
2. Count-family split is now clearer:
   - `COUNT` and `COUNTA` can share a packet without sharing the same semantic kernel,
   - so aggregate families need explicit count-policy tags rather than being folded into one generic aggregate profile.
3. Reference-sensitive control lanes confirm `refs_visible_in_adapter` is necessary:
   - `IFERROR` needs lazy fallback preparation,
   - `OFFSET` and `CELL` need preserved reference identity.
4. Numeric helper candidates are becoming visible:
   - `ROUND`, `AND`, and `DATE` each expose small reusable coercion/kernel patterns,
   - and the current W12 pass was enough to close those slices without needing a broader shared numeric helper abstraction.
5. Prepared-text helper reuse is now justified:
   - `TEXTJOIN`, `CLEAN`, and `EXACT` all consume the same bounded scalar-to-text path,
   - which is now centralized in `functions::adapters`.
   - `TEXTJOIN` is now closed for the current phase once row-major flattening, delimiter textification, and the observed `32768 -> #CALC!` overflow lane were pinned empirically.
   - `EXACT` is also now closed for the current phase with blank-as-empty, scalar-to-text coercion, and code-unit-sensitive Unicode comparison pinned directly.
   - `CLEAN` is now closed for the current phase with the observed extra C1 removal subset (`129`, `141`, `143`, `144`, `157`) recorded explicitly rather than assumed from public-doc summaries.
6. Provider seam standardization is stronger after W12:
   - `TODAY` and `RAND` reuse the same provider-seam posture as `NOW`,
   - and they produce concrete follow-back candidates for W11 volatile mapping.
7. Bounded A1 utility support is now shared infrastructure:
   - `OFFSET` and `CELL` both needed stable A1 parse/format helpers,
   - so a small function-local reference-text utility layer is now justified before broader resolver work.
   - `OFFSET` is now closed for the admitted A1/sheet-prefixed slice, while `CELL` remains the deferred info/macro seam.
8. Dynamic-array payload modeling turned out essential:
   - `HSTACK` turned out not to be merely shape-sensitive in practice,
   - and the current runtime/formal closure now treats it as a payload-materializing function with explicit `#N/A` padding.

# WORKSET - Implicit Intersection Operator (W14)

## 1. Purpose
Run a focused cross-lane investigation for the Excel implicit-intersection operator `@`.

Primary intent:
1. turn the existing provisional `@` placeholder into a concrete OxFunc work packet,
2. determine the minimum OxFunc runtime and Lean shape needed to model `@` honestly,
3. determine which distinctions must survive OxFml/FEC preparation so `@` does not collapse into a lossy "top-left array pick" shortcut,
4. seed the empirical matrix needed to characterize modern dynamic-array behavior, legacy-compatibility normalization, and `_xlfn.SINGLE(...)` serialization seams.

## 2. Position and Dependencies
Program position:
1. post-W13 packet (`W14`).

Dependencies:
1. W4 coercion and ref-resolution seam work.
2. W10/W12/W13 function-packet patterns for mixed reference, caller-context, and dynamic-array seams.
3. `docs/function-lane/EXCEL_FUNCTION_DEFINITION_PRELIM_SPEC.md` section 9 and `FDEF-018`.
4. `docs/function-lane/FUNCTION_ADAPTER_LAYERING_PRELIM_SPEC.md`.
5. `docs/function-lane/FEC_F3E_INTERFACE_REVIEW_PREP.md`.
6. `../OxFml/docs/spec/formula-language/EXCEL_FORMULA_LANGUAGE_CONCRETE_RULES.md` rule `FML-R-003`.
7. `../OxFml/docs/spec/fec-f3e/FEC_F3E_REDESIGN_SPEC.md`.

Reviewed inbound observations:
1. `../OxFml/docs/upstream/NOTES_FOR_OXFUNC.md` reviewed on `2026-03-14`.
2. Current state: sibling `docs/upstream/` directory exists in `OxFml`, but `NOTES_FOR_OXFUNC.md` is absent.

Backlog ownership note:
1. `W014` remains the provenance/evidence owner for `@`.
2. Active current-version backlog tracking now sits in `W051`.

## 3. Scope
In scope for W14:
1. canonical semantic identity for the operator as `OP_IMPLICIT_INTERSECTION` with surface token `@`.
2. operator behavior over:
   - scalar values,
   - range/reference-like inputs,
   - array payloads,
   - spill-anchor and reference-returning expression inputs where admitted.
3. caller-relative selection versus array top-left selection as an explicit semantic distinction.
4. compatibility/serialization seams:
   - legacy implicit-intersection preservation,
   - explicit `@` in modern formulas,
   - `_xlfn.SINGLE(...)` compatibility representation on the current reference baseline.
5. OxFml/FEC prepared-call requirements:
   - provenance retention,
   - caller anchor/context,
   - spill/reference identity,
   - feature/compatibility gating.
6. Lean executable-model shape and alignment plan.
7. seed empirical matrix for Excel replay.

Out of current W14 closure target:
1. alternate locale sweeps.
2. cross-channel/build-wide replay beyond the current reference baseline.
3. full pre-dynamic-array compatibility-version sweep as an orthogonal version/interop phase rather than a blocker to current-baseline OxFunc support.
4. full structured-reference/table `[@Column]` closure as a separate syntax-context packet.
5. final OxFml acknowledgment or integration of seam changes.

## 4. Deliverables
1. W14 workset spec.
2. OxFunc-local investigation note for `@` covering:
   - semantic classification,
   - runtime shape,
   - OxFml/FEC requirements,
   - Lean characterization,
   - test plan.
3. dedicated OxFunc slice/contract note:
   - `docs/function-lane/FUNCTION_SLICE_OP_IMPLICIT_INTERSECTION_CONTRACT_PRELIM.md`
4. seed empirical scenario manifest for W14.
5. OxFml handoff packet and register row for evaluator/FEC seam impacts.
6. in-progress register update reflecting the new work packet.

## 5. Gate Model
### G1 - Semantic Baseline
Pass when:
1. the existing repo evidence and official-support summary for `@` are consolidated into one OxFunc-local investigation note,
2. open semantic lanes are explicit rather than implied.

### G2 - Boundary Contract Baseline
Pass when:
1. OxFunc has a concrete statement of what OxFml/FEC must preserve for `@`,
2. a handoff packet is filed for evaluator-facing impacts.

### G3 - Runtime/Formal Design Baseline
Pass when:
1. OxFunc has a candidate runtime seam shape and Lean module shape for `@`,
2. the proposed design is tied to existing OxFunc adapter/resolver patterns rather than treated as a blank-sheet redesign.

### G4 - Replay Seed Baseline
Pass when:
1. a deterministic replay seed matrix exists for the first empirical `@` lanes,
2. compatibility and normalization lanes are represented explicitly.

## 6. Current Status
Execution state:
1. `complete`

Claim confidence:
1. `provisional`

Assurance maturity:
1. `exercised`

Completeness axes:
1. `scope_completeness`: `scope_complete`
2. `target_completeness`: `target_complete`
3. `integration_completeness`: `integrated`

Open lanes:
1. none in declared W14 current-phase scope.

Reference-related prework inventory:
1. `INDEX`, `INDIRECT`, `OFFSET`, and `XLOOKUP` reference-return lanes are already `function-phase-complete` for the current reference baseline from W10/W12.
2. `OP_SPILL_REF` (`#`) is now explicit in OxFunc docs/runtime/formal artifacts, but current seam doctrine does not require spill provenance to cross from OxFml into OxFunc.
3. current working position is that `#` should be discharged on the OxFml side into the current resolved spill region or error before OxFunc evaluation, unless later evidence proves a true OxFunc-owned spill-sensitive semantic lane.
4. current W14 implementation now targets `@` scalarization over resolved operand classes rather than preserving spill-link provenance by default.

Legacy CSE interaction baseline:
1. ordinary formula mode and legacy Ctrl+Shift+Enter array mode are distinct evaluation contexts.
2. implicit intersection is the ordinary-formula scalarization story; legacy CSE formulas instead opt into array calculation throughout.
3. dynamic-array Excel makes old implicit intersection visible as `@`, while legacy-array compatibility may rewrite or preserve formulas differently on roundtrip.
4. mixed formulas combining explicit `@` and array-calculation behavior are a dedicated characterization lane rather than a safe simplifying assumption.
5. future `@` / operator refresh passes should be checked against Microsoft's operator reference page:
   `https://support.microsoft.com/en-us/office/calculation-operators-and-precedence-in-excel-48be406d-4975-4d31-b2b8-7af9e0e2878a`

Current OxFunc-side completion reading:
1. the admitted current-baseline OxFunc runtime slice is real:
   - Rust implementation exists,
   - Lean binding exists,
   - native Excel replay exists,
   - the current Excel host baseline now pins `_xlfn.SINGLE(...)` normalization onto the same modern `@` surface,
   - and the OxFml adapter and semantic-plan lanes exercise both explicit `@` and legacy-single compatibility semantics end-to-end.
2. `W14` is therefore complete for declared current-phase OxFunc scope.
3. broader pre-dynamic-array compatibility-version sweeps and structured-reference context remain future validation/interop lanes rather than blockers to the current-baseline OxFunc support claim.

## 7. Future Questions (Non-Blocking)
These no longer block declared current-phase support for `W014`, but they remain useful follow-on questions for future validation or architecture refinement:
1. Does `@` need a new primary semantic substrate in OxFunc rather than being forced into an existing reference family?
2. Should scalarization happen inside OxFunc as an operator-function, or earlier in OxFml/FEC with explicit trace/provenance?
3. What is the minimum prepared-argument vocabulary that preserves the distinction between:
   - range/reference input,
   - array payload input,
   - spilled-range input?
4. What worksheet-visible outcome should OxFunc expect when caller-relative selection fails to pick a unique item?
5. Which reference-seam operators must already exist before `@` can be implemented honestly?
   - current answer: `OP_SPILL_REF` must exist somewhere in the overall architecture, but current seam doctrine leaves it primarily on the OxFml side unless future evidence proves OxFunc needs spill-sensitive provenance.
6. Does legacy Control+Shift+Enter (`{=...}`) array-formula context require any OxFunc-visible semantic mode, or is it entirely an OxFml/admission/publication concern?
   - current note: keep this explicitly open; it may interact with `@` migration/scalarization behavior and with whether some array-era formulas need a distinct prepared-call or publication mode even when the function kernel is unchanged.
7. Can the OxFml -> OxFunc seam expose `@` as a function-evaluation mode rather than requiring OxFunc to inspect general parse structure?
   - current answer: possibly for bound function-call operands, but not as the only model for all `@` operands.

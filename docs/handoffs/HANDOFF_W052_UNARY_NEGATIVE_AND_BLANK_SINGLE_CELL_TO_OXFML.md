# Handoff - W52 Unary Negative Literal And Blank Single-Cell Seam Corrections To OxFml

Status: `acknowledged`
Source lane: `OxFunc`
Source workset: `W052`
Target lane: `OxFml`
Target workset: `W049` / `W050` follow-up

## 1. Scope And Profile Bounds
Affected domains:
1. formula-language parse/bind/evaluation handling of unary negative literals inside function arguments
2. stand-in host/reference resolution behavior for absent single-cell worksheet references
3. OxFml <-> OxFunc seam validation through the existing adapter corpus

Current scope bound:
1. current reference Excel baseline only
2. bounded seam-correction packet only
3. no request here to widen OxFunc function metadata or change OxFunc function kernels

## 2. Core Message
Two remaining OxFunc-side seam-corpus failures now read as OxFml-side boundary issues rather than OxFunc function-definition issues:

1. unary negative literals are not surviving the current OxFml path correctly for at least:
   - `SIGN(-5)`
   - `PV(0.05,10,-100)`
   - `FV(0.05,10,-100)`
2. absent single-cell references in the stand-in host are being treated as unresolved rather than blank, surfaced by:
   - `ISBLANK(A1)` with blank `A1`

These are requests for OxFml seam correction, not for OxFunc semantic widening.

## 3. Evidence

### 3.1 Unary Negative Literal Lanes
Direct Excel readback on `2026-03-26`:
1. `=SIGN(-5)` -> `-1`
2. `=PV(0.05,10,-100)` -> `772.1734929184813`
3. `=FV(0.05,10,-100)` -> `1257.789253554883`

Current OxFunc-side adapter result after fixture cleanup:
1. `FN-SIGN-01` fails before value comparison with `OxFunc surface evaluation failed for SIGN: Value`
2. `FN-PV-01` fails before value comparison with `OxFunc surface evaluation failed for PV: Value`
3. `FN-FV-01` fails before value comparison with `OxFunc surface evaluation failed for FV: Value`

Current OxFunc reading:
1. the shared pattern is the negative literal, not the function family,
2. OxFunc `SIGN`, `PV`, and `FV` kernels are not the first suspect,
3. OxFml parser/evaluator handling of prefix unary minus is the primary seam candidate.

### 3.2 Blank Single-Cell Stand-In Lane
Direct Excel readback on `2026-03-26`:
1. `=ISBLANK(A1)` on a blank `A1` -> `TRUE`

Current OxFunc-side adapter result after fixture cleanup:
1. `FN-ISBLANK-01` fails with `reference resolution failed: UnresolvedReference { target: "A1" }`

Current OxFunc reading:
1. OxFml's local resolver currently returns `UnresolvedReference` when a single-cell target is absent from the fixture map,
2. the same stand-in machinery already treats absent cells inside area-reference expansion as `EmptyCell`,
3. the single-cell path should align with blank-cell semantics rather than unresolved-reference failure for this lane.

## 4. Candidate Target Areas
Candidate OxFml target areas:
1. `crates/oxfml_core/src/syntax/parser.rs`
2. `crates/oxfml_core/src/eval/mod.rs`
3. `crates/oxfml_core/src/oxfunc_adapter/mod.rs`
4. `crates/oxfml_core/tests/fixtures/w050_oxfunc_pinned_fixture_corpus.json`
5. `docs/spec/formula-language/*`
6. `docs/spec/OXFML_FIXTURE_HOST_AND_COORDINATOR_STANDIN_PACKET.md`

## 5. Requested OxFml Decisions
1. Does OxFml agree unary `-` should survive parse/bind/evaluation as a first-class operand transformation rather than collapse into an adapter-time `#VALUE!` failure?
2. If OxFml already believes unary minus is implemented, where does it believe the failing path now sits: parse, bind, prepared-call formation, or stand-in evaluation?
3. Does OxFml agree the stand-in host should treat absent single-cell worksheet references as blank cells for ordinary worksheet-reference evaluation in this packet?
4. If not, what explicit alternate blank-cell fixture convention does OxFml want OxFunc to adopt for single-cell seam rows?

## 6. Evidence And Artifact Links
OxFunc anchors:
1. `docs/function-lane/W52_RESIDUAL_SEAM_REVIEW_20260326.md`
2. `CURRENT_BLOCKERS.md` entries `BLK-FN-012` and `BLK-FN-013`
3. `crates/oxfunc_core/tests/fixtures/oxfunc_adapter_function_corpus.json`
4. `crates/oxfunc_core/tests/oxfml_seam_integration.rs`
5. `docs/function-lane/W33_EXECUTION_RECORD.md`
6. `docs/function-lane/W45_EXECUTION_RECORD.md`

Current OxFml-local likely touch points already visible from OxFunc:
1. `../OxFml/crates/oxfml_core/src/syntax/parser.rs` currently routes `parse_prefix()` directly into `parse_postfix()`
2. `../OxFml/crates/oxfml_core/src/eval/mod.rs` local reference resolution currently returns `UnresolvedReference` when a single-cell target is absent from the fixture map

## 7. Risk If Deferred
1. The broad OxFunc seam corpus remains unable to distinguish ordinary function parity drift from OxFml-side unary-literal handling defects.
2. Blank-sensitive worksheet functions cannot be trusted through the stand-in harness while absent single-cell references fail instead of behaving as blank cells.
3. Future OxFunc/OxFml mismatch rounds will keep rediscovering the same seam defects under different functions.

## 8. Current Readback
OxFml's `NOTES_FOR_OXFUNC.md` Section 29 now reports both seam corrections as implemented locally, and OxFunc-side rerun of:
1. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --test oxfml_seam_integration -- --nocapture`

confirms that the previous seam failures for:
1. `SIGN`
2. `PV`
3. `FV`
4. `ISBLANK`

no longer fail through the OxFml-backed path as seam defects. The remaining residuals are now low-order worksheet-value mismatches, which stay OxFunc-local under `W053`.

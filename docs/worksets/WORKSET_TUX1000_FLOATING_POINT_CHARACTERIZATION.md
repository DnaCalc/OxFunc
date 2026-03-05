# WORKSET - TUX1000 Floating-Point Characterization

## 1. Why This Exists
This is the second cross-cutting workset after `PI()`.

Goal:
1. fully characterize Excel floating-point behavior for worksheet operations,
2. explicitly test edge cases around IEEE-754 boundaries,
3. separate behavior at expression-evaluation boundary versus stored-sheet-value boundary.

## 2. Research Snapshot (Primary-Source Baseline)
Based on Microsoft documentation, Excel:
1. uses IEEE-754 double precision as its numeric storage basis,
2. does not fully implement optional IEEE-754 features in the same way as some scientific runtimes,
3. historically normalizes/surfaces overflow and invalid operations as worksheet errors rather than exposing raw `+/-inf` or NaN values,
4. limits precision to 15 significant digits for entered numbers.

Primary sources:
1. Microsoft Learn: Floating-point arithmetic behavior in Excel.
2. Microsoft Support: Excel specifications and limits.
3. Microsoft Learn: Data types used by Excel and XLL API references.

## 3. Scope
In scope:
1. `-0` observability and sign propagation.
2. overflow/underflow behavior and error mapping.
3. NaN and infinities (generation, import, propagation, display/storage mapping).
4. denormal/subnormal handling.
5. formula-only evaluation vs cross-cell reference behavior.
6. worksheet display/serialization effects versus internal evaluation outcomes.

Out of scope:
1. full statistical analysis for every worksheet function in this pass.
2. non-worksheet language domains (Power Query/DAX internals).

## 4. Work Lanes
1. Lane FP-A: pure formula expression tests (single-cell direct expressions).
2. Lane FP-B: sheet boundary tests (value in one cell, consumed by references in others).
3. Lane FP-C: import/interop boundary tests (XLL/UDF-provided doubles, including special values).
4. Lane FP-D: persistence tests (save/load round-trip and text export/import effects).

## 5. Required Matrix Axes
Every empirical row must include:
1. Excel app version/channel.
2. Workbook Compatibility Version.
3. locale profile (decimal/thousands separators where relevant).
4. boundary lane (`FP-A/B/C/D`).

## 6. Candidate Scenario Set
1. Signed-zero candidates:
   - underflow from negative-side operations,
   - arithmetic identities that may preserve sign bit.
2. Overflow candidates:
   - large multiplication/exponentiation expected to exceed double finite range.
3. Invalid-operation candidates:
   - expressions that would be NaN in IEEE math runtimes.
4. Denormal candidates:
   - extremely small magnitudes near subnormal range.
5. Boundary-comparison candidates:
   - run same operation direct in formula and via reference chain.
6. Interop candidates:
   - inject `-0`, `+inf`, `-inf`, and quiet/signaling NaN from UDF/XLL boundary.

## 7. Expected Outputs
1. empirical scenario manifest with reproducible formulas/actions.
2. result captures with raw observable outcomes.
3. promotion candidates for `EMP-*` findings on stable behaviors.
4. function/value-contract updates:
   - value-type rules,
   - coercion/error policy notes,
   - conformance references.

## 8. Gate Model
1. G1 baseline:
   - scenario matrix defined and runnable.
2. G2 observation:
   - baseline runs executed for at least one declared Excel version/channel and Compatibility Version.
3. G3 characterization:
   - normalized behavior map produced for `-0`, infinities, NaN, and denormals across lanes.
4. G4 promotion:
   - promoted `EMP-*` findings linked to conformance rows and value-model docs.

## 9. Status
Current: `planned`.


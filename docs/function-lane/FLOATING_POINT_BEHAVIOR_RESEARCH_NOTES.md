# Floating-Point Behavior Research Notes (Excel)

Status: `active-research`

## 1. Purpose
Capture online research findings and convert them into empirical TODOs for OxFunc characterization.

This note is intentionally not final policy; it is a planning input for the floating-point workset.

## 2. Primary Sources Reviewed
1. Microsoft Learn: "Floating-point arithmetic may give inaccurate results in Excel"
   - https://learn.microsoft.com/en-US/troubleshoot/microsoft-365-apps/excel/floating-point-arithmetic-inaccurate-result
2. Microsoft Support: "Excel specifications and limits"
   - https://support.microsoft.com/en-us/office/excel-specifications-and-limits-1672b34d-7043-467e-8e27-269d656771c3
3. Microsoft Learn (Excel C API / XLL docs entry point):
   - https://learn.microsoft.com/en-us/office/client-developer/excel/
4. Microsoft Learn (xlfEvaluate):
   - https://learn.microsoft.com/en-us/office/client-developer/excel/xlfevaluate
5. Microsoft Learn (xlCoerce):
   - https://learn.microsoft.com/en-us/office/client-developer/excel/xlcoerce

## 3. Current Working Interpretations
1. Excel numeric core uses IEEE-754 double precision representation.
2. Excel worksheet behavior appears to normalize exceptional IEEE-style outcomes into worksheet error surfaces in many cases.
3. Excel documentation indicates 15 significant digits for entered numeric precision.
4. It is plausible that behavior differs between:
   - direct expression evaluation in a formula,
   - value materialization in a cell,
   - re-consumption through references/UDF boundaries.
5. Distinct NaN payloads/signaling behaviors are likely not preserved at worksheet surface, but this must be empirically verified.

These are hypotheses until measured across explicit version axes.

## 4. Open Questions (Targeted)
1. Can signed zero (`-0`) be observed, preserved, or distinguished anywhere in worksheet-visible semantics?
2. Are overflow outcomes always mapped to the same worksheet error class?
3. How does Excel map invalid operations that would produce NaN in IEEE runtimes?
4. Does behavior differ between direct formula evaluation and referenced cell values?
5. Do XLL/UDF boundaries allow injection of raw `-0`, infinities, and NaN variants, and if so how are they normalized?
6. Are denormals/subnormals preserved, rounded to zero, or mapped to errors at any boundary?

## 5. Empirical TODO List
1. Build `FP-A` single-cell formula matrix:
   - overflow, underflow, invalid operation, boundary comparisons.
2. Build `FP-B` reference-chain matrix:
   - compute in `A1`, consume in `B1/C1`, compare against direct-expression equivalent.
3. Build `FP-C` interop matrix:
   - XLL/UDF return values for `-0`, `+inf`, `-inf`, quiet NaN, signaling NaN.
4. Build `FP-D` persistence matrix:
   - save/load round-trip and CSV/text round-trip for edge numeric values.
5. For each case, capture:
   - displayed value,
   - formula bar representation,
   - dependent formula outcomes,
   - error type and code where applicable.
6. Record required metadata:
   - Excel channel/build,
   - Compatibility Version,
   - locale profile.
7. Promote stable observations to `EMP-*` findings and bind in conformance rows.

## 6. Integration Targets in OxFunc
1. Value model:
   - clarify signed-zero/infinity/NaN representation policy.
2. Function contracts:
   - explicit domain/error behavior for numeric functions around edge cases.
3. Adapter contracts:
   - pre-call/post-call normalization rules across worksheet/UDF boundaries.
4. Conformance table:
   - add floating-point characterization references to relevant `FDEF-*` rows.


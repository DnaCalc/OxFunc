# Floating-Point Behavior Research Notes (Excel)

Status: `baseline-observed`

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
7. Is NaN payload information preserved at any worksheet-visible boundary in Excel?

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

## 7. Execution Artifacts (Current)
1. Scenario seed manifest:
   - `FLOATING_POINT_SCENARIO_MANIFEST_SEED.csv` (`FP2-001..FP2-026`).
2. Execution tracker:
   - `FLOATING_POINT_EXECUTION_RECORD.md`.
3. Current lane posture:
   - `FP-A`, `FP-B`, `FP-C`, and `FP-D` are executable with current tooling.
   - `FP-C` requires XLL build/load path (`tools/fp-probe/xll/build-fp-edge-xll.ps1` + `-XllPath`).
4. Comparative deviation ledger:
   - `FLOATING_POINT_LEAN_EXCEL_DEVIATION_LEDGER.csv`.

## 8. Lean-Comparison Working Position
1. We do not currently claim that Excel preserves NaN payload identity at worksheet-visible boundaries.
2. We do not currently claim Lean runtime FP behavior is fully Excel-equivalent.
3. W2 will establish empirical equivalence classes and explicit divergence records before any stronger claims.
4. If divergence is isolated and non-impacting for function contracts, we will document and bound it explicitly rather than introducing custom FP64 theory by default.

## 9. Baseline Observations (2026-03-05)
Observed scope:
1. Excel lanes executed: `FP-A`, `FP-B`, `FP-C`, `FP-D`.
2. Lean comparable scenarios executed: `FP2-001..FP2-010`.
3. Baseline metadata:
   - Excel `16.0 (build 19725)`,
   - channel `http://officecdn.microsoft.com/pr/492350f6-3a01-4f97-b9c0-c7c6ddf67d60`,
   - compatibility `default|CalculationVersion=191029|CheckCompatibility=False|FileFormat=51`,
   - locale `en-US`.

Key outcomes:
1. Formula-surface signed-zero display normalizes to `0` in tested `FP-A/FP-B` paths.
2. `FP-C` UDF ingress shows:
   - `+/-inf`, `qNaN`, `sNaN` -> `#NUM!`,
   - UDF `-0` -> observable `-0` display.
3. Formula divide-by-zero/invalid (`0/0`, `1/0`, `-1/0`) -> `#DIV/0!`.
4. Tiny/subnormal candidates in tested matrix show worksheet `0` and `Value2=0`.
5. Lean runtime preserves nonzero subnormal payloads in comparable cases (`FP2-008..FP2-010`), yielding explicit divergence against Excel surface normalization.

## 10. Baseline Policy Map (Provisional)
1. Signed zero:
   - Excel formula/reference surface often hides sign (`0`),
   - interop return path can expose `-0`.
2. Infinity/NaN:
   - Excel worksheet surface normalizes to error values (`#NUM!` via UDF ingress; `#DIV/0!` for divide-by-zero formula lanes).
3. Subnormals/tiny values:
   - tested worksheet/value surface normalizes to zero.
4. Lean runtime model:
   - executable IEEE-like behavior retains infinities/NaN/subnormal values as floats, including payload-distinct NaN generation support in runtime pathways.
5. Contract stance:
   - OxFunc must model Excel worksheet-observable normalization as primary contract truth,
   - Lean float behavior is comparative evidence, not default equivalence proof.

## 11. Promotion Candidates (`EMP-*`) and Deferrals
Candidates (for Foundation editorial promotion):
1. `EMP-CAND-FP-001`:
   - formula divide-by-zero/invalid operations normalize to worksheet errors rather than exposing IEEE infinities/NaNs.
2. `EMP-CAND-FP-002`:
   - formula/reference lanes normalize tested tiny/subnormal candidates to worksheet zero.
3. `EMP-CAND-FP-003`:
   - interop UDF ingress normalizes `+/-inf` and NaN variants to `#NUM!` at worksheet surface.
4. `EMP-CAND-FP-004`:
   - interop UDF ingress can surface `-0` as display `-0` even where formula lanes display `0`.

Deferral rationale:
1. Promotions are deferred pending cross-build/channel and non-default compatibility reruns.
2. Promotions are deferred pending Foundation-side `EMP-*` ID assignment and acceptance review.
3. Historical Foundation editorial prompts now live behind `docs/HISTORY.md` and the `OxFunc_V1` tag.

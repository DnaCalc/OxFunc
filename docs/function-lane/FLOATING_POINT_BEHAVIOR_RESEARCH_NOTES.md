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
6. Grid reverse-engineering note on Excel floating-point arithmetic:
   - https://docs.grid.is/apiary/explain/excel-floating-point-arithmetic/
7. Wikipedia overview of numeric precision in Microsoft Excel:
   - https://en.wikipedia.org/wiki/Numeric_precision_in_Microsoft_Excel
8. Newton Excel Bach note on floating-point comparison behavior:
   - https://newtonexcelbach.com/2012/01/07/comparing-floating-point-numbers/

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

## 12. Numeric Comparison Follow-On (2026-04-08)
Observed scope:
1. Live Excel replay plus manifest-driven `FP-E` run for:
   - `IF` / `IFS` empty-text conditions,
   - ordinary operator near-equality comparisons,
   - criteria/database numeric criteria matching,
   - `SWITCH`,
   - exact-match contrast lanes for `MATCH`, `XMATCH`, and `DELTA`,
   - arithmetic-generated 15-significant-digit boundary rows that distinguish
     truncation-style comparison normalization from round-to-nearest.
2. Baseline metadata:
   - Excel `16.0 (build 19822)`,
   - channel `http://officecdn.microsoft.com/pr/492350f6-3a01-4f97-b9c0-c7c6ddf67d60`,
   - compatibility `default|CalculationVersion=191029|CheckCompatibility=False|FileFormat=51`,
   - locale `en-US`.

Key outcomes:
1. `IF("",1,2)` and `IFS("",1,TRUE,2)` both return `#VALUE!`; the earlier
   false-branch hypothesis was not supported by replay.
2. Ordinary compare operators use a tolerant near-equality lane on the tested
   cases, including the arithmetic-generated boundary pair
   `((123456789012345*10)+5)/1E25` versus `((123456789012345*10)+4)/1E25`.
3. Criteria/database numeric criteria matching and `SWITCH` share that tolerant
   lane on the tested cases, including the same arithmetic-generated boundary
   pair.
4. `MATCH`, `XMATCH`, and `DELTA` exact-match paths remain exact on the tested
   near-equality cases, including the arithmetic-generated boundary pair, and
   therefore must not be folded into the tolerant helper by default.
5. The current observed compare rule is consistent with truncation toward zero
   to 15 significant decimal digits on the tolerant families' tested paths; the
   earlier round-to-nearest surrogate is too weak and diverges on the stronger
   boundary pair.

Working policy consequence:
1. near-equality numeric comparison is not one global OxFunc rule.
2. it must be carried only by the families empirically shown to share it:
   - ordinary compare operators,
   - criteria family,
   - database family,
   - `SWITCH`.
3. the tolerant-family helper must follow the truncation-style 15-significant-
   digit boundary currently pinned by replay, not round-to-nearest.
4. exact-match contrast families remain separate until contrary evidence exists.

## 13. Accuracy Literature Review Follow-On (2026-04-10)
Observed scope:
1. Public Microsoft documentation plus older public numerical-accuracy reviews
   were reread specifically to frame the reopened `W086` (`NORM.*`) and `W087`
   (`XIRR`) lanes.
2. Sources reviewed:
   - Microsoft Learn: "Floating-point arithmetic may give inaccurate result in
     Excel"
   - Microsoft KB archive `828888` overview on statistical-function
     improvements
   - Microsoft Support: "Excel Statistical Functions: NORMSINV"
   - Microsoft Support: "XIRR function"
   - Knusel (1998), McCullough/Wilson (1999, 2002, 2005), and
     McCullough/Heiser (2008) as cited by Microsoft and the public literature
   - GRID's public reverse-engineering note on Excel floating-point behavior

Key takeaways:
1. Microsoft explicitly documents that Excel uses IEEE-754 doubles but limits
   worksheet-visible precision to 15 significant digits and does not implement
   IEEE denormals, infinities, or NaN surfaces directly.
2. Microsoft also explicitly acknowledges that multiple Excel statistical
   functions had real algorithmic accuracy issues in older releases and that
   several distribution functions, including `NORMSDIST`/`NORMSINV`, were
   revised in Excel 2002/2003.
3. The public statistical-accuracy literature is therefore directly relevant to
   `W086`: small exact-value drifts in `NORM.*` are not automatically "display
   only" differences; they sit in a historically explored class of genuine
   numerical-approximation issues.
4. Microsoft describes `XIRR` only in terms of an iterative search from a
   `guess`, a `0.000001 percent` accuracy target, and a `100`-try cap. That is
   not a full returned-root specification.
5. Our current local replay, combined with Microsoft's sparse `XIRR`
   documentation, supports a narrower framing for `W087`: matching Excel may
   require reproducing Excel's returned-root/stopping behavior, not merely
   finding a mathematically "better" root for the same `XNPV` equation.
6. GRID's public reverse-engineering note is useful as secondary evidence that
   Excel often mixes exact internal doubles with output-facing normalization and
   comparison policies. That supports keeping tiny last-digit finance rows
   separate from `W086`/`W087` until the comparison-policy packet is decided.

Working consequence for current OxFunc packets:
1. `W086` is a legitimate exact-value reconciliation packet; the external
   literature supports treating this as a known Excel-accuracy class, not a
   mere formatting issue, and the bounded current-scope repair is now landed on
   committed ref `8234dce5f3e0c50a3c634466ead38c67fa93937e`.
2. `W087` no longer needs open characterization for the bounded current scope:
   the widened empirical model of Excel's returned `XIRR` value now exists and
   the bounded OxFunc repair is landed on committed ref
   `8234dce5f3e0c50a3c634466ead38c67fa93937e`.
3. `PPMT`, `CUMPRINC`, and `EFFECT` remain better framed as comparison-policy
   rows for now unless a larger empirical delta appears.

## 14. XIRR Publication Follow-On (2026-04-11)
Observed scope:
1. Live Excel `Value2` replay was widened from the original `W087` seed witness
   to adjacent multi-cashflow positive-root `XIRR` rows across guesses `0.01`,
   omitted/`0.1`, `0.5`, and `1.0`.
2. The widened exact targets are now pinned in
   `docs/function-lane/W24_BATCH12_CASHFLOW_RATE_SCENARIO_MANIFEST_SEED.csv`.
3. The current OxFunc working-tree publication rule was compared against those
   exact `Value2` rows and against the midpoint ladder of the existing
   bracket-and-bisection path.

Key outcomes:
1. The widened live Excel targets are exact bisection midpoints on the same
   bracket path as the current OxFunc implementation. That rules out bracket
   construction as the remaining source of drift.
2. Earlier characterization narrowed the candidate mismatch to
   midpoint-publication choice: on some adjacent rows Excel appeared to return
   an earlier midpoint, while on others it appeared to return a later midpoint,
   even though all targets sat on the same ladder.
3. No single simple current-midpoint rule based on:
   - relative width,
   - absolute width,
   - residual magnitude,
   - residual sign,
   - or the Microsoft-documented `r +/- 1e-8` bracket check
   explains the widened exact `Value2` matrix.
4. The Microsoft `0.000001 percent` documentation remains useful as a necessary
   bound, but not a sufficient returned-root specification for exact Excel
   parity on the widened `XIRR` family.
5. Two additional simple reverse-engineering candidates were tested and ruled
   out on the widened matrix:
   - first midpoint where 15-significant-digit decimal truncation or rounding
     stabilizes,
   - first midpoint whose local Newton correction `|f/f'|` falls below a
     single fixed threshold.

Working consequence:
1. `W087` remains an empirical publication-rule packet, not a generic
   solver-accuracy packet.
2. Focused local re-verification now shows the landed ref
   `8234dce5f3e0c50a3c634466ead38c67fa93937e` matches the bounded widened
   witness matrix pinned for `W087`; broader repo verification is orthogonal,
   not an open blocker on the bounded `XIRR` lane.
3. The exact-value witness floor is now strong enough to prevent regression
   back to displayed-value approximations while the remaining publication rule
   is characterized.

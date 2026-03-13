# XLL Nil Propagation Execution Record

Status: `provisional`
Owner lane: `OxFunc`
Evidence ID: `W9-XLL-NIL-20260312`

## 1. Purpose
Record empirical behavior for raw `xltypeNil` returns from the XLL bridge, including direct publication, nested propagation, array-element preservation, and scalarization.

## 2. Executed Scope
Execution date:
1. `2026-03-12`

Environment:
1. Excel version/build: `16.0 (build 19725)`
2. Excel channel: `http://officecdn.microsoft.com/pr/492350f6-3a01-4f97-b9c0-c7c6ddf67d60`
3. Locale profile: `en-US`

Inputs:
1. Manifest:
   - `docs/function-lane/XLL_NIL_PROPAGATION_SCENARIO_MANIFEST_SEED.csv`
2. Probe runner:
   - `tools/xll-addin/run-xll-nil-probe.ps1`
3. Probe exports:
   - `tools/xll-addin/oxfunc_xll/src/lib.rs`

Outputs:
1. Result rows:
   - `.tmp/xll-nil-probe-results.csv`

## 3. Observed Outcomes
### 3.1 Scalar raw `xltypeNil`
1. Direct scalar return `=ox_PROBE_RET_NIL()` published to the worksheet as numeric zero.
2. Nested custom observation `=ox_PROBE_DESCRIBE(ox_PROBE_RET_NIL())` observed the inner result as `number`, not `empty_cell`.
3. Built-in consumers matched numeric-zero semantics:
   - `TYPE(ox_PROBE_RET_NIL()) -> 1`
   - `N(ox_PROBE_RET_NIL()) -> 0`
   - `T(ox_PROBE_RET_NIL()) -> ""`
   - `1 + ox_PROBE_RET_NIL() -> 1`
4. Echoing the raw nil through another XLL probe still presented as `number` to the outer observer.

### 3.2 Arrays containing raw `xltypeNil` elements
1. A direct array return with nil elements spilled as worksheet-visible zeros in the nil positions.
2. Outer custom observation of the returned array preserved nil-origin elements as `empty_cell` inside the array value:
   - `array(2x2)[empty_cell,number,text,empty_cell]`
3. Scalarization through `INDEX` collapsed a nil element to numeric-zero semantics:
   - `ox_PROBE_DESCRIBE(INDEX(...,1,1)) -> number`
   - `TYPE(INDEX(...,1,1)) -> 1`
   - `N(INDEX(...,1,1)) -> 0`
   - `T(INDEX(...,1,1)) -> ""`

## 4. Interpretation
1. Raw scalar `xltypeNil` is not preserved as an intermediate observable value across ordinary formula nesting in the current Excel/XLL baseline.
2. Raw scalar `xltypeNil` normalizes before outer argument binding; the outer function/operator sees numeric-zero semantics.
3. Raw `xltypeNil` inside an array is different:
   - it can survive inside a returned array as `empty_cell`-like element state,
   - but it still collapses to numeric-zero semantics when scalarized or published into worksheet cells.
4. This means the value-universe split should distinguish:
   - raw function return,
   - array-element payload state,
   - published formula result.

## 5. Implications for OxFunc / OxFml
1. Built-in function completion doctrine should continue to exclude scalar `empty_cell` / `nil` from ordinary published scalar results.
2. OxFunc should still be able to model a broader raw return universe for interop/UDF seams.
3. Formalization should make the normalization map explicit rather than treating `nil` as either globally admitted or globally forbidden.
4. Current evidence supports:
   - scalar raw nil -> published/intermediate numeric zero,
   - array raw nil element -> intermediate array `empty_cell` element, with later collapse on scalarization/publication.

## 6. Artifacts
1. `docs/function-lane/XLL_NIL_PROPAGATION_SCENARIO_MANIFEST_SEED.csv`
2. `tools/xll-addin/run-xll-nil-probe.ps1`
3. `tools/xll-addin/oxfunc_xll/src/lib.rs`
4. `.tmp/xll-nil-probe-results.csv`

# BUGREP-FUNC-007: Local Excel probe on POWER zero-to-zero domain error

## Intake
- **Report id**: `BUGREP-FUNC-007`
- **Filed**: 2026-04-08
- **Source channel**: local test
- **Reporter/source**: live Excel COM probe on current workstation
- **Reported against ref**: `7989fafaef703f15f2bfbdded323c03345da1072`
- **Reported against kind**: `commit`
- **Reported against note**: Exact working ref pinned with `git rev-parse HEAD` at intake time.
- **Canonical bug id**: `BUG-FUNC-005`
- **Status**: `triaged`

## Observed Symptom
Live Excel reports both `=0^0` and `=POWER(0,0)` as `#NUM!`, while the current
local OxFunc shared power kernel publishes `1` for the same zero-to-zero lane.

## Reproduction
1. Evaluate `=0^0` and `=POWER(0,0)` against a live local Excel instance.
2. Expected result: `#NUM!` on both entrypoints.
3. Actual current OxFunc result before the local fix: `1` on both entrypoints
   because the shared zero-exponent fast path returns `1` before checking the
   zero-base domain lane.

## Initial Ownership Read
- **Initial classification**: `OxFunc-owned bug`
- **Reason**: `POWER` and `OP_POWER` both route through the shared OxFunc-local
  `power_kernel`, and the divergence is in that local runtime/formal stack.

## Links
1. `crates/oxfunc_core/src/functions/power_fn.rs`
2. `crates/oxfunc_core/src/functions/operator_arithmetic_family.rs`
3. `docs/function-lane/W45_WAVEA_OPERATOR_ARITHMETIC_SCENARIO_MANIFEST_SEED.csv`
4. `docs/function-lane/W53_NUMERIC_FORENSICS_20260326.md`
5. `docs/worksets/W078_POWER_ZERO_TO_ZERO_NUM_ERROR_PARITY.md`

## Triage Notes
This is a unique local parity bug, not a duplicate. It opens canonical stream
`BUG-FUNC-005` and bounded owner workset `W078`.

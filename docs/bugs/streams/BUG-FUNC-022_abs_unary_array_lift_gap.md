# BUG-FUNC-022: ABS unary array-lift gap from W089 scenario seeds

## Summary
- **Bug id**: `BUG-FUNC-022`
- **Opened**: `2026-04-30`
- **Status**: `closed`
- **Owner workset**: `W089`
- **Bead**: `oxf-xmhu`

## Source Refs
- **Reported against ref**: W089 comprehensive seed run working tree before
  `add56eeb6a0fdc49055fcab4222bb680a30c05ff`
- **Reproduced on ref**: `w089-comprehensive-seed-20260430-001`
- **Introduced in ref**: `unknown`
- **Fixed in ref**: `add56eeb6a0fdc49055fcab4222bb680a30c05ff`
- **Ref notes**: live Excel COM comparison used Excel `16.0` build `19929`,
  workbook Compatibility Version `2`, and exact typed bit comparison.

## Ownership And Root Cause
- **Ownership class**: `OxFunc-owned bug`
- **Root cause class**: `test_gap`
- **Root cause summary**: `ABS` was omitted from the observed scalar
  array-lift position table. Local execution therefore treated inline array
  arguments as scalar-inadmissible and returned `#VALUE!`, while current Excel
  spills elementwise for `ABS` over inline array literals.

## Reproduction
Initial W089 scenario-seed replay:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File smart-fuzzer\tools\Run-ArraySupportTranche.ps1 `
  -RunId w089-comprehensive-seed-20260430-001 `
  -CaseSetPath smart-fuzzer\cache\scenario-seed-executable-cases-v0.json
```

Representative rows from that replay:

1. `=ABS({-1,2})`: local `#VALUE!`, Excel `array:1x2`.
2. `=ABS({-1,"asd",2})`: local `#VALUE!`, Excel elementwise array with
   `#VALUE!` only for the text cell.

## Fix
Landed on `add56eeb6a0fdc49055fcab4222bb680a30c05ff`:

1. added `FUNC_ID_ABS` to `observed_scalar_array_lift_positions`,
2. added focused dispatcher coverage for `ABS({-1,"bad",2})` returning
   `{1,#VALUE!,2}`.

## Validation
1. `cargo test --manifest-path crates\oxfunc_core\Cargo.toml --lib observed_scalar_array_lift_handles_abs_arrays`
   - `1` passed, `0` failed.
2. `cargo check --manifest-path smart-fuzzer\tools\pmt_ppmt_local_eval\Cargo.toml`
   - passed.
3. `w089-comprehensive-seed-20260430-004`
   - `FUNC.ABS`: `7` exact typed bit matches and `1` Excel harness blocker
     for invalid `ABS()` formula assignment; no ABS array-lift mismatch remains.

## Similar-Risk Scan
This was found by the first broad W089 manifest-seed replay after W090 array
support repairs. Adjacent unary array-lift surfaces covered by the same replay
either matched or are represented in the residual W089/W090 exactness streams;
this stream only claims the observed `ABS` array-lift gap.

## Evidence
1. `smart-fuzzer/runs/w089-comprehensive-seed-20260430-001/`
2. `smart-fuzzer/runs/w089-comprehensive-seed-20260430-004/`
3. `crates/oxfunc_core/src/functions/surface_dispatch.rs`
4. Bead: `oxf-xmhu`

## Closure Checklist
- [x] fix landed or non-OxFunc ownership recorded
- [x] validation recorded
- [x] root cause recorded
- [x] similar-risk scan recorded
- [x] spec/matrix/contract updated if required: not required
- [x] handoff filed if required: not required; no FEC/F3E boundary change

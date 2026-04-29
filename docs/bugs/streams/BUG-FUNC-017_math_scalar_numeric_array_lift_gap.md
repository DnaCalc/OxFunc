# BUG-FUNC-017: Math scalar-numeric array-lift gap

## Summary
- **Bug id**: `BUG-FUNC-017`
- **Opened**: `2026-04-30`
- **Status**: `closed`
- **Owner workset**: `W090`

## Source Refs
- **Reported against ref**: `e9886ef7b57ba24be2ec5f8814b651b7d640c80f`
- **Reproduced on ref**: `e9886ef7b57ba24be2ec5f8814b651b7d640c80f`
- **Introduced in ref**: `unknown`
- **Fixed in ref**: `0b966d0ee7c8ce4a327b0b3090f9a108248c37fd`
- **Ref notes**: W090 tranche A used live Excel COM on Excel `16.0`
  build `19929`, workbook Compatibility Version `2`, with exact typed
  equality and bit-exact numeric comparison.

## Ownership And Root Cause
- **Ownership class**: `OxFunc-owned bug`
- **Root cause class**: `test_gap`
- **Root cause summary**: several scalar numeric math surfaces still used
  scalar-only prepared coercion for value arguments even though current Excel
  spills inline array-valued scalar parameters elementwise or by broadcast.

## Reproduction
Run:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File smart-fuzzer\tools\Run-ArraySupportTranche.ps1 -RunId w090-array-tranche-a-local-010
```

Initial run `w090-array-tranche-a-local-010`:

1. cases: `34`,
2. exact typed bit matches: `2`,
3. unexpected mismatches: `32`,
4. matched surface: `MROUND`,
5. mismatching surfaces: `ROUND`, `ROUNDDOWN`, `ROUNDUP`, `TRUNC`,
   `CEILING`, `CEILING.MATH`, `CEILING.PRECISE`, `FLOOR`, `FLOOR.MATH`,
   `FLOOR.PRECISE`, `ISO.CEILING`, `ATAN2`, and `BASE`.

Representative mismatch classes:

1. `ROUND({1.234,2.345},1)`: local `#VALUE!`, Excel `array:1x2`.
2. `TRUNC({1.234;2.345})`: local `#VALUE!`, Excel `array:2x1`.
3. `ATAN2({0,1},{0,1})`: local `#VALUE!`, Excel `{#DIV/0!,0.785398...}`.
4. `BASE(15,16,{2,4})`: local `#VALUE!`, Excel `{"0F","000F"}`.

## Fix
Landed on `0b966d0ee7c8ce4a327b0b3090f9a108248c37fd`:

1. added a generic prepared-argument broadcast helper for value-only surfaces,
2. switched `ROUND`, `ROUNDDOWN`, `ROUNDUP`, and `ATAN2` onto the existing
   binary numeric broadcast evaluator,
3. added optional-argument array lift for `TRUNC`,
4. added exact/optional two- and three-argument broadcast support for the
   ceiling/floor family,
5. added text-returning broadcast support for `BASE`,
6. added focused Rust unit coverage and the W090 array-tranche comparator.

## Validation
1. `cargo check --manifest-path crates/oxfunc_core/Cargo.toml`
2. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib -- --nocapture`
   - `1249` passed, `0` failed, `1` ignored
3. Rerun:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File smart-fuzzer\tools\Run-ArraySupportTranche.ps1 -RunId w090-array-tranche-a-local-011
```

Rerun `w090-array-tranche-a-local-011`:

1. cases: `34`,
2. exact typed bit matches: `34`,
3. mismatches: `0`,
4. failure packets: `0`,
5. Excel environment: Excel `16.0` build `19929`, workbook Compatibility
   Version `2`.

## Similar-Risk Scan
The W090 candidate inventory remains the successor queue for broader
array-support review. This repair covers only tranche-A surfaces and the
argument shapes exercised by the W090 replay matrix. It does not claim the full
supported function surface has been reviewed.

## Evidence
1. `smart-fuzzer/tools/Run-ArraySupportTranche.ps1`
2. `smart-fuzzer/tools/pmt_ppmt_local_eval/src/bin/array_tranche_local_eval.rs`
3. ignored run artifacts under:
   - `smart-fuzzer/runs/w090-array-tranche-a-local-010/`
   - `smart-fuzzer/runs/w090-array-tranche-a-local-011/`
4. Bead: `oxf-k7ux`

## Closure Checklist
- [x] fix landed or non-OxFunc ownership recorded
- [x] validation recorded
- [x] root cause recorded
- [x] similar-risk scan recorded
- [x] spec/matrix/contract updated if required
- [x] handoff filed if required: not required; no FEC/F3E boundary change

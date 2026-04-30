# BUG-FUNC-020: EXPAND array-valued pad_with panic

## Summary
- **Bug id**: `BUG-FUNC-020`
- **Opened**: `2026-04-30`
- **Status**: `open`
- **Owner workset**: `W090`
- **Bead**: `oxf-9pcl`

## Source Refs
- **Reported against ref**: `8b140b50bf7f07153f87ac197cf99c470cad9ae8`
- **Reproduced on ref**: current W090 successor sweep working tree
- **Introduced in ref**: `unknown`
- **Fixed in ref**: `unfixed`

## Ownership And Root Cause
- **Ownership class**: `OxFunc-owned bug`
- **Root cause class**: `robustness_gap`
- **Root cause summary**: `EXPAND` reaches an `unreachable!()` path in
  `dynamic_array_reshape_family.rs` when `pad_with` is array-valued. Current
  Excel returns `#VALUE!` for the same probe.

## Reproduction
Run:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File smart-fuzzer\tools\Run-ArraySupportTranche.ps1 `
  -RunId w090-successor-lookup-and-reference-functions-final-003 `
  -CaseSetPath smart-fuzzer\cache\array-support-successor-executable-tranches-v0.json `
  -CaseSetTrancheId w090-successor-lookup-and-reference-functions
```

Representative mismatch:

```excel
=EXPAND({1,2;3,4},3,4,{"x","x"})
```

Observed result:

1. local execution status: `local_eval_panic`
2. local digest: `harness_error:internal error: entered unreachable code`
3. Excel outcome: `#VALUE!`

The smart-fuzzer local evaluator now catches this panic per case so the sweep
can continue, but the function implementation must still return an ordinary
worksheet error instead of panicking.

## Evidence
1. `smart-fuzzer/runs/w090-successor-lookup-and-reference-functions-final-003/`
2. `smart-fuzzer/tools/pmt_ppmt_local_eval/src/bin/array_tranche_local_eval.rs`
3. `smart-fuzzer/planning/ARRAY_SUPPORT_SUCCESSOR_SWEEP_20260430.md`
4. Bead: `oxf-9pcl`

## Closure Checklist
- [ ] fix landed or non-OxFunc ownership recorded
- [ ] validation recorded
- [ ] root cause recorded
- [ ] similar-risk scan recorded
- [ ] spec/matrix/contract updated if required
- [ ] handoff filed if required

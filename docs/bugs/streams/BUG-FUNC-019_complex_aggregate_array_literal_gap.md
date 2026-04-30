# BUG-FUNC-019: Complex aggregate array-literal gap

## Summary
- **Bug id**: `BUG-FUNC-019`
- **Opened**: `2026-04-30`
- **Status**: `open`
- **Owner workset**: `W090`
- **Bead**: `oxf-bp23`

## Source Refs
- **Reported against ref**: `8b140b50bf7f07153f87ac197cf99c470cad9ae8`
- **Reproduced on ref**: current W090 successor sweep working tree
- **Introduced in ref**: `unknown`
- **Fixed in ref**: `unfixed`

## Ownership And Root Cause
- **Ownership class**: `OxFunc-owned bug`
- **Root cause class**: `test_gap`
- **Root cause summary**: `IMPRODUCT` and `IMSUM` currently reject array
  literal arguments with `#VALUE!`, while current Excel flattens/aggregates
  those array literals and returns a scalar complex text result.

## Reproduction
Run:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File smart-fuzzer\tools\Run-ArraySupportTranche.ps1 `
  -RunId w090-successor-engineering-functions-final-002 `
  -CaseSetPath smart-fuzzer\cache\array-support-successor-executable-tranches-v0.json `
  -CaseSetTrancheId w090-successor-engineering-functions
```

Representative mismatch anchors from
`w090-successor-engineering-functions-final-002`:

1. `IMPRODUCT({"3+4i","3+4i"},"1-2i")`
   - local: `#VALUE!`
   - Excel: `41+38i`
2. `IMPRODUCT("3+4i",{"1-2i","1-2i"})`
   - local: `#VALUE!`
   - Excel: `7-24i`
3. `IMSUM({"3+4i","3+4i"},"1-2i")`
   - local: `#VALUE!`
   - Excel: `7+6i`
4. `IMSUM("3+4i",{"1-2i","1-2i"})`
   - local: `#VALUE!`
   - Excel: `5`

## Repair Direction
Treat this as aggregate array-literal admission, not shape-preserving scalar
lift. Confirm whether adjacent complex aggregate functions share the same
flattening policy before changing a shared adapter.

## Evidence
1. `smart-fuzzer/runs/w090-successor-engineering-functions-final-002/`
2. `smart-fuzzer/planning/ARRAY_SUPPORT_SUCCESSOR_SWEEP_20260430.md`
3. Bead: `oxf-bp23`

## Closure Checklist
- [ ] fix landed or non-OxFunc ownership recorded
- [ ] validation recorded
- [ ] root cause recorded
- [ ] similar-risk scan recorded
- [ ] spec/matrix/contract updated if required
- [ ] handoff filed if required

# BUG-FUNC-019: Complex aggregate array-literal gap

## Summary
- **Bug id**: `BUG-FUNC-019`
- **Opened**: `2026-04-30`
- **Status**: `closed`
- **Owner workset**: `W090`
- **Bead**: `oxf-bp23`

## Source Refs
- **Reported against ref**: `8b140b50bf7f07153f87ac197cf99c470cad9ae8`
- **Reproduced on ref**: current W090 successor sweep working tree
- **Introduced in ref**: `unknown`
- **Fixed in ref**: `pending repair commit`

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

## Repair Outcome
The repair keeps scalar complex binary functions (`IMDIV`, `IMPOWER`, `IMSUB`)
on scalar preparation so dispatch-level array lift can preserve result shape,
while `IMSUM` and `IMPRODUCT` now flatten array literals through the aggregate
value expansion path.

The same pass also changed complex text number rendering to Excel-style
15-significant-digit output for these engineering functions and changed
`IMTAN`/`IMCOT` to the stable closed-form identities that match the live Excel
baseline for the W090 probe.

Validation:

1. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib`
   - `1254` passed, `0` failed, `1` ignored.
2. `smart-fuzzer/runs/w090-repair-final-engineering-functions-001`
   - `32/32` exact typed bit matches.

## Evidence
1. `smart-fuzzer/runs/w090-successor-engineering-functions-final-002/`
2. `smart-fuzzer/planning/ARRAY_SUPPORT_SUCCESSOR_SWEEP_20260430.md`
3. Bead: `oxf-bp23`

## Closure Checklist
- [x] fix landed or non-OxFunc ownership recorded
- [x] validation recorded
- [x] root cause recorded
- [x] similar-risk scan recorded
- [x] spec/matrix/contract updated if required
- [x] handoff filed if required

No handoff was required for this local OxFunc repair.

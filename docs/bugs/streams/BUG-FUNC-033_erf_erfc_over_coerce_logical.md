# BUG-FUNC-033: ERF/ERFC over-coerce logical operands (Excel returns #VALUE!)

## Summary
- **Bug id**: `BUG-FUNC-033`
- **Opened**: `2026-05-28`
- **Status**: `fixed` (2026-05-28)
- **Owner workset**: `W090` (smart-fuzzer un-poked completion sweep)

## Source Refs
- **Reported against ref**: working tree at run `scalar-swept-structural-001`
- **Reproduced on ref**: runs `scalar-swept-structural-001`, `erf-logical-recheck-001`
- **Introduced in ref**: `unknown`
- **Fixed in ref**: `erf-logical-recheck-001 (working tree)`
- **Ref notes**: live Excel COM, Excel `16.0` build `20026`, exact typed equality.

## Ownership And Root Cause
- **Ownership class**: `OxFunc-owned bug`
- **Root cause class**: `spec_mismatch`
- **Root cause summary**: `ERF`, `ERF.PRECISE`, `ERFC`, `ERFC.PRECISE`
  coerced a logical operand to a number (TRUE→1) and computed, where Excel
  returns `#VALUE!` for a logical operand. Excel's ERF/ERFC family rejects
  logicals but still accepts numeric text (`=ERFC("2")` coerces). The sibling
  GAMMA/GAMMALN family does the opposite — it accepts logicals
  (`=GAMMALN.PRECISE(TRUE)` → GAMMALN(1) ≈ 0) — so the rejection must be
  scoped to ERF/ERFC, not the shared unary special-dist path.

## Reproduction
```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File smart-fuzzer\tools\Build-UnsweptStructuralProbes.ps1 `
  -IncludeStatuses structural_bug_open -OnlySurfaces ERF,ERF.PRECISE,ERFC,ERFC.PRECISE,GAMMALN.PRECISE `
  -TrancheId erf-logical-recheck-v0 -CaseIdPrefix erfrecheck -OutputPath smart-fuzzer\cache\erf-logical-recheck-v0.json
powershell -NoProfile -ExecutionPolicy Bypass -File smart-fuzzer\tools\Run-ArraySupportTranche.ps1 `
  -RunId erf-logical-recheck-001 -CaseSetPath smart-fuzzer\cache\erf-logical-recheck-v0.json
```

| Formula | OxFunc (before) | Excel |
| --- | --- | --- |
| `=ERF(TRUE)` | `number: erf(1)` | `#VALUE!` |
| `=ERFC(TRUE)` | `number: erfc(1)` | `#VALUE!` |
| `=ERF("2")` | `erf(2)` | `erf(2)` (accepted; unchanged) |

## Fix
Fixed. Added a logical-policy coercion in
`crates/oxfunc_core/src/functions/special_dist_family.rs`
(`coerce_operand_with_logical_policy`) and a `reject_logical` flag on the
shared `eval_unary_prepared`: ERF/ERF.PRECISE/ERFC/ERFC.PRECISE pass
`reject_logical = true` (logical operand → `#VALUE!`); GAMMA/GAMMALN/
GAMMALN.PRECISE pass `false` (logicals accepted). Numeric and numeric-text
operands are unaffected.

## Validation
- Rust unit test `erf_family_rejects_logical_but_gamma_family_accepts_it`;
  full `oxfunc_core` lib suite green (`1315 passed`).
- Excel differential `erf-logical-recheck-001`: all four ERF/ERFC
  `arg0_logical` probes now `error:Value` (match Excel); ERF `arg0_text_number`
  still matches (text accepted); GAMMALN.PRECISE still accepts logical.

## Similar-Risk Scan
- GAMMA/GAMMALN family verified to still accept logical (not over-rejected).
- Remaining ERFC `arg0_text_number` 2-ULP drift and GAMMALN.PRECISE(TRUE)
  near-zero drift are numeric-drift findings (separate beads
  oxf-wkvk / GAMMALN drift), not coercion.
- ERF-family array-lift gap remains open under BUG-FUNC-028 (oxf-0f9r); when
  fixed, the per-element coercion must keep this logical rejection.

## Closure Checklist
- [x] fix landed or non-OxFunc ownership recorded
- [x] validation recorded
- [x] root cause recorded
- [x] similar-risk scan recorded
- [ ] spec/matrix/contract updated if required
- [ ] handoff filed if required

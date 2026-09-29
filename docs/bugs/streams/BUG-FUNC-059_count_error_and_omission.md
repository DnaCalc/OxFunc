# BUG-FUNC-059: COUNT and COUNTA error and omitted-slot behavior

## Current campaign observation (2026-09-29)

COUNT, COUNTA and COUNTBLANK match 2,238 admitted typed observations. Eighteen COUNTBLANK formula-entry rejections and 17 misdeclared IF reference-return cases are withheld; corrected controls pass. Provider/reference integration and wider argument domains remain open.

Status remains `investigating`: `scope_partial`, `target_partial`, integration `partial`.
The historical discovery text below is retained; its pending counts are superseded
by this paragraph and the linked family evidence.

## Retained discovery record

Status: `investigating`; owner W111; bead `oxf-mwue.28.10`.

Fresh public-interface Excel 16.0 build 20430, 64-bit, Compatibility Version 2
observations show COUNT ignores error values across direct, array and reference
origins. Both COUNT and COUNTA count every explicitly omitted argument slot as
one. The current candidate corrects these local policies; separate COUNTBLANK
findings reopen BUG-FUNC-011. No common error rule is imposed on other aggregates.

Evidence: `docs/function-lane/evidence/w111-broad-20260929/count-family/` and
`smart-fuzzer/runs/w111-count-origin-typed-20260929/`. The 476-case packet has
458 admitted observations and 18 COUNTBLANK formula-entry rejections. Keep those
admission failures separate from runtime values. The channel is unverified.

`scope_completeness=scope_partial`, `target_completeness=target_partial`,
`integration_completeness=partial`. Open lanes: computed-value controls,
independent heldout, full source-kind characterization, shared text/context
integration and combined validation. Reference/array origin is preserved; any
receiving-evaluator semantic change requires handoff and acknowledgment.

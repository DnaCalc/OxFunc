# BUG-FUNC-064: INT parity TRUNC GCD LCM and PERMUTATIONA conversion gaps

Status: `investigating`; owner W111; bead `oxf-mwue.28.14`.

The refined INT graph matches 123,229 retained observations, including 31,210 fresh inputs after the final boundary refinement. ISEVEN and ISODD independently match 36,200 and 36,150 fresh rows. GCD/LCM match 13,742 numeric and 2,078 typed rows; their order and blank rules are in HO-FN-027. PERMUTATIONA matches the second independent 25,321-row bank and 85 discovery plus 252 fresh prepared rows after decimal-power and missing/padding repairs. Rounding endpoint arithmetic, raw numeric payloads, contextual parsing and receiving dependencies remain open; no whole-function promotion follows.

Reference: public-interface Excel 16.0 build 20430, 64-bit, workbook Compatibility
Version 2; channel unverified. Retained evidence:
`docs/function-lane/evidence/w111-broad-20260929/integer-publication/`.
All observations use exact numeric Value 2 inputs or explicitly qualified typed
fixtures. Discovery and failed independent candidates remain distinct.

`scope_completeness=scope_partial`, `target_completeness=target_partial`,
`integration_completeness=partial`. Open lanes: stated residuals, independent
refinement validation, primitive/platform alignment, preparation/host context,
evaluator integration and combined campaign verification. No whole-function
completion claim is made.

QUOTIENT follow-up (`oxf-mwue.28.14.5`): the local reference-origin and
ordered-padding refinements now match 3,141 numeric and 1,720 typed observations,
including 624 fresh independent cases. Failed earlier candidates remain retained
in `docs/function-lane/evidence/w111-broad-20260929/quotient/`; HO-FN-027 now
records RefsVisibleInAdapter/Custom and the position-sensitive rejection rule.
Receiving integration and broader context/reference classes remain open.

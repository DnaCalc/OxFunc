# HO-FN-027: GCD, LCM and QUOTIENT argument semantics

Direction: OxFunc → OxFml. Status: `filed`, acknowledgment pending.
Owner: W111, `oxf-mwue.28.14`; receiving dependency `oxf-mwue.28.14.2`
(`BLK-W111-AGGREGATE-SEAM`).
The later QUOTIENT refinement is tracked by `oxf-mwue.28.14.5`.

The [impact assessment](../function-lane/evidence/w111-broad-20260929/gcd-lcm/HO_FN_027_IMPACT_DRAFT.md)
and [replay evidence](../function-lane/evidence/w111-broad-20260929/gcd-lcm/README.md)
record GCD/LCM preparation and reduction. The candidate matches 13,742 numeric
and 2,078 admitted typed observations, including 7,104 numeric and 1,160 typed
fresh independent cases. Two earlier empty-call Formula2 entry failures are
retained separately and provide no worksheet result.

Coercion visits arguments and array cells in forward order before domain checks.
Numeric text is parsed; logical values are rejected. A missing first argument
produces NA, while later missing arguments are ignored. Scalar blank references
are ignored, but blank coordinates within a multi-cell reference contribute
zero. Calls with no remaining values produce VALUE. Preserve raw negative
fractions until domain validation rather than truncating them in the evaluator.

LCM then reduces the validated numeric stream in reverse argument and reverse
row-major cell order. Binary64 rounding of intermediate products makes that
ordering observable. Sparse reference enumeration must retain declared extent,
cell positions, implicit blanks and stable row-major order. The existing provider
facts suffice; no new API is proposed. OxFml should confirm that its preparation
and cache/declaration interpretation preserve those facts.

QUOTIENT's separately exercised binary policy rejects logical values with VALUE
and Missing with NA, treats a unit blank reference as zero, and preserves binary
array broadcasting and argument error order. The earlier 504 typed observations
contained no multi-cell references. A later 80-case reference diagnostic shows
that multi-cell references reject with VALUE at their argument position, even
for aligned callers; explicit materialization as an array lifts. QUOTIENT now
declares RefsVisibleInAdapter and Custom, so consumers must preserve reference
origin until its native surface makes that distinction.

A first 512-case independent packet confirmed the reference rule but exposed
five unequal-array cases where a later absent coordinate replaced an earlier
coercion error. The refined local ordered binary call preserves each coercion
or padded NA at its own position. Its subsequent 624-case independent packet
matches entirely. All seven frozen source hashes and the function's metadata
row were unchanged. Current retained evidence is 3,141 numeric and 1,720 typed
matches; earlier failures remain retained. See the
[QUOTIENT evidence](../function-lane/evidence/w111-broad-20260929/quotient/README.md).
Generic adapters and the numerical kernel are unchanged by this refinement.

The reference is Excel 16.0 build 20430, 64-bit, Compatibility Version 2; channel
unverified. Contextual numeric-text parsing remains the separate HO-FN-022
dependency. Receiving-side tests and acknowledgment are required before any
integration claim.

`scope_completeness=scope_partial`, `target_completeness=target_partial`,
`integration_completeness=partial`. Open lanes: contextual parsing, wider
reference classes, resolver failure order, receiving acknowledgment and exercised
evaluator integration.

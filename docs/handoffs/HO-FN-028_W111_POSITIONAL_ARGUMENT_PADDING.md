# HO-FN-028: financial, PERMUTATIONA and rounding argument preparation

Direction: OxFunc → OxFml. Status: `filed`, acknowledgment pending.
Owner: W111, `oxf-mwue.28.14`; receiving dependency `oxf-mwue.28.14.3`
(`BLK-W111-POSITIONAL-SEAM`).

The [financial assessment](../function-lane/evidence/w111-broad-20260929/depreciation/POSITIONAL_PADDING.md)
and [PERMUTATIONA assessment](../function-lane/evidence/w111-broad-20260929/integer-publication/PERMUTATIONA_PREPARATION_IMPACT.md)
and [rounding assessment](../function-lane/evidence/w111-broad-20260929/rounding-boundaries/PREPARED_IMPACT_ASSESSMENT.md)
record the preparation change for SLN, SYD, DB, DDB, VDB, PERMUTATIONA,
ROUND, ROUNDUP, ROUNDDOWN and TRUNC.
All ten surfaces preserve a coordinate absent from an argument as NA at that
argument's original position. Left-to-right coercion then selects the error.
An earlier explicit error or invalid text takes precedence over a later padded
coordinate. Numeric domain validation runs after coercion of all arguments.

Required explicit missing numeric arguments become zero. For optional
financial parameters, argument absence selects the existing default; explicit
missing or blank remains a present numeric zero. Padding NA is an error value,
and must remain distinct from either missing or blank. Preserve reference
origins, positions and extents until function preparation.

The financial wrappers already declare SurfaceNative. PERMUTATIONA retains
SurfaceNative and changes its coercion declaration from
UnaryNumericScalarOnly to Custom to describe its exercised two-argument
preparation. The dispatcher and generic adapter were not changed by these
repairs. OxFml should retain the native result and avoid imposing an additional
padding-priority rule or a unary coercion interpretation.

ROUND also changes UnaryNumericScalarOnly to Custom to describe its exercised
two-argument preparation. ROUNDUP, ROUNDDOWN and TRUNC retain Custom. Their
function-local preparation preserves positional errors and converts explicit
missing numeric arguments to zero; TRUNC's absent second argument defaults to
zero. These changes do not alter the generic numeric adapter. OxFml must retain
Missing and the original argument positions until the function policy applies.

The former financial adapter matched 15/75 discovery cases; the refined source
matches all 75 and a fresh 584-case bank (SLN 83, SYD 124, DB 125, DDB 125,
VDB 127). PERMUTATIONA matched 69/85 typed discovery cases before its repair;
the current source matches all 85 and a fresh 252-case bank. Strict retained
replays include unequal arrays, unit arrays, omitted arguments and references.
These are function preparation observations, not evidence of expression
scheduling or receiving evaluator integration.

Rounding preparation matches 690 admitted discovery observations and a fresh
928-case independent bank. The rejected `TRUNC()` spelling in the original
691-case request supplies no worksheet observation and is withheld. The four
rounding functions retain separate initial-precision and raw numeric-payload
residuals; the latter are documented in HO-FN-029.

The reference is Excel 16.0 build 20430, 64-bit, Compatibility Version 2,
1900 date system, with precision-as-displayed disabled; channel unverified.
Shared contextual numeric-text parsing remains the HO-FN-022 dependency.
Switched VDB and other family-specific numerical residuals remain open.

`scope_completeness=scope_partial`, `target_completeness=target_partial`,
`integration_completeness=partial`. Open lanes: contextual coercion, retained
numeric residuals, receiving acknowledgment and exercised evaluator integration.

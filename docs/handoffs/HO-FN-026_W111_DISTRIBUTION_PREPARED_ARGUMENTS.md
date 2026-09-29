# HO-FN-026: distribution prepared arguments and positional padding

Direction: OxFunc → OxFml. Status: `filed`, acknowledgment pending.
Owner: W111, `oxf-mwue.28.14`; receiving dependency `oxf-mwue.28.14.1`
(`BLK-W111-DISTRIBUTION-SEAM`).

The [source impact assessment](../function-lane/evidence/w111-broad-20260929/distribution-surface/IMPACT_ASSESSMENT.md)
names all 26 observed surfaces. Their prepared-value rules distinguish explicit
missing numeric arguments, blank cells, strict cumulative logical text, and the
optional fourth BINOM.DIST.RANGE argument. Explicit omission of that fourth
argument selects its default; a blank or zero remains a present zero.

Elementwise broadcasting must preserve an absent coordinate as an NA error at
its original argument position. Earlier argument coercion errors take precedence
over later missing coordinates. The first independent packet exposed 27 failures
in a candidate that discarded this ordering; the refined local distribution
helper matches that 1,296-case bank and a fresh 912-case bank.

OxFml should preserve origin, missing versus blank, argument position and array
shape until the function-specific preparation runs. It should not replace the
cumulative parameter's grammar with generic numeric-text coercion. Verify alias
behavior and the interaction between native lifting and generic LiftAt metadata,
including one-cell arrays. Fresh one-cell-array controls exposed 30 structural
differences on ten legacy aliases: CONFIDENCE, NORMINV, LOGNORMDIST, BINOMDIST,
CRITBINOM, POISSON, HYPGEOMDIST, NEGBINOMDIST, NORMDIST and EXPONDIST. Their
declarations now use SurfaceNative. The subsequent independent 768-case bank
has 552 exact results and 216 retained numerical discrepancies, with no shape
or error-class discrepancy. OxFml must honor these corrected declarations.
No new resolver API is proposed.

Local prepared-value replays do not establish expression scheduling or laziness.
Normal CDF and several discrete probability kernels have retained numerical
residuals; those do not disappear when coercion and shape observations agree.
The reference is Excel 16.0 build 20430, 64-bit, Compatibility Version 2, with
channel unverified and locale recorded in the retained capture manifests.

`scope_completeness=scope_partial`, `target_completeness=target_partial`,
`integration_completeness=partial`. Open lanes: remaining numerical/domain
refinements, receiving acknowledgment and
exercised evaluator integration. Filing this packet opens the dependency.

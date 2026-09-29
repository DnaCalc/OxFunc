# Integer conversion and numerical publication campaign

W111, BUG-FUNC-063/064, bead `oxf-mwue.28.14`. Reference: Excel 16.0 build 20430,
64-bit, workbook Compatibility Version 2; channel unverified. Exact Value2
inputs and exact returned binary64 bits are authoritative. Capture manifests,
generators, source freezes and failed candidates remain distinct.

Initial discovery spans 45,888 numeric and 302 typed observations across 16
functions. Subsequent packets separate numerical publication, integer
conversion, rounding, combinatorics and prepared-value behavior. The broader
distribution and GCD/LCM records have their own evidence directories.

## Independently exercised candidates

- SQRT's extended square-root followed by a binary64 store matches 35,900 fresh
  observations. The ordinary platform square-root differed at rounding midpoints.
- PHI's checked square and publication match 9,130 fresh observations. STANDARDIZE
  matches 6,500; EXPON.DIST and EXPONDIST each match 6,500. Typed prepared values
  and distribution numerical residuals are tracked separately.
- RADIANS and SECH initially failed five and two fresh rows respectively.
  Staged multiplication and reciprocal explain those failures; a second frozen
  independent packet supplies 22,000 exact matches for each function.
- ISEVEN and ISODD use the observed addition-before-floor tolerance and avoid
  signed-integer saturation. Their fresh independent banks match 36,200 and
  36,150 observations respectively.
- INT's first independent packet exposed ten large fractional inputs where
  decimal precision reduction was inappropriate. A second packet exposed two
  negative predecessors of powers of two where addition-before-truncation
  differs from mathematical floor. The refined candidate matches all retained
  banks, totaling 123,229 observations, including 31,210 fresh inputs generated
  after the latest freeze. Older failed candidates remain retained.

These are admitted numerical observations, not whole-function promotions.
The first NORM.DIST/NORM.S.DIST heldout exposed density operation ordering and
335 CDF residuals per function. The density refinement and prepared arguments
are recorded in `../distribution-surface/`; CDF arithmetic remains partial.

## Open numerical refinements

TRUNC matches ROUNDDOWN in both the 3,312-row initial refinement and 7,344-row
digit-boundary discovery. The first shared rounding candidate failed substantial
parts of a fresh 12,000-row bank per function. Those 48,000 comparisons are
retained in `later-heldout/` and `validation/`; subsequent digit-conversion and
decimal-assembly research must retain that failed freeze. Current rounding
refinement is owned by the rounding family, including endpoint behavior and
exceptional initial decimal precision.

PERMUTATIONA discovery distinguishes raw negative-base rejection, exponent
truncation, the upper admission bound, staged repeated squaring, and a separate
base-ten decimal-power path. A fresh 25,751-row packet found two differences at
10^126, rejecting ordinary correctly-rounded decimal parsing. The existing
independently evidenced staged decimal-scale primitive resolves both; a second
fresh bank matches all 25,321 numeric observations. Its typed bank initially
matched 69/85, exposing explicit missing and positional padding rules. The
refined surface matches all 85 and a fresh 252-case bank. Strict retained replay
passes both banks, including reference subranges. The first typed-test fixture
resolver failed a contained-subrange request; correcting that fixture, without
changing production, reproduces the already passing live local runner.
Numerical and prepared bindings are in `PermutationPower.lean` and
`Functions/PermutationAFn.lean`; the latter's focused build passes ten jobs.
HO-FN-028 records the Custom coercion declaration and receiving dependency.

`IntegerPreparation.lean` models INT and the parity predicates;
`NumericPublication.lean` binds the publication and staged-operation rules.
The numerical primitives are explicit empirical bindings, not universal proofs
of Excel identity. Focused model and public-dispatch validation logs are retained
or linked with each packet. Shared parser and generic formatter residuals remain
separate and must not be erased by these family results.

`scope_completeness=scope_partial`, `target_completeness=target_partial`,
`integration_completeness=partial`. Open lanes: rounding and probability
residuals; typed/contextual preparation; primitive
alignment; receiving dependencies HO-FN-022/026/027/028/029; combined repository checks
and canonical ledger reconciliation. `artifact-manifest.json` records file hashes.

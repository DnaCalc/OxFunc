# Distribution preparation, density and discrete domains

`scope_completeness=scope_partial`, `target_completeness=target_partial`,
`integration_completeness=partial`.

Open lanes: normal CDF and other interior
distribution arithmetic, evaluator integration of prepared coercion rules, and
broader function contexts. No whole-function parity claim is made.

The observed surfaces are NORM.S.DIST, NORM.DIST, NORMDIST, EXPON.DIST and
EXPONDIST. Public Excel COM captures carry version/compatibility provenance in
their retained manifests. Numeric parameters admit explicit missing as zero.
Cumulative parameters accept numbers, logical values, blank/missing false, and
untrimmed ASCII-case-insensitive TRUE/FALSE text. Numeric-looking cumulative text
returns VALUE. Arrays broadcast singleton dimensions and preserve per-cell errors.

| Capture | Rows | Current exact outcomes | Open numeric | Input limitations |
|---|---:|---:|---:|---:|
| Discovery | 132 | 129 | 3 | 0 |
| Surface controls | 1,083 | 1,064 | 3 | 16 |
| First independent | 1,296 | 1,296 | 0 | 0 |
| Independent padding refinement | 912 | 912 | 0 | 0 |
| Adjacent 21 surfaces | 1,620 | 1,493 | 121 | 6 |
| Further prepared holdout | 2,541 | 2,261 | 280 | 0 |
| Unit-array padding discovery | 132 | 108 | 24 | 0 |
| Independent unit-array padding | 768 | 552 | 216 | 0 |

The 16 input limitations are invalid generator fixtures containing syntactic
missing arguments as stored cell values; neither oracle nor local harness ran
those functions. `surface-input-limits.json` lists every row. The captured
generator is retained before its fixture rule was corrected. The adjacent six
limitations are unary zero-argument formulas that Excel refused to enter; they
do not justify a unary missing-argument rule. The 647 numerical differences
remain pinned as discrepancies, not counted as matches.

`candidate-freeze.json` records the first prepared-value candidate before the
independent capture. It matched 1,269/1,296; the 27 failures and raw local results
remain in `heldout-frozen-*`. Earlier Ref/Div0/Num errors were incorrectly replaced
by later missing-coordinate NA errors. The refined distribution-only helper
materializes NA at the missing argument coordinate before scalar coercion. All
1,296 outcomes then agree, including the converse order where earlier NA wins.
`padding-candidate-freeze.json` precedes the fresh 912-case validation packet;
all 912 outcomes agree without another source change.

`w111_distribution_surface_live_replay` runs the retained cases through public
dispatch and checks exact numeric bits, error identities, logical values, and
array shapes. Eight replay tests exercise 7,815 exact observations, 647 explicitly
open numeric observations, and 22 explicitly excluded input-limited rows. Focused
Lean NormalLogFamily/DiscreteDistFamily builds pass (11 jobs); the executable
DistributionArguments model binds missing handling, logical grammar, ordered
coercion, and padding represented as an argument error.

The separate numeric graph research preserves all tested candidates. The former
NORM.DIST PDF path matched 5,330/5,426 retained numeric observations. A general
candidate that divides the exponential by sigma before multiplying the normal
constant, with RN64 operations and binary64 publication, matches all 5,426.
It also matches all 6,586 independent density observations, where the old
production graph matches 5,064. A further 128 mathematical discriminators selected
from 708,359 fresh draws, without using Excel answers, distinguish staged division
(128 matches) from ordinary binary64 division (zero matches). Another 144
near-half-ULP controls distinguish staged subtraction (144 matches) from ordinary
binary64 subtraction (130 matches). The resulting PDF-only graph is now bound in
production and Lean and agrees on all 12,284 retained numeric observations. NORM.S.DIST
PDF matches all 4,803. Each family's CDF capture retains 335 differences; no
CDF arithmetic was changed in that density refinement. The later CDF wrapper
correction is described below. The additional 30 NEGBINOM.DIST controls include the
older exact test `(5,3,0.4,TRUE)`: Excel reproduces expected bits
`0x3fe5e849aaeed68d`; the local result is one ULP larger. Its regression
expectation remains unchanged and that separate kernel discrepancy stays open.

See `IMPACT_ASSESSMENT.md` for the evaluator-facing correction and explicit
prepared-values limit. Canonical contracts and handoffs belong to the campaign
owner.

The adjacent preparation sweep names all 21 additional surfaces in the impact
assessment. Each had its own coercion and shape controls. It exposed one optional
argument exception: BINOM.DIST.RANGE's explicit missing fourth argument selects
the omitted upper-bound default; a blank or zero remains a present zero value.
Original and intermediate local results are retained, including the 94 domain
errors exposed after the preparation repair.

Dedicated discrete controls establish strict open probability intervals for
BINOM.INV (both probability and alpha) and NEGBINOM.DIST (probability), and
separate invalid hypergeometric populations from outcomes outside valid support.
Above support, HYPGEOM PDF returns 0 and CDF returns 1; below support both return 0.
The 3,009-row discovery agrees on every error classification. Exact interior
numerics remain partial: BINOM.INV 402/405, NEGBINOM 234/288, HYPGEOM 2,185/2,316.
The numeric replay target has seven passing tests covering density, discrete, and CDF
captures, preserving every retained numerical discrepancy as a mismatch. Existing NEGBINOM exact regression remains a
known failing test; its expectation is not rewritten to the local result.

`production-freeze.json` records the three production source hashes before the
fresh 6,810-density/501-binomial-inverse/181-negative-binomial/1,001-hypergeometric
packet and 2,541 prepared cases were generated. These fresh packets include both
new inputs and stable controls. Density matches all 6,810, bringing its retained
total to 19,094 exact observations; the fresh inputs include 5,724 previously
unseen argument tuples and 1,086 stable controls. BINOM.INV matches 501/501;
NEGBINOM matches 115/181 with 66 numerical differences. HYPGEOM originally
matched 600/1,001: 368 failures came from accepting raw negative fractions after
truncation. A raw nonnegative guard yields 968/1,001 with 33 numerical differences.
The subsequent independent 480 cases match 446, with 34 numerical differences
and no domain mismatches. `hypergeom-raw-freeze.json` precedes those 480 cases;
later metadata and POISSON edits change the file hash without changing the
hypergeometric arithmetic. Old source freezes, rejected candidates, all
mismatches and raw Excel provenance remain available in this directory.

The 2,541 prepared holdout originally matched 2,255: 280 numerical differences
and six POISSON zero-count branch errors remain in `prepared-heldout-frozen-*`.
Dedicated POISSON discovery (312 cases) distinguishes three steps: reject raw
negative count, truncate an admitted nonnegative count, and return EXP(-mean)
for count zero before rejecting a negative mean. This applies to both cumulative
flags. Overflow returns NUM; observed subnormal exponential results are preserved
(e.g. count 0, mean 710 gives `0x00033802fd28b3c3`). The refined discovery agrees
on 300/312 with 12 existing positive-count numerical differences. It also repairs
all six prepared branch errors; the 280 numerical differences remain explicit.
The Lean model binds branch order with supplied exponential/publication behavior.

Unit-array controls exposed 30 metadata-induced structural failures in ten legacy
aliases: a second generic by-index lift overwrote native positional error order.
Those ten aliases now declare the existing SurfaceNative profile. Unary
NORMSDIST/NORMSINV and the generic dispatcher remain unchanged. Current discovery
results are 108/132 exact plus 24 numerical differences, with no structural
differences. Three previously structural HYPGEOM cases also had numerical
differences, so the remaining numerical count grows from 21 to 24 when shapes and
errors are repaired. `unit-branch-candidate-freeze.json` retains source copies
before fresh unit-array 768 and POISSON 744 follow-ups were generated; their
unchanged candidate gives 552/768 exact with 216 numerical differences and no
structural differences. The POISSON packet gives 711/744 exact with 33 numerical
differences, all at positive counts. Every zero-count and error-class outcome
agrees. The three frozen production hashes were checked before these replays.
Positive-count underflow publication remains among the numerical residuals;
it is not inferred from the distinct zero-count exponential branch. The metadata
golden comparison passes all ten aliases; its separately owned TANH policy row
was reported to the campaign owner.

The subsequent normal-CDF dependency work is retained in `cdf-research/`. Public
ERFC composition distinguishes RN64 multiplication followed by a binary64 store
for the wrapper's z value: fresh NORM.S.DIST and ordinary GAUSS each agree on
512/512 with that graph; native multiplication agrees on 379/512 and 470/512.
That single wrapper operation is corrected. The ERFC backend remains unchanged
and inaccurate: the full 9,606 NORM.S.DIST observations retain 335 mismatches,
and the targeted 512-case packets retain 236 NORM.S.DIST and 60 GAUSS local
backend mismatches. A fifth numeric replay test pins these failures explicitly.
The independent public-dependency identity is not a claim that the local full
functions are bit-exact.

`poisson-publication/` separately records the positive-count final-probability
correction: subnormal results publish as +0 after the positive-count backend,
while the zero-count early exponential preserves its subnormal outputs. The
1,428-case discovery improves from 404 to 959 exact outcomes, retaining 469
backend discrepancies. A frozen independent 1,100-case packet gives 734 exact
outcomes and 366 backend discrepancies; 1,040 inputs are new and 60 are repeated
branch controls. The earlier 744-case packet improves from 711 to 727 exact,
with 17 numerical discrepancies. Its old replay is retained. Current numeric
tests cover 38,510 observations: 36,672 exact and 1,838 explicitly open numerical
outcomes. Seven numeric and eight typed tests pass; focused Lean builds pass.

# Distribution prepared argument and domain corrections

Status: `scope_completeness=scope_partial`, `target_completeness=target_partial`,
`integration_completeness=partial`. Open lanes: normal CDF and discrete interior arithmetic residuals, broader
function contexts, and evaluator integration of corrected prepared coercion rules.

The existing numeric-publication capture supplies 132 prepared argument cases
for NORM.S.DIST, NORM.DIST, EXPON.DIST and EXPONDIST. The original local outcomes
are preserved separately from the oracle. Examples through public Excel formulas:

- `NORM.S.DIST(,FALSE)` evaluates density at zero; explicit missing numeric
  arguments therefore need the same zero value as a blank cell in these surfaces.
- `NORM.S.DIST(-2.125,"3.25")` returns VALUE; the cumulative parameter does not
  use the generic numeric-text parser even though numeric cumulative values are
  admitted.
- Modern NORM/EXPON names return per-element arrays for direct arrays and ranges;
  their original local scalar prepared evaluators returned a single VALUE error.
  Legacy EXPONDIST already receives evaluator-side lifting, making the gap visible
  between aliases.

The correction is confined to the distribution evaluators evidenced by
this packet: missing numeric arguments become zero; cumulative arguments use
their separately observed logical grammar; and prepared broadcasting maps
per-element errors. A 1,083-case packet adds NORMDIST and exercises
logical spellings, numeric text, all argument origins, error order, unit arrays,
and incompatible shapes before extending the rule beyond the initial capture.
Other normal/lognormal/inverse/binomial/Poisson surfaces in the same source files
are not automatically changed by family membership.

Of that packet, 1,067 are admitted observations and 16 are invalid generator
fixtures that tried to store a syntactic missing argument in a cell. The invalid
rows remain retained and excluded explicitly. The valid observations show three
existing normal CDF differences and agreement on types, coercion, and shapes.
The first independent 1,296-case capture exposed 27 padding precedence failures:
an early argument's present error must win over a later absent coordinate.
The rejected candidate and every failure are retained. The refined distribution
helper represents padding as an #N/A value at its original argument position,
then runs ordinary left-to-right scalar coercion. It agrees on all 1,296 captured
outcomes in public-dispatch tests. A fresh 912-case packet also agrees entirely
with the unchanged refinement.
The generic broadcast adapter and unrelated functions remain unchanged.

The initial modern-surface correction required no metadata change. Modern surfaces already declare
native lifting, and the implementation needs to exercise that declaration.
Prepared coercion is nevertheless an evaluator-facing semantic correction:
OxFml must preserve missing versus blank carriers until this function rule runs,
and must not pre-coerce cumulative numeric text using generic numeric rules.
The campaign owner handles canonical contracts and cross-repository registration.
Local prepared-value tests cannot establish caller evaluation order or laziness.

Existing normal density and CDF numerical differences remain independent and visible.
The scalar arithmetic graphs were preserved during the initial structural work;
the separately observed numeric changes are described below.

## Adjacent prepared evaluators

A separate 1,620-case public COM capture examines 21 additional surfaces before
their preparation rules change: CONFIDENCE/CONFIDENCE.NORM; NORM.INV/NORMINV;
NORM.S.INV/NORMSINV/NORMSDIST; LOGNORM.DIST/LOGNORM.INV/LOGNORMDIST;
BINOM.DIST/BINOMDIST/BINOM.DIST.RANGE/BINOM.INV/CRITBINOM;
POISSON/POISSON.DIST; HYPGEOM.DIST/HYPGEOMDIST; and
NEGBINOM.DIST/NEGBINOMDIST.

Each surface has its own per-parameter direct, reference, array, missing and
error controls. These support the same explicit-missing-as-zero rule for the
multiargument numeric parameters and strict logical-text cumulative rule for
the surfaces with that parameter. The single-argument `FUNC()` formulas fail
at Excel formula entry; their six rows are not function-semantic evidence and
do not justify changing unary missing coercion. Array lifting applies to the
admitted existing argument domains; it does not establish evaluator scheduling.
The optional fourth BINOM.DIST.RANGE argument is an observed exception: explicit
missing selects the same upper-bound default as omission, while a blank cell or
numeric zero remains a present zero upper bound. Its distinct rule is preserved.

Original outcomes remain retained: 1,152 exact, 69 numerical-bit differences,
393 type/shape/error differences, and six formula-entry limitations. Some of
those 393 differences are separate numerical-domain errors, including zero
probability boundaries and impossible hypergeometric support. They are not
silently treated as coercion repairs. The proposed adjacent change only routes
the observed prepared evaluators through positional padding and their observed
numeric/logical conversion rules. Their arithmetic and domain guards remain
unchanged until separately characterized. The later unit-array observations below
justify a separate legacy metadata correction.

The adjacent production replay now agrees on 1,493 of the 1,620 rows, with 121
explicit numerical-bit discrepancies and six formula-entry limitations. Every
admitted type, shape and worksheet-error classification agrees after the
separately evidenced domain corrections below. The original and intermediate
outcomes remain retained. Eight typed replay tests and the focused executable Lean preparation models
pass. The 2,541-case frozen independent prepared packet exposes six separately
characterized POISSON zero-count domain errors and 280 numerical differences.
After the branch correction, all admitted type, shape and error classifications
agree; every numerical discrepancy remains recorded.

## Separately observed numeric changes

NORM.DIST/NORMDIST density now uses RN64 operations with binary64 publication for
mean subtraction, normalization division, square, scale division and constant
multiply. The exponential is divided by sigma before multiplying the normal
constant; only the final density is flushed if subnormal. Four retained cohorts
total 12,284 exact observations. A further frozen 6,810-case packet agrees in
full, bringing density observations to 19,094. CDF arithmetic is unchanged and remains partial.

The discrete domain packet independently covers 405 BINOM.INV, 288 NEGBINOM.DIST
and 2,316 HYPGEOM.DIST cases. BINOM.INV requires probability and alpha strictly
between zero and one; NEGBINOM.DIST requires probability strictly between zero
and one. HYPGEOM accepts valid population parameters with an impossible outcome,
returning the exact outside-support probability (PDF zero; CDF zero below support
or one above support). These are function-kernel rules after prepared coercion;
integer conversion and interior probability arithmetic remain separate. All
error classifications agree on these observations, with 188 interior numerical
discrepancies still explicitly retained. Fresh BINOM.INV 501/501 agrees;
NEGBINOM retains 66/181 numerical discrepancies. HYPGEOM raw negative fractions
are rejected before integer conversion, while positive population comparisons
follow conversion. The fresh 1,001 cases retain 33 numerical differences after
that guard refinement; subsequent independent 480 cases retain 34 numerical
differences with no domain mismatches.

For HO-FN-026 integration, callers should deliver unmodified prepared values,
preserve explicit optional omission until the function applies its default,
and avoid substituting generic numeric-text coercion for cumulative logical
coercion. Function-specific elementwise lifting must retain absent coordinates
at their argument positions so earlier coercion errors win. No resolver API
change is required or claimed. A further
132-case control packet confirms 30 structural failures from legacy lift metadata
with a unit-array argument: generic dispatch re-enters its fallback lifter even
after the native function returns an array, replacing earlier errors with later
padding. The correction sets the ten observed multiargument legacy aliases to
the existing SurfaceNative profile: CONFIDENCE, NORMINV, LOGNORMDIST, NORMDIST,
BINOMDIST, CRITBINOM, POISSON, HYPGEOMDIST, NEGBINOMDIST and EXPONDIST.
Their native prepared evaluator already owns all argument positions and preserves
ordered errors. The generic dispatcher and NORMSDIST/NORMSINV profiles remain
unchanged. This changes an evaluator-facing declaration; HO-FN-026 must carry the
updated ten profiles. Replay now agrees on every structural outcome (108 exact
and 24 numerical discrepancies out of 132). The frozen independent 768 cases
give 552 exact and 216 numerical discrepancies, with no type, shape or error
classification discrepancies. The three production hashes were checked unchanged
before replay.
Receiving-repository acknowledgement and integration remain open.

POISSON/POISSON.DIST also have a separately evidenced kernel branch correction:
raw negative counts are rejected before truncation; an admitted count truncated
to zero returns EXP(-mean) before negative-mean rejection, for both flags. Public
COM examples include `POISSON.DIST(0.5,-1,FALSE)` returning EXP(1), while
`POISSON.DIST(-0.5,1,FALSE)` returns NUM. Exponential overflow maps to NUM;
subnormal zero-count results remain observable. Discovery 312 has 300 exact
outcomes and 12 positive-count numerical discrepancies; the frozen independent
744 cases give 711 exact and 33 positive-count numerical discrepancies. Every
zero-count and error-class outcome agrees. Positive-count underflow publication
is a separate open numerical lane. This kernel rule does not alter evaluator scheduling or the numeric
coercion grammar.

A later numerical wrapper correction replaces native multiplication by RN64
multiplication followed by binary64 publication for the normal-CDF z value.
Fresh public-ERFC composition confirms 512 NORM.S.DIST and 512 ordinary GAUSS
cases; the distinct GAUSS tiny helper and ERFC backend remain unchanged. This
is a kernel arithmetic correction, with no new FEC declaration or evaluator
preparation rule. Full-function numerical discrepancies remain explicit.

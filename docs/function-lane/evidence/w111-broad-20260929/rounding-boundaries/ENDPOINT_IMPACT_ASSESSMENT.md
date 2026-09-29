# Rounding endpoint publication: assessment before further promotion

State: in_progress; scope_completeness=scope_partial;
target_completeness=target_partial; integration_completeness=partial.
Open lanes: the conflicting earlier TRUNC MAX
capture, complete endpoint arithmetic inference, evaluator publication contract,
and acknowledgement of any cross-repository boundary change.

The fresh numeric endpoint capture and a separate 224-row public worksheet/COM
capture agree that ROUND, ROUNDDOWN and TRUNC can produce numeric worksheet
results whose Value2 binary64 encodings fall in the IEEE nonfinite range. For
MAX_NORMAL and count0, Value2 is 0x7ff000000000000a; ROUND with count-308 gives
0x7ff1ccf385ebc8a0. Their signed counterparts preserve the sign bit. Excel's cell
Text displays numerical strings (including 2E+308), ISNUMBER is TRUE, ISERROR is
FALSE, TYPE returns1, IFERROR preserves the value, and self-equality is TRUE.
Adding zero produces NUM. Both nested and cell-reference ISNUMBER/ISERROR
controls agree. Raw observations are retained without JSON numeric coercion.

These observations do not justify interpreting the payload as an ordinary IEEE
NaN for every downstream operation, nor as a worksheet error. A function-local
finite-to-NUM guard loses observable result kind and bits. Conversely, merely
passing an f64 NaN to generic arithmetic/comparison/rendering would not reproduce
the observed numerical Text or self-equality behavior. The existing
CalcValue::Number(f64) can carry the bits, but the evaluator and consumers must
agree on admission, propagation, comparison and display policy before an
integration claim. No global value/coercion policy has been changed here.

Directed rounding at the lower endpoint also returns subnormal numeric values
from admitted normal inputs: minimum normal becomes 0x000ffffffffffffa in the
observed exceptional normalization lane. This survives Value2 and addition of
zero, is classified as Number, and displays0. It is distinct from the already
measured Value2 ingress behavior that transforms source subnormals to zero.

The older broad TRUNC capture disagrees specifically on MAX input: it records a
lower finite decimal where the fresh endpoint and worksheet controls record a
payload number. Both report Excel16.0 build20430 and CV2. Their bulk-runner file
timestamps differ, but the retained source diff adds only an arity-zero guard;
that change does not explain a two-argument TRUNC difference. The old result
must remain explicit counterevidence. A two-process repeat compares General
and scientific cell formats, Calculate and CalculateFull, Formula2 and relative
Formula2R1C1, exact input readback, and outcomes before/after dependent formulas.
All 960 observations in two distinct Excel processes agree on numeric kind and
result bits, with exact input readback throughout. None of those tested settings
explains the historical discrepancy. This narrows possible explanations without
discarding the earlier counterevidence or claiming a cause that was not observed.

A uniform offline arithmetic model reproduces all 14,224 fresh endpoint rows:
the existing RN64 nibble scaling and final RN53 yield a normalized mantissa and
exponent; at the exceptional initial directed lower endpoint, constructing the
fraction with exponent -1023 returns the observed raw subnormal before the count
is used. At overflow, the same raw exponent/fraction construction yields the
observed nonfinite-range encodings. Ordinary reduced subnormal decimal pairs
still publish zero. This is behavioral inference, not an internal-code claim.
Only the finite initial directed subnormal policy has been added to production.
The parser and PERMUTATIONA retain their existing publication policy; all 572
fresh payload-number observations remain explicit unresolved production rows.

The 2026-03-05 floating-point records are scoped primarily to UDF ingress and
selected invalid/tiny formulas on build19725. They do not establish a universal
finite-publication rule for these newly observed built-in endpoint lanes.
Any record update must distinguish those earlier exercised boundaries instead
of retroactively replacing their observations.

Requested evaluator review, after reproduction: specify how function result
bits and numeric kind are preserved, whether an extended numeric publication
carrier is required, and which subsequent operations map these values to NUM.
The parent owns the actual handoff, registry and dependency record. Filing such
a packet opens a dependency and supplies no acknowledgement by itself.

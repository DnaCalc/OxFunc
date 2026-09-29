# Rejected SINH arithmetic hypotheses

State: in_progress; scope_completeness=scope_partial;
target_completeness=target_partial; integration_completeness=partial.
Open lanes: small-input SINH arithmetic and dependent TANH/COTH publication.

These are negative research results; no production correction follows.
The source examples enumerate mixed binary64/extended multiplication and
division stores, exponential-return precision, asymmetric half lifetimes and
cancellation-safe algebraic identities. No candidate matches both the older
small-input bank and the newly targeted dependency bank. Matching x directly
on most new tiny inputs is also refuted by older observations.

The first lifetimes output predates an edit to its example. Its original source
was not retained, so that output is historical and not a replayable candidate.
The current lifetimes source corresponds to the mixed-lifetimes output. The
remaining three source/output pairs are retained together. This limitation
prevents using the first output as conformance evidence.

Public dependency capture contains EXP 388, EXPON.DIST 194, GAMMA.DIST 194,
and a later five-row LN control batch. The original request manifest predates
the LN addition; the artifact manifest here hashes every later request and
answer as well. Reconstructing the negative expm1 half from the captured EXP/LN
values matches the existing Kahan graph on 194/194 values. GAMMA.DIST and
EXPON.DIST match each other on 194/194. Those controls do not establish that
SINH uses the same private half graph. All observations are from public Excel
interfaces and preserve exact bits, host profile and capture metadata.

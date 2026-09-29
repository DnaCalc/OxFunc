# ROUNDUP signed residual rule

State: in_progress; scope_completeness=scope_partial;
target_completeness=target_partial; integration_completeness=partial.
Open lanes: initial15 precision,
raw overflow numeric publication, contextual numeric-text preparation and
receiving acknowledgement of the prepared-value handoff.

The frozen 6,780-row ROUNDUP candidate failed one new finite observation:
source bits 0x0210be08d0527f0a and count 305. Excel returned 1e-298; the candidate
added 1e-305. The source normalizes to 1.00000000000005e-298 before truncation,
leaving a very small positive difference from 1e-298.

The subsequent 3,666-row paired-sign discovery found 33 differences, all on
positive inputs. Simply flushing a subtraction below minimum normal was
rejected: using the normalized value introduced 275 errors and using the raw
source introduced 251. Those failed hypotheses are retained in
tiny-difference-models.json; no production code used either rule.

A further 4,604-row binary/decimal residual ladder varied base magnitude,
requested unit, sign and adjacent input encodings. Positive suppression changes
at 2^-1026, while negative residuals remain distinguishable. Source-adjacent
controls establish that the subtraction starts from the assembled initial15
value, rather than the original input. A uniform representation of the observed
predicate is to subtract the signed down result from the signed initial15 value
in binary64 and inspect the result's upper16 bits. A zero upper16 suppresses the
unit. Negative residuals retain the sign bit and do not meet that test. An exact
zero decimal remainder continues to suppress the unit independently.

This is a behavioral model inferred from public numerical observations, not a
claim about Excel's source code, processor instructions or internal layout.
It uses no per-input table, fitted coefficient or exceptional source lookup.
The single predicate agrees with all 38,896 retained ROUNDUP observations in
tiny-difference-top16-model.json, including the formerly failed independent row.
Those captures now provide refinement evidence. The new independent packet is
w111-roundup-residual-heldout-20260929: 6,542 ROUNDUP and 3,470 ROUNDDOWN controls,
with seed 202609293035 and immutable source snapshots before capture.
The packet has now been captured without cache and replayed through production:
all 6,542 ROUNDUP and 3,470 ROUNDDOWN observations agree bit for bit. Frozen
source hashes remained unchanged through replay. The result and its limits are
recorded in signed-residual-validation.json; it does not remove other open lanes.

The Rust candidate uses the existing staged decimal assembler and a local
signed subtraction; no parser arithmetic or generic numerical publication
policy changed. DecimalRounding.lean mirrors the predicate and includes the
positive/negative formerly failing pair. Nine numerical integration tests and
two prepared integration tests pass; the separate new independent replay test
also passes all 10,012 observations. The targeted Lean build passes 13 jobs.

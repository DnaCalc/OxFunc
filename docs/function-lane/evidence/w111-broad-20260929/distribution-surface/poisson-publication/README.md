# POISSON positive-count probability publication

Status: `scope_completeness=scope_partial`, `target_completeness=target_partial`,
`integration_completeness=partial`.

Open lanes: positive-count PDF/CDF arithmetic, premature CDF backend underflow,
other uncharacterized domains, and the separately tracked distribution evaluator
integration. This evidence supports a function-local publication correction;
it does not establish whole-function bit identity.

Public Excel 16.0 build 20430, workbook compatibility version 2, was captured
through bulk cell Value2 with no oracle cache. Each answer file retains full
capture provenance. Inputs were chosen from mathematical PDF/CDF minimum-normal
boundaries and seeded controls; Excel outputs were not used for input selection.

| Cohort | Observations | Exact current outcomes | Open numeric discrepancies |
|---|---:|---:|---:|
| Discovery | 1,428 | 959 | 469 |
| Frozen independent | 1,100 | 734 | 366 |

The independent cohort uses new positive counts 4, 11, 29, and 89, versus
discovery counts 1, 2, 3, 7, 19, and 53. It has 1,040 new argument tuples and
60 repeated branch controls. All four hashes in `candidate-freeze.json` were
unchanged before replay. These are observation counts, not globally unique
inputs across the campaign.

After the raw-negative guard and zero-count early exponential branch, the
positive-count result now publishes magnitudes below binary64 minimum normal
as positive zero. Results at or above that boundary remain unchanged. This
is a general final-result rule; neither count-specific cutoffs nor fitted
numerical corrections were introduced. The zero-count exponential still
preserves its observed subnormal results.

Discovery contains 628 positive-count normal outputs and 740 positive-count
zeros. The independent bank contains 533 positive-count normal outputs and
507 positive-count zeros. Neither has a positive-count subnormal Excel output.
Each bank also has 18 zero-count subnormal outputs, 12 zero-count normal
outputs, six zero-count zero outputs, and 24 raw-negative count errors, all
matching the retained candidate.

Before this correction discovery matched 404/1,428. The publication change
repairs 555 outcomes without changing any prior match. The earlier independent
744-row branch packet improves from 711 exact / 33 numeric discrepancies to
727 exact / 17 discrepancies. Its old replay remains in the parent directory;
the new replay is separate here. The earlier 312-row branch packet remains
300 exact / 12 discrepancies.

The remaining discrepancies must not be treated as publication agreement.
Discovery includes 136 cases where the current CDF backend already returns
zero but Excel returns a normal probability; the independent bank has 159
such cases. The other 333 discovery and 207 independent differences remain
ordinary numerical backend discrepancies. No backend arithmetic changed.

The numeric Rust replay binds both packets and preserves every residual's
Excel expectation and current local result. `DiscreteDistFamily.lean` models
positive-count publication with a supplied minimum-normal boundary and
backend, and proves the zero-count branch bypasses that publication. This
does not assert correctness of the supplied interior backend.

Retained files include discovery inputs/answers, baseline and current replays,
the pre-correction source, the frozen candidate source/hashes, the generator,
and the independent inputs/answers/replay. No Excel binary inspection or
proprietary source was used.

The subsequent `research/` comparison isolates an inherited conservative
GRATIO cutoff: the existing large-shape path exits when `a*rlog(x/a)>=700`,
discarding some still-normal probabilities. A reproducible research-only
variant disables that one exit, without changing production. It removes
134/136 discovery and 159/159 independent zero-versus-normal errors, but its
exact positive-count CDF scores are only 243/684 discovery (unchanged) and
156/520 independent (versus 154). Remaining normal numerical errors reach
about 1.6e-13 relative in the independent bank. Two discovery probabilities
still cross the normal boundary incorrectly because of backend arithmetic.
This diagnoses an early cutoff; it does not validate a replacement gamma
kernel. The variant, generator provenance, and both comparisons are retained.

# HARMEAN, DEVSQ and MROUND follow-up

State: in_progress; scope_completeness=scope_partial;
target_completeness=target_partial; integration_completeness=partial.
Owner: W111, BUG-FUNC-070, oxf-mwue.28.20.

These observations use public Excel Value2 input and output, normal finite
inputs or positive zero, Excel 16.0 build 20430, Compatibility Version 2,
1900 date system, precision-as-displayed disabled; channel unverified.

| Function | Baseline discovery | Refined discovery | Fresh frozen candidate |
|---|---:|---:|---:|
| HARMEAN | 7,333/9,004 | 9,004/9,004 | 9,000/9,000 |
| DEVSQ | 6,891/10,951 | 10,951/10,951 | 10,935/10,935 |
| MROUND | 16,007/16,018 | Research continues | Not yet captured |

HARMEAN uses reciprocal(mean(reciprocals)), with staged reciprocal, sum and
division. Reassociating it to count/sum changes output bits. A nonfinite
reciprocal sum publishes NUM before taking the outer reciprocal. DEVSQ retains
the existing ordered binary64 sum, mean and deviations; each squared deviation
uses staged multiplication and publishes tiny magnitude as zero before joining
the sum. A nonfinite result publishes NUM. These rules explain both the older
broad-sweep differences and the much wider publication bank. Source hashes were
frozen before generating the independent HARMEAN and DEVSQ inputs, and remain
unchanged through judging.

MROUND currently has three unresolved discovery halfway cases after staged
division/product, positive zero and finite-result research candidates. Neither
reciprocal multiplication nor shifted-numerator alternatives match all cases.
The generated MROUND heldout request remains uncaptured and is not validation
of those candidates. Its candidate source copy is the earlier baseline.

AggregatePublication.lean binds the numeric HARMEAN and DEVSQ graphs after
aggregate preparation and exercises overflow and tiny-term publication.
Primitive identity remains empirical. Eight focused Lean build jobs pass.
The source examples and output stages preserve the rejected arithmetic choices;
earlier output versions are identified by their smaller candidate arrays.

Open lanes: MROUND arithmetic and independent validation, prepared arguments
and referenced values, shared coercion, primitive/platform alignment, broader
version phases and evaluator integration. No whole-function promotion follows.

# HARMEAN and DEVSQ prepared arguments

State: in_progress; scope_completeness=scope_partial;
target_completeness=target_partial; integration_completeness=partial.
Root owns BUG-FUNC-070 and HO-FN-030 registration.

| Function | Initial discovery | Revised discovery | Fresh frozen typed |
|---|---:|---:|---:|
| HARMEAN | 109/124 | 124/124 | 283/283 |
| DEVSQ | 113/124 | 124/124 | 283/283 |

Every row executed on both sides. Strict typed digests preserve exact bits,
error identity and array shape; no output was normalized. Independent inputs
were generated after the source freeze and the frozen hashes remained unchanged
through their first judgement. They use independent positive numeric anchors,
three argument positions, permutations of domain/coercion/explicit errors,
direct row/column arrays and separately declared reference cells.

Both functions ignore Empty cells and admit direct explicit Missing as zero.
HARMEAN collects/coerces the sequence before rejecting nonpositive numbers, so
a later invalid text or worksheet error remains visible. Direct scalar coercion
errors are checked before array/reference errors, while numeric collection stays
in its original order, and array/reference text and logical values keep their existing ignored
policy. No-value outcomes remain HARMEAN NA and DEVSQ NUM. No generic aggregate
helper, function declaration or metadata changed.

The arithmetic remains the separately frozen numeric graph: all 18,004 HARMEAN
and 21,886 DEVSQ numerical observations still match exactly (39,890 total).
Four focused public-dispatch Rust tests pass. AggregatePublication's existing
numeric bindings remain unchanged; the new executable prepared layer names
Empty/Missing, origin policy, ordered error collection and no-value outcomes.
The focused eight-job build succeeded, but its command used wrong-case
HarmeanFn/DevsqFn aliases on Windows. The parent full build exposed generated
module-name collisions. Canonical identities are HarMeanFn and DevSqFn; the
parent reports the canonical526-job full build now passes after removing only
colliding generated artifacts. Source files are unchanged. The earlier focused result is not whole-project link
validation; the correction and final canonical result are recorded separately.

A later separate cross-origin discovery exposed16/225 failures per family:
later direct scalar errors or invalid text outrank earlier array/reference
errors. The approved local precheck now matches225/225 per family, and the
separate675-row shared MEDIAN/HARMEAN/DEVSQ packet matches675/675. This follow-up
does not reinterpret the earlier814 successful observations as exhaustive.
The revised frozen prepared layer has a direct-only validation pass followed by
ordered collection. A fresh shared2,190-case origin-order bank (730 per family)
has a qualified serialized repeat:2,190/2,190 exact,730/730 per family. Only
w111-aggregate-origin-serial-typed-20260929 is counted; the first overlapping
capture is retained as provisional. Coverage's strict comparison finds no full
outcome differences and all11 frozen source/model/helper hashes unchanged.
The total qualified prepared observations here are1,362 per family
(124 discovery +283 first independent +225 origin discovery +730 serial
independent), or2,724 for HARMEAN and DEVSQ together. The unchanged numerical39,890 bank still matches.

[The impact assessment](IMPACT_ASSESSMENT.md) describes evaluator-facing changes.
The observations support the local prepared repair; they do not imply receiving
acknowledgement or whole-function completion. Open lanes include shared
locale-sensitive numeric-text context, HO-FN-030 receiving integration and
alternate baseline/platform validation. Current Excel baseline: 16.0 build 20430,
Compatibility Version 2, 1900 date system, precision-as-displayed disabled;
channel unverified. Full capture provenance is retained with the run manifests.

`artifact-manifest.json` maps retained lossless cases/outcomes and judgments to
source and retained SHA-256 hashes. Original failing local outcomes remain
retained beside revised and independent outcomes. Source freezes preserve the
exact Rust bodies and formal bindings. No session transcript is included.

# Text defaults, slicing and array publication: pre-edit assessment

State: in_progress; scope_completeness=scope_partial;
target_completeness=target_partial; integration_completeness=partial.
Open lanes: integer count/start conversion and width, independent validation,
general number-to-text formatting, compatibility/locale context, and evaluator
declarations and handoff acknowledgement.

The 1,952-row typed discovery capture has 1,945 executed results and seven
empty-call formulas rejected at entry. Of those executions, 599 differ from
the current local dispatcher. The 2,772-row raw UTF16 capture has 2,508 exact
ingress rows; 264 NUL-containing input rows changed at Value2 ingress and must
remain withheld. Every raw unit is retained numerically, including isolated
surrogates, so neither JSON encoding nor lossy text conversion is an authority.

The typed controls separate absent LEFT/RIGHT/LEFTB/RIGHTB counts (one) from an
explicitly omitted count (zero). MID/MIDB also prepare omitted numeric slots
as zero before domain validation. All six slicing functions reject negative
raw counts, including fractions between minus one and zero. Positive neighbors
and large MID starts expose an integer-conversion lane under further probing.

The current Compatibility Version 2 baseline treats valid surrogate pairs as
one character in LEFT, RIGHT, LEFTB, RIGHTB and MID. MIDB slices raw UTF16 units;
LEN counts decoded units while LENB counts raw units. Isolated surrogates are
single positions and must survive slicing unchanged. The aliases therefore
need an explicit MIDB path; changing MID alone would otherwise silently change
MIDB behavior. Non-DBCS locale evidence does not establish DBCS behavior.

The planned array repair reuses the existing values-only elementwise broadcast
helper locally for these six slicers, LEN, LENB, EXACT and ENCODEURL. The helper
already admits text, number, logical and per-cell error results and pads absent
coordinates with #N/A. No generic array preparation or scalar coercion policy
change is proposed. Every broadcast rule will be checked against retained
multiple-array and reference-array controls before freezing a candidate.

These changes affect evaluator-visible coercion/lift declarations. Several
existing text metadata entries declare no lift despite array support; ENCODEURL
also has a pre-existing surface dependency declaration requiring review. This
candidate leaves shared metadata unchanged for the parent campaign's cross-repo
assessment. The parent owns handoff registration and acknowledgement. General
numeric textification remains the existing shared policy and is not established
by these simple-number controls. No whole-function parity claim follows.

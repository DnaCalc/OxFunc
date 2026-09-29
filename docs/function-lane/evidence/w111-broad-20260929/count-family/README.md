# COUNT, COUNTA and COUNTBLANK origin rules

State: in_progress; scope_completeness=scope_partial;
target_completeness=target_partial; integration_completeness=partial.
Open lanes: COUNT locale-dependent numeric-text context; COUNTBLANK declared
arity and Empty/Missing scalar-carrier admission; evaluator reference-returning
expression admission and handoff acknowledgement. Canonical records remain owned by the parent campaign.

The retained production-dispatch replay has 2,238 admitted live observations:
458 origin discovery rows, 57 computed-value controls, and 1,647 independent
heldout rows, plus 76 fresh value/reference-origin controls. Five count-filtered replay tests and 18 existing count unit tests
pass. The Lean Count, CountA and CountBlankFn modules build successfully.
`repair-validation.json` records the checked source hashes. The current Excel
profile is version 16.0 build 20430, 64-bit, workbook Compatibility Version 2.

COUNT ignores error values in scalar, computed-array and reference origins;
each explicitly omitted argument counts as one. COUNTA also counts explicitly
omitted arguments, while a blank referenced cell contributes zero. These local
classification helpers are used only by their respective functions; no other
aggregate fold or shared parser policy changed.

COUNTBLANK scans reference cells for empty values or empty text and ignores
reference error cells. An admitted computed scalar Number/Text/Logical value
returns #VALUE!, while an error value preserves its code. A computed array
applies this rule at each position. All seven worksheet error codes occur in
the discovery controls. The candidate preserves the existing uncharacterized
Empty/Missing scalar-carrier and arity declarations.

The 1,664-row independent packet was generated after the candidate source
freeze. Its 17 differences expose a packet origin error: the generator wrote
`COUNTBLANK(IF(FALSE,A100,A100))`, which can return a reference, but declared the
local argument as a Number value. Every such row is withheld with its original
case and outcome in `heldout.json`; none was rewritten or treated as a kernel
failure. The fresh 76-row follow-up distinguishes reference-returning
IF expressions from `IF(FALSE,A100+0,A100+0)` numeric values and matches all 76
observations. The corrected origin declarations require no kernel adjustment.

The discovery packet additionally has 18 Formula2 assignment failures for
direct literals. Those failures establish no function result and are retained
separately in `discovery.json`. No #VALUE! outcome is invented for them.
Reference-returning expressions and literal syntax admission belong in the
function/evaluator boundary discussion, alongside the pre-edit assessment in
`IMPACT_ASSESSMENT.md`. No whole-function parity claim is made.

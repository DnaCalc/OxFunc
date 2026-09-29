# COUNT-family argument-kind correction: pre-edit assessment

State: in_progress; scope_completeness=scope_partial;
target_completeness=target_partial; integration_completeness=partial.
Open lanes: computed-scalar COUNTBLANK admission, independent validation,
COUNT locale-dependent numeric text, and evaluator-facing handoff assessment.

The 476-row live packet admits 458 executions and rejects 18 COUNTBLANK direct
literal formulas during Formula2 assignment. Those 18 rows have no observed
function result and must not be turned into expected #VALUE! outcomes. Exact
small literals 0, 2 and 0.5 were deliberately preserved as literals because
COUNTBLANK's reference admission can distinguish them from cell references.

The 245 admitted differences establish three argument-kind rules. COUNT ignores
error values in direct, array and reference origins; COUNT and COUNTA include
explicitly omitted slots; COUNTBLANK ignores error-valued cells in references,
while a computed array preserves existing error values at their positions and
returns #VALUE! for other array values. All seven error codes were exercised.
Declared B1:Bn array fixtures avoid the independent harness issue assembling
separately declared cells into an area.

`count_argument_included` is called only by COUNT, and
`counta_argument_included` only by COUNTA. Their changes can remain in these
local classification functions without changing SUM/AVERAGE/MIN/MAX folds,
reference resolution, shared numeric parsing, or public Rust interfaces.
COUNTBLANK's referenced error rule belongs in its local scan; its computed-array
publication belongs in its existing local array result construction. No metadata
or generic array preparation changes are proposed in this first candidate.

These are evaluator-visible semantic corrections. The root campaign owns
handoff registration/acknowledgement and any declaration changes. In particular,
COUNTBLANK's existing 1..255 arity and reference/value admission declaration need
separate review; literal formula rejection alone does not establish the entire
function/evaluator boundary contract. The function remains partial.

# MROUND prepared-value impact assessment

Status: `scope_completeness=scope_partial`, `target_completeness=target_partial`,
`integration_completeness=partial`. This assessment precedes the local prepared
surface change. Root owns canonical contracts, metadata evidence and any handoff.

The 618 public COM observations find 111 differences in the previous surface:
77 scalar coercion/error rows, four explicit-reference rows, twelve array-cell
coercion rows, and eighteen positional-padding rows. The initial report and raw
observations remain retained; the numerical candidate is independent of these
surface corrections.

Observed public examples:

- `MROUND(7,TRUE)` and `MROUND(0,FALSE)` return `#VALUE!`; so do logical values
  supplied through a cell reference or a materialized array cell.
- `MROUND(7,)` and `MROUND(,#NUM!)` return `#N/A`. An empty cell still becomes
  numerical zero, so an explicit omitted slot must stay distinct from a blank.
- With first array `{7,7,#REF!}` and second array `{2,#NUM!}`, the result is
  `{8,#NUM!,#REF!}`. The absent third coordinate of the second argument supplies
  `#N/A` in that position; the earlier first-argument `#REF!` is selected first.
- Numeric text remains admitted through the shared parser; nonnumeric text and
  worksheet errors are selected in left-to-right argument order before numeric
  domain checks. The 400 scalar cross-product controls cover both orders.

The proposed MROUND-local preparation resolves references and normalizes unit
arrays using the existing value preparation. It recursively replaces logical
values with `Value` and explicit Missing with `NA`, then calls the already
characterized `elementary_prepared::ordered_binary`. That helper's Missing→0
branch is unreachable for the converted MROUND inputs. Its per-position padding
and scalar coercion order are reused without editing the frozen helper.

MROUND should declare `Custom` coercion/lifting instead of the inaccurate
`UnaryNumericScalarOnly` label. Arity, numerical signature, reference dependency,
and numerical kernel remain unchanged. The function has its own existing
dispatch arm, so no generator change is needed. Generic adapters, operators,
QUOTIENT, and other consumers retain their current behavior. In particular,
QUOTIENT's similar logical/Missing mapping is not a reason to alter its shape
behavior without its own observations.

Evaluator-facing requirements are to preserve explicit Missing versus Empty,
logical versus numeric values, input-array shape, and worksheet errors until
the function's prepared surface applies its policy. No claim is made about
lazy argument evaluation, unresolved reference failure ordering, reference
identity/provenance returned by an evaluator, or arbitrary nested arrays.
Those seams require their own evaluator evidence and acknowledgment.

Validation required before the local prepared candidate is banked: replay all
618 discovery rows through both direct surface and public dispatch; retain all
44,764 numerical matches; build the executable Lean coercion/order binding;
freeze source and helper hashes; generate and capture fresh typed/origin/shape
controls. Integration remains partial pending the parent's canonical handoff
and receiving-repository acknowledgment.

## Independent reference-origin counterexamples

The frozen prepared candidate matches 276/432 fresh observations after a
qualified serialized repeat. All 156 failures involve multi-cell references;
all 273 cases without them match. `MROUND(B1:D1,F1:G2)` returns scalar `Value`
instead of a two-by-three lifted result. When a reference is paired with an
explicit array, Excel preserves only the explicit array's output shape.
These observations require preserving reference origin before materialization;
the present `ValuesOnlyPreAdapter` declaration does not express that distinction.

The 80 additional public controls now distinguish the rules: unconditional
multi-cell-reference rejection matches 80/80, caller-aligned intersection 55/80,
and the frozen production candidate 26/80. Aligned row/column positions still
reject references. Unary-plus materialization lifts, while INDEX/IF reference
results remain references and reject. No caller-context dependency is needed.

The narrow refinement preserves the original argument's reference origin until
after value preparation. A reference resolving to a non-unit array contributes
scalar `Value` in that argument position; explicit arrays and unit references
retain their existing preparation. Logical/Missing conversion and ordered
padding then run unchanged. `RefsVisibleInAdapter` replaces
`ValuesOnlyPreAdapter` so the evaluator must not erase origin first. Reference
dependency stays `RefOnly`, the pure numerical kernel stays context-free, and
no generic adapter or operator changes. The existing error-order tests continue
to distinguish earlier scalar errors from the later reference rejection.

The original failed freeze and both qualified/provisional heldouts remain
immutable. After this local correction, fresh independent reference/origin,
materialization, unit-array and error-order controls must be captured. The
numerical kernel remains unchanged and exact on all 47,964 retained numerical
observations. Multi-area/structured/external references, unresolved provider
failures and broader evaluator integration remain separate open lanes.

## Exercised refinement and receiving-repository requirements

The final frozen candidate matches all 384 fresh serially captured reference
controls. Its seven frozen source/dependency hashes remain unchanged. The four
typed replay tests now cover 1,514 observations through direct and public
dispatch; four numerical tests cover 47,964 observations. Metadata golden and
the seven-job Lean build pass. Prior failures remain preserved as evidence.

The receiving evaluator must preserve explicit omission, logical cell values,
array shapes, and original reference origin until the custom surface runs.
It must distinguish a true reference from its unary-plus materialization.
True resolver failures remain separately uncharacterized because preparation
resolves references before the scalar argument-order selection. These results
support the local correction and handoff; they do not establish receiving-repo
integration or a whole-function parity claim.

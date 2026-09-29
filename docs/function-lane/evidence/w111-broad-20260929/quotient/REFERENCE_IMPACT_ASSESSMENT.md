# QUOTIENT reference-origin preparation assessment

Status: in_progress; scope_completeness=scope_partial;
target_completeness=target_partial; integration_completeness=partial.

The existing 504 prepared observations contain direct arrays and scalar
references but no multi-cell-reference argument. A separate 80-case Excel
capture now produces 54 structural differences. Replacing every non-unit
reference with VALUE predicts 80/80 outcomes; caller intersection predicts
55/80. Aligned and unaligned row, column and matrix calls, reference-valued
INDEX/IF expressions, and explicit unary-plus materialization distinguish
reference rejection from array-value lifting. The complete failed baseline
and current pre-repair sources are retained in reference-discovery/.

The local repair retains original argument kinds until after resolution. An
original Reference that resolves to more than one cell becomes VALUE at that
argument position. Unit references retain existing scalar coercion; direct
materialized arrays retain their existing lifting. No generic adapter or
numeric arithmetic change is proposed. Reference-valued expression tests are
prepared-origin observations, not an implementation of evaluator laziness.

The declaration must expose references to the function adapter rather than
pre-converting them into arrays: RefsVisibleInAdapter replaces ValuesOnlyPreAdapter.
Custom replaces UnaryNumericScalarOnly to describe the already exercised binary
logical/Missing policy and array-value lifting. The public by-index route already
calls the function surface; numeric Q dispatch retains the unchanged kernel.
The matching Lean declaration and reference-rejection binding change together.
Only QUOTIENT's golden metadata row changes, preserving no final newline.

The root will extend HO-FN-027. OxFml must preserve reference origin, resolved
extent and argument position through this function's local preparation, and
acknowledge both declarations. Multi-area/unresolved reference forms, broader
ordered array padding, contextual numeric text and receiving integration remain
open. Fresh independent reference/error/array controls follow a source freeze;
passing the 80 discovery rows does not substitute for that validation.

The first frozen 512-row independent packet validates the reference distinction
but exposes five materialized-array padding failures. Every failure has an
earlier left coercion error at a coordinate absent from the right array. Excel
retains the earlier error; the old generic broadcast helper emits NA. The failed
freeze and all first outcomes remain immutable under reference-heldout/.

The successor changes only QUOTIENT's final local prepared call to the existing
ordered binary helper after its own logical, Missing and reference-kind mapping.
Padding becomes an NA value at its original argument position; left-to-right
scalar coercion then selects the observed error. Numeric arithmetic, reference
admission and shared adapters remain unchanged. This observed clause extends
HO-FN-027; fresh asymmetric-shape validation is required after the next freeze.

The successor's fresh 624 controls all match: 480 asymmetric shape/error rows,
96 multi-cell references and 48 unit references. All seven frozen source hashes
and the QUOTIENT golden-row hash remain unchanged. Direct and generated public
surfaces both replay the entire new bank. Current prepared replay totals are
1,720 exact observations; numerical replay remains 3,141 exact observations and
the numerical kernel body is unchanged. Five prepared tests, two numerical
tests, the metadata golden and seven targeted Lean jobs pass. No further source
change follows this freeze. The earlier five failures remain historical; wider
reference/evaluator/context obligations and HO-FN-027 acknowledgment stay open.

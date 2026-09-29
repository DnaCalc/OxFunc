# Proposed HO-FN-027 impact text: GCD/LCM and QUOTIENT argument semantics

State: in_progress; scope_completeness=scope_partial;
target_completeness=target_partial; integration_completeness=partial.
Open lanes: shared contextual numeric-text parsing, metadata review and
receiving-repository acknowledgement. Root owns the actual handoff and register.

GCD/LCM require family-specific numeric preparation. The initial 50 typed rows
and 652 admitted follow-up rows show numeric text parsed and logical values
rejected with VALUE in direct arguments, computed arrays and reference values.
The first explicitly omitted argument produces NA; later omitted arguments are
ignored. Scalar blank cells are ignored, and a call with no remaining values
returns VALUE. A blank within a multi-cell reference contributes zero. A
singleton blank reference remains scalar; 1x2, 2x1 and 2x2 all-blank references
reduce to zero. The further 90 typed refinement rows exercise these distinctions.
Two empty-call formulas failed Formula2 assignment and supply no function result.

All argument coercion runs before numeric-domain validation. Consequently a later
text/coercion error or explicit worksheet error outranks an earlier negative or
oversized numeric value. Among coercion errors, the first encountered argument
error is retained. For example GCD(-1,#DIV/0!) gives DIV0, while GCD("x",#DIV/0!)
gives VALUE. Both families preserve raw numeric values until their own domain
check and ordinary truncation; an evaluator must not pre-truncate negatives.
GCD/LCM reduce arrays to one value. The 126-row LCM array-order capture establishes
that numerical reduction reverses both argument order and row-major cell order,
after forward coercion and domain validation. Rounded intermediate products make
this observable: LCM(3,3,3002399751580331) returns NUM, whereas its reverse returns
9007199254740992. The current local candidate replays all 6,638 retained numeric
and 918 admitted typed rows exactly. Two empty-call Formula2 assignment failures
remain excluded. Its independently seeded 7,104 numeric and 1,160 typed controls
also replay exactly with zero exclusions and no candidate refinement. The full
retained bank now has 13,742 numeric and 2,078 typed exact observations.

The current provider seam already exposes declared reference extent and defined
cell positions. Those facts are sufficient to distinguish scalar blank from
multi-cell blank contributions without a new API, provided OxFml preserves them
and row-major error order. LCM also needs the ordered numeric stream. Sparse enumeration must not silently discard the
semantic effect of blank coordinates for LCM. The functions currently declare
ValuesOnlyPreAdapter/Custom with RefOnly surface dependence while performing
reference expansion in the adapter. OxFml should confirm that its preparation
and cache/declaration interpretation preserve the observed shapes and values;
the draft does not assert that a new metadata enum is required.

QUOTIENT has a separate, already exercised binary policy: prepare values through
the existing resolver, reject logical values with VALUE and Missing with NA,
coerce ordinary numeric text through the shared parser, and treat a referenced
blank as zero. Its binary broadcasting preserves per-argument errors and shape
behavior. The retained bank contains 3,141 admitted numeric and 504 typed rows.
Its pre-existing UnaryNumericScalarOnly coercion-lift declaration does not
describe the exercised binary array lift and custom logical/missing policy.
The receiving repository should review the appropriate declaration and verify
that its dispatch does not replace this policy with a generic unary numeric lift.

Requested acknowledgement: confirm or revise the evaluator-facing preparation
and metadata interpretation for these three functions, preserve origin/shape
facts needed by the reducer and binary policies, and identify any necessary
OxFml changes with exercised tests. Locale/date/current-year numeric parsing
remains the separate HO-FN-022 dependency. Filing this packet does not resolve
that dependency or authorize a function completion claim.

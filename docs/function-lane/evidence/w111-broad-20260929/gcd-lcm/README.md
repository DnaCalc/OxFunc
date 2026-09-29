# GCD/LCM: W111 bounds, preparation and rounded reduction

State: in_progress; scope_completeness=scope_partial;
target_completeness=target_partial; integration_completeness=partial.
Open lanes: shared contextual numeric
text (HO-FN-022), receiving-repository acknowledgement of preparation and
metadata (proposed HO-FN-027), and final whole-function review.

The retained discovery bank has 3,319 numeric rows per function and 918 admitted
typed rows across both functions. Numeric packets use hexadecimal binary64 input
bits and exact expected output bits/error identities. They contain no negative
zero or nonzero subnormal inputs requiring Value2 exclusions. Typed packets retain
full cases, fixtures, formulas and raw Excel results. Two zero-argument formulas
failed Formula2 assignment and are withheld; they establish no function result.

The candidate performs forward coercion of all arguments before checking any
numeric domain. Logical and unparseable text values return VALUE in every tested
origin. Numeric text is parsed in scalar, array and reference positions. An
explicitly missing first argument gives NA; later missing arguments are ignored.
Scalar blank cells and normalized singleton blank arrays are ignored. Blanks in
multi-cell arrays or ranges contribute zero. An empty reduction gives VALUE.
Forward argument/row-major cell order selects the first coercion error, so
GCD(-1,#DIV/0!) returns DIV0 and GCD("x",#DIV/0!) returns VALUE.

After coercion, every raw number must be finite and within [0,2^53], inclusive.
Negative fractions are rejected before ordinary truncation toward zero. Exact
2^53 is empirically admitted, contrary to the published GCD/LCM statement that
values greater than or equal to 2^53 are rejected. The discrepancy is recorded
with public source links in IMPACT_ASSESSMENT.md. GCD then uses integer Euclid.

LCM first recognizes any zero after all coercion and domain checks. Otherwise it
reduces the entire flattened stream in reverse order. Each step divides by the
integer GCD, forms the exact product in u128, rounds it to binary64, and rejects
a rounded value above 2^53. The exact product 2^53+1 rounds down to admitted
2^53. The 126 typed order discriminators distinguish reversal of outer arguments
from reversal within arrays; only reversing both matches every observation.
The u128 bound follows from both admitted operands being at most 2^53. This
removes the former unchecked i64 multiplication panic. Its observed original
local failure log is retained as original-local-overflow.log, alongside the raw
Excel outcomes; a host-runner failure was never treated as an Excel result.

The sparse reference path consumes existing declared extent and defined-cell
positions, orders cells row-major, and adds one zero when a multi-cell range
contains implicit blanks. One zero is sufficient for both reductions, avoiding
dense allocation. Local tests exercise unordered sparse enumeration, first-error
ordering, singleton blanks and multi-cell blanks. No resolver API was changed.

The production-dispatch test target w111_gcd_lcm_live_replay has ten passing
tests covering all retained numeric and admitted typed discovery rows. The
executable Lean GcdLcmReduction model includes the same preparation, exact GCD,
reverse LCM fold, rounded upper boundary and zero publication. Its focused build
with GcdFn and LcmFn passes. It models admitted finite numbers and prepared shapes;
it does not claim to model contextual text parsing or the host reference engine.

The candidate freeze records source hashes before the independent packet was
sent for capture. Seed 202609292701 generates 3,552 numeric rows per function and
1,160 typed rows: full-bit normals, common factors and multiples, new product
boundary factors, integer neighbors, 255-argument controls, grouped arrays,
paired scalar/array/reference origins, blanks, missing values and error order.
All 7,104 independent numeric rows and 1,160 independent typed rows match the
frozen candidate exactly. The typed runner reports no excluded execution or
ingress rows, and the numeric packet contains only admitted normal values and
explicit positive zero. Thirteen production-dispatch replay tests now cover
13,742 numeric and 2,078 typed observations (15,820 total), with zero differences.
The independent capture did not require a candidate refinement. A read-only
review also found no defect on the admitted surface; the public integer kernels
themselves require nonnegative, bounded, already-prepared operands.

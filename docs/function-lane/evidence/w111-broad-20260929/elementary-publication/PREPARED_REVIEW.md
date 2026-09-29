# Read-only review of the numerical candidate

Status: `in_progress`; `scope_completeness=scope_partial`,
`target_completeness=target_partial`, `integration_completeness=partial`.
Open lanes: typed argument preparation, coercion/lift declarations, broader
numerical primitive and platform alignment, and receiving-repo acknowledgment.
No family source or canonical register was changed by this review.

The reviewed LOG, FISHER, MEDIAN and integer POWER arithmetic agrees with the
stated admitted numeric graph and its executable `ElementaryPublication` binding.
No additional arithmetic defect was found on that declared finite-source slice.
In particular, MEDIAN's half-difference is nonnegative after sorting finite
inputs, so Rust's direct lower-bound comparison and Lean's absolute-value
`flushTiny` agree under the documented ordering precondition. Overflow reaches
NUM. The integer POWER model has sufficient fuel for the admitted unsigned32
range and explicitly separates signed decimal-scale conversion at positive ten.
Its negative fractional-root model remains outside the integer binding.

The following source-derived risks need worksheet observations before they are
reported as new Excel mismatches:

| Surface | Current preparation behavior | Distinguishing observation |
|---|---|---|
| LOG | Only the number argument lifts; an array base is rejected. Array number elements accept only Number or Error, unlike scalar coercion. | Paired scalar/array/reference text and logical values; scalar number with array base; two array arguments. |
| LOG | Scalar base coercion occurs before mapping number-array cells. | Earlier number-array errors against a different scalar base error. |
| LOG / POWER | Generic numeric coercion rejects explicit Missing. LOG separately defaults an omitted base to ten. | Explicit missing first/second slots compared with omitted LOG base and blank reference cells. |
| POWER | The binary helper expands arrays, but metadata still declares `UnaryNumericScalarOnly`. A missing broadcast coordinate becomes NA before either argument is coerced. | Unit and mismatched arrays, with errors in opposite argument positions; declarations assessed after capture. |
| FISHER | Uses the existing unary numeric executor and scalar/element policy. | Direct, reference, unit-array and larger-array text/logical/blank values. |
| MEDIAN | Reuses the existing aggregate direct/range dual policy. | Numeric text and logicals as direct values, unit arrays, mixed arrays and reference areas; explicit Missing alongside a numeric argument. |

The compact discovery packet is
`smart-fuzzer/runs/w111-elementary-prepared-audit-20260929-002/typed.json`:
433 cases (LOG 149, POWER 135, FISHER 46, MEDIAN 103). The first 398-case local
design was superseded before execution by the `-002` packet, which adds unit
arrays and mixed reference-area elements. It is not an independent arithmetic
holdout. A sole unary Missing renders as `FISHER()`, which the current runner's
arity guard rejects; that specific lane is deferred rather than silently
represented by a computed numeric zero.

The older exact-rational POWER functions in `PowerFn.lean` are idealized reference
definitions, not the executable binary64 graph. Its existing surface-class model
also rejects fractional negative bases that the production odd-root rule admits.
The newer `powerIntegerGraphBinding` is the numerical binding for the admitted
integer slice. These distinct scopes must remain explicit in any formalization
claim. POWER source comments referring to an integer-detection tolerance and
shared financial growth callers predate the exact current dispatch and the
separate consumer audit; they should not be read as the current admission contract.

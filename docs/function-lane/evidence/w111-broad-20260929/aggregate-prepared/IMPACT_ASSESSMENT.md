# HARMEAN and DEVSQ prepared-argument assessment

State: in_progress; scope_completeness=scope_partial;
target_completeness=target_partial; integration_completeness=partial.
Root owns BUG-FUNC-070 and the planned HO-FN-030 receiving dependency.

The 248 public Excel cases execute successfully on both sides. Initial results
are 109/124 HARMEAN and 113/124 DEVSQ. The proposal was sent to the root agent
before the local source edit; this repository record was saved afterward.

Both functions must ignore Empty cells and treat an explicit DirectScalar
Missing as numeric zero. Missing values inside non-direct prepared collections
retain the existing ignored policy; no unobserved generalization is made.
HARMEAN must collect scalar coercion and explicit worksheet errors before
positivity validation. An earlier 0 or negative number does not hide a later
invalid scalar text or explicit DIV/0, including inside arrays/reference areas.
The no-values results remain HARMEAN NA and DEVSQ NUM, as ignored-only ranges
already match. References retain the existing resolver and origin semantics.

The change is local to prepared collection in harmean_fn.rs and devsq_fn.rs.
There is no generic aggregate helper, declaration or metadata change. The
HARMEAN reciprocal/average graph and DEVSQ square-publication graph are retained.
Their numeric frozen banks must be replayed after the collection change.
AggregatePublication's existing numeric binding remains intact, with a separate
prepared input binding added for origin, Empty/Missing and error ordering.

This changes evaluator-facing prepared-value semantics and belongs in HO-FN-030.
No new receiving acknowledgement is implied. Shared locale-sensitive text
coercion remains an open receiving dependency; these observations do not prove
all numeric-text grammar. The generic typed harness has verified fixture ingress
and uses strict ordinal output digests with no output normalization.


## Follow-up: cross-origin error priority

The separate 675-case cross-origin packet includes 225 observations per function
for MEDIAN, HARMEAN and DEVSQ. HARMEAN and DEVSQ each have 16 discrepancies after
the first 814 observations matched. The prior success remains a bounded result.
Later DirectScalar explicit/coercion errors outrank earlier array/reference
errors; unit arrays keep collection origin. The proposed repair checks only
DirectScalar coercion errors first, then performs the existing ordered full
collection. It does not reorder numeric values or arithmetic, and it retains
left-to-right priority within each phase. This assessment is saved before that
source edit. Parent approved the local repair and a separate independent packet.
The receiving HO-FN-030 assessment must include this observed priority rule.

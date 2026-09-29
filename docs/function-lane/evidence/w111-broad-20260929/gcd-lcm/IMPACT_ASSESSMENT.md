# GCD / LCM preparation and integer-bound assessment

Historical assessment written before the candidate edits. Later captures refine
the provisional arithmetic and shape model below; README.md and
HO_FN_027_IMPACT_DRAFT.md contain the current evidenced rules.

State: in_progress; scope_completeness=scope_partial;
target_completeness=target_partial; integration_completeness=partial.
Open lanes: aggregate-origin policy, missing/error order, near-limit result
arithmetic, independent validation and evaluator-boundary acknowledgement.

The current functions expand aggregate arguments through the existing reference
provider, then apply scalar numeric coercion to every expanded item and discard
its origin tag. They use the shared factorial truncation helper, which saturates
oversized float-to-integer casts instead of rejecting the GCD/LCM input domain.
LCM also uses unchecked signed integer multiplication. This assessment precedes
any source edits; the shared factorial helper must remain unchanged.

All 2,110 numeric discovery observations per function agree with the following
provisional model: reject raw inputs below zero or above 2^53, truncate admitted
values toward zero, compute integer GCD/LCM, and reject a final LCM above 2^53.
The upper bound is inclusive in this capture. An explicit zero can cancel an
earlier oversized LCM intermediate, but does not admit an oversized raw input.
Follow-up products at 2^53+1 distinguish exact integer product checking from
binary64-rounded checking; this distinction is not inferred from the initial
corpus's agreement.

Documentation discrepancy, checked 2026-09-29: Microsoft's
[GCD reference](https://support.microsoft.com/en-us/excel/functions/gcd-function)
describes rejection at an input greater than or equal to 2^53; its
[LCM reference](https://support.microsoft.com/en-us/excel/functions/lcm-function)
describes the same inclusive rejection for the result. This live baseline admits
exactly 2^53 in both observed lanes, so the candidate must follow the empirical
inclusive admission boundary, not those documented inequalities.

The initial typed observations show logical scalar arguments rejected with
VALUE, blank cells ignored, an omitted first argument returning NA, and an
omitted trailing argument ignored. Singleton references, area references and
computed arrays require separate controls before an origin rule is selected.
All ordinary argument errors and domain errors must retain the observed order.

The proposed scope is local to gcd_fn.rs, lcm_fn.rs and gcd_lcm_common.rs, plus
their executable Lean models and retained production-dispatch tests. Reference
resolution, aggregate enumeration and other function coercions are not modified.
Any revised local consumption of origin/shape is still evaluator-visible. The
existing declarations say ValuesOnlyPreAdapter despite runtime aggregate
expansion inside the adapter; that pre-existing declaration needs a parent-owned
boundary assessment before an integration claim. The parent owns BUG-FUNC-064,
bead oxf-mwue.28.14, canonical records and cross-repository handoff handling.

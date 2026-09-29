# BUG-FUNC-055: SLN sign admission and stored arithmetic publication

## Current campaign observation (2026-09-29)

SLN stored subtraction, staged division and publication match 8,854 numerical and 304 typed observations, including independent banks of 7,250 numeric and 265 typed cases. Explicit Missing is zero. Current build/CV evidence does not establish other-platform primitive parity or discharge the full function review.

Status remains `investigating`: `scope_partial`, `target_partial`, integration `partial`.
The historical discovery text below is retained; its pending counts are superseded
by this paragraph and the linked family evidence.

## Retained discovery record

Status: `investigating`; owner W111, bead `oxf-mwue.28.6` (parent `oxf-mwue.28`).

Negative cost/salvage/life and tiny nonzero life were incorrectly rejected. Discovery1205 and discriminator399 identify a stored subtraction, extended-precision quotient and final zero/subnormal publication. Candidate typed39 agree; independent7250 numeric holdout is pending.

Reported/reproduced baseline: `11b23504fe8180d08397be5435bd215e1bbb9722`; introduced ref unknown. Candidate repairs
are in the 2026-09-29 working tree and have no integration/sign-off claim.
Ownership: OxFunc function semantics. Cause: initial implementation gap and/or
numeric operation graph; exact cause is documented in the family evidence.

Evidence: [sln observations](../../function-lane/evidence/w111-broad-20260929/sln)
and [campaign provenance](../../function-lane/evidence/w111-broad-20260929/README.md).
Live oracle: Excel16.0 build20430,64-bit,CV2,1900date system; channel unverified.
Both version axes and all known transport exclusions are part of the claim.
The numeric baseline is replayable from the campaign generator/outcome bank.

Public-doc/empirical relationship: empirical results are authoritative for this
build. Any finer documented-rule discrepancy is retained in the family record;
no unpublished implementation source or binary inspection was used.

Cross-repo impact: the candidate repairs preserve function metadata, FEC/F3E
contracts and evaluator-facing signatures. Existing host-provider/reference
limitations remain separate. A later discovered boundary-contract change must
receive an OxFml handoff before promotion.

`scope_completeness: scope_partial`; `target_completeness: target_partial`;
`integration_completeness: partial`. Open lanes: final candidate evidence,
all declared argument axes, formal alignment, combined validation and canonical
parity reconciliation. This stream remains in progress until those obligations
and the OPERATIONS12/14 audits establish the intended scope.

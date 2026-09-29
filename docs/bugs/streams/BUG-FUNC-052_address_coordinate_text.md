# BUG-FUNC-052: ADDRESS coordinate conversion, selector and sheet text rules

Status: `investigating`; owner W111, bead `oxf-mwue.28.4` (parent `oxf-mwue.28`).

Negative relative R1C1 coordinates, grid limits, near-integer conversion, style text, error order and sheet quoting/rendering differ. The numeric candidate matches20729 discovery observations; typed refinement and a fresh final heldout remain open.

Reported/reproduced baseline: `11b23504fe8180d08397be5435bd215e1bbb9722`; introduced ref unknown. Candidate repairs
are in the 2026-09-29 working tree and have no integration/sign-off claim.
Ownership: OxFunc function semantics. Cause: initial implementation gap and/or
numeric operation graph; exact cause is documented in the family evidence.

Evidence: [address observations](../../function-lane/evidence/w111-broad-20260929/address)
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

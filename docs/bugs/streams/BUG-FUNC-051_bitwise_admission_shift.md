# BUG-FUNC-051: Bitwise integer admission, shift graph and omitted arguments

## Current campaign observation (2026-09-29)

The original bitwise banks retain 34,246 qualified numeric and 862 typed matches. A later shared numeric-text audit finds 50 mismatches among 240 admitted text observations. These failures remain open; the earlier successful banks do not establish whole-function parity.

Status remains `investigating`: `scope_partial`, `target_partial`, integration `partial`.
The historical discovery text below is retained; its pending counts are superseded
by this paragraph and the linked family evidence.

## Retained discovery record

Status: `investigating`; owner W111, bead `oxf-mwue.28.3` (parent `oxf-mwue.28`).

Fractional operands were truncated; 48-bit overflow and shifts49..53 were handled differently from Excel; explicit omitted positions did not become zero. The repaired candidate matches34246 admitted numeric observations and862 typed observations, including independent heldouts. Input-ingress exclusions and candidate hashes are retained.

Reported/reproduced baseline: `11b23504fe8180d08397be5435bd215e1bbb9722`; introduced ref unknown. Candidate repairs
are in the 2026-09-29 working tree and have no integration/sign-off claim.
Ownership: OxFunc function semantics. Cause: initial implementation gap and/or
numeric operation graph; exact cause is documented in the family evidence.

Evidence: [bitwise observations](../../function-lane/evidence/w111-broad-20260929/bitwise)
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

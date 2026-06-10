# W71 Frozen Gap Reconciliation — Shell-Counting Caveat (2026-06-11)

This note accompanies `W71_FROZEN_GAP_RECONCILIATION.csv` and corrects a
misleading claim in its `witness_covered_rows` row.

## Caveat

The CSV reports:

```
witness_covered_rows = 517
remaining_witness_gap_rows = 0
```

These figures count placeholder shell artifacts (generated W69/W71 JSON files
with no real signature or argument data) as "covered". They are NOT an
accurate measure of real witness coverage.

## Real Placeholder Count (as of 2026-06-10 review)

- 229 of 528 `registry_signature_seed.rs` rows carry a placeholder
  `signature_display` of the form `"FUNCNAME(...)"` with an empty parameter
  list and `trailing_repeats: true`.
- These concentrate in: Database (100%), Financial (91%), Date/time (88%),
  Compatibility (68%), Engineering (54%), Statistical (41%).
- The 229 placeholder rows have no parameter names, no parameter descriptions,
  and no real signature string.

## Why The CSV Counts Are Stale

The W69 generator emitted shell JSON files (TRANCHE_T1_ORDINARY_EXTRACTED,
TRANCHE_T2_ORDINARY_CURATED, SH1) that counted as "covered" in the ledger,
but those shells contained only placeholder signatures. The W71 hand-authored
batch pass re-authored real content for T1 (201 entries, complete) and the
beginning of T2 (36 of 267 curated rows), leaving 231 T2 rows unreauthored.
The W091 registry-seed projection then correctly captured that state, leaving
229 placeholder entries in the runtime seed.

## Corrected Reading

A consumer of `W71_FROZEN_GAP_RECONCILIATION.csv` should read:

- `witness_covered_rows = 517` means 517 surface ids have a corresponding
  JSON artifact file (shell or real); it does NOT mean 517 rows have real
  signature data.
- The true count of rows with real (non-placeholder) signature data is
  approximately 299 (528 total minus 229 placeholders).
- The "remaining gap" for real signature coverage is approximately 229 rows,
  not 0.

## Tracking

The 229 placeholder rows are being burned down under W106 (help fill, per
the 2026-06-10 cleanup pass plan). A guard test tracking the placeholder count
as a monotonically decreasing number is planned per docs-help recommendation 2
of the 2026-06-10 review.

## Source

2026-06-10 review digest `docs-help-gaps.md` findings F2, F4; cleanup plan
`.tmp/CLEANUP_IMPROVEMENT_PASS_PLAN_2026-06-10.md` W101-B item 7.

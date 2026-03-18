# W16 Batch 72 - Compatibility Statistical Test Aliases

Status: `packet-evidenced`
Workset: `W16`

## Scope
1. `CHISQ.TEST`
2. `CHITEST`
3. `F.TEST`
4. `FTEST`
5. `T.TEST`
6. `TTEST`
7. `ZTEST`

## Current Implementation Notes
1. This note is now historically narrow rather than current-state authoritative.
2. On the current integrated surface, `CHISQ.TEST`, `CHITEST`, `F.TEST`, `FTEST`, `T.TEST`, and `TTEST` are all executable through `statistical_tests_family.rs`.
3. `ZTEST` remains the only legacy statistical-test name routed through `test_alias_family.rs`, where it delegates to modern `Z.TEST`.
4. `W24` Batch 07 supplies the native packet evidence for all seven names.

## Seeded Unit Lanes
1. `ZTEST(A1:A5,4,1.5)` matches the modern `Z.TEST(A1:A5,4,1.5)` delegate exactly.
2. `CHITEST(...)`, `FTEST(...)`, and `TTEST(...)` all match their modern counterparts on the current baseline.

## Open Lanes
1. The old open-lane wording is obsolete after `W24` Batch 07.
2. The only remaining limitation is that `test_alias_family.rs` itself is now a partial historical shim, not the authoritative semantic surface for the whole family.

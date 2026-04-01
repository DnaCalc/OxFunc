# WORKSET - Financial Core Misc Family (W064)

## 1. Purpose
Promote the sixth ordinary successor packet from the normalized `W051` backlog by closing the financial core miscellaneous family for the current reference Excel baseline.

This packet closes:
1. `CUMIPMT`
2. `CUMPRINC`
3. `DB`
4. `DDB`
5. `DISC`
6. `DOLLARFR`
7. `INTRATE`
8. `PRICEDISC`
9. `RECEIVED`
10. `SLN`
11. `SYD`
12. `TBILLEQ`
13. `TBILLPRICE`
14. `TBILLYIELD`
15. `VDB`

## 2. Dependencies
1. `W057_HIDDEN_ORDINARY_BACKLOG_SYSTEMATIC_COMPLETION_PLAN.md`
2. `W058_GROUPED_ROW_NORMALIZATION_AND_HIDDEN_BACKLOG_SPLIT.md`
3. `docs/function-lane/W58_SUCCESSOR_PACKET_SPLIT.csv`
4. prior implementation/evidence lineage from:
   - `crates/oxfunc_core/src/functions/cumulative_finance_family.rs`
   - `crates/oxfunc_core/src/functions/depreciation_family.rs`
   - `crates/oxfunc_core/src/functions/discount_bill_yearfrac_family.rs`
   - `crates/oxfunc_core/src/functions/dollar_fraction_family.rs`
   - `formal/lean/OxFunc/Functions/CumulativeFinanceFamily.lean`
   - `formal/lean/OxFunc/Functions/DepreciationFamily.lean`
   - `formal/lean/OxFunc/Functions/DiscountBillYearfracFamily.lean`
   - `formal/lean/OxFunc/Functions/DollarFractionFamily.lean`
   - `docs/function-lane/W16_BATCH37_DOLLAR_FRACTION_NOTES.md`
   - `docs/function-lane/W16_BATCH63_DEPRECIATION_NOTES.md`
   - `docs/function-lane/W16_BATCH70_CUMULATIVE_FINANCE_NOTES.md`
   - `docs/function-lane/W16_BATCH76_DISCOUNT_BILL_YEARFRAC_NOTES.md`

## 3. Scope
In scope:
1. native Excel replay for the declared current-baseline cumulative-finance, depreciation, discount-security, Treasury-bill, and `DOLLARFR` slice,
2. closure-grade OxFunc runtime and dispatch evidence for all `15` rows,
3. Lean substrate/binding alignment confirmation for the four existing finance-family formal artifacts,
4. snapshot/export promotion so these rows stop reading as hidden `catalog_only` backlog,
5. `W051` removal/update for the `15` rows.

Out of scope:
1. locale/version sweeps beyond the declared current reference baseline,
2. broader bond/odd-bond finance families and `YEARFRAC` follow-on work outside the admitted `W064` rows,
3. `DOLLARDE`, `YIELDDISC`, and finance families already owned by other packets,
4. workbook date-system expansion beyond the pinned `1900` baseline.

## 4. Output Contract
This packet must produce:
1. this workset spec,
2. `docs/function-lane/FUNCTION_SLICE_FINANCIAL_CORE_MISC_FAMILY_CONTRACT_PRELIM.md`,
3. `docs/function-lane/W64_SCENARIO_MANIFEST_SEED.csv`,
4. `docs/function-lane/W64_RUNTIME_REQUIREMENTS.md`,
5. `docs/function-lane/W64_SCOPE_RECONCILIATION.csv`,
6. `docs/function-lane/W64_EXECUTION_RECORD.md`,
7. `tools/w64-probe/run-w64-financial-core-misc-baseline.ps1`,
8. `.tmp/w64-financial-core-misc-results.csv`,
9. `tools/w44-probe/generate-w44-library-context-snapshot.ps1`,
10. refreshed downstream `W051` / snapshot / policy / worklist counters.

## 5. Completion Gate
`W064` may be reported complete only when:
1. every covered row has at least one explicit native Excel replay lane in `W64_SCENARIO_MANIFEST_SEED.csv`,
2. native Excel replay matches the seeded `W64` manifest for all declared lanes,
3. the targeted Rust tests for the four family modules pass,
4. `cargo fmt --manifest-path crates/oxfunc_core/Cargo.toml --all --check` passes,
5. `lake build` passes without introducing new `W054` active gaps,
6. all `15` `W064` rows move out of `catalog_only` in the regenerated `W44` snapshot,
7. `W051` and downstream counts reconcile to the post-`W064` backlog.

## 6. Notes
1. `W064` is expected to be a closure-by-evidence-and-publication packet rather than a large semantic rewrite packet unless the native replay exposes a mismatch.
2. `DOLLARFR` remains in scope here because it was still hidden backlog even though the paired `DOLLARDE` lineage had earlier evidence; `DOLLARDE` itself stays outside this packet because it is not part of the active ordinary backlog.

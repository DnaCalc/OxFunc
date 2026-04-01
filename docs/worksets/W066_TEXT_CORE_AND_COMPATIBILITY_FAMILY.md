# WORKSET - Text Core And Compatibility Family (W066)

## 1. Purpose
Promote the eighth ordinary successor packet from the normalized `W051` backlog by closing the text core and compatibility family for the current reference Excel baseline.

This packet closes:
1. `CODE`
2. `CONCATENATE`
3. `FIND`
4. `FINDB`
5. `LEFT`
6. `LEFTB`
7. `LEN`
8. `LENB`
9. `LOWER`
10. `MID`
11. `MIDB`
12. `PROPER`
13. `REPLACE`
14. `REPLACEB`
15. `REPT`
16. `RIGHT`
17. `RIGHTB`
18. `SEARCH`
19. `SEARCHB`
20. `SUBSTITUTE`
21. `TRIM`
22. `UNICODE`
23. `UPPER`

## 2. Dependencies
1. `W057_HIDDEN_ORDINARY_BACKLOG_SYSTEMATIC_COMPLETION_PLAN.md`
2. `W058_GROUPED_ROW_NORMALIZATION_AND_HIDDEN_BACKLOG_SPLIT.md`
3. `docs/function-lane/W58_SUCCESSOR_PACKET_SPLIT.csv`
4. prior implementation/evidence lineage from:
   - `docs/function-lane/W16_BATCH31_TEXT_SCALAR_MISC_NOTES.md`
   - `docs/function-lane/W16_BATCH35_TEXT_UNICODE_NOTES.md`
   - `docs/function-lane/W16_BATCH36_TEXT_SLICE_NOTES.md`
   - `docs/function-lane/W16_BATCH40_HELPER_CONCAT_NOTES.md`
   - `docs/function-lane/W16_BATCH42_TEXT_SEARCH_REPLACE_NOTES.md`
   - `docs/function-lane/W16_BATCH73_TEXT_B_COMPAT_NOTES.md`
   - `crates/oxfunc_core/src/functions/text_scalar_misc.rs`
   - `crates/oxfunc_core/src/functions/concat_family.rs`
   - `crates/oxfunc_core/src/functions/text_slice_family.rs`
   - `crates/oxfunc_core/src/functions/text_search_replace_family.rs`
   - `crates/oxfunc_core/src/functions/text_b_compat_family.rs`
   - `crates/oxfunc_core/src/functions/text_unicode_fn.rs`
   - `formal/lean/OxFunc/Functions/CodeFn.lean`
   - `formal/lean/OxFunc/Functions/LowerFn.lean`
   - `formal/lean/OxFunc/Functions/UpperFn.lean`
   - `formal/lean/OxFunc/Functions/TrimFn.lean`
   - `formal/lean/OxFunc/Functions/ReptFn.lean`
   - `formal/lean/OxFunc/Functions/ConcatFamily.lean`
   - `formal/lean/OxFunc/Functions/TextSliceFamily.lean`
   - `formal/lean/OxFunc/Functions/TextSearchReplaceFamily.lean`
   - `formal/lean/OxFunc/Functions/TextBCompatFamily.lean`
   - `formal/lean/OxFunc/Functions/TextUnicodeFn.lean`

## 3. Scope
In scope:
1. current-baseline promotion of the already-evidenced text core and compatibility family into the ordinary-backlog closure program,
2. native Excel replay for all `23` normalized rows through a dedicated `W066` manifest and runner,
3. closure-grade OxFunc runtime and dispatch evidence for the admitted current-baseline slice,
4. Lean substrate/binding alignment confirmation for the existing text-family formal artifacts,
5. snapshot/export promotion so these rows stop reading as hidden `catalog_only` backlog,
6. `W051` removal/update for the `16` hidden snapshot entries and `23` normalized execution rows.

Out of scope:
1. `CHAR`, `CONCAT`, and `UNICHAR`, which are already out of the hidden backlog and remain provenance-only neighbors for this packet,
2. broader locale/collation characterization beyond the current ASCII-seeded `SEARCH` baseline,
3. older DBCS-byte semantics beyond the current Unicode-baseline delegate posture for the `*B` compatibility rows,
4. locale/version sweeps beyond the declared current reference baseline,
5. workbook-version sweeps and historical-version sweeps.

## 4. Output Contract
This packet must produce:
1. this workset spec,
2. `docs/function-lane/FUNCTION_SLICE_TEXT_CORE_AND_COMPATIBILITY_FAMILY_CONTRACT_PRELIM.md`,
3. `docs/function-lane/W66_SCENARIO_MANIFEST_SEED.csv`,
4. `docs/function-lane/W66_RUNTIME_REQUIREMENTS.md`,
5. `docs/function-lane/W66_SCOPE_RECONCILIATION.csv`,
6. `docs/function-lane/W66_EXECUTION_RECORD.md`,
7. `tools/w66-probe/run-w66-text-core-compat-baseline.ps1`,
8. `.tmp/w66-text-core-compat-results.csv`,
9. `tools/w44-probe/generate-w44-library-context-snapshot.ps1`,
10. refreshed downstream `W051` / snapshot / policy / worklist counters.

## 5. Completion Gate
`W066` may be reported complete only when:
1. every covered row has at least one explicit native Excel replay lane in `W66_SCENARIO_MANIFEST_SEED.csv`,
2. native Excel replay matches the seeded `W66` manifest for all declared lanes,
3. the targeted Rust tests for the text-family sources pass,
4. `cargo fmt --manifest-path crates/oxfunc_core/Cargo.toml --all --check` passes,
5. `lake build` passes without introducing new active `W054` gaps,
6. all `16` `W066` hidden snapshot entries move out of `catalog_only` in the regenerated `W44` snapshot,
7. `W051` and downstream counts reconcile to the post-`W066` backlog (`18` hidden snapshot entries / `18` normalized execution rows).

## 6. Notes
1. `W066` is a promotion/reconciliation packet, not a fresh semantic-discovery packet: the current-baseline text-family semantics already have local runtime and empirical evidence across the `W16` batches.
2. The main current-phase gap before `W066` is publication drift: these rows remained hidden ordinary backlog in the published snapshot even after the underlying runtime/formal/evidence floor existed locally.

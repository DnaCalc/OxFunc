# WORKSET - Lookup And Logical Residuals (W068)

## 1. Purpose
Promote the tenth and final ordinary successor packet from the normalized `W051` backlog by closing the remaining lookup/logical residuals for the current reference Excel baseline.

This packet closes:
1. `HLOOKUP`
2. `IFS`
3. `VLOOKUP`

## 2. Dependencies
1. `W057_HIDDEN_ORDINARY_BACKLOG_SYSTEMATIC_COMPLETION_PLAN.md`
2. `W058_GROUPED_ROW_NORMALIZATION_AND_HIDDEN_BACKLOG_SPLIT.md`
3. `docs/function-lane/W58_SUCCESSOR_PACKET_SPLIT.csv`
4. prior implementation/evidence lineage from:
   - `docs/function-lane/W16_BATCH43_CHOOSE_IFS_NOTES.md`
   - `crates/oxfunc_core/src/functions/choose_ifs_family.rs`
   - `crates/oxfunc_core/src/functions/vhlookup_family.rs`
   - `formal/lean/OxFunc/Functions/ChooseIfsFamily.lean`
   - `formal/lean/OxFunc/Functions/VhlookupFamily.lean`

## 3. Scope
In scope:
1. current-baseline promotion of `HLOOKUP`, `IFS`, and `VLOOKUP` into the ordinary-backlog closure program,
2. dedicated native Excel replay for all `3` residual rows through a `W068` manifest and runner,
3. closure-grade OxFunc runtime and dispatch evidence for the admitted current-baseline slice,
4. Lean substrate/binding alignment confirmation for the existing `IFS` short-circuit substrate and the current-baseline `HLOOKUP` / `VLOOKUP` exact-vs-approximate index-selection substrate,
5. snapshot/export promotion so these rows stop reading as hidden `catalog_only` backlog,
6. `W051` removal/update for the last `3` hidden snapshot entries and `3` normalized execution rows.

Out of scope:
1. `LOOKUP` and `XLOOKUP`, which are already covered elsewhere and are not part of the remaining ordinary backlog,
2. locale/version sweeps beyond the declared current reference baseline,
3. workbook-version sweeps and historical-version sweeps,
4. broader wildcard/collation characterization beyond the seeded current-baseline exact-match lanes.

## 4. Output Contract
This packet must produce:
1. this workset spec,
2. `docs/function-lane/FUNCTION_SLICE_LOOKUP_AND_LOGICAL_RESIDUALS_CONTRACT_PRELIM.md`,
3. `docs/function-lane/W68_SCENARIO_MANIFEST_SEED.csv`,
4. `docs/function-lane/W68_RUNTIME_REQUIREMENTS.md`,
5. `docs/function-lane/W68_SCOPE_RECONCILIATION.csv`,
6. `docs/function-lane/W68_EXECUTION_RECORD.md`,
7. `tools/w68-probe/run-w68-lookup-logical-baseline.ps1`,
8. `.tmp/w68-lookup-logical-results.csv`,
9. `tools/w44-probe/generate-w44-library-context-snapshot.ps1`,
10. refreshed downstream `W051` / snapshot / policy / worklist counters.

## 5. Completion Gate
`W068` may be reported complete only when:
1. every covered row has at least one explicit native Excel replay lane in `W68_SCENARIO_MANIFEST_SEED.csv`,
2. native Excel replay matches the seeded `W068` manifest for all declared lanes,
3. the targeted Rust tests for `choose_ifs_family` and `vhlookup_family` pass,
4. `cargo fmt --manifest-path crates/oxfunc_core/Cargo.toml --all --check` passes,
5. `lake build` passes without introducing new active `W054` gaps,
6. all `3` `W068` hidden snapshot entries move out of `catalog_only` in the regenerated `W44` snapshot,
7. `W051` and downstream counts reconcile to the post-`W068` backlog (`0` hidden snapshot entries / `0` normalized execution rows).

## 6. Notes
1. `W068` is a promotion/reconciliation packet, not a fresh seam or substrate-discovery packet: `IFS` already has explicit `W16` empirical and formal support, and `HLOOKUP` / `VLOOKUP` already have local runtime plus Lean metadata support.
2. `W068` carries the first closure-grade dedicated native replay packet for `HLOOKUP` / `VLOOKUP` rather than relying on earlier implicit coverage.

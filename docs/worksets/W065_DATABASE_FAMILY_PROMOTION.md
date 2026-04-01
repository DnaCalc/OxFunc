# WORKSET - Database Family Promotion (W065)

## 1. Purpose
Promote the seventh ordinary successor packet from the normalized `W051` backlog by closing the database family for the current reference Excel baseline.

This packet closes:
1. `DAVERAGE`
2. `DCOUNT`
3. `DCOUNTA`
4. `DGET`
5. `DMAX`
6. `DMIN`
7. `DPRODUCT`
8. `DSTDEV`
9. `DSTDEVP`
10. `DSUM`
11. `DVAR`
12. `DVARP`

## 2. Dependencies
1. `W057_HIDDEN_ORDINARY_BACKLOG_SYSTEMATIC_COMPLETION_PLAN.md`
2. `W058_GROUPED_ROW_NORMALIZATION_AND_HIDDEN_BACKLOG_SPLIT.md`
3. `docs/function-lane/W58_SUCCESSOR_PACKET_SPLIT.csv`
4. prior implementation/evidence lineage from:
   - `docs/worksets/W023_DEFERRED_HOST_METADATA_AND_DATABASE_FUNCTIONS.md`
   - `docs/function-lane/FUNCTION_SLICE_DATABASE_FAMILY_CONTRACT_PRELIM.md`
   - `docs/function-lane/W23_DATABASE_SCENARIO_MANIFEST_SEED.csv`
   - `docs/function-lane/W23_DATABASE_RUNTIME_REQUIREMENTS.md`
   - `docs/function-lane/W23_EXECUTION_RECORD.md`
   - `tools/w23-probe/run-w23-database-baseline.ps1`
   - `.tmp/w23-database-results.csv`
   - `crates/oxfunc_core/src/functions/database_family.rs`
   - `formal/lean/OxFunc/Functions/DatabaseFamily.lean`

## 3. Scope
In scope:
1. current-baseline promotion of the already-evidenced database family into the ordinary-backlog closure program,
2. native Excel replay for the admitted database slice through a dedicated `W065` manifest and runner,
3. closure-grade OxFunc runtime and dispatch evidence for all `12` rows,
4. Lean substrate/binding alignment confirmation for the existing database-family formal artifact,
5. snapshot/export promotion so these rows stop reading as hidden `catalog_only` backlog,
6. `W051` removal/update for the `12` rows.

Out of scope:
1. broader host-sensitive `W023` members (`ISFORMULA`, `SUBTOTAL`, `AGGREGATE`, `HYPERLINK`, `IMAGE`),
2. criteria formulas evaluated in full worksheet context beyond the already admitted W23 slice,
3. locale/version sweeps beyond the declared current reference baseline,
4. workbook-version sweeps and historical-version sweeps.

## 4. Output Contract
This packet must produce:
1. this workset spec,
2. `docs/function-lane/FUNCTION_SLICE_DATABASE_FAMILY_CURRENT_BASELINE_PROMOTION_PRELIM.md`,
3. `docs/function-lane/W65_SCENARIO_MANIFEST_SEED.csv`,
4. `docs/function-lane/W65_RUNTIME_REQUIREMENTS.md`,
5. `docs/function-lane/W65_SCOPE_RECONCILIATION.csv`,
6. `docs/function-lane/W65_EXECUTION_RECORD.md`,
7. `tools/w65-probe/run-w65-database-baseline.ps1`,
8. `.tmp/w65-database-results.csv`,
9. `tools/w44-probe/generate-w44-library-context-snapshot.ps1`,
10. refreshed downstream `W051` / snapshot / policy / worklist counters.

## 5. Completion Gate
`W065` may be reported complete only when:
1. every covered row has at least one explicit native Excel replay lane in `W65_SCENARIO_MANIFEST_SEED.csv`,
2. native Excel replay matches the seeded `W65` manifest for all declared lanes,
3. the targeted Rust tests for `database_family.rs` pass,
4. `cargo fmt --manifest-path crates/oxfunc_core/Cargo.toml --check` passes,
5. `lake build` passes without introducing new `W054` active gaps,
6. all `12` `W065` rows move out of `catalog_only` in the regenerated `W44` snapshot,
7. `W051` and downstream counts reconcile to the post-`W065` backlog (`34` hidden snapshot entries / `41` normalized execution rows).

## 6. Notes
1. `W065` is a promotion/reconciliation packet, not a fresh semantic-discovery packet: the database family already has closure-grade current-baseline evidence under `W023`.
2. The main current-phase gap before `W065` is publication drift: the `D*` family remained hidden ordinary backlog in the published snapshot even after the W23 evidence existed locally.

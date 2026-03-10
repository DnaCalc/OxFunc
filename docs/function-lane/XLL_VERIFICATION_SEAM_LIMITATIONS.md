# XLL Verification Seam Limitations

Status: `active`
Owner lane: `OxFunc`

## 1. Purpose
Record known limitations in the Rust XLL verification seam so they are not mistaken for function-semantic gaps and are not omitted from function verification records when relevant.

## 2. Scope Rule
1. These are external seam limitations, not acceptable justifications for incomplete core function semantics.
2. When any limitation below materially affects a function evidence claim, the affected function or packet verification record must cite it explicitly.

## 3. Current Limitation Set
1. Registration-flag mapping is only partially profile-derived:
   - ordinary `volatile_full` user-facing exports now emit `!`, but broader `!`, `$`, and `#` mapping remains under dedicated evidence work rather than full normal export generation.
2. Macro-type and caller-context behavior are only partially evidenced through the bridge:
   - admission and limited parity rows exist, but macro-required host behavior is not fully reproduced.
3. Reference-return and non-scalar payload lanes are still bounded in the bridge:
   - some functions can export or return shape/reference-like outcomes, but the bridge evidence does not yet demonstrate full Excel parity for all such lanes.
   - current lookup-family bridge replay distinguishes array-constant parity from reference-range limits:
     - `XMATCH` and `MATCH` array-constant parity is now replayed directly through `LOOKUP_XLL_BRIDGE_SCENARIO_MANIFEST_SEED.csv`,
     - `XLOOKUP` reference-resolved lookup arrays and blank-cell range lookup lanes are still tracked as explicit `known_divergence_not_equal` rows rather than parity claims.
4. Concurrency/thread-safety evidence is incomplete:
   - current probes show registration acceptance and scalar parity, not full scheduler or multithread execution behavior.
5. Host-entrypoint parity is contextual:
   - worksheet formula behavior, COM evaluate paths, and XLL invocation are related but not interchangeable evidence surfaces.
6. Post-evaluation format-hinting is not currently exercised through the XLL test seam:
   - caller-cell format mutation/application (for example `NOW` or `TODAY` entered into a `General` cell) is treated as an engine-surface responsibility above the core function result.
   - XLL verification may check value and recalc semantics for such functions, but absence of caller-format application in the XLL seam is not a function-semantic failure by itself.

## 4. Primary Evidence Records
1. `docs/function-lane/XLL_ADDIN_BRIDGE_EXECUTION_RECORD.md`
2. `docs/function-lane/XLL_ADDIN_BRIDGE_SHIM_CONTRACT_PRELIM.md`
3. `docs/function-lane/XLL_REGISTRATION_FLAG_EXECUTION_RECORD.md`

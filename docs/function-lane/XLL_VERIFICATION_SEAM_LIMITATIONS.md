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
   - current lookup-family bridge replay is green for the admitted manifest scope:
      - `XMATCH`, `MATCH`, and scalar `XLOOKUP` rows match directly through `LOOKUP_XLL_BRIDGE_SCENARIO_MANIFEST_SEED.csv`,
      - `XLOOKUP` reference-return address and range-composition rows also match in the current bridge scope.
   - current baseline replay is also green for the admitted `SUM` aggregate rows (`direct scalar`, `array literal`, `reference-derived`) through `XLL_ADDIN_BRIDGE_VALIDATION_SCENARIO_MANIFEST_SEED.csv`.
   - current baseline replay is also green for admitted `TEXTJOIN`, `DATE`, `OFFSET`, and `HSTACK` rows through `XLL_ADDIN_BRIDGE_VALIDATION_SCENARIO_MANIFEST_SEED.csv`.
   - remaining bounded lanes now include broader reference construction/info functions and general non-scalar payload coverage outside that manifest scope.
4. Concurrency/thread-safety evidence is incomplete:
   - current probes show registration acceptance and scalar parity, not full scheduler or multithread execution behavior.
5. Host-entrypoint parity is contextual:
   - worksheet formula behavior, COM evaluate paths, and XLL invocation are related but not interchangeable evidence surfaces.
6. Modern worksheet-only error classes are not yet fully preserved through the XLL seam:
   - legacy XLL error transport is narrower than the current worksheet error universe,
   - observed modern errors such as `#CALC!` may currently degrade to `#VALUE!` when returned through the XLL bridge.
7. Post-evaluation format-hinting is not currently exercised through the XLL test seam:
   - caller-cell format mutation/application (for example `NOW` or `TODAY` entered into a `General` cell) is treated as an engine-surface responsibility above the core function result.
   - XLL verification may check value and recalc semantics for such functions, but absence of caller-format application in the XLL seam is not a function-semantic failure by itself.
8. The bridge baseline can improve independently of core function closure:
   - `CLEAN` extra-C1 removal and `DATE(1900,1,0)` are now parity-closed through the rebuilt XLL baseline,
   - but that kind of bridge improvement does not change the rule that XLL limitations must be documented whenever they materially affect evidence claims.
9. Callable/lambda worksheet values are not yet transportable through the current XLL bridge:
   - helper-family worksheet surfaces such as `MAP`, `REDUCE`, `SCAN`, `BYROW`, `BYCOL`, `MAKEARRAY`, and workbook Defined Name callable invocation may be wired through core dispatch/export admission,
   - but the present XLL seam does not yet carry callable worksheet values or workbook Defined Name callable bindings into OxFunc in a way that can prove Excel parity end-to-end through the bridge,
   - so `W38` evidence remains a combination of native Excel worksheet replay and core/runtime dispatch tests rather than XLL bridge replay.

## 4. Primary Evidence Records
1. `docs/function-lane/XLL_ADDIN_BRIDGE_EXECUTION_RECORD.md`
2. `docs/function-lane/XLL_ADDIN_BRIDGE_SHIM_CONTRACT_PRELIM.md`
3. `docs/function-lane/XLL_REGISTRATION_FLAG_EXECUTION_RECORD.md`
4. `docs/function-lane/XLL_NIL_PROPAGATION_EXECUTION_RECORD.md`

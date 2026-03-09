# XLL Verification Seam Limitations

Status: `active`
Owner lane: `OxFunc`

## 1. Purpose
Record known limitations in the Rust XLL verification seam so they are not mistaken for function-semantic gaps and are not omitted from function verification records when relevant.

## 2. Scope Rule
1. These are external seam limitations, not acceptable justifications for incomplete core function semantics.
2. When any limitation below materially affects a function evidence claim, the affected function or packet verification record must cite it explicitly.

## 3. Current Limitation Set
1. Registration-flag mapping is not yet profile-derived:
   - `!`, `$`, and `#` remain under dedicated evidence work rather than normal export generation.
2. Macro-type and caller-context behavior are only partially evidenced through the bridge:
   - admission and limited parity rows exist, but macro-required host behavior is not fully reproduced.
3. Reference-return and non-scalar payload lanes are still bounded in the bridge:
   - some functions can export or return shape/reference-like outcomes, but the bridge evidence does not yet demonstrate full Excel parity for all such lanes.
4. Concurrency/thread-safety evidence is incomplete:
   - current probes show registration acceptance and scalar parity, not full scheduler or multithread execution behavior.
5. Host-entrypoint parity is contextual:
   - worksheet formula behavior, COM evaluate paths, and XLL invocation are related but not interchangeable evidence surfaces.

## 4. Primary Evidence Records
1. `docs/function-lane/XLL_ADDIN_BRIDGE_EXECUTION_RECORD.md`
2. `docs/function-lane/XLL_ADDIN_BRIDGE_SHIM_CONTRACT_PRELIM.md`
3. `docs/function-lane/XLL_REGISTRATION_FLAG_EXECUTION_RECORD.md`

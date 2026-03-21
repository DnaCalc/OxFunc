# W36 Execution Record - Provider Language Capability Baseline

Status: `complete`
Workset: `W036`

## 1. Purpose
Close the OxFunc-side seam for `TRANSLATE` without pretending cross-language translation is a local kernel.

## 2. Empirical Baseline
Native replay source:
1. `tools/w26-probe/run-w26-host-profile-provider-baseline.ps1`
2. `.tmp/w26-host-profile-provider-results.csv`
3. `docs/function-lane/W36_SCENARIO_MANIFEST_SEED.csv`

Pinned facts:
1. same-language translation is pass-through,
2. cross-language translation is provider-bound and `#BUSY!` on the current baseline.

## 3. OxFunc-Side Result
Artifacts:
1. `docs/function-lane/FUNCTION_SLICE_TRANSLATE_PROVIDER_LANGUAGE_CONTRACT_PRELIM.md`
2. `crates/oxfunc_core/src/host_info.rs`
3. `crates/oxfunc_core/src/functions/number_regex_translate_family.rs`
4. `formal/lean/OxFunc/HostInfoSeam.lean`
5. `formal/lean/OxFunc/Functions/NumberRegexTranslateFamily.lean`

Pinned seam:
1. `query_translate(request) -> TranslateProviderResult`
2. OxFunc keeps same-language passthrough local,
3. OxFunc maps provider outcomes to worksheet projections.

## 4. Verification
Ran:
1. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml number_regex_translate_family -- --nocapture`
2. `powershell -ExecutionPolicy Bypass -File tools/w26-probe/run-w26-host-profile-provider-baseline.ps1`
3. `lake build`

## 5. XLL Seam Limitation
1. The native Excel baseline proves `#BUSY!` for the seeded provider lane.
2. The current XLL bridge cannot preserve `#BUSY!` as a distinct Excel-C API error and currently downgrades it to `XLERR_VALUE`.
3. That is a bridge limitation, not an OxFunc-core semantic gap.

## 6. Completeness Axes
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`
5. open_lanes:
   - none in declared `W036` scope

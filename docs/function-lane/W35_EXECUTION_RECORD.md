# W35 Execution Record - NUMBERVALUE Locale-Default Profile Baseline

Status: `complete`
Workset: `W035`

## 1. Purpose
Close the OxFunc-side seam for the omitted-default behavior of `NUMBERVALUE`.

## 2. Empirical Baseline
Native replay source:
1. `tools/w26-probe/run-w26-host-profile-provider-baseline.ps1`
2. `.tmp/w26-host-profile-provider-results.csv`
3. `docs/function-lane/W35_SCENARIO_MANIFEST_SEED.csv`

Pinned facts:
1. omitted defaults reject the seeded English-style sample on the current host profile,
2. explicit separators succeed and stay OxFunc-local.

## 3. OxFunc-Side Result
Artifacts:
1. `docs/function-lane/FUNCTION_SLICE_NUMBERVALUE_LOCALE_DEFAULT_CONTRACT_PRELIM.md`
2. `crates/oxfunc_core/src/functions/number_regex_translate_family.rs`
3. `formal/lean/OxFunc/Functions/NumberRegexTranslateFamily.lean`

Pinned seam:
1. omitted separators come from `LocaleFormatContext.profile`,
2. explicit separators bypass locale-default dependence,
3. missing locale context on omitted-default lanes is a seam failure and currently projects `#VALUE!`.

## 4. Verification
Ran:
1. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml number_regex_translate_family -- --nocapture`
2. `powershell -ExecutionPolicy Bypass -File tools/w26-probe/run-w26-host-profile-provider-baseline.ps1`
3. `lake build`

## 5. Completeness Axes
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`
5. open_lanes:
   - none in declared `W035` scope


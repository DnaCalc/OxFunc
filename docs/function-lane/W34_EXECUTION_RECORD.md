# W34 Execution Record - Width Conversion Host/Profile Capability Baseline

Status: `complete`
Workset: `W034`

## 1. Purpose
Close the OxFunc-side seam for `ASC`, `DBCS`, and `JIS` without pretending they are ordinary pure text functions on the current baseline.

## 2. Empirical Baseline
Native replay source:
1. `tools/w26-probe/run-w26-host-profile-provider-baseline.ps1`
2. `.tmp/w26-host-profile-provider-results.csv`
3. `docs/function-lane/W34_SCENARIO_MANIFEST_SEED.csv`

Pinned current-host facts:
1. `ASC` is pass-through on the seeded current-host lane.
2. `DBCS` is pass-through on the seeded current-host lane.
3. `JIS` is unavailable on the seeded current-host lane.

## 3. OxFunc-Side Result
Artifacts:
1. `docs/function-lane/FUNCTION_SLICE_WIDTH_CONVERSION_HOST_PROFILE_CONTRACT_PRELIM.md`
2. `crates/oxfunc_core/src/host_info.rs`
3. `crates/oxfunc_core/src/functions/text_compat_locale_family.rs`
4. `formal/lean/OxFunc/HostInfoSeam.lean`
5. `formal/lean/OxFunc/Functions/TextCompatLocaleFamily.lean`

Pinned seam:
1. `query_width_conversion_mode(function) -> WidthConversionMode`
2. host/profile owns the active mode,
3. OxFunc owns the UTF-16 transform once the mode is known.

## 4. Verification
Ran:
1. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml text_compat_locale_family -- --nocapture`
2. `powershell -ExecutionPolicy Bypass -File tools/w26-probe/run-w26-host-profile-provider-baseline.ps1`
3. `lake build`

## 5. Completeness Axes
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`
5. open_lanes:
   - none in declared `W034` scope


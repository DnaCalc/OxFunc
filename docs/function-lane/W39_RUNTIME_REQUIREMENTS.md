# W39 Runtime Requirements

Status: `provisional`
Workset: `W39`

## 1. Native Probe
1. use desktop Excel via COM automation.
2. assign each seeded formula through `Formula2`.
3. serialize the observed result through the formula in the manifest itself, using `ARRAYTOTEXT(...,1)` for deterministic array output text.
4. compare observed `Text` against `expected_text`.

## 2. Local Runtime Verification
1. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml dynamic_array_reshape_family -- --nocapture`
2. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml all_catalog_functions_have_at_least_one_export -- --nocapture`
3. `cargo check --manifest-path tools/xll-addin/oxfunc_xll/Cargo.toml`
4. `powershell -ExecutionPolicy Bypass -File tools/xll-addin/sync-export-specs.ps1`
5. `lake build`

## 3. Expected Output Artifact
1. `.tmp/w39-dynamic-array-reshape-results.csv`

## 4. Notes
1. XLL publication-path limitations are not used to qualify this packet; native worksheet replay is the authoritative empirical surface.
2. locale/version sweeps remain out of current packet scope unless reopened later.

# W64 Runtime Requirements

1. Native Excel baseline runner:
   - `powershell -ExecutionPolicy Bypass -File tools/w64-probe/run-w64-financial-core-misc-baseline.ps1`
2. Targeted Rust coverage:
   - `cargo test --manifest-path crates/oxfunc_core/Cargo.toml cumulative_finance_family -- --nocapture`
   - `cargo test --manifest-path crates/oxfunc_core/Cargo.toml depreciation_family -- --nocapture`
   - `cargo test --manifest-path crates/oxfunc_core/Cargo.toml discount_bill_yearfrac_family -- --nocapture`
   - `cargo test --manifest-path crates/oxfunc_core/Cargo.toml dollar_fraction_family -- --nocapture`
   - `cargo test --manifest-path crates/oxfunc_core/Cargo.toml all_catalog_functions_have_at_least_one_export -- --nocapture`
3. Cross-surface build checks:
   - `cargo fmt --manifest-path crates/oxfunc_core/Cargo.toml --all --check`
   - `cargo check --manifest-path tools/xll-addin/oxfunc_xll/Cargo.toml`
   - `lake build`
4. Publication refresh:
   - `powershell -ExecutionPolicy Bypass -File tools/w44-probe/generate-w44-library-context-snapshot.ps1`

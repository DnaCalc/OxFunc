# W67 Runtime Requirements

1. Native Excel baseline runner:
   - `powershell -ExecutionPolicy Bypass -File tools/w67-probe/run-w67-math-matrix-rounding-baseline.ps1`
2. Targeted Rust coverage:
   - `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib ceiling_floor_family -- --nocapture`
   - `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib matrix_family -- --nocapture`
   - `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib sumproduct_family -- --nocapture`
   - `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib all_catalog_functions_have_at_least_one_export -- --nocapture`
3. Cross-surface build checks:
   - `cargo fmt --manifest-path crates/oxfunc_core/Cargo.toml --all --check`
   - `cargo check --manifest-path tools/xll-addin/oxfunc_xll/Cargo.toml`
   - `lake build`
4. Publication refresh:
   - `powershell -ExecutionPolicy Bypass -File tools/w44-probe/generate-w44-library-context-snapshot.ps1`

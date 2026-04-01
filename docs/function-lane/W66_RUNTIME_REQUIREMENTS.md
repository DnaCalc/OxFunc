# W66 Runtime Requirements

1. Native Excel baseline runner:
   - `powershell -ExecutionPolicy Bypass -File tools/w66-probe/run-w66-text-core-compat-baseline.ps1`
2. Targeted Rust coverage:
   - `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib text_scalar_misc -- --nocapture`
   - `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib concat_family -- --nocapture`
   - `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib text_slice_family -- --nocapture`
   - `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib text_search_replace_family -- --nocapture`
   - `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib text_b_compat_family -- --nocapture`
   - `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib text_unicode_fn -- --nocapture`
   - `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib all_catalog_functions_have_at_least_one_export -- --nocapture`
3. Cross-surface build checks:
   - `cargo fmt --manifest-path crates/oxfunc_core/Cargo.toml --all --check`
   - `cargo check --manifest-path tools/xll-addin/oxfunc_xll/Cargo.toml`
   - `lake build`
4. Publication refresh:
   - `powershell -ExecutionPolicy Bypass -File tools/w44-probe/generate-w44-library-context-snapshot.ps1`

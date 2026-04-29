# Locale/Format Seam Execution Record

Status: `complete-provisional`
Evidence ID: `W13-LOCALE-SHIM-20260314`

## 1. Purpose
Track closure of the local Rust/Lean locale-format seam used to ground `VALUE`, `TEXT`, `DOLLAR`, and `FIXED` against the current Excel host without pretending to own Excel's full formatting language.

## 2. Closed Scope
1. explicit seam choice: split parse/render facilities (`LocaleValueParser`, `FormatCodeEngine`, `FormatProfile`, `WorkbookDateSystem`)
2. concrete local shim profiles:
   - `en-US`
   - `current_excel_host`
3. admitted parser rows:
   - plain numeric text
   - grouped numeric text
   - current-host currency text
   - percent text
   - current-host ISO date text
   - explicit current-host slash-date rejection
   - explicit `en-US` slash-date acceptance seed
4. admitted render rows:
   - `TEXT(...,"0")`
   - `TEXT(...,"0.00")`
   - `TEXT(...,"0%")`
   - `TEXT(...,"yyyy-mm-dd")`
   - `DOLLAR`
   - `FIXED`
5. tester-XLL support wrappers for `GET.CELL`, `GET.DOCUMENT`, `GET.WORKBOOK`, `GET.WORKBOOK(1)`, and `GET.WORKSPACE` are implemented and parity-closed for the seeded lanes in `XLL_GET_INFO_EXECUTION_RECORD.md`.

## 3. Key Artifacts
1. Rust seam and function modules:
   - `crates/oxfunc_core/src/locale_format.rs`
   - `crates/oxfunc_core/src/functions/value_fn.rs`
   - `crates/oxfunc_core/src/functions/text_fn.rs`
   - `crates/oxfunc_core/src/functions/dollar_fn.rs`
   - `crates/oxfunc_core/src/functions/fixed_fn.rs`
2. Lean seam and bindings:
   - `formal/lean/OxFunc/LocaleFormat.lean`
   - `formal/lean/OxFunc/Functions/Value.lean`
   - `formal/lean/OxFunc/Functions/Text.lean`
   - `formal/lean/OxFunc/Functions/Dollar.lean`
   - `formal/lean/OxFunc/Functions/Fixed.lean`
3. empirical manifests:
   - `docs/function-lane/FORMAT_PROFILE_SEED.csv`
   - `docs/function-lane/VALUE_PARSE_SCENARIO_MANIFEST_SEED.csv`
   - `docs/function-lane/FORMAT_RENDER_SCENARIO_MANIFEST_SEED.csv`
4. design/doctrine notes:
   - `docs/function-lane/LOCALE_AND_FORMAT_INTERFACE_OPTIONS.md`
   - `docs/function-lane/FORMAT_SHIM_AND_GET_INFO_REFERENCE_NOTES.md`

## 4. Findings
1. the split parse/render seam is sufficient for the initial local host-grounded formatter/parser subset.
2. the current Excel host on this machine does not match simple `en-US` grouping/currency defaults; `GET.WORKSPACE(37)` grounding matters.
3. `VALUE` and `TEXT` both require parser and renderer awareness to behave honestly even in small seeded lanes.
4. `DOLLAR` and `FIXED` are renderer-dependent rather than pure numeric formatting helpers.
5. this seam is complete enough for continued OxFunc development and XLL-based empirical testing, but it is not a claim that the full Excel format-code language has been implemented.

## 5. Verification
1. `cargo test -p oxfunc_core`
2. `cargo check --manifest-path tools/xll-addin/oxfunc_xll/Cargo.toml`
3. `lake build`
4. XLL GET-info probe recorded separately under `W9-XLL-GETINFO-20260314`; that wrapper lane is now seeded and closed and continues to support the local parser/renderer seam

## 6. Remaining Open Work
1. full Excel format-code language and locale sweep remain open.
2. `VALUE`, `TEXT`, `DOLLAR`, and `FIXED` are `function-phase-complete` for the current admitted reference-baseline slice; broader locale/version expansion remains orthogonal validation work.
3. future ownership of the full formatting language remains with OxFml/FEC rather than per-function OxFunc kernels.
4. `W082` has since removed the old ordinary `en_us_context()` /
   `current_excel_host_context()` constructors from `oxfunc_core`; the local
   parser/formatter survives only as explicit test support while downstream
   caller migration remains open.
5. The OxFunc XLL host bridge no longer imports a removed OxFunc-core
   convenience context; it supplies an explicit caller-owned
   `LocaleFormatContext` and delegates parser/formatter behavior to Excel via
   `xlfEvaluate`.




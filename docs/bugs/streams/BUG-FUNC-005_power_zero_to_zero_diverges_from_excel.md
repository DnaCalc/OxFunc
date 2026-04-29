# BUG-FUNC-005: POWER zero-to-zero diverges from Excel

## Summary
- **Bug id**: `BUG-FUNC-005`
- **Opened**: 2026-04-08
- **Status**: `closed`
- **Owner workset**: `W078`

## Fresh Confirmation Note
Fresh Excel COM replay on 2026-04-29 confirmed that the active installed
baseline still returns `#NUM!` for both `=0^0` and `=POWER(0,0)`. The local
Rust and Lean correction is landed on committed ref
`5d54d7f4ab2cdde6458272292d15ae1b317a0fef`, so this stream is closed as a
fixed historical OxFunc bug rather than a current known deviation.

## Source Refs
- **Reported against ref**: `7989fafaef703f15f2bfbdded323c03345da1072`
- **Reproduced on ref**: `7989fafaef703f15f2bfbdded323c03345da1072`
- **Introduced in ref**: `unknown`
- **Fixed in ref**: `5d54d7f4ab2cdde6458272292d15ae1b317a0fef`
- **Ref notes**: Intake pinned the original current working ref with
  `git rev-parse HEAD`. Live Excel COM replay on 2026-04-08 reproduced the
  domain lane directly against the installed baseline. Fresh Excel COM replay on
  2026-04-29 confirmed the same `#NUM!` rule on Excel 16.0 build 19929, and the
  local runtime/formal correction is landed on the fixed ref above.

## Ownership And Root Cause
- **Ownership class**: `OxFunc-owned bug`
- **Root cause class**: `test_gap`
- **Root cause summary**: the shared `power_kernel` zero-exponent fast path
  returned `1` for every `power == 0` case, while the admitted empirical packet
  never pinned the `0^0` domain row. That let both Rust and Lean keep an
  incorrect zero-to-zero publication lane even after the earlier integer-
  exponent publication repair.

## Why Did We Get This Wrong?
- **Spec already correct and code was wrong?**: `yes`
- **Spec vague or missing?**: `yes`
- **Code once correct and later regressed?**: `no`
- **Likely introduced in ref**: `unknown`
- **Explanation**: the repo-wide target is empirical Excel parity, but the
  concrete local evidence set for `POWER` covered integer-publication drift and
  negative/real-domain errors without ever exercising `0^0`. The zero-exponent
  shortcut therefore survived as an unchallenged implementation assumption.

## Reproduction
1. Live Excel on 2026-04-08 observed:
   - `=0^0 -> #NUM!`
   - `=POWER(0,0) -> #NUM!`
   - `=0^-1 -> #DIV/0!`
   - `=0^1 -> 0`
   - `=1^0 -> 1`
2. Actual pre-fix OxFunc behavior:
   - `OP_POWER(0,0) -> 1`
   - `POWER(0,0) -> 1`
3. Shared local cause:
   - `power_kernel` returned `1` for every integer zero exponent before
     checking the zero-base domain lane.

## Spec And Contract Relationship
- **Spec references**:
  1. `docs/function-lane/W45_WAVEA_OPERATOR_ARITHMETIC_SCENARIO_MANIFEST_SEED.csv`
  2. `docs/function-lane/W45_EXECUTION_RECORD.md`
  3. `docs/function-lane/W53_NUMERIC_FORENSICS_20260326.md`
  4. `formal/lean/OxFunc/Functions/PowerFn.lean`
- **Spec state at intake**: `vague`
- **Notes**: the admitted packet covered `POWER` publication and some power
  domain rows, but it did not pin the zero-to-zero lane explicitly, so the
  current truth surfaces overclaimed the function/operator family.

## Investigation Log
1. 2026-04-08: live Excel COM replay pinned `=0^0` and `=POWER(0,0)` as
   `#NUM!`.
2. 2026-04-08: confirmed `power_kernel` is shared by `POWER`, `OP_POWER`, the
   `q_binary_number` surface, and the financial growth helper.
3. 2026-04-08: confirmed the local Rust implementation returns `1` for every
   zero exponent before checking the zero-base domain lane.
4. 2026-04-08: confirmed the Lean executable model also treated `(0,0)` as a
   successful numeric lane.
5. 2026-04-08: opened bounded owner `W078` and bead `oxf-x3fk`.
6. 2026-04-08: corrected the shared Rust and Lean zero-to-zero lane, widened
   operator/function tests, added an OxFml-adapter fixture for `POWER(0,0)`,
   and added a W45 Wave A empirical row for `0^0`.
7. 2026-04-29: fresh Excel COM replay confirmed `=0^0 -> #NUM!` and
   `=POWER(0,0) -> #NUM!` on Excel 16.0 build 19929; focused local
   `power_fn` tests pass on the landed correction.

## Similar-Risk Scan
### Adjacent families to check
1. `POWER`
2. `OP_POWER`
3. `eval_surface_q_binary_number`
4. `financial_time_value_family::growth`

### Check method
1. searched all local `power_kernel` callsites,
2. replayed live Excel for `0^0`, `POWER(0,0)`, `0^-1`, `0^1`, and `1^0`,
3. added focused Rust/unit/adapter-entry tests and a Wave A native replay row,
4. checked the Lean executable model for the same zero-to-zero lane.

### Results
1. `POWER` and `OP_POWER` both inherited the same wrong zero-to-zero result.
2. `eval_surface_q_binary_number` also inherited that same shared kernel lane,
   so entrypoint-level validation had to be widened.
3. `financial_time_value_family::growth` is not a new open bug from this lane
   because it rejects `base <= 0` before calling `power_kernel`.
4. no additional bug stream was needed beyond this canonical packet.

### Follow-on Openings
1. `W078`

## Fix Plan
1. reject `number == 0 && power == 0` with `WorksheetErrorCode::Num` before the
   integer-publication fast path,
2. align the Lean executable model to the same zero-to-zero domain rule,
3. add focused Rust, surface-dispatch, adapter-fixture, and W45 Wave A
   evidence rows for the corrected lane,
4. reopen current-gap truth so `POWER` is not silently left in the supported
   surface while the fix is only on the working tree.

## Validation
1. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib power_fn -- --nocapture`
2. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib operator_arithmetic_family -- --nocapture`
3. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib surface_dispatch -- --nocapture`
4. attempted `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --test oxfml_seam_integration -- --nocapture`, but it is still blocked by the pre-existing `oxfml_core::substrate` path mismatch in the current local seam-test harness
5. `powershell -ExecutionPolicy Bypass -File tools/w45-probe/run-w45-wavea-operator-arithmetic-baseline.ps1`
6. `lake build OxFunc.Functions.PowerFn` (run from `formal/lean`)
7. 2026-04-29 fresh Excel COM replay:
   - Excel version/build: `16.0` / `19929`
   - `=0^0 -> #NUM!`
   - `=POWER(0,0) -> #NUM!`
   - `=0^-1 -> #DIV/0!`
   - `=0^1 -> 0`
   - `=1^0 -> 1`
8. 2026-04-29 replayed
   `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib power_fn -- --nocapture`
   with 4 passed tests.

## Linked Reports
1. `BUGREP-FUNC-007`

## Evidence
1. `crates/oxfunc_core/src/functions/power_fn.rs`
2. `crates/oxfunc_core/src/functions/operator_arithmetic_family.rs`
3. `crates/oxfunc_core/src/functions/surface_dispatch.rs`
4. `crates/oxfunc_core/tests/fixtures/oxfunc_adapter_function_corpus.json`
5. `docs/function-lane/W45_WAVEA_OPERATOR_ARITHMETIC_SCENARIO_MANIFEST_SEED.csv`
6. `formal/lean/OxFunc/Functions/PowerFn.lean`

## Closure Checklist
- [x] fix landed or non-OxFunc ownership recorded
- [x] validation recorded
- [x] root cause recorded
- [x] similar-risk scan recorded
- [x] spec/matrix/contract updated if required
- [x] handoff filed if required
- [x] linked reports updated

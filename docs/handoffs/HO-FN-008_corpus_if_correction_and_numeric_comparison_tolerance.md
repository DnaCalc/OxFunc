# HO-FN-008 - Corpus IF correction and numeric comparison tolerance

## 1. Direction
- **From**: `OxFunc`
- **To**: `OxFml`
- **Filed date**: `2026-04-08`
- **Source workset**: `W077`
- **Related inbound handoff**: `HANDOFF-OXFUNC-003`

## 2. Purpose
Reply to the corpus follow-on packet with the corrected local findings:
`IF("",1,2)` is not a live OxFunc bug on current head, while near-equality
numeric comparison tolerance is a real OxFunc semantic lane that extends beyond
ordinary operators into criteria/database selection and `SWITCH`.

## 3. Packet Summary
Local Excel replay on 2026-04-08 yields this split:
1. `=IF("",1,2)` and `=IFS("",1,TRUE,2)` both return `#VALUE!`; current OxFunc
   already matches that and the inbound `IF` claim is closed locally as
   `BUGREP-FUNC-004` with no canonical OxFunc bug stream.
2. Ordinary compare operators are tolerant on tested near-equality cases such as
   `=0.1+0.2=0.3`.
3. Criteria/database numeric criteria matching and `SWITCH` share that tolerant
   lane.
4. `MATCH`, `XMATCH`, and `DELTA` exact-match paths remain exact on the tested
   near-equality cases.
5. The stronger arithmetic-generated boundary pair
   `((123456789012345*10)+5)/1E25` versus `((123456789012345*10)+4)/1E25`
   stays on the same family split:
   - operators, criteria/database, and `SWITCH` collapse it as equal,
   - `MATCH`, `XMATCH`, and `DELTA` remain exact.
6. OxFunc now routes the tolerant families through one shared local helper while
   leaving the exact-match contrast families unchanged; the current local helper
   follows the empirically pinned truncation-style 15-significant-digit lane
   rather than round-to-nearest.

## 4. Concrete Local Outcomes
1. the inbound `IF` claim is now corrected locally and pinned by tests,
2. operator compare, criteria/database selection, and `SWITCH` use one shared
   Excel-style numeric comparison helper,
3. `MATCH`, `XMATCH`, and `DELTA` are pinned explicitly as exact-match contrast
   lanes,
4. the local floating-point and W45 empirical manifests now carry replayable
   near-equality scenarios, including the arithmetic-generated 15-digit
   boundary rows,
5. `W051` is reopened honestly for the affected criteria/database/`SWITCH`
   families until the packet is landed on a committed ref and acknowledged
   downstream.

## 5. Validation Run
1. `cargo fmt --manifest-path crates/oxfunc_core/Cargo.toml`
2. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib if_fn -- --nocapture`
3. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib choose_ifs_family -- --nocapture`
4. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib operator_compare_concat_family -- --nocapture`
5. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib criteria_family -- --nocapture`
6. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib database_family -- --nocapture`
7. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib misc_switch_info_family -- --nocapture`
8. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib match_fn -- --nocapture`
9. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib xmatch -- --nocapture`
10. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib delta_fn -- --nocapture`
11. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib surface_dispatch -- --nocapture`
12. `powershell -ExecutionPolicy Bypass -File tools/w45-probe/run-w45-waveb-operator-compare-concat-baseline.ps1`
13. `powershell -ExecutionPolicy Bypass -File tools/fp-probe/run-fp-excel-baseline.ps1 -Manifest docs/function-lane/FLOATING_POINT_SCENARIO_MANIFEST_SEED.csv -Out .tmp/fp-results-excel-e.csv -Lanes FP-E`
14. `powershell -ExecutionPolicy Bypass -File scripts/check-worksets.ps1`

All listed validation passed on the current working tree.

## 6. OxFml Ask
1. correct the upstream corpus read for the `IF` empty-text lane,
2. consume the broader numeric comparison family split rather than treating this
   as an ordinary-operators-only change,
3. keep `MATCH` / `XMATCH` / `DELTA` exact-match logic separate from the
   tolerant operator/criteria/`SWITCH` lane,
4. align any downstream compare helper to the stronger arithmetic-generated
   boundary rows rather than a round-to-nearest 15-digit surrogate,
5. acknowledge the reply packet once the landed OxFunc ref is available.

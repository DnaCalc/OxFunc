# W24 Batch 10 Execution Record - Regex Triad

Status: `complete-provisional` (historical; administratively qualified by uncheckpointed `BUG-FUNC-041` repair)
Workset: `W24`
Evidence ID: `W24-B10-REGEX-20260318`

Current qualification (`2026-06-26`): the original W24 packet remains the replay record for
the bounded March 2026 regex slice, and `BUG-FUNC-041` later found an escape-admission gap.
The 2026-06-26 local repair for the expanded admitted-escape slice is live-Excel signed off
with `40/40` exact typed matches; the remaining qualification is checkpoint/landing.

## 1. Purpose
Record the pure regex trio closure packet inside the `W24` ordinary mega-batch.

## 2. Scope
1. close `REGEXEXTRACT`, `REGEXREPLACE`, and `REGEXTEST` for the admitted current reference baseline,
2. split the pure regex trio from the mixed `NUMBERVALUE` / `TRANSLATE` family after live host replay showed those remaining members are seam-sensitive,
3. bind the runtime and Lean substrate to a replayable native worksheet packet.

## 3. Completeness Axes
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`
5. open_lanes:
   - broader Excel regex syntax remains outside this packet.
   - `NUMBERVALUE` and `TRANSLATE` are intentionally excluded and separately blocked.

## 4. Executed Scope
Artifacts created or updated:
1. `docs/function-lane/FUNCTION_SLICE_REGEX_TRIAD_CONTRACT_PRELIM.md`
2. `docs/function-lane/W24_BATCH10_REGEX_SCENARIO_MANIFEST_SEED.csv`
3. `docs/function-lane/W24_BATCH10_REGEX_RUNTIME_REQUIREMENTS.md`
4. `docs/function-lane/W24_BATCH10_REGEX_EXECUTION_RECORD.md`
5. `tools/w24-probe/run-w24-batch10-regex-baseline.ps1`
6. `docs/function-lane/EXCEL_FUNCTION_DEFINITION_PRELIM_CONFORMANCE.csv`
7. `docs/function-lane/FUNCTION_LANE_EVIDENCE_ID_REGISTRY.md`
8. `docs/function-lane/W24_ORDINARY_FUNCTIONS_MEGA_BATCH_CHECKLIST.csv`
9. `docs/function-lane/W16_BATCH75_NUMBER_REGEX_TRANSLATE_NOTES.md`

## 5. Empirical Findings
From `.tmp/w24-batch10-regex-results.csv`:
1. `REGEXEXTRACT("abc123def","\d+")` -> `123`
2. `REGEXEXTRACT("abc123def","[a-z]+")` -> `abc`
3. `REGEXEXTRACT("abc123def","XYZ")` -> `#N/A`
4. `REGEXREPLACE("abc123def","\d+","X")` -> `abcXdef`
5. `REGEXREPLACE("abc123def","[a-z]+","X",2)` -> `abc123X`
6. `REGEXTEST("abc123def","[A-Z]+",1)` -> `TRUE`
7. `REGEXTEST("abc123def","[A-Z]+",0)` -> `FALSE`

## 6. Implementation Result
1. The existing local regex kernel already matched the admitted current-baseline worksheet packet for the triad.
2. The packet confirmed that the current ASCII-only case-sensitivity flag behavior matches the admitted local implementation.
3. The batch note is now split honestly: pure regex stays in `W024`, while `NUMBERVALUE` locale defaults and `TRANSLATE` provider behavior are blocked separately.

## 7. Verification Runs
1. `powershell -ExecutionPolicy Bypass -File tools/w24-probe/run-w24-batch10-regex-baseline.ps1`
2. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml number_regex_translate_family`
3. `cargo check --manifest-path tools/xll-addin/oxfunc_xll/Cargo.toml`
4. `lake build`

## 8. Standing
1. Historical W24 standing: `REGEXEXTRACT`, `REGEXREPLACE`, and `REGEXTEST` were treated as function-phase-complete for the bounded admitted current reference baseline captured by this packet.
2. The closure is bounded to the admitted regex-triad slice above.
3. `W024` continues with the next unblocked family after removing the mixed-family seam confusion.
4. Current standing is qualified by `BUG-FUNC-041`: the expanded admitted escape slice is live-Excel signed off locally and remains open pending checkpoint/landing.

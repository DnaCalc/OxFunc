# W24 Batch 03 Execution Record - Text Delimiter Family

Status: `complete-provisional`
Workset: `W24`
Evidence ID: `W24-B03-TEXT-DELIM-20260318`

## 1. Purpose
Record the text delimiter family closure packet inside the `W24` ordinary mega-batch.

## 2. Scope
1. close `TEXTAFTER` and `TEXTBEFORE` for the admitted current reference baseline,
2. bind the existing runtime and Lean substrate to a replayable native worksheet packet,
3. make the scalar delimiter and fallback boundary explicit.

## 3. Completeness Axes
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`
5. open_lanes:
   - broader locale/version sweeps remain orthogonal validation work.

## 4. Executed Scope
Artifacts created or updated:
1. `docs/function-lane/FUNCTION_SLICE_TEXT_DELIM_FAMILY_CONTRACT_PRELIM.md`
2. `docs/function-lane/W24_BATCH03_TEXT_DELIM_SCENARIO_MANIFEST_SEED.csv`
3. `docs/function-lane/W24_BATCH03_TEXT_DELIM_RUNTIME_REQUIREMENTS.md`
4. `docs/function-lane/W24_BATCH03_TEXT_DELIM_EXECUTION_RECORD.md`
5. `tools/w24-probe/run-w24-batch03-text-delim-baseline.ps1`
6. `docs/function-lane/EXCEL_FUNCTION_DEFINITION_PRELIM_CONFORMANCE.csv`
7. `docs/function-lane/FUNCTION_LANE_EVIDENCE_ID_REGISTRY.md`
8. `docs/function-lane/W24_ORDINARY_FUNCTIONS_MEGA_BATCH_CHECKLIST.csv`
9. `crates/oxfunc_core/src/functions/text_delim_family.rs`
10. `docs/function-lane/W16_BATCH61_TEXT_DELIM_NOTES.md`

## 5. Empirical Findings
From `.tmp/w24-batch03-text-delim-results.csv`:
1. `TEXTAFTER("One,Two,Three", ",", 1)` -> `Two,Three`
2. `TEXTAFTER("One,Two,Three", ",", -1)` -> `Three`
3. `TEXTBEFORE("One,Two,Three", ",", 1)` -> `One`
4. `TEXTBEFORE("One,Two,Three", ",", -1)` -> `One,Two`
5. `TEXTAFTER("Socrates", " ", 1, 0, 1, "fallback")` -> `""`
6. `TEXTBEFORE("Socrates", " ", 1, 0, 1, "fallback")` -> `Socrates`
7. `TEXTAFTER("abc", "/", 1, 0, 0, 7)` -> `7`
8. `TEXTAFTER("ABcdEF", "CD", 1, 1, 0, "fallback")` -> `EF`
9. `TEXTBEFORE("abc", "b", 0)` -> `#VALUE!`
10. `TEXTAFTER("abc", "b", 1, 2, 0, "fallback")` -> `#VALUE!`
11. `TEXTAFTER("abc", "b", 1, 0, 2, "fallback")` -> `#VALUE!`

## 6. Implementation Result
1. `TEXTAFTER` and `TEXTBEFORE` were already wired through dispatch, export, and Lean surfaces.
2. The current packet exposed and corrected a real semantic bug: the runtime had the optional arguments ordered incorrectly.
3. The runtime now matches the real Excel syntax `(..., [instance_num], [match_mode], [match_end], [if_not_found])`.

## 7. Verification Runs
1. `powershell -ExecutionPolicy Bypass -File tools/w24-probe/run-w24-batch03-text-delim-baseline.ps1`
2. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml text_delim_family`
3. `cargo check --manifest-path tools/xll-addin/oxfunc_xll/Cargo.toml`
4. `lake build`
5. `cargo fmt --manifest-path crates/oxfunc_core/Cargo.toml`

## 8. Standing
1. `TEXTAFTER` and `TEXTBEFORE` are now function-phase-complete for the admitted current reference baseline.
2. The closure is bounded to the observed scalar slice and current ASCII-only case-insensitive behavior.
3. The next work in `W24` continues with the remaining Wave 1 families.

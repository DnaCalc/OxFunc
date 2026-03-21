# W41 Execution Record - External Data, Add-In, and Cube Functions

Status: `in_progress`
Workset: `W41`
Evidence IDs:
1. `W41-WEBTEXTXML-BL-20260321`

## 1. Purpose
1. convert the local `ENCODEURL` / `FILTERXML` slice into a real evidenced sub-packet,
2. narrow the true remaining `W41` scope to the external/provider/cube/add-in lanes that still need seams,
3. keep the packet honest about the difference between local web-text utilities and live provider surfaces.

## 2. Scope
Artifacts created or updated in this pass:
1. `docs/worksets/W041_EXTERNAL_DATA_PROVIDER_AND_CUBE_FUNCTIONS.md`
2. `docs/function-lane/W41_EXTERNAL_DATA_PROVIDER_AND_CUBE_INVENTORY.csv`
3. `docs/function-lane/FUNCTION_SLICE_WEB_TEXT_XML_LOCAL_FUNCTIONS_CONTRACT_PRELIM.md`
4. `docs/function-lane/W41_WEB_TEXT_XML_SCENARIO_MANIFEST_SEED.csv`
5. `docs/function-lane/W41_WEB_TEXT_XML_RUNTIME_REQUIREMENTS.md`
6. `docs/function-lane/W41_EXECUTION_RECORD.md`
7. `tools/w41-probe/run-w41-web-text-xml-baseline.ps1`
8. `.tmp/w41-web-text-xml-results.csv`
9. `crates/oxfunc_core/src/functions/web_text_xml_family.rs`
10. `formal/lean/OxFunc/Functions/WebTextXmlFamily.lean`

## 3. Completeness Axes
1. execution_state: `in_progress`
2. scope_completeness: `scope_partial`
3. target_completeness: `target_partial`
4. integration_completeness: `partial`
5. open_lanes:
   - cube functions remain untouched
   - `WEBSERVICE` and `STOCKHISTORY` still need provider/service seams
   - `COPILOT` is now classified into the packet but still needs add-in/feature gating semantics

## 4. Packet Result
1. `ENCODEURL` now has a bounded local runtime implementation and tests.
2. `FILTERXML` now has a bounded local runtime implementation for the admitted node-set-only XPath slice.
3. native Excel replay now pins the current-baseline local slice for both functions and ran green for all `8` seeded rows in `.tmp/w41-web-text-xml-results.csv`.
4. the packet scope is narrower and cleaner: the unresolved remainder is now the true provider/cube/add-in subset.

## 5. Main Findings
1. `ENCODEURL` is an ordinary local scalar-to-text transform on the current baseline.
2. `FILTERXML` is also locally implementable on the current baseline, but only honestly on a node-set XPath slice.
3. scalar XPath result types such as `string(...)` are not admitted on the current baseline seeded slice and surface `#VALUE!`.
4. `COPILOT` looks like an absent feature/add-in lane on the current installed baseline, not a local kernel function.

## 6. Verification Runs
1. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml web_text_xml_family -- --nocapture`
2. `powershell -ExecutionPolicy Bypass -File tools/w41-probe/run-w41-web-text-xml-baseline.ps1`

## 7. Standing
1. `W41` is still partial.
2. the local web-text/xml pair is now evidenced rather than merely listed in the packet inventory.

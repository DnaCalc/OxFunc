# WORKSET - TUX1000 Ten-Function Mixed Seams (W10)

## 1. Purpose
Run one focused ten-function packet that exercises both non-interesting and adventurous seams under the current OxFunc architecture and the Rust-only XLL bridge/codegen flow.

Primary intent:
1. stress-test function classification and adapter layering at breadth,
2. validate whether current declarative registration/coercion/deref patterns scale,
3. capture where function-specific behavior must remain inside function surfaces versus shared infrastructure.

## 2. Position and Dependencies
Program position:
1. post-kickoff extension workset (`W10`), after W1..W7, W8.1, and W9.

Dependencies:
1. W3 value-universe taxonomy.
2. W4 coercion + reference-resolution seam decisions.
3. W5 ABS layering (`kernel` vs adapter vs pre-adapter).
4. W6 lookup-function quirk posture and error-lane handling.
5. W9 Rust-only XLL bridge and core-driven export codegen.

## 3. Function Scope
Baseline insight set:
1. `SUM`
2. `IF`
3. `INDEX`
4. `MATCH`
5. `ISNUMBER`

Adventurous set:
1. `NOW`
2. `XLOOKUP` (including value/range/reference-return lanes)
3. `INDIRECT`
4. `SEQUENCE`
5. `OP_ADD` (`+` operator-as-function lane)

## 4. Seams to Force
For each function, explicitly classify and test:
1. value admission + coercion boundary.
2. reference policy:
   - pre-adapter deref allowed/required,
   - adapter-visible refs required/optional.
3. array-lift/spill behavior and shape policy.
4. error propagation policy (including elementwise vs immediate fail).
5. determinism/volatility/thread-safety classification.
6. FEC dependency profiles:
   - adapter-level,
   - surface-level.
7. XLL U-vs-Q registration suitability and expected differences.

## 5. Execution Slices
### S1 - Deterministic Value/Coercion Pack
Functions:
1. `ISNUMBER`
2. `OP_ADD`
3. `SUM`

Goal:
1. lock coercion and aggregate/elementwise policy under shared infrastructure.

### S2 - Control + Indexing Pack
Functions:
1. `IF`
2. `INDEX`
3. `MATCH`

Goal:
1. settle lazy-evaluation expectations, index/reference return behavior, and lookup baseline policy.

### S3 - Dynamic/Volatile Pack
Functions:
1. `NOW`
2. `SEQUENCE`

Goal:
1. close volatility/classification and spill-shape obligations.

### S4 - Indirection + Rich Lookup Pack
Functions:
1. `INDIRECT`
2. `XLOOKUP`

Goal:
1. close host-context and reference/provenance-sensitive lanes and map required FEC capabilities.

## 6. Deliverables
1. one function-slice contract doc per function (`docs/function-lane/FUNCTION_SLICE_*_CONTRACT_PRELIM.md`).
2. one scenario manifest pack per function family (seed + expanded rows).
3. runtime implementations/surfaces in `crates/oxfunc_core/src/functions/*`.
4. Lean formalization slices for admitted kernels/surfaces with explicit theorem inventory.
5. XLL export rows added through core `xll_export_specs` where appropriate (U/Q variants declared explicitly).
6. execution records and evidence IDs for each function pack.
7. correlation-ledger updates for all ten function IDs.

## 7. Gate Model
### G1 - Classification Closure
Pass when:
1. each function has explicit classification rows (determinism, volatility, thread safety, host interaction, FEC profiles, preparation/coercion/kernel signatures).

### G2 - Runtime/Formal Pairing Closure
Pass when:
1. each function has declared runtime artifact and corresponding formal artifact scope,
2. unresolved formal lanes are explicit and bounded.

### G3 - Empirical Closure
Pass when:
1. scenario packs are replayable,
2. mismatches are classified (`spec gap`, `policy ambiguity`, `implementation defect`, `environment variability`),
3. intentional failures are explicitly marked as non-regressions.

### G4 - XLL Export Closure
Pass when:
1. each function has an explicit decision for U export, Q export, or both,
2. export declarations are generated from core `xll_export_specs`,
3. side-by-side workbook replays exist for selected critical rows per function.

### G5 - Promotion Readiness
Pass when:
1. all ten functions reach full empirically aligned Excel semantics for the declared version axes,
2. no known semantic gaps remain except external XLL verification-seam limits,
3. outstanding policy questions are either closed or shown to be non-semantic integration issues.

## 8. Status
Execution state:
1. `in_progress`.

Claim confidence:
1. `provisional` (useful scaffolding and empirical replay landed, but multiple functions remain semantically incomplete).

Assurance maturity:
1. `exercised`.

## 9. Progress Notes
1. Runtime slices implemented in `crates/oxfunc_core/src/functions/*` for all ten functions.
2. Lean slices implemented in `formal/lean/OxFunc/Functions/*` for all ten functions.
3. Function contracts published:
   - `docs/function-lane/FUNCTION_SLICE_SUM_CONTRACT_PRELIM.md`
   - `docs/function-lane/FUNCTION_SLICE_IF_CONTRACT_PRELIM.md`
   - `docs/function-lane/FUNCTION_SLICE_INDEX_CONTRACT_PRELIM.md`
   - `docs/function-lane/FUNCTION_SLICE_MATCH_CONTRACT_PRELIM.md`
   - `docs/function-lane/FUNCTION_SLICE_ISNUMBER_CONTRACT_PRELIM.md`
   - `docs/function-lane/FUNCTION_SLICE_NOW_CONTRACT_PRELIM.md`
   - `docs/function-lane/FUNCTION_SLICE_XLOOKUP_CONTRACT_PRELIM.md`
   - `docs/function-lane/FUNCTION_SLICE_INDIRECT_CONTRACT_PRELIM.md`
   - `docs/function-lane/FUNCTION_SLICE_SEQUENCE_CONTRACT_PRELIM.md`
   - `docs/function-lane/FUNCTION_SLICE_OP_ADD_CONTRACT_PRELIM.md`
4. Scenario packs added and expanded:
   - `docs/function-lane/W10_S1_SCENARIO_MANIFEST_SEED.csv`
   - `docs/function-lane/W10_S2_SCENARIO_MANIFEST_SEED.csv`
   - `docs/function-lane/W10_S3_SCENARIO_MANIFEST_SEED.csv`
   - `docs/function-lane/W10_S4_SCENARIO_MANIFEST_SEED.csv` (includes `XLOOKUP` reference-return address and range-composition rows)
   - current expansion adds `IF` omitted-false and `ISNUMBER` error/reference rows.
5. Execution/evidence/correlation closure:
   - `docs/function-lane/W10_EXECUTION_RECORD.md`
   - `docs/function-lane/FUNCTION_LANE_EVIDENCE_ID_REGISTRY.md` (`W10-TENMIX-SEED-20260308`)
   - `docs/function-lane/FUNCTION_SLICE_CORRELATION_LEDGER.csv` (10 new rows)
6. Side-note ledger maintained:
   - `docs/function-lane/W10_PROFILE_SYSTEM_SIDE_NOTES.md`
7. Replay tooling promoted from ad hoc commands to dedicated scripts:
   - `tools/w10-probe/run-w10-suite.ps1`
   - `tools/w10-probe/analyze-w10-results.ps1`
   - `tools/w10-probe/new-w10-compat-template.ps1`
8. dual-run Excel replay reran green across `56` observed rows with `expectation_mismatched=0`.
9. W10 remains open because `SUM`, `INDEX`, `MATCH`, `XLOOKUP`, `INDIRECT`, and `SEQUENCE` still carry known Excel-semantic gaps.
10. Individual W10 slices now considered `function-phase-complete` for the current implementation phase:
   - `IF`
   - `ISNUMBER`
   - `NOW`

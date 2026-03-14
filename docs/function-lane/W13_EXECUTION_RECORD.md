# W13 Execution Record

Status: `complete-provisional`
Workset: `W13`
Evidence ID: `W13-NONLOCALE-BL-20260314`; `W13-LOCALE-SHIM-20260314`

## 1. Purpose
Track W13 execution status, artifacts, and gate closure for the deceptively simple boundary-functions packet.

## 2. Scope
1. functions: `SIN`, `ASIN`, `N`, `T`, `TYPE`, `VALUE`, `ROW`, `COLUMN`, `TEXT`, `DOLLAR`, `FIXED`
2. pressure-test the non-interesting-function parity hypothesis on a mixed batch of scalar numeric, type-classification, caller-context, and locale/format functions

## 3. Completeness Axes
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `partial`
5. open_lanes:
   - broader locale/alternate-version sweeps remain orthogonal validation-phase work.
   - XLL seam limitations continue to be tracked separately where they qualify worksheet-parity claims.

## 4. Current Findings
1. direct worksheet probes and packet replay show that `SIN`, `ASIN`, `N`, `T`, `TYPE`, `ROW`, and `COLUMN` close cleanly on the current substrate:
   - `SIN("1")` admits numeric text coercion
   - `SIN("asd")` -> `#VALUE!`
   - `ASIN(2)` -> `#NUM!`
   - `N({1,"x"})` spills `{1,0}`
   - `TYPE(A1:A2)` -> `64`
   - `ROW(A1:B2)` spills vertically
   - `COLUMN(A1:B2)` spills horizontally
2. the locale/format seam is now explicit in Rust and Lean for the admitted current-host plus `en-US` slice:
   - current-host `VALUE` rows include grouped numeric, currency, percent, ISO date, and slash-date rejection lanes
   - current-host `TEXT` rows include `0`, `0.00`, `0%`, and `yyyy-mm-dd`
   - current-host `DOLLAR` and `FIXED` rows pin the local `R` currency symbol and space grouping rules
3. the tester XLL now exposes Rust-based wrappers for selected legacy `GET.*` info surfaces, and the seeded wrapper lane is parity-closed:
   - `ox_GET_CELL`
   - `ox_GET_DOCUMENT`
   - `ox_GET_WORKBOOK`
   - `ox_GET_WORKBOOK_ACTIVE`
   - `ox_GET_WORKSPACE`
4. the main non-locale seam pressure was blank single-cell classification:
   - `TYPE(A2)` on a true blank single-cell reference returned `1`
   - `N(A2)` returned `0`
   - `T(A2)` returned `""`
   - this means OxFunc needs an explicit prepared-boundary path for dereferenced blank single-cell results rather than collapsing them to empty string, missing argument, or ordinary arrays too early

## 5. Output Artifacts
1. workset spec:
   - `docs/worksets/W013_DECEPTIVELY_SIMPLE_BOUNDARY_FUNCTIONS.md`
2. execution records:
   - `docs/function-lane/W13_EXECUTION_RECORD.md`
   - `docs/function-lane/LOCALE_FORMAT_SEAM_EXECUTION_RECORD.md`
   - `docs/function-lane/XLL_GET_INFO_EXECUTION_RECORD.md`
3. function contracts:
   - `docs/function-lane/FUNCTION_SLICE_SIN_CONTRACT_PRELIM.md`
   - `docs/function-lane/FUNCTION_SLICE_ASIN_CONTRACT_PRELIM.md`
   - `docs/function-lane/FUNCTION_SLICE_N_CONTRACT_PRELIM.md`
   - `docs/function-lane/FUNCTION_SLICE_T_CONTRACT_PRELIM.md`
   - `docs/function-lane/FUNCTION_SLICE_TYPE_CONTRACT_PRELIM.md`
   - `docs/function-lane/FUNCTION_SLICE_ROW_CONTRACT_PRELIM.md`
   - `docs/function-lane/FUNCTION_SLICE_COLUMN_CONTRACT_PRELIM.md`
   - `docs/function-lane/FUNCTION_SLICE_VALUE_CONTRACT_PRELIM.md`
   - `docs/function-lane/FUNCTION_SLICE_TEXT_CONTRACT_PRELIM.md`
   - `docs/function-lane/FUNCTION_SLICE_DOLLAR_CONTRACT_PRELIM.md`
   - `docs/function-lane/FUNCTION_SLICE_FIXED_CONTRACT_PRELIM.md`
4. function-phase-complete slices within W13:
   - `SIN`
   - `ASIN`
   - `N`
   - `T`
   - `TYPE`
   - `ROW`
   - `COLUMN`
   - `VALUE`
   - `TEXT`
   - `DOLLAR`
   - `FIXED`

## 6. Verification Runs
1. direct Excel COM spot-checks run locally on `2026-03-12` and `2026-03-14` for:
   - scalar numeric coercion and domain seeds
   - type/classification seeds
   - caller-context and reference-shape seeds
   - locale/format parser-render seeds
   - `GET.*` host-profile and info follow-up through the tester XLL
2. packet closure also binds to the machine-readable W13 manifests and replay tooling now present under `docs/function-lane/W13_S*.csv` and `tools/w13-probe/*`.

## 7. Gate Tracking
### G1 - Classification Closure
1. Status: `closed`

### G2 - Runtime/Formal Pairing Closure
1. Status: `closed`
2. Notes:
   - all eleven W13 functions now have Rust and Lean artifacts aligned to the admitted current-baseline slice.

### G3 - Empirical Closure
1. Status: `closed-provisional`
2. Notes:
   - W13 current-baseline workbook evidence is sufficient for current-phase closure.
   - broader locale/alternate-version sweeps remain explicit orthogonal validation work.

### G4 - Promotion Readiness
1. Status: `closed-provisional`
2. Notes:
   - all eleven W13 functions are now `function-phase-complete` for the current reference baseline.

## 8. Current Decision Pressure
1. W13 now closes as a packet for the current reference baseline.
2. The principal non-locale semantic surprise was not in trigonometry but in boundary behavior:
   - `TYPE(A2)` on a true blank single-cell reference is `1`, not `2` or `64`.
   - `N(A2)` is `0` and `T(A2)` is empty string, which means prepared blank single-cell input must survive through the values-only seam as `empty_cell`.
3. `ROW` and `COLUMN` are caller-context/reference-shape functions rather than ordinary scalar helpers:
   - omitted-argument calls depend on caller position,
   - area references spill one-dimensional distinct index vectors,
   - whole-axis references denote large arrays whose host publication may surface as `#SPILL!` when the anchor cannot accommodate them.

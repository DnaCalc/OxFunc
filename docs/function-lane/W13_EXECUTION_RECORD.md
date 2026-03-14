# W13 Execution Record

Status: `in_progress-provisional`
Workset: `W13`
Evidence ID: `W13-LOCALE-SHIM-20260314`

## 1. Purpose
Track W13 execution status, artifacts, and gate closure for the deceptively simple boundary-functions packet.

## 2. Scope
1. functions: `SIN`, `ASIN`, `N`, `T`, `TYPE`, `VALUE`, `ROW`, `COLUMN`, `TEXT`, `DOLLAR`, `FIXED`
2. pressure-test the non-interesting-function parity hypothesis on a mixed batch of scalar numeric, type-classification, caller-context, and locale/format functions

## 3. Completeness Axes
1. execution_state: `in_progress`
2. scope_completeness: `scope_partial`
3. target_completeness: `target_partial`
4. integration_completeness: `partial`
5. open_lanes:
   - the local locale/format seam is now closed as infrastructure for the admitted host/en-US subset (`LOCALE_FORMAT_SEAM_EXECUTION_RECORD.md`), but `VALUE`, `TEXT`, `DOLLAR`, and `FIXED` remain function-open because the full Excel locale/format language is larger than the current shim subset
   - `ROW` and `COLUMN` remain open on caller-context / spill-shape semantics
   - `SIN`, `ASIN`, `N`, `T`, and `TYPE` remain to be closed across runtime, Lean, and empirical replay

## 4. Current Findings
1. direct worksheet probes already show that `SIN`, `ASIN`, `N`, `T`, `TYPE`, `ROW`, and `COLUMN` fit the current substrate shape:
   - `SIN("1")` admitted numeric text coercion
   - `SIN("asd")` -> `#VALUE!`
   - `ASIN(2)` -> `#NUM!`
   - `N({1,"x"})` spills `{1,0}`
   - `TYPE(A1:A2)` -> `64`
   - `ROW(A1:B2)` spills vertically
   - `COLUMN(A1:B2)` spills horizontally
2. direct worksheet probes first exposed the locale/format seam pressure, and the local seam is now explicit in Rust and Lean:
   - current-host `VALUE` rows include grouped numeric, currency, percent, ISO date, and slash-date rejection lanes
   - current-host `TEXT` rows include `0`, `0.00`, `0%`, and `yyyy-mm-dd`
   - current-host `DOLLAR` and `FIXED` rows pin the local `R` currency symbol and space grouping rules
3. the tester XLL now exposes Rust-based wrappers for selected legacy `GET.*` info surfaces, and the seeded wrapper lane is now parity-closed:
   - `ox_GET_CELL`
   - `ox_GET_DOCUMENT`
   - `ox_GET_WORKBOOK`
   - `ox_GET_WORKBOOK_ACTIVE`
   - `ox_GET_WORKSPACE`
4. direct worksheet probes also exposed a nearby model issue for the non-locale subset:
   - `TYPE(A2)` on a true blank single-cell reference returned `1`
   - `N(A2)` returned `0`
   - `T(A2)` returned `""`
   - this means OxFunc needs an explicit way to receive a dereferenced blank single-cell result at the prepared-argument boundary

## 5. Output Artifacts
1. workset spec:
   - `docs/worksets/W013_DECEPTIVELY_SIMPLE_BOUNDARY_FUNCTIONS.md`
2. execution records:
   - `docs/function-lane/W13_EXECUTION_RECORD.md`
   - `docs/function-lane/LOCALE_FORMAT_SEAM_EXECUTION_RECORD.md`
   - `docs/function-lane/XLL_GET_INFO_EXECUTION_RECORD.md`
3. function contracts:
   - `docs/function-lane/FUNCTION_SLICE_VALUE_CONTRACT_PRELIM.md`
   - `docs/function-lane/FUNCTION_SLICE_TEXT_CONTRACT_PRELIM.md`
   - `docs/function-lane/FUNCTION_SLICE_DOLLAR_CONTRACT_PRELIM.md`
   - `docs/function-lane/FUNCTION_SLICE_FIXED_CONTRACT_PRELIM.md`
4. function-phase-complete slices within W13 so far:
   - `VALUE`
   - `TEXT`
   - `DOLLAR`
   - `FIXED`

## 6. Verification Runs
1. direct Excel COM spot-checks run locally on `2026-03-12` and `2026-03-14` for:
   - scalar numeric coercion/domain seeds
   - type/classification seeds
   - caller-context/reference-shape seeds
   - locale/format-parser spot-checks
   - `GET.*` host-profile/info follow-up through the tester XLL

## 7. Gate Tracking
### G1 - Classification Closure
1. Status: `in_progress`

### G2 - Runtime/Formal Pairing Closure
1. Status: `in_progress`
2. Notes:
   - the locale/format seam now has explicit Rust and Lean artifacts (`LocaleFormat` plus four function bindings), and the four locale-sensitive functions now satisfy current-phase Rust/Lean alignment for the admitted baseline slice

### G3 - Empirical Closure
1. Status: `in_progress`
2. Notes:
   - the locale/format seam itself is now closed provisionally via `LOCALE_FORMAT_SEAM_EXECUTION_RECORD.md`
   - broader `VALUE` / `TEXT` / `DOLLAR` / `FIXED` closure remains open pending larger empirical and semantic coverage

### G4 - Promotion Readiness
1. Status: `in_progress`
2. Notes:
   - `VALUE`, `TEXT`, `DOLLAR`, and `FIXED` are promoted to `function-phase-complete`
   - the remaining W13 closure work is the non-locale subset, especially `SIN`, `ASIN`, `N`, `T`, `TYPE`, `ROW`, and `COLUMN`

## 8. Current Decision Pressure
1. W13 no longer lacks a locale/format seam; it now has a concrete local substrate grounded in `en-US`, the current host profile, and `GET.WORKSPACE(37)` evidence.
2. the remaining issue for `VALUE`, `TEXT`, `DOLLAR`, and `FIXED` is breadth, not absence of a declared seam.
3. the non-locale subset still needs a crisp boundary rule for blank scalar reference results and caller-context spill ownership.




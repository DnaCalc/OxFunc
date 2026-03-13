# W13 Execution Record

Status: `in_progress-provisional`
Workset: `W13`
Evidence ID: `pending`

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
   - `VALUE`, `TEXT`, `DOLLAR`, and `FIXED` are currently blocked on a broader locale/format parser and rendering substrate
   - `N`, `T`, `TYPE`, `ROW`, and `COLUMN` also expose a supporting seam question around blank single-cell reference representation and array-lift ownership, which should be made explicit before promotion claims
   - `SIN`, `ASIN`, `N`, `T`, `TYPE`, `ROW`, and `COLUMN` remain to be closed across runtime, Lean, and empirical replay

## 4. Early Triage Findings
1. direct worksheet probes already show that `SIN`, `ASIN`, `N`, `T`, `TYPE`, `ROW`, and `COLUMN` fit the current substrate shape:
   - `SIN("1")` admitted numeric text coercion
   - `SIN("asd")` -> `#VALUE!`
   - `ASIN(2)` -> `#NUM!`
   - `N({1,"x"})` spills `{1,0}`
   - `TYPE(A1:A2)` -> `64`
   - `ROW(A1:B2)` spills vertically
   - `COLUMN(A1:B2)` spills horizontally
2. direct worksheet probes also show a real blocker for the formatting/parser subset:
   - `DOLLAR(1234.567,2)` returned locale-shaped currency text
   - `FIXED(1234.567,2)` returned locale-shaped grouping text
   - `TEXT(0.5,"0%")` clearly depends on Excel format-code rendering
   - `VALUE` accepted some numeric-text lanes (`"1E-3"`, `"12%"`) but rejected at least one date-like text lane (`"1/2/2024"`) in the current direct probe, showing locale/profile-sensitive parsing pressure
3. direct worksheet probes also exposed a nearby model issue for the non-locale subset:
   - `TYPE(A2)` on a true blank single-cell reference returned `1`
   - `N(A2)` returned `0`
   - `T(A2)` returned `""`
   - this means OxFunc needs an explicit way to receive a dereferenced blank single-cell result at the prepared-argument boundary

## 5. Output Artifacts
1. workset spec:
   - `docs/worksets/W013_DECEPTIVELY_SIMPLE_BOUNDARY_FUNCTIONS.md`
2. execution record:
   - `docs/function-lane/W13_EXECUTION_RECORD.md`

## 6. Verification Runs
1. direct Excel COM spot-checks run locally on `2026-03-12` for:
   - scalar numeric coercion/domain seeds
   - type/classification seeds
   - caller-context/reference-shape seeds
   - locale/format-parser spot-checks

## 7. Gate Tracking
### G1 - Classification Closure
1. Status: `in_progress`

### G2 - Runtime/Formal Pairing Closure
1. Status: `in_progress`

### G3 - Empirical Closure
1. Status: `in_progress`
2. Notes:
   - initial direct probes were sufficient to identify a real blocker for the locale/format subset before full W13 suite construction

### G4 - Promotion Readiness
1. Status: `in_progress`
2. Notes:
   - no W13 functions are yet promoted from this packet

## 8. Current Decision Pressure
1. W13 can likely close as a mixed packet only if the locale/format subset is either:
   - split into a dedicated substrate packet, or
   - explicitly blocked while the closeable subset proceeds
2. the relevant substrate is not just "string formatting" in the abstract; it is the combined Excel locale-profile parse/render seam for `VALUE`, `TEXT`, `DOLLAR`, and `FIXED`
3. the non-locale subset also needs a crisp boundary rule for:
   - blank scalar reference results
   - declarative array-lift ownership between OxFml/FEC and OxFunc

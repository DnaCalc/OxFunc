# WORKSET - TUX1000 Deceptively Simple Boundary Functions (W13)

## 1. Purpose
Run one packet over functions that look low-drama on the surface but are likely to teach important seam rules about coercion, caller context, type classification, and formatting/locale dependencies.

Primary intent:
1. pressure-test the "non-interesting function UDF parity hypothesis" on functions that should mostly be implementable without exotic reference construction,
2. close a reusable scalar-numeric lane (`SIN`, `ASIN`) and a reusable type/classification lane (`N`, `T`, `TYPE`),
3. close caller-context reference-observability for `ROW` and `COLUMN`,
4. determine whether `VALUE`, `TEXT`, `DOLLAR`, and `FIXED` are closeable on the current OxFunc substrate or whether they require a broader locale/format engine first.

## 2. Position and Dependencies
Program position:
1. post-W12 packet (`W13`).

Dependencies:
1. W4 coercion and ref-resolution seam work.
2. W7 string characterization.
3. W10/W12 function-packet pattern.
4. the new date serial-system seam note in `docs/function-lane/DATE_SERIAL_SYSTEM_AND_WORKBOOK_MODE_NOTES.md`.

## 3. Function Scope
1. `SIN`
2. `ASIN`
3. `N`
4. `T`
5. `TYPE`
6. `VALUE`
7. `ROW`
8. `COLUMN`
9. `TEXT`
10. `DOLLAR`
11. `FIXED`

Selection rationale:
1. numeric unary / domain / array-lift: `SIN`, `ASIN`
2. type/value classification: `N`, `T`, `TYPE`
3. caller-context and reference-shape pressure: `ROW`, `COLUMN`
4. locale/format parser and formatter pressure: `VALUE`, `TEXT`, `DOLLAR`, `FIXED`

## 4. Current W13 Triage
The batch currently splits into two semantic groups.

### 4.1 Closeable On Current Substrate
1. `SIN`
2. `ASIN`
3. `N`
4. `T`
5. `TYPE`
6. `ROW`
7. `COLUMN`

Current signal:
1. direct worksheet probes already confirm the key baseline lanes for these functions:
   - `SIN("1")`
   - `SIN("asd")`
   - `ASIN(2)`
   - `N({1,"x"})`
   - `TYPE(A1:A2)`
   - `ROW()`, `ROW(A1:B2)`
   - `COLUMN()`, `COLUMN(A1:B2)`
2. none of those probes currently require a general locale-sensitive number-format engine.

### 4.2 Locale / Format Pressure Packet
1. `VALUE`
2. `TEXT`
3. `DOLLAR`
4. `FIXED`

Current signal:
1. direct Excel spot-checks show that these functions depend materially on locale/profile-sensitive parsing or formatting.
2. in the current environment:
   - `DOLLAR(1234.567,2)` returned locale-shaped text (`R1 234.57`)
   - `FIXED(1234.567,2)` returned locale-shaped grouping text (`1 234.57`)
   - `TEXT(0.5,"0%")` clearly depends on Excel's format-code engine
   - `VALUE` accepts some textual numeric forms (`"1E-3"`, `"12%"`) but rejected at least one date-like text lane (`"1/2/2024"`) in the current direct probe
3. OxFunc now has a declared local shim substrate for admitted current-host and `en-US` rows, but not yet the full Excel locale/format language needed to close these functions honestly.

## 5. Deliverables
1. W13 workset spec and execution record.
2. function-slice contracts for all eleven functions, with explicit blocked status where appropriate.
3. runtime implementations and dispatch integration for any functions closed in W13.
4. Lean executable-semantic model coverage for any functions closed in W13.
5. W13 empirical manifests and replay tooling.
6. explicit blocked-lane note for locale/format-parser functions if closure is not yet defensible.

## 6. Gate Model
### G1 - Classification Closure
Pass when:
1. all eleven functions have explicit profile fields and primary semantic-substrate homes.

### G2 - Runtime/Formal Pairing Closure
Pass when:
1. each closeable W13 function has Rust and Lean artifacts aligned to its admitted slice.

### G3 - Empirical Closure
Pass when:
1. the W13 suite replays against the current reference baseline and compatibility lane,
2. the locale/format subset has explicit seam artifacts, manifests, and blocker evidence rather than vague postponement.

### G4 - Promotion Readiness
Pass when:
1. any closeable W13 functions reach `function-phase-complete`,
2. any blocked W13 functions are explicitly left `scope_partial` with the locale/format substrate named as the blocker.

## 7. Status
Execution state:
1. `in_progress`

Claim confidence:
1. `provisional`

Assurance maturity:
1. `exercised`

## 8. Immediate Questions
1. Should W13 remain one mixed packet with an explicit formatting/parser blocker inside it, or should the locale-sensitive sub-batch (`VALUE`, `TEXT`, `DOLLAR`, `FIXED`) be split into a dedicated follow-on substrate packet?
2. If the formatting/parser sub-batch remains in W13, which substrate becomes normative first:
   - locale-sensitive text-to-number/date parsing,
   - Excel number-format-code rendering,
   - or both together?


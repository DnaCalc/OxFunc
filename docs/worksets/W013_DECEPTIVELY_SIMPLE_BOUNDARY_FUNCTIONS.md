# WORKSET - TUX1000 Deceptively Simple Boundary Functions (W13)

## 1. Purpose
Run one packet over functions that look low-drama on the surface but are likely to teach important seam rules about coercion, caller context, type classification, and formatting/locale dependencies.

Primary intent:
1. pressure-test the non-interesting-function UDF parity hypothesis on functions that should mostly be implementable without exotic reference construction,
2. close a reusable scalar-numeric lane (`SIN`, `ASIN`) and a reusable type/classification lane (`N`, `T`, `TYPE`),
3. close caller-context reference-observability for `ROW` and `COLUMN`,
4. close `VALUE`, `TEXT`, `DOLLAR`, and `FIXED` on top of an explicit local locale-format seam for the current reference baseline.

## 2. Position and Dependencies
Program position:
1. post-W12 packet (`W13`).

Dependencies:
1. W4 coercion and ref-resolution seam work.
2. W7 string characterization.
3. W10/W12 function-packet pattern.
4. the date serial-system seam note in `docs/function-lane/DATE_SERIAL_SYSTEM_AND_WORKBOOK_MODE_NOTES.md`.

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
1. numeric unary and domain plus array-lift: `SIN`, `ASIN`
2. type/value classification: `N`, `T`, `TYPE`
3. caller-context and reference-shape pressure: `ROW`, `COLUMN`
4. locale/format parser and formatter pressure: `VALUE`, `TEXT`, `DOLLAR`, `FIXED`

## 4. Current W13 Triage
The packet is now closed for the current reference baseline.

### 4.1 Non-locale Boundary Slice
1. `SIN`
2. `ASIN`
3. `N`
4. `T`
5. `TYPE`
6. `ROW`
7. `COLUMN`

Closure signal:
1. worksheet probes and packet replay pin numeric-text admission, `ASIN` domain failure, blank single-cell classification, and caller-context/reference-shape behavior.
2. the main semantic surprise was that blank single-cell dereferences must remain explicit at the prepared boundary for `N`, `T`, and `TYPE`.
3. `ROW` and `COLUMN` are properly caller-context/reference-shape functions, not scalar-only helpers.

### 4.2 Locale / Format Slice
1. `VALUE`
2. `TEXT`
3. `DOLLAR`
4. `FIXED`

Closure signal:
1. OxFunc now has a declared local shim substrate for admitted current-host and `en-US` rows.
2. the admitted local slice is sufficient to close these functions honestly for the current reference baseline.
3. broader locale/format-language expansion remains an orthogonal validation phase.

## 5. Deliverables
1. W13 workset spec and execution record.
2. function-slice contracts for all eleven functions.
3. runtime implementations and dispatch integration for the full packet.
4. Lean executable-semantic model coverage for the full packet.
5. W13 empirical manifests and replay tooling.

## 6. Gate Model
### G1 - Classification Closure
Pass when:
1. all eleven functions have explicit profile fields and primary semantic-substrate homes.

### G2 - Runtime/Formal Pairing Closure
Pass when:
1. each W13 function has Rust and Lean artifacts aligned to its admitted slice.

### G3 - Empirical Closure
Pass when:
1. the W13 suite replays against the current reference baseline and compatibility lane,
2. locale/format and caller-context seams are documented explicitly rather than deferred vaguely.

### G4 - Promotion Readiness
Pass when:
1. all eleven W13 functions reach `function-phase-complete` for the current reference baseline.

## 7. Status
Execution state:
1. `complete`

Claim confidence:
1. `provisional`

Assurance maturity:
1. `exercised`

## 8. Immediate Questions
1. Follow-on work should move from W13 function closure to broader locale/format-language and alternate-version sweeps, not reopen the current packet.

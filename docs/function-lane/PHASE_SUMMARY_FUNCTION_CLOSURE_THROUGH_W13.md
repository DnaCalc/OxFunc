# Phase Summary - Function Closure Through W13

Status: `active`
Scope: current reference-baseline implementation phase through W13

## 1. Purpose
Summarize which functions are now `function-phase-complete`, what semantic seams were clarified, and what follow-on work remains after W13.

## 2. Function-Phase-Complete Set
The following functions are now `function-phase-complete` for the current reference Excel baseline:

1. W1:
   - `PI`
2. W5:
   - `ABS`
3. W6:
   - `XMATCH`
4. W10:
   - `SUM`
   - `IF`
   - `INDEX`
   - `MATCH`
   - `ISNUMBER`
   - `NOW`
   - `XLOOKUP`
   - `INDIRECT`
   - `SEQUENCE`
   - `OP_ADD`
5. W12:
   - `AVERAGE`
   - `COUNT`
   - `COUNTA`
   - `IFERROR`
   - `ROUND`
   - `TEXTJOIN`
   - `TODAY`
   - `RAND`
   - `OFFSET`
   - `AND`
   - `CLEAN`
   - `DATE`
   - `EXACT`
   - `HSTACK`
6. W13:
   - `SIN`
   - `ASIN`
   - `N`
   - `T`
   - `TYPE`
   - `VALUE`
   - `ROW`
   - `COLUMN`
   - `TEXT`
   - `DOLLAR`
   - `FIXED`

`CELL` remains intentionally deferred.

## 3. Main Semantic Lessons So Far
### 3.1 Values-Only Versus Reference-Observable
1. The useful split is not “all provenance everywhere”.
2. The practical split is:
   - values-only input versus references-visible input
   - value-result-only versus may-return-reference result
3. `SUM` is values-only but argument-structure-sensitive.
4. `XLOOKUP`, `INDEX`, `OFFSET`, and `CELL` remain reference-observable.
5. `INDIRECT` is values-only on input but may-return-reference.

### 3.2 Blank Single-Cell Boundary
1. Blank single-cell dereference cannot be collapsed too early.
2. Current pinned examples:
   - `TYPE(A2)` on a true blank referenced cell is `1`
   - `N(A2)` is `0`
   - `T(A2)` is `""`
3. OxFunc therefore needs an explicit prepared-boundary blank-cell path.

### 3.3 Format Hinting
1. `NOW()` and `TODAY()` produce ordinary numeric/date serials plus format-hint semantics at the worksheet boundary.
2. This belongs above the pure kernel and is currently modeled as result metadata rather than direct grid mutation.
3. XLL verification does not need to reproduce caller-cell format application.

### 3.4 Whole-Axis And Compatibility Sensitivity
1. `ROW` and `COLUMN` are caller-context/reference-shape functions, not scalar-only helpers.
2. Area references produce one-dimensional distinct index vectors.
3. Whole-axis cardinality is compatibility-sensitive:
   - default workbook lane: `ROW(A:A)` has height `1048576`; `COLUMN(1:1)` has width `16384`
   - compat-template `.xls` lane: `ROW(A:A)` has height `65536`; `COLUMN(1:1)` has width `256`
4. The worksheet may surface `#SPILL!` depending on publication context, but the core function result is the large axis vector.

### 3.5 Locale/Format Boundary
1. `VALUE`, `TEXT`, `DOLLAR`, and `FIXED` required an explicit locale/format seam.
2. The current admitted closure uses a small local seam with:
   - `en-US`
   - `current_excel_host`
3. Broader locale/format-language expansion is still follow-on validation work, not an open semantic gap in the current phase.

## 4. XLL Seams
1. XLL remains valuable as a verification seam.
2. XLL limitations are explicit rather than function-semantic compromises.
3. Important current examples:
   - format-hint application is outside the XLL obligation
   - modern worksheet-only error surfaces may degrade through legacy XLL transport
   - native `GET.*` wrapper parity now works on the seeded slice when properly macro-registered

## 5. What Remains After W13
The next open fronts are substrate and validation topics, not unfinished current-phase function closure inside W1 through W13.

1. `CELL` and adjacent `GET.*` / info-macro design work
2. broader locale/format-language coverage
3. date-system and workbook-mode follow-up
4. alternate-version and alternate-locale validation sweeps across the completed functions
5. upstream OxFml/FEC/F3E interface refinement based on the semantic seams now pinned

## 6. Reporting Guidance
1. Use this note as the compact summary of what is closed through W13.
2. Use per-function contracts and packet records for detailed evidence and artifact bindings.
3. Treat later locale/version sweeps as orthogonal validation phases unless explicitly pulled into scope for a new packet.

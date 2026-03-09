# WORKSET - TUX1000 String W8.1 Checklist

## 1. Purpose
Define the immediate post-baseline string work packet: close high-value gaps after W7 baseline before deep W6 (`XMATCH`) closure.

## 2. Scope
In scope:
1. compatibility-version-sensitive Unicode behavior checks,
2. formula admission pathway characterization for long literals,
3. control/unprintable edge matrix expansion,
4. collation/order expansion in `en-US`,
5. UDF/XLL string boundary reconnaissance.

Out of scope:
1. full multi-locale collation closure,
2. full text-function catalog closure.

## 3. Process Requirements
1. Use `expected_status` and `expected_observable` on every scenario row.
2. Run via suite tooling (`run-string-suite.ps1`) and publish analyzer output.
3. Emit run metadata sidecars and archive them with results.
4. Populate cross-boundary invariants using:
   - `docs/function-lane/CROSS_BOUNDARY_INVARIANT_CHECKLIST_TEMPLATE.md`.

## 4. Checklist
1. `W8.1-01` Compatibility Version matrix
   - build template-backed runs for at least one non-default compatibility workbook.
   - focus: `LEN/LEFT/RIGHT/MID/UNICODE` on surrogate and combining-sequence cases.
   - closure artifact: compatibility-differential table.
2. `W8.1-02` Formula admission matrix expansion
   - compare long-literal admission across: UI entry, `Range.Formula`, `Range.Formula2`, `Evaluate`, file-ingress.
   - closure artifact: admission boundary matrix with mechanism-specific limits/errors.
3. `W8.1-03` Control-character matrix
   - include `CHAR(0)`, selected C0/C1 controls, zero-width marks, NBSP variants.
   - verify behavior through formula, interop set, reference reuse, CSV roundtrip.
   - closure artifact: normalization/removal matrix for `TRIM`, `CLEAN`, and equality/search paths.
4. `W8.1-04` Collation/order mini-pack (`en-US`)
   - expand punctuation/case/diacritic ordering probes using `<`, `>`, sort-relevant functions.
   - closure artifact: provisional ordering policy section update.
5. `W8.1-05` XLL/UDF boundary reconnaissance
   - test string return/arg edge cases (length cap, over-cap behavior, potential normalization).
   - closure artifact: boundary note feeding W4/W6 contract decisions.

## 5. Deliverables
1. updated scenario manifest rows (`STR8.1-*` additions),
2. suite outputs:
   - results CSV(s),
   - analyzer report,
   - drift report (when baseline provided),
   - metadata sidecars,
3. updated policy map and execution record,
4. invariant checklist instance for W8.1,
5. conformance updates if policy shifts are confirmed.

## 6. Gate Model
### G1 - Scenario Readiness
Pass when:
1. all new rows carry expectation fields and boundary tags.

### G2 - Replay Closure
Pass when:
1. default and compatibility-template runs execute with metadata sidecars.

### G3 - Analysis Closure
Pass when:
1. analyzer reports zero unexpected failures,
2. drift deltas (if any) are classified.

### G4 - Policy Feed Closure
Pass when:
1. W8.1 findings are merged into string policy + W3/W6 dependency notes.

## 7. Status
Execution state:
1. `planned`.

Claim confidence:
1. `draft`.

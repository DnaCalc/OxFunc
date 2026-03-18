# WORKSET - TUX1000 CELL and INFO Host-Query Functions (W15)

## 1. Purpose
Complete the deferred host-information function seam around `CELL` and `INFO`.

Primary intent:
1. finish the deferred `CELL` work from `W12` without misclassifying it as a pure local kernel,
2. characterize `INFO` through reproducible host-baseline evidence before runtime admission,
3. define the standard OxFunc/FEC host-query seam these functions need,
4. close as much of `CELL` and `INFO` as current host evidence and callback facilities allow.

## 2. Position and Dependencies
Program position:
1. post-`W12` deferred `CELL` follow-on,
2. post-`W13` locale/format seam and tester-XLL `GET.*` wrapper work,
3. parallel to but independent from deferred `W14` `@` work.

Dependencies:
1. `W12` bounded `CELL` preprobe and current partial runtime slice,
2. `W13` locale/format seam artifacts where `CELL` returns display/format-adjacent metadata,
3. `W9` tester-XLL `GET.CELL` / `GET.WORKBOOK` / `GET.WORKSPACE` wrappers for reproducible host evidence,
4. existing OxFunc resolver and caller-context seams.

## 3. Function Scope
1. `CELL`
2. `INFO`

## 4. Working Thesis
`CELL` and `INFO` are not pure kernels and are not arbitrary evaluator callbacks either.

The intended split is:
1. OxFunc owns:
   - `info_type` / `type_text` normalization,
   - admission and coercion policy,
   - mapping from query kind to expected result type,
   - worksheet-visible error mapping,
   - any sub-lanes that are computable from preserved reference identity or dereferenced value alone.
2. FEC/F3E owns:
   - actual cell metadata,
   - workbook/application/environment facts,
   - any live host state needed to answer typed queries.

The target seam is therefore a typed host-query capability view, not raw parse-tree access and not stringly callback logic embedded in OxFunc kernels.

## 5. Initial Host-Query Split
### 5.1 `CELL`
Likely OxFunc-local or mostly local lanes:
1. `address`
2. `row`
3. `col`
4. `contents`
5. `type`

Likely host-query lanes:
1. `filename`
2. `format`
3. `color`
4. `parentheses`
5. `prefix`
6. `protect`
7. `width`
8. omitted-reference active-selection forms across the full admitted info-type set
9. broader formatting/alignment/font/protection families if admitted later

### 5.2 `INFO`
Likely host-query lanes:
1. `directory`
2. `numfile`
3. `origin`
4. `osversion`
5. `recalc`
6. `release`
7. `system`
8. `memavail`
9. `memused`
10. `totmem`

## 6. Evidence Posture
Current evidence anchors:
1. `W12-CELL-PRE-20260309`
2. `W9-XLL-GETINFO-20260314`
3. `W15-INFO-PRE-20260315`
4. `W15-CELL-HOST-PRE-20260315`
5. `W15-XLL-BRIDGE-20260315`

Observed `INFO` baseline on `2026-03-15` from the local Excel host:
1. `directory` -> current default workbook directory
2. `numfile` -> `1`
3. `origin` -> `$A:$A$1`
4. `osversion` -> `Windows (64-bit) NT 10.00`
5. `recalc` -> `Automatic`
6. `release` -> `16.0`
7. `system` -> `pcdos`
8. `memavail` / `memused` / `totmem` -> `#N/A`

Observed broadened `CELL` baseline on `2026-03-15` from the local Excel host:
1. `filename` -> saved-workbook path including `[workbook]Sheet`
2. `format` -> `F2` for the seeded fixed-decimal numeric format
3. `color` -> `1` for the seeded red-negative format
4. `prefix` -> `'` for the seeded left-aligned text lane
5. `protect` -> `1` for the seeded locked-cell lane
6. `width` -> `20` in ordinary single-cell formula context, with a second boolean item available through `INDEX(...,2)` and a two-column shape visible through `COLUMNS(...)`
7. `parentheses` -> `1` for a positive custom format that displays values in parentheses
8. omitted-reference `CELL(info_type)` follows the active selected cell, not the formula cell, across the admitted current-baseline matrix:
   - `row`, `address`, `col`, `contents`, `type`
   - `filename`, `format`, `color`, `prefix`, `protect`, `width`, `parentheses`

## 7. Deliverables
1. `W15` workset packet and execution record.
2. `FUNCTION_SLICE_INFO_CONTRACT_PRELIM.md`.
3. host-query seam note covering `CELL` and `INFO`.
4. reproducible `INFO` preprobe manifest and runner.
5. reproducible `CELL` host-lane preprobe manifest and runner.
6. runtime seam changes and implementations needed for admitted `CELL` / `INFO` lanes.
7. Lean modules and substrate bindings required by the admitted slices.
8. function-level verification and replay artifacts.

## 8. Gate Model
### G1 - Seam Definition Closure
Pass when:
1. the OxFunc/FEC split for `CELL` and `INFO` is documented explicitly,
2. the required typed host-query categories are stated,
3. no claim remains that OxFunc can answer workbook/cell metadata without host help.

### G2 - Empirical Narrowing Closure
Pass when:
1. current-baseline `INFO` option lanes are replayed through a deterministic probe,
2. seeded `CELL` host-sensitive option lanes are replayed through a deterministic probe,
3. the remaining `CELL` option families are either replayed or explicitly deferred with reasons,
4. unsupported or host-variable lanes are documented instead of guessed.

### G3 - Runtime/Formal Pairing Closure
Pass when:
1. `CELL` and `INFO` have Rust/runtime artifacts for the admitted slices,
2. Lean/formal artifacts exist for the admitted semantic substrate and bindings,
3. the host-query seam is explicit in code, not hidden in ad hoc workbook inspection logic.

### G4 - Function Closure Readiness
Pass when:
1. no known semantic gap remains in the admitted current-baseline `CELL` and `INFO` scope,
2. evidence and implementation agree on the same typed query contract,
3. any still-open option families remain explicit and keep the packet `scope_partial`.

## 9. Status
Execution state:
1. `complete`

Claim confidence:
1. `provisional`

Assurance maturity:
1. `exercised`

## 10. Closure Position
1. local semantic/evidence/runtime/formal closure is achieved for the admitted current-baseline `CELL` / `INFO` slice,
2. upstream seam acknowledgment and integration for `HO-FN-002` are now recorded on both the OxFunc and OxFml sides,
3. `CELL` and `INFO` are therefore `function-phase-complete` for the declared current-baseline scope of `W015`,
4. broader locale/version sweeps remain orthogonal validation-phase work rather than an open lane in declared scope.

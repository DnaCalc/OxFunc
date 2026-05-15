# BUG-FUNC-013: Normal-distribution exact-value accuracy gap

## Summary
- **Bug id**: `BUG-FUNC-013`
- **Opened**: `2026-04-10`
- **Status**: `closed`
- **Owner workset**: `W086`

## Source Refs
- **Reported against ref**: `2e818f03a71ba393690275a7fb437ddd9a6bf760`
- **Reproduced on ref**: `2e818f03a71ba393690275a7fb437ddd9a6bf760`
- **Introduced in ref**: `unknown`
- **Fixed in ref**: `8234dce5f3e0c50a3c634466ead38c67fa93937e`
- **Ref notes**: direct local replay on 2026-04-10 confirmed current OxFunc
  exact-value drift on bounded `NORM.DIST` and `NORM.INV` witnesses, while
  live Excel `Value2` replay on the same date pinned the current baseline
  targets.

## Ownership And Root Cause
- **Ownership class**: `OxFunc-owned bug`
- **Root cause class**: `initial_impl_gap`
- **Root cause summary**: the admitted W062 statistical-distribution evidence
  pinned rounded witness values rather than current-baseline exact `Value2`
  parity, and the local error-function helper was still a coarse approximation
  rather than a current-baseline-exact path.

## Why Did We Get This Wrong?
- **Spec already correct and code was wrong?**: `yes`
- **Spec vague or missing?**: `no`
- **Code once correct and later regressed?**: `no`
- **Likely introduced in ref**: `unknown`
- **Explanation**: the current `normal_log_family.rs` kernels were accepted on a
  bounded rounded-evidence floor. That floor was too weak to pin the exact
  current-baseline witnesses now under review, so the local approximation path
  remained good enough for prior rounded assertions while still diverging from
  Excel `Value2`.

## Reproduction
1. Direct local replay on 2026-04-10:
   - `NORM.DIST(0,0,1,TRUE) -> 0.49999998499999976`
   - `NORM.INV(0.975,0,1) -> 1.9599639471668913`
2. Live Excel `Value2` replay on 2026-04-10:
   - `NORM.DIST(0,0,1,TRUE) -> 0.5`
   - `NORM.INV(0.975,0,1) -> 1.9599639845400536`

## Spec And Contract Relationship
- **Spec references**:
1. `docs/function-lane/W16_EXECUTION_RECORD.md`
2. `docs/function-lane/FUNCTION_SLICE_STATISTICAL_DISTRIBUTIONS_AND_COMPAT_B_CONTRACT_PRELIM.md`
3. `docs/function-lane/W51_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_INVENTORY.csv`
- **Spec state at intake**: `correct but incomplete`
- **Notes**: the statistical-distribution slice was previously reported as
  closed on a rounded witness floor. This bug reopens only the bounded exact-
  value accuracy lane for `NORM.DIST` and `NORM.INV`; it does not yet reopen
  the wider W062 distribution family by analogy.

## Investigation Log
1. 2026-04-10: user asked to confirm whether several small-variation
   differences were real function issues or display/comparison-policy issues.
2. 2026-04-10: direct local replay confirmed the exact current OxFunc values
   `0.49999998499999976` and `1.9599639471668913`.
3. 2026-04-10: live Excel `Value2` replay confirmed exact current-baseline
   values `0.5` and `1.9599639845400536`.
4. 2026-04-10: code review located the bounded approximation paths in
   `normal_log_family.rs` and confirmed that earlier evidence used rounded
   witness rows rather than exact-value parity.
5. 2026-04-11: replacing the local error-function approximation with `libm::erf`
   aligned the bounded `NORM.*` exact-value witnesses with live Excel `Value2`
   and also tightened the adjacent `Z.TEST` survivor lane to the current Excel
   observable.
6. 2026-04-11: bounded helper-adjacent replay confirmed current exact-value
   alignment for `NORM.S.DIST(0,TRUE)`, `NORM.S.INV(0.975)`, `GAUSS(1)`,
   `PHI(0)`, `ERF(1)`, `ERFC(1)`, and `Z.TEST({3,6,7,8,6},4,1.5)` without
   reopening broader W062 distribution rows.
7. 2026-04-12: landed the bounded W086 repair on committed ref
   `8234dce5f3e0c50a3c634466ead38c67fa93937e`, reran the focused exact-value
   regression floor on that ref, and removed the reopened `NORM.DIST` /
   `NORM.INV` rows from `W051`.

## Similar-Risk Scan
### Adjacent families to check
1. same implementation family:
   - `NORM.S.DIST`
   - `NORM.S.INV`
   - legacy aliases `NORMDIST`, `NORMINV`, `NORMSDIST`, `NORMSINV`
2. broader W062 statistical-distribution rows that may also rely on bounded
   approximation-quality assumptions:
   - `LOGNORM.*`
   - `T.*`
   - `CHISQ.*`

### Check method
1. direct local replay on exact-value current-baseline witnesses
2. live Excel `Value2` replay for the same witnesses
3. local code review for shared approximation helpers and alias delegation
4. exact local regression tests on helper-adjacent witness rows

### Results
1. `NORM.DIST` and `NORM.INV` are real local accuracy gaps against exact Excel
   `Value2`, not display-only differences.
2. bounded helper-adjacent replay showed `NORM.S.*`, `GAUSS`, `ERF`, `ERFC`,
   and `Z.TEST` now align on the pinned current-baseline exact-value rows.
3. `PHI` remains aligned on its direct witness and was not part of the faulty
   `erf` path.
4. broader statistical-distribution functions remain a review surface only;
   no family-wide reopening is claimed from this bounded intake alone.

### Follow-on Openings
1. `W086`

## Fix Plan
1. characterize the exact current approximation gap in `normal_log_family.rs`
2. reconcile `NORM.DIST` and `NORM.INV` against the bounded live Excel
   witnesses
3. add focused exact-value regression coverage
4. reconcile `W051` and related truth surfaces honestly
5. widen only directly replayed adjacent rows if the shared helper path proves
   they are also affected

## Validation
1. focused local Rust tests for `normal_log_family`
2. direct exact-value local replay for the pinned witnesses
3. live Excel `Value2` replay for the same witnesses
4. `powershell -ExecutionPolicy Bypass -File scripts/check-worksets.ps1`

## Linked Reports
1. `BUGREP-FUNC-017`

## 2026-05-10 W097 R-G Cell-Ref Re-Sweep

W097 R-G confirms BUG-FUNC-013 closure on the direct witnesses under
cell-ref Excel input plumbing:

| Witness                          | local                     | Excel                     |
| -------------------------------- | ------------------------- | ------------------------- |
| `=NORM.DIST(0, 0, 1, TRUE)`      | `0.5` (`0x3fe0000000000000`) | `0.5`                     |
| `=NORM.INV(0.975, 0, 1)`         | `0x3fff5c0331eeff82`      | `0x3fff5c0331eeff82`      |
| `=NORMSDIST(0)`                  | `0x3fe0000000000000`      | `0x3fe0000000000000`      |
| `=NORMSINV(0.975)`               | `0x3fff5c0331eeff82`      | `0x3fff5c0331eeff82`      |
| `=NORM.S.DIST(0, TRUE)`          | `0x3fe0000000000000`      | `0x3fe0000000000000`      |
| `=NORM.S.INV(0.975)`             | `0x3fff5c0331eeff82`      | `0x3fff5c0331eeff82`      |
| `=ERF(1)`                        | `0x3feaf767a741088b`      | `0x3feaf767a741088b`      |
| `=ERFC(1)`                       | `0x3fc4226162fbddd5`      | `0x3fc4226162fbddd5`      |

The four direct closure witnesses (`NORM.DIST`, `NORM.INV`,
`NORMSDIST`, `NORMSINV`) plus the two `NORM.S.*` aliases plus
`ERF(1)` / `ERFC(1)` all match Excel bit-for-bit. Closure remains
tight on the direct surface.

The bounded helper-adjacent rows pinned in Investigation Log item 6
were re-checked. Two of them drift `1-2` ULP under cell-ref:

| Witness         | local                | Excel                | ULP |
| --------------- | -------------------- | -------------------- | --: |
| `=GAUSS(1)`     | `0x3fd5d897a241a6fa` | `0x3fd5d897a241a6fc` | `2` |
| `=PHI(0)`       | `0x3fd9884533d43650` | `0x3fd9884533d43651` | `1` |

These are not the direct closure witnesses, so this stream is **not
reopened in place**. The two helper-adjacent rows are recorded as a
follow-up candidate for a successor `BUG-FUNC-NNN` if a downstream
consumer needs bit-exact `GAUSS / PHI` parity. Absolute drift is
`~2E-16` on values of order `0.34..`, which is unobservable in
typical rendered output.

Run record: `smart-fuzzer/runs/W097-R-GH-closed-streams-cellref/`.
Tranche record:
`smart-fuzzer/planning/W097-R-GH-closed-streams-cell-ref-resweep.md`.

## Evidence
1. `crates/oxfunc_core/src/functions/normal_log_family.rs`
2. `docs/function-lane/FUNCTION_SLICE_STATISTICAL_DISTRIBUTIONS_AND_COMPAT_B_CONTRACT_PRELIM.md`
3. `docs/worksets/W086_NORMAL_DISTRIBUTION_EXACT_VALUE_ACCURACY.md`
4. W097 R-G cell-ref confirmation:
   `smart-fuzzer/runs/W097-R-GH-closed-streams-cellref/`

## Closure Checklist
- [x] local fix implemented
- [x] validation recorded
- [x] root cause recorded
- [x] similar-risk scan recorded
- [x] spec/matrix/contract updated if required
- [x] linked reports updated
- [x] handoff filed if required
- [x] fix landed or non-OxFunc ownership recorded

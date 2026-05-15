# W097 R-G / R-H — Closed BUG-FUNC streams cell-ref re-sweep

Status: `tranche_complete`

Owning workset: `docs/worksets/W097_BIT_EXACT_RESWEEP_OF_KNOWN_MISMATCHES.md`
Owning bead: `oxf-ic1h.7`
Plumbing rule: `smart-fuzzer/planning/EXCEL_RUNNER_PLUMBING_NOTE.md`

## 1. What this record is

R-G / R-H confirm closure (or surface a regression) of three closed
BUG-FUNC exactness streams under the cell-ref Excel input plumbing:

- `BUG-FUNC-005` (POWER zero-to-zero and adjacency)
- `BUG-FUNC-013` (NORM.* exact-value accuracy)
- `BUG-FUNC-014` (XIRR solver-precision)

## 2. New tooling

`smart-fuzzer/tools/Run-ClosedStreamResweep.ps1` — generates the
witness case set for the three streams, evaluates locally through
`matrix_local_eval` (handles scalar / logical / matrix args), and
runs Excel via `Invoke-ExcelCellRefBatch`.

## 3. Run

`smart-fuzzer/runs/W097-R-GH-closed-streams-cellref/`

- Cases: `15` (4 BUG-FUNC-005 + 10 BUG-FUNC-013 + 1 BUG-FUNC-014)
- Rollup: matches `13`, drifts `2`, kind drift `0`, blocked `0`
- Excel environment: `16.0` build `19929`

| Stream         | total | match | drift | kind | blocked |
| -------------- | ----: | ----: | ----: | ---: | ------: |
| `BUG-FUNC-005` |   `4` |   `4` |   `0` |  `0` |    `0`  |
| `BUG-FUNC-013` |  `10` |   `8` |   `2` |  `0` |    `0`  |
| `BUG-FUNC-014` |   `1` |   `1` |   `0` |  `0` |    `0`  |

## 4. Per-stream closure status

### BUG-FUNC-005 (closed) — confirmed tight

Under cell-ref plumbing, all four canonical witnesses match Excel
bit-for-bit:

| Witness                | local       | Excel       |
| ---------------------- | ----------- | ----------- |
| `=POWER(0, 0)`         | `error:Num` | `error:Num` |
| `=POWER(0, -1)`        | `error:Div0`| `error:Div0`|
| `=POWER(0, 1)`         | `0` (`0x0`) | `0` (`0x0`) |
| `=POWER(1, 0)`         | `1` (`0x3ff0000000000000`) | `1` (`0x3ff0000000000000`) |

Closure remains tight; no successor stream opened.

### BUG-FUNC-013 (closed) — direct witnesses confirmed; two
helper-adjacent witnesses drift

Direct closure witnesses all bit-exact:

| Witness                          | local                   | Excel                    |
| -------------------------------- | ----------------------- | ------------------------ |
| `=NORM.DIST(0, 0, 1, TRUE)`      | `0.5` (`0x3fe0000000000000`) | `0.5` |
| `=NORM.INV(0.975, 0, 1)`         | `0x3fff5c0331eeff82`    | `0x3fff5c0331eeff82`     |
| `=NORMSDIST(0)`                  | `0x3fe0000000000000`    | `0x3fe0000000000000`     |
| `=NORMSINV(0.975)`               | `0x3fff5c0331eeff82`    | `0x3fff5c0331eeff82`     |
| `=NORM.S.DIST(0, TRUE)`          | `0x3fe0000000000000`    | `0x3fe0000000000000`     |
| `=NORM.S.INV(0.975)`             | `0x3fff5c0331eeff82`    | `0x3fff5c0331eeff82`     |
| `=ERF(1)`                        | `0x3feaf767a741088b`    | `0x3feaf767a741088b`     |
| `=ERFC(1)`                       | `0x3fc4226162fbddd5`    | `0x3fc4226162fbddd5`     |

Helper-adjacent witnesses with new drift under cell-ref:

| Witness         | local                | Excel                | ULP |
| --------------- | -------------------- | -------------------- | --: |
| `=GAUSS(1)`     | `0x3fd5d897a241a6fa` | `0x3fd5d897a241a6fc` | `2` |
| `=PHI(0)`       | `0x3fd9884533d43650` | `0x3fd9884533d43651` | `1` |

The BUG-FUNC-013 Investigation Log item 6 records that "bounded
helper-adjacent replay confirmed current exact-value alignment for
`NORM.S.DIST(0,TRUE)`, `NORM.S.INV(0.975)`, `GAUSS(1)`, `PHI(0)`,
`ERF(1)`, `ERFC(1)`, and `Z.TEST({3,6,7,8,6},4,1.5)`". Under the
sharper cell-ref plumbing the `NORM.S.*` and `ERF`/`ERFC` rows do
align bit-exactly, but `GAUSS(1)` shows `2` ULP and `PHI(0)` shows
`1` ULP. These are **not** the BUG-FUNC-013 direct closure witnesses
(those are the four `NORM.*` rows above) so the closure is **not**
reopened in place. The two helper-adjacent rows are recorded as a
successor candidate `BUG-FUNC-NNN` for the `GAUSS / PHI` substrate
exactness, to be opened only if a downstream consumer relies on
bit-exact `GAUSS / PHI` parity (the OxFunc kernel and the Excel
kernel are both well within absolute-tolerance precision; the
"closed under no-tolerance" claim was over-stated by `1-2` ULP).

### BUG-FUNC-014 (closed) — confirmed tight

Witness `=XIRR({-10000, 2750, 4250, 3250, 2750}, {44927, 45108,
45292, 45473, 45658})` returns `0x3fcf4b8226666664` (≈
`0.24449183344840997`) on both sides bit-for-bit. Closure remains
tight; no successor stream opened.

## 5. Doctrine

- BUG-FUNC-005 closure: confirmed tight under cell-ref. No
  follow-up.
- BUG-FUNC-013 closure: direct witnesses confirmed tight. Helper-
  adjacent `GAUSS(1)` and `PHI(0)` rows drift `1-2` ULP under
  the sharper plumbing — this is recorded as a follow-up
  observation, not a reopening, since the direct closure rows
  (NORM.* family and the `erf`/`erfc` substrate) are unchanged.
- BUG-FUNC-014 closure: confirmed tight under cell-ref. No
  follow-up.

The two helper-adjacent drift rows are filed as a follow-up
candidate (potential `BUG-FUNC-NNN` for `GAUSS`/`PHI` substrate
exactness). Whether to open it depends on whether any downstream
consumer needs bit-exact parity for those two helpers; absolute
magnitude is `1.7E-16` on `0.34134474606854293` (PHI) and `~2E-16`
on `0.34134474606854293` (GAUSS), well within the kind of
unobservable-in-rendered-output drift that closes-but-not-bit-exact
streams routinely accept.

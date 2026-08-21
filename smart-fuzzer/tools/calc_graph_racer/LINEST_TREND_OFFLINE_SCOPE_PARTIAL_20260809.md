# G3-03 LINEST/TREND offline discovery checkpoint (2026-08-09)

`execution_state: in_progress`

- `scope_completeness: scope_partial`
- `target_completeness: target_partial`
- `integration_completeness: partial`
- `open_lanes:` exact LINEST coefficient accumulator/solver graph; TREND
  coefficient-to-prediction publication graph; LOGEST logarithm/coefficient
  publication; multivariate/rank-deficient/stats/coercion/error/version axes;
  prior-disjoint heldout; production integration.

## Scope and provenance

This is an offline-only clean-room audit. It made no Excel/COM call and changed
no production code, shared documentation, state, beads, staging area, or git
history. The research binary reads only three already-banked answer files and
their three named answer-blind batch files, then calls the current OxFunc
production surfaces in-process.

The corrected coefficient graph family is isolated in the lane-local
`src/bin/regression_research/common.rs`. It imports only the arithmetic and
ordering primitives from the canonical GROWTH/LOGEST helper. That canonical
helper remains tracked-clean and byte-identical to its recorded
`A0FC652C...C780` replay hash.

The July answer files contain raw witness IDs, arguments, and result bits but no
embedded Excel build, architecture, Compatibility Version, cache, or capture-
method metadata. The results below therefore characterize the current
production code against that inherited bank; they are not current-reference
promotion evidence.

Before any score, the binary now fails closed unless each answer/batch pair has
the expected function and frozen row count, unique IDs on both sides, and exact
ordered ID and argument equality. The exercised assertions passed `12/12`,
`35/35`, and `35/35`; an independent offline audit reproduced the same parity.

## Reproduction

From `smart-fuzzer/tools/calc_graph_racer`:

```powershell
$env:CARGO_TARGET_DIR='C:\Work\DnaCalc\OxFunc\.tmp\target-regression-kernel'
cargo run --release --offline --bin race_regression_linest_offline
```

The exercised release run exited zero. The repaired tool races 3,264 uniquely
identified structural graphs and asserts that every candidate ID is unique:

- 1,968 common graphs: 246 coefficient graphs crossed independently with eight
  publication graphs. The coefficient space consists of 54 centered, 54
  raw-normal, 72 determinant, 42 Welford/Youngs-Cramer, and 24 direct
  modified-Gram-Schmidt/Householder graphs. The publication space contains two
  intercept forms plus binary64 and stored-x87 point-slope forms at each of
  forward, reverse, and pairwise publication-mean order.
- 1,296 extra graphs: 162 centered-QR, max-scaled-centered, and
  max-scaled-centered-QR coefficient graphs crossed with the same eight
  publication graphs.

Every counted axis changes an arithmetic graph. Direct QR coefficient graphs no
longer enumerate unused mean/intercept axes; their intercept is explicitly the
solver coefficient. Determinant raw-intercept and `mean_y-slope*mean_x` graphs
have distinct IDs. MGS reverse accumulation now constructs products in natural
order and lets the reverse reducer own ordering. Pairwise Welford is a genuine
public Chan-Golub-LeVeque merge recurrence rather than a relabelled forward
traversal. On these 12 cells, the 3,264 structural graphs collapse to 277
distinct output-bit vectors; the tool reports both counts.

## Deterministic scores

Current production replay:

| Surface | Exact | Structural result | Numeric residual |
|---|---:|---|---|
| `FORECAST` | `70/70` | `65/65` numeric plus `5/5` `#DIV/0!` controls | max `0` ULP, sum `0` |
| `TREND` | `1/12` | three translated-offset rows return `#NUM!` where the bank is numeric | among nine numeric publications: max `38` ULP, sum `84` ULP |

The four banked TREND datasets replay as follows:

| Dataset | n | Current LINEST coefficient result | Current TREND exact |
|---|---:|---|---:|
| `tr-000` | 4 | slope `0x3fffeb851eb851f0`, intercept `0x3f99999999999800` | `1/3` |
| `tr-005` | 5 | slope `0x4007e6ff25ca03fc`, intercept `0xbfbe822186f60e00` | `0/3` |
| `tr-010` | 4 | `#NUM!` | `0/3` |
| `tr-015` | 12 | slope `0xbfc162c2e7f95a00`, intercept `0x402c0e6ab448b1d8` | `0/3` |

No candidate graph is exact on the 12 TREND cells. The corrected 3,264-graph
race preserves the leader:

```text
centered / binary64 / forward means / pairwise centered moments /
intercept = mean_y - slope*mean_x / publish intercept + slope*new_x
exact=7/12, max_ulp=13, sum_ulp=23
```

Exactly four graphs have the leader's aggregate score, and the tool asserts that
all four output-bit vectors are identical on all 12 cells
(`aggregate_score_ties=4`, `bit_identical_output_ties=4`). They cross
binary64/stored-x87 coefficient arithmetic with binary64/stored-x87 intercept
publication. Forward and reverse centered moments both reach `7/12` with max
`16` ULP and sum `26` ULP. The best explicit max-scaled-centered-QR graph
reaches only `6/12`, max `13` ULP, sum `24` ULP. Thus the bank does not identify
the LINEST coefficient schedule; it rules out an exact survivor in the repaired
bounded graph space.

The deterministic raw 72-line UTF-8 stdout (LF endings, final LF, 6,757 bytes)
has SHA-256
`0A8EA88DDC1E3C8E4830CAF2E6FAA6FB06D3A12C39F62B9EE573CB903897BE16`.
PowerShell `Out-String` converts that stream to CRLF (6,829 bytes), whose
SHA-256 is
`1FBDEB0C15C9C285485F50D82BFCF1EF7171950E52BDC49D0749F75C72BF5E17`.

The replay materially separates the surfaces: the already-promoted centered
FORECAST kernel remains bit-identical on its full inherited bank, while the
current normal-equations `LINEST`/`TREND` path is both numerically non-identical
and unable to admit one translated-offset dataset. FORECAST must not be used as
evidence for the matrix/regression path.

## Exact-survivor next gate

The coherent next discriminator is a frozen, answer-blind direct-coefficient
capture before any further graph expansion:

1. Reuse the four already-banked TREND datasets and add their row reversals.
2. Capture both cells of `LINEST(y,x,TRUE,FALSE)` directly (slope and
   intercept), rather than inferring coefficients from TREND predictions.
3. On the same inputs, capture the ten `stats=TRUE` cells, especially coefficient
   standard errors and regression/residual sums of squares, to expose the QR
   norm/R-factor and residual accumulation schedule.
4. Add minimal 2- and 3-point non-integer-`Scxx` cases and one cancellation-heavy
   translated pair. Keep all answers absent until the batch hash is frozen.
5. Race coefficient construction first; only after a coefficient survivor
   exists, race TREND publication and freeze a prior-disjoint heldout.

No answer-blind batch was emitted here because the existing scalar bulk-batch
schema cannot express `INDEX(LINEST(...), cell)` or the stats array without a
formula-wrapper extension. That wrapper is the exact prerequisite for the next
serialized oracle round.

## Artifact hashes (SHA-256)

```text
9B3D2A57DB47986C9453A53B19B7EFABAB5DE510F98A30DFE2CF710D670ED668  src/bin/race_regression_linest_offline.rs
28E2A822E85D6188F2FB6874CD0BE2AC92BC574B7CF59BEACE72469B9C035FB1  src/bin/regression_research/common.rs
A0FC652CE2497F1C6A2D258CE9438C3C4E946E4BE6842D490515E49ABAAEC780  src/bin/growth_research/common.rs
FA92800BB9B11CB61987E20993E8E434DF333155D23F45850843ED7092E06D9B  ../../work/w109/G3-03-answers-trend.json
1679D480AABCA00597010A607F92A5B23FA795FA209BAC83710B8406B2B2D85B  ../../work/w109/G3-03-answers-forecast.json
D5DBF6288BB133956C4B1D521D0FDBD3F38C21E0422E9A91D1BFD4A74CFA7EBB  ../../work/w109/G3-03-answers-fc-adv.json
347DF69548C86BCB2FE69AECD7EE1AD8DEE82BE1A105FA65352D79A70BD521B4  ../../work/w109/G3-03-batch-trend.json
424E7C2E060B6CF54B499962CC0424C6DE8F33EC594BED0A7793731244779EDD  ../../work/w109/G3-03-batch-forecast.json
3B06CD24D4A095DA8142B2FC36E80A1FD9C6A6FBA695DD16CBA0A262E2FFFDB5  ../../work/w109/G3-03-batch-fc-adv.json
```

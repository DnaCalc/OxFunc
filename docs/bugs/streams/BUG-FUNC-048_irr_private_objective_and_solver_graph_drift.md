# BUG-FUNC-048: IRR private objective and solver graph drift

## Summary

- **Bug id**: `BUG-FUNC-048`
- **Opened**: `2026-08-09`
- **Status**: `investigating`
- **Owner workset**: `W109`
- **Bead**: `oxf-jwh5.10`

Excel and OxFunc publish different IRR result bits on bounded witnesses. The
current-build discovery establishes that this is not merely worksheet NPV's
publication behavior: Excel's worksheet evaluator applies a cancellation-to-+0
correction to `NPV(...)+c0`, while IRR does not inherit those decisions. The
private IRR objective, scale/error boundary, iteration update, and publication
graph remain unidentified.

## Source refs

- **Reported against ref**: W109 current working tree before `5173fcc`
- **Reproduced on ref**: Excel 16.0 build 20228 x64, workbook Compatibility
  Version 2, `Range.Value2`, `NoCache`
- **Introduced in ref**: unknown
- **Fixed in ref**: not yet fixed
- **Evidence checkpoint**:
  `docs/function-lane/W109_IRR_EXACT_GRAPH_DISCOVERY_CHECKPOINT_20260809.md`

## Ownership and root cause

- **Ownership class**: OxFunc-owned bug
- **Root cause class**: numeric_algorithm_exactness_gap
- **Current root-cause boundary**: the existing IRR objective and solver do not
  reproduce Excel's private calculation graph. The exact missing arithmetic
  association/store schedule is still open; no point correction or least-wrong
  graph is eligible for production.

## Reproduction and current evidence

1. A frozen 300-row IRR discovery set contains 270 numeric and 30 scale-sensitive
   `#NUM!` results under the current reference host.
2. An answer-blind 900-point companion captures raw worksheet NPV, direct
   `NPV+c0`, and referenced-raw-cell `+c0` at the supplied guess and both sides
   of a binary32 `0.001` perturbation in discount-factor space.
3. Direct and referenced-cell composition agree `900/900`. Eighteen nonzero
   binary64 cancellations publish +0. A scale-relative threshold classifies the
   discovery, but its exact constant remains bracketed rather than identified.
4. IRR does not return the supplied guess for any of those 18 rows: 16 publish
   different numeric roots and two publish `#NUM!`.
5. Adding the worksheet evaluator snap to the best reverse worksheet-tail
   objective worsens the guaranteed two-step subset from `40/72` to `37/72`.
6. Raw worksheet NPV has no exact graph in the enumerated family; reverse-Horner
   division leads at `636/900`, max 4 ULP.
7. The best frozen no-snap IRR objective/schedule candidate is only `44/72` on
   the guaranteed two-step subset. The public Microsoft.VisualBasic
   `Financial.IRR` control is `2/300`.

## Guardrails and next work

1. Keep the 180-row heldout answer surface sealed until one coherent objective
   and schedule graph is exact on discovery.
2. Identify the private objective accumulator/store graph before interpreting
   solver stopping or iterate publication.
3. Pin the large-scale `#NUM!` boundary and exact discount-factor perturbation.
4. Only after an answer-blind heldout pass may production, regression tests,
   formal bindings, or closure state be promoted.
5. Continue under the strict clean-room rule: public sources and public Excel
   interfaces only; no Microsoft binary inspection.

## Similar-risk scan

- Worksheet NPV remains a separate open calculation-graph substrate and cannot
  be substituted for the private IRR objective.
- XIRR has an independently bounded publication reconciliation under
  `BUG-FUNC-014`; this record does not reopen or close that distinct lane.
- RATE, YIELD, and ODDFYIELD have separate private objective/schedule records;
  shared solver scaffolding is a hypothesis to test, not an inherited closure.

## Evidence

1. `docs/function-lane/W109_IRR_EXACT_GRAPH_DISCOVERY_CHECKPOINT_20260809.md`
2. `smart-fuzzer/tools/Run-W109IrrNpvObjectiveCompanion.ps1`
3. `smart-fuzzer/tools/calc_graph_racer/src/bin/generate_irr_exact_graph_discriminator.rs`
4. `smart-fuzzer/tools/calc_graph_racer/src/bin/generate_irr_npv_objective_companion.rs`
5. `smart-fuzzer/tools/calc_graph_racer/src/bin/race_irr_reverse_horner.rs`
6. `smart-fuzzer/tools/calc_graph_racer/src/bin/race_irr_npv_objective_companion.rs`

## Status axes

- `scope_completeness`: `scope_partial`
- `target_completeness`: `target_partial`
- `integration_completeness`: `partial`
- `open_lanes`: exact private objective; scale/error boundary; exact solver
  update and publication; frozen heldout; production/tests/formal integration;
  pre-closure audit.

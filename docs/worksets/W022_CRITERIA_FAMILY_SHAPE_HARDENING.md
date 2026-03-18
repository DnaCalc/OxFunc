# WORKSET - Criteria Family Shape Hardening (W22)

## 1. Purpose
Close the remaining criteria-family open lane extracted into `W17` by pinning mismatched-range behavior empirically and aligning the kernel to the observed Excel baseline.

Primary intent:
1. determine which of `COUNTIF`, `COUNTIFS`, `SUMIFS`, `AVERAGEIF`, `AVERAGEIFS`, `MAXIFS`, and `MINIFS` permit top-left-anchored mismatched ranges,
2. implement the observed rule set in OxFunc,
3. replace the old generic “criteria-family anchoring still open” statement with a narrower, evidenced rule.

## 2. Position and Dependencies
Program position:
1. focused semantic-hardening successor under the `W17` umbrella,
2. first concrete closure attempt against the `W16_batch51` residual row,
3. replay-friendly packet candidate after `W021`.

Dependencies:
1. `W004` coercion/reference primitives,
2. `W016` Batch 51 criteria-family baseline,
3. `W017` deferred residual packet,
4. existing Rust/Lean criteria-family substrate.

## 3. Scope
In scope:
1. native Excel replay for mismatched-shape criteria-family lanes,
2. criteria-family kernel hardening for the observed current-baseline rule,
3. targeted unit tests,
4. execution/evidence updates for the `W17` residual row.

Out of scope:
1. `SUMIF`,
2. locale/version sweeps,
3. witness distillation,
4. host-seam functions from the `W17` host-sensitive cluster.

## 4. Working Thesis
The old open issue was too broad.

Current expected narrowing:
1. `AVERAGEIF` permits top-left anchoring of `average_range`,
2. the `*IFS` family remains exact-shape in the current Excel baseline,
3. residual closure depends on proving that distinction rather than keeping a generic “criteria-family shape still open” note.

## 5. Deliverables
1. native criteria-shape scenario manifest and probe runner,
2. persisted result artifact for the current baseline,
3. kernel updates in `criteria_family.rs`,
4. targeted execution record and evidence registration,
5. updated residual wording in `W17`.

## 6. Gate Model
### G1 - Empirical Pinning
Pass when:
1. native Excel rows exist for matched and mismatched lanes,
2. `AVERAGEIF` anchoring and non-anchoring `*IFS` behavior are explicitly evidenced.

### G2 - Kernel Alignment
Pass when:
1. OxFunc matches the pinned rule set,
2. targeted Rust tests cover the split.

### G3 - Residual Narrowing
Pass when:
1. the `W17` criteria-family residual wording is updated from generic openness to explicit closure or any remaining narrower gap,
2. evidence ids and execution records cite the new packet.

## 7. Status
Execution state:
1. `complete`

Claim confidence:
1. `provisional`

Assurance maturity:
1. `exercised-locally`

## 8. Completeness Axes
1. `scope_completeness`: `scope_complete`
2. `target_completeness`: `target_complete`
3. `integration_completeness`: `integrated`
4. `open_lanes`:
   - broader locale/version sweeps remain orthogonal validation work.

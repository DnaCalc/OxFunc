# WORKSET - TUX1000 Floating-Point Characterization (W2)

## 1. Purpose
Characterize Excel floating-point behavior at worksheet-observable boundaries, and pin down observable differences from the Lean executable floating-point model used in this repo.

Focus lanes:
1. `-0` observability and propagation,
2. infinity/NaN handling and normalization,
3. subnormal behavior,
4. formula-eval vs sheet-materialization vs reference-reuse differences.

Comparative policy:
1. prefer empirical parity-first execution and documentation over premature custom FP64 formal theory,
2. explicitly log any Lean-vs-Excel divergence in a dedicated ledger,
3. never claim parity without run artifacts and version scope metadata.

## 2. Kickoff Position and Dependencies
Kickoff role:
1. W2 in `WORKSET_TUX1000_KICKOFF_PROGRAM_W1_W6.md`.

Dependencies:
1. depends on W1 method template.

Downstream consumers:
1. W3 value-universe decisions,
2. W5 `ABS` edge policy,
3. W6 numeric-comparison edge interpretation.

## 3. Scope
In scope:
1. boundary behavior mapping for formula-only, sheet, reference-chain, and interop lanes.
2. version-scoped observation capture.
3. Lean executable-model observation capture for matching scenario classes.
4. explicit Lean-vs-Excel deviation classification and documentation.
5. empirical promotion candidates for stable `EMP-*` findings.

Out of scope:
1. full statistical closure for every function.
2. non-worksheet language domains.

## 4. Required Axes per Observation Row
1. Excel app version/channel.
2. workbook Compatibility Version.
3. locale profile.
4. boundary lane (`FP-A/B/C/D`).
5. reproducibility metadata (runner/tool revision).
6. model source (`excel` or `lean-runtime`) where applicable.

## 5. Deliverables
1. `docs/function-lane/FLOATING_POINT_BEHAVIOR_RESEARCH_NOTES.md` (updated)
2. `docs/function-lane/FLOATING_POINT_SCENARIO_MANIFEST_SEED.csv`
3. `docs/function-lane/FLOATING_POINT_EXECUTION_RECORD.md`
4. `docs/function-lane/LEAN_FLOAT_MODEL_NOTES.md`
5. `docs/function-lane/FLOATING_POINT_LEAN_EXCEL_DEVIATION_LEDGER.csv`
6. `docs/function-lane/FLOATING_POINT_PROBE_RUNTIME_REQUIREMENTS.md`
7. promoted findings candidates and notes for `EMP-*` promotion
8. conformance-row linkage updates (`FDEF-027` and affected rows)

## 6. Gate Model
### G1 - Scenario Closure
Pass when:
1. scenario matrix is explicit and reproducible.

### G2 - Observation Closure
Pass when:
1. at least one declared version/channel and Compatibility Version pass is executed for all required Excel lanes.
2. corresponding Lean-runtime observations are captured for comparable scenario classes.

### G3 - Characterization Closure
Pass when:
1. normalized policy map exists for `-0`, infinities, NaN, and subnormals.
2. Lean-vs-Excel deviation ledger is populated with explicit relation status per comparable scenario.

### G4 - Promotion Closure
Pass when:
1. candidate findings are either promoted or explicitly deferred with rationale.
2. downstream contract touchpoints are identified.

## 7. Status
Execution state:
1. `in_progress`.

Claim confidence:
1. `draft` (pending execution).

# WORKSET - TUX1000 XMATCH Deterministic Quirks Scaffolding (W6)

## 1. Purpose
Formalize and characterize `XMATCH` as a behavior-rich deterministic candidate.

This workset advances a cross-cutting decision:
1. downgrade to lower-interest deterministic class, or
2. retain high-interest classification with explicit evidence-backed rationale.

## 2. Kickoff Position and Dependencies
Kickoff role:
1. W6 in `W000_KICKOFF_PROGRAM_W001_W006.md`.

Dependencies:
1. depends on W3 value-universe closure,
2. depends on W4 coercion/resolver seam closure,
3. depends on W7 string characterization closure for text comparison/matching behavior (baseline satisfied via `W7-STR-BL-20260305`; expansion replays may still refine),
4. consumes W2 numeric-edge findings where comparison behavior is impacted.

## 3. Scope
In scope:
1. full `XMATCH` contract draft (admission, defaults, mode behavior, coercion, arrays, errors).
2. Lean and Rust seed path for declared `XMATCH` obligations.
3. empirical behavior matrix for mode and boundary quirks.
4. explicit final classification decision record.

Out of scope:
1. `XLOOKUP` full closure.

## 4. Required Behavior Lanes
1. admission/default lane,
2. `match_mode` lane,
3. `search_mode` and order lane,
4. coercion/comparison lane (consumes W4 `capability_record_model` seam baseline),
5. array-shape lane,
6. error behavior lane,
7. formula-vs-referenced-input boundary lane.

## 5. Deliverables
1. `docs/function-lane/FUNCTION_SLICE_XMATCH_CONTRACT_PRELIM.md`
2. conformance row updates (`FDEF-031` and related rows)
3. Lean module scaffold + theorem inventory for selected obligations
4. Rust implementation/tests for selected core lanes
5. empirical matrix scaffold and promoted findings where required:
   - `docs/function-lane/XMATCH_SCENARIO_MANIFEST_SEED.csv`
   - `docs/function-lane/XMATCH_PROBE_RUNTIME_REQUIREMENTS.md`
   - `tools/xmatch-probe/*`
6. correlation-ledger row for `FUNC.XMATCH`
7. classification decision note updating interest-tier rationale
8. `docs/function-lane/XMATCH_EXECUTION_RECORD.md`

## 6. Classification Objective
Starting assumption:
1. deterministic,
2. nonvolatile,
3. host interaction `none`,
4. layered FEC dependency declaration:
   - adapter `fec_dependency_profile=none`
   - surface `surface_fec_dependency_profile=ref_only` (provisional, to be validated).

Decision closure requirement:
1. either downgrade tier with evidence,
2. or retain high-interest with explicit bounded rationale and follow-up obligations.

## 7. Gate Model
### G1 - Contract Closure
Pass when:
1. mode/default/error semantics are explicit.

### G2 - Formal and Runtime Closure
Pass when:
1. Lean/Rust scaffolds compile for selected obligations.
2. deterministic behavior obligations are encoded and test-backed.

### G3 - Empirical Closure
Pass when:
1. mode matrix is replayable across declared version axes.

### G4 - Classification Closure
Pass when:
1. tier decision is recorded with evidence bindings and rationale.

## 8. Status
Execution state:
1. `in_progress`.

Claim confidence:
1. `provisional` (scaffold + empirical replay baseline + classification decision recorded, but XMATCH remains semantically incomplete).

Assurance maturity:
1. `exercised`.


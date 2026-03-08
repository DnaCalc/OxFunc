# OxFunc TUX-1000 Plan

## 1. Purpose
`TUX1000_PLAN.md` is the aspirational execution adjunct to `CHARTER.md`.

It defines how OxFunc executes with high rigor while preserving throughput: small slices, full closure, explicit gates, and reusable method infrastructure.

## 2. North-Star Objective
Convert function compatibility from folklore into a repeatable assurance pipeline:
1. explicit contracts,
2. Lean formal obligations,
3. Rust runtime implementation,
4. differential and empirical evidence,
5. promotion gates that block unscoped claims.

## 3. Non-negotiable Operating Constraints
1. Sequence-only planning (no date-based commitments).
2. Dual-axis version scope on behavior claims.
3. Clean-room evidence only.
4. No promotion without replayable artifacts.
5. No hidden ambiguity; unresolved lanes remain explicit and bounded.

## 4. Slice Architecture (Coupled Lanes)
Every function/operator slice must traverse six synchronized artifacts:
1. Contract artifact.
2. Formal artifact.
3. Runtime artifact.
4. Verification artifact.
5. Evidence artifact.
6. Correlation artifact.

Promotion intent:
1. `draft` means incomplete or open-boundary contract.
2. `provisional` means complete shape with bounded unresolved lanes.
3. `validated` means scope-bounded closure with required artifacts and reproducible evidence.

Assurance maturity intent:
1. `exercised` for OxFunc-local closure.
2. `green-validated` for Foundation-level pack closure.

## 5. Kickoff Program (W1-W7)
This is one combined kickoff program, not seven unrelated documents.

### 5.1 W1 - `PI()` End-to-End Method Seed
Purpose:
1. establish the reusable slice template and correlation discipline.

Primary outcome:
1. reusable method pattern proven on a minimal deterministic function.

### 5.2 W2 - Floating-point Characterization
Purpose:
1. characterize IEEE-edge behavior as Excel-observable policy input.

Primary outcome:
1. normalized behavior map for `-0`, infinities, NaN, and subnormal lanes across formula/materialization/reference boundaries.

### 5.3 W3 - Value Universe and Extended Types
Purpose:
1. lock the value algebra used by formal and runtime lanes.

Primary outcome:
1. explicit typed value sets and boundary-specific admissibility model.

### 5.4 W4 - Coercion and Ref->Val Seam
Purpose:
1. formalize coercion primitives and the explicit out-of-model resolver seam.

Primary outcome:
1. one selected baseline seam model plus documented alternatives and tradeoffs.

### 5.5 W5 - `ABS` Full Formality
Purpose:
1. first nontrivial scalar function with full adapter/kernel/array/edge behavior closure.

Primary outcome:
1. complete contract/formal/runtime/evidence closure for `ABS` under declared scope.

### 5.6 W6 - `XMATCH` Deterministic Quirks Closure
Purpose:
1. close a behavior-rich deterministic candidate and settle classification confidence.

Primary outcome:
1. evidence-backed decision: downgrade interest tier or retain high-interest with explicit rationale.

### 5.7 W7 - String Characterization
Purpose:
1. characterize Excel string comparison/normalization/limit behavior via source extraction and empirical runs.

Primary outcome:
1. version-scoped string policy map for comparison semantics, control/unicode behavior, and boundary normalization.

## 6. Dependency Graph and Gate Discipline
Dependencies:
1. W1 has no upstream dependency.
2. W2 depends on W1 method template.
3. W3 depends on W2 characterization baseline.
4. W4 depends on W3 taxonomy closure.
5. W5 depends on W2 + W3 + W4.
6. W6 depends on W3 + W4 + W7 and consumes W2 numeric-edge findings.
7. W7 depends on W1 method template and feeds W3/W6.
8. W3 may start before W7 closure but must absorb W7 outputs before W3 validation closure.

Combined kickoff gates:
1. KG1 Method gate: W1 closure is reusable without ad-hoc process edits.
2. KG2 Numeric-policy gate: W2 yields replayable FP behavior map.
3. KG3 Value-core gate: W3 yields stable value universe and open-question ledger.
4. KG4 Coercion-seam gate: W4 yields selected seam contract plus alternatives.
5. KG5 Function-closure gate: W5 reaches at least `provisional` with complete artifact chain.
6. KG6 Deterministic-quirks gate: W6 records classification decision with evidence.
7. KG7 String semantics gate: W7 yields replayable string characterization and policy map.

## 7. Shared Artifact Contract for Kickoff
Mandatory outputs across W1-W7:
1. workset spec with explicit state and gate status.
2. conformance-row binding updates (`FDEF-*` lineage).
3. function-lane narrative spec updates for each scope area.
4. machine-readable correlation/evidence links where applicable.
5. explicit unknowns register (never implicit drift).

## 8. Foundation Handoff Expectations
For each completed workset, prepare a Foundation-consumable handoff bundle:
1. claimed scope and profile bounds,
2. requirement/evidence bindings,
3. replay artifacts and tool provenance,
4. unresolved-policy notes requiring Foundation decision.

Rule:
1. OxFunc kickoff closure is a precondition for robust Foundation pack integration, not a substitute for it.

## 9. Failure and Divergence Policy
Any divergence discovered during execution must be classified and persisted as one of:
1. spec gap,
2. policy ambiguity,
3. implementation defect,
4. environmental variability.

Each divergence becomes a replayable case and a tracked closure obligation.

## 10. Operating Posture
1. Keep slices small and complete.
2. Prefer one closed chain over wide unverified surface.
3. Treat each validated slice as reusable infrastructure.
4. Expand breadth only when the method itself remains stable.

## 11. Relationship to Doctrine
1. `CHARTER.md` is normative for mission/scope/done criteria.
2. `OPERATIONS.md` is normative for execution doctrine.
3. This plan is aspirational and directional, never overriding charter or Foundation doctrine.

## 12. Post-kickoff Extension Packets
1. `W8.1` String follow-up checklist:
   - `docs/worksets/WORKSET_TUX1000_STRING_W8_1_CHECKLIST.md`
2. `W9` XLL add-in bridge packet:
   - `docs/worksets/WORKSET_TUX1000_XLL_ADDIN_BRIDGE.md`
   - goal: build `OxFunc64.xll` as an adapter around OxFunc core functions, with seed exports (for example `ox_ABS`) and side-by-side native-vs-OxFunc workbook validation packs.

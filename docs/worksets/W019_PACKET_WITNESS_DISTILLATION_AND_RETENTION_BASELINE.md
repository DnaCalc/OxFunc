# WORKSET - TUX1000 Packet Witness Distillation and Retention Baseline (W19)

## 1. Purpose
Define the OxFunc-local baseline for packet-first witness distillation, lifecycle, retention, and quarantine policy without overclaiming replay-valid reduced witnesses.

Primary intent:
1. define OxFunc reduction-unit hierarchy for packet and row witnesses,
2. bind Foundation predicate, lifecycle, and reduction-status vocabulary into OxFunc packet doctrine,
3. define retention, supersession, and quarantine rules for future reduced witnesses,
4. prepare the evidence requirements for a future honest `cap.C4.distill_valid` claim.

## 2. Position and Dependencies
Program position:
1. immediate successor to `W018`,
2. replay-rollout governance packet rather than direct function-semantic breadth,
3. packet-first support work that can later serve `W017` and future interesting-function replay packets.

Dependencies:
1. `W018` packet adapter baseline and capability manifest,
2. Foundation witness-distillation, predicate-registry, and witness-lifecycle handoff docs.

## 3. Scope
In scope:
1. packet-level and row-level reduction planning,
2. lifecycle state usage rules,
3. retention, quarantine, and supersession policy,
4. limitation-aware reduced-witness doctrine,
5. explicit evidence requirements for future promotion.

Out of scope:
1. pack-grade witness promotion,
2. claiming reduced-witness replay validity without proof,
3. generic source rewrites beyond lane-declared replay-safe transforms,
4. any claim of `cap.C5.pack_valid`.

## 4. Working Thesis
OxFunc witness distillation must remain packet-first and row-first.

That means:
1. a reduced witness is valid only if packet replay identity remains intact,
2. retained rows must keep their manifest definition, run labels, compatibility descriptors, evidence ids, and limitation bindings,
3. explanatory-only or quarantined witnesses cannot be treated as pack-grade evidence,
4. reduction policy is lane-owned even when executed through shared Replay tooling.

## 5. Deliverables
1. workset baseline defining packet-first reduction units,
2. lifecycle, quarantine, and supersession doctrine for OxFunc reduced witnesses,
3. canonical-doc updates binding replay bundles, invariants, predicates, and retention expectations,
4. explicit evidence threshold for any future `cap.C4.distill_valid` claim.

## 6. Gate Model
### G1 - Reduction Unit Closure
Pass when:
1. packet, row cluster, row, summary, invariant, limitation, and sidecar partition units are explicit,
2. closure dependencies are explicit,
3. fake event-stream reduction is prohibited.

### G2 - Lifecycle and Quarantine Closure
Pass when:
1. witness lifecycle states are bound into OxFunc docs,
2. retention and supersession policy is explicit,
3. quarantine expectations are explicit.

### G3 - Evidence Threshold Closure
Pass when:
1. the next proving artifacts for packet-first reduced witnesses are explicit,
2. the workset does not overclaim `cap.C4`,
3. pack-grade promotion remains out of scope.

## 7. Status
Execution state:
1. `planned`

Claim confidence:
1. `provisional`

Assurance maturity:
1. `not-yet-exercised`

## 8. Completeness Axes
1. `scope_completeness`: `scope_partial`
2. `target_completeness`: `target_partial`
3. `integration_completeness`: `partial`
4. `open_lanes`:
   - no reduced packet or row witness has yet been proven replay-valid,
   - no explicit irreducibility or unstable-oracle OxFunc witness case has yet been recorded,
   - no pack-grade witness export is in scope.

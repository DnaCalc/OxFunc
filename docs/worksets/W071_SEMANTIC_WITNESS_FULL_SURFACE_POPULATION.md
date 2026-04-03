# WORKSET - Semantic Witness Full-Surface Population (W71)

## 1. Purpose
Populate the remaining supported non-deferred OxFunc surface with actual
semantic-witness detail so downstream consumers can query help, signatures,
evidence, and formal references for the whole supported baseline.

This workset is the execution successor to the closed W069 framework packet.
W069 established the witness schema, tranche freeze, seed artifacts, and
publication rules. W071 carries the actual row-filling program for the
remaining supported surface.

## 2. Why This Packet Exists
Current OxFunc has:
1. a parked `V1` catalog/profile export,
2. a live `V2` witness schema and publication convention,
3. `70` seeded witness rows,
4. a frozen remaining-surface gap inventory for `447` supported rows,
5. a tranche register that partitions the remaining rows into deterministic
   ordinary, curated, operator, and seam-heavy lanes.

What OxFunc still does not have is a filled semantic-detail surface for the
remaining supported rows. The framework exists. The witness rows do not yet.

This workset exists to turn that gap into executed row coverage.

## 3. Product Thesis
The right next move is to bulk-populate the witness surface, tranche by
tranche, until every supported non-deferred row has a real witness payload and
the final reconciliation shows no leftover supported-row witness gap.

This workset is accretive because it makes every already-supported function more
useful to downstream consumers without changing the established `V1` identity
surface.

## 4. Inherited Framework
W071 inherits the frozen framework surfaces from W069:
1. [W69_SUPPORTED_SURFACE_WITNESS_GAP_LEDGER.md](../function-lane/W69_SUPPORTED_SURFACE_WITNESS_GAP_LEDGER.md)
2. [W69_SUPPORTED_SURFACE_WITNESS_GAP_INVENTORY.csv](../function-lane/W69_SUPPORTED_SURFACE_WITNESS_GAP_INVENTORY.csv)
3. [W69_SUPPORTED_SURFACE_WITNESS_TRANCHE_LEDGER.md](../function-lane/W69_SUPPORTED_SURFACE_WITNESS_TRANCHE_LEDGER.md)
4. [W69_SUPPORTED_SURFACE_WITNESS_TRANCHE_REGISTER.csv](../function-lane/W69_SUPPORTED_SURFACE_WITNESS_TRANCHE_REGISTER.csv)
5. [W69_SEAM_HEAVY_SUPPORTED_SURFACE_LEDGER.md](../function-lane/W69_SEAM_HEAVY_SUPPORTED_SURFACE_LEDGER.md)
6. [W69_SEAM_HEAVY_SUPPORTED_SURFACE_INVENTORY.csv](../function-lane/W69_SEAM_HEAVY_SUPPORTED_SURFACE_INVENTORY.csv)
7. [W69_SEAM_HEAVY_WITNESS_AUTHORING_RULES.md](../function-lane/W69_SEAM_HEAVY_WITNESS_AUTHORING_RULES.md)
8. [W69_OPERATOR_AND_MODELED_WITNESS_CONVENTIONS.md](../function-lane/W69_OPERATOR_AND_MODELED_WITNESS_CONVENTIONS.md)
9. [W69_FINAL_SUPPORTED_SURFACE_COVERAGE_RECONCILIATION.json](../function-lane/W69_FINAL_SUPPORTED_SURFACE_COVERAGE_RECONCILIATION.json)
10. [W69_FINAL_SUPPORTED_SURFACE_COVERAGE_LEDGER.md](../function-lane/W69_FINAL_SUPPORTED_SURFACE_COVERAGE_LEDGER.md)

The remaining population work starts from the frozen tranche order and the
remaining supported-surface gap counts already pinned there.

## 5. Coverage Target
W071 owns the remaining semantic witness rollout for the parked supported
baseline.

Target surface:
1. current supported non-deferred rows: `517`
2. current witness-covered rows: `85`
3. current remaining supported rows needing witness population: `432`
4. deferred rows in `W050`: excluded until intentionally reopened

The rollout target is not a second catalog.
It is the actual semantic-detail population of the remaining supported surface.

## 6. Tranche Shape
The remaining supported surface is frozen into the following rollout lanes:
1. `T1` ordinary extracted non-operator rows
2. `T2` ordinary curated non-operator rows
3. `T3` operator surface
4. `T4` special extracted surface, dependency-gated by retained live authorities
5. `T5` special curated surface, dependency-gated by retained live authorities

W071 should populate real witness rows for each lane, keeping dependency gates
explicit where the row remains seam-sensitive.

## 7. Initial Deliverables
This packet should produce:
1. tranche-level generation rules for the remaining 447 rows,
2. actual witness-row batches for the ordinary extracted tranche,
3. actual witness-row batches for the ordinary curated tranche,
4. actual witness-row batches for the operator and seam-heavy tranches,
5. a frozen gap reconciliation artifact proving that the remaining supported
   surface still matches the tranche control surface before row-filling starts,
6. a final reconciliation draft proving the eventual publication shape for the
   remaining supported surface,
7. a final reconciliation artifact proving that the remaining supported surface
   has been populated in witness form.

## 8. Design Rules
1. Every new witness row must be keyed by the existing `surface_stable_id`.
2. The `V1` identity surface must remain unchanged.
3. Witness rows must keep support status, admitted-slice evidence, and
   orthogonal validation separate.
4. Seamed or dependency-gated rows must keep the retained authority explicit.
5. The rollout must stay deterministic across runs.
6. The final publication surface must reconcile against the frozen gap ledger
   and tranche register.

## 9. Closure Condition
W071 may claim completion only when:
1. the remaining supported rows in the frozen W69 gap inventory have actual
   witness rows or explicit dependency-gated witness records,
2. the final coverage reconciliation shows no leftover supported-row witness
   gap in the parked baseline,
3. downstream reading/index docs point at the populated witness surface rather
   than only at the frozen framework ledgers.

## 10. Notes
W069 is the closed framework and seed packet.
W071 is the live execution workset for filling the remaining semantic detail
surface.

The first ordinary curated witness batch is seeded in:
1. [OXFUNC_SEMANTIC_WITNESS_SNAPSHOT_V2_W71_TRANCHE_T2_BATCH1.json](../function-lane/OXFUNC_SEMANTIC_WITNESS_SNAPSHOT_V2_W71_TRANCHE_T2_BATCH1.json)

The second ordinary curated witness batch is seeded in:
1. [OXFUNC_SEMANTIC_WITNESS_SNAPSHOT_V2_W71_TRANCHE_T2_BATCH2.json](../function-lane/OXFUNC_SEMANTIC_WITNESS_SNAPSHOT_V2_W71_TRANCHE_T2_BATCH2.json)

The first ordinary extracted witness batch is seeded in:
1. [OXFUNC_SEMANTIC_WITNESS_SNAPSHOT_V2_W71_TRANCHE_T1_BATCH1.json](../function-lane/OXFUNC_SEMANTIC_WITNESS_SNAPSHOT_V2_W71_TRANCHE_T1_BATCH1.json)

The second ordinary extracted witness batch is seeded in:
1. [OXFUNC_SEMANTIC_WITNESS_SNAPSHOT_V2_W71_TRANCHE_T1_BATCH2.json](../function-lane/OXFUNC_SEMANTIC_WITNESS_SNAPSHOT_V2_W71_TRANCHE_T1_BATCH2.json)

The third ordinary extracted witness batch is seeded in:
1. [OXFUNC_SEMANTIC_WITNESS_SNAPSHOT_V2_W71_TRANCHE_T1_BATCH3.json](../function-lane/OXFUNC_SEMANTIC_WITNESS_SNAPSHOT_V2_W71_TRANCHE_T1_BATCH3.json)

The fourth ordinary extracted witness batch is seeded in:
1. [OXFUNC_SEMANTIC_WITNESS_SNAPSHOT_V2_W71_TRANCHE_T1_BATCH4.json](../function-lane/OXFUNC_SEMANTIC_WITNESS_SNAPSHOT_V2_W71_TRANCHE_T1_BATCH4.json)

The fifth ordinary extracted witness batch is seeded in:
1. [OXFUNC_SEMANTIC_WITNESS_SNAPSHOT_V2_W71_TRANCHE_T1_BATCH5.json](../function-lane/OXFUNC_SEMANTIC_WITNESS_SNAPSHOT_V2_W71_TRANCHE_T1_BATCH5.json)

The sixth ordinary extracted witness batch is seeded in:
1. [OXFUNC_SEMANTIC_WITNESS_SNAPSHOT_V2_W71_TRANCHE_T1_BATCH6.json](../function-lane/OXFUNC_SEMANTIC_WITNESS_SNAPSHOT_V2_W71_TRANCHE_T1_BATCH6.json)

The seventh ordinary extracted witness batch is seeded in:
1. [OXFUNC_SEMANTIC_WITNESS_SNAPSHOT_V2_W71_TRANCHE_T1_BATCH7.json](../function-lane/OXFUNC_SEMANTIC_WITNESS_SNAPSHOT_V2_W71_TRANCHE_T1_BATCH7.json)

The eighth ordinary extracted witness batch is seeded in:
1. [OXFUNC_SEMANTIC_WITNESS_SNAPSHOT_V2_W71_TRANCHE_T1_BATCH8.json](../function-lane/OXFUNC_SEMANTIC_WITNESS_SNAPSHOT_V2_W71_TRANCHE_T1_BATCH8.json)

The ninth ordinary extracted witness batch is seeded in:
1. [OXFUNC_SEMANTIC_WITNESS_SNAPSHOT_V2_W71_TRANCHE_T1_BATCH9.json](../function-lane/OXFUNC_SEMANTIC_WITNESS_SNAPSHOT_V2_W71_TRANCHE_T1_BATCH9.json)

The tenth ordinary extracted witness batch is seeded in:
1. [OXFUNC_SEMANTIC_WITNESS_SNAPSHOT_V2_W71_TRANCHE_T1_BATCH10.json](../function-lane/OXFUNC_SEMANTIC_WITNESS_SNAPSHOT_V2_W71_TRANCHE_T1_BATCH10.json)

The third ordinary curated witness batch is seeded in:
1. [OXFUNC_SEMANTIC_WITNESS_SNAPSHOT_V2_W71_TRANCHE_T2_BATCH3.json](../function-lane/OXFUNC_SEMANTIC_WITNESS_SNAPSHOT_V2_W71_TRANCHE_T2_BATCH3.json)

The third operator witness batch is seeded in:
1. [OXFUNC_SEMANTIC_WITNESS_SNAPSHOT_V2_W71_TRANCHE_T3_BATCH3.json](../function-lane/OXFUNC_SEMANTIC_WITNESS_SNAPSHOT_V2_W71_TRANCHE_T3_BATCH3.json)

The third seam-heavy witness batch is seeded in:
1. [OXFUNC_SEMANTIC_WITNESS_SNAPSHOT_V2_W71_TRANCHE_SH1_BATCH3.json](../function-lane/OXFUNC_SEMANTIC_WITNESS_SNAPSHOT_V2_W71_TRANCHE_SH1_BATCH3.json)

The fourth seam-heavy witness batch is seeded in:
1. [OXFUNC_SEMANTIC_WITNESS_SNAPSHOT_V2_W71_TRANCHE_SH1_BATCH4.json](../function-lane/OXFUNC_SEMANTIC_WITNESS_SNAPSHOT_V2_W71_TRANCHE_SH1_BATCH4.json)

The first operator witness batch is seeded in:
1. [OXFUNC_SEMANTIC_WITNESS_SNAPSHOT_V2_W71_TRANCHE_T3_BATCH1.json](../function-lane/OXFUNC_SEMANTIC_WITNESS_SNAPSHOT_V2_W71_TRANCHE_T3_BATCH1.json)

The second operator witness batch is seeded in:
1. [OXFUNC_SEMANTIC_WITNESS_SNAPSHOT_V2_W71_TRANCHE_T3_BATCH2.json](../function-lane/OXFUNC_SEMANTIC_WITNESS_SNAPSHOT_V2_W71_TRANCHE_T3_BATCH2.json)

The first seam-heavy witness batch is seeded in:
1. [OXFUNC_SEMANTIC_WITNESS_SNAPSHOT_V2_W71_TRANCHE_SH1_BATCH1.json](../function-lane/OXFUNC_SEMANTIC_WITNESS_SNAPSHOT_V2_W71_TRANCHE_SH1_BATCH1.json)

The second seam-heavy witness batch is seeded in:
1. [OXFUNC_SEMANTIC_WITNESS_SNAPSHOT_V2_W71_TRANCHE_SH1_BATCH2.json](../function-lane/OXFUNC_SEMANTIC_WITNESS_SNAPSHOT_V2_W71_TRANCHE_SH1_BATCH2.json)

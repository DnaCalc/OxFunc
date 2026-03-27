# WORKSET - Lean Formalization Gap Reconciliation (W54)

## 1. Purpose
Turn the current Lean formalization coverage gaps into an explicit tracked packet instead of leaving them as ad hoc inventory output.

This packet exists to:
1. pin the current Rust-vs-Lean function-id gap set,
2. separate genuine missing Lean function rows from inventory noise such as internal base metas and legacy/modern naming mismatches,
3. drive the next formalization closure wave in a bounded, reviewable order.

## 2. Provenance
Opened on `2026-03-27` after a direct function-id comparison between:
1. `crates/oxfunc_core/src/functions/**/*.rs`
2. `formal/lean/OxFunc/Functions/**/*.lean`

Comparison rule used for the opening inventory:
1. collect quoted `FUNC.*` ids from Rust function metadata,
2. collect quoted `FUNC.*` ids from Lean function metadata/alignment modules,
3. subtract Lean ids from Rust ids,
4. exclude internal non-function/base/sentinel rows from the opening to-do set.

Relevant context:
1. `README.md`
2. `OPERATIONS.md`
3. `docs/IN_PROGRESS_FEATURE_WORKLIST.md`
4. `docs/function-lane/FORMALIZATION_STRATEGY_EXECUTABLE_SEMANTIC_MODEL.md`
5. `formal/lean/OxFunc/Functions/*`

## 3. Scope
Machine-readable inventory:
1. `docs/function-lane/W54_LEAN_FORMALIZATION_GAP_INVENTORY.csv`

Current total:
1. `37` function rows in the opening gap set.

Current remaining after the first reconciled wave:
1. `32` function rows still marked missing or alias-misaligned in the inventory.

Families in the opening gap set:
1. beta/gamma stats modern + legacy ids,
2. chi/f/t distribution and inverse variants,
3. financial time-value follow-on rows,
4. host/service-shaped functions,
5. modern dotted variance/stdev ids whose legacy compat Lean rows already exist.

## 4. Inventory Rules
Included:
1. Rust function ids with no matching quoted Lean `functionId` row.
2. Rows where the current repo reading is still "modern function id missing in Lean" even if a nearby legacy compat Lean row exists.

Excluded from this packet's opening count:
1. internal base metas such as `*_BASE`,
2. helper-family scaffolding ids whose purpose is substrate reuse rather than surface function coverage,
3. the `FUNC.UNKNOWN` sentinel,
4. pure naming-drift false positives already covered by exact matching Lean ids.

## 5. Why This Packet Matters
1. The repo doctrine requires Lean/formal alignment for admitted current-phase closure claims.
2. The current formalization register is still too coarse; it says substrate-level work exists, but not which function ids remain uncovered.
3. A pinned gap packet gives us an honest owner for formalization follow-on without confusing runtime completeness with formal coverage completeness.

## 6. In Scope
1. maintain the canonical gap inventory for missing Lean function ids,
2. classify each row as:
   - genuine missing Lean row,
   - covered via nearby substrate with missing modern alias/id,
   - blocked on unresolved function-scope doctrine,
3. add or align Lean function metadata/alignment modules for the opening gap set,
4. update workset and execution records when a row moves out of the gap inventory.

## 7. Out Of Scope
1. changing Rust function semantics just to match Lean inventory,
2. treating substrate-level Lean coverage as absent when the opening review already established a valid aligned substrate owner,
3. locale/version sweep work,
4. OxFml seam work unless a specific formalization row depends on a cross-repo seam decision.

## 8. Gate Criteria
This packet can only be reported `scope_complete` when:
1. every row in `W54_LEAN_FORMALIZATION_GAP_INVENTORY.csv` is reconciled as either:
   - Lean-added,
   - alias-covered and explicitly documented,
   - extracted to a successor packet with explicit owner,
2. the remaining formalization-deepening register reflects the reduced gap honestly,
3. no function in the opening W54 inventory is still silently missing from Lean coverage.

## 9. Initial Status
1. execution_state: `in_progress`
2. scope_completeness: `scope_partial`
3. target_completeness: `target_partial`
4. integration_completeness: `partial`
5. open_lanes:
   - beta/gamma stats family Lean coverage
   - chi/f/t distribution and inverse family Lean coverage
   - finance follow-on Lean coverage beyond the current shared publication substrate
   - host/service function Lean coverage for `HYPERLINK` and `RTD`
   - modern dotted `STDEV.P` / `VAR.P` id alignment

Progress note:
1. The first `W054` finance wave reconciled `FV`, `NPER`, `IPMT`, `PPMT`, and `ISPMT` into `formal/lean/OxFunc/Functions/FinancialTimeValueFamily.lean`.
2. The direct Rust-vs-Lean function-id diff now reports `32` remaining rows after applying the packet exclusions.

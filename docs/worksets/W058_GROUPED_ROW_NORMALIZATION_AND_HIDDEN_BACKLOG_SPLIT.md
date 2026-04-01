# WORKSET - Grouped-Row Normalization And Hidden Backlog Split (W58)

## 1. Purpose
Normalize the hidden ordinary `W051` backlog from snapshot-facing grouped catalog entries into machine-clean execution rows so the remaining ordinary backlog can be implemented family-by-family without ambiguous row identity.

This packet exists to:
1. preserve the first-pass `185`-entry snapshot reading for downstream consumers,
2. derive the machine-clean execution backlog from that same source,
3. split the seven grouped text-compat rows into explicit function members,
4. freeze exact successor-packet ownership counts for `W059` through `W068`.

## 2. Position and Dependencies
Program position:
1. post-`W057` packet (`W58`).

Dependencies:
1. `W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md`
2. `W057_HIDDEN_ORDINARY_BACKLOG_SYSTEMATIC_COMPLETION_PLAN.md`
3. `docs/function-lane/W51_HIDDEN_NON_DEFERRED_BACKLOG_FIRST_PASS.csv`
4. `docs/function-lane/W57_PACKET_REGISTER.csv`
5. `W044_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_BASELINE.md`

## 3. Scope
In scope for `W58`:
1. preserve the original `185` hidden snapshot entries as provenance,
2. split the grouped snapshot rows:
   - `FIND, FINDB`
   - `LEFT, LEFTB`
   - `LEN, LENB`
   - `MID, MIDB`
   - `REPLACE, REPLACEB`
   - `RIGHT, RIGHTB`
   - `SEARCH, SEARCHB`
3. emit the authoritative normalized execution inventory,
4. freeze exact successor-packet ownership counts after normalization,
5. update `W051` and downstream summary docs so they distinguish snapshot-facing and execution-facing counts.

Out of scope for `W58`:
1. semantic implementation of the normalized rows,
2. promotion of any normalized row to supported,
3. changing the published `W044` snapshot export artifact itself,
4. revisiting the `17` deferred rows in `W050`.

## 4. Inputs And Outputs
Input artifacts:
1. `docs/function-lane/W51_HIDDEN_NON_DEFERRED_BACKLOG_FIRST_PASS.csv`
2. `docs/function-lane/W57_PACKET_REGISTER.csv`

Output artifacts:
1. `docs/function-lane/W58_HIDDEN_ORDINARY_BACKLOG_NORMALIZED.csv`
2. `docs/function-lane/W58_GROUPED_ROW_NORMALIZATION_MAP.csv`
3. `docs/function-lane/W58_SUCCESSOR_PACKET_SPLIT.csv`
4. `docs/function-lane/W58_EXECUTION_RECORD.md`
5. updated `W051` and downstream summary docs

## 5. Gate Criteria
`W58` is complete when:
1. the first-pass `185` hidden snapshot entries remain preserved as provenance,
2. the normalized execution inventory contains `192` function rows,
3. the grouped-row normalization map contains `14` split-member rows derived from the `7` grouped snapshot entries,
4. every normalized row has exactly one successor execution owner in `W059` through `W068`,
5. downstream summary docs state both truths clearly:
   - `185` hidden backlog snapshot entries for current published-catalog reading,
   - `192` normalized execution rows for the ordinary backlog implementation program.

## 6. Result
`W58` resolves the hidden-backlog identity ambiguity without changing the consumer-facing snapshot count.

The frozen normalized successor-packet split is:
1. `W059`: `16`
2. `W060`: `26`
3. `W061`: `29`
4. `W062`: `35`
5. `W063`: `18`
6. `W064`: `15`
7. `W065`: `12`
8. `W066`: `23`
9. `W067`: `15`
10. `W068`: `3`

## 7. Status
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`
5. open_lanes:
   - semantic closure work for normalized successor packets `W059` through `W068` has not started yet

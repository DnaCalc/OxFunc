# WORKSET - Corpus IF Condition And Numeric Comparison Tolerance (W077)

## 1. Purpose
Own the OxFunc-side follow-on requested by `HANDOFF-OXFUNC-003`: correct the
current corpus read around `IF` empty-text conditions, characterize Excel's
observed floating numeric comparison tolerance, and land the bounded local
runtime/test/doctrine updates for the families that actually share that
semantics.

## 2. Why This Packet Exists
The incoming OxFml handoff named two remaining semantic lanes:
1. `IF("",1,2)` condition coercion,
2. ordinary numeric comparison equality for floating results.

Current-head Excel verification changed that packet shape:
1. `IF("",1,2)` and `IFS("",1,TRUE,2)` both return `#VALUE!`, so the handoff's
   false-branch expectation was wrong and the first job is local empirical pin
   plus upstream correction,
2. ordinary numeric comparisons do show tolerant equality/order behavior, and
   the stronger arithmetic-generated boundary rows are consistent with a
   truncation-style 15-significant-digit comparison normalization rather than
   round-to-nearest,
3. adjacent local families split:
   - `COUNTIF`/`SUMIF` and database criteria matching share the tolerant lane,
   - `SWITCH` shares the tolerant exact-match lane,
   - `MATCH`/`XMATCH`/`DELTA` exact-match paths remain exact on the tested
     near-equality cases.

## 3. Provenance
1. `../OxFml/docs/handoffs/HANDOFF-OXFUNC-003_CORPUS_IF_EMPTY_TEXT_AND_FLOAT_COMPARE.md`
2. `../OxFml/docs/upstream/NOTES_FOR_OXFUNC.md`
3. `docs/function-lane/FLOATING_POINT_BEHAVIOR_RESEARCH_NOTES.md`
4. `docs/function-lane/FLOATING_POINT_EXECUTION_RECORD.md`
5. `docs/function-lane/FUNCTION_SLICE_OPERATOR_COMPARE_CONCAT_FAMILY_CONTRACT_PRELIM.md`
6. `docs/function-lane/FUNCTION_SLICE_CRITERIA_FAMILY_CONTRACT_PRELIM.md`
7. `docs/function-lane/FUNCTION_SLICE_DATABASE_FAMILY_CONTRACT_PRELIM.md`

## 4. Scope
In scope:
1. record the incoming corpus handoff as bug reports and canonical local bug
   stream(s),
2. pin `IF` / `IFS` empty-text condition behavior empirically and reconcile the
   local bug read honestly,
3. add empirical floating-point probe lanes for operator comparisons,
   criteria/database selection, `SWITCH`, and exact-match contrast cases,
   including arithmetic-generated 15-significant-digit boundary rows,
4. land one shared local numeric comparison helper for the tolerant families
   actually supported by the observed Excel behavior, including the stronger
   arithmetic-generated boundary lane,
5. keep `MATCH`, `XMATCH`, and `DELTA` on their current exact-match lanes,
6. refresh current-gap truth surfaces and file the OxFunc -> OxFml reply packet.

Out of scope:
1. speculative changes to every numeric comparison in the repo,
2. alternate locale or Excel-channel sweeps beyond the current installed
   baseline,
3. changing `MATCH` / `XMATCH` exact-match semantics without contrary Excel
   evidence,
4. broader floating-point normalization policy outside comparison semantics.

## 5. Initial Epic Lanes
1. handoff intake and bug triage
2. empirical Excel sweep and floating-point lane expansion
3. tolerant-family runtime correction
4. focused local validation
5. downstream reply handoff

## 6. Closure Condition
`W077` is complete for declared scope only when:
1. the `IF` empty-text report is either fixed or honestly closed as no local
   bug,
2. the tolerant numeric comparison family split is pinned with reproducible
   local Excel evidence, including arithmetic-generated 15-significant-digit
   boundary rows,
3. local runtime helpers match that split across operators, criteria/database,
   and `SWITCH`, without diverging from the stronger boundary evidence,
4. exact-match contrast families remain explicitly pinned,
5. focused local validation is recorded,
6. the OxFunc -> OxFml reply packet is filed.

## 7. Current Reading
1. execution_state: `in_progress`
2. scope_completeness: `scope_partial`
3. target_completeness: `target_partial`
4. integration_completeness: `integrated`
5. open_lanes:
   - landed-ref promotion for the local numeric comparison tolerance packet
   - downstream OxFml acknowledgment under the reply handoff

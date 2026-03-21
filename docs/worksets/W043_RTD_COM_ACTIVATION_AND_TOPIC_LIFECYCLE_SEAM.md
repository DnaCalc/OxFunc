# WORKSET - RTD COM Activation And Topic Lifecycle Seam (W43)

## 1. Purpose
Split `RTD` out of the generic external-provider packet and treat it as its own seam-focused workset.

`RTD` is singular because its main complexity is not ordinary external-service fetching. It is the surrounding Excel machinery:
1. COM activation,
2. topic-string subscription,
3. topic lifetime tracking,
4. update-triggered invalidation and recalc,
5. value projection from the RTD server back into the worksheet formula.

## 2. Provenance
Opened after:
1. `W041` grouped `RTD` with general provider/cube functions,
2. the current review concluded that `RTD` should not stay bundled there,
3. local reference captures were stored for the Microsoft RTD FAQ and the Excel-DNA RTD tutorial.

Relevant context:
1. `docs/function-lane/RTD_REFERENCE_CAPTURE_AND_SEAM_NOTES.md`
2. `docs/function-lane/FUNCTION_SLICE_RTD_CONTRACT_PRELIM.md`
3. `docs/function-lane/W43_EXECUTION_RECORD.md`
4. `docs/function-lane/EXCEL_FUNCTION_DEFINITION_DISCUSSION.md`
5. `docs/function-lane/EXCEL_FUNCTION_DEFINITION_PRELIM_SPEC.md`
6. `docs/worksets/W041_EXTERNAL_DATA_PROVIDER_AND_CUBE_FUNCTIONS.md`

## 3. Scope
Machine-readable inventory:
1. `docs/function-lane/W43_RTD_COM_AND_TOPIC_LIFECYCLE_INVENTORY.csv`

Current total:
1. `1` function.

Members:
1. `RTD`

## 4. Why This Packet Matters
1. `RTD` exposes one of Excel's clearest hidden-machinery seams.
2. It pressures the boundary between formula semantics and host/application runtime responsibilities.
3. It is a strong test of the availability / capability / provider-failure taxonomy without pretending the function is a pure local kernel.
4. It should help refine the OxFml <-> OxFunc seam without overcomplicating OxFunc.

## 5. In Scope
1. canonical local reference capture and extraction,
2. explicit packet-level statement of what belongs above OxFunc versus inside OxFunc,
3. empirical characterization of current-baseline worksheet outcomes where practical,
4. explicit modeling of topic strings, current value projection, and classified outcome surfaces,
5. handoff/seam preparation for OxFml where needed.

## 6. Out Of Scope
1. implementing a COM RTD server,
2. owning topic subscription tables inside OxFunc,
3. hosting callback threading or `UpdateNotify` dispatch inside OxFunc,
4. pretending `RTD` is just another web/provider fetch function,
5. bundling `RTD` back into `W041`.

## 7. Gate Criteria
This workset can only be reported `scope_complete` when:
1. the Microsoft and Excel-DNA references are captured locally and cited,
2. the OxFunc-side role is stated minimally and explicitly,
3. the OxFml <-> OxFunc seam for `RTD` request shape and result shape is documented honestly,
4. topic lifetime, invalidation, and callback responsibilities are explicitly assigned above OxFunc,
5. any residual empirical uncertainty is either probed or pushed into an explicit successor packet.

## 8. Status
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`
5. open_lanes:
   - none in declared `W043` OxFunc-side scope

## 9. Current Closure Reading
`W043` is complete on the OxFunc side because:
1. the reference corpus is captured locally,
2. the minimal `RtdRequest` / `RtdProvider` / `RtdProviderResult` seam is implemented and tested,
3. lifecycle/state ownership is explicitly assigned above OxFunc,
4. the current OxFml sync bundle is explicit.

The following are intentionally not counted as OxFunc-local open lanes:
1. RTD server activation,
2. topic connect/disconnect ownership,
3. cell-topic map ownership,
4. host-side callback scheduling,
5. workbook-saved-value and reconnect policy.

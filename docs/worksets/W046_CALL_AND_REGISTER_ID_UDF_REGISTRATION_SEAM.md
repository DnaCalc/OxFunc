# WORKSET - CALL and REGISTER.ID UDF Registration Seam (W46)

## 1. Purpose
Own the `CALL` / `REGISTER.ID` pair as a distinct worksheet-to-UDF-registration seam packet rather than leaving them mixed into generic host metadata work.

These functions are not ordinary worksheet kernels. They sit on the boundary between formula evaluation, XLL/C API registration, DLL/code-resource lookup, and broader add-in/VBA automation worlds.

## 2. Provenance
Opened as an extraction from `W023` after the host/database packet was narrowed.

Relevant context:
1. `docs/worksets/W023_DEFERRED_HOST_METADATA_AND_DATABASE_FUNCTIONS.md`
2. `tools/fp-probe/xll/README.md`
3. `tools/fp-probe/xll/FpEdgeHarnessContract.md`
4. `docs/function-lane/FUNCTION_CATALOG_CURRENT_BASELINE_LOCAL.csv`
5. `docs/function-lane/XLCALL_CODE_CATALOG.csv`
6. `docs/function-lane/FUNCTION_SLICE_CALL_REGISTER_ID_UDF_REGISTRATION_SEAM_PRELIM.md`

## 3. Scope
Machine-readable inventory:
1. `docs/function-lane/W46_CALL_REGISTER_ID_INVENTORY.csv`

Current total:
1. `2` functions.

Members:
1. `CALL`
2. `REGISTER.ID`

## 4. Why This Packet Matters
1. the pair is really about registration and invocation of externally provided routines, not about ordinary worksheet semantics.
2. OxFunc already has XLL registration knowledge in the repo through `xlfRegister`-based harness/add-in plumbing, but that is not the same thing as worksheet `CALL` / `REGISTER.ID` semantics.
3. the packet should make the split explicit between:
   - XLL/C API self-registration infrastructure already exercised in repo tooling,
   - worksheet formulas that reach into registered DLL/code-resource surfaces,
   - future VBA/Automation/UDF broader surface characterization.

## 5. In Scope
1. classify the current repo floor for registration plumbing that already exists,
2. define the minimal OxFml ↔ host ↔ OxFunc seam for worksheet `CALL` / `REGISTER.ID`,
3. ingest the local `XLCALL.H` built-in function identities into the OxFunc catalog surfaces,
4. decide what is intentionally out of current supported scope,
5. avoid treating these functions as ordinary host-query leftovers.

## 6. Out Of Scope
1. full DLL/code-resource invocation support,
2. VBA/Automation/UDF broad-surface characterization beyond the registration seam,
3. pretending the existing XLL registration evidence already closes worksheet `CALL` / `REGISTER.ID`.

## 7. Initial Status
1. execution_state: `in_progress`
2. scope_completeness: `scope_partial`
3. target_completeness: `target_partial`
4. integration_completeness: `partial`
5. open_lanes:
   - no packet-specific scenario manifest yet
   - no empirical worksheet replay packet yet for `CALL` / `REGISTER.ID`
   - no host-backed invocation/runtime implementation yet
   - current repo floor is XLL self-registration plus `XLCALL.H` identity ingest, not worksheet closure

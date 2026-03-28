# FUNCTION SLICE - CALL and REGISTER.ID UDF Registration Seam Prelim

## 1. Purpose
Pin the admitted OxFunc-side runtime seam for worksheet/macro `REGISTER.ID` and macro-sheet `CALL` without collapsing raw Excel C API exposure, DLL/code-resource loading, or external invocation execution into OxFunc.

## 2. Current OxFunc Reading
OxFunc is steward of the function registration catalog and of the worksheet-facing normalization for `REGISTER.ID` / `CALL`.

That means:
1. OxFunc owns the catalog identity of built-in worksheet functions and their legacy `XLCALL.H` codes where those exist.
2. OxFunc should also own the catalog identity of host-registered external functions once the host/OxFml side supplies registration descriptors.
3. OxFunc owns:
   - `REGISTER.ID` request normalization,
   - `CALL` target normalization,
   - register-id lookup vs direct-call target split,
   - worksheet result projection.
4. Host/OxFml still owns:
   - raw Excel C API dispatch,
   - registration handle allocation,
   - DLL/code-resource lookup and loading,
   - actual external invocation.

## 3. Built-In Function Identity Layer
Current built-in ingest artifacts:
1. `docs/function-lane/XLCALL_CODE_CATALOG.csv`
2. `docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv`

Current intended reading:
1. `XLCALL.H` built-in `xlf*` numbers are legacy built-in aliases for OxFunc catalog rows, not replacements for OxFunc stable ids.
2. The preferred downstream identity remains `surface_stable_id`.
3. `xlf*` code and symbol travel as compatibility/interoperability metadata for host C API routing.

## 4. Admitted Current-Baseline Excel Reading
Pinned from `ExecuteExcel4Macro(...)` on the seeded Windows baseline:
1. `REGISTER.ID("Kernel32","GetTickCount","J!")` returns a numeric register id.
2. `CALL(register_id)` succeeds for the seeded zero-argument `GetTickCount` lane.
3. `CALL("Kernel32","GetTickCount","J!")` succeeds directly.
4. `CALL("Kernel32","MulDiv","JJJJ",6,7,3)` succeeds and returns `14`.
5. `CALL(register_id,6,7,3)` succeeds for the seeded `MulDiv` lane and returns `14`.
6. The seeded zero-argument `GetTickCount` lane also succeeds when `type_text` is omitted:
   - `REGISTER.ID("Kernel32","GetTickCount")`
   - `CALL("Kernel32","GetTickCount")`
7. This does not yet pin a general omission rule for argument-bearing direct-call lanes.

Important admission distinction:
1. Microsoft documents `CALL` as an Excel 4 macro-sheet function.
2. `REGISTER.ID` is documented as worksheet-usable.
3. The current replay artifact uses `ExecuteExcel4Macro(...)` to avoid conflating host sheet admission rules with the function-side registration seam.

## 5. Typed OxFunc Runtime Seam
Current first-pass runtime seam is:
1. `RegisterIdRequest`
2. `RegisteredExternalDescriptor`
3. `RegisteredExternalCallRequest`
4. `RegisteredExternalProvider`

Current request/target shape:
1. `REGISTER.ID`
   - `library_name`
   - `procedure`
   - optional `declared_type_text`
2. `CALL`
   - by register id, or
   - direct target `{ library_name, procedure, optional declared_type_text }`
   - plus trailing invocation args preserved as raw `CallArgValue`

Current descriptor shape:
1. `stable_registration_id`
2. `register_id`
3. `origin_kind`
4. `display_name`
5. `library_name`
6. `procedure`
7. `declared_type_text`

## 6. Provider Ownership Split
Current first-pass split:
1. host/OxFml resolves `REGISTER.ID` requests into a `RegisteredExternalDescriptor`,
2. host/OxFml looks up existing descriptors by numeric register id,
3. host/OxFml performs the actual external invocation,
4. OxFunc returns the projected worksheet value/error from the host-supplied outcome.

Why this is above OxFunc:
1. registration state is global host state,
2. external routine loading/execution is not function-kernel work,
3. registration/removal must later align with `W049` immutable snapshot generation.

## 7. Relation To W047 / W049
This packet pressures the first-freeze runtime seam in two places:
1. `W047`
   - `RegisteredExternalProvider` should be a distinct typed bundle member rather than hidden inside `HostInfoProvider`.
2. `W049`
   - future registered-external additions/removals should publish a fresh immutable `LibraryContextSnapshot` generation.

## 8. Current Honest Status
This is no longer only a note-only seam.

What is real now:
1. local SDK `XLCALL.H` ingest is reproducible,
2. built-in `xlf*` codes are cataloged against current OxFunc stable ids where possible,
3. native Excel macro replay exists for seeded `REGISTER.ID` / `CALL` lanes,
4. OxFunc core now has:
   - typed `RegisteredExternalProvider`,
   - typed request/descriptor/call-request structs,
   - `REGISTER.ID` runtime surface,
   - `CALL` runtime surface.

What remains open:
1. OxFml now has a real proving-host floor for:
   - worksheet `REGISTER.ID`,
   - worksheet `CALL`,
   - reference-visible `CALL` arguments,
   - host API registration,
   - VBA shim registration,
   - unregister packet carriage.
2. direct adoption of OxFunc-owned `RegisterIdRequest`, `RegisteredExternalDescriptor`, and `RegisteredExternalCallRequest` packet types is now exercised on the OxFml side.
3. the remaining shared-note decisions are now narrower:
   - exact shared field naming,
   - the minimum final `RegisteredExternalDescriptor` field set needed for OxFunc-owned dereference/coercion decisions,
   - whether `RegisteredExternalCatalogMutation*` / `RegisteredExternalCatalogController` stay OxFml-owned funnel packets or become shared OxFunc-owned runtime packet families,
   - minimum register/unregister consequences for `LibraryContextSnapshot` generation.
4. broader omitted-`type_text` and worksheet-vs-macro-sheet admission characterization remain useful evidence topics, but OxFml no longer treats them as the primary live shared-note freeze items.

## 9. Current Closure Suggestions
1. shared field naming:
   - freeze the current packet names and field names exactly as they already exist
   - no rename round is currently justified
2. minimum `RegisteredExternalDescriptor` field set:
   - keep the current shared descriptor shape:
     - `stable_registration_id`
     - `register_id`
     - `origin_kind`
     - `display_name`
     - `library_name`
     - `procedure`
     - `declared_type_text`
   - OxFunc currently needs no additional descriptor fields for dereference/coercion decisions
3. mutation/controller family ownership:
   - keep `RegisteredExternalCatalogMutation*` and `RegisteredExternalCatalogController` OxFml-owned funnel packets for the current phase
   - revisit only if OxFunc or another downstream consumer needs to consume them directly as shared runtime ABI
4. snapshot-generation consequences:
   - bind-visible registration or unregister should publish a new `LibraryContextSnapshot` generation
   - descriptor mutation used only through worksheet `CALL` / `REGISTER.ID` should default to targeted reevaluation rather than broad rebinding
5. non-blocking follow-on evidence:
   - omitted-`type_text` widening
   - worksheet-vs-macro-sheet admission/version widening
6. current closure read:
   - the OxFunc/OxFml seam note now converges on this packet split
   - remaining work is no longer first-pass packet design
   - remaining work is shared seam-freeze promotion and any later OxCalc-side acknowledgment

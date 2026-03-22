# FUNCTION SLICE - CALL and REGISTER.ID UDF Registration Seam Prelim

## 1. Purpose
Pin the current OxFunc-side registration/catalog role for worksheet `CALL` and `REGISTER.ID` without collapsing the Excel C API host surface, DLL/code-resource loading, or external invocation runtime into OxFunc.

## 2. Current OxFunc Reading
OxFunc should act as steward of the function registration catalog.

That means:
1. OxFunc owns the catalog identity of built-in worksheet functions and their legacy `XLCALL.H` built-in codes where those exist.
2. OxFunc should also own the catalog identity of host-registered external functions once the host/OxFml side supplies their registration descriptors.
3. The host side still owns exposure of the Excel C API surface and any actual external DLL/code-resource invocation.

## 3. Built-In Function Identity Layer
Current built-in ingest artifacts:
1. `docs/function-lane/XLCALL_CODE_CATALOG.csv`
2. `docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv`

Current intended reading:
1. `XLCALL.H` built-in `xlf*` numbers are treated as legacy built-in aliases for OxFunc catalog rows, not as replacements for OxFunc stable ids.
2. The preferred downstream identity remains `surface_stable_id`.
3. The `xlf*` code and symbol should travel as compatibility/interoperability metadata for host C API routing.

## 4. Minimal OxFml <-> Host <-> OxFunc Seam
Current first-pass seam split:
1. host owns the Excel C API callback surface and raw function-number dispatch (`xlf*`, `xlc*`, `xlSpecial`, `xlUDF`),
2. OxFunc owns the catalog mapping from built-in `xlf*` rows to OxFunc stable ids,
3. host/OxFml should call into OxFunc using stable ids and prepared arguments once built-in dispatch is resolved,
4. host owns any eventual external-registration handle allocation and raw external call execution.

## 5. UDF Registration Direction
Current best-attempt registration direction:
1. host/OxFml observes or receives a registration request analogous to `REGISTER.ID` / host-side `xlfRegister`,
2. host registers the external routine with the host runtime,
3. OxFunc receives or stores a catalog descriptor for the registered function:
   - stable catalog id or host registration handle,
   - surface name,
   - declared arity/signature information,
   - volatility / thread / reference-argument posture,
   - origin kind,
4. registration or removal should produce an explicit new immutable library-context snapshot generation rather than mutating downstream catalog truth invisibly,
5. later worksheet `CALL`-style invocation routes through that registered descriptor.

Current constraint:
1. OxFunc should not own DLL loading, code-resource lookup, or external call execution itself.

## 6. Current Catalog Fields Expected To Matter
For built-in rows, OxFml should expect:
1. `surface_stable_id`
2. `canonical_surface_name`
3. `registration_source_kind`
4. `xlcall_builtin_symbol`
5. `xlcall_builtin_code`

For future registered external rows, OxFunc will likely need:
1. a stable registration/catalog id,
2. registration source kind,
3. declared signature / arity metadata,
4. host invocation boundary kind.

## 7. What Is Intentionally Above OxFunc
Not OxFunc-owned:
1. raw `Excel12v` / callback dispatch,
2. `XLCALL.H` command execution,
3. DLL/code-resource discovery and loading,
4. VBA/Automation integration runtime,
5. host-side registration handle allocation,
6. actual external invocation.

## 8. Current Honest Status
This is a first-pass seam definition, not worksheet-semantic closure for `CALL` / `REGISTER.ID`.

What is real now:
1. local SDK `XLCALL.H` ingest is reproducible,
2. built-in `xlf*` codes are now cataloged against current OxFunc stable ids where possible,
3. the preferred registration/catalog ownership split is explicit for the next OxFml exchange.

What remains open:
1. empirical worksheet baseline for `CALL` / `REGISTER.ID`,
2. final registered-external descriptor shape,
3. full host registration and invocation path,
4. exact runtime `LibraryContextProvider` / `LibraryContextSnapshot` registration-update shape.

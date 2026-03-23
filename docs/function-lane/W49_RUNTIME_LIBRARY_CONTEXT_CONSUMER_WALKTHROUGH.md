# W49 Runtime Library Context Consumer Walkthrough

Status: `active`
Packet: `W049`

## 1. Purpose
Show one honest first-pass consumer flow over the current covered scope using:
1. runtime `LibraryContextProvider`
2. immutable `LibraryContextSnapshot`
3. `W047` typed context/query bundle
4. `W048` return-surface split

## 2. Consumer Example - Built-In Function
Formula example:
1. `=NOW()`

Consumer flow:
1. OxFml asks `LibraryContextProvider.current_snapshot()`.
2. Binder resolves `NOW` to the snapshot entry with:
   - `surface_stable_id = FUNC.NOW`
   - `entry_kind = built_in_function`
   - `runtime_boundary_kind = extended_value_with_presentation_hint`
3. Bound node preserves:
   - `surface_stable_id`
   - `arity`
   - `arg_preparation_profile`
   - seam-facing guidance fields
4. Evaluator uses `W047`:
   - `NowProvider.now_serial()`
5. OxFunc returns according to `W048`:
   - `ValueWithPresentation`
   - numeric serial value
   - `number_format` hint
6. OxFml/host applies or ignores the presentation hint according to publication policy.

## 3. Consumer Example - Host Query Function
Formula example:
1. `=FORMULATEXT(A1)`

Consumer flow:
1. Binder resolves `FORMULATEXT` from the current snapshot entry.
2. Entry guidance tells OxFml this is not a pure value-only kernel.
3. Evaluator uses `W047`:
   - `HostInfoProvider.query_formula_text(reference)`
4. OxFunc receives the typed result and returns an ordinary worksheet value.

## 4. Consumer Example - Provider Projection Function
Formula example:
1. `=RTD("Prog.Id", , "topic1", "topic2")`

Consumer flow:
1. Binder resolves `RTD` from the current snapshot entry.
2. Entry guidance preserves:
   - `special_interface_kind = host_subscription_provider`
   - `runtime_boundary_kind = host_provider_projection`
3. OxFml/host owns:
   - server activation
   - topic lifecycle
   - cell-to-topic mapping
   - current value availability
4. Evaluator passes prepared `RtdRequest` to `RtdProvider`.
5. OxFunc projects the typed `RtdProviderResult` into worksheet-visible value/error result.

## 5. Consumer Example - Registration Change
Event example:
1. host registers a new external worksheet entry through the future `W046` path

Consumer flow:
1. host/OxFml updates library-context truth above OxFunc
2. `LibraryContextProvider` emits a fresh immutable `LibraryContextSnapshot`
3. new snapshot gets a fresh `snapshot_generation`
4. existing consumers may:
   - continue using the prior snapshot deterministically
   - or explicitly switch to the newer generation
5. CSV export remains useful for debugging and fixture pinning, but does not define the runtime mutation model.

## 6. Why This Is Better Than CSV-Only Consumption
1. runtime lookup indexes are natural in object form
2. immutability is explicit
3. generation changes become explicit events
4. consumer code groups entry fields by meaning instead of parsing many flat CSV columns at runtime

## 7. Current Honest Limit
This walkthrough is a first-pass consumer model. It does not claim:
1. final ABI naming lock
2. final registered-external descriptor shape
3. a requirement that runtime objects be serialized exactly like the CSV

# Function Slice - Return Surface And Publication Hint Contract (Prelim)

Workset: `W048`

## 1. Purpose
Freeze the first shared return-surface split for the already-covered OxFunc scope.

The current shared split is:
1. ordinary value
2. `ValueWithPresentation`
3. typed host/provider outcome projection

## 2. Shared Reading
The first freeze candidate is:
1. preserve ordinary `EvalValue` as the default function result surface,
2. preserve `ExtendedValue::ValueWithPresentation { value, hint }` as the publication-aware wrapper when Excel changes formatting/style without changing the underlying scalar value,
3. preserve typed host/provider outcome enums on the callback boundary where the host/provider classification is semantically important, with OxFunc projecting them into worksheet-visible values/errors.

Current reading:
1. the third class is not a demand for a new general published-value carrier,
2. it is the current pattern where OxFunc consumes a typed upstream result family and projects it into the worksheet-visible result universe,
3. only concrete implementation evidence should force a narrower or broader factorization.

## 3. Return Classes
### 3.1 Ordinary Value
Representation:
1. `EvalValue`

Current covered examples:
1. `CELL`
2. `INFO`
3. `ISFORMULA`
4. `FORMULATEXT`
5. `SHEET`
6. `SHEETS`
7. `SUBTOTAL`
8. `AGGREGATE`
9. `ASC`
10. `DBCS`
11. `JIS`
12. `NUMBERVALUE`
13. plain `HYPERLINK` value path
14. plain `NOW` / `TODAY` value path
15. `RAND`

### 3.2 `ValueWithPresentation`
Representation:
1. `ExtendedValue::ValueWithPresentation { value, hint }`

Current hint fields:
1. `hint.number_format`
2. `hint.style`

Current covered examples:
1. `NOW`
   - numeric serial
   - plus number-format hint
2. `TODAY`
   - numeric serial
   - plus number-format hint
3. `HYPERLINK`
   - ordinary text value
   - plus `style=hyperlink`

Current ownership split:
1. OxFunc owns emission of the presentation-aware return shape,
2. OxFml / host owns application/publication of the hint,
3. the hint does not change the underlying scalar value semantics.

### 3.3 Typed Host / Provider Outcome Projection
Representation pattern:
1. a typed upstream outcome family remains explicit at the callback boundary,
2. OxFunc projects that typed outcome into worksheet-visible values/errors,
3. the projected worksheet result itself normally lands back in the ordinary value universe.

Current covered examples:
1. `TRANSLATE`
   - input typed outcome: `TranslateProviderResult`
   - projected outputs:
     - `Text(text) -> text`
     - `Busy -> #BUSY!`
     - `CapabilityDenied -> #BLOCKED!`
     - `ProviderError(code) -> code`
2. `RTD`
   - input typed outcome: `RtdProviderResult`
   - projected outputs:
     - `Value(v) -> v`
     - `NoValueYet -> #N/A`
     - `CapabilityDenied -> #BLOCKED!`
     - `ConnectionFailed -> #CONNECT!`
     - `ProviderError(code) -> code`

Current reading:
1. typed outcome projection is part of the shared seam even when the final worksheet-visible result is an ordinary value or worksheet error,
2. this keeps provider/runtime classification explicit above OxFunc while avoiding ad hoc stringly result channels.

## 4. What This Packet Does Not Freeze
This packet does not freeze:
1. full rich-value publication semantics,
2. final callable publication policy,
3. future provider families beyond the current covered seams,
4. any requirement that OxFunc itself apply presentation hints.

Clarification:
1. `IMAGE` remains in the current overall completion scope,
2. but as a sibling rich-value/publication packet rather than as a reason to widen `W048` prematurely.

## 5. Covered Function Mapping
### 5.1 Ordinary Value Class
1. `CELL`
2. `INFO`
3. `ISFORMULA`
4. `FORMULATEXT`
5. `SHEET`
6. `SHEETS`
7. `SUBTOTAL`
8. `AGGREGATE`
9. `ASC`
10. `DBCS`
11. `JIS`
12. `NUMBERVALUE`
13. `RAND`

### 5.2 Presentation-Aware Class
1. `NOW`
2. `TODAY`
3. `HYPERLINK`

### 5.3 Typed Outcome Projection Class
1. `TRANSLATE`
2. `RTD`

## 6. Evidence Posture
This packet freezes a shared return reading from already exercised surfaces in:
1. `docs/function-lane/VALUE_UNIVERSE_PRELIM_SPEC.md`
2. `docs/function-lane/FUNCTION_SLICE_NOW_CONTRACT_PRELIM.md`
3. `docs/function-lane/FUNCTION_SLICE_TODAY_CONTRACT_PRELIM.md`
4. `docs/function-lane/FUNCTION_SLICE_HYPERLINK_IMAGE_VALUE_MODEL_PRELIM.md`
5. `docs/function-lane/FUNCTION_SLICE_TRANSLATE_PROVIDER_LANGUAGE_CONTRACT_PRELIM.md`
6. `docs/function-lane/FUNCTION_SLICE_RTD_CONTRACT_PRELIM.md`
7. `docs/function-lane/W36_EXECUTION_RECORD.md`
8. `docs/function-lane/W43_EXECUTION_RECORD.md`

## 7. Artifact Bindings
1. workset: `docs/worksets/W048_RETURN_SURFACE_AND_PUBLICATION_HINT_FREEZE.md`
2. mapping table: `docs/function-lane/W48_RETURN_SURFACE_CLASS_MAP.csv`
3. execution record: `docs/function-lane/W48_EXECUTION_RECORD.md`
4. core value model:
   - `crates/oxfunc_core/src/value.rs`
   - `formal/lean/OxFunc/ValueUniverse.lean`

# Function Slice - Width Conversion Host/Profile Contract (Prelim)

Workset: `W034`

## 1. Scope
This contract covers the current-baseline OxFunc-side seam for:
1. `ASC`
2. `DBCS`
3. `JIS`

## 2. Current Baseline Reading
From the native host replay in `.tmp/w26-host-profile-provider-results.csv`:
1. `ASC("ＡＢＣ　１２３")` returned the input unchanged on the current host/profile.
2. `DBCS("ABC ｶﾞ")` returned the input unchanged on the current host/profile.
3. `JIS("ABC ｶﾞ")` returned `#NAME?` on the current host/profile.

So the honest current seam is not "pure local text conversion". It is:
1. host/profile decides whether width conversion is active,
2. OxFunc owns the admitted UTF-16 transformation once the profile mode is known,
3. non-admission on the current host baseline remains above OxFunc, with `Unavailable` retained as a defensive runtime projection.

## 3. Typed OxFunc-Side Contract
Pinned host query:
1. `query_width_conversion_mode(function) -> WidthConversionMode`

Pinned request enum:
1. `WidthConversionFunction::Asc`
2. `WidthConversionFunction::Dbcs`
3. `WidthConversionFunction::Jis`

Pinned result enum:
1. `PassThrough`
2. `NarrowBasicWidthAndKana`
3. `WidenBasicWidthAndKana`
4. `Unavailable`

## 4. OxFunc Ownership
Once the host/profile query result is supplied, OxFunc owns:
1. text coercion,
2. UTF-16-unit width conversion kernels,
3. value projection,
4. defensive `Unavailable -> #NAME?` runtime projection if the function was still admitted above OxFunc.

Current runtime hooks:
1. `crates/oxfunc_core/src/functions/text_compat_locale_family.rs`
2. `crates/oxfunc_core/src/host_info.rs`

## 5. OxFml Ownership
OxFml / host owns:
1. whether the function is admitted in the current library-context snapshot,
2. which width-conversion mode is active for the current host/profile,
3. any future cross-profile widening of the mode matrix.

## 6. Admitted Current Slice
This contract claims the current admitted slice only:
1. current-host passthrough for `ASC` and `DBCS`,
2. current-host non-admission for `JIS`,
3. OxFunc-local support for the actual narrow/widen transforms when a future host/profile supplies those modes.


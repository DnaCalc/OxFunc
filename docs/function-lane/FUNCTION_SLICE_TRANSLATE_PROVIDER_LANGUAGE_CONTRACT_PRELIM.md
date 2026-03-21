# Function Slice - TRANSLATE Provider-Language Contract (Prelim)

Workset: `W036`

## 1. Scope
This contract covers the current-baseline OxFunc-side seam for `TRANSLATE`.

## 2. Current Baseline Reading
From `.tmp/w26-host-profile-provider-results.csv`:
1. `TRANSLATE("hola","es","es")` returned `"hola"`.
2. `TRANSLATE("hello","en","es")` returned `#BUSY!`.

So the honest current split is:
1. same-language passthrough is local and deterministic,
2. actual translation is provider-bound and must come from above OxFunc.

## 3. Typed OxFunc-Side Contract
Pinned request:
1. `TranslateRequest { text, source_language, target_language }`

Pinned provider query:
1. `query_translate(request) -> TranslateProviderResult`

Pinned provider result enum:
1. `Text(text)`
2. `Busy`
3. `CapabilityDenied`
4. `ProviderError(code)`

Current runtime hook:
1. `crates/oxfunc_core/src/functions/number_regex_translate_family.rs`

## 4. OxFunc Ownership
OxFunc owns:
1. argument preparation and text coercion,
2. same-language passthrough when source and target normalize equal,
3. typed projection of provider results to worksheet values/errors.

Current projections:
1. `Text(text) -> text`
2. `Busy -> #BUSY!`
3. `CapabilityDenied -> #BLOCKED!`
4. `ProviderError(code) -> code`

## 5. OxFml / Host Ownership
OxFml / host owns:
1. provider invocation,
2. provider capability truth,
3. provider failure classification beyond the current seeded lanes,
4. any future shared provider family with `DETECTLANGUAGE`.

## 6. Admitted Current Slice
This contract claims:
1. same-language passthrough,
2. cross-language provider-busy projection,
3. typed provider result handling on the OxFunc side,
4. no claim of local translation semantics.


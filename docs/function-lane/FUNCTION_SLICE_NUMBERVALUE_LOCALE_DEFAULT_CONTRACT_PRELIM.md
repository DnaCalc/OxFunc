# Function Slice - NUMBERVALUE Locale-Default Contract (Prelim)

Workset: `W035`

## 1. Scope
This contract covers the omitted-default separator behavior of `NUMBERVALUE`.

## 2. Current Baseline Reading
From `.tmp/w26-host-profile-provider-results.csv`:
1. `NUMBERVALUE("1,234.5%")` returned `#VALUE!` when separator defaults were omitted.
2. `NUMBERVALUE("1,234.5%",".",",")` returned `12.345`.

So the honest current split is:
1. explicit separator lanes are pure OxFunc parsing,
2. omitted-default lanes depend on locale/profile defaults supplied above OxFunc.

## 3. Typed OxFunc-Side Contract
Pinned locale-default source:
1. `LocaleFormatContext.profile.decimal_separator`
2. `LocaleFormatContext.profile.thousands_separator`

Pinned rule:
1. if separator args are explicit, OxFunc uses the explicit chars and does not require locale context,
2. if either separator arg is omitted, OxFunc requires `LocaleFormatContext`,
3. omitted-default failure to supply locale context is a seam failure and currently projects `#VALUE!`.

Current runtime hook:
1. `crates/oxfunc_core/src/functions/number_regex_translate_family.rs`

## 4. OxFunc Ownership
OxFunc owns:
1. text coercion,
2. separator validation,
3. percent handling,
4. normalized numeric parse,
5. projection of locale-default absence to the current runtime error path.

## 5. OxFml Ownership
OxFml / FEC owns:
1. supplying the active locale/profile context for omitted-default parsing,
2. keeping explicit-separator lanes separate from omitted-default lanes at the interface boundary.

## 6. Admitted Current Slice
This contract claims:
1. current-host omitted-default rejection for the seeded English-style sample,
2. explicit-separator parity,
3. profile-driven omitted-default parsing through the locale context,
4. no broader locale sweep beyond the current baseline.


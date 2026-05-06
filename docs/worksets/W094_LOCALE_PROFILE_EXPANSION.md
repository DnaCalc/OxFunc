# W094 Locale Profile Expansion

Status: `in_progress`

## 1. Purpose

Process `HANDOFF-OXFUNC-006` from OxFml W070 by expanding OxFunc's canonical locale profile identity and `FormatProfile` constants without moving OxFml formatting grammar behavior into OxFunc.

## 2. Problem Statement

OxFml is blocked on `BLK-FML-005` because it can only consume `LocaleProfileId::EnUs` and `LocaleProfileId::CurrentExcelHost`. Adding locale-keyed month names, weekday names, parser branches, General rendering expectations, and optional locale-prefix format-code parsing directly in OxFml would force OxFml to own a duplicate profile registry. OxFunc must provide the canonical locale profile ids and constants that sibling repos consume.

## 3. Scope

In scope:

1. explicit `LocaleProfileId` variants for the concrete locale set requested by the inbound OxFml note,
2. `FormatProfile` constants through the existing `format_profile(...)` constructor,
3. stable profile names and enumerability for downstream publication/tests,
4. documentation that locale profile id and workbook date system are orthogonal axes,
5. intake registration and bead tracking for `HANDOFF-OXFUNC-006`.

Out of scope:

1. OxFml month and weekday render tables,
2. OxFml parser branches,
3. OxFml General rendering,
4. Excel locale-prefix custom-format grammar,
5. DNA OneCalc workspace-locale UI cleanup,
6. claiming full locale semantic parity from separators/currency constants alone.

## 4. OxFunc-Local Surface

The W094 OxFunc-local slice adds profile ids and constants for:

1. `en-US`
2. `en-GB`
3. `en-IE`
4. `en-AU`
5. `en-NZ`
6. `en-ZA`
7. `en-IN`
8. `en-CA`
9. `en-PH`
10. `de-DE`
11. `ru-RU`
12. `fi-FI`
13. `et-EE`
14. `lv-LV`
15. `lt-LT`
16. `sk-SK`
17. `cs-CZ`
18. `nb-NO`
19. `nn-NO`
20. `fr-FR`
21. `es-ES`
22. `pt-PT`
23. `it-IT`
24. `nl-NL`
25. `pl-PL`
26. `pt-BR`
27. `ja-JP`
28. `ko-KR`
29. `zh-CN`
30. `hu-HU`
31. `current-excel-host`

The surface also adds:

1. stable BCP-47-style profile names,
2. canonical profile-id enumeration,
3. `LocaleProfileId::from_bcp47_language_tag(...)` for the DNA OneCalc ambient language-tag table.

## 5. Compatibility Direction

1. Workbook date system remains independent of locale profile id.
2. `LocaleFormatContext` remains the seam object that combines profile, date system, parser, and formatter.
3. Explicit locale ids are deterministic profile identities; `CurrentExcelHost` is a live-host placeholder.
4. The current `FormatProfile` shape carries separators, currency symbol, and currency decimals, but not month names, weekday names, currency placement, or currency positive/negative patterns.

## 6. Tracking

1. Inbound handoff: `docs/handoffs/HANDOFF-OXFUNC-006_W070_LOCALE_PROFILE_EXPANSION_REQUEST.md`
2. Upstream source: `../OxFml/docs/handoffs/HANDOFF-OXFUNC-006_W070_LOCALE_PROFILE_EXPANSION_REQUEST.md`
3. Related upstream source: `../DnaOneCalc/docs/HANDOFF_OXFML_LOCALE_EXPANSION.md`
4. Bead: `oxf-84x3`

## 7. Validation Evidence

1. `cargo fmt --manifest-path crates\oxfunc_core\Cargo.toml`: passed.
2. `cargo test --manifest-path crates\oxfunc_core\Cargo.toml --lib profile`: passed, `8` passed, `0` failed, `1264` filtered out.
3. `locale_format::tests::profile_ids_cover_dna_onecalc_ambient_language_tags` pins the BCP-47 mapping surface.
4. `locale_format::tests::expanded_profile_constants_carry_locale_separators_and_currency_defaults` pins representative separator and currency defaults.

## 8. Reporting Contract

All W094 reports must include:

1. `execution_state`,
2. `scope_completeness`,
3. `target_completeness`,
4. `integration_completeness`,
5. explicit `open_lanes` while any axis remains partial.

Initial status axes:

1. `scope_completeness`: `scope_partial`
2. `target_completeness`: `target_complete` for the OxFunc-local W094 profile identity/constants slice
3. `integration_completeness`: `partial`
4. `open_lanes`: downstream OxFml locale table/parser/general consumption, optional locale-prefix grammar, full Excel currency-pattern semantics beyond the current `FormatProfile` shape, and landed-ref promotion.

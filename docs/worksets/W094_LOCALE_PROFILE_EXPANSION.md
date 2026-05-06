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
5. short-date order and short-date pattern facts,
6. currency placement, spacing, and negative-pattern facts,
7. invariant Excel custom-format token-policy facts,
8. Excel LCID to canonical profile-id mapping,
9. intake registration and bead tracking for `HANDOFF-OXFUNC-006`.

Out of scope:

1. OxFml month and weekday render tables,
2. OxFml parser branches,
3. OxFml General rendering,
4. custom-format parsing/rendering behavior,
5. DNA OneCalc workspace-locale UI cleanup,
6. Excel file-storage behavior for localized function names and locale prefixes,
7. claiming full locale semantic parity from culture-profile seed constants alone.

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
3. `LocaleProfileId::from_bcp47_language_tag(...)` for the DNA OneCalc ambient language-tag table,
4. `LocaleProfileId::from_excel_lcid(...)` for supported Excel locale-prefix LCID aliases,
5. short-date order and pattern fields,
6. currency layout and negative-pattern fields,
7. invariant Excel format-code token-policy fields.

## 5. Compatibility Direction

1. Workbook date system remains independent of locale profile id.
2. `LocaleFormatContext` remains the seam object that combines profile, date system, parser, and formatter.
3. Explicit locale ids are deterministic profile identities; `CurrentExcelHost` is a live-host placeholder.
4. `FormatProfile` carries separators, currency symbol/decimals, short-date order/pattern, currency layout, negative-pattern class, and invariant format-code token policy.
5. `FormatProfile` does not carry month names or weekday names; OxFml owns those rendering tables.
6. `FormatProfile` distinguishes invariant stored format-code tokens from localized display separators; OxFml owns parser/render behavior over those facts.

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
5. `locale_format::tests::excel_lcid_mapping_covers_supported_locale_profile_aliases` pins the LCID mapping surface.
6. `locale_format::tests::expanded_profile_constants_carry_locale_separators_and_currency_defaults` now also pins representative date-order, currency-layout, negative-pattern, and invariant token-policy facts.
7. No new validation command was run for the 2026-05-06 final-state locale detail processing pass.

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
4. `open_lanes`: downstream OxFml final-profile-field consumption, Excel file-storage and locale-prefix research, full locale semantic parity sweeps beyond the current profile seed rows, and landed-ref promotion.

## 2026-05-06 Final-State Locale Detail Processing

OxFunc processed the OxFml request to make `oxfunc_core::locale_format` the
canonical owner of the remaining locale facts needed by OxFml without creating
an intermediate workaround registry.

Added code surface:
1. `DateComponentOrder`
2. `CurrencyPlacement`
3. `CurrencySpacing`
4. `CurrencyNegativePattern`
5. `FormatCodeTokenPolicy`
6. `FormatProfile.short_date_order`
7. `FormatProfile.short_date_pattern`
8. `FormatProfile.two_digit_year_pivot`
9. `FormatProfile.currency_placement`
10. `FormatProfile.currency_spacing`
11. `FormatProfile.currency_negative_pattern`
12. `FormatProfile.format_code_decimal_token`
13. `FormatProfile.format_code_group_token`
14. `FormatProfile.format_code_token_policy`
15. `LocaleProfileId::from_excel_lcid(...)`

Execution-state update:
1. `execution_state`: `final_profile_detail_surface_seeded`
2. `scope_completeness`: `scope_partial`
3. `target_completeness`: `target_partial`
4. `integration_completeness`: `partial`
5. `open_lanes`: downstream OxFml consumption, Excel file-storage and
   locale-prefix research, full locale semantic parity sweeps beyond seed
   rows, and landed-ref promotion.

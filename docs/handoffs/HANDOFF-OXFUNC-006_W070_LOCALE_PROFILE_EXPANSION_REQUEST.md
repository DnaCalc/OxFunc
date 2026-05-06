# HANDOFF-OXFUNC-006 - W070 Locale Profile Expansion Request

Status: `acknowledged`
Direction: `OxFml -> OxFunc`
Source repo/workset: `OxFml/W070`
Target repo/workset: `OxFunc/W094`
Filed date: `2026-05-04`
Acknowledged date: `2026-05-04`
Upstream source: `../OxFml/docs/handoffs/HANDOFF-OXFUNC-006_W070_LOCALE_PROFILE_EXPANSION_REQUEST.md`
Related inbound: `../DnaOneCalc/docs/HANDOFF_OXFML_LOCALE_EXPANSION.md`
Tracking bead: `oxf-84x3`

## OxFunc acknowledgement

OxFunc acknowledges the ownership direction in the OxFml handoff:

1. OxFunc owns canonical locale profile identity and `FormatProfile` constants.
2. OxFml should not create a second comprehensive locale registry.
3. OxFml remains owner of locale-keyed month and weekday render tables, parser branches, General rendering behavior, and optional locale-prefix format-code grammar.
4. `CurrentExcelHost` is a host-regional-settings placeholder and should not be treated as a reproducible locale profile id.

## OxFunc-local profile surface

W094 expands `oxfunc_core::locale_format` with explicit profile ids and `format_profile(...)` rows for the full DNA OneCalc ambient language-tag table plus the current-host placeholder:

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

The public seam now also exposes stable profile names and profile id lists so downstream consumers can enumerate canonical profile identities without maintaining a duplicate list.

W094 also adds `LocaleProfileId::from_bcp47_language_tag(...)` so downstream consumers can map the same BCP-47 language tags used by DNA OneCalc's ambient context into canonical OxFunc profile ids without owning a second mapping table.

## Compatibility expectations

1. Locale profile id and workbook date system are orthogonal axes.
2. `LocaleFormatContext` remains the binding surface that combines a `FormatProfile`, a `WorkbookDateSystem`, a parser, and a formatter.
3. `CurrentExcelHost` remains useful for live host regional settings, but deterministic publication and replay surfaces should prefer explicit profile ids.
4. Currency placement, currency spacing, negative currency pattern class, short-date order/pattern, invariant format-code token policy, and Excel LCID-to-profile mapping are now represented by the OxFunc `FormatProfile` / `LocaleProfileId` surface.

## Validation evidence

1. `cargo fmt --manifest-path crates\oxfunc_core\Cargo.toml`: passed.
2. `cargo test --manifest-path crates\oxfunc_core\Cargo.toml --lib profile`: passed, `8` passed, `0` failed, `1264` filtered out.
3. The focused test set includes `locale_format::tests::profile_ids_cover_dna_onecalc_ambient_language_tags`.
4. The focused test set includes `locale_format::tests::expanded_profile_constants_carry_locale_separators_and_currency_defaults`.

## Open lanes

1. OxFml still needs to consume the final OxFunc `FormatProfile` semantic fields.
2. OxFml still owns month/weekday render tables, parser branches, General rendering, and custom-format rendering behavior.
3. Excel file-storage behavior for localized function names and locale prefixes remains a later cross-repo research lane.
4. Locale constants are culture-profile seed rows, not full Excel locale semantic closure across every application channel/workbook compatibility version.
5. Landed-ref promotion remains open.

## Status report

execution_state: `validated_local_downstream_consumption_pending`

scope_completeness: `scope_partial`

target_completeness: `target_complete` for the OxFunc-local W094 profile identity/constants slice

integration_completeness: `partial`

open_lanes:
1. downstream OxFml consumption of the final profile fields,
2. Excel file-storage and locale-prefix research,
3. full locale semantic parity sweeps beyond the current profile seed rows,
4. landed-ref promotion.

## 2026-05-06 final-state locale detail handoff processing

OxFunc processed the OxFml final-state request for the W094 `FormatProfile`
surface.

Added OxFunc-owned facts:
1. `DateComponentOrder`
2. `FormatProfile.short_date_order`
3. `FormatProfile.short_date_pattern`
4. `FormatProfile.two_digit_year_pivot`
5. `CurrencyPlacement`
6. `CurrencySpacing`
7. `CurrencyNegativePattern`
8. `FormatProfile.currency_placement`
9. `FormatProfile.currency_spacing`
10. `FormatProfile.currency_negative_pattern`
11. `FormatCodeTokenPolicy`
12. `FormatProfile.format_code_decimal_token`
13. `FormatProfile.format_code_group_token`
14. `FormatProfile.format_code_token_policy`
15. `LocaleProfileId::from_excel_lcid(...)`

Surface interpretation:
1. `format_profile(id)` remains the single canonical constructor.
2. Format-code token fields are invariant Excel tokens for the current surface:
   decimal token `.` and group token `,`.
3. Display separators remain the localized `decimal_separator` and
   `thousands_separator` fields.
4. LCID mapping covers the current supported profile set plus same-language
   aliases that fold into the existing canonical profile ids.
5. OxFunc still does not implement OxFml custom-format parsing/rendering.

Deterministic tests added in `crates/oxfunc_core/src/locale_format.rs`:
1. profile matrix assertions for date order/pattern, currency placement,
   spacing, negative pattern, and invariant format-code token policy,
2. Excel LCID mapping coverage for the supported profile set and aliases,
3. representative non-US date order and currency placement cases.

Validation note:
1. no new validation command was run for this processing pass.

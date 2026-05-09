# W094 Locale Profile Expansion

Status: `local_target_satisfied_followups_split`

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
7. No validation command was run during the initial 2026-05-06 final-state locale detail processing pass; follow-up W094 validation evidence is listed below.
8. `powershell -NoProfile -ExecutionPolicy Bypass -File .tmp\run-w094-locale-excel-sweep.ps1`: passed for the focused live-Excel W094 locale comparison sweep; result summary: current-host international fields `11/11` matched, format-code token-policy observations `5/5` matched, LCID-prefix storage samples `9/9` matched after Excel's leading-zero normalization, and LCID-prefix render non-empty checks `9/9` matched.
9. `cargo test --manifest-path crates\oxfunc_core\Cargo.toml --lib locale_format -- --nocapture`: passed, `6` passed, `0` failed, `1270` filtered out.

## 8. Reporting Contract

All W094 reports must include:

1. `execution_state`,
2. `scope_completeness`,
3. `target_completeness`,
4. `integration_completeness`,
5. explicit `open_lanes` while any axis remains partial.

Current cleanup status axes:

1. `scope_completeness`: `scope_complete` for the OxFunc-local W094 profile identity and `FormatProfile` field slice requested by OxFml
2. `target_completeness`: `target_partial`
3. `integration_completeness`: `partial`
4. `open_lanes`: `oxf-swy6` downstream OxFml final-profile-field consumption tracking, `oxf-mxwo` Excel locale-prefix/localized-function storage research, `oxf-2nc0` culture-profile seed mismatch triage, and full locale semantic parity sweeps beyond the current profile seed rows.

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

## 2026-05-06 Focused Live-Excel Locale Comparison Sweep

Sweep command:
1. `powershell -NoProfile -ExecutionPolicy Bypass -File .tmp\run-w094-locale-excel-sweep.ps1`

Result artifacts:
1. `.tmp/w094-locale-excel-sweep-results.csv`
2. `.tmp/w094-locale-excel-sweep-summary.csv`

Observed Excel baseline:
1. Excel version: `16.0`
2. Windows current culture: `en-ZA`

Result summary:
1. current-host international fields matched OxFunc `CurrentExcelHost` seed:
   `11/11`,
2. invariant format-code token-policy observations matched: `5/5`,
3. LCID-prefix storage samples matched after accounting for Excel's leading-zero
   normalization of prefixes such as `[$-0409]` to `[$-409]`: `9/9`,
4. LCID-prefix render non-empty checks matched: `9/9`.

Interpretation:
1. the live Excel current-host profile matches the OxFunc
   `CurrentExcelHost` seed for separators, currency symbol/decimals, date
   order, currency placement, currency spacing, and negative-pattern class,
2. Excel `NumberFormat` stores invariant format-code decimal/group tokens,
3. Excel accepts the sampled LCID prefixes and renders localized month names
   for those prefixes,
4. this remains a focused current-host/sampled-LCID comparison, not a full
   locale semantic parity sweep for every supported profile.

Focused Rust validation command:
1. `cargo test --manifest-path crates\oxfunc_core\Cargo.toml --lib locale_format -- --nocapture`

Focused Rust validation result:
1. `6` passed,
2. `0` failed,
3. `1270` filtered out.

## 2026-05-06 All-Locale Sweep Extension

Sweep command:
1. `powershell -NoProfile -ExecutionPolicy Bypass -File .tmp\run-w094-all-locale-sweep.ps1`

Result artifacts:
1. `.tmp/w094-all-locale-excel-sweep-results.csv`
2. `.tmp/w094-all-locale-excel-sweep-summary.csv`

Coverage:
1. `30` canonical W094 locale profiles,
2. `11` same-language LCID aliases,
3. culture-profile matrix comparison for separators, short-date pattern/order,
   currency symbol/decimals/layout/spacing/negative-pattern class,
4. live Excel LCID-prefix storage and rendering for `mmmm` and `dddd`,
5. invariant format-code token-policy checks for every canonical profile.

Summary:
1. culture-profile matrix: `347` matched, `13` mismatched,
2. invariant format-code token-policy checks: `90` matched, `0` mismatched,
3. live Excel LCID-prefix storage checks: `82` matched, `0` mismatched,
4. live Excel LCID-prefix render non-empty checks: `82` matched, `0`
   mismatched,
5. live Excel LCID-prefix render text vs local culture reference: `82`
   matched, `0` mismatched.

Culture-profile matrix mismatches to triage:
1. `en-US`: currency negative-pattern class differs between current OxFunc seed
   and Windows PowerShell/.NET Framework culture data.
2. `en-AU`: short-date pattern differs: culture `d/MM/yyyy`, OxFunc seed
   `d/M/yyyy`.
3. `en-IN`: date separator, short-date pattern, currency spacing, and
   negative-pattern class differ.
4. `sk-SK`: culture date separator includes a trailing space.
5. `fr-FR`: currency symbol spacing is a regular symbol-space in the formatted
   culture sample; OxFunc currently classifies it as narrow no-break space.
6. `es-ES`: short-date pattern differs: culture `dd/MM/yyyy`, OxFunc seed
   `d/M/yyyy`.
7. `nl-NL`: short-date pattern differs: culture `d-M-yyyy`, OxFunc seed
   `dd-MM-yyyy`.
8. `ko-KR`: date separator and short-date pattern differ.
9. `hu-HU`: culture date separator includes a trailing space.

Interpretation:
1. the live Excel LCID-prefix surface accepted and rendered every sampled
   canonical and alias LCID in this sweep,
2. invariant format-code token-policy remains supported by the Excel
   `NumberFormat` observations,
3. the culture-profile mismatches are not live Excel LCID-prefix failures, but
   they are concrete W094 seed rows requiring triage before any stronger
   locale semantic-parity claim,
4. all-locale default date/currency behavior still needs a richer Excel-side
   capture method because COM `Application.International(...)` reports only
   the active host profile, not arbitrary LCID defaults.

## 2026-05-07 W094 Cleanup

Cleanup result:
1. The OxFunc-local W094 surface requested by `HANDOFF-OXFUNC-006` is satisfied
   for the declared profile identity and `FormatProfile` fact slice.
2. The remaining lanes have been split out of the active W094 bead rather than
   keeping the delivered OxFunc-local request indefinitely `in_progress`.
3. W094 is not a full locale semantic-parity claim.

Successor follow-up beads:
1. `oxf-swy6`: track downstream OxFml final-profile-field consumption.
2. `oxf-mxwo`: research Excel locale-prefix storage and localized function-name
   file behavior.
3. `oxf-2nc0`: triage the 13 culture-profile seed mismatches from the
   all-locale sweep.

Final W094 local status axes:
1. `scope_completeness`: `scope_complete`
2. `target_completeness`: `target_partial`
3. `integration_completeness`: `partial`
4. `open_lanes`: successor follow-up beads `oxf-swy6`, `oxf-mxwo`,
   `oxf-2nc0`, plus broader future locale semantic-parity sweeps.

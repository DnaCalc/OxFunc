//! Explicit, checked binding to OxFml's production locale provider. Never an oracle.
use oxfunc_core::locale_format::{
    format_profile, CurrencyNegativePattern, CurrencyPlacement, CurrencySpacing,
    DateComponentOrder, FormatProfile, LocaleFormatContext, LocaleProfileId, WorkbookDateSystem,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

pub struct LocaleBinding {
    pub context: LocaleFormatContext<'static>,
    pub provenance: Value,
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn expect(record: &Value, key: &str, expected: Value) -> Result<(), String> {
    let actual = record
        .get(key)
        .ok_or_else(|| format!("locale snapshot missing {key}"))?;
    if actual != &expected
        && !(actual.is_number() && expected.is_number() && actual.as_f64() == expected.as_f64())
    {
        return Err(format!(
            "locale profile mismatch for {key}: recorded={actual}, profile={expected}"
        ));
    }
    Ok(())
}

fn text_field(record: &Value, key: &str) -> Result<&'static str, String> {
    let value = record
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("locale snapshot missing string {key}"))?;
    // FormatProfile's public string fields are static; this command loads one
    // immutable profile for its entire process lifetime.
    Ok(Box::leak(value.to_owned().into_boxed_str()))
}

fn profile_json(profile: &FormatProfile) -> Value {
    json!({"decimal_separator":profile.decimal_separator,"thousands_separator":profile.thousands_separator,
        "list_separator":profile.list_separator,"currency_symbol":profile.currency_symbol,
        "date_separator":profile.date_separator,"time_separator":profile.time_separator,
        "short_date_pattern":profile.short_date_pattern,"two_digit_year_pivot":profile.two_digit_year_pivot,
        "currency_decimals":profile.currency_decimals,"currency_placement":format!("{:?}",profile.currency_placement),
        "currency_spacing":format!("{:?}",profile.currency_spacing),"currency_negative_pattern":format!("{:?}",profile.currency_negative_pattern)})
}

pub fn bind(
    profile_id: &str,
    record_path: &Path,
    recorded_settings: bool,
) -> Result<LocaleBinding, String> {
    let id = LocaleProfileId::from_bcp47_language_tag(profile_id)
        .filter(|id| id.stable_name().eq_ignore_ascii_case(profile_id))
        .ok_or_else(|| {
            format!("an explicit canonical locale profile is required, got {profile_id}")
        })?;
    let mut profile = format_profile(id);
    let canonical_profile = profile_json(&profile);
    let bytes = std::fs::read(record_path).map_err(|e| e.to_string())?;
    let record: Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    expect(
        &record,
        "schema_version",
        json!("w111.live_locale_profile.v1"),
    )?;
    let international = record
        .get("international")
        .ok_or("locale snapshot missing international")?;
    if recorded_settings {
        let windows = record
            .get("windows_culture")
            .ok_or("locale snapshot missing windows_culture")?;
        expect(windows, "name", json!(id.stable_name()))?;
        profile.decimal_separator = text_field(international, "decimal_separator")?;
        profile.thousands_separator = text_field(international, "thousands_separator")?;
        profile.list_separator = text_field(international, "list_separator")?;
        profile.currency_symbol = text_field(international, "currency_code")?;
        profile.date_separator = text_field(international, "date_separator")?;
        profile.time_separator = text_field(international, "time_separator")?;
        profile.short_date_pattern = text_field(windows, "short_date_pattern")?;
        profile.two_digit_year_pivot = windows
            .get("two_digit_year_max")
            .and_then(Value::as_u64)
            .and_then(|n| u16::try_from(n).ok());
        profile.currency_decimals = international
            .get("currency_digits")
            .and_then(Value::as_f64)
            .filter(|n| n.fract() == 0.0 && (0.0..=99.0).contains(n))
            .ok_or("invalid currency_digits")? as i32;
        profile.short_date_order = match international.get("date_order").and_then(Value::as_f64) {
            Some(0.0) => DateComponentOrder::Mdy,
            Some(1.0) => DateComponentOrder::Dmy,
            Some(2.0) => DateComponentOrder::Ymd,
            _ => return Err("invalid recorded date_order".into()),
        };
        profile.currency_placement = match international
            .get("currency_before")
            .and_then(Value::as_bool)
        {
            Some(true) => CurrencyPlacement::Before,
            Some(false) => CurrencyPlacement::After,
            _ => return Err("missing currency_before".into()),
        };
        profile.currency_spacing = match international
            .get("currency_space_before")
            .and_then(Value::as_bool)
        {
            Some(true) => CurrencySpacing::Space,
            Some(false) => CurrencySpacing::None,
            _ => return Err("missing currency_space_before".into()),
        };
        profile.currency_negative_pattern = match international
            .get("currency_negative")
            .and_then(Value::as_f64)
        {
            Some(0.0) => CurrencyNegativePattern::Parentheses,
            Some(1.0) if profile.currency_placement == CurrencyPlacement::Before => {
                CurrencyNegativePattern::MinusBeforeSymbol
            }
            Some(1.0) | Some(2.0) => CurrencyNegativePattern::LeadingMinus,
            Some(3.0) => CurrencyNegativePattern::TrailingMinus,
            _ => return Err("invalid recorded currency_negative".into()),
        };
    }
    for (key, value) in [
        ("decimal_separator", profile.decimal_separator),
        ("thousands_separator", profile.thousands_separator),
        ("list_separator", profile.list_separator),
        ("currency_code", profile.currency_symbol),
        ("date_separator", profile.date_separator),
        ("time_separator", profile.time_separator),
    ] {
        expect(international, key, json!(value))?;
    }
    if !recorded_settings {
        expect(
            &record,
            "application_decimal_separator",
            json!(profile.decimal_separator),
        )?;
        expect(
            &record,
            "application_thousands_separator",
            json!(profile.thousands_separator),
        )?;
    }
    expect(
        international,
        "currency_digits",
        json!(profile.currency_decimals),
    )?;
    expect(
        international,
        "date_order",
        json!(match profile.short_date_order {
            DateComponentOrder::Mdy => 0,
            DateComponentOrder::Dmy => 1,
            DateComponentOrder::Ymd => 2,
        }),
    )?;
    expect(
        international,
        "currency_before",
        json!(profile.currency_placement == CurrencyPlacement::Before),
    )?;
    expect(
        international,
        "currency_space_before",
        json!(profile.currency_spacing != CurrencySpacing::None),
    )?;
    let negative_code = match profile.currency_negative_pattern {
        CurrencyNegativePattern::Parentheses => 0,
        CurrencyNegativePattern::TrailingMinus => 3,
        CurrencyNegativePattern::MinusBeforeSymbol => 1,
        CurrencyNegativePattern::LeadingMinus => {
            if profile.currency_placement == CurrencyPlacement::Before {
                2
            } else {
                1
            }
        }
    };
    expect(international, "currency_negative", json!(negative_code))?;
    let date_system = match record.get("date_system").and_then(Value::as_str) {
        Some("1900") => WorkbookDateSystem::System1900,
        Some("1904") => WorkbookDateSystem::System1904,
        other => return Err(format!("unsupported recorded date system {other:?}")),
    };
    // Evidence is tied to the provider sources actually used by this binary.
    // No test-only parser and no Excel-backed XLL formatter can satisfy this.
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let source_hashes = record
        .get("provider_source_sha256")
        .and_then(Value::as_object)
        .ok_or("locale snapshot missing provider_source_sha256")?;
    for source in [
        "crates/oxfunc_core/src/locale_format.rs",
        "../OxFml/crates/oxfml_core/src/format/engine.rs",
        "../OxFml/crates/oxfml_core/src/format/number.rs",
        "../OxFml/crates/oxfml_core/src/format/datetime.rs",
        "../OxFml/crates/oxfml_core/src/format/general.rs",
    ] {
        let expected = source_hashes
            .get(source)
            .and_then(Value::as_str)
            .ok_or_else(|| format!("locale snapshot missing source hash {source}"))?;
        let actual = sha256(&std::fs::read(root.join(source)).map_err(|e| e.to_string())?);
        if actual != expected {
            return Err(format!(
                "locale provider source changed since profile capture: {source}"
            ));
        }
    }
    let mut sentinel_validation = Vec::new();
    if recorded_settings {
        let sentinels = record
            .get("sentinels")
            .and_then(Value::as_array)
            .ok_or("missing locale sentinels")?;
        let value = record
            .get("sentinel_input_value2")
            .and_then(Value::as_f64)
            .ok_or("missing sentinel input")?;
        for (formula, actual) in [
            (
                "=DOLLAR(A1)",
                oxfml_core::format::render_currency(&profile, value, 2),
            ),
            (
                "=FIXED(A1)",
                oxfml_core::format::render_fixed(&profile, value, 2, false),
            ),
            (
                "=TEXT(A1,\"0.00\")",
                oxfml_core::format::render_with_code(&profile, date_system, value, "0.00"),
            ),
        ] {
            let expected = sentinels
                .iter()
                .find(|s| s["formula"] == formula)
                .and_then(|s| s["value2"].as_str())
                .ok_or_else(|| format!("missing text sentinel {formula}"))?;
            let actual = actual.map_err(|e| format!("profile sentinel {formula} failed: {e:?}"))?;
            if actual != expected {
                return Err(format!(
                    "profile sentinel mismatch {formula}: provider={actual:?}, Excel={expected:?}"
                ));
            }
            sentinel_validation.push(json!({"formula":formula,"exact_text_match":true}));
        }
    }
    Ok(LocaleBinding {
        context: oxfml_core::format::oxfml_locale_context(profile, date_system),
        provenance: json!({
            "provider":"oxfml_core::format::oxfml_locale_context",
            "profile_id":id.stable_name(), "date_system":record["date_system"],
            "profile_record":record_path, "profile_record_sha256":sha256(&bytes),
            "profile_validation":if recorded_settings {"explicit_recorded_settings_and_three_exact_text_sentinels"} else {"canonical_fields_match_recorded_Excel_regional_settings"},
            "canonical_profile":canonical_profile,"applied_profile":profile_json(&profile),
            "sentinel_validation":sentinel_validation,
            "recorded_application_separators":{"decimal":record["application_decimal_separator"],"thousands":record["application_thousands_separator"]},
            "effective_separator_source":"Excel.International; recorded mode additionally validates DOLLAR/FIXED output sentinels",
            "provider_source_sha256":source_hashes,
            "version_axes":{"excel_version":record["excel_version"],"excel_build":record["excel_build"],
                "channel":record["channel"],"compatibility_version":record["compatibility_version"]},
            "open_profile_lanes":["two_digit_year_pivot_from_Windows_snapshot_not_independently_Excel_probed", "negative_currency_rendering_not_in_profile_sentinels"]
        }),
    })
}

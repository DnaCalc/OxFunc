#[cfg(test)]
use crate::functions::round_fn::round_kernel;
use crate::value::ExcelText;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocaleProfileId {
    EnUs,
    CurrentExcelHost,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkbookDateSystem {
    System1900,
    System1904,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FormatProfile {
    pub id: LocaleProfileId,
    pub decimal_separator: &'static str,
    pub thousands_separator: &'static str,
    pub list_separator: &'static str,
    pub currency_symbol: &'static str,
    pub date_separator: &'static str,
    pub time_separator: &'static str,
    pub currency_decimals: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseFailure {
    UnsupportedText(String),
    UnsupportedByProfile(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormatFailure {
    UnsupportedCode(String),
    InvalidDateSerial,
}

pub trait LocaleValueParser {
    fn parse_value_text(
        &self,
        profile: &FormatProfile,
        date_system: WorkbookDateSystem,
        text: &str,
    ) -> Result<f64, ParseFailure>;
}

pub trait FormatCodeEngine {
    fn render_with_code(
        &self,
        profile: &FormatProfile,
        date_system: WorkbookDateSystem,
        value: f64,
        code: &str,
    ) -> Result<ExcelText, FormatFailure>;

    fn render_currency(
        &self,
        profile: &FormatProfile,
        value: f64,
        decimals: i32,
    ) -> Result<ExcelText, FormatFailure>;

    fn render_fixed(
        &self,
        profile: &FormatProfile,
        value: f64,
        decimals: i32,
        no_commas: bool,
    ) -> Result<ExcelText, FormatFailure>;
}

pub struct LocaleFormatContext<'a> {
    pub profile: FormatProfile,
    pub date_system: WorkbookDateSystem,
    pub parser: &'a dyn LocaleValueParser,
    pub formatter: &'a dyn FormatCodeEngine,
}

#[cfg(test)]
struct TestOnlyLocaleValueParser;
#[cfg(test)]
struct TestOnlyFormatCodeEngine;

#[cfg(test)]
static TEST_ONLY_LOCALE_VALUE_PARSER: TestOnlyLocaleValueParser = TestOnlyLocaleValueParser;
#[cfg(test)]
static TEST_ONLY_FORMAT_CODE_ENGINE: TestOnlyFormatCodeEngine = TestOnlyFormatCodeEngine;

pub const fn format_profile(id: LocaleProfileId) -> FormatProfile {
    match id {
        LocaleProfileId::EnUs => FormatProfile {
            id,
            decimal_separator: ".",
            thousands_separator: ",",
            list_separator: ",",
            currency_symbol: "$",
            date_separator: "/",
            time_separator: ":",
            currency_decimals: 2,
        },
        LocaleProfileId::CurrentExcelHost => FormatProfile {
            id,
            decimal_separator: ".",
            thousands_separator: " ",
            list_separator: ",",
            currency_symbol: "R",
            date_separator: "/",
            time_separator: ":",
            currency_decimals: 2,
        },
    }
}

#[cfg(test)]
pub(crate) fn test_locale_format_context(profile: LocaleProfileId) -> LocaleFormatContext<'static> {
    LocaleFormatContext {
        profile: format_profile(profile),
        date_system: WorkbookDateSystem::System1900,
        parser: &TEST_ONLY_LOCALE_VALUE_PARSER,
        formatter: &TEST_ONLY_FORMAT_CODE_ENGINE,
    }
}

#[cfg(test)]
pub(crate) fn test_en_us_context() -> LocaleFormatContext<'static> {
    test_locale_format_context(LocaleProfileId::EnUs)
}

#[cfg(test)]
pub(crate) fn test_current_excel_host_context() -> LocaleFormatContext<'static> {
    test_locale_format_context(LocaleProfileId::CurrentExcelHost)
}

#[cfg(test)]
fn text_from_string(s: String) -> ExcelText {
    ExcelText::from_utf16_code_units(s.encode_utf16().collect())
}

#[cfg(test)]
fn normalize_numeric_text(profile: &FormatProfile, raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    let (negative, body) = if let Some(rest) = trimmed.strip_prefix('-') {
        (true, rest)
    } else if let Some(rest) = trimmed.strip_prefix('+') {
        (false, rest)
    } else {
        (false, trimmed)
    };

    let mut normalized = body.replace(profile.thousands_separator, "");
    if profile.decimal_separator != "." {
        normalized = normalized.replace(profile.decimal_separator, ".");
    }

    if normalized.matches('.').count() > 1 {
        return None;
    }

    if negative {
        normalized.insert(0, '-');
    }
    Some(normalized)
}

#[cfg(test)]
fn parse_number_with_profile(profile: &FormatProfile, raw: &str) -> Option<f64> {
    let normalized = normalize_numeric_text(profile, raw)?;
    let parsed = normalized.parse::<f64>().ok()?;
    if parsed.is_finite() {
        Some(parsed)
    } else {
        None
    }
}

#[cfg(test)]
fn parse_iso_ymd(text: &str) -> Option<(i64, i64, i64)> {
    let parts: Vec<&str> = text.split('-').collect();
    if parts.len() != 3 {
        return None;
    }
    Some((
        parts[0].parse::<i64>().ok()?,
        parts[1].parse::<i64>().ok()?,
        parts[2].parse::<i64>().ok()?,
    ))
}

#[cfg(test)]
fn parse_en_us_slash_date(text: &str) -> Option<(i64, i64, i64)> {
    let parts: Vec<&str> = text.split('/').collect();
    if parts.len() != 3 {
        return None;
    }
    Some((
        parts[2].parse::<i64>().ok()?,
        parts[0].parse::<i64>().ok()?,
        parts[1].parse::<i64>().ok()?,
    ))
}

fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = year - i64::from(month <= 2);
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let yoe = year - era * 400;
    let mp = month + if month > 2 { -3 } else { 9 };
    let doy = (153 * mp + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = mp + if mp < 10 { 3 } else { -9 };
    let year = y + i64::from(m <= 2);
    (year, m, d)
}

pub fn excel_serial_from_ymd(
    date_system: WorkbookDateSystem,
    year: i64,
    month: i64,
    day: i64,
) -> Option<f64> {
    match date_system {
        WorkbookDateSystem::System1900 => {
            if year == 1900 && month == 2 && day == 29 {
                return Some(60.0);
            }
            let base = days_from_civil(1899, 12, 31);
            let days = days_from_civil(year, month, 1) - base + (day - 1);
            if days < 0 {
                return None;
            }
            Some(if days >= 60 {
                (days + 1) as f64
            } else {
                days as f64
            })
        }
        WorkbookDateSystem::System1904 => {
            let base = days_from_civil(1904, 1, 1);
            let days = days_from_civil(year, month, 1) - base + (day - 1);
            Some(days as f64)
        }
    }
}

pub fn ymd_from_excel_serial(
    date_system: WorkbookDateSystem,
    serial: f64,
) -> Option<(i64, i64, i64)> {
    let whole = serial.trunc() as i64;
    match date_system {
        WorkbookDateSystem::System1900 => {
            if whole == 60 {
                return Some((1900, 2, 29));
            }
            let adjusted = if whole >= 60 { whole - 1 } else { whole };
            let base = days_from_civil(1899, 12, 31);
            Some(civil_from_days(base + adjusted))
        }
        WorkbookDateSystem::System1904 => {
            let base = days_from_civil(1904, 1, 1);
            Some(civil_from_days(base + whole))
        }
    }
}

#[cfg(test)]
impl LocaleValueParser for TestOnlyLocaleValueParser {
    fn parse_value_text(
        &self,
        profile: &FormatProfile,
        date_system: WorkbookDateSystem,
        text: &str,
    ) -> Result<f64, ParseFailure> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Err(ParseFailure::UnsupportedText(trimmed.to_string()));
        }

        if let Some(stripped) = trimmed.strip_suffix('%') {
            return parse_number_with_profile(profile, stripped)
                .map(|v| v / 100.0)
                .ok_or_else(|| ParseFailure::UnsupportedText(trimmed.to_string()));
        }

        let (negative, body) = if let Some(rest) = trimmed.strip_prefix('-') {
            (true, rest.trim_start())
        } else {
            (false, trimmed)
        };
        if let Some(rest) = body.strip_prefix(profile.currency_symbol) {
            let parsed = parse_number_with_profile(profile, rest.trim_start())
                .ok_or_else(|| ParseFailure::UnsupportedText(trimmed.to_string()))?;
            return Ok(if negative { -parsed } else { parsed });
        }

        if let Some((year, month, day)) = parse_iso_ymd(trimmed) {
            return excel_serial_from_ymd(date_system, year, month, day)
                .ok_or_else(|| ParseFailure::UnsupportedText(trimmed.to_string()));
        }

        if profile.id == LocaleProfileId::EnUs {
            if let Some((year, month, day)) = parse_en_us_slash_date(trimmed) {
                return excel_serial_from_ymd(date_system, year, month, day)
                    .ok_or_else(|| ParseFailure::UnsupportedText(trimmed.to_string()));
            }
        }

        parse_number_with_profile(profile, trimmed)
            .ok_or_else(|| ParseFailure::UnsupportedText(trimmed.to_string()))
    }
}

#[cfg(test)]
fn grouped_integer_string(int_part: &str, sep: &str) -> String {
    if int_part.len() <= 3 || sep.is_empty() {
        return int_part.to_string();
    }
    let mut out = String::new();
    let bytes = int_part.as_bytes();
    let first = int_part.len() % 3;
    let mut index = 0;
    if first > 0 {
        out.push_str(&int_part[..first]);
        index = first;
    }
    while index < bytes.len() {
        if !out.is_empty() {
            out.push_str(sep);
        }
        out.push_str(&int_part[index..index + 3]);
        index += 3;
    }
    out
}

#[cfg(test)]
fn render_fixed_common(
    profile: &FormatProfile,
    value: f64,
    decimals: i32,
    use_grouping: bool,
    prefix: &str,
) -> String {
    let rounded = round_kernel(value, decimals);
    let is_negative = rounded.is_sign_negative() && rounded != 0.0;
    let abs_value = rounded.abs();
    let frac_digits = decimals.max(0) as usize;
    let base = format!("{:.*}", frac_digits, abs_value);
    let (int_part, frac_part) = match base.split_once('.') {
        Some((lhs, rhs)) => (lhs.to_string(), Some(rhs.to_string())),
        None => (base, None),
    };
    let grouped = if use_grouping {
        grouped_integer_string(&int_part, profile.thousands_separator)
    } else {
        int_part
    };

    let mut rendered = String::new();
    if is_negative {
        rendered.push('-');
    }
    rendered.push_str(prefix);
    rendered.push_str(&grouped);
    if let Some(frac) = frac_part {
        if frac_digits > 0 {
            rendered.push_str(profile.decimal_separator);
            rendered.push_str(&frac);
        }
    }
    rendered
}

#[cfg(test)]
impl FormatCodeEngine for TestOnlyFormatCodeEngine {
    fn render_with_code(
        &self,
        profile: &FormatProfile,
        date_system: WorkbookDateSystem,
        value: f64,
        code: &str,
    ) -> Result<ExcelText, FormatFailure> {
        let rendered = match code.trim() {
            "0" => render_fixed_common(profile, value, 0, false, ""),
            "0.00" => render_fixed_common(profile, value, 2, false, ""),
            "0%" => {
                let body = render_fixed_common(profile, value * 100.0, 0, false, "");
                format!("{body}%")
            }
            "yyyy-mm-dd" => {
                let Some((year, month, day)) = ymd_from_excel_serial(date_system, value) else {
                    return Err(FormatFailure::InvalidDateSerial);
                };
                format!("{year:04}-{month:02}-{day:02}")
            }
            other => return Err(FormatFailure::UnsupportedCode(other.to_string())),
        };
        Ok(text_from_string(rendered))
    }

    fn render_currency(
        &self,
        profile: &FormatProfile,
        value: f64,
        decimals: i32,
    ) -> Result<ExcelText, FormatFailure> {
        Ok(text_from_string(render_fixed_common(
            profile,
            value,
            decimals,
            true,
            profile.currency_symbol,
        )))
    }

    fn render_fixed(
        &self,
        profile: &FormatProfile,
        value: f64,
        decimals: i32,
        no_commas: bool,
    ) -> Result<ExcelText, FormatFailure> {
        Ok(text_from_string(render_fixed_common(
            profile, value, decimals, !no_commas, "",
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_handles_current_host_seed_rows() {
        let ctx = test_current_excel_host_context();
        assert_eq!(
            ctx.parser
                .parse_value_text(&ctx.profile, ctx.date_system, "1 234.5"),
            Ok(1234.5)
        );
        assert_eq!(
            ctx.parser
                .parse_value_text(&ctx.profile, ctx.date_system, "R1 234.57"),
            Ok(1234.57)
        );
        assert_eq!(
            ctx.parser
                .parse_value_text(&ctx.profile, ctx.date_system, "12%"),
            Ok(0.12)
        );
        assert!(
            ctx.parser
                .parse_value_text(&ctx.profile, ctx.date_system, "1/2/2024")
                .is_err()
        );
        assert_eq!(
            ctx.parser
                .parse_value_text(&ctx.profile, ctx.date_system, "2024-02-03"),
            Ok(45325.0)
        );
    }

    #[test]
    fn parser_handles_en_us_slash_date() {
        let ctx = test_en_us_context();
        assert_eq!(
            ctx.parser
                .parse_value_text(&ctx.profile, ctx.date_system, "1/2/2024"),
            Ok(45293.0)
        );
    }

    #[test]
    fn formatter_handles_current_host_seed_rows() {
        let ctx = test_current_excel_host_context();
        assert_eq!(
            ctx.formatter
                .render_with_code(&ctx.profile, ctx.date_system, 0.125, "0%")
                .unwrap()
                .to_string_lossy(),
            "13%"
        );
        assert_eq!(
            ctx.formatter
                .render_currency(&ctx.profile, -1234.567, 2)
                .unwrap()
                .to_string_lossy(),
            "-R1 234.57"
        );
        assert_eq!(
            ctx.formatter
                .render_fixed(&ctx.profile, 1234.567, 2, false)
                .unwrap()
                .to_string_lossy(),
            "1 234.57"
        );
        assert_eq!(
            ctx.formatter
                .render_with_code(&ctx.profile, ctx.date_system, 45325.0, "yyyy-mm-dd")
                .unwrap()
                .to_string_lossy(),
            "2024-02-03"
        );
    }
}

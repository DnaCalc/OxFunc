#[cfg(test)]
use crate::functions::round_fn::round_kernel;
use crate::value::ExcelText;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocaleProfileId {
    EnUs,
    EnGb,
    EnIe,
    EnAu,
    EnNz,
    EnZa,
    EnIn,
    EnCa,
    EnPh,
    DeDe,
    RuRu,
    FiFi,
    EtEe,
    LvLv,
    LtLt,
    SkSk,
    CsCz,
    NbNo,
    NnNo,
    FrFr,
    EsEs,
    PtPt,
    ItIt,
    NlNl,
    PlPl,
    PtBr,
    JaJp,
    KoKr,
    ZhCn,
    HuHu,
    /// Host-regional-settings placeholder. This is not a stable locale identity;
    /// callers that need reproducible locale-keyed behavior should use an
    /// explicit locale profile id.
    CurrentExcelHost,
}

impl LocaleProfileId {
    pub const fn stable_name(self) -> &'static str {
        match self {
            LocaleProfileId::EnUs => "en-US",
            LocaleProfileId::EnGb => "en-GB",
            LocaleProfileId::EnIe => "en-IE",
            LocaleProfileId::EnAu => "en-AU",
            LocaleProfileId::EnNz => "en-NZ",
            LocaleProfileId::EnZa => "en-ZA",
            LocaleProfileId::EnIn => "en-IN",
            LocaleProfileId::EnCa => "en-CA",
            LocaleProfileId::EnPh => "en-PH",
            LocaleProfileId::DeDe => "de-DE",
            LocaleProfileId::RuRu => "ru-RU",
            LocaleProfileId::FiFi => "fi-FI",
            LocaleProfileId::EtEe => "et-EE",
            LocaleProfileId::LvLv => "lv-LV",
            LocaleProfileId::LtLt => "lt-LT",
            LocaleProfileId::SkSk => "sk-SK",
            LocaleProfileId::CsCz => "cs-CZ",
            LocaleProfileId::NbNo => "nb-NO",
            LocaleProfileId::NnNo => "nn-NO",
            LocaleProfileId::FrFr => "fr-FR",
            LocaleProfileId::EsEs => "es-ES",
            LocaleProfileId::PtPt => "pt-PT",
            LocaleProfileId::ItIt => "it-IT",
            LocaleProfileId::NlNl => "nl-NL",
            LocaleProfileId::PlPl => "pl-PL",
            LocaleProfileId::PtBr => "pt-BR",
            LocaleProfileId::JaJp => "ja-JP",
            LocaleProfileId::KoKr => "ko-KR",
            LocaleProfileId::ZhCn => "zh-CN",
            LocaleProfileId::HuHu => "hu-HU",
            LocaleProfileId::CurrentExcelHost => "current-excel-host",
        }
    }

    pub fn from_bcp47_language_tag(tag: &str) -> Option<Self> {
        let normalized = tag.trim().replace('_', "-").to_ascii_lowercase();
        let mut parts = normalized.split('-').filter(|part| !part.is_empty());
        let language = parts.next()?;
        let region = parts.next().unwrap_or("");

        match (language, region) {
            ("en", "") | ("en", "us") => Some(LocaleProfileId::EnUs),
            ("en", "gb") => Some(LocaleProfileId::EnGb),
            ("en", "ie") => Some(LocaleProfileId::EnIe),
            ("en", "au") => Some(LocaleProfileId::EnAu),
            ("en", "nz") => Some(LocaleProfileId::EnNz),
            ("en", "za") => Some(LocaleProfileId::EnZa),
            ("en", "in") => Some(LocaleProfileId::EnIn),
            ("en", "ca") => Some(LocaleProfileId::EnCa),
            ("en", "ph") => Some(LocaleProfileId::EnPh),
            ("de", _) => Some(LocaleProfileId::DeDe),
            ("ru", _) => Some(LocaleProfileId::RuRu),
            ("fi", _) => Some(LocaleProfileId::FiFi),
            ("et", _) => Some(LocaleProfileId::EtEe),
            ("lv", _) => Some(LocaleProfileId::LvLv),
            ("lt", _) => Some(LocaleProfileId::LtLt),
            ("sk", _) => Some(LocaleProfileId::SkSk),
            ("cs", _) => Some(LocaleProfileId::CsCz),
            ("nb", _) => Some(LocaleProfileId::NbNo),
            ("nn", _) => Some(LocaleProfileId::NnNo),
            ("fr", _) => Some(LocaleProfileId::FrFr),
            ("es", _) => Some(LocaleProfileId::EsEs),
            ("pt", "") | ("pt", "pt") => Some(LocaleProfileId::PtPt),
            ("pt", "br") => Some(LocaleProfileId::PtBr),
            ("it", _) => Some(LocaleProfileId::ItIt),
            ("nl", _) => Some(LocaleProfileId::NlNl),
            ("pl", _) => Some(LocaleProfileId::PlPl),
            ("zh", _) => Some(LocaleProfileId::ZhCn),
            ("ja", _) => Some(LocaleProfileId::JaJp),
            ("ko", _) => Some(LocaleProfileId::KoKr),
            ("hu", _) => Some(LocaleProfileId::HuHu),
            _ => None,
        }
    }

    pub const fn from_excel_lcid(lcid: u16) -> Option<Self> {
        match lcid {
            0x0409 => Some(LocaleProfileId::EnUs),
            0x0809 => Some(LocaleProfileId::EnGb),
            0x1809 => Some(LocaleProfileId::EnIe),
            0x0C09 => Some(LocaleProfileId::EnAu),
            0x1409 => Some(LocaleProfileId::EnNz),
            0x1C09 => Some(LocaleProfileId::EnZa),
            0x4009 => Some(LocaleProfileId::EnIn),
            0x1009 => Some(LocaleProfileId::EnCa),
            0x3409 => Some(LocaleProfileId::EnPh),
            0x0407 | 0x0807 | 0x0C07 => Some(LocaleProfileId::DeDe),
            0x0419 => Some(LocaleProfileId::RuRu),
            0x040B => Some(LocaleProfileId::FiFi),
            0x0425 => Some(LocaleProfileId::EtEe),
            0x0426 => Some(LocaleProfileId::LvLv),
            0x0427 => Some(LocaleProfileId::LtLt),
            0x041B => Some(LocaleProfileId::SkSk),
            0x0405 => Some(LocaleProfileId::CsCz),
            0x0414 => Some(LocaleProfileId::NbNo),
            0x0814 => Some(LocaleProfileId::NnNo),
            0x040C | 0x080C | 0x0C0C | 0x100C => Some(LocaleProfileId::FrFr),
            0x040A | 0x080A | 0x0C0A => Some(LocaleProfileId::EsEs),
            0x0816 => Some(LocaleProfileId::PtPt),
            0x0410 => Some(LocaleProfileId::ItIt),
            0x0413 | 0x0813 => Some(LocaleProfileId::NlNl),
            0x0415 => Some(LocaleProfileId::PlPl),
            0x0416 => Some(LocaleProfileId::PtBr),
            0x0411 => Some(LocaleProfileId::JaJp),
            0x0412 => Some(LocaleProfileId::KoKr),
            0x0804 | 0x1004 | 0x0404 | 0x0C04 => Some(LocaleProfileId::ZhCn),
            0x040E => Some(LocaleProfileId::HuHu),
            _ => None,
        }
    }
}

pub const CANONICAL_LOCALE_PROFILE_IDS: [LocaleProfileId; 30] = [
    LocaleProfileId::EnUs,
    LocaleProfileId::EnGb,
    LocaleProfileId::EnIe,
    LocaleProfileId::EnAu,
    LocaleProfileId::EnNz,
    LocaleProfileId::EnZa,
    LocaleProfileId::EnIn,
    LocaleProfileId::EnCa,
    LocaleProfileId::EnPh,
    LocaleProfileId::DeDe,
    LocaleProfileId::RuRu,
    LocaleProfileId::FiFi,
    LocaleProfileId::EtEe,
    LocaleProfileId::LvLv,
    LocaleProfileId::LtLt,
    LocaleProfileId::SkSk,
    LocaleProfileId::CsCz,
    LocaleProfileId::NbNo,
    LocaleProfileId::NnNo,
    LocaleProfileId::FrFr,
    LocaleProfileId::EsEs,
    LocaleProfileId::PtPt,
    LocaleProfileId::ItIt,
    LocaleProfileId::NlNl,
    LocaleProfileId::PlPl,
    LocaleProfileId::PtBr,
    LocaleProfileId::JaJp,
    LocaleProfileId::KoKr,
    LocaleProfileId::ZhCn,
    LocaleProfileId::HuHu,
];

pub const LOCALE_PROFILE_IDS: [LocaleProfileId; 31] = [
    LocaleProfileId::EnUs,
    LocaleProfileId::EnGb,
    LocaleProfileId::EnIe,
    LocaleProfileId::EnAu,
    LocaleProfileId::EnNz,
    LocaleProfileId::EnZa,
    LocaleProfileId::EnIn,
    LocaleProfileId::EnCa,
    LocaleProfileId::EnPh,
    LocaleProfileId::DeDe,
    LocaleProfileId::RuRu,
    LocaleProfileId::FiFi,
    LocaleProfileId::EtEe,
    LocaleProfileId::LvLv,
    LocaleProfileId::LtLt,
    LocaleProfileId::SkSk,
    LocaleProfileId::CsCz,
    LocaleProfileId::NbNo,
    LocaleProfileId::NnNo,
    LocaleProfileId::FrFr,
    LocaleProfileId::EsEs,
    LocaleProfileId::PtPt,
    LocaleProfileId::ItIt,
    LocaleProfileId::NlNl,
    LocaleProfileId::PlPl,
    LocaleProfileId::PtBr,
    LocaleProfileId::JaJp,
    LocaleProfileId::KoKr,
    LocaleProfileId::ZhCn,
    LocaleProfileId::HuHu,
    LocaleProfileId::CurrentExcelHost,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkbookDateSystem {
    System1900,
    System1904,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateComponentOrder {
    Mdy,
    Dmy,
    Ymd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurrencyPlacement {
    Before,
    After,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurrencySpacing {
    None,
    Space,
    NarrowNoBreakSpace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurrencyNegativePattern {
    LeadingMinus,
    TrailingMinus,
    Parentheses,
    MinusBeforeSymbol,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatCodeTokenPolicy {
    InvariantExcel,
    LocalizedExcel,
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
    pub short_date_order: DateComponentOrder,
    pub short_date_pattern: &'static str,
    pub two_digit_year_pivot: Option<u16>,
    pub currency_placement: CurrencyPlacement,
    pub currency_spacing: CurrencySpacing,
    pub currency_negative_pattern: CurrencyNegativePattern,
    pub format_code_decimal_token: &'static str,
    pub format_code_group_token: &'static str,
    pub format_code_token_policy: FormatCodeTokenPolicy,
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

const fn short_date_order(id: LocaleProfileId) -> DateComponentOrder {
    match id {
        LocaleProfileId::EnUs | LocaleProfileId::EnPh => DateComponentOrder::Mdy,
        LocaleProfileId::EnZa
        | LocaleProfileId::EnCa
        | LocaleProfileId::LtLt
        | LocaleProfileId::JaJp
        | LocaleProfileId::KoKr
        | LocaleProfileId::ZhCn
        | LocaleProfileId::HuHu
        | LocaleProfileId::CurrentExcelHost => DateComponentOrder::Ymd,
        _ => DateComponentOrder::Dmy,
    }
}

const fn short_date_pattern(id: LocaleProfileId) -> &'static str {
    match id {
        LocaleProfileId::EnUs | LocaleProfileId::EnPh => "M/d/yyyy",
        LocaleProfileId::EnGb | LocaleProfileId::EnIe | LocaleProfileId::EnIn => "dd/MM/yyyy",
        LocaleProfileId::EnAu | LocaleProfileId::EsEs => "d/M/yyyy",
        LocaleProfileId::EnNz => "d/MM/yyyy",
        LocaleProfileId::EnZa => "yyyy/MM/dd",
        LocaleProfileId::EnCa | LocaleProfileId::LtLt => "yyyy-MM-dd",
        LocaleProfileId::DeDe
        | LocaleProfileId::RuRu
        | LocaleProfileId::EtEe
        | LocaleProfileId::LvLv
        | LocaleProfileId::CsCz
        | LocaleProfileId::NbNo
        | LocaleProfileId::NnNo => "dd.MM.yyyy",
        LocaleProfileId::FiFi => "d.M.yyyy",
        LocaleProfileId::PlPl => "d.MM.yyyy",
        LocaleProfileId::SkSk => "d. M. yyyy",
        LocaleProfileId::FrFr
        | LocaleProfileId::PtPt
        | LocaleProfileId::ItIt
        | LocaleProfileId::PtBr => "dd/MM/yyyy",
        LocaleProfileId::NlNl => "dd-MM-yyyy",
        LocaleProfileId::JaJp => "yyyy/MM/dd",
        LocaleProfileId::KoKr => "yyyy. M. d.",
        LocaleProfileId::ZhCn => "yyyy/M/d",
        LocaleProfileId::HuHu => "yyyy. MM. dd.",
        LocaleProfileId::CurrentExcelHost => "yyyy/MM/dd",
    }
}

const fn currency_placement(id: LocaleProfileId) -> CurrencyPlacement {
    match id {
        LocaleProfileId::DeDe
        | LocaleProfileId::RuRu
        | LocaleProfileId::FiFi
        | LocaleProfileId::EtEe
        | LocaleProfileId::LvLv
        | LocaleProfileId::LtLt
        | LocaleProfileId::SkSk
        | LocaleProfileId::CsCz
        | LocaleProfileId::NnNo
        | LocaleProfileId::FrFr
        | LocaleProfileId::EsEs
        | LocaleProfileId::PtPt
        | LocaleProfileId::ItIt
        | LocaleProfileId::PlPl
        | LocaleProfileId::HuHu => CurrencyPlacement::After,
        _ => CurrencyPlacement::Before,
    }
}

const fn currency_spacing(id: LocaleProfileId) -> CurrencySpacing {
    match id {
        LocaleProfileId::FrFr => CurrencySpacing::NarrowNoBreakSpace,
        LocaleProfileId::DeDe
        | LocaleProfileId::RuRu
        | LocaleProfileId::FiFi
        | LocaleProfileId::EtEe
        | LocaleProfileId::LvLv
        | LocaleProfileId::LtLt
        | LocaleProfileId::SkSk
        | LocaleProfileId::CsCz
        | LocaleProfileId::NbNo
        | LocaleProfileId::NnNo
        | LocaleProfileId::EsEs
        | LocaleProfileId::PtPt
        | LocaleProfileId::ItIt
        | LocaleProfileId::NlNl
        | LocaleProfileId::PlPl
        | LocaleProfileId::PtBr
        | LocaleProfileId::HuHu => CurrencySpacing::Space,
        _ => CurrencySpacing::None,
    }
}

const fn currency_negative_pattern(id: LocaleProfileId) -> CurrencyNegativePattern {
    match id {
        LocaleProfileId::EnUs
        | LocaleProfileId::EnGb
        | LocaleProfileId::EnIe
        | LocaleProfileId::EnAu
        | LocaleProfileId::EnNz
        | LocaleProfileId::EnZa
        | LocaleProfileId::EnIn
        | LocaleProfileId::EnCa
        | LocaleProfileId::EnPh
        | LocaleProfileId::JaJp
        | LocaleProfileId::KoKr
        | LocaleProfileId::ZhCn
        | LocaleProfileId::CurrentExcelHost => CurrencyNegativePattern::MinusBeforeSymbol,
        LocaleProfileId::PtBr => CurrencyNegativePattern::MinusBeforeSymbol,
        _ => CurrencyNegativePattern::LeadingMinus,
    }
}

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
            short_date_order: short_date_order(id),
            short_date_pattern: short_date_pattern(id),
            two_digit_year_pivot: None,
            currency_placement: currency_placement(id),
            currency_spacing: currency_spacing(id),
            currency_negative_pattern: currency_negative_pattern(id),
            format_code_decimal_token: ".",
            format_code_group_token: ",",
            format_code_token_policy: FormatCodeTokenPolicy::InvariantExcel,
        },
        LocaleProfileId::EnGb => FormatProfile {
            id,
            decimal_separator: ".",
            thousands_separator: ",",
            list_separator: ",",
            currency_symbol: "\u{00A3}",
            date_separator: "/",
            time_separator: ":",
            currency_decimals: 2,
            short_date_order: short_date_order(id),
            short_date_pattern: short_date_pattern(id),
            two_digit_year_pivot: None,
            currency_placement: currency_placement(id),
            currency_spacing: currency_spacing(id),
            currency_negative_pattern: currency_negative_pattern(id),
            format_code_decimal_token: ".",
            format_code_group_token: ",",
            format_code_token_policy: FormatCodeTokenPolicy::InvariantExcel,
        },
        LocaleProfileId::EnIe => FormatProfile {
            id,
            decimal_separator: ".",
            thousands_separator: ",",
            list_separator: ",",
            currency_symbol: "\u{20AC}",
            date_separator: "/",
            time_separator: ":",
            currency_decimals: 2,
            short_date_order: short_date_order(id),
            short_date_pattern: short_date_pattern(id),
            two_digit_year_pivot: None,
            currency_placement: currency_placement(id),
            currency_spacing: currency_spacing(id),
            currency_negative_pattern: currency_negative_pattern(id),
            format_code_decimal_token: ".",
            format_code_group_token: ",",
            format_code_token_policy: FormatCodeTokenPolicy::InvariantExcel,
        },
        LocaleProfileId::EnAu => FormatProfile {
            id,
            decimal_separator: ".",
            thousands_separator: ",",
            list_separator: ",",
            currency_symbol: "$",
            date_separator: "/",
            time_separator: ":",
            currency_decimals: 2,
            short_date_order: short_date_order(id),
            short_date_pattern: short_date_pattern(id),
            two_digit_year_pivot: None,
            currency_placement: currency_placement(id),
            currency_spacing: currency_spacing(id),
            currency_negative_pattern: currency_negative_pattern(id),
            format_code_decimal_token: ".",
            format_code_group_token: ",",
            format_code_token_policy: FormatCodeTokenPolicy::InvariantExcel,
        },
        LocaleProfileId::EnNz => FormatProfile {
            id,
            decimal_separator: ".",
            thousands_separator: ",",
            list_separator: ",",
            currency_symbol: "$",
            date_separator: "/",
            time_separator: ":",
            currency_decimals: 2,
            short_date_order: short_date_order(id),
            short_date_pattern: short_date_pattern(id),
            two_digit_year_pivot: None,
            currency_placement: currency_placement(id),
            currency_spacing: currency_spacing(id),
            currency_negative_pattern: currency_negative_pattern(id),
            format_code_decimal_token: ".",
            format_code_group_token: ",",
            format_code_token_policy: FormatCodeTokenPolicy::InvariantExcel,
        },
        LocaleProfileId::EnZa => FormatProfile {
            id,
            decimal_separator: ",",
            thousands_separator: "\u{00A0}",
            list_separator: ",",
            currency_symbol: "R",
            date_separator: "/",
            time_separator: ":",
            currency_decimals: 2,
            short_date_order: short_date_order(id),
            short_date_pattern: short_date_pattern(id),
            two_digit_year_pivot: None,
            currency_placement: currency_placement(id),
            currency_spacing: currency_spacing(id),
            currency_negative_pattern: currency_negative_pattern(id),
            format_code_decimal_token: ".",
            format_code_group_token: ",",
            format_code_token_policy: FormatCodeTokenPolicy::InvariantExcel,
        },
        LocaleProfileId::EnIn => FormatProfile {
            id,
            decimal_separator: ".",
            thousands_separator: ",",
            list_separator: ",",
            currency_symbol: "\u{20B9}",
            date_separator: "/",
            time_separator: ":",
            currency_decimals: 2,
            short_date_order: short_date_order(id),
            short_date_pattern: short_date_pattern(id),
            two_digit_year_pivot: None,
            currency_placement: currency_placement(id),
            currency_spacing: currency_spacing(id),
            currency_negative_pattern: currency_negative_pattern(id),
            format_code_decimal_token: ".",
            format_code_group_token: ",",
            format_code_token_policy: FormatCodeTokenPolicy::InvariantExcel,
        },
        LocaleProfileId::EnCa => FormatProfile {
            id,
            decimal_separator: ".",
            thousands_separator: ",",
            list_separator: ",",
            currency_symbol: "$",
            date_separator: "-",
            time_separator: ":",
            currency_decimals: 2,
            short_date_order: short_date_order(id),
            short_date_pattern: short_date_pattern(id),
            two_digit_year_pivot: None,
            currency_placement: currency_placement(id),
            currency_spacing: currency_spacing(id),
            currency_negative_pattern: currency_negative_pattern(id),
            format_code_decimal_token: ".",
            format_code_group_token: ",",
            format_code_token_policy: FormatCodeTokenPolicy::InvariantExcel,
        },
        LocaleProfileId::EnPh => FormatProfile {
            id,
            decimal_separator: ".",
            thousands_separator: ",",
            list_separator: ",",
            currency_symbol: "\u{20B1}",
            date_separator: "/",
            time_separator: ":",
            currency_decimals: 2,
            short_date_order: short_date_order(id),
            short_date_pattern: short_date_pattern(id),
            two_digit_year_pivot: None,
            currency_placement: currency_placement(id),
            currency_spacing: currency_spacing(id),
            currency_negative_pattern: currency_negative_pattern(id),
            format_code_decimal_token: ".",
            format_code_group_token: ",",
            format_code_token_policy: FormatCodeTokenPolicy::InvariantExcel,
        },
        LocaleProfileId::DeDe => FormatProfile {
            id,
            decimal_separator: ",",
            thousands_separator: ".",
            list_separator: ";",
            currency_symbol: "\u{20AC}",
            date_separator: ".",
            time_separator: ":",
            currency_decimals: 2,
            short_date_order: short_date_order(id),
            short_date_pattern: short_date_pattern(id),
            two_digit_year_pivot: None,
            currency_placement: currency_placement(id),
            currency_spacing: currency_spacing(id),
            currency_negative_pattern: currency_negative_pattern(id),
            format_code_decimal_token: ".",
            format_code_group_token: ",",
            format_code_token_policy: FormatCodeTokenPolicy::InvariantExcel,
        },
        LocaleProfileId::RuRu => FormatProfile {
            id,
            decimal_separator: ",",
            thousands_separator: "\u{00A0}",
            list_separator: ";",
            currency_symbol: "\u{20BD}",
            date_separator: ".",
            time_separator: ":",
            currency_decimals: 2,
            short_date_order: short_date_order(id),
            short_date_pattern: short_date_pattern(id),
            two_digit_year_pivot: None,
            currency_placement: currency_placement(id),
            currency_spacing: currency_spacing(id),
            currency_negative_pattern: currency_negative_pattern(id),
            format_code_decimal_token: ".",
            format_code_group_token: ",",
            format_code_token_policy: FormatCodeTokenPolicy::InvariantExcel,
        },
        LocaleProfileId::FiFi => FormatProfile {
            id,
            decimal_separator: ",",
            thousands_separator: "\u{00A0}",
            list_separator: ";",
            currency_symbol: "\u{20AC}",
            date_separator: ".",
            time_separator: ".",
            currency_decimals: 2,
            short_date_order: short_date_order(id),
            short_date_pattern: short_date_pattern(id),
            two_digit_year_pivot: None,
            currency_placement: currency_placement(id),
            currency_spacing: currency_spacing(id),
            currency_negative_pattern: currency_negative_pattern(id),
            format_code_decimal_token: ".",
            format_code_group_token: ",",
            format_code_token_policy: FormatCodeTokenPolicy::InvariantExcel,
        },
        LocaleProfileId::EtEe => FormatProfile {
            id,
            decimal_separator: ",",
            thousands_separator: "\u{00A0}",
            list_separator: ";",
            currency_symbol: "\u{20AC}",
            date_separator: ".",
            time_separator: ":",
            currency_decimals: 2,
            short_date_order: short_date_order(id),
            short_date_pattern: short_date_pattern(id),
            two_digit_year_pivot: None,
            currency_placement: currency_placement(id),
            currency_spacing: currency_spacing(id),
            currency_negative_pattern: currency_negative_pattern(id),
            format_code_decimal_token: ".",
            format_code_group_token: ",",
            format_code_token_policy: FormatCodeTokenPolicy::InvariantExcel,
        },
        LocaleProfileId::LvLv => FormatProfile {
            id,
            decimal_separator: ",",
            thousands_separator: "\u{00A0}",
            list_separator: ";",
            currency_symbol: "\u{20AC}",
            date_separator: ".",
            time_separator: ":",
            currency_decimals: 2,
            short_date_order: short_date_order(id),
            short_date_pattern: short_date_pattern(id),
            two_digit_year_pivot: None,
            currency_placement: currency_placement(id),
            currency_spacing: currency_spacing(id),
            currency_negative_pattern: currency_negative_pattern(id),
            format_code_decimal_token: ".",
            format_code_group_token: ",",
            format_code_token_policy: FormatCodeTokenPolicy::InvariantExcel,
        },
        LocaleProfileId::LtLt => FormatProfile {
            id,
            decimal_separator: ",",
            thousands_separator: "\u{00A0}",
            list_separator: ";",
            currency_symbol: "\u{20AC}",
            date_separator: "-",
            time_separator: ":",
            currency_decimals: 2,
            short_date_order: short_date_order(id),
            short_date_pattern: short_date_pattern(id),
            two_digit_year_pivot: None,
            currency_placement: currency_placement(id),
            currency_spacing: currency_spacing(id),
            currency_negative_pattern: currency_negative_pattern(id),
            format_code_decimal_token: ".",
            format_code_group_token: ",",
            format_code_token_policy: FormatCodeTokenPolicy::InvariantExcel,
        },
        LocaleProfileId::SkSk => FormatProfile {
            id,
            decimal_separator: ",",
            thousands_separator: "\u{00A0}",
            list_separator: ";",
            currency_symbol: "\u{20AC}",
            date_separator: ".",
            time_separator: ":",
            currency_decimals: 2,
            short_date_order: short_date_order(id),
            short_date_pattern: short_date_pattern(id),
            two_digit_year_pivot: None,
            currency_placement: currency_placement(id),
            currency_spacing: currency_spacing(id),
            currency_negative_pattern: currency_negative_pattern(id),
            format_code_decimal_token: ".",
            format_code_group_token: ",",
            format_code_token_policy: FormatCodeTokenPolicy::InvariantExcel,
        },
        LocaleProfileId::CsCz => FormatProfile {
            id,
            decimal_separator: ",",
            thousands_separator: "\u{00A0}",
            list_separator: ";",
            currency_symbol: "K\u{010D}",
            date_separator: ".",
            time_separator: ":",
            currency_decimals: 2,
            short_date_order: short_date_order(id),
            short_date_pattern: short_date_pattern(id),
            two_digit_year_pivot: None,
            currency_placement: currency_placement(id),
            currency_spacing: currency_spacing(id),
            currency_negative_pattern: currency_negative_pattern(id),
            format_code_decimal_token: ".",
            format_code_group_token: ",",
            format_code_token_policy: FormatCodeTokenPolicy::InvariantExcel,
        },
        LocaleProfileId::NbNo | LocaleProfileId::NnNo => FormatProfile {
            id,
            decimal_separator: ",",
            thousands_separator: "\u{00A0}",
            list_separator: ";",
            currency_symbol: "kr",
            date_separator: ".",
            time_separator: ":",
            currency_decimals: 2,
            short_date_order: short_date_order(id),
            short_date_pattern: short_date_pattern(id),
            two_digit_year_pivot: None,
            currency_placement: currency_placement(id),
            currency_spacing: currency_spacing(id),
            currency_negative_pattern: currency_negative_pattern(id),
            format_code_decimal_token: ".",
            format_code_group_token: ",",
            format_code_token_policy: FormatCodeTokenPolicy::InvariantExcel,
        },
        LocaleProfileId::FrFr => FormatProfile {
            id,
            decimal_separator: ",",
            thousands_separator: "\u{202F}",
            list_separator: ";",
            currency_symbol: "\u{20AC}",
            date_separator: "/",
            time_separator: ":",
            currency_decimals: 2,
            short_date_order: short_date_order(id),
            short_date_pattern: short_date_pattern(id),
            two_digit_year_pivot: None,
            currency_placement: currency_placement(id),
            currency_spacing: currency_spacing(id),
            currency_negative_pattern: currency_negative_pattern(id),
            format_code_decimal_token: ".",
            format_code_group_token: ",",
            format_code_token_policy: FormatCodeTokenPolicy::InvariantExcel,
        },
        LocaleProfileId::EsEs => FormatProfile {
            id,
            decimal_separator: ",",
            thousands_separator: ".",
            list_separator: ";",
            currency_symbol: "\u{20AC}",
            date_separator: "/",
            time_separator: ":",
            currency_decimals: 2,
            short_date_order: short_date_order(id),
            short_date_pattern: short_date_pattern(id),
            two_digit_year_pivot: None,
            currency_placement: currency_placement(id),
            currency_spacing: currency_spacing(id),
            currency_negative_pattern: currency_negative_pattern(id),
            format_code_decimal_token: ".",
            format_code_group_token: ",",
            format_code_token_policy: FormatCodeTokenPolicy::InvariantExcel,
        },
        LocaleProfileId::PtPt => FormatProfile {
            id,
            decimal_separator: ",",
            thousands_separator: "\u{00A0}",
            list_separator: ";",
            currency_symbol: "\u{20AC}",
            date_separator: "/",
            time_separator: ":",
            currency_decimals: 2,
            short_date_order: short_date_order(id),
            short_date_pattern: short_date_pattern(id),
            two_digit_year_pivot: None,
            currency_placement: currency_placement(id),
            currency_spacing: currency_spacing(id),
            currency_negative_pattern: currency_negative_pattern(id),
            format_code_decimal_token: ".",
            format_code_group_token: ",",
            format_code_token_policy: FormatCodeTokenPolicy::InvariantExcel,
        },
        LocaleProfileId::ItIt => FormatProfile {
            id,
            decimal_separator: ",",
            thousands_separator: ".",
            list_separator: ";",
            currency_symbol: "\u{20AC}",
            date_separator: "/",
            time_separator: ":",
            currency_decimals: 2,
            short_date_order: short_date_order(id),
            short_date_pattern: short_date_pattern(id),
            two_digit_year_pivot: None,
            currency_placement: currency_placement(id),
            currency_spacing: currency_spacing(id),
            currency_negative_pattern: currency_negative_pattern(id),
            format_code_decimal_token: ".",
            format_code_group_token: ",",
            format_code_token_policy: FormatCodeTokenPolicy::InvariantExcel,
        },
        LocaleProfileId::NlNl => FormatProfile {
            id,
            decimal_separator: ",",
            thousands_separator: ".",
            list_separator: ";",
            currency_symbol: "\u{20AC}",
            date_separator: "-",
            time_separator: ":",
            currency_decimals: 2,
            short_date_order: short_date_order(id),
            short_date_pattern: short_date_pattern(id),
            two_digit_year_pivot: None,
            currency_placement: currency_placement(id),
            currency_spacing: currency_spacing(id),
            currency_negative_pattern: currency_negative_pattern(id),
            format_code_decimal_token: ".",
            format_code_group_token: ",",
            format_code_token_policy: FormatCodeTokenPolicy::InvariantExcel,
        },
        LocaleProfileId::PlPl => FormatProfile {
            id,
            decimal_separator: ",",
            thousands_separator: "\u{00A0}",
            list_separator: ";",
            currency_symbol: "z\u{0142}",
            date_separator: ".",
            time_separator: ":",
            currency_decimals: 2,
            short_date_order: short_date_order(id),
            short_date_pattern: short_date_pattern(id),
            two_digit_year_pivot: None,
            currency_placement: currency_placement(id),
            currency_spacing: currency_spacing(id),
            currency_negative_pattern: currency_negative_pattern(id),
            format_code_decimal_token: ".",
            format_code_group_token: ",",
            format_code_token_policy: FormatCodeTokenPolicy::InvariantExcel,
        },
        LocaleProfileId::PtBr => FormatProfile {
            id,
            decimal_separator: ",",
            thousands_separator: ".",
            list_separator: ";",
            currency_symbol: "R$",
            date_separator: "/",
            time_separator: ":",
            currency_decimals: 2,
            short_date_order: short_date_order(id),
            short_date_pattern: short_date_pattern(id),
            two_digit_year_pivot: None,
            currency_placement: currency_placement(id),
            currency_spacing: currency_spacing(id),
            currency_negative_pattern: currency_negative_pattern(id),
            format_code_decimal_token: ".",
            format_code_group_token: ",",
            format_code_token_policy: FormatCodeTokenPolicy::InvariantExcel,
        },
        LocaleProfileId::JaJp => FormatProfile {
            id,
            decimal_separator: ".",
            thousands_separator: ",",
            list_separator: ",",
            currency_symbol: "\u{00A5}",
            date_separator: "/",
            time_separator: ":",
            currency_decimals: 0,
            short_date_order: short_date_order(id),
            short_date_pattern: short_date_pattern(id),
            two_digit_year_pivot: None,
            currency_placement: currency_placement(id),
            currency_spacing: currency_spacing(id),
            currency_negative_pattern: currency_negative_pattern(id),
            format_code_decimal_token: ".",
            format_code_group_token: ",",
            format_code_token_policy: FormatCodeTokenPolicy::InvariantExcel,
        },
        LocaleProfileId::KoKr => FormatProfile {
            id,
            decimal_separator: ".",
            thousands_separator: ",",
            list_separator: ",",
            currency_symbol: "\u{20A9}",
            date_separator: "/",
            time_separator: ":",
            currency_decimals: 0,
            short_date_order: short_date_order(id),
            short_date_pattern: short_date_pattern(id),
            two_digit_year_pivot: None,
            currency_placement: currency_placement(id),
            currency_spacing: currency_spacing(id),
            currency_negative_pattern: currency_negative_pattern(id),
            format_code_decimal_token: ".",
            format_code_group_token: ",",
            format_code_token_policy: FormatCodeTokenPolicy::InvariantExcel,
        },
        LocaleProfileId::ZhCn => FormatProfile {
            id,
            decimal_separator: ".",
            thousands_separator: ",",
            list_separator: ",",
            currency_symbol: "\u{00A5}",
            date_separator: "/",
            time_separator: ":",
            currency_decimals: 2,
            short_date_order: short_date_order(id),
            short_date_pattern: short_date_pattern(id),
            two_digit_year_pivot: None,
            currency_placement: currency_placement(id),
            currency_spacing: currency_spacing(id),
            currency_negative_pattern: currency_negative_pattern(id),
            format_code_decimal_token: ".",
            format_code_group_token: ",",
            format_code_token_policy: FormatCodeTokenPolicy::InvariantExcel,
        },
        LocaleProfileId::HuHu => FormatProfile {
            id,
            decimal_separator: ",",
            thousands_separator: "\u{00A0}",
            list_separator: ";",
            currency_symbol: "Ft",
            date_separator: ".",
            time_separator: ":",
            currency_decimals: 2,
            short_date_order: short_date_order(id),
            short_date_pattern: short_date_pattern(id),
            two_digit_year_pivot: None,
            currency_placement: currency_placement(id),
            currency_spacing: currency_spacing(id),
            currency_negative_pattern: currency_negative_pattern(id),
            format_code_decimal_token: ".",
            format_code_group_token: ",",
            format_code_token_policy: FormatCodeTokenPolicy::InvariantExcel,
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
            short_date_order: short_date_order(id),
            short_date_pattern: short_date_pattern(id),
            two_digit_year_pivot: None,
            currency_placement: currency_placement(id),
            currency_spacing: currency_spacing(id),
            currency_negative_pattern: currency_negative_pattern(id),
            format_code_decimal_token: ".",
            format_code_group_token: ",",
            format_code_token_policy: FormatCodeTokenPolicy::InvariantExcel,
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
const WEEKDAY_ABBREVIATIONS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

#[cfg(test)]
const MONTH_NAMES_FULL: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

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
            if whole == 0 {
                return Some((1900, 1, 0));
            }
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
fn render_two_digit_integer(value: f64) -> Result<String, FormatFailure> {
    if !value.is_finite() {
        return Err(FormatFailure::UnsupportedCode("00".to_string()));
    }
    let rounded = round_kernel(value, 0);
    let magnitude = rounded.abs() as i64;
    if rounded.is_sign_negative() && rounded != 0.0 {
        Ok(format!("-{magnitude:02}"))
    } else {
        Ok(format!("{magnitude:02}"))
    }
}

#[cfg(test)]
fn render_date_component(
    date_system: WorkbookDateSystem,
    value: f64,
    token: &str,
) -> Result<String, FormatFailure> {
    let Some((year, month, day)) = ymd_from_excel_serial(date_system, value) else {
        return Err(FormatFailure::InvalidDateSerial);
    };
    let whole = value.trunc() as i64;
    let weekday_serial = match date_system {
        WorkbookDateSystem::System1900 if whole >= 60 => whole - 1,
        _ => whole,
    };

    match token {
        "dd" => Ok(format!("{day:02}")),
        "DDD" => Ok(WEEKDAY_ABBREVIATIONS[weekday_serial.rem_euclid(7) as usize].to_string()),
        "MMMM" => Ok(MONTH_NAMES_FULL[(month - 1) as usize].to_string()),
        "yyyy-mm-dd" => Ok(format!("{year:04}-{month:02}-{day:02}")),
        other => Err(FormatFailure::UnsupportedCode(other.to_string())),
    }
}

#[cfg(test)]
fn parse_conditional_section(raw: &str) -> Option<(char, f64, &str)> {
    let trimmed = raw.trim_start();
    let op = if trimmed.starts_with("[<") {
        '<'
    } else if trimmed.starts_with("[>") {
        '>'
    } else {
        return None;
    };
    let closing = trimmed.find(']')?;
    let threshold = trimmed[2..closing].parse::<f64>().ok()?;
    Some((op, threshold, &trimmed[closing + 1..]))
}

#[cfg(test)]
fn resolve_conditional_format_section(value: f64, code: &str) -> Option<String> {
    let sections: Vec<&str> = code.split(';').collect();
    if sections.len() < 3 {
        return None;
    }

    for section in &sections[..2] {
        let (op, threshold, body) = parse_conditional_section(section)?;
        let matched = match op {
            '<' => value < threshold,
            '>' => value > threshold,
            _ => false,
        };
        if matched {
            return Some(body.to_string());
        }
    }

    Some(sections[2].trim().to_string())
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
        let trimmed_code = code.trim();
        if let Some(section) = resolve_conditional_format_section(value, trimmed_code) {
            if section.is_empty() || section.chars().all(|ch| ch == ' ') {
                return Ok(text_from_string(section));
            }
            return self.render_with_code(profile, date_system, value, &section);
        }

        let rendered = match trimmed_code {
            "0" => render_fixed_common(profile, value, 0, false, ""),
            "00" => render_two_digit_integer(value)?,
            "0.00" => render_fixed_common(profile, value, 2, false, ""),
            "#,##0.00" => render_fixed_common(profile, value, 2, true, ""),
            "0%" => {
                let body = render_fixed_common(profile, value * 100.0, 0, false, "");
                format!("{body}%")
            }
            "yyyy-mm-dd" | "dd" | "DDD" | "MMMM" => {
                render_date_component(date_system, value, trimmed_code)?
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
                .render_with_code(&ctx.profile, ctx.date_system, 1234567.89, "#,##0.00")
                .unwrap()
                .to_string_lossy(),
            "1 234 567.89"
        );
        assert_eq!(
            ctx.formatter
                .render_with_code(&ctx.profile, ctx.date_system, 0.0, "yyyy-mm-dd")
                .unwrap()
                .to_string_lossy(),
            "1900-01-00"
        );
        assert_eq!(
            ctx.formatter
                .render_with_code(&ctx.profile, ctx.date_system, 45325.0, "yyyy-mm-dd")
                .unwrap()
                .to_string_lossy(),
            "2024-02-03"
        );
        assert_eq!(
            ctx.formatter
                .render_with_code(&ctx.profile, ctx.date_system, 45474.0, "MMMM")
                .unwrap()
                .to_string_lossy(),
            "July"
        );
        assert_eq!(
            ctx.formatter
                .render_with_code(&ctx.profile, ctx.date_system, 45298.0, "DDD")
                .unwrap()
                .to_string_lossy(),
            "Sun"
        );
        assert_eq!(
            ctx.formatter
                .render_with_code(&ctx.profile, ctx.date_system, 15.0, "00")
                .unwrap()
                .to_string_lossy(),
            "15"
        );
        assert_eq!(
            ctx.formatter
                .render_with_code(
                    &ctx.profile,
                    ctx.date_system,
                    45366.0,
                    "[<45352] ;[>45382] ;dd"
                )
                .unwrap()
                .to_string_lossy(),
            "15"
        );
        assert_eq!(
            ctx.formatter
                .render_with_code(
                    &ctx.profile,
                    ctx.date_system,
                    45350.0,
                    "[<45352] ;[>45382] ;dd"
                )
                .unwrap()
                .to_string_lossy(),
            " "
        );

        let en_us = test_en_us_context();
        assert_eq!(
            en_us
                .formatter
                .render_with_code(&en_us.profile, en_us.date_system, 1234567.89, "#,##0.00")
                .unwrap()
                .to_string_lossy(),
            "1,234,567.89"
        );
    }

    #[test]
    fn profile_ids_cover_dna_onecalc_ambient_language_tags() {
        let cases = [
            ("en", LocaleProfileId::EnUs),
            ("en-US", LocaleProfileId::EnUs),
            ("en-GB", LocaleProfileId::EnGb),
            ("en-IE", LocaleProfileId::EnIe),
            ("en-AU", LocaleProfileId::EnAu),
            ("en-NZ", LocaleProfileId::EnNz),
            ("en-ZA", LocaleProfileId::EnZa),
            ("en-IN", LocaleProfileId::EnIn),
            ("en-CA", LocaleProfileId::EnCa),
            ("en-PH", LocaleProfileId::EnPh),
            ("de-DE", LocaleProfileId::DeDe),
            ("de-AT", LocaleProfileId::DeDe),
            ("ru-RU", LocaleProfileId::RuRu),
            ("fi-FI", LocaleProfileId::FiFi),
            ("et-EE", LocaleProfileId::EtEe),
            ("lv-LV", LocaleProfileId::LvLv),
            ("lt-LT", LocaleProfileId::LtLt),
            ("sk-SK", LocaleProfileId::SkSk),
            ("cs-CZ", LocaleProfileId::CsCz),
            ("nb-NO", LocaleProfileId::NbNo),
            ("nn-NO", LocaleProfileId::NnNo),
            ("fr-FR", LocaleProfileId::FrFr),
            ("fr-CA", LocaleProfileId::FrFr),
            ("es-ES", LocaleProfileId::EsEs),
            ("pt-PT", LocaleProfileId::PtPt),
            ("pt-BR", LocaleProfileId::PtBr),
            ("it-IT", LocaleProfileId::ItIt),
            ("nl-NL", LocaleProfileId::NlNl),
            ("pl-PL", LocaleProfileId::PlPl),
            ("zh-CN", LocaleProfileId::ZhCn),
            ("zh-Hant-TW", LocaleProfileId::ZhCn),
            ("ja-JP", LocaleProfileId::JaJp),
            ("ko-KR", LocaleProfileId::KoKr),
            ("hu-HU", LocaleProfileId::HuHu),
            ("de_DE", LocaleProfileId::DeDe),
        ];

        for (tag, expected) in cases {
            assert_eq!(
                LocaleProfileId::from_bcp47_language_tag(tag),
                Some(expected)
            );
        }

        assert_eq!(LocaleProfileId::from_bcp47_language_tag("sv-SE"), None);
        assert_eq!(LocaleProfileId::from_bcp47_language_tag(""), None);
    }

    #[test]
    fn excel_lcid_mapping_covers_supported_locale_profile_aliases() {
        let cases = [
            (0x0409, LocaleProfileId::EnUs),
            (0x0809, LocaleProfileId::EnGb),
            (0x1809, LocaleProfileId::EnIe),
            (0x0C09, LocaleProfileId::EnAu),
            (0x1409, LocaleProfileId::EnNz),
            (0x1C09, LocaleProfileId::EnZa),
            (0x4009, LocaleProfileId::EnIn),
            (0x1009, LocaleProfileId::EnCa),
            (0x3409, LocaleProfileId::EnPh),
            (0x0407, LocaleProfileId::DeDe),
            (0x0807, LocaleProfileId::DeDe),
            (0x0C07, LocaleProfileId::DeDe),
            (0x0419, LocaleProfileId::RuRu),
            (0x040B, LocaleProfileId::FiFi),
            (0x0425, LocaleProfileId::EtEe),
            (0x0426, LocaleProfileId::LvLv),
            (0x0427, LocaleProfileId::LtLt),
            (0x041B, LocaleProfileId::SkSk),
            (0x0405, LocaleProfileId::CsCz),
            (0x0414, LocaleProfileId::NbNo),
            (0x0814, LocaleProfileId::NnNo),
            (0x040C, LocaleProfileId::FrFr),
            (0x080C, LocaleProfileId::FrFr),
            (0x0C0C, LocaleProfileId::FrFr),
            (0x100C, LocaleProfileId::FrFr),
            (0x040A, LocaleProfileId::EsEs),
            (0x080A, LocaleProfileId::EsEs),
            (0x0C0A, LocaleProfileId::EsEs),
            (0x0816, LocaleProfileId::PtPt),
            (0x0410, LocaleProfileId::ItIt),
            (0x0413, LocaleProfileId::NlNl),
            (0x0813, LocaleProfileId::NlNl),
            (0x0415, LocaleProfileId::PlPl),
            (0x0416, LocaleProfileId::PtBr),
            (0x0411, LocaleProfileId::JaJp),
            (0x0412, LocaleProfileId::KoKr),
            (0x0804, LocaleProfileId::ZhCn),
            (0x1004, LocaleProfileId::ZhCn),
            (0x0404, LocaleProfileId::ZhCn),
            (0x0C04, LocaleProfileId::ZhCn),
            (0x040E, LocaleProfileId::HuHu),
        ];

        for (lcid, expected) in cases {
            assert_eq!(LocaleProfileId::from_excel_lcid(lcid), Some(expected));
        }

        assert_eq!(LocaleProfileId::from_excel_lcid(0x041D), None);
    }

    #[test]
    fn expanded_profile_constants_carry_locale_separators_and_currency_defaults() {
        assert_eq!(CANONICAL_LOCALE_PROFILE_IDS.len(), 30);
        assert_eq!(LOCALE_PROFILE_IDS.len(), 31);

        let de = format_profile(LocaleProfileId::DeDe);
        assert_eq!(de.id.stable_name(), LocaleProfileId::DeDe.stable_name());
        assert_eq!(de.decimal_separator, ",");
        assert_eq!(de.thousands_separator, ".");
        assert_eq!(de.list_separator, ";");
        assert_eq!(de.currency_symbol, "\u{20AC}");
        assert_eq!(de.date_separator, ".");
        assert_eq!(de.short_date_order, DateComponentOrder::Dmy);
        assert_eq!(de.short_date_pattern, "dd.MM.yyyy");
        assert_eq!(de.currency_placement, CurrencyPlacement::After);
        assert_eq!(de.currency_spacing, CurrencySpacing::Space);
        assert_eq!(
            de.currency_negative_pattern,
            CurrencyNegativePattern::LeadingMinus
        );
        assert_eq!(
            de.format_code_token_policy,
            FormatCodeTokenPolicy::InvariantExcel
        );
        assert_eq!(de.format_code_decimal_token, ".");
        assert_eq!(de.format_code_group_token, ",");

        let fr = format_profile(LocaleProfileId::FrFr);
        assert_eq!(fr.thousands_separator, "\u{202F}");
        assert_eq!(fr.currency_spacing, CurrencySpacing::NarrowNoBreakSpace);

        let en_za = format_profile(LocaleProfileId::EnZa);
        assert_eq!(en_za.decimal_separator, ",");
        assert_eq!(en_za.thousands_separator, "\u{00A0}");
        assert_eq!(en_za.currency_symbol, "R");
        assert_eq!(en_za.short_date_order, DateComponentOrder::Ymd);
        assert_eq!(en_za.short_date_pattern, "yyyy/MM/dd");

        let fi = format_profile(LocaleProfileId::FiFi);
        assert_eq!(fi.time_separator, ".");

        let ja = format_profile(LocaleProfileId::JaJp);
        assert_eq!(ja.currency_symbol, "\u{00A5}");
        assert_eq!(ja.currency_decimals, 0);
        assert_eq!(ja.short_date_order, DateComponentOrder::Ymd);

        let ko = format_profile(LocaleProfileId::KoKr);
        assert_eq!(ko.currency_symbol, "\u{20A9}");
        assert_eq!(ko.currency_decimals, 0);

        let en_us = format_profile(LocaleProfileId::EnUs);
        assert_eq!(en_us.short_date_order, DateComponentOrder::Mdy);
        assert_eq!(en_us.currency_placement, CurrencyPlacement::Before);
        assert_eq!(en_us.currency_spacing, CurrencySpacing::None);
        assert_eq!(
            en_us.currency_negative_pattern,
            CurrencyNegativePattern::MinusBeforeSymbol
        );
    }
}


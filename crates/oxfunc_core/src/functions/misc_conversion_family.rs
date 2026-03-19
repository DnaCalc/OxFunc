use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    PreparedArgValue, coerce_prepared_to_number, coerce_prepared_to_text, expand_aggregate_arg,
    run_values_only_prepared,
};
use crate::functions::aggregate_common::sum_argument_value;
use crate::resolver::ReferenceResolver;
use crate::value::{
    ArrayCellValue, ArrayShape, CallArgValue, EvalArray, EvalValue, ExcelText, WorksheetErrorCode,
};

const MISC_CONVERSION_META_BASE: FunctionMeta = FunctionMeta {
    function_id: "FUNC.MISC_CONVERSION_BASE",
    arity: Arity::exact(1),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
};

pub const BAHTTEXT_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.BAHTTEXT",
    ..MISC_CONVERSION_META_BASE
};
pub const CONVERT_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.CONVERT",
    arity: Arity::exact(3),
    ..MISC_CONVERSION_META_BASE
};
pub const EUROCONVERT_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.EUROCONVERT",
    arity: Arity { min: 3, max: 5 },
    ..MISC_CONVERSION_META_BASE
};
pub const PERCENTOF_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.PERCENTOF",
    arity: Arity::exact(2),
    ..MISC_CONVERSION_META_BASE
};
pub const RANDARRAY_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.RANDARRAY",
    arity: Arity { min: 0, max: 5 },
    determinism: DeterminismClass::PseudoRandom,
    volatility: VolatilityClass::VolatileFull,
    host_interaction: HostInteractionClass::ApplicationState,
    thread_safety: ThreadSafetyClass::HostSerialized,
    arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::RandomProvider,
    surface_fec_dependency_profile: FecDependencyProfile::RandomProvider,
};

#[derive(Debug, Clone, PartialEq)]
pub enum MiscConversionError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
    Domain(WorksheetErrorCode),
    RandomProviderOutOfRange(f64),
}

pub trait RandomArrayProvider {
    fn random_unit(&self) -> f64;
}

const THAI_DIGITS: [&str; 10] = [
    "ศูนย์",
    "หนึ่ง",
    "สอง",
    "สาม",
    "สี่",
    "ห้า",
    "หก",
    "เจ็ด",
    "แปด",
    "เก้า",
];
const THAI_PLACES: [&str; 6] = ["", "สิบ", "ร้อย", "พัน", "หมื่น", "แสน"];

fn render_thai_under_million(mut value: u32) -> String {
    if value == 0 {
        return String::new();
    }
    let mut digits = [0_u32; 6];
    for idx in 0..6 {
        digits[idx] = value % 10;
        value /= 10;
    }
    let mut out = String::new();
    for pos in (0..6).rev() {
        let digit = digits[pos];
        if digit == 0 {
            continue;
        }
        match pos {
            1 if digit == 1 => out.push_str("สิบ"),
            1 if digit == 2 => out.push_str("ยี่สิบ"),
            1 => {
                out.push_str(THAI_DIGITS[digit as usize]);
                out.push_str("สิบ");
            }
            0 if digit == 1 && digits[1..].iter().any(|d| *d != 0) => out.push_str("เอ็ด"),
            0 => out.push_str(THAI_DIGITS[digit as usize]),
            _ => {
                out.push_str(THAI_DIGITS[digit as usize]);
                out.push_str(THAI_PLACES[pos]);
            }
        }
    }
    out
}

fn render_thai_integer(mut value: u128) -> String {
    if value == 0 {
        return "ศูนย์".to_string();
    }
    let mut groups = Vec::new();
    while value > 0 {
        groups.push((value % 1_000_000) as u32);
        value /= 1_000_000;
    }
    let mut out = String::new();
    for (idx, group) in groups.iter().enumerate().rev() {
        if *group == 0 {
            continue;
        }
        out.push_str(&render_thai_under_million(*group));
        for _ in 0..idx {
            out.push_str("ล้าน");
        }
    }
    out
}

pub fn bahttext_kernel(value: f64) -> Result<ExcelText, WorksheetErrorCode> {
    if !value.is_finite() {
        return Err(WorksheetErrorCode::Value);
    }
    if value < 0.0 || value > 9_999_999_999_999_999.0 {
        return Err(WorksheetErrorCode::Num);
    }
    let satang_total = (value * 100.0).round() as u128;
    let baht = satang_total / 100;
    let satang = (satang_total % 100) as u32;
    let mut rendered = render_thai_integer(baht);
    rendered.push_str("บาท");
    if satang == 0 {
        rendered.push_str("ถ้วน");
    } else {
        rendered.push_str(&render_thai_under_million(satang));
        rendered.push_str("สตางค์");
    }
    Ok(ExcelText::from_utf16_code_units(
        rendered.encode_utf16().collect(),
    ))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConvertCategory {
    Length,
    Mass,
    Time,
    Pressure,
    Volume,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct LinearUnit {
    category: ConvertCategory,
    factor_to_base: f64,
}

fn convert_direct_linear_unit(unit: &str) -> Option<LinearUnit> {
    match unit {
        "m" => Some(LinearUnit {
            category: ConvertCategory::Length,
            factor_to_base: 1.0,
        }),
        "in" => Some(LinearUnit {
            category: ConvertCategory::Length,
            factor_to_base: 0.0254,
        }),
        "ft" => Some(LinearUnit {
            category: ConvertCategory::Length,
            factor_to_base: 0.3048,
        }),
        "yd" => Some(LinearUnit {
            category: ConvertCategory::Length,
            factor_to_base: 0.9144,
        }),
        "mi" => Some(LinearUnit {
            category: ConvertCategory::Length,
            factor_to_base: 1609.344,
        }),
        "Nmi" => Some(LinearUnit {
            category: ConvertCategory::Length,
            factor_to_base: 1852.0,
        }),
        "g" => Some(LinearUnit {
            category: ConvertCategory::Mass,
            factor_to_base: 1.0,
        }),
        "lbm" => Some(LinearUnit {
            category: ConvertCategory::Mass,
            factor_to_base: 453.59237,
        }),
        "ozm" => Some(LinearUnit {
            category: ConvertCategory::Mass,
            factor_to_base: 28.349_523_125,
        }),
        "sec" => Some(LinearUnit {
            category: ConvertCategory::Time,
            factor_to_base: 1.0,
        }),
        "mn" => Some(LinearUnit {
            category: ConvertCategory::Time,
            factor_to_base: 60.0,
        }),
        "hr" => Some(LinearUnit {
            category: ConvertCategory::Time,
            factor_to_base: 3600.0,
        }),
        "day" => Some(LinearUnit {
            category: ConvertCategory::Time,
            factor_to_base: 86_400.0,
        }),
        "Pa" => Some(LinearUnit {
            category: ConvertCategory::Pressure,
            factor_to_base: 1.0,
        }),
        "bar" => Some(LinearUnit {
            category: ConvertCategory::Pressure,
            factor_to_base: 100_000.0,
        }),
        "atm" => Some(LinearUnit {
            category: ConvertCategory::Pressure,
            factor_to_base: 101_325.0,
        }),
        "psi" => Some(LinearUnit {
            category: ConvertCategory::Pressure,
            factor_to_base: 6894.757_293_168,
        }),
        "l" => Some(LinearUnit {
            category: ConvertCategory::Volume,
            factor_to_base: 1.0,
        }),
        "tsp" => Some(LinearUnit {
            category: ConvertCategory::Volume,
            factor_to_base: 0.004_928_921_593_75,
        }),
        "tbs" => Some(LinearUnit {
            category: ConvertCategory::Volume,
            factor_to_base: 0.014_786_764_781_25,
        }),
        "oz" => Some(LinearUnit {
            category: ConvertCategory::Volume,
            factor_to_base: 0.029_573_529_562_5,
        }),
        "cup" => Some(LinearUnit {
            category: ConvertCategory::Volume,
            factor_to_base: 0.236_588_236_5,
        }),
        "pt" => Some(LinearUnit {
            category: ConvertCategory::Volume,
            factor_to_base: 0.473_176_473,
        }),
        "qt" => Some(LinearUnit {
            category: ConvertCategory::Volume,
            factor_to_base: 0.946_352_946,
        }),
        "gal" => Some(LinearUnit {
            category: ConvertCategory::Volume,
            factor_to_base: 3.785_411_784,
        }),
        _ => None,
    }
}

fn convert_prefix_scale(prefix: &str) -> Option<f64> {
    match prefix {
        "Y" => Some(1.0e24),
        "Z" => Some(1.0e21),
        "E" => Some(1.0e18),
        "P" => Some(1.0e15),
        "T" => Some(1.0e12),
        "G" => Some(1.0e9),
        "M" => Some(1.0e6),
        "k" => Some(1.0e3),
        "h" => Some(1.0e2),
        "da" => Some(1.0e1),
        "d" => Some(1.0e-1),
        "c" => Some(1.0e-2),
        "m" => Some(1.0e-3),
        "u" => Some(1.0e-6),
        "n" => Some(1.0e-9),
        "p" => Some(1.0e-12),
        "f" => Some(1.0e-15),
        _ => None,
    }
}

fn convert_prefixable_base(unit: &str) -> Option<LinearUnit> {
    match unit {
        "m" => Some(LinearUnit {
            category: ConvertCategory::Length,
            factor_to_base: 1.0,
        }),
        "g" => Some(LinearUnit {
            category: ConvertCategory::Mass,
            factor_to_base: 1.0,
        }),
        "l" => Some(LinearUnit {
            category: ConvertCategory::Volume,
            factor_to_base: 1.0,
        }),
        "Pa" => Some(LinearUnit {
            category: ConvertCategory::Pressure,
            factor_to_base: 1.0,
        }),
        "sec" => Some(LinearUnit {
            category: ConvertCategory::Time,
            factor_to_base: 1.0,
        }),
        _ => None,
    }
}

fn convert_linear_unit(unit: &str) -> Option<LinearUnit> {
    if let Some(direct) = convert_direct_linear_unit(unit) {
        return Some(direct);
    }
    for base in ["sec", "Pa", "m", "g", "l"] {
        if let Some(prefix) = unit.strip_suffix(base) {
            return Some(LinearUnit {
                category: convert_prefixable_base(base)?.category,
                factor_to_base: convert_prefixable_base(base)?.factor_to_base
                    * convert_prefix_scale(prefix)?,
            });
        }
    }
    None
}

fn convert_temperature_to_kelvin(value: f64, unit: &str) -> Option<f64> {
    match unit {
        "K" => Some(value),
        "C" => Some(value + 273.15),
        "F" => Some((value - 32.0) / 1.8 + 273.15),
        _ => None,
    }
}

fn convert_temperature_from_kelvin(value: f64, unit: &str) -> Option<f64> {
    match unit {
        "K" => Some(value),
        "C" => Some(value - 273.15),
        "F" => Some((value - 273.15) * 1.8 + 32.0),
        _ => None,
    }
}

pub fn convert_kernel(
    number: f64,
    from_unit: &str,
    to_unit: &str,
) -> Result<f64, WorksheetErrorCode> {
    if !number.is_finite() {
        return Err(WorksheetErrorCode::Value);
    }
    let from = from_unit.trim();
    let to = to_unit.trim();
    if let (Some(k), Some(_)) = (
        convert_temperature_to_kelvin(number, from),
        convert_temperature_from_kelvin(0.0, to),
    ) {
        return convert_temperature_from_kelvin(k, to).ok_or(WorksheetErrorCode::NA);
    }
    let from = convert_linear_unit(from).ok_or(WorksheetErrorCode::NA)?;
    let to = convert_linear_unit(to).ok_or(WorksheetErrorCode::NA)?;
    if from.category != to.category {
        return Err(WorksheetErrorCode::NA);
    }
    Ok(number * from.factor_to_base / to.factor_to_base)
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct EuroCurrency {
    rate: f64,
    decimals: i32,
}

fn euro_currency(code: &str) -> Option<EuroCurrency> {
    match code {
        "EUR" => Some(EuroCurrency {
            rate: 1.0,
            decimals: 2,
        }),
        "ATS" => Some(EuroCurrency {
            rate: 13.7603,
            decimals: 2,
        }),
        "BEF" => Some(EuroCurrency {
            rate: 40.3399,
            decimals: 0,
        }),
        "DEM" => Some(EuroCurrency {
            rate: 1.95583,
            decimals: 2,
        }),
        "ESP" => Some(EuroCurrency {
            rate: 166.386,
            decimals: 0,
        }),
        "FIM" => Some(EuroCurrency {
            rate: 5.94573,
            decimals: 2,
        }),
        "FRF" => Some(EuroCurrency {
            rate: 6.55957,
            decimals: 2,
        }),
        "GRD" => Some(EuroCurrency {
            rate: 340.75,
            decimals: 0,
        }),
        "IEP" => Some(EuroCurrency {
            rate: 0.787564,
            decimals: 2,
        }),
        "ITL" => Some(EuroCurrency {
            rate: 1936.27,
            decimals: 0,
        }),
        "LUF" => Some(EuroCurrency {
            rate: 40.3399,
            decimals: 0,
        }),
        "NLG" => Some(EuroCurrency {
            rate: 2.20371,
            decimals: 2,
        }),
        "PTE" => Some(EuroCurrency {
            rate: 200.482,
            decimals: 2,
        }),
        _ => None,
    }
}

fn round_to_decimals(value: f64, decimals: i32) -> f64 {
    let scale = 10_f64.powi(decimals);
    (value * scale).round() / scale
}

fn round_to_significant_digits(value: f64, digits: usize) -> f64 {
    if value == 0.0 {
        return 0.0;
    }
    let exponent = value.abs().log10().floor();
    let scale = 10_f64.powf((digits as f64) - exponent - 1.0);
    (value * scale).round() / scale
}

pub fn euroconvert_kernel(
    number: f64,
    source: &str,
    target: &str,
    full_precision: Option<bool>,
    triangulation_precision: Option<f64>,
) -> Result<f64, WorksheetErrorCode> {
    if !number.is_finite() {
        return Err(WorksheetErrorCode::Value);
    }
    let source = source.trim().to_ascii_uppercase();
    let target = target.trim().to_ascii_uppercase();
    let source_currency = euro_currency(&source).ok_or(WorksheetErrorCode::NA)?;
    let target_currency = euro_currency(&target).ok_or(WorksheetErrorCode::NA)?;
    let tri_digits = match triangulation_precision {
        None => None,
        Some(raw) if !raw.is_finite() => return Err(WorksheetErrorCode::Num),
        Some(raw) => {
            let digits = raw.trunc() as i64;
            if !(3..=15).contains(&digits) {
                return Err(WorksheetErrorCode::Num);
            }
            Some(digits as usize)
        }
    };
    let mut euro_intermediate = if source == "EUR" {
        number
    } else {
        number / source_currency.rate
    };
    if source != "EUR" && target != "EUR" {
        if let Some(digits) = tri_digits {
            euro_intermediate = round_to_significant_digits(euro_intermediate, digits);
        }
    }
    let converted = if target == "EUR" {
        euro_intermediate
    } else {
        euro_intermediate * target_currency.rate
    };
    Ok(if full_precision.unwrap_or(false) {
        converted
    } else {
        round_to_decimals(converted, target_currency.decimals)
    })
}

pub fn percentof_kernel(subset_sum: f64, total_sum: f64) -> Result<f64, WorksheetErrorCode> {
    if !subset_sum.is_finite() || !total_sum.is_finite() {
        return Err(WorksheetErrorCode::Value);
    }
    if total_sum == 0.0 {
        return Err(WorksheetErrorCode::Div0);
    }
    Ok(subset_sum / total_sum)
}

fn validate_random_unit(value: f64) -> Result<f64, MiscConversionError> {
    if value.is_finite() && (0.0..1.0).contains(&value) {
        Ok(value)
    } else {
        Err(MiscConversionError::RandomProviderOutOfRange(value))
    }
}

pub fn RANDARRAY_kernel(
    provider: &impl RandomArrayProvider,
    rows: usize,
    cols: usize,
    min: f64,
    max: f64,
    whole_number: bool,
) -> Result<EvalValue, MiscConversionError> {
    if rows == 0 || cols == 0 || !min.is_finite() || !max.is_finite() {
        return Err(MiscConversionError::Domain(WorksheetErrorCode::Value));
    }
    let cell_count = rows
        .checked_mul(cols)
        .ok_or(MiscConversionError::Domain(WorksheetErrorCode::Num))?;
    let mut cells = Vec::with_capacity(cell_count);
    for _ in 0..cell_count {
        let unit = validate_random_unit(provider.random_unit())?;
        let number = if whole_number {
            let lo = min.ceil();
            let hi = max.floor();
            if hi < lo {
                return Err(MiscConversionError::Domain(WorksheetErrorCode::Num));
            }
            let count = hi - lo + 1.0;
            lo + (unit * count).floor().min(count - 1.0)
        } else {
            if max < min {
                return Err(MiscConversionError::Domain(WorksheetErrorCode::Num));
            }
            if max == min {
                min
            } else {
                min + unit * (max - min)
            }
        };
        cells.push(ArrayCellValue::Number(number));
    }
    Ok(EvalValue::Array(
        EvalArray::new(ArrayShape { rows, cols }, cells)
            .expect("RANDARRAY dimensions were validated"),
    ))
}

fn arity_error(meta: &FunctionMeta, actual: usize) -> MiscConversionError {
    MiscConversionError::ArityMismatch {
        expected_min: meta.arity.min,
        expected_max: meta.arity.max,
        actual,
    }
}

fn coerce_boolish_arg(arg: &PreparedArgValue) -> Result<bool, MiscConversionError> {
    Ok(match arg {
        PreparedArgValue::Eval(EvalValue::Logical(b)) => *b,
        other => coerce_prepared_to_number(other).map_err(MiscConversionError::Coercion)? != 0.0,
    })
}

fn sum_surface_arg(
    arg: &CallArgValue,
    resolver: &impl ReferenceResolver,
) -> Result<f64, MiscConversionError> {
    let prepared = expand_aggregate_arg(arg, resolver).map_err(MiscConversionError::Coercion)?;
    let mut total = 0.0;
    for item in &prepared {
        match sum_argument_value(item).map_err(MiscConversionError::Coercion)? {
            Some(value) => total += value,
            None => {}
        }
    }
    Ok(total)
}

pub fn eval_bahttext_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, MiscConversionError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| {
            if !BAHTTEXT_META.arity.accepts(prepared.len()) {
                return Err(arity_error(&BAHTTEXT_META, prepared.len()));
            }
            let value =
                coerce_prepared_to_number(&prepared[0]).map_err(MiscConversionError::Coercion)?;
            Ok(EvalValue::Text(
                bahttext_kernel(value).map_err(MiscConversionError::Domain)?,
            ))
        },
        MiscConversionError::Coercion,
    )
}

pub fn eval_convert_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, MiscConversionError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| {
            if !CONVERT_META.arity.accepts(prepared.len()) {
                return Err(arity_error(&CONVERT_META, prepared.len()));
            }
            let number =
                coerce_prepared_to_number(&prepared[0]).map_err(MiscConversionError::Coercion)?;
            let from =
                coerce_prepared_to_text(&prepared[1]).map_err(MiscConversionError::Coercion)?;
            let to =
                coerce_prepared_to_text(&prepared[2]).map_err(MiscConversionError::Coercion)?;
            Ok(EvalValue::Number(
                convert_kernel(number, &from.to_string_lossy(), &to.to_string_lossy())
                    .map_err(MiscConversionError::Domain)?,
            ))
        },
        MiscConversionError::Coercion,
    )
}

pub fn eval_euroconvert_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, MiscConversionError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| {
            if !EUROCONVERT_META.arity.accepts(prepared.len()) {
                return Err(arity_error(&EUROCONVERT_META, prepared.len()));
            }
            let number =
                coerce_prepared_to_number(&prepared[0]).map_err(MiscConversionError::Coercion)?;
            let source =
                coerce_prepared_to_text(&prepared[1]).map_err(MiscConversionError::Coercion)?;
            let target =
                coerce_prepared_to_text(&prepared[2]).map_err(MiscConversionError::Coercion)?;
            let full_precision = prepared.get(3).map(coerce_boolish_arg).transpose()?;
            let precision = prepared
                .get(4)
                .map(|arg| coerce_prepared_to_number(arg).map_err(MiscConversionError::Coercion))
                .transpose()?;
            Ok(EvalValue::Number(
                euroconvert_kernel(
                    number,
                    &source.to_string_lossy(),
                    &target.to_string_lossy(),
                    full_precision,
                    precision,
                )
                .map_err(MiscConversionError::Domain)?,
            ))
        },
        MiscConversionError::Coercion,
    )
}

pub fn eval_percentof_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, MiscConversionError> {
    if !PERCENTOF_META.arity.accepts(args.len()) {
        return Err(arity_error(&PERCENTOF_META, args.len()));
    }
    let subset_sum = sum_surface_arg(&args[0], resolver)?;
    let total_sum = sum_surface_arg(&args[1], resolver)?;
    Ok(EvalValue::Number(
        percentof_kernel(subset_sum, total_sum).map_err(MiscConversionError::Domain)?,
    ))
}

pub fn eval_RANDARRAY_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
    provider: &impl RandomArrayProvider,
) -> Result<EvalValue, MiscConversionError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| {
            if !RANDARRAY_META.arity.accepts(prepared.len()) {
                return Err(arity_error(&RANDARRAY_META, prepared.len()));
            }
            let rows = prepared
                .first()
                .map(|arg| coerce_prepared_to_number(arg).map_err(MiscConversionError::Coercion))
                .transpose()?
                .unwrap_or(1.0)
                .trunc();
            let cols = prepared
                .get(1)
                .map(|arg| coerce_prepared_to_number(arg).map_err(MiscConversionError::Coercion))
                .transpose()?
                .unwrap_or(1.0)
                .trunc();
            let min = prepared
                .get(2)
                .map(|arg| coerce_prepared_to_number(arg).map_err(MiscConversionError::Coercion))
                .transpose()?
                .unwrap_or(0.0);
            let max = prepared
                .get(3)
                .map(|arg| coerce_prepared_to_number(arg).map_err(MiscConversionError::Coercion))
                .transpose()?
                .unwrap_or(1.0);
            let whole_number = prepared
                .get(4)
                .map(coerce_boolish_arg)
                .transpose()?
                .unwrap_or(false);
            if rows < 1.0 || cols < 1.0 {
                return Err(MiscConversionError::Domain(WorksheetErrorCode::Value));
            }
            RANDARRAY_kernel(
                provider,
                rows as usize,
                cols as usize,
                min,
                max,
                whole_number,
            )
        },
        MiscConversionError::Coercion,
    )
}

pub fn map_misc_conversion_error_to_ws(error: &MiscConversionError) -> WorksheetErrorCode {
    match error {
        MiscConversionError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        MiscConversionError::Coercion(CoercionError::WorksheetError(code)) => *code,
        MiscConversionError::Coercion(_) => WorksheetErrorCode::Value,
        MiscConversionError::Domain(code) => *code,
        MiscConversionError::RandomProviderOutOfRange(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::VecDeque;

    struct QueueRandomProvider {
        values: RefCell<VecDeque<f64>>,
    }

    impl RandomArrayProvider for QueueRandomProvider {
        fn random_unit(&self) -> f64 {
            self.values
                .borrow_mut()
                .pop_front()
                .expect("test provider exhausted")
        }
    }

    fn assert_close(actual: f64, expected: f64, tolerance: f64) {
        let delta = (actual - expected).abs();
        assert!(
            delta <= tolerance,
            "expected {expected}, got {actual}, delta {delta}"
        );
    }

    #[test]
    fn metadata_matches_owned_family_shape() {
        assert_eq!(BAHTTEXT_META.function_id, "FUNC.BAHTTEXT");
        assert_eq!(CONVERT_META.arity, Arity::exact(3));
        assert_eq!(EUROCONVERT_META.arity, Arity { min: 3, max: 5 });
        assert_eq!(
            PERCENTOF_META.surface_fec_dependency_profile,
            FecDependencyProfile::RefOnly
        );
        assert_eq!(RANDARRAY_META.volatility, VolatilityClass::VolatileFull);
    }

    #[test]
    fn bahttext_renders_integer_and_satang_lanes() {
        assert_eq!(
            bahttext_kernel(1234.0).unwrap().to_string_lossy(),
            "หนึ่งพันสองร้อยสามสิบสี่บาทถ้วน"
        );
        assert_eq!(
            bahttext_kernel(1234.56).unwrap().to_string_lossy(),
            "หนึ่งพันสองร้อยสามสิบสี่บาทห้าสิบหกสตางค์"
        );
        assert_eq!(
            bahttext_kernel(21.01).unwrap().to_string_lossy(),
            "ยี่สิบเอ็ดบาทหนึ่งสตางค์"
        );
        assert_eq!(bahttext_kernel(-1.0), Err(WorksheetErrorCode::Num));
    }

    #[test]
    fn convert_matches_bounded_catalog_examples() {
        assert_close(
            convert_kernel(1.0, "lbm", "kg").unwrap(),
            0.453_592_37,
            1.0e-12,
        );
        assert_close(convert_kernel(68.0, "F", "C").unwrap(), 20.0, 1.0e-12);
        assert_close(convert_kernel(3.5, "km", "m").unwrap(), 3500.0, 1.0e-9);
        assert_close(convert_kernel(500.0, "ml", "l").unwrap(), 0.5, 1.0e-12);
        assert_eq!(
            convert_kernel(2.5, "ft", "sec"),
            Err(WorksheetErrorCode::NA)
        );
        assert_eq!(
            convert_kernel(1.0, "stone", "kg"),
            Err(WorksheetErrorCode::NA)
        );
    }

    #[test]
    fn euroconvert_supports_default_rounding_and_full_precision() {
        assert_close(
            euroconvert_kernel(10.0, "DEM", "EUR", None, None).unwrap(),
            5.11,
            1.0e-12,
        );
        assert_close(
            euroconvert_kernel(10.0, "DEM", "EUR", Some(true), None).unwrap(),
            10.0 / 1.95583,
            1.0e-12,
        );
        assert_close(
            euroconvert_kernel(10.0, "DEM", "FRF", Some(true), Some(6.0)).unwrap(),
            round_to_significant_digits(10.0 / 1.95583, 6) * 6.55957,
            1.0e-12,
        );
        assert_eq!(
            euroconvert_kernel(10.0, "DEM", "USD", None, None),
            Err(WorksheetErrorCode::NA)
        );
        assert_eq!(
            euroconvert_kernel(10.0, "DEM", "FRF", None, Some(2.0)),
            Err(WorksheetErrorCode::Num)
        );
    }

    #[test]
    fn percentof_maps_ratio_and_div0() {
        assert_close(percentof_kernel(15.0, 60.0).unwrap(), 0.25, 1.0e-12);
        assert_eq!(percentof_kernel(1.0, 0.0), Err(WorksheetErrorCode::Div0));
    }

    #[test]
    fn RANDARRAY_builds_scalar_and_rectangular_outputs() {
        let scalar_provider = QueueRandomProvider {
            values: RefCell::new(VecDeque::from([0.25])),
        };
        assert_eq!(
            RANDARRAY_kernel(&scalar_provider, 1, 1, 0.0, 1.0, false).unwrap(),
            EvalValue::Array(
                EvalArray::from_rows(vec![vec![ArrayCellValue::Number(0.25)]]).unwrap()
            )
        );

        let grid_provider = QueueRandomProvider {
            values: RefCell::new(VecDeque::from([0.0, 0.49, 0.50, 0.99])),
        };
        assert_eq!(
            RANDARRAY_kernel(&grid_provider, 2, 2, 10.0, 12.0, true).unwrap(),
            EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Number(10.0), ArrayCellValue::Number(11.0)],
                    vec![ArrayCellValue::Number(11.0), ArrayCellValue::Number(12.0)],
                ])
                .unwrap()
            )
        );
    }

    #[test]
    fn RANDARRAY_rejects_bad_provider_values_and_invalid_ranges() {
        let bad_provider = QueueRandomProvider {
            values: RefCell::new(VecDeque::from([1.5])),
        };
        assert_eq!(
            RANDARRAY_kernel(&bad_provider, 1, 1, 0.0, 1.0, false),
            Err(MiscConversionError::RandomProviderOutOfRange(1.5))
        );
        let range_provider = QueueRandomProvider {
            values: RefCell::new(VecDeque::from([0.0])),
        };
        assert_eq!(
            RANDARRAY_kernel(&range_provider, 1, 1, 5.0, 4.0, false),
            Err(MiscConversionError::Domain(WorksheetErrorCode::Num))
        );
    }
}

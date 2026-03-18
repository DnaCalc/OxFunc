use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    PreparedArgValue, coerce_prepared_to_number, coerce_prepared_to_text, run_values_only_prepared,
};
use crate::functions::base_fn::base_kernel;
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, ExcelText, WorksheetErrorCode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RadixSpec {
    radix: u32,
    bits: u32,
    max_chars: usize,
}

const BIN_SPEC: RadixSpec = RadixSpec {
    radix: 2,
    bits: 10,
    max_chars: 10,
};
const OCT_SPEC: RadixSpec = RadixSpec {
    radix: 8,
    bits: 30,
    max_chars: 10,
};
const HEX_SPEC: RadixSpec = RadixSpec {
    radix: 16,
    bits: 40,
    max_chars: 10,
};

macro_rules! engineering_meta {
    ($id:literal, $min:expr, $max:expr) => {
        FunctionMeta {
            function_id: $id,
            arity: Arity {
                min: $min,
                max: $max,
            },
            determinism: DeterminismClass::Deterministic,
            volatility: VolatilityClass::NonVolatile,
            host_interaction: HostInteractionClass::None,
            thread_safety: ThreadSafetyClass::SafePure,
            arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
            coercion_lift_profile: CoercionLiftProfile::Custom,
            kernel_signature_class: KernelSignatureClass::Custom,
            fec_dependency_profile: FecDependencyProfile::None,
            surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
        }
    };
}

pub const DEC2BIN_META: FunctionMeta = engineering_meta!("FUNC.DEC2BIN", 1, 2);
pub const DEC2HEX_META: FunctionMeta = engineering_meta!("FUNC.DEC2HEX", 1, 2);
pub const DEC2OCT_META: FunctionMeta = engineering_meta!("FUNC.DEC2OCT", 1, 2);
pub const BIN2DEC_META: FunctionMeta = engineering_meta!("FUNC.BIN2DEC", 1, 1);
pub const BIN2HEX_META: FunctionMeta = engineering_meta!("FUNC.BIN2HEX", 1, 2);
pub const BIN2OCT_META: FunctionMeta = engineering_meta!("FUNC.BIN2OCT", 1, 2);
pub const HEX2BIN_META: FunctionMeta = engineering_meta!("FUNC.HEX2BIN", 1, 2);
pub const HEX2DEC_META: FunctionMeta = engineering_meta!("FUNC.HEX2DEC", 1, 1);
pub const HEX2OCT_META: FunctionMeta = engineering_meta!("FUNC.HEX2OCT", 1, 2);
pub const OCT2BIN_META: FunctionMeta = engineering_meta!("FUNC.OCT2BIN", 1, 2);
pub const OCT2DEC_META: FunctionMeta = engineering_meta!("FUNC.OCT2DEC", 1, 1);
pub const OCT2HEX_META: FunctionMeta = engineering_meta!("FUNC.OCT2HEX", 1, 2);

#[derive(Debug, Clone, PartialEq)]
pub enum EngineeringRadixEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
}

fn map_domain_result(result: Result<EvalValue, WorksheetErrorCode>) -> EvalValue {
    match result {
        Ok(value) => value,
        Err(code) => EvalValue::Error(code),
    }
}

fn validate_arity(
    meta: &FunctionMeta,
    args: &[PreparedArgValue],
) -> Result<(), EngineeringRadixEvalError> {
    if meta.arity.accepts(args.len()) {
        Ok(())
    } else {
        Err(EngineeringRadixEvalError::ArityMismatch {
            expected_min: meta.arity.min,
            expected_max: meta.arity.max,
            actual: args.len(),
        })
    }
}

fn prepared_number_arg(
    args: &[PreparedArgValue],
    index: usize,
) -> Result<f64, EngineeringRadixEvalError> {
    coerce_prepared_to_number(&args[index]).map_err(EngineeringRadixEvalError::Coercion)
}

fn prepared_text_arg(
    args: &[PreparedArgValue],
    index: usize,
) -> Result<String, EngineeringRadixEvalError> {
    Ok(coerce_prepared_to_text(&args[index])
        .map_err(EngineeringRadixEvalError::Coercion)?
        .to_string_lossy())
}

fn prepared_optional_places_arg(
    args: &[PreparedArgValue],
) -> Result<Option<f64>, EngineeringRadixEvalError> {
    if args.len() > 1 {
        prepared_number_arg(args, 1).map(Some)
    } else {
        Ok(None)
    }
}

fn upper_ascii_digit_value(ch: char) -> Option<u32> {
    match ch {
        '0'..='9' => Some((ch as u8 - b'0') as u32),
        'A'..='Z' => Some((ch as u8 - b'A') as u32 + 10),
        _ => None,
    }
}

fn normalize_radix_text(text: &str) -> String {
    text.trim_start().to_ascii_uppercase()
}

fn parse_radix_text(text: &str, source: RadixSpec) -> Result<i64, WorksheetErrorCode> {
    let normalized = normalize_radix_text(text);
    if normalized.is_empty() || normalized.len() > source.max_chars {
        return Err(WorksheetErrorCode::Num);
    }

    let mut raw: u64 = 0;
    for ch in normalized.chars() {
        let Some(digit) = upper_ascii_digit_value(ch) else {
            return Err(WorksheetErrorCode::Num);
        };
        if digit >= source.radix {
            return Err(WorksheetErrorCode::Num);
        }
        raw = raw
            .checked_mul(source.radix as u64)
            .and_then(|value| value.checked_add(digit as u64))
            .ok_or(WorksheetErrorCode::Num)?;
    }

    if normalized.len() == source.max_chars {
        let sign_mask = 1_u64 << (source.bits - 1);
        if raw & sign_mask != 0 {
            return Ok(raw as i64 - (1_i64 << source.bits));
        }
    }

    Ok(raw as i64)
}

fn normalize_places(places: Option<f64>) -> Result<Option<usize>, WorksheetErrorCode> {
    match places {
        None => Ok(None),
        Some(raw) => {
            let truncated = raw.trunc();
            if truncated <= 0.0 {
                Err(WorksheetErrorCode::Num)
            } else {
                Ok(Some(truncated as usize))
            }
        }
    }
}

fn min_signed_value(spec: RadixSpec) -> i64 {
    -(1_i64 << (spec.bits - 1))
}

fn max_signed_value(spec: RadixSpec) -> i64 {
    (1_i64 << (spec.bits - 1)) - 1
}

fn encode_signed_value(
    value: i64,
    target: RadixSpec,
    places: Option<f64>,
) -> Result<ExcelText, WorksheetErrorCode> {
    if value < min_signed_value(target) || value > max_signed_value(target) {
        return Err(WorksheetErrorCode::Num);
    }

    if value < 0 {
        let raw = ((1_i128 << target.bits) + i128::from(value)) as f64;
        return base_kernel(raw, target.radix as f64, Some(target.max_chars as f64));
    }

    let digits = base_kernel(value as f64, target.radix as f64, None)?;
    let digits_string = digits.to_string_lossy();

    match normalize_places(places)? {
        Some(min_length) => {
            if digits_string.len() > min_length {
                Err(WorksheetErrorCode::Num)
            } else {
                let padded = format!("{digits_string:0>width$}", width = min_length);
                Ok(ExcelText::from_utf16_code_units(
                    padded.encode_utf16().collect(),
                ))
            }
        }
        None => Ok(digits),
    }
}

fn trunc_i64(number: f64) -> i64 {
    number.trunc() as i64
}

fn dec_to_target_kernel(
    number: f64,
    places: Option<f64>,
    target: RadixSpec,
) -> Result<ExcelText, WorksheetErrorCode> {
    encode_signed_value(trunc_i64(number), target, places)
}

fn source_to_decimal_kernel(
    source_text: &str,
    source: RadixSpec,
) -> Result<f64, WorksheetErrorCode> {
    parse_radix_text(source_text, source).map(|value| value as f64)
}

fn source_to_target_kernel(
    source_text: &str,
    places: Option<f64>,
    source: RadixSpec,
    target: RadixSpec,
) -> Result<ExcelText, WorksheetErrorCode> {
    let decimal = parse_radix_text(source_text, source)?;
    encode_signed_value(decimal, target, places)
}

pub fn dec2bin_kernel(number: f64, places: Option<f64>) -> Result<ExcelText, WorksheetErrorCode> {
    dec_to_target_kernel(number, places, BIN_SPEC)
}

pub fn dec2hex_kernel(number: f64, places: Option<f64>) -> Result<ExcelText, WorksheetErrorCode> {
    dec_to_target_kernel(number, places, HEX_SPEC)
}

pub fn dec2oct_kernel(number: f64, places: Option<f64>) -> Result<ExcelText, WorksheetErrorCode> {
    dec_to_target_kernel(number, places, OCT_SPEC)
}

pub fn bin2dec_kernel(number: &str) -> Result<f64, WorksheetErrorCode> {
    source_to_decimal_kernel(number, BIN_SPEC)
}

pub fn bin2hex_kernel(number: &str, places: Option<f64>) -> Result<ExcelText, WorksheetErrorCode> {
    source_to_target_kernel(number, places, BIN_SPEC, HEX_SPEC)
}

pub fn bin2oct_kernel(number: &str, places: Option<f64>) -> Result<ExcelText, WorksheetErrorCode> {
    source_to_target_kernel(number, places, BIN_SPEC, OCT_SPEC)
}

pub fn hex2bin_kernel(number: &str, places: Option<f64>) -> Result<ExcelText, WorksheetErrorCode> {
    source_to_target_kernel(number, places, HEX_SPEC, BIN_SPEC)
}

pub fn hex2dec_kernel(number: &str) -> Result<f64, WorksheetErrorCode> {
    source_to_decimal_kernel(number, HEX_SPEC)
}

pub fn hex2oct_kernel(number: &str, places: Option<f64>) -> Result<ExcelText, WorksheetErrorCode> {
    source_to_target_kernel(number, places, HEX_SPEC, OCT_SPEC)
}

pub fn oct2bin_kernel(number: &str, places: Option<f64>) -> Result<ExcelText, WorksheetErrorCode> {
    source_to_target_kernel(number, places, OCT_SPEC, BIN_SPEC)
}

pub fn oct2dec_kernel(number: &str) -> Result<f64, WorksheetErrorCode> {
    source_to_decimal_kernel(number, OCT_SPEC)
}

pub fn oct2hex_kernel(number: &str, places: Option<f64>) -> Result<ExcelText, WorksheetErrorCode> {
    source_to_target_kernel(number, places, OCT_SPEC, HEX_SPEC)
}

fn eval_dec_to_target_prepared(
    meta: &FunctionMeta,
    args: &[PreparedArgValue],
    kernel: fn(f64, Option<f64>) -> Result<ExcelText, WorksheetErrorCode>,
) -> Result<EvalValue, EngineeringRadixEvalError> {
    validate_arity(meta, args)?;
    let number = prepared_number_arg(args, 0)?;
    let places = prepared_optional_places_arg(args)?;
    Ok(map_domain_result(
        kernel(number, places).map(EvalValue::Text),
    ))
}

fn eval_source_to_decimal_prepared(
    meta: &FunctionMeta,
    args: &[PreparedArgValue],
    kernel: fn(&str) -> Result<f64, WorksheetErrorCode>,
) -> Result<EvalValue, EngineeringRadixEvalError> {
    validate_arity(meta, args)?;
    let number = prepared_text_arg(args, 0)?;
    Ok(map_domain_result(kernel(&number).map(EvalValue::Number)))
}

fn eval_source_to_target_prepared(
    meta: &FunctionMeta,
    args: &[PreparedArgValue],
    kernel: fn(&str, Option<f64>) -> Result<ExcelText, WorksheetErrorCode>,
) -> Result<EvalValue, EngineeringRadixEvalError> {
    validate_arity(meta, args)?;
    let number = prepared_text_arg(args, 0)?;
    let places = prepared_optional_places_arg(args)?;
    Ok(map_domain_result(
        kernel(&number, places).map(EvalValue::Text),
    ))
}

pub fn eval_dec2bin_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, EngineeringRadixEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| eval_dec_to_target_prepared(&DEC2BIN_META, prepared, dec2bin_kernel),
        EngineeringRadixEvalError::Coercion,
    )
}

pub fn eval_dec2hex_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, EngineeringRadixEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| eval_dec_to_target_prepared(&DEC2HEX_META, prepared, dec2hex_kernel),
        EngineeringRadixEvalError::Coercion,
    )
}

pub fn eval_dec2oct_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, EngineeringRadixEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| eval_dec_to_target_prepared(&DEC2OCT_META, prepared, dec2oct_kernel),
        EngineeringRadixEvalError::Coercion,
    )
}

pub fn eval_bin2dec_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, EngineeringRadixEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| eval_source_to_decimal_prepared(&BIN2DEC_META, prepared, bin2dec_kernel),
        EngineeringRadixEvalError::Coercion,
    )
}

pub fn eval_bin2hex_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, EngineeringRadixEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| eval_source_to_target_prepared(&BIN2HEX_META, prepared, bin2hex_kernel),
        EngineeringRadixEvalError::Coercion,
    )
}

pub fn eval_bin2oct_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, EngineeringRadixEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| eval_source_to_target_prepared(&BIN2OCT_META, prepared, bin2oct_kernel),
        EngineeringRadixEvalError::Coercion,
    )
}

pub fn eval_hex2bin_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, EngineeringRadixEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| eval_source_to_target_prepared(&HEX2BIN_META, prepared, hex2bin_kernel),
        EngineeringRadixEvalError::Coercion,
    )
}

pub fn eval_hex2dec_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, EngineeringRadixEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| eval_source_to_decimal_prepared(&HEX2DEC_META, prepared, hex2dec_kernel),
        EngineeringRadixEvalError::Coercion,
    )
}

pub fn eval_hex2oct_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, EngineeringRadixEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| eval_source_to_target_prepared(&HEX2OCT_META, prepared, hex2oct_kernel),
        EngineeringRadixEvalError::Coercion,
    )
}

pub fn eval_oct2bin_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, EngineeringRadixEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| eval_source_to_target_prepared(&OCT2BIN_META, prepared, oct2bin_kernel),
        EngineeringRadixEvalError::Coercion,
    )
}

pub fn eval_oct2dec_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, EngineeringRadixEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| eval_source_to_decimal_prepared(&OCT2DEC_META, prepared, oct2dec_kernel),
        EngineeringRadixEvalError::Coercion,
    )
}

pub fn eval_oct2hex_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, EngineeringRadixEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| eval_source_to_target_prepared(&OCT2HEX_META, prepared, oct2hex_kernel),
        EngineeringRadixEvalError::Coercion,
    )
}

pub fn map_engineering_radix_error_to_ws(error: &EngineeringRadixEvalError) -> WorksheetErrorCode {
    match error {
        EngineeringRadixEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        EngineeringRadixEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        EngineeringRadixEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_text_eq(result: Result<ExcelText, WorksheetErrorCode>, expected: &str) {
        let actual = result.unwrap().to_string_lossy();
        assert_eq!(actual, expected);
    }

    #[test]
    fn decimal_to_engineering_radix_matches_seed_rows() {
        assert_text_eq(dec2bin_kernel(9.0, Some(4.0)), "1001");
        assert_text_eq(dec2bin_kernel(-100.0, Some(2.0)), "1110011100");
        assert_text_eq(dec2hex_kernel(100.0, Some(4.0)), "0064");
        assert_text_eq(dec2hex_kernel(-54.0, Some(3.0)), "FFFFFFFFCA");
        assert_text_eq(dec2oct_kernel(58.0, Some(3.0)), "072");
        assert_text_eq(dec2oct_kernel(-100.0, None), "7777777634");
    }

    #[test]
    fn decimal_to_engineering_radix_enforces_ranges_and_places() {
        assert_eq!(dec2bin_kernel(512.0, None), Err(WorksheetErrorCode::Num));
        assert_eq!(
            dec2hex_kernel(549_755_813_888.0, None),
            Err(WorksheetErrorCode::Num)
        );
        assert_eq!(
            dec2oct_kernel(58.0, Some(1.0)),
            Err(WorksheetErrorCode::Num)
        );
        assert_eq!(dec2bin_kernel(1.0, Some(0.0)), Err(WorksheetErrorCode::Num));
        assert_text_eq(dec2hex_kernel(255.0, Some(4.8)), "00FF");
    }

    #[test]
    fn radix_to_decimal_matches_signed_ten_digit_rules() {
        assert_eq!(bin2dec_kernel("1100100"), Ok(100.0));
        assert_eq!(bin2dec_kernel("1111111111"), Ok(-1.0));
        assert_eq!(hex2dec_kernel("3DA408B9"), Ok(1_034_160_313.0));
        assert_eq!(hex2dec_kernel("FFFFFFFF5B"), Ok(-165.0));
        assert_eq!(oct2dec_kernel("54"), Ok(44.0));
        assert_eq!(oct2dec_kernel("7777777533"), Ok(-165.0));
    }

    #[test]
    fn cross_radix_text_conversions_match_seed_rows() {
        assert_text_eq(bin2hex_kernel("11111011", Some(4.0)), "00FB");
        assert_text_eq(bin2hex_kernel("1111111111", Some(1.0)), "FFFFFFFFFF");
        assert_text_eq(bin2oct_kernel("1001", Some(3.0)), "011");
        assert_text_eq(hex2bin_kernel("F", Some(8.0)), "00001111");
        assert_text_eq(hex2oct_kernel("FFFFFFFF5B", Some(1.0)), "7777777533");
        assert_text_eq(oct2bin_kernel("3", Some(4.0)), "0011");
        assert_text_eq(oct2hex_kernel("100", Some(4.0)), "0040");
    }

    #[test]
    fn cross_radix_text_conversions_reject_invalid_inputs() {
        assert_eq!(bin2hex_kernel("102", None), Err(WorksheetErrorCode::Num));
        assert_eq!(
            bin2oct_kernel("11111111111", None),
            Err(WorksheetErrorCode::Num)
        );
        assert_eq!(
            hex2bin_kernel("1000000000", None),
            Err(WorksheetErrorCode::Num)
        );
        assert_eq!(hex2oct_kernel("", None), Err(WorksheetErrorCode::Num));
        assert_eq!(oct2bin_kernel("8", None), Err(WorksheetErrorCode::Num));
    }
}

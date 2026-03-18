use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    PreparedArgValue, coerce_prepared_to_number, coerce_prepared_to_text, run_values_only_prepared,
};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

pub const DECIMAL_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.DECIMAL",
    arity: Arity::exact(2),
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

#[derive(Debug, Clone, PartialEq)]
pub enum DecimalEvalError {
    ArityMismatch { expected: usize, actual: usize },
    Coercion(CoercionError),
}

fn digit_value(c: char) -> Option<u32> {
    match c {
        '0'..='9' => Some((c as u8 - b'0') as u32),
        'A'..='Z' => Some((c as u8 - b'A') as u32 + 10),
        _ => None,
    }
}

pub fn decimal_kernel(text: &str, radix: f64) -> Result<f64, WorksheetErrorCode> {
    let radix = radix.trunc();
    if !(2.0..=36.0).contains(&radix) {
        return Err(WorksheetErrorCode::Num);
    }
    let radix = radix as u32;
    let normalized = text.trim_start().to_ascii_uppercase();
    if normalized.is_empty() {
        return Ok(0.0);
    }

    let mut value: u128 = 0;
    for ch in normalized.chars() {
        let Some(digit) = digit_value(ch) else {
            return Err(WorksheetErrorCode::Num);
        };
        if digit >= radix {
            return Err(WorksheetErrorCode::Num);
        }
        value = value
            .checked_mul(radix as u128)
            .and_then(|v| v.checked_add(digit as u128))
            .ok_or(WorksheetErrorCode::Num)?;
    }
    Ok(value as f64)
}

fn eval_decimal_prepared(args: &[PreparedArgValue]) -> Result<EvalValue, DecimalEvalError> {
    if !DECIMAL_META.arity.accepts(args.len()) {
        return Err(DecimalEvalError::ArityMismatch {
            expected: DECIMAL_META.arity.min,
            actual: args.len(),
        });
    }
    let text = coerce_prepared_to_text(&args[0]).map_err(DecimalEvalError::Coercion)?;
    let radix = coerce_prepared_to_number(&args[1]).map_err(DecimalEvalError::Coercion)?;
    match decimal_kernel(&text.to_string_lossy(), radix) {
        Ok(value) => Ok(EvalValue::Number(value)),
        Err(code) => Ok(EvalValue::Error(code)),
    }
}

pub fn eval_decimal_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DecimalEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_decimal_prepared,
        DecimalEvalError::Coercion,
    )
}

pub fn map_decimal_error_to_ws(e: &DecimalEvalError) -> WorksheetErrorCode {
    match e {
        DecimalEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        DecimalEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        DecimalEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decimal_kernel_matches_excel_seed_rows() {
        assert_eq!(decimal_kernel("FF", 16.0), Ok(255.0));
        assert_eq!(decimal_kernel("111", 2.0), Ok(7.0));
        assert_eq!(decimal_kernel("Z", 36.0), Ok(35.0));
        assert_eq!(decimal_kernel("G", 16.0), Err(WorksheetErrorCode::Num));
        assert_eq!(decimal_kernel("10", 1.0), Err(WorksheetErrorCode::Num));
        assert_eq!(decimal_kernel("10", 37.0), Err(WorksheetErrorCode::Num));
        assert_eq!(decimal_kernel("", 16.0), Ok(0.0));
        assert_eq!(decimal_kernel("ff", 16.0), Ok(255.0));
        assert_eq!(decimal_kernel("10.5", 2.0), Err(WorksheetErrorCode::Num));
        assert_eq!(decimal_kernel("  10", 2.0), Ok(2.0));
        assert_eq!(decimal_kernel("10\t", 2.0), Err(WorksheetErrorCode::Num));
        assert_eq!(decimal_kernel("10", 2.9), Ok(2.0));
    }
}

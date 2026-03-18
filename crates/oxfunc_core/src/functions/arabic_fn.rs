use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{coerce_prepared_to_text, run_values_only_prepared};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, ExcelText, WorksheetErrorCode};

pub const ARABIC_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.ARABIC",
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

#[derive(Debug, Clone, PartialEq)]
pub enum ArabicEvalError {
    ArityMismatch { expected: usize, actual: usize },
    Coercion(CoercionError),
    Domain(WorksheetErrorCode),
}

fn roman_value(ch: char) -> Option<i64> {
    match ch {
        'I' => Some(1),
        'V' => Some(5),
        'X' => Some(10),
        'L' => Some(50),
        'C' => Some(100),
        'D' => Some(500),
        'M' => Some(1000),
        _ => None,
    }
}

pub fn arabic_kernel(text: &ExcelText) -> Result<f64, WorksheetErrorCode> {
    let s = String::from_utf16_lossy(text.utf16_code_units()).to_uppercase();
    if s.is_empty() {
        return Ok(0.0);
    }
    let mut total = 0i64;
    let mut prev = 0i64;
    for ch in s.chars().rev() {
        let value = roman_value(ch).ok_or(WorksheetErrorCode::Value)?;
        if value < prev {
            total -= value;
        } else {
            total += value;
            prev = value;
        }
    }
    Ok(total as f64)
}

pub fn eval_arabic_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, ArabicEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| {
            if !ARABIC_META.arity.accepts(prepared.len()) {
                return Err(ArabicEvalError::ArityMismatch {
                    expected: ARABIC_META.arity.min,
                    actual: prepared.len(),
                });
            }
            let text = coerce_prepared_to_text(&prepared[0]).map_err(ArabicEvalError::Coercion)?;
            arabic_kernel(&text)
                .map(EvalValue::Number)
                .map_err(ArabicEvalError::Domain)
        },
        ArabicEvalError::Coercion,
    )
}

pub fn map_arabic_error_to_ws(e: &ArabicEvalError) -> WorksheetErrorCode {
    match e {
        ArabicEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        ArabicEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        ArabicEvalError::Coercion(_) => WorksheetErrorCode::Value,
        ArabicEvalError::Domain(code) => *code,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn txt(s: &str) -> ExcelText {
        ExcelText::from_utf16_code_units(s.encode_utf16().collect())
    }

    #[test]
    fn arabic_kernel_matches_excel_seed_rows() {
        assert_eq!(arabic_kernel(&txt("LVII")), Ok(57.0));
        assert_eq!(arabic_kernel(&txt("mcmxii")), Ok(1912.0));
        assert_eq!(arabic_kernel(&txt("")), Ok(0.0));
        assert_eq!(arabic_kernel(&txt("IV")), Ok(4.0));
        assert_eq!(arabic_kernel(&txt("ABC")), Err(WorksheetErrorCode::Value));
    }
}

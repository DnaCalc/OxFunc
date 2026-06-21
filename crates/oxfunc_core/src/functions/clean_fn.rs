use crate::coercion::CoercionError;
use crate::function::{
    Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile, FunctionMeta,
    HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{coerce_prepared_to_text, run_values_only_prepared_lifted};
use crate::resolver::ReferenceSystemProvider;
use crate::value::CalcValue;
use crate::value::{ExcelText, WorksheetErrorCode};

pub const CLEAN_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.CLEAN",
    arity: Arity::exact(1),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: FunctionMeta::DEFAULT_ARG_PREPARATION_PROFILE,
    coercion_lift_profile: CoercionLiftProfile::None,
    lift_broadcast_profile: FunctionMeta::DEFAULT_LIFT_BROADCAST_PROFILE,
    kernel_signature_class: KernelSignatureClass::TextToText,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
    real_result_policy: FunctionMeta::DEFAULT_REAL_RESULT_POLICY,
    error_collapse_profile: FunctionMeta::DEFAULT_ERROR_COLLAPSE_PROFILE,
    precision_rounding_profile: FunctionMeta::DEFAULT_PRECISION_ROUNDING_PROFILE,
};

#[derive(Debug, Clone, PartialEq)]
pub enum CleanEvalError {
    ArityMismatch { expected: usize, actual: usize },
    Coercion(CoercionError),
}

pub fn excel_clean_removes_utf16_unit(u: u16) -> bool {
    u < 32 || matches!(u, 129 | 141 | 143 | 144 | 157)
}

pub fn clean_kernel(text: &ExcelText) -> ExcelText {
    let filtered: Vec<u16> = text
        .utf16_code_units()
        .iter()
        .copied()
        .filter(|u| !excel_clean_removes_utf16_unit(*u))
        .collect();
    ExcelText::from_utf16_code_units(filtered)
}

pub fn eval_clean_adapter_prepared(args: &[CalcValue]) -> Result<CalcValue, CleanEvalError> {
    if !CLEAN_META.arity.accepts(args.len()) {
        return Err(CleanEvalError::ArityMismatch {
            expected: CLEAN_META.arity.min,
            actual: args.len(),
        });
    }

    let text = coerce_prepared_to_text(&args[0]).map_err(CleanEvalError::Coercion)?;
    Ok(CalcValue::text(clean_kernel(&text)))
}

pub fn eval_clean_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, CleanEvalError> {
    run_values_only_prepared_lifted(
        args,
        resolver,
        eval_clean_adapter_prepared,
        map_clean_error_to_ws,
        CleanEvalError::Coercion,
    )
}

pub fn map_clean_error_to_ws(e: &CleanEvalError) -> WorksheetErrorCode {
    match e {
        CleanEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        CleanEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        CleanEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::ReferenceSystemCapabilities;

    struct NoResolver;

    impl ReferenceSystemProvider for NoResolver {
        fn capabilities(&self) -> ReferenceSystemCapabilities {
            ReferenceSystemCapabilities::permissive_local()
        }

        fn dereference(
            &self,
            request: &crate::resolver::ReferenceDereferenceRequest,
        ) -> Result<CalcValue, crate::resolver::ReferenceResolutionError> {
            let reference = &request.reference;
            Err(
                crate::resolver::ReferenceResolutionError::UnresolvedReference {
                    target: reference.target().to_string(),
                },
            )
        }
    }

    #[test]
    fn clean_kernel_removes_low_ascii_control_chars() {
        let input = ExcelText::from_utf16_code_units(vec![65, 9, 66, 31, 67]);
        assert_eq!(
            clean_kernel(&input),
            ExcelText::from_utf16_code_units("ABC".encode_utf16().collect())
        );
    }

    #[test]
    fn clean_kernel_removes_excel_c1_subset() {
        let input = ExcelText::from_utf16_code_units(vec![129, 65, 141, 66, 143, 144, 157, 67]);
        assert_eq!(
            clean_kernel(&input),
            ExcelText::from_utf16_code_units("ABC".encode_utf16().collect())
        );
    }

    #[test]
    fn clean_kernel_preserves_char_127_and_zero_width_space() {
        let input = ExcelText::from_utf16_code_units(vec![127, 8203, 65]);
        assert_eq!(clean_kernel(&input), input);
    }

    #[test]
    fn eval_clean_coerces_logical_to_text() {
        let got = eval_clean_surface(&[(CalcValue::logical(true))], &NoResolver);
        assert_eq!(
            got,
            Ok(CalcValue::text(ExcelText::from_utf16_code_units(
                "TRUE".encode_utf16().collect(),
            )))
        );
    }
}

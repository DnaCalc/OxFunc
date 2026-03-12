use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{PreparedArgValue, coerce_prepared_to_text, run_values_only_prepared};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, ExcelText, WorksheetErrorCode};

pub const CLEAN_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.CLEAN",
    arity: Arity::exact(1),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
    coercion_lift_profile: CoercionLiftProfile::None,
    kernel_signature_class: KernelSignatureClass::TextToText,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
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

pub fn eval_clean_adapter_prepared(args: &[PreparedArgValue]) -> Result<EvalValue, CleanEvalError> {
    if !CLEAN_META.arity.accepts(args.len()) {
        return Err(CleanEvalError::ArityMismatch {
            expected: CLEAN_META.arity.min,
            actual: args.len(),
        });
    }

    let text = coerce_prepared_to_text(&args[0]).map_err(CleanEvalError::Coercion)?;
    Ok(EvalValue::Text(clean_kernel(&text)))
}

pub fn eval_clean_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, CleanEvalError> {
    run_values_only_prepared(args, resolver, eval_clean_adapter_prepared, CleanEvalError::Coercion)
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
    use crate::resolver::{RefResolutionError, ResolverCapabilities};
    use crate::value::ReferenceLike;

    struct NoResolver;

    impl ReferenceResolver for NoResolver {
        fn capabilities(&self) -> ResolverCapabilities {
            ResolverCapabilities::permissive_local()
        }

        fn resolve_reference(
            &self,
            reference: &ReferenceLike,
        ) -> Result<EvalValue, RefResolutionError> {
            Err(RefResolutionError::UnresolvedReference {
                target: reference.target.clone(),
            })
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
        let got = eval_clean_surface(&[CallArgValue::Eval(EvalValue::Logical(true))], &NoResolver);
        assert_eq!(
            got,
            Ok(EvalValue::Text(ExcelText::from_utf16_code_units(
                "TRUE".encode_utf16().collect(),
            )))
        );
    }
}

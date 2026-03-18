use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    PreparedArgValue, coerce_prepared_to_text, run_values_only_prepared,
};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

pub const EXACT_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.EXACT",
    arity: Arity::exact(2),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
    coercion_lift_profile: CoercionLiftProfile::None,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
};

#[derive(Debug, Clone, PartialEq)]
pub enum ExactEvalError {
    ArityMismatch { expected: usize, actual: usize },
    Coercion(CoercionError),
}

pub fn exact_kernel(lhs: &crate::value::ExcelText, rhs: &crate::value::ExcelText) -> bool {
    lhs.utf16_code_units() == rhs.utf16_code_units()
}

pub fn eval_exact_adapter_prepared(args: &[PreparedArgValue]) -> Result<EvalValue, ExactEvalError> {
    if !EXACT_META.arity.accepts(args.len()) {
        return Err(ExactEvalError::ArityMismatch {
            expected: EXACT_META.arity.min,
            actual: args.len(),
        });
    }

    let lhs = coerce_prepared_to_text(&args[0]).map_err(ExactEvalError::Coercion)?;
    let rhs = coerce_prepared_to_text(&args[1]).map_err(ExactEvalError::Coercion)?;
    Ok(EvalValue::Logical(exact_kernel(&lhs, &rhs)))
}

pub fn eval_exact_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, ExactEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_exact_adapter_prepared,
        ExactEvalError::Coercion,
    )
}

pub fn map_exact_error_to_ws(e: &ExactEvalError) -> WorksheetErrorCode {
    match e {
        ExactEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        ExactEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        ExactEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::{RefResolutionError, ResolverCapabilities};
    use crate::value::{ExcelText, ReferenceLike};

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
    fn eval_exact_is_case_sensitive() {
        let got = eval_exact_surface(
            &[
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
                    "Abc".encode_utf16().collect(),
                ))),
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
                    "abc".encode_utf16().collect(),
                ))),
            ],
            &NoResolver,
        );
        assert_eq!(got, Ok(EvalValue::Logical(false)));
    }

    #[test]
    fn eval_exact_coerces_numbers_to_text() {
        let got = eval_exact_surface(
            &[
                CallArgValue::Eval(EvalValue::Number(1.0)),
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
                    "1".encode_utf16().collect(),
                ))),
            ],
            &NoResolver,
        );
        assert_eq!(got, Ok(EvalValue::Logical(true)));
    }

    #[test]
    fn eval_exact_coerces_logical_to_text() {
        let got = eval_exact_surface(
            &[
                CallArgValue::Eval(EvalValue::Logical(true)),
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
                    "TRUE".encode_utf16().collect(),
                ))),
            ],
            &NoResolver,
        );
        assert_eq!(got, Ok(EvalValue::Logical(true)));
    }

    #[test]
    fn eval_exact_treats_blank_as_empty_text() {
        let got = eval_exact_adapter_prepared(&[
            PreparedArgValue::EmptyCell,
            PreparedArgValue::Eval(EvalValue::Text(
                ExcelText::from_utf16_code_units(Vec::new()),
            )),
        ]);
        assert_eq!(got, Ok(EvalValue::Logical(true)));
    }

    #[test]
    fn eval_exact_distinguishes_precomposed_and_combining_unicode() {
        let got = eval_exact_surface(
            &[
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(vec![233]))),
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(vec![
                    101, 769,
                ]))),
            ],
            &NoResolver,
        );
        assert_eq!(got, Ok(EvalValue::Logical(false)));
    }

    #[test]
    fn eval_exact_matches_identical_surrogate_pair_text() {
        let emoji = ExcelText::from_utf16_code_units(vec![0xD83D, 0xDE00]);
        let got = eval_exact_surface(
            &[
                CallArgValue::Eval(EvalValue::Text(emoji.clone())),
                CallArgValue::Eval(EvalValue::Text(emoji)),
            ],
            &NoResolver,
        );
        assert_eq!(got, Ok(EvalValue::Logical(true)));
    }
}

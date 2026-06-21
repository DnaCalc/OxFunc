use crate::coercion::CoercionError;
use crate::function::{
    Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile, FunctionMeta,
    HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{coerce_prepared_to_text, run_values_only_prepared};
use crate::resolver::ReferenceSystemProvider;
use crate::value::CalcValue;
use crate::value::WorksheetErrorCode;

pub const EXACT_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.EXACT",
    arity: Arity::exact(2),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: FunctionMeta::DEFAULT_ARG_PREPARATION_PROFILE,
    coercion_lift_profile: CoercionLiftProfile::None,
    lift_broadcast_profile: FunctionMeta::DEFAULT_LIFT_BROADCAST_PROFILE,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
    real_result_policy: FunctionMeta::DEFAULT_REAL_RESULT_POLICY,
    error_collapse_profile: FunctionMeta::DEFAULT_ERROR_COLLAPSE_PROFILE,
    precision_rounding_profile: FunctionMeta::DEFAULT_PRECISION_ROUNDING_PROFILE,
};

#[derive(Debug, Clone, PartialEq)]
pub enum ExactEvalError {
    ArityMismatch { expected: usize, actual: usize },
    Coercion(CoercionError),
}

pub fn exact_kernel(lhs: &crate::value::ExcelText, rhs: &crate::value::ExcelText) -> bool {
    lhs.utf16_code_units() == rhs.utf16_code_units()
}

pub fn eval_exact_adapter_prepared(args: &[CalcValue]) -> Result<CalcValue, ExactEvalError> {
    if !EXACT_META.arity.accepts(args.len()) {
        return Err(ExactEvalError::ArityMismatch {
            expected: EXACT_META.arity.min,
            actual: args.len(),
        });
    }

    let lhs = coerce_prepared_to_text(&args[0]).map_err(ExactEvalError::Coercion)?;
    let rhs = coerce_prepared_to_text(&args[1]).map_err(ExactEvalError::Coercion)?;
    Ok(CalcValue::logical(exact_kernel(&lhs, &rhs)))
}

pub fn eval_exact_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, ExactEvalError> {
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
    use crate::resolver::ReferenceSystemCapabilities;
    use crate::value::ExcelText;

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
    fn eval_exact_is_case_sensitive() {
        let got = eval_exact_surface(
            &[
                (CalcValue::text(ExcelText::from_utf16_code_units(
                    "Abc".encode_utf16().collect(),
                ))),
                (CalcValue::text(ExcelText::from_utf16_code_units(
                    "abc".encode_utf16().collect(),
                ))),
            ],
            &NoResolver,
        );
        assert_eq!(got, Ok(CalcValue::logical(false)));
    }

    #[test]
    fn eval_exact_coerces_numbers_to_text() {
        let got = eval_exact_surface(
            &[
                (CalcValue::number(1.0)),
                (CalcValue::text(ExcelText::from_utf16_code_units(
                    "1".encode_utf16().collect(),
                ))),
            ],
            &NoResolver,
        );
        assert_eq!(got, Ok(CalcValue::logical(true)));
    }

    #[test]
    fn eval_exact_coerces_logical_to_text() {
        let got = eval_exact_surface(
            &[
                (CalcValue::logical(true)),
                (CalcValue::text(ExcelText::from_utf16_code_units(
                    "TRUE".encode_utf16().collect(),
                ))),
            ],
            &NoResolver,
        );
        assert_eq!(got, Ok(CalcValue::logical(true)));
    }

    #[test]
    fn eval_exact_treats_blank_as_empty_text() {
        let got = eval_exact_adapter_prepared(&[
            CalcValue::empty(),
            (CalcValue::text(ExcelText::from_utf16_code_units(Vec::new()))),
        ]);
        assert_eq!(got, Ok(CalcValue::logical(true)));
    }

    #[test]
    fn eval_exact_distinguishes_precomposed_and_combining_unicode() {
        let got = eval_exact_surface(
            &[
                (CalcValue::text(ExcelText::from_utf16_code_units(vec![233]))),
                (CalcValue::text(ExcelText::from_utf16_code_units(vec![101, 769]))),
            ],
            &NoResolver,
        );
        assert_eq!(got, Ok(CalcValue::logical(false)));
    }

    #[test]
    fn eval_exact_matches_identical_surrogate_pair_text() {
        let emoji = ExcelText::from_utf16_code_units(vec![0xD83D, 0xDE00]);
        let got = eval_exact_surface(
            &[(CalcValue::text(emoji.clone())), (CalcValue::text(emoji))],
            &NoResolver,
        );
        assert_eq!(got, Ok(CalcValue::logical(true)));
    }
}

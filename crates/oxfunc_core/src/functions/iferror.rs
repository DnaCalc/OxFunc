use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, ErrorCollapseProfile,
    FecDependencyProfile, FunctionMeta, HostInteractionClass, KernelSignatureClass,
    ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::prepare_arg_values_only;
use crate::resolver::ReferenceSystemProvider;
use crate::value::{CalcValue, CoreValue, WorksheetErrorCode};

pub const IFERROR_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.IFERROR",
    arity: Arity::exact(2),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::RefsVisibleInAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    lift_broadcast_profile: FunctionMeta::DEFAULT_LIFT_BROADCAST_PROFILE,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
    real_result_policy: FunctionMeta::DEFAULT_REAL_RESULT_POLICY,
    error_collapse_profile: ErrorCollapseProfile::SelectorBranch,
    precision_rounding_profile: FunctionMeta::DEFAULT_PRECISION_ROUNDING_PROFILE,
};

#[derive(Debug, Clone, PartialEq)]
pub enum IfErrorEvalError {
    ArityMismatch { expected: usize, actual: usize },
    PrimaryPreparation(CoercionError),
    FallbackPreparation(CoercionError),
}

fn prepared_to_eval(arg: CalcValue) -> CalcValue {
    match arg.core() {
        CoreValue::Missing => CalcValue::error(WorksheetErrorCode::Value),
        CoreValue::Empty => CalcValue::number(0.0),
        _ => arg,
    }
}

pub fn eval_iferror_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, IfErrorEvalError> {
    if !IFERROR_META.arity.accepts(args.len()) {
        return Err(IfErrorEvalError::ArityMismatch {
            expected: IFERROR_META.arity.min,
            actual: args.len(),
        });
    }

    let primary = prepare_arg_values_only(&args[0], resolver)
        .map_err(IfErrorEvalError::PrimaryPreparation)?;
    match primary.core() {
        CoreValue::Error(_) => {
            let fallback = prepare_arg_values_only(&args[1], resolver)
                .map_err(IfErrorEvalError::FallbackPreparation)?;
            Ok(prepared_to_eval(fallback))
        }
        _ => Ok(prepared_to_eval(primary)),
    }
}

pub fn map_iferror_error_to_ws(e: &IfErrorEvalError) -> WorksheetErrorCode {
    match e {
        IfErrorEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        IfErrorEvalError::PrimaryPreparation(CoercionError::WorksheetError(code)) => *code,
        IfErrorEvalError::FallbackPreparation(CoercionError::WorksheetError(code)) => *code,
        _ => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::ReferenceSystemCapabilities;
    use crate::value::{ExcelText, ReferenceLike};

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
    fn eval_iferror_returns_primary_when_not_error() {
        let got = eval_iferror_surface(
            &[(CalcValue::number(2.0)), (CalcValue::number(4.0))],
            &NoResolver,
        );
        assert_eq!(got, Ok(CalcValue::number(2.0)));
    }

    #[test]
    fn eval_iferror_returns_fallback_for_error_primary() {
        let got = eval_iferror_surface(
            &[
                (CalcValue::error(WorksheetErrorCode::Div0)),
                (CalcValue::text(ExcelText::from_utf16_code_units(
                    "alt".encode_utf16().collect(),
                ))),
            ],
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(CalcValue::text(ExcelText::from_utf16_code_units(
                "alt".encode_utf16().collect(),
            )))
        );
    }

    #[test]
    fn eval_iferror_does_not_touch_fallback_when_primary_is_not_error() {
        let got = eval_iferror_surface(
            &[
                CalcValue::empty(),
                CalcValue::reference(ReferenceLike::new(
                    crate::value::ReferenceKind::A1,
                    "Z99".to_string(),
                )),
            ],
            &NoResolver,
        );
        assert_eq!(got, Ok(CalcValue::number(0.0)));
    }

    #[test]
    fn eval_iferror_blank_fallback_becomes_zero_and_missing_fallback_is_value_error() {
        let blank_fallback = eval_iferror_surface(
            &[
                (CalcValue::error(WorksheetErrorCode::NA)),
                CalcValue::empty(),
            ],
            &NoResolver,
        );
        assert_eq!(blank_fallback, Ok(CalcValue::number(0.0)));

        let missing_fallback = eval_iferror_surface(
            &[
                (CalcValue::error(WorksheetErrorCode::NA)),
                CalcValue::missing(),
            ],
            &NoResolver,
        );
        assert_eq!(
            missing_fallback,
            Ok(CalcValue::error(WorksheetErrorCode::Value))
        );
    }
}

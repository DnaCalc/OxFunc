use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::prepare_arg_values_only;
use crate::resolver::ReferenceSystemProvider;
use crate::value::{CalcValue, CoreValue, WorksheetErrorCode};

pub const IFNA_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.IFNA",
    arity: Arity::exact(2),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::RefsVisibleInAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
};

#[derive(Debug, Clone, PartialEq)]
pub enum IfNaEvalError {
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

pub fn eval_ifna_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, IfNaEvalError> {
    if !IFNA_META.arity.accepts(args.len()) {
        return Err(IfNaEvalError::ArityMismatch {
            expected: IFNA_META.arity.min,
            actual: args.len(),
        });
    }

    let primary =
        prepare_arg_values_only(&args[0], resolver).map_err(IfNaEvalError::PrimaryPreparation)?;
    match primary.core() {
        CoreValue::Error(WorksheetErrorCode::NA) => {
            let fallback = prepare_arg_values_only(&args[1], resolver)
                .map_err(IfNaEvalError::FallbackPreparation)?;
            Ok(prepared_to_eval(fallback))
        }
        _ => Ok(prepared_to_eval(primary)),
    }
}

pub fn map_ifna_error_to_ws(e: &IfNaEvalError) -> WorksheetErrorCode {
    match e {
        IfNaEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        IfNaEvalError::PrimaryPreparation(CoercionError::WorksheetError(code)) => *code,
        IfNaEvalError::FallbackPreparation(CoercionError::WorksheetError(code)) => *code,
        _ => WorksheetErrorCode::Value,
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
    fn ifna_catches_only_na() {
        assert_eq!(
            eval_ifna_surface(
                &[
                    (CalcValue::error(WorksheetErrorCode::NA)),
                    (CalcValue::number(7.0)),
                ],
                &NoResolver,
            ),
            Ok(CalcValue::number(7.0))
        );
        assert_eq!(
            eval_ifna_surface(
                &[
                    (CalcValue::error(WorksheetErrorCode::Div0)),
                    (CalcValue::number(7.0)),
                ],
                &NoResolver,
            ),
            Ok(CalcValue::error(WorksheetErrorCode::Div0))
        );
    }

    #[test]
    fn ifna_returns_primary_when_not_na() {
        assert_eq!(
            eval_ifna_surface(
                &[
                    (CalcValue::text(ExcelText::from_utf16_code_units(
                        "x".encode_utf16().collect(),
                    ))),
                    (CalcValue::number(7.0)),
                ],
                &NoResolver,
            ),
            Ok(CalcValue::text(ExcelText::from_utf16_code_units(
                "x".encode_utf16().collect(),
            )))
        );
    }
}

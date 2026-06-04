use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::prepare_arg_values_only;
use crate::resolver::ReferenceSystemProvider;
use crate::value::{CalcValue, CoreValue, WorksheetErrorCode};

pub const TYPE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.TYPE",
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
pub enum TypeEvalError {
    ArityMismatch { expected: usize, actual: usize },
    Coercion(CoercionError),
}

fn type_code(prepared: CalcValue) -> f64 {
    match prepared.core() {
        CoreValue::Number(_) => 1.0,
        CoreValue::Text(_) => 2.0,
        CoreValue::Logical(_) => 4.0,
        CoreValue::Error(_) => 16.0,
        CoreValue::Array(_) => 64.0,
        CoreValue::Reference(_) => 16.0,
        CoreValue::Missing | CoreValue::Empty => 1.0,
    }
}

pub fn eval_type_surface(
    args: &[crate::value::CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, TypeEvalError> {
    if !TYPE_META.arity.accepts(args.len()) {
        return Err(TypeEvalError::ArityMismatch {
            expected: TYPE_META.arity.min,
            actual: args.len(),
        });
    }
    let prepared = prepare_arg_values_only(&args[0], resolver).map_err(TypeEvalError::Coercion)?;
    Ok(CalcValue::number(type_code(prepared)))
}

pub fn map_type_error_to_ws(e: &TypeEvalError) -> WorksheetErrorCode {
    match e {
        TypeEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        TypeEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        TypeEvalError::Coercion(_) => WorksheetErrorCode::Value,
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
    fn eval_type_returns_expected_codes() {
        assert_eq!(
            eval_type_surface(&[(CalcValue::number(1.0))], &NoResolver),
            Ok(CalcValue::number(1.0))
        );
        assert_eq!(
            eval_type_surface(
                &[(CalcValue::text(ExcelText::from_utf16_code_units(
                    "x".encode_utf16().collect(),
                )))],
                &NoResolver,
            ),
            Ok(CalcValue::number(2.0))
        );
    }
}

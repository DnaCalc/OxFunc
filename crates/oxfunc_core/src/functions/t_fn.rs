use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{PreparedValue, prepare_arg_values_only};
use crate::resolver::ReferenceSystemProvider;
use crate::value::{
    ExcelText, FunctionArray, FunctionArrayCell, FunctionValue, WorksheetErrorCode,
};

pub const T_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.T",
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
pub enum TEvalError {
    ArityMismatch { expected: usize, actual: usize },
    Coercion(CoercionError),
}

fn empty_text() -> ExcelText {
    ExcelText::from_utf16_code_units(Vec::new())
}

fn map_array(array: &FunctionArray) -> FunctionArray {
    let cells = array
        .iter_row_major()
        .map(|cell| match cell {
            FunctionArrayCell::Text(t) => FunctionArrayCell::Text(t.clone()),
            FunctionArrayCell::Error(code) => FunctionArrayCell::Error(*code),
            FunctionArrayCell::Number(_)
            | FunctionArrayCell::Logical(_)
            | FunctionArrayCell::EmptyCell => FunctionArrayCell::Text(empty_text()),
        })
        .collect();
    FunctionArray::new(array.shape(), cells).expect("shape preserved")
}

fn map_prepared(prepared: PreparedValue) -> FunctionValue {
    match prepared {
        PreparedValue::Eval(FunctionValue::Text(t)) => FunctionValue::Text(t),
        PreparedValue::Eval(FunctionValue::Error(code)) => FunctionValue::Error(code),
        PreparedValue::Eval(FunctionValue::Array(array)) => FunctionValue::Array(map_array(&array)),
        PreparedValue::Eval(FunctionValue::Reference(_)) => {
            FunctionValue::Error(WorksheetErrorCode::Value)
        }
        PreparedValue::Eval(FunctionValue::Number(_))
        | PreparedValue::Eval(FunctionValue::Logical(_))
        | PreparedValue::MissingArg
        | PreparedValue::EmptyCell => FunctionValue::Text(empty_text()),
        _ => FunctionValue::Text(empty_text()),
    }
}

pub fn eval_t_surface(
    args: &[crate::value::FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<FunctionValue, TEvalError> {
    if !T_META.arity.accepts(args.len()) {
        return Err(TEvalError::ArityMismatch {
            expected: T_META.arity.min,
            actual: args.len(),
        });
    }
    let prepared = prepare_arg_values_only(&args[0], resolver).map_err(TEvalError::Coercion)?;
    Ok(map_prepared(prepared))
}

pub fn map_t_error_to_ws(e: &TEvalError) -> WorksheetErrorCode {
    match e {
        TEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        TEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        TEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::ReferenceSystemCapabilities;
    use crate::value::FunctionArg;

    struct NoResolver;

    impl ReferenceSystemProvider for NoResolver {
        fn capabilities(&self) -> ReferenceSystemCapabilities {
            ReferenceSystemCapabilities::permissive_local()
        }

        fn dereference(
            &self,
            request: &crate::resolver::ReferenceDereferenceRequest,
        ) -> Result<FunctionValue, crate::resolver::ReferenceResolutionError> {
            let reference = &request.reference;
            Err(
                crate::resolver::ReferenceResolutionError::UnresolvedReference {
                    target: reference.target().to_string(),
                },
            )
        }
    }

    #[test]
    fn eval_t_maps_number_to_empty_string() {
        assert_eq!(
            eval_t_surface(
                &[FunctionArg::Eval(FunctionValue::Number(42.0))],
                &NoResolver
            ),
            Ok(FunctionValue::Text(empty_text()))
        );
    }
}

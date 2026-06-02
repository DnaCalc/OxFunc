use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{PreparedValue, prepare_arg_values_only};
use crate::resolver::ReferenceSystemProvider;
use crate::value::{FunctionArray, FunctionArrayCell, FunctionValue, WorksheetErrorCode};

pub const N_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.N",
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
pub enum NEvalError {
    ArityMismatch { expected: usize, actual: usize },
    Coercion(CoercionError),
}

fn map_array(array: &FunctionArray) -> FunctionArray {
    let cells = array
        .iter_row_major()
        .map(|cell| match cell {
            FunctionArrayCell::Number(n) => FunctionArrayCell::Number(*n),
            FunctionArrayCell::Logical(b) => FunctionArrayCell::Number(if *b { 1.0 } else { 0.0 }),
            FunctionArrayCell::Text(_) | FunctionArrayCell::EmptyCell => {
                FunctionArrayCell::Number(0.0)
            }
            FunctionArrayCell::Error(code) => FunctionArrayCell::Error(*code),
        })
        .collect();
    FunctionArray::new(array.shape(), cells).expect("shape preserved")
}

fn map_prepared(prepared: PreparedValue) -> FunctionValue {
    match prepared {
        PreparedValue::Eval(FunctionValue::Number(n)) => FunctionValue::Number(n),
        PreparedValue::Eval(FunctionValue::Logical(b)) => {
            FunctionValue::Number(if b { 1.0 } else { 0.0 })
        }
        PreparedValue::Eval(FunctionValue::Text(_)) => FunctionValue::Number(0.0),
        PreparedValue::Eval(FunctionValue::Error(code)) => FunctionValue::Error(code),
        PreparedValue::Eval(FunctionValue::Array(array)) => FunctionValue::Array(map_array(&array)),
        PreparedValue::Eval(FunctionValue::Reference(_)) => {
            FunctionValue::Error(WorksheetErrorCode::Value)
        }
        PreparedValue::MissingArg | PreparedValue::EmptyCell => FunctionValue::Number(0.0),
        _ => FunctionValue::Error(WorksheetErrorCode::Value),
    }
}

pub fn eval_n_surface(
    args: &[crate::value::FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<FunctionValue, NEvalError> {
    if !N_META.arity.accepts(args.len()) {
        return Err(NEvalError::ArityMismatch {
            expected: N_META.arity.min,
            actual: args.len(),
        });
    }
    let prepared = prepare_arg_values_only(&args[0], resolver).map_err(NEvalError::Coercion)?;
    Ok(map_prepared(prepared))
}

pub fn map_n_error_to_ws(e: &NEvalError) -> WorksheetErrorCode {
    match e {
        NEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        NEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        NEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::ReferenceSystemCapabilities;
    use crate::value::{ExcelText, FunctionArg};

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
    fn eval_n_maps_text_to_zero_and_logical_to_number() {
        assert_eq!(
            eval_n_surface(
                &[FunctionArg::Eval(FunctionValue::Text(
                    ExcelText::from_utf16_code_units("x".encode_utf16().collect(),)
                ))],
                &NoResolver,
            ),
            Ok(FunctionValue::Number(0.0))
        );
        assert_eq!(
            eval_n_surface(
                &[FunctionArg::Eval(FunctionValue::Logical(true))],
                &NoResolver
            ),
            Ok(FunctionValue::Number(1.0))
        );
    }
}

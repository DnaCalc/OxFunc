use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{PreparedArgValue, prepare_arg_values_only};
use crate::resolver::ReferenceResolver;
use crate::value::{ArrayCellValue, EvalArray, EvalValue, WorksheetErrorCode};

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

fn map_array(array: &EvalArray) -> EvalArray {
    let cells = array
        .iter_row_major()
        .map(|cell| match cell {
            ArrayCellValue::Number(n) => ArrayCellValue::Number(*n),
            ArrayCellValue::Logical(b) => ArrayCellValue::Number(if *b { 1.0 } else { 0.0 }),
            ArrayCellValue::Text(_) | ArrayCellValue::EmptyCell => ArrayCellValue::Number(0.0),
            ArrayCellValue::Error(code) => ArrayCellValue::Error(*code),
        })
        .collect();
    EvalArray::new(array.shape(), cells).expect("shape preserved")
}

fn map_prepared(prepared: PreparedArgValue) -> EvalValue {
    match prepared {
        PreparedArgValue::Eval(EvalValue::Number(n)) => EvalValue::Number(n),
        PreparedArgValue::Eval(EvalValue::Logical(b)) => {
            EvalValue::Number(if b { 1.0 } else { 0.0 })
        }
        PreparedArgValue::Eval(EvalValue::Text(_)) => EvalValue::Number(0.0),
        PreparedArgValue::Eval(EvalValue::Error(code)) => EvalValue::Error(code),
        PreparedArgValue::Eval(EvalValue::Array(array)) => EvalValue::Array(map_array(&array)),
        PreparedArgValue::Eval(EvalValue::Reference(_)) => {
            EvalValue::Error(WorksheetErrorCode::Value)
        }
        PreparedArgValue::Eval(EvalValue::Lambda(_)) => EvalValue::Error(WorksheetErrorCode::Value),
        PreparedArgValue::MissingArg | PreparedArgValue::EmptyCell => EvalValue::Number(0.0),
    }
}

pub fn eval_n_surface(
    args: &[crate::value::CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, NEvalError> {
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
    use crate::resolver::{RefResolutionError, ResolverCapabilities};
    use crate::value::{CallArgValue, ExcelText, ReferenceLike};

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
    fn eval_n_maps_text_to_zero_and_logical_to_number() {
        assert_eq!(
            eval_n_surface(
                &[CallArgValue::Eval(EvalValue::Text(
                    ExcelText::from_utf16_code_units("x".encode_utf16().collect(),)
                ))],
                &NoResolver,
            ),
            Ok(EvalValue::Number(0.0))
        );
        assert_eq!(
            eval_n_surface(&[CallArgValue::Eval(EvalValue::Logical(true))], &NoResolver),
            Ok(EvalValue::Number(1.0))
        );
    }
}

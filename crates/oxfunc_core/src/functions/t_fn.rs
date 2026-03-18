use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{PreparedArgValue, prepare_arg_values_only};
use crate::resolver::ReferenceResolver;
use crate::value::{ArrayCellValue, EvalArray, EvalValue, ExcelText, WorksheetErrorCode};

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

fn map_array(array: &EvalArray) -> EvalArray {
    let cells = array
        .iter_row_major()
        .map(|cell| match cell {
            ArrayCellValue::Text(t) => ArrayCellValue::Text(t.clone()),
            ArrayCellValue::Error(code) => ArrayCellValue::Error(*code),
            ArrayCellValue::Number(_) | ArrayCellValue::Logical(_) | ArrayCellValue::EmptyCell => {
                ArrayCellValue::Text(empty_text())
            }
        })
        .collect();
    EvalArray::new(array.shape(), cells).expect("shape preserved")
}

fn map_prepared(prepared: PreparedArgValue) -> EvalValue {
    match prepared {
        PreparedArgValue::Eval(EvalValue::Text(t)) => EvalValue::Text(t),
        PreparedArgValue::Eval(EvalValue::Error(code)) => EvalValue::Error(code),
        PreparedArgValue::Eval(EvalValue::Array(array)) => EvalValue::Array(map_array(&array)),
        PreparedArgValue::Eval(EvalValue::Reference(_)) => {
            EvalValue::Error(WorksheetErrorCode::Value)
        }
        PreparedArgValue::Eval(EvalValue::Number(_))
        | PreparedArgValue::Eval(EvalValue::Logical(_))
        | PreparedArgValue::Eval(EvalValue::Lambda(_))
        | PreparedArgValue::MissingArg
        | PreparedArgValue::EmptyCell => EvalValue::Text(empty_text()),
    }
}

pub fn eval_t_surface(
    args: &[crate::value::CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, TEvalError> {
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
    use crate::resolver::{RefResolutionError, ResolverCapabilities};
    use crate::value::{CallArgValue, ReferenceLike};

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
    fn eval_t_maps_number_to_empty_string() {
        assert_eq!(
            eval_t_surface(&[CallArgValue::Eval(EvalValue::Number(42.0))], &NoResolver),
            Ok(EvalValue::Text(empty_text()))
        );
    }
}

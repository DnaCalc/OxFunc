use crate::coercion::CoercionError;
use crate::function::{
    Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile, FunctionMeta,
    HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::prepare_arg_values_only;
use crate::resolver::ReferenceSystemProvider;
use crate::value::{CalcArray, ExcelText, WorksheetErrorCode};
use crate::value::{CalcValue, CoreValue};

pub const T_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.T",
    arity: Arity::exact(1),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: FunctionMeta::DEFAULT_ARG_PREPARATION_PROFILE,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
    real_result_policy: FunctionMeta::DEFAULT_REAL_RESULT_POLICY,
};

#[derive(Debug, Clone, PartialEq)]
pub enum TEvalError {
    ArityMismatch { expected: usize, actual: usize },
    Coercion(CoercionError),
}

fn empty_text() -> ExcelText {
    ExcelText::from_utf16_code_units(Vec::new())
}

fn map_array(array: &CalcArray) -> CalcArray {
    let cells = array
        .iter_row_major()
        .map(|cell| match cell.core() {
            CoreValue::Text(t) => CalcValue::text(t.clone()),
            CoreValue::Error(code) => CalcValue::error(*code),
            CoreValue::Number(_)
            | CoreValue::Logical(_)
            | CoreValue::Empty
            | CoreValue::Missing => CalcValue::text(empty_text()),
            CoreValue::Array(_) | CoreValue::Reference(_) => {
                CalcValue::error(WorksheetErrorCode::Value)
            }
        })
        .collect();
    CalcArray::new(array.shape(), cells).expect("shape preserved")
}

fn map_prepared(prepared: CalcValue) -> CalcValue {
    match prepared.core() {
        CoreValue::Text(t) => CalcValue::text(t.clone()),
        CoreValue::Error(code) => CalcValue::error(*code),
        CoreValue::Array(array) => CalcValue::array(map_array(array)),
        CoreValue::Reference(_) => CalcValue::error(WorksheetErrorCode::Value),
        CoreValue::Number(_) | CoreValue::Logical(_) | CoreValue::Missing | CoreValue::Empty => {
            CalcValue::text(empty_text())
        }
    }
}

pub fn eval_t_surface(
    args: &[crate::value::CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, TEvalError> {
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
    use crate::value::CalcValue;

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
    fn eval_t_maps_number_to_empty_string() {
        assert_eq!(
            eval_t_surface(&[(CalcValue::number(42.0))], &NoResolver),
            Ok(CalcValue::text(empty_text()))
        );
    }
}

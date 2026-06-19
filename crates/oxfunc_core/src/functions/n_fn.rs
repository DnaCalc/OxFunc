use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::prepare_arg_values_only;
use crate::resolver::ReferenceSystemProvider;
use crate::value::{CalcArray, WorksheetErrorCode};
use crate::value::{CalcValue, CoreValue};

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
    real_result_policy: FunctionMeta::DEFAULT_REAL_RESULT_POLICY,
};

#[derive(Debug, Clone, PartialEq)]
pub enum NEvalError {
    ArityMismatch { expected: usize, actual: usize },
    Coercion(CoercionError),
}

fn map_array(array: &CalcArray) -> CalcArray {
    let cells = array
        .iter_row_major()
        .map(|cell| match cell.core() {
            CoreValue::Number(n) => CalcValue::number(*n),
            CoreValue::Logical(b) => CalcValue::number(if *b { 1.0 } else { 0.0 }),
            CoreValue::Text(_) | CoreValue::Empty | CoreValue::Missing => CalcValue::number(0.0),
            CoreValue::Error(code) => CalcValue::error(*code),
            CoreValue::Array(_) | CoreValue::Reference(_) => {
                CalcValue::error(WorksheetErrorCode::Value)
            }
        })
        .collect();
    CalcArray::new(array.shape(), cells).expect("shape preserved")
}

fn map_prepared(prepared: CalcValue) -> CalcValue {
    match prepared.core() {
        CoreValue::Number(n) => CalcValue::number(*n),
        CoreValue::Logical(b) => CalcValue::number(if *b { 1.0 } else { 0.0 }),
        CoreValue::Text(_) => CalcValue::number(0.0),
        CoreValue::Error(code) => CalcValue::error(*code),
        CoreValue::Array(array) => CalcValue::array(map_array(array)),
        CoreValue::Reference(_) => CalcValue::error(WorksheetErrorCode::Value),
        CoreValue::Missing | CoreValue::Empty => CalcValue::number(0.0),
    }
}

pub fn eval_n_surface(
    args: &[crate::value::CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, NEvalError> {
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
    fn eval_n_maps_text_to_zero_and_logical_to_number() {
        assert_eq!(
            eval_n_surface(
                &[(CalcValue::text(ExcelText::from_utf16_code_units(
                    "x".encode_utf16().collect(),
                )))],
                &NoResolver,
            ),
            Ok(CalcValue::number(0.0))
        );
        assert_eq!(
            eval_n_surface(&[(CalcValue::logical(true))], &NoResolver),
            Ok(CalcValue::number(1.0))
        );
    }
}

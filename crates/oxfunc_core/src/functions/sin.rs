use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    apply_unary_numeric_scalar_prepared, expand_arg_values_only, prepare_arg_values_only,
};
use crate::resolver::ReferenceSystemProvider;
use crate::value::{CalcArray, CalcValue, CoreValue, WorksheetErrorCode};

pub const SIN_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.SIN",
    arity: Arity::exact(1),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
    coercion_lift_profile: CoercionLiftProfile::UnaryNumericScalarOrArrayElementwise,
    kernel_signature_class: KernelSignatureClass::NumToNum,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
};

#[derive(Debug, Clone, PartialEq)]
pub enum SinEvalError {
    ArityMismatch { expected: usize, actual: usize },
    Coercion(CoercionError),
}

pub fn sin_kernel(n: f64) -> f64 {
    n.sin()
}

pub fn eval_sin_surface(
    args: &[crate::value::CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, SinEvalError> {
    if !SIN_META.arity.accepts(args.len()) {
        return Err(SinEvalError::ArityMismatch {
            expected: SIN_META.arity.min,
            actual: args.len(),
        });
    }

    let prepared = prepare_arg_values_only(&args[0], resolver).map_err(SinEvalError::Coercion)?;
    match prepared.core() {
        CoreValue::Array(array) => {
            let mapped = expand_arg_values_only(&args[0], resolver)
                .map_err(SinEvalError::Coercion)?
                .into_iter()
                .map(
                    |item| match apply_unary_numeric_scalar_prepared(&item, sin_kernel) {
                        Ok(n) => CalcValue::number(n),
                        Err(CoercionError::WorksheetError(code)) => CalcValue::error(code),
                        Err(_) => CalcValue::error(WorksheetErrorCode::Value),
                    },
                )
                .collect::<Vec<_>>();
            Ok(CalcValue::array(
                CalcArray::new(array.shape(), mapped).expect("shape preserved"),
            ))
        }
        _ => Ok(CalcValue::number(
            apply_unary_numeric_scalar_prepared(&prepared, sin_kernel)
                .map_err(SinEvalError::Coercion)?,
        )),
    }
}

pub fn map_sin_error_to_ws(e: &SinEvalError) -> WorksheetErrorCode {
    match e {
        SinEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        SinEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        SinEvalError::Coercion(_) => WorksheetErrorCode::Value,
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
    fn eval_sin_accepts_numeric_text() {
        let got = eval_sin_surface(
            &[(CalcValue::text(ExcelText::from_utf16_code_units(
                "1".encode_utf16().collect(),
            )))],
            &NoResolver,
        )
        .unwrap();
        match got.core() {
            CoreValue::Number(n) => assert!((*n - 1f64.sin()).abs() < 1e-12),
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn eval_sin_array_lifts_elementwise() {
        let got = eval_sin_surface(
            &[(CalcValue::array(
                CalcArray::from_rows(vec![vec![
                    CalcValue::number(1.0),
                    CalcValue::text(ExcelText::from_utf16_code_units(
                        "asd".encode_utf16().collect(),
                    )),
                ]])
                .unwrap(),
            ))],
            &NoResolver,
        )
        .unwrap();
        assert_eq!(
            got,
            CalcValue::array(
                CalcArray::from_rows(vec![vec![
                    CalcValue::number(1f64.sin()),
                    CalcValue::error(WorksheetErrorCode::Value),
                ]])
                .unwrap()
            )
        );
    }
}

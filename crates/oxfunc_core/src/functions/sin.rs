use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    PreparedValue, apply_unary_numeric_scalar_prepared, expand_arg_values_only,
    prepare_arg_values_only,
};
use crate::resolver::ReferenceSystemProvider;
use crate::value::{FunctionArray, FunctionArrayCell, FunctionValue, WorksheetErrorCode};

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
    args: &[crate::value::FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<FunctionValue, SinEvalError> {
    if !SIN_META.arity.accepts(args.len()) {
        return Err(SinEvalError::ArityMismatch {
            expected: SIN_META.arity.min,
            actual: args.len(),
        });
    }

    let prepared = prepare_arg_values_only(&args[0], resolver).map_err(SinEvalError::Coercion)?;
    match prepared {
        PreparedValue::Eval(FunctionValue::Array(array)) => {
            let mapped = expand_arg_values_only(&args[0], resolver)
                .map_err(SinEvalError::Coercion)?
                .into_iter()
                .map(
                    |item| match apply_unary_numeric_scalar_prepared(&item, sin_kernel) {
                        Ok(n) => FunctionArrayCell::Number(n),
                        Err(CoercionError::WorksheetError(code)) => FunctionArrayCell::Error(code),
                        Err(_) => FunctionArrayCell::Error(WorksheetErrorCode::Value),
                    },
                )
                .collect::<Vec<_>>();
            Ok(FunctionValue::Array(
                FunctionArray::new(array.shape(), mapped).expect("shape preserved"),
            ))
        }
        other => Ok(FunctionValue::Number(
            apply_unary_numeric_scalar_prepared(&other, sin_kernel)
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
    fn eval_sin_accepts_numeric_text() {
        let got = eval_sin_surface(
            &[FunctionArg::Eval(FunctionValue::Text(
                ExcelText::from_utf16_code_units("1".encode_utf16().collect()),
            ))],
            &NoResolver,
        )
        .unwrap();
        match got {
            FunctionValue::Number(n) => assert!((n - 1f64.sin()).abs() < 1e-12),
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn eval_sin_array_lifts_elementwise() {
        let got = eval_sin_surface(
            &[FunctionArg::Eval(FunctionValue::Array(
                FunctionArray::from_rows(vec![vec![
                    FunctionArrayCell::Number(1.0),
                    FunctionArrayCell::Text(ExcelText::from_utf16_code_units(
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
            FunctionValue::Array(
                FunctionArray::from_rows(vec![vec![
                    FunctionArrayCell::Number(1f64.sin()),
                    FunctionArrayCell::Error(WorksheetErrorCode::Value),
                ]])
                .unwrap()
            )
        );
    }
}

use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    PreparedArgValue, apply_unary_numeric_scalar_prepared, expand_arg_values_only,
    prepare_arg_values_only,
};
use crate::resolver::ReferenceResolver;
use crate::value::{ArrayCellValue, EvalArray, EvalValue, WorksheetErrorCode};

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
    args: &[crate::value::CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, SinEvalError> {
    if !SIN_META.arity.accepts(args.len()) {
        return Err(SinEvalError::ArityMismatch {
            expected: SIN_META.arity.min,
            actual: args.len(),
        });
    }

    let prepared = prepare_arg_values_only(&args[0], resolver).map_err(SinEvalError::Coercion)?;
    match prepared {
        PreparedArgValue::Eval(EvalValue::Array(array)) => {
            let mapped = expand_arg_values_only(&args[0], resolver)
                .map_err(SinEvalError::Coercion)?
                .into_iter()
                .map(
                    |item| match apply_unary_numeric_scalar_prepared(&item, sin_kernel) {
                        Ok(n) => ArrayCellValue::Number(n),
                        Err(CoercionError::WorksheetError(code)) => ArrayCellValue::Error(code),
                        Err(_) => ArrayCellValue::Error(WorksheetErrorCode::Value),
                    },
                )
                .collect::<Vec<_>>();
            Ok(EvalValue::Array(
                EvalArray::new(array.shape(), mapped).expect("shape preserved"),
            ))
        }
        other => Ok(EvalValue::Number(
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
    fn eval_sin_accepts_numeric_text() {
        let got = eval_sin_surface(
            &[CallArgValue::Eval(EvalValue::Text(
                ExcelText::from_utf16_code_units("1".encode_utf16().collect()),
            ))],
            &NoResolver,
        )
        .unwrap();
        match got {
            EvalValue::Number(n) => assert!((n - 1f64.sin()).abs() < 1e-12),
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn eval_sin_array_lifts_elementwise() {
        let got = eval_sin_surface(
            &[CallArgValue::Eval(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Number(1.0),
                    ArrayCellValue::Text(ExcelText::from_utf16_code_units(
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
            EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Number(1f64.sin()),
                    ArrayCellValue::Error(WorksheetErrorCode::Value),
                ]])
                .unwrap()
            )
        );
    }
}

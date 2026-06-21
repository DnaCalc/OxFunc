use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, ExcelRealPolicy,
    FecDependencyProfile, FunctionMeta, HostInteractionClass, KernelSignatureClass,
    ThreadSafetyClass, VolatilityClass,
};
use crate::functions::unary_numeric::{
    UnaryNumericExecSpec, UnaryNumericSurfaceError, eval_unary_numeric_via_executor,
    map_unary_numeric_error_to_ws,
};
use crate::resolver::ReferenceSystemProvider;
use crate::value::CalcValue;
use crate::value::WorksheetErrorCode;

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
    // BUG-FUNC-027 CLASS-B2: circular trig is `#NUM!` once `|x| >= 2^27`.
    real_result_policy: ExcelRealPolicy::CIRCULAR_TRIG,
};

pub fn sin_kernel(n: f64) -> f64 {
    n.sin()
}

pub fn eval_sin_surface(
    args: &[crate::value::CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, UnaryNumericSurfaceError> {
    eval_unary_numeric_via_executor(
        args,
        resolver,
        UnaryNumericExecSpec::raw(sin_kernel, SIN_META.real_result_policy),
    )
}

pub fn map_sin_error_to_ws(e: &UnaryNumericSurfaceError) -> WorksheetErrorCode {
    map_unary_numeric_error_to_ws(e)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::ReferenceSystemCapabilities;
    use crate::value::{CalcArray, CoreValue, ExcelText};

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
    fn sin_meta_function_id_is_stable() {
        assert_eq!(SIN_META.function_id, "FUNC.SIN");
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

    // BUG-FUNC-027 CLASS-B2: |x| >= 2^27 -> #NUM! (live Excel SIN(2^27)=#NUM!).
    #[test]
    fn eval_sin_large_argument_is_num() {
        let got = eval_sin_surface(&[CalcValue::number(134_217_728.0)], &NoResolver);
        assert_eq!(
            got,
            Err(UnaryNumericSurfaceError::Domain(WorksheetErrorCode::Num))
        );
        assert!(eval_sin_surface(&[CalcValue::number(134_217_727.0)], &NoResolver).is_ok());
    }
}

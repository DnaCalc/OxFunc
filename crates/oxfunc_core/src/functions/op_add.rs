use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::binary_numeric::{
    BinaryNumericSurfaceError, eval_binary_numeric_prepared, eval_binary_numeric_surface,
    map_binary_numeric_error_to_ws,
};
use crate::functions::excel_numeric::excel_underflow_to_zero;
use crate::resolver::ReferenceSystemProvider;
use crate::value::{FunctionArg, FunctionValue, WorksheetErrorCode};

pub const OP_ADD_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.OP_ADD",
    arity: Arity::exact(2),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::NumsToNum,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
};

pub type OpAddEvalError = BinaryNumericSurfaceError;

pub fn op_add_kernel(lhs: f64, rhs: f64) -> f64 {
    excel_underflow_to_zero(lhs + rhs)
}

pub fn eval_op_add_adapter_prepared(
    args: &[crate::functions::adapters::PreparedValue],
) -> Result<FunctionValue, OpAddEvalError> {
    eval_binary_numeric_prepared(args, |lhs, rhs| Ok(op_add_kernel(lhs, rhs)))
}

pub fn eval_op_add_surface(
    args: &[FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<FunctionValue, OpAddEvalError> {
    eval_binary_numeric_surface(args, resolver, |lhs, rhs| Ok(op_add_kernel(lhs, rhs)))
}

pub fn map_op_add_error_to_ws(e: &OpAddEvalError) -> WorksheetErrorCode {
    map_binary_numeric_error_to_ws(e)
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
    fn eval_op_add_two_numbers() {
        let args = [
            FunctionArg::Eval(FunctionValue::Number(2.0)),
            FunctionArg::Eval(FunctionValue::Number(3.0)),
        ];
        let got = eval_op_add_surface(&args, &NoResolver);
        assert_eq!(got, Ok(FunctionValue::Number(5.0)));
    }

    #[test]
    fn eval_op_add_flushes_excel_denormalized_results() {
        let args = [
            FunctionArg::Eval(FunctionValue::Number(1.0e-308)),
            FunctionArg::Eval(FunctionValue::Number(1.0e-308)),
        ];
        let got = eval_op_add_surface(&args, &NoResolver);
        assert_eq!(got, Ok(FunctionValue::Number(0.0)));
    }

    #[test]
    fn eval_op_add_numeric_text_and_logical() {
        let args = [
            FunctionArg::Eval(FunctionValue::Text(ExcelText::from_utf16_code_units(
                "2".encode_utf16().collect(),
            ))),
            FunctionArg::Eval(FunctionValue::Logical(true)),
        ];
        let got = eval_op_add_surface(&args, &NoResolver);
        assert_eq!(got, Ok(FunctionValue::Number(3.0)));
    }

    #[test]
    fn eval_op_add_non_numeric_text_fails() {
        let args = [
            FunctionArg::Eval(FunctionValue::Text(ExcelText::from_utf16_code_units(
                "bad".encode_utf16().collect(),
            ))),
            FunctionArg::Eval(FunctionValue::Number(1.0)),
        ];
        let got = eval_op_add_surface(&args, &NoResolver);
        assert!(matches!(got, Err(OpAddEvalError::Coercion(_))));
    }

    #[test]
    fn eval_op_add_lifts_array_involved_calls() {
        let scalar_array = eval_op_add_surface(
            &[
                FunctionArg::Eval(FunctionValue::Number(10.0)),
                FunctionArg::Eval(FunctionValue::Array(
                    crate::value::FunctionArray::from_rows(vec![vec![
                        crate::value::FunctionArrayCell::Number(1.0),
                        crate::value::FunctionArrayCell::Text(ExcelText::from_utf16_code_units(
                            "2".encode_utf16().collect(),
                        )),
                    ]])
                    .unwrap(),
                )),
            ],
            &NoResolver,
        )
        .unwrap();
        assert_eq!(
            scalar_array,
            FunctionValue::Array(
                crate::value::FunctionArray::from_rows(vec![vec![
                    crate::value::FunctionArrayCell::Number(11.0),
                    crate::value::FunctionArrayCell::Number(12.0),
                ]])
                .unwrap()
            )
        );

        let array_array = eval_op_add_surface(
            &[
                FunctionArg::Eval(FunctionValue::Array(
                    crate::value::FunctionArray::from_rows(vec![
                        vec![
                            crate::value::FunctionArrayCell::Number(1.0),
                            crate::value::FunctionArrayCell::Number(2.0),
                        ],
                        vec![
                            crate::value::FunctionArrayCell::Number(3.0),
                            crate::value::FunctionArrayCell::Number(4.0),
                        ],
                    ])
                    .unwrap(),
                )),
                FunctionArg::Eval(FunctionValue::Array(
                    crate::value::FunctionArray::from_rows(vec![
                        vec![
                            crate::value::FunctionArrayCell::Number(10.0),
                            crate::value::FunctionArrayCell::Number(20.0),
                        ],
                        vec![
                            crate::value::FunctionArrayCell::Number(30.0),
                            crate::value::FunctionArrayCell::Number(40.0),
                        ],
                    ])
                    .unwrap(),
                )),
            ],
            &NoResolver,
        )
        .unwrap();
        assert_eq!(
            array_array,
            FunctionValue::Array(
                crate::value::FunctionArray::from_rows(vec![
                    vec![
                        crate::value::FunctionArrayCell::Number(11.0),
                        crate::value::FunctionArrayCell::Number(22.0),
                    ],
                    vec![
                        crate::value::FunctionArrayCell::Number(33.0),
                        crate::value::FunctionArrayCell::Number(44.0),
                    ],
                ])
                .unwrap()
            )
        );
    }
}

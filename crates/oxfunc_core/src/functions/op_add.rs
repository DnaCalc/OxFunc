use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::binary_numeric::{
    BinaryNumericSurfaceError, eval_binary_numeric_prepared, eval_binary_numeric_surface,
    map_binary_numeric_error_to_ws,
};
use crate::functions::excel_numeric::excel_underflow_to_zero;
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

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
    args: &[crate::functions::adapters::PreparedArgValue],
) -> Result<EvalValue, OpAddEvalError> {
    eval_binary_numeric_prepared(args, |lhs, rhs| Ok(op_add_kernel(lhs, rhs)))
}

pub fn eval_op_add_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, OpAddEvalError> {
    eval_binary_numeric_surface(args, resolver, |lhs, rhs| Ok(op_add_kernel(lhs, rhs)))
}

pub fn map_op_add_error_to_ws(e: &OpAddEvalError) -> WorksheetErrorCode {
    map_binary_numeric_error_to_ws(e)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::{RefResolutionError, ResolverCapabilities};
    use crate::value::{ExcelText, ReferenceLike};

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
    fn eval_op_add_two_numbers() {
        let args = [
            CallArgValue::Eval(EvalValue::Number(2.0)),
            CallArgValue::Eval(EvalValue::Number(3.0)),
        ];
        let got = eval_op_add_surface(&args, &NoResolver);
        assert_eq!(got, Ok(EvalValue::Number(5.0)));
    }

    #[test]
    fn eval_op_add_flushes_excel_denormalized_results() {
        let args = [
            CallArgValue::Eval(EvalValue::Number(1.0e-308)),
            CallArgValue::Eval(EvalValue::Number(1.0e-308)),
        ];
        let got = eval_op_add_surface(&args, &NoResolver);
        assert_eq!(got, Ok(EvalValue::Number(0.0)));
    }

    #[test]
    fn eval_op_add_numeric_text_and_logical() {
        let args = [
            CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
                "2".encode_utf16().collect(),
            ))),
            CallArgValue::Eval(EvalValue::Logical(true)),
        ];
        let got = eval_op_add_surface(&args, &NoResolver);
        assert_eq!(got, Ok(EvalValue::Number(3.0)));
    }

    #[test]
    fn eval_op_add_non_numeric_text_fails() {
        let args = [
            CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
                "bad".encode_utf16().collect(),
            ))),
            CallArgValue::Eval(EvalValue::Number(1.0)),
        ];
        let got = eval_op_add_surface(&args, &NoResolver);
        assert!(matches!(got, Err(OpAddEvalError::Coercion(_))));
    }

    #[test]
    fn eval_op_add_lifts_array_involved_calls() {
        let scalar_array = eval_op_add_surface(
            &[
                CallArgValue::Eval(EvalValue::Number(10.0)),
                CallArgValue::Eval(EvalValue::Array(
                    crate::value::EvalArray::from_rows(vec![vec![
                        crate::value::ArrayCellValue::Number(1.0),
                        crate::value::ArrayCellValue::Text(ExcelText::from_utf16_code_units(
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
            EvalValue::Array(
                crate::value::EvalArray::from_rows(vec![vec![
                    crate::value::ArrayCellValue::Number(11.0),
                    crate::value::ArrayCellValue::Number(12.0),
                ]])
                .unwrap()
            )
        );

        let array_array = eval_op_add_surface(
            &[
                CallArgValue::Eval(EvalValue::Array(
                    crate::value::EvalArray::from_rows(vec![
                        vec![
                            crate::value::ArrayCellValue::Number(1.0),
                            crate::value::ArrayCellValue::Number(2.0),
                        ],
                        vec![
                            crate::value::ArrayCellValue::Number(3.0),
                            crate::value::ArrayCellValue::Number(4.0),
                        ],
                    ])
                    .unwrap(),
                )),
                CallArgValue::Eval(EvalValue::Array(
                    crate::value::EvalArray::from_rows(vec![
                        vec![
                            crate::value::ArrayCellValue::Number(10.0),
                            crate::value::ArrayCellValue::Number(20.0),
                        ],
                        vec![
                            crate::value::ArrayCellValue::Number(30.0),
                            crate::value::ArrayCellValue::Number(40.0),
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
            EvalValue::Array(
                crate::value::EvalArray::from_rows(vec![
                    vec![
                        crate::value::ArrayCellValue::Number(11.0),
                        crate::value::ArrayCellValue::Number(22.0),
                    ],
                    vec![
                        crate::value::ArrayCellValue::Number(33.0),
                        crate::value::ArrayCellValue::Number(44.0),
                    ],
                ])
                .unwrap()
            )
        );
    }
}

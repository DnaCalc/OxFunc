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
use crate::value::CalcValue;
use crate::value::WorksheetErrorCode;

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
    args: &[crate::functions::adapters::CalcValue],
) -> Result<CalcValue, OpAddEvalError> {
    eval_binary_numeric_prepared(args, |lhs, rhs| Ok(op_add_kernel(lhs, rhs)))
}

pub fn eval_op_add_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, OpAddEvalError> {
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
    fn eval_op_add_two_numbers() {
        let args = [(CalcValue::number(2.0)), (CalcValue::number(3.0))];
        let got = eval_op_add_surface(&args, &NoResolver);
        assert_eq!(got, Ok(CalcValue::number(5.0)));
    }

    #[test]
    fn eval_op_add_flushes_excel_denormalized_results() {
        let args = [(CalcValue::number(1.0e-308)), (CalcValue::number(1.0e-308))];
        let got = eval_op_add_surface(&args, &NoResolver);
        assert_eq!(got, Ok(CalcValue::number(0.0)));
    }

    #[test]
    fn eval_op_add_numeric_text_and_logical() {
        let args = [
            (CalcValue::text(ExcelText::from_utf16_code_units(
                "2".encode_utf16().collect(),
            ))),
            (CalcValue::logical(true)),
        ];
        let got = eval_op_add_surface(&args, &NoResolver);
        assert_eq!(got, Ok(CalcValue::number(3.0)));
    }

    #[test]
    fn eval_op_add_non_numeric_text_fails() {
        let args = [
            (CalcValue::text(ExcelText::from_utf16_code_units(
                "bad".encode_utf16().collect(),
            ))),
            (CalcValue::number(1.0)),
        ];
        let got = eval_op_add_surface(&args, &NoResolver);
        assert!(matches!(got, Err(OpAddEvalError::Coercion(_))));
    }

    #[test]
    fn eval_op_add_lifts_array_involved_calls() {
        let scalar_array = eval_op_add_surface(
            &[
                (CalcValue::number(10.0)),
                (CalcValue::array(
                    crate::value::CalcArray::from_rows(vec![vec![
                        crate::value::CalcValue::number(1.0),
                        crate::value::CalcValue::text(ExcelText::from_utf16_code_units(
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
            CalcValue::array(
                crate::value::CalcArray::from_rows(vec![vec![
                    crate::value::CalcValue::number(11.0),
                    crate::value::CalcValue::number(12.0),
                ]])
                .unwrap()
            )
        );

        let array_array = eval_op_add_surface(
            &[
                (CalcValue::array(
                    crate::value::CalcArray::from_rows(vec![
                        vec![
                            crate::value::CalcValue::number(1.0),
                            crate::value::CalcValue::number(2.0),
                        ],
                        vec![
                            crate::value::CalcValue::number(3.0),
                            crate::value::CalcValue::number(4.0),
                        ],
                    ])
                    .unwrap(),
                )),
                (CalcValue::array(
                    crate::value::CalcArray::from_rows(vec![
                        vec![
                            crate::value::CalcValue::number(10.0),
                            crate::value::CalcValue::number(20.0),
                        ],
                        vec![
                            crate::value::CalcValue::number(30.0),
                            crate::value::CalcValue::number(40.0),
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
            CalcValue::array(
                crate::value::CalcArray::from_rows(vec![
                    vec![
                        crate::value::CalcValue::number(11.0),
                        crate::value::CalcValue::number(22.0),
                    ],
                    vec![
                        crate::value::CalcValue::number(33.0),
                        crate::value::CalcValue::number(44.0),
                    ],
                ])
                .unwrap()
            )
        );
    }
}

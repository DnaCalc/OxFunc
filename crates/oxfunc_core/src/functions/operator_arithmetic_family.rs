use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{PreparedValue, expand_arg_values_only, prepare_arg_values_only};
use crate::functions::binary_numeric::{
    BinaryNumericSurfaceError, eval_binary_numeric_surface, map_binary_numeric_error_to_ws,
};
use crate::functions::excel_numeric::excel_underflow_to_zero;
use crate::functions::power_fn::power_kernel;
use crate::functions::unary_numeric::{
    UnaryNumericSurfaceError, eval_unary_numeric_surface, map_unary_numeric_error_to_ws,
};
use crate::resolver::ReferenceSystemProvider;
use crate::value::{
    FunctionArg, FunctionArray, FunctionArrayCell, FunctionValue, WorksheetErrorCode,
};

const OP_UNARY_NUMERIC_BASE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.OP_UNARY_NUMERIC_BASE",
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

const OP_BINARY_NUMERIC_BASE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.OP_BINARY_NUMERIC_BASE",
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

pub const OP_UNARY_PLUS_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.OP_UNARY_PLUS",
    ..OP_UNARY_NUMERIC_BASE_META
};

pub const OP_NEGATE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.OP_NEGATE",
    ..OP_UNARY_NUMERIC_BASE_META
};

pub const OP_PERCENT_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.OP_PERCENT",
    ..OP_UNARY_NUMERIC_BASE_META
};

pub const OP_SUBTRACT_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.OP_SUBTRACT",
    ..OP_BINARY_NUMERIC_BASE_META
};

pub const OP_MULTIPLY_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.OP_MULTIPLY",
    ..OP_BINARY_NUMERIC_BASE_META
};

pub const OP_DIVIDE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.OP_DIVIDE",
    ..OP_BINARY_NUMERIC_BASE_META
};

pub const OP_POWER_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.OP_POWER",
    ..OP_BINARY_NUMERIC_BASE_META
};

pub fn op_unary_plus_kernel(value: f64) -> Result<f64, WorksheetErrorCode> {
    Ok(excel_underflow_to_zero(value))
}

pub fn op_negate_kernel(value: f64) -> Result<f64, WorksheetErrorCode> {
    Ok(excel_underflow_to_zero(-value))
}

pub fn op_percent_kernel(value: f64) -> Result<f64, WorksheetErrorCode> {
    Ok(excel_underflow_to_zero(value / 100.0))
}

pub fn op_subtract_kernel(lhs: f64, rhs: f64) -> Result<f64, WorksheetErrorCode> {
    Ok(excel_underflow_to_zero(lhs - rhs))
}

pub fn op_multiply_kernel(lhs: f64, rhs: f64) -> Result<f64, WorksheetErrorCode> {
    Ok(excel_underflow_to_zero(lhs * rhs))
}

pub fn op_divide_kernel(lhs: f64, rhs: f64) -> Result<f64, WorksheetErrorCode> {
    if rhs == 0.0 {
        Err(WorksheetErrorCode::Div0)
    } else {
        Ok(excel_underflow_to_zero(lhs / rhs))
    }
}

/// Excel's unary plus (`+x`) is a type-preserving identity, NOT a numeric
/// coercion (unlike unary minus). `+"2"` stays text `"2"`, `+TRUE` stays
/// logical `TRUE`, arrays are mapped elementwise unchanged. The only
/// adjustments are: a blank/empty operand becomes `0`, and numeric values
/// get Excel's underflow-to-zero normalization. Empirically confirmed against
/// Excel `16.0` across number/text/logical/error/array/blank operands
/// (BUG-FUNC-029; run `unary-plus-operand-001`).
pub fn eval_op_unary_plus_surface(
    args: &[FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<FunctionValue, UnaryNumericSurfaceError> {
    if args.len() != 1 {
        return Err(UnaryNumericSurfaceError::ArityMismatch {
            expected: 1,
            actual: args.len(),
        });
    }
    let prepared =
        prepare_arg_values_only(&args[0], resolver).map_err(UnaryNumericSurfaceError::Coercion)?;
    match prepared {
        PreparedValue::Eval(FunctionValue::Array(array)) => {
            let cells = expand_arg_values_only(&args[0], resolver)
                .map_err(UnaryNumericSurfaceError::Coercion)?
                .into_iter()
                .map(unary_plus_identity_cell)
                .collect::<Vec<_>>();
            Ok(FunctionValue::Array(
                FunctionArray::new(array.shape(), cells).expect("shape preserved"),
            ))
        }
        other => unary_plus_identity_scalar(other),
    }
}

/// Identity map for a scalar unary-plus operand (blank -> 0, number -> underflow-normalized).
fn unary_plus_identity_scalar(
    prepared: PreparedValue,
) -> Result<FunctionValue, UnaryNumericSurfaceError> {
    match prepared {
        PreparedValue::Eval(FunctionValue::Number(n)) => {
            Ok(FunctionValue::Number(excel_underflow_to_zero(n)))
        }
        PreparedValue::Eval(value @ (FunctionValue::Text(_) | FunctionValue::Logical(_))) => {
            Ok(value)
        }
        PreparedValue::Eval(FunctionValue::Error(code)) => Ok(FunctionValue::Error(code)),
        PreparedValue::EmptyCell => Ok(FunctionValue::Number(0.0)),
        // Reference / Lambda / Array(unreachable here) / MissingArg are not
        // valid scalar unary-plus operands -> #VALUE!.
        _ => Err(UnaryNumericSurfaceError::Domain(WorksheetErrorCode::Value)),
    }
}

/// Identity map for one array cell under unary plus.
fn unary_plus_identity_cell(item: PreparedValue) -> FunctionArrayCell {
    match item {
        PreparedValue::Eval(FunctionValue::Number(n)) => {
            FunctionArrayCell::Number(excel_underflow_to_zero(n))
        }
        PreparedValue::Eval(FunctionValue::Text(t)) => FunctionArrayCell::Text(t),
        PreparedValue::Eval(FunctionValue::Logical(b)) => FunctionArrayCell::Logical(b),
        PreparedValue::Eval(FunctionValue::Error(code)) => FunctionArrayCell::Error(code),
        PreparedValue::EmptyCell => FunctionArrayCell::Number(0.0),
        _ => FunctionArrayCell::Error(WorksheetErrorCode::Value),
    }
}

pub fn eval_op_negate_surface(
    args: &[FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<FunctionValue, UnaryNumericSurfaceError> {
    eval_unary_numeric_surface(args, resolver, op_negate_kernel)
}

pub fn eval_op_percent_surface(
    args: &[FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<FunctionValue, UnaryNumericSurfaceError> {
    eval_unary_numeric_surface(args, resolver, op_percent_kernel)
}

pub fn eval_op_subtract_surface(
    args: &[FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<FunctionValue, BinaryNumericSurfaceError> {
    eval_binary_numeric_surface(args, resolver, op_subtract_kernel)
}

pub fn eval_op_multiply_surface(
    args: &[FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<FunctionValue, BinaryNumericSurfaceError> {
    eval_binary_numeric_surface(args, resolver, op_multiply_kernel)
}

pub fn eval_op_divide_surface(
    args: &[FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<FunctionValue, BinaryNumericSurfaceError> {
    eval_binary_numeric_surface(args, resolver, op_divide_kernel)
}

pub fn eval_op_power_surface(
    args: &[FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<FunctionValue, BinaryNumericSurfaceError> {
    eval_binary_numeric_surface(args, resolver, power_kernel)
}

pub fn map_operator_unary_error_to_ws(e: &UnaryNumericSurfaceError) -> WorksheetErrorCode {
    map_unary_numeric_error_to_ws(e)
}

pub fn map_operator_binary_error_to_ws(e: &BinaryNumericSurfaceError) -> WorksheetErrorCode {
    map_binary_numeric_error_to_ws(e)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::{ReferenceSystemCapabilities, ReferenceSystemProvider};
    use crate::value::{ExcelText, FunctionArray, FunctionArrayCell};

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

    fn txt(s: &str) -> FunctionValue {
        FunctionValue::Text(ExcelText::from_utf16_code_units(s.encode_utf16().collect()))
    }

    #[test]
    fn negate_follows_numeric_coercion() {
        // Unary minus DOES coerce (unlike unary plus).
        assert_eq!(
            eval_op_negate_surface(&[FunctionArg::Eval(txt("2"))], &NoResolver),
            Ok(FunctionValue::Number(-2.0))
        );
        assert_eq!(
            eval_op_negate_surface(
                &[FunctionArg::Eval(FunctionValue::Logical(true))],
                &NoResolver
            ),
            Ok(FunctionValue::Number(-1.0))
        );
    }

    #[test]
    fn unary_plus_is_type_preserving_identity() {
        // BUG-FUNC-029: +x preserves type (text stays text, logical stays logical),
        // matching Excel; it must NOT coerce to number.
        assert_eq!(
            eval_op_unary_plus_surface(&[FunctionArg::Eval(txt("2"))], &NoResolver),
            Ok(txt("2"))
        );
        assert_eq!(
            eval_op_unary_plus_surface(
                &[FunctionArg::Eval(FunctionValue::Logical(true))],
                &NoResolver
            ),
            Ok(FunctionValue::Logical(true))
        );
        // number is unchanged; error propagates; blank -> 0.
        assert_eq!(
            eval_op_unary_plus_surface(
                &[FunctionArg::Eval(FunctionValue::Number(2.0))],
                &NoResolver
            ),
            Ok(FunctionValue::Number(2.0))
        );
        assert_eq!(
            eval_op_unary_plus_surface(
                &[FunctionArg::Eval(FunctionValue::Error(
                    WorksheetErrorCode::NA
                ))],
                &NoResolver
            ),
            Ok(FunctionValue::Error(WorksheetErrorCode::NA))
        );
        assert_eq!(
            eval_op_unary_plus_surface(&[FunctionArg::EmptyCell], &NoResolver),
            Ok(FunctionValue::Number(0.0))
        );
    }

    #[test]
    fn unary_plus_maps_arrays_elementwise_preserving_type() {
        let got = eval_op_unary_plus_surface(
            &[FunctionArg::Eval(FunctionValue::Array(
                FunctionArray::from_rows(vec![vec![
                    FunctionArrayCell::Number(1.0),
                    FunctionArrayCell::Text(ExcelText::from_utf16_code_units(
                        "a".encode_utf16().collect(),
                    )),
                    FunctionArrayCell::Logical(true),
                ]])
                .unwrap(),
            ))],
            &NoResolver,
        );
        let expected = FunctionValue::Array(
            FunctionArray::from_rows(vec![vec![
                FunctionArrayCell::Number(1.0),
                FunctionArrayCell::Text(ExcelText::from_utf16_code_units(
                    "a".encode_utf16().collect(),
                )),
                FunctionArrayCell::Logical(true),
            ]])
            .unwrap(),
        );
        assert_eq!(got, Ok(expected));
    }

    #[test]
    fn percent_lifts_arrays_elementwise() {
        let got = eval_op_percent_surface(
            &[FunctionArg::Eval(FunctionValue::Array(
                FunctionArray::from_rows(vec![vec![
                    FunctionArrayCell::Number(5.0),
                    FunctionArrayCell::Number(25.0),
                ]])
                .unwrap(),
            ))],
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(FunctionValue::Array(
                FunctionArray::from_rows(vec![vec![
                    FunctionArrayCell::Number(0.05),
                    FunctionArrayCell::Number(0.25),
                ]])
                .unwrap()
            ))
        );
    }

    #[test]
    fn subtract_multiply_divide_and_power_cover_seed_numeric_lanes() {
        assert_eq!(
            eval_op_subtract_surface(
                &[
                    FunctionArg::Eval(FunctionValue::Number(5.0)),
                    FunctionArg::Eval(FunctionValue::Number(2.0)),
                ],
                &NoResolver,
            ),
            Ok(FunctionValue::Number(3.0))
        );
        assert_eq!(
            eval_op_multiply_surface(
                &[
                    FunctionArg::Eval(FunctionValue::Number(5.0)),
                    FunctionArg::Eval(FunctionValue::Number(2.0)),
                ],
                &NoResolver,
            ),
            Ok(FunctionValue::Number(10.0))
        );
        assert_eq!(
            eval_op_divide_surface(
                &[
                    FunctionArg::Eval(FunctionValue::Number(5.0)),
                    FunctionArg::Eval(FunctionValue::Number(2.0)),
                ],
                &NoResolver,
            ),
            Ok(FunctionValue::Number(2.5))
        );
        assert_eq!(
            eval_op_power_surface(
                &[
                    FunctionArg::Eval(FunctionValue::Number(2.0)),
                    FunctionArg::Eval(FunctionValue::Number(3.0)),
                ],
                &NoResolver,
            ),
            Ok(FunctionValue::Number(8.0))
        );
    }

    #[test]
    fn divide_by_zero_maps_domain_error() {
        let got = eval_op_divide_surface(
            &[
                FunctionArg::Eval(FunctionValue::Number(1.0)),
                FunctionArg::Eval(FunctionValue::Number(0.0)),
            ],
            &NoResolver,
        );
        assert_eq!(
            got,
            Err(BinaryNumericSurfaceError::Domain(WorksheetErrorCode::Div0))
        );
    }

    #[test]
    fn arithmetic_kernels_flush_excel_denormalized_results_to_zero() {
        assert_eq!(op_unary_plus_kernel(1.0e-309), Ok(0.0));
        assert_eq!(op_negate_kernel(1.0e-309), Ok(0.0));
        assert_eq!(op_percent_kernel(1.0e-307), Ok(0.0));
        assert_eq!(op_subtract_kernel(2.0e-308, 1.0e-308), Ok(0.0));
        assert_eq!(op_multiply_kernel(1.0e-307, 0.01), Ok(0.0));
        assert_eq!(op_divide_kernel(2.3e-308, 2.0), Ok(0.0));
        assert_eq!(op_divide_kernel(5.0e-308, 2.0), Ok(2.5e-308));
    }

    #[test]
    fn binary_operator_surfaces_cover_handoff_array_lift_cases() {
        let multiply = eval_op_multiply_surface(
            &[
                FunctionArg::Eval(FunctionValue::Array(
                    FunctionArray::from_rows(vec![
                        vec![
                            FunctionArrayCell::Number(1.0),
                            FunctionArrayCell::Number(2.0),
                            FunctionArrayCell::Number(3.0),
                        ],
                        vec![
                            FunctionArrayCell::Number(2.0),
                            FunctionArrayCell::Number(3.0),
                            FunctionArrayCell::Number(4.0),
                        ],
                    ])
                    .unwrap(),
                )),
                FunctionArg::Eval(FunctionValue::Number(-1.0)),
            ],
            &NoResolver,
        )
        .unwrap();
        assert_eq!(
            multiply,
            FunctionValue::Array(
                FunctionArray::from_rows(vec![
                    vec![
                        FunctionArrayCell::Number(-1.0),
                        FunctionArrayCell::Number(-2.0),
                        FunctionArrayCell::Number(-3.0),
                    ],
                    vec![
                        FunctionArrayCell::Number(-2.0),
                        FunctionArrayCell::Number(-3.0),
                        FunctionArrayCell::Number(-4.0),
                    ],
                ])
                .unwrap()
            )
        );

        let add = eval_op_subtract_surface(
            &[
                FunctionArg::Eval(FunctionValue::Array(
                    FunctionArray::from_rows(vec![
                        vec![
                            FunctionArrayCell::Number(11.0),
                            FunctionArrayCell::Number(22.0),
                        ],
                        vec![
                            FunctionArrayCell::Number(33.0),
                            FunctionArrayCell::Number(44.0),
                        ],
                    ])
                    .unwrap(),
                )),
                FunctionArg::Eval(FunctionValue::Array(
                    FunctionArray::from_rows(vec![
                        vec![
                            FunctionArrayCell::Number(10.0),
                            FunctionArrayCell::Number(20.0),
                        ],
                        vec![
                            FunctionArrayCell::Number(30.0),
                            FunctionArrayCell::Number(40.0),
                        ],
                    ])
                    .unwrap(),
                )),
            ],
            &NoResolver,
        )
        .unwrap();
        assert_eq!(
            add,
            FunctionValue::Array(
                FunctionArray::from_rows(vec![
                    vec![
                        FunctionArrayCell::Number(1.0),
                        FunctionArrayCell::Number(2.0)
                    ],
                    vec![
                        FunctionArrayCell::Number(3.0),
                        FunctionArrayCell::Number(4.0)
                    ],
                ])
                .unwrap()
            )
        );

        let divide = eval_op_divide_surface(
            &[
                FunctionArg::Eval(FunctionValue::Array(
                    FunctionArray::from_rows(vec![
                        vec![
                            FunctionArrayCell::Number(1.0),
                            FunctionArrayCell::Number(2.0),
                        ],
                        vec![
                            FunctionArrayCell::Number(6.0),
                            FunctionArrayCell::Number(8.0),
                        ],
                    ])
                    .unwrap(),
                )),
                FunctionArg::Eval(FunctionValue::Array(
                    FunctionArray::from_rows(vec![
                        vec![
                            FunctionArrayCell::Number(1.0),
                            FunctionArrayCell::Number(0.0),
                        ],
                        vec![
                            FunctionArrayCell::Number(3.0),
                            FunctionArrayCell::Number(2.0),
                        ],
                    ])
                    .unwrap(),
                )),
            ],
            &NoResolver,
        )
        .unwrap();
        assert_eq!(
            divide,
            FunctionValue::Array(
                FunctionArray::from_rows(vec![
                    vec![
                        FunctionArrayCell::Number(1.0),
                        FunctionArrayCell::Error(WorksheetErrorCode::Div0)
                    ],
                    vec![
                        FunctionArrayCell::Number(2.0),
                        FunctionArrayCell::Number(4.0)
                    ],
                ])
                .unwrap()
            )
        );
    }

    #[test]
    fn power_preserves_excel_domain_errors() {
        let got = eval_op_power_surface(
            &[
                FunctionArg::Eval(FunctionValue::Number(-1.0)),
                FunctionArg::Eval(FunctionValue::Number(0.5)),
            ],
            &NoResolver,
        );
        assert_eq!(
            got,
            Err(BinaryNumericSurfaceError::Domain(WorksheetErrorCode::Num))
        );

        let zero_zero = eval_op_power_surface(
            &[
                FunctionArg::Eval(FunctionValue::Number(0.0)),
                FunctionArg::Eval(FunctionValue::Number(0.0)),
            ],
            &NoResolver,
        );
        assert_eq!(
            zero_zero,
            Err(BinaryNumericSurfaceError::Domain(WorksheetErrorCode::Num))
        );
    }
}

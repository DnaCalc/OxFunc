use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{expand_arg_values_only, prepare_arg_values_only};
use crate::functions::binary_numeric::{
    BinaryNumericSurfaceError, eval_binary_numeric_surface, map_binary_numeric_error_to_ws,
};
use crate::functions::excel_numeric::excel_underflow_to_zero;
use crate::functions::power_fn::power_kernel;
use crate::functions::unary_numeric::{
    UnaryNumericSurfaceError, eval_unary_numeric_surface, map_unary_numeric_error_to_ws,
};
use crate::resolver::ReferenceSystemProvider;
use crate::value::{CalcArray, CalcValue, CoreValue, WorksheetErrorCode};

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
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, UnaryNumericSurfaceError> {
    if args.len() != 1 {
        return Err(UnaryNumericSurfaceError::ArityMismatch {
            expected: 1,
            actual: args.len(),
        });
    }
    let prepared =
        prepare_arg_values_only(&args[0], resolver).map_err(UnaryNumericSurfaceError::Coercion)?;
    if let CoreValue::Array(array) = prepared.core() {
        let cells = expand_arg_values_only(&args[0], resolver)
            .map_err(UnaryNumericSurfaceError::Coercion)?
            .into_iter()
            .map(unary_plus_identity_cell)
            .collect::<Vec<_>>();
        Ok(CalcValue::array(
            CalcArray::new(array.shape(), cells).expect("shape preserved"),
        ))
    } else {
        unary_plus_identity_scalar(prepared)
    }
}

/// Identity map for a scalar unary-plus operand (blank -> 0, number -> underflow-normalized).
fn unary_plus_identity_scalar(prepared: CalcValue) -> Result<CalcValue, UnaryNumericSurfaceError> {
    match prepared.core() {
        CoreValue::Number(n) => Ok(CalcValue::number(excel_underflow_to_zero(*n))),
        CoreValue::Text(_) | CoreValue::Logical(_) | CoreValue::Error(_) => Ok(prepared),
        CoreValue::Empty => Ok(CalcValue::number(0.0)),
        // Reference / Lambda / Array(unreachable here) / MissingArg are not
        // valid scalar unary-plus operands -> #VALUE!.
        _ => Err(UnaryNumericSurfaceError::Domain(WorksheetErrorCode::Value)),
    }
}

/// Identity map for one array cell under unary plus.
fn unary_plus_identity_cell(item: CalcValue) -> CalcValue {
    match item.core() {
        CoreValue::Number(n) => CalcValue::number(excel_underflow_to_zero(*n)),
        CoreValue::Text(_) | CoreValue::Logical(_) | CoreValue::Error(_) => item,
        CoreValue::Empty => CalcValue::number(0.0),
        _ => CalcValue::error(WorksheetErrorCode::Value),
    }
}

pub fn eval_op_negate_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, UnaryNumericSurfaceError> {
    eval_unary_numeric_surface(args, resolver, op_negate_kernel)
}

pub fn eval_op_percent_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, UnaryNumericSurfaceError> {
    eval_unary_numeric_surface(args, resolver, op_percent_kernel)
}

pub fn eval_op_subtract_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, BinaryNumericSurfaceError> {
    eval_binary_numeric_surface(args, resolver, op_subtract_kernel)
}

pub fn eval_op_multiply_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, BinaryNumericSurfaceError> {
    eval_binary_numeric_surface(args, resolver, op_multiply_kernel)
}

pub fn eval_op_divide_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, BinaryNumericSurfaceError> {
    eval_binary_numeric_surface(args, resolver, op_divide_kernel)
}

pub fn eval_op_power_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, BinaryNumericSurfaceError> {
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
    use crate::value::{CalcArray, ExcelText};

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

    fn txt(s: &str) -> CalcValue {
        CalcValue::text(ExcelText::from_utf16_code_units(s.encode_utf16().collect()))
    }

    #[test]
    fn negate_follows_numeric_coercion() {
        // Unary minus DOES coerce (unlike unary plus).
        assert_eq!(
            eval_op_negate_surface(&[(txt("2"))], &NoResolver),
            Ok(CalcValue::number(-2.0))
        );
        assert_eq!(
            eval_op_negate_surface(&[(CalcValue::logical(true))], &NoResolver),
            Ok(CalcValue::number(-1.0))
        );
    }

    #[test]
    fn unary_plus_is_type_preserving_identity() {
        // BUG-FUNC-029: +x preserves type (text stays text, logical stays logical),
        // matching Excel; it must NOT coerce to number.
        assert_eq!(
            eval_op_unary_plus_surface(&[(txt("2"))], &NoResolver),
            Ok(txt("2"))
        );
        assert_eq!(
            eval_op_unary_plus_surface(&[(CalcValue::logical(true))], &NoResolver),
            Ok(CalcValue::logical(true))
        );
        // number is unchanged; error propagates; blank -> 0.
        assert_eq!(
            eval_op_unary_plus_surface(&[(CalcValue::number(2.0))], &NoResolver),
            Ok(CalcValue::number(2.0))
        );
        assert_eq!(
            eval_op_unary_plus_surface(&[(CalcValue::error(WorksheetErrorCode::NA))], &NoResolver),
            Ok(CalcValue::error(WorksheetErrorCode::NA))
        );
        assert_eq!(
            eval_op_unary_plus_surface(&[CalcValue::empty()], &NoResolver),
            Ok(CalcValue::number(0.0))
        );
    }

    #[test]
    fn unary_plus_maps_arrays_elementwise_preserving_type() {
        let got = eval_op_unary_plus_surface(
            &[(CalcValue::array(
                CalcArray::from_rows(vec![vec![
                    CalcValue::number(1.0),
                    CalcValue::text(ExcelText::from_utf16_code_units(
                        "a".encode_utf16().collect(),
                    )),
                    CalcValue::logical(true),
                ]])
                .unwrap(),
            ))],
            &NoResolver,
        );
        let expected = CalcValue::array(
            CalcArray::from_rows(vec![vec![
                CalcValue::number(1.0),
                CalcValue::text(ExcelText::from_utf16_code_units(
                    "a".encode_utf16().collect(),
                )),
                CalcValue::logical(true),
            ]])
            .unwrap(),
        );
        assert_eq!(got, Ok(expected));
    }

    #[test]
    fn percent_lifts_arrays_elementwise() {
        let got = eval_op_percent_surface(
            &[(CalcValue::array(
                CalcArray::from_rows(vec![vec![CalcValue::number(5.0), CalcValue::number(25.0)]])
                    .unwrap(),
            ))],
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(CalcValue::array(
                CalcArray::from_rows(vec![
                    vec![CalcValue::number(0.05), CalcValue::number(0.25),]
                ])
                .unwrap()
            ))
        );
    }

    #[test]
    fn subtract_multiply_divide_and_power_cover_seed_numeric_lanes() {
        assert_eq!(
            eval_op_subtract_surface(
                &[(CalcValue::number(5.0)), (CalcValue::number(2.0)),],
                &NoResolver,
            ),
            Ok(CalcValue::number(3.0))
        );
        assert_eq!(
            eval_op_multiply_surface(
                &[(CalcValue::number(5.0)), (CalcValue::number(2.0)),],
                &NoResolver,
            ),
            Ok(CalcValue::number(10.0))
        );
        assert_eq!(
            eval_op_divide_surface(
                &[(CalcValue::number(5.0)), (CalcValue::number(2.0)),],
                &NoResolver,
            ),
            Ok(CalcValue::number(2.5))
        );
        assert_eq!(
            eval_op_power_surface(
                &[(CalcValue::number(2.0)), (CalcValue::number(3.0)),],
                &NoResolver,
            ),
            Ok(CalcValue::number(8.0))
        );
    }

    #[test]
    fn divide_by_zero_maps_domain_error() {
        let got = eval_op_divide_surface(
            &[(CalcValue::number(1.0)), (CalcValue::number(0.0))],
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
                (CalcValue::array(
                    CalcArray::from_rows(vec![
                        vec![
                            CalcValue::number(1.0),
                            CalcValue::number(2.0),
                            CalcValue::number(3.0),
                        ],
                        vec![
                            CalcValue::number(2.0),
                            CalcValue::number(3.0),
                            CalcValue::number(4.0),
                        ],
                    ])
                    .unwrap(),
                )),
                (CalcValue::number(-1.0)),
            ],
            &NoResolver,
        )
        .unwrap();
        assert_eq!(
            multiply,
            CalcValue::array(
                CalcArray::from_rows(vec![
                    vec![
                        CalcValue::number(-1.0),
                        CalcValue::number(-2.0),
                        CalcValue::number(-3.0),
                    ],
                    vec![
                        CalcValue::number(-2.0),
                        CalcValue::number(-3.0),
                        CalcValue::number(-4.0),
                    ],
                ])
                .unwrap()
            )
        );

        let add = eval_op_subtract_surface(
            &[
                (CalcValue::array(
                    CalcArray::from_rows(vec![
                        vec![CalcValue::number(11.0), CalcValue::number(22.0)],
                        vec![CalcValue::number(33.0), CalcValue::number(44.0)],
                    ])
                    .unwrap(),
                )),
                (CalcValue::array(
                    CalcArray::from_rows(vec![
                        vec![CalcValue::number(10.0), CalcValue::number(20.0)],
                        vec![CalcValue::number(30.0), CalcValue::number(40.0)],
                    ])
                    .unwrap(),
                )),
            ],
            &NoResolver,
        )
        .unwrap();
        assert_eq!(
            add,
            CalcValue::array(
                CalcArray::from_rows(vec![
                    vec![CalcValue::number(1.0), CalcValue::number(2.0)],
                    vec![CalcValue::number(3.0), CalcValue::number(4.0)],
                ])
                .unwrap()
            )
        );

        let divide = eval_op_divide_surface(
            &[
                (CalcValue::array(
                    CalcArray::from_rows(vec![
                        vec![CalcValue::number(1.0), CalcValue::number(2.0)],
                        vec![CalcValue::number(6.0), CalcValue::number(8.0)],
                    ])
                    .unwrap(),
                )),
                (CalcValue::array(
                    CalcArray::from_rows(vec![
                        vec![CalcValue::number(1.0), CalcValue::number(0.0)],
                        vec![CalcValue::number(3.0), CalcValue::number(2.0)],
                    ])
                    .unwrap(),
                )),
            ],
            &NoResolver,
        )
        .unwrap();
        assert_eq!(
            divide,
            CalcValue::array(
                CalcArray::from_rows(vec![
                    vec![
                        CalcValue::number(1.0),
                        CalcValue::error(WorksheetErrorCode::Div0)
                    ],
                    vec![CalcValue::number(2.0), CalcValue::number(4.0)],
                ])
                .unwrap()
            )
        );
    }

    #[test]
    fn power_preserves_excel_domain_errors() {
        let got = eval_op_power_surface(
            &[(CalcValue::number(-1.0)), (CalcValue::number(0.5))],
            &NoResolver,
        );
        assert_eq!(
            got,
            Err(BinaryNumericSurfaceError::Domain(WorksheetErrorCode::Num))
        );

        let zero_zero = eval_op_power_surface(
            &[(CalcValue::number(0.0)), (CalcValue::number(0.0))],
            &NoResolver,
        );
        assert_eq!(
            zero_zero,
            Err(BinaryNumericSurfaceError::Domain(WorksheetErrorCode::Num))
        );
    }
}

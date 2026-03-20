use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::binary_numeric::{
    BinaryNumericSurfaceError, eval_binary_numeric_surface, map_binary_numeric_error_to_ws,
};
use crate::functions::power_fn::power_kernel;
use crate::functions::unary_numeric::{
    UnaryNumericSurfaceError, eval_unary_numeric_surface, map_unary_numeric_error_to_ws,
};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

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
    Ok(value)
}

pub fn op_negate_kernel(value: f64) -> Result<f64, WorksheetErrorCode> {
    Ok(-value)
}

pub fn op_percent_kernel(value: f64) -> Result<f64, WorksheetErrorCode> {
    Ok(value / 100.0)
}

pub fn op_subtract_kernel(lhs: f64, rhs: f64) -> Result<f64, WorksheetErrorCode> {
    Ok(lhs - rhs)
}

pub fn op_multiply_kernel(lhs: f64, rhs: f64) -> Result<f64, WorksheetErrorCode> {
    Ok(lhs * rhs)
}

pub fn op_divide_kernel(lhs: f64, rhs: f64) -> Result<f64, WorksheetErrorCode> {
    if rhs == 0.0 {
        Err(WorksheetErrorCode::Div0)
    } else {
        Ok(lhs / rhs)
    }
}

pub fn eval_op_unary_plus_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, UnaryNumericSurfaceError> {
    eval_unary_numeric_surface(args, resolver, op_unary_plus_kernel)
}

pub fn eval_op_negate_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, UnaryNumericSurfaceError> {
    eval_unary_numeric_surface(args, resolver, op_negate_kernel)
}

pub fn eval_op_percent_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, UnaryNumericSurfaceError> {
    eval_unary_numeric_surface(args, resolver, op_percent_kernel)
}

pub fn eval_op_subtract_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, BinaryNumericSurfaceError> {
    eval_binary_numeric_surface(args, resolver, op_subtract_kernel)
}

pub fn eval_op_multiply_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, BinaryNumericSurfaceError> {
    eval_binary_numeric_surface(args, resolver, op_multiply_kernel)
}

pub fn eval_op_divide_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, BinaryNumericSurfaceError> {
    eval_binary_numeric_surface(args, resolver, op_divide_kernel)
}

pub fn eval_op_power_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, BinaryNumericSurfaceError> {
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
    use crate::resolver::{RefResolutionError, ReferenceResolver, ResolverCapabilities};
    use crate::value::{ArrayCellValue, EvalArray, ExcelText, ReferenceLike};

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

    fn txt(s: &str) -> EvalValue {
        EvalValue::Text(ExcelText::from_utf16_code_units(s.encode_utf16().collect()))
    }

    #[test]
    fn unary_plus_and_negate_follow_numeric_coercion() {
        assert_eq!(
            eval_op_unary_plus_surface(&[CallArgValue::Eval(txt("2"))], &NoResolver),
            Ok(EvalValue::Number(2.0))
        );
        assert_eq!(
            eval_op_negate_surface(&[CallArgValue::Eval(EvalValue::Logical(true))], &NoResolver),
            Ok(EvalValue::Number(-1.0))
        );
    }

    #[test]
    fn percent_lifts_arrays_elementwise() {
        let got = eval_op_percent_surface(
            &[CallArgValue::Eval(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Number(5.0),
                    ArrayCellValue::Number(25.0),
                ]])
                .unwrap(),
            ))],
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Number(0.05),
                    ArrayCellValue::Number(0.25),
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
                    CallArgValue::Eval(EvalValue::Number(5.0)),
                    CallArgValue::Eval(EvalValue::Number(2.0)),
                ],
                &NoResolver,
            ),
            Ok(EvalValue::Number(3.0))
        );
        assert_eq!(
            eval_op_multiply_surface(
                &[
                    CallArgValue::Eval(EvalValue::Number(5.0)),
                    CallArgValue::Eval(EvalValue::Number(2.0)),
                ],
                &NoResolver,
            ),
            Ok(EvalValue::Number(10.0))
        );
        assert_eq!(
            eval_op_divide_surface(
                &[
                    CallArgValue::Eval(EvalValue::Number(5.0)),
                    CallArgValue::Eval(EvalValue::Number(2.0)),
                ],
                &NoResolver,
            ),
            Ok(EvalValue::Number(2.5))
        );
        assert_eq!(
            eval_op_power_surface(
                &[
                    CallArgValue::Eval(EvalValue::Number(2.0)),
                    CallArgValue::Eval(EvalValue::Number(3.0)),
                ],
                &NoResolver,
            ),
            Ok(EvalValue::Number(8.0))
        );
    }

    #[test]
    fn divide_by_zero_maps_domain_error() {
        let got = eval_op_divide_surface(
            &[
                CallArgValue::Eval(EvalValue::Number(1.0)),
                CallArgValue::Eval(EvalValue::Number(0.0)),
            ],
            &NoResolver,
        );
        assert_eq!(
            got,
            Err(BinaryNumericSurfaceError::Domain(WorksheetErrorCode::Div0))
        );
    }

    #[test]
    fn power_preserves_excel_domain_errors() {
        let got = eval_op_power_surface(
            &[
                CallArgValue::Eval(EvalValue::Number(-1.0)),
                CallArgValue::Eval(EvalValue::Number(0.5)),
            ],
            &NoResolver,
        );
        assert_eq!(
            got,
            Err(BinaryNumericSurfaceError::Domain(WorksheetErrorCode::Num))
        );
    }
}

use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    PreparedArgValue, coerce_prepared_to_text, run_values_only_prepared,
};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, ExcelText, WorksheetErrorCode};

const OP_CONCAT_BASE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.OP_CONCAT_BASE",
    arity: Arity::exact(2),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::TextToText,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
};

const OP_COMPARE_BASE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.OP_COMPARE_BASE",
    arity: Arity::exact(2),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
};

pub const OP_CONCAT_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.OP_CONCAT",
    arity: Arity { min: 2, max: 2 },
    ..OP_CONCAT_BASE_META
};

pub const OP_EQUAL_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.OP_EQUAL",
    arity: Arity { min: 2, max: 2 },
    ..OP_COMPARE_BASE_META
};

pub const OP_NOT_EQUAL_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.OP_NOT_EQUAL",
    arity: Arity { min: 2, max: 2 },
    ..OP_COMPARE_BASE_META
};

pub const OP_LESS_THAN_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.OP_LESS_THAN",
    arity: Arity { min: 2, max: 2 },
    ..OP_COMPARE_BASE_META
};

pub const OP_LESS_EQUAL_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.OP_LESS_EQUAL",
    arity: Arity { min: 2, max: 2 },
    ..OP_COMPARE_BASE_META
};

pub const OP_GREATER_THAN_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.OP_GREATER_THAN",
    arity: Arity { min: 2, max: 2 },
    ..OP_COMPARE_BASE_META
};

pub const OP_GREATER_EQUAL_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.OP_GREATER_EQUAL",
    arity: Arity { min: 2, max: 2 },
    ..OP_COMPARE_BASE_META
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CompareOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

#[derive(Debug, Clone, PartialEq)]
enum CompareValue {
    Blank,
    Number(f64),
    Text(String),
    Logical(bool),
}

#[derive(Debug, Clone, PartialEq)]
pub enum OperatorCompareConcatError {
    ArityMismatch { expected: usize, actual: usize },
    Coercion(CoercionError),
}

fn compare_value_from_prepared(
    prepared: &PreparedArgValue,
) -> Result<CompareValue, OperatorCompareConcatError> {
    match prepared {
        PreparedArgValue::Eval(EvalValue::Number(n)) => Ok(CompareValue::Number(*n)),
        PreparedArgValue::Eval(EvalValue::Text(t)) => Ok(CompareValue::Text(t.to_string_lossy())),
        PreparedArgValue::Eval(EvalValue::Logical(b)) => Ok(CompareValue::Logical(*b)),
        PreparedArgValue::Eval(EvalValue::Error(code)) => Err(
            OperatorCompareConcatError::Coercion(CoercionError::WorksheetError(*code)),
        ),
        PreparedArgValue::Eval(EvalValue::Array(_)) => Err(OperatorCompareConcatError::Coercion(
            CoercionError::UnsupportedValueKind("array"),
        )),
        PreparedArgValue::Eval(EvalValue::Reference(_)) => Err(
            OperatorCompareConcatError::Coercion(CoercionError::UnsupportedValueKind("reference")),
        ),
        PreparedArgValue::Eval(EvalValue::Lambda(_)) => Err(OperatorCompareConcatError::Coercion(
            CoercionError::UnsupportedValueKind("lambda_value"),
        )),
        PreparedArgValue::MissingArg | PreparedArgValue::EmptyCell => Ok(CompareValue::Blank),
    }
}

fn normalize_blank_pair(lhs: CompareValue, rhs: CompareValue) -> (CompareValue, CompareValue) {
    match (lhs, rhs) {
        (CompareValue::Blank, CompareValue::Blank) => (CompareValue::Blank, CompareValue::Blank),
        (CompareValue::Blank, CompareValue::Number(rhs)) => {
            (CompareValue::Number(0.0), CompareValue::Number(rhs))
        }
        (CompareValue::Number(lhs), CompareValue::Blank) => {
            (CompareValue::Number(lhs), CompareValue::Number(0.0))
        }
        (CompareValue::Blank, CompareValue::Text(rhs)) => {
            (CompareValue::Text(String::new()), CompareValue::Text(rhs))
        }
        (CompareValue::Text(lhs), CompareValue::Blank) => {
            (CompareValue::Text(lhs), CompareValue::Text(String::new()))
        }
        (CompareValue::Blank, CompareValue::Logical(rhs)) => {
            (CompareValue::Logical(false), CompareValue::Logical(rhs))
        }
        (CompareValue::Logical(lhs), CompareValue::Blank) => {
            (CompareValue::Logical(lhs), CompareValue::Logical(false))
        }
        other => other,
    }
}

fn text_cmp(lhs: &str, rhs: &str) -> std::cmp::Ordering {
    lhs.to_lowercase().cmp(&rhs.to_lowercase())
}

fn type_rank(value: &CompareValue) -> u8 {
    match value {
        CompareValue::Blank => 0,
        CompareValue::Number(_) => 0,
        CompareValue::Text(_) => 1,
        CompareValue::Logical(_) => 2,
    }
}

fn ordering_for_values(lhs: CompareValue, rhs: CompareValue) -> std::cmp::Ordering {
    let (lhs, rhs) = normalize_blank_pair(lhs, rhs);
    match (&lhs, &rhs) {
        (CompareValue::Blank, CompareValue::Blank) => std::cmp::Ordering::Equal,
        (CompareValue::Number(lhs), CompareValue::Number(rhs)) => {
            lhs.partial_cmp(rhs).unwrap_or(std::cmp::Ordering::Equal)
        }
        (CompareValue::Text(lhs), CompareValue::Text(rhs)) => text_cmp(lhs, rhs),
        (CompareValue::Logical(lhs), CompareValue::Logical(rhs)) => lhs.cmp(rhs),
        _ => type_rank(&lhs).cmp(&type_rank(&rhs)),
    }
}

fn compare_values(op: CompareOp, lhs: CompareValue, rhs: CompareValue) -> bool {
    let ord = ordering_for_values(lhs, rhs);
    match op {
        CompareOp::Eq => ord == std::cmp::Ordering::Equal,
        CompareOp::Ne => ord != std::cmp::Ordering::Equal,
        CompareOp::Lt => ord == std::cmp::Ordering::Less,
        CompareOp::Le => ord != std::cmp::Ordering::Greater,
        CompareOp::Gt => ord == std::cmp::Ordering::Greater,
        CompareOp::Ge => ord != std::cmp::Ordering::Less,
    }
}

fn eval_operator_compare_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
    op: CompareOp,
) -> Result<EvalValue, OperatorCompareConcatError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| {
            if prepared.len() != 2 {
                return Err(OperatorCompareConcatError::ArityMismatch {
                    expected: 2,
                    actual: prepared.len(),
                });
            }
            let lhs = compare_value_from_prepared(&prepared[0])?;
            let rhs = compare_value_from_prepared(&prepared[1])?;
            Ok(EvalValue::Logical(compare_values(op, lhs, rhs)))
        },
        OperatorCompareConcatError::Coercion,
    )
}

pub fn eval_op_concat_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, OperatorCompareConcatError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| {
            if prepared.len() != 2 {
                return Err(OperatorCompareConcatError::ArityMismatch {
                    expected: 2,
                    actual: prepared.len(),
                });
            }
            let lhs = coerce_prepared_to_text(&prepared[0])
                .map_err(OperatorCompareConcatError::Coercion)?;
            let rhs = coerce_prepared_to_text(&prepared[1])
                .map_err(OperatorCompareConcatError::Coercion)?;
            let mut out = lhs.utf16_code_units().to_vec();
            out.extend_from_slice(rhs.utf16_code_units());
            Ok(EvalValue::Text(ExcelText::from_utf16_code_units(out)))
        },
        OperatorCompareConcatError::Coercion,
    )
}

pub fn eval_op_equal_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, OperatorCompareConcatError> {
    eval_operator_compare_surface(args, resolver, CompareOp::Eq)
}

pub fn eval_op_not_equal_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, OperatorCompareConcatError> {
    eval_operator_compare_surface(args, resolver, CompareOp::Ne)
}

pub fn eval_op_less_than_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, OperatorCompareConcatError> {
    eval_operator_compare_surface(args, resolver, CompareOp::Lt)
}

pub fn eval_op_less_equal_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, OperatorCompareConcatError> {
    eval_operator_compare_surface(args, resolver, CompareOp::Le)
}

pub fn eval_op_greater_than_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, OperatorCompareConcatError> {
    eval_operator_compare_surface(args, resolver, CompareOp::Gt)
}

pub fn eval_op_greater_equal_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, OperatorCompareConcatError> {
    eval_operator_compare_surface(args, resolver, CompareOp::Ge)
}

pub fn map_operator_compare_concat_error_to_ws(
    e: &OperatorCompareConcatError,
) -> WorksheetErrorCode {
    match e {
        OperatorCompareConcatError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        OperatorCompareConcatError::Coercion(CoercionError::WorksheetError(code)) => *code,
        OperatorCompareConcatError::Coercion(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::{RefResolutionError, ResolverCapabilities};
    use crate::value::ReferenceLike;

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
    fn concat_matches_seed_scalar_lanes() {
        assert_eq!(
            eval_op_concat_surface(
                &[
                    CallArgValue::Eval(txt("a")),
                    CallArgValue::Eval(EvalValue::Number(1.0))
                ],
                &NoResolver,
            ),
            Ok(txt("a1"))
        );
        assert_eq!(
            eval_op_concat_surface(
                &[
                    CallArgValue::Eval(EvalValue::Number(1.0)),
                    CallArgValue::Eval(EvalValue::Logical(true)),
                ],
                &NoResolver,
            ),
            Ok(txt("1TRUE"))
        );
    }

    #[test]
    fn comparisons_match_case_mixed_and_type_ordering_lanes() {
        assert_eq!(
            eval_op_equal_surface(
                &[CallArgValue::Eval(txt("a")), CallArgValue::Eval(txt("A"))],
                &NoResolver,
            ),
            Ok(EvalValue::Logical(true))
        );
        assert_eq!(
            eval_op_equal_surface(
                &[
                    CallArgValue::Eval(EvalValue::Number(1.0)),
                    CallArgValue::Eval(txt("1"))
                ],
                &NoResolver,
            ),
            Ok(EvalValue::Logical(false))
        );
        assert_eq!(
            eval_op_less_than_surface(
                &[
                    CallArgValue::Eval(EvalValue::Logical(false)),
                    CallArgValue::Eval(EvalValue::Logical(true)),
                ],
                &NoResolver,
            ),
            Ok(EvalValue::Logical(true))
        );
        assert_eq!(
            eval_op_greater_than_surface(
                &[
                    CallArgValue::Eval(txt("10")),
                    CallArgValue::Eval(EvalValue::Number(2.0))
                ],
                &NoResolver,
            ),
            Ok(EvalValue::Logical(true))
        );
    }

    #[test]
    fn comparisons_handle_blank_against_number_text_and_logical() {
        assert_eq!(
            eval_op_equal_surface(
                &[
                    CallArgValue::EmptyCell,
                    CallArgValue::Eval(EvalValue::Number(0.0))
                ],
                &NoResolver,
            ),
            Ok(EvalValue::Logical(true))
        );
        assert_eq!(
            eval_op_equal_surface(
                &[CallArgValue::EmptyCell, CallArgValue::Eval(txt(""))],
                &NoResolver,
            ),
            Ok(EvalValue::Logical(true))
        );
        assert_eq!(
            eval_op_equal_surface(
                &[
                    CallArgValue::EmptyCell,
                    CallArgValue::Eval(EvalValue::Logical(false))
                ],
                &NoResolver,
            ),
            Ok(EvalValue::Logical(true))
        );
        assert_eq!(
            eval_op_less_than_surface(
                &[
                    CallArgValue::EmptyCell,
                    CallArgValue::Eval(EvalValue::Number(1.0))
                ],
                &NoResolver,
            ),
            Ok(EvalValue::Logical(true))
        );
    }

    #[test]
    fn comparison_error_operand_propagates() {
        let got = eval_op_equal_surface(
            &[
                CallArgValue::Eval(EvalValue::Error(WorksheetErrorCode::NA)),
                CallArgValue::Eval(EvalValue::Number(1.0)),
            ],
            &NoResolver,
        );
        assert_eq!(
            got,
            Err(OperatorCompareConcatError::Coercion(
                CoercionError::WorksheetError(WorksheetErrorCode::NA,)
            ))
        );
    }
}

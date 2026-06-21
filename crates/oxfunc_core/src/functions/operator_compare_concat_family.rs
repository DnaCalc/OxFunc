use crate::coercion::CoercionError;
use crate::function::{
    Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile, FunctionMeta,
    HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    BroadcastPreparedPair, coerce_prepared_to_text, expand_binary_broadcast_grid,
    run_values_only_prepared,
};
use crate::functions::excel_numeric_compare::compare_excel_numbers;
use crate::resolver::ReferenceSystemProvider;
use crate::value::{CalcArray, CalcValue, CoreValue, ExcelText, WorksheetErrorCode};

const OP_CONCAT_BASE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.OP_CONCAT_BASE",
    arity: Arity::exact(2),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: FunctionMeta::DEFAULT_ARG_PREPARATION_PROFILE,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    lift_broadcast_profile: FunctionMeta::DEFAULT_LIFT_BROADCAST_PROFILE,
    kernel_signature_class: KernelSignatureClass::TextToText,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
    real_result_policy: FunctionMeta::DEFAULT_REAL_RESULT_POLICY,
    error_collapse_profile: FunctionMeta::DEFAULT_ERROR_COLLAPSE_PROFILE,
    precision_rounding_profile: FunctionMeta::DEFAULT_PRECISION_ROUNDING_PROFILE,
};

const OP_COMPARE_BASE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.OP_COMPARE_BASE",
    arity: Arity::exact(2),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: FunctionMeta::DEFAULT_ARG_PREPARATION_PROFILE,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    lift_broadcast_profile: FunctionMeta::DEFAULT_LIFT_BROADCAST_PROFILE,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
    real_result_policy: FunctionMeta::DEFAULT_REAL_RESULT_POLICY,
    error_collapse_profile: FunctionMeta::DEFAULT_ERROR_COLLAPSE_PROFILE,
    precision_rounding_profile: FunctionMeta::DEFAULT_PRECISION_ROUNDING_PROFILE,
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
    prepared: &CalcValue,
) -> Result<CompareValue, OperatorCompareConcatError> {
    match prepared.core() {
        CoreValue::Number(n) => Ok(CompareValue::Number(*n)),
        CoreValue::Text(t) => Ok(CompareValue::Text(t.to_string_lossy())),
        CoreValue::Logical(b) => Ok(CompareValue::Logical(*b)),
        CoreValue::Error(code) => Err(OperatorCompareConcatError::Coercion(
            CoercionError::WorksheetError(*code),
        )),
        CoreValue::Array(_) => Err(OperatorCompareConcatError::Coercion(
            CoercionError::UnsupportedValueKind("array"),
        )),
        CoreValue::Reference(_) => Err(OperatorCompareConcatError::Coercion(
            CoercionError::UnsupportedValueKind("reference"),
        )),
        CoreValue::Missing | CoreValue::Empty => Ok(CompareValue::Blank),
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
        (CompareValue::Number(lhs), CompareValue::Number(rhs)) => compare_excel_numbers(*lhs, *rhs),
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

fn eval_compare_scalar_pair(
    lhs: &CalcValue,
    rhs: &CalcValue,
    op: CompareOp,
) -> Result<CalcValue, OperatorCompareConcatError> {
    let lhs = compare_value_from_prepared(lhs)?;
    let rhs = compare_value_from_prepared(rhs)?;
    Ok(CalcValue::logical(compare_values(op, lhs, rhs)))
}

fn map_compare_item(lhs: &CalcValue, rhs: &CalcValue, op: CompareOp) -> CalcValue {
    match eval_compare_scalar_pair(lhs, rhs, op) {
        Ok(value) if matches!(value.core(), CoreValue::Logical(_)) => value,
        Ok(_) => CalcValue::error(WorksheetErrorCode::Value),
        Err(OperatorCompareConcatError::Coercion(CoercionError::WorksheetError(code))) => {
            CalcValue::error(code)
        }
        Err(_) => CalcValue::error(WorksheetErrorCode::Value),
    }
}

fn eval_concat_scalar_pair(
    lhs: &CalcValue,
    rhs: &CalcValue,
) -> Result<CalcValue, OperatorCompareConcatError> {
    let lhs = coerce_prepared_to_text(lhs).map_err(OperatorCompareConcatError::Coercion)?;
    let rhs = coerce_prepared_to_text(rhs).map_err(OperatorCompareConcatError::Coercion)?;
    let mut out = lhs.utf16_code_units().to_vec();
    out.extend_from_slice(rhs.utf16_code_units());
    Ok(CalcValue::text(ExcelText::from_utf16_code_units(out)))
}

fn map_concat_item(lhs: &CalcValue, rhs: &CalcValue) -> CalcValue {
    match eval_concat_scalar_pair(lhs, rhs) {
        Ok(value) if matches!(value.core(), CoreValue::Text(_)) => value,
        Ok(_) => CalcValue::error(WorksheetErrorCode::Value),
        Err(OperatorCompareConcatError::Coercion(CoercionError::WorksheetError(code))) => {
            CalcValue::error(code)
        }
        Err(_) => CalcValue::error(WorksheetErrorCode::Value),
    }
}

fn eval_operator_compare_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
    op: CompareOp,
) -> Result<CalcValue, OperatorCompareConcatError> {
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
            if let Some((shape, cells)) = expand_binary_broadcast_grid(&prepared[0], &prepared[1]) {
                let mapped = cells
                    .into_iter()
                    .map(|cell| match cell {
                        BroadcastPreparedPair::Pair(lhs, rhs) => map_compare_item(&lhs, &rhs, op),
                        BroadcastPreparedPair::MissingCoordinate => {
                            CalcValue::error(WorksheetErrorCode::NA)
                        }
                    })
                    .collect();
                Ok(CalcValue::array(
                    CalcArray::new(shape, mapped).expect("shape preserved"),
                ))
            } else {
                eval_compare_scalar_pair(&prepared[0], &prepared[1], op)
            }
        },
        OperatorCompareConcatError::Coercion,
    )
}

pub fn eval_op_concat_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, OperatorCompareConcatError> {
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
            if let Some((shape, cells)) = expand_binary_broadcast_grid(&prepared[0], &prepared[1]) {
                let mapped = cells
                    .into_iter()
                    .map(|cell| match cell {
                        BroadcastPreparedPair::Pair(lhs, rhs) => map_concat_item(&lhs, &rhs),
                        BroadcastPreparedPair::MissingCoordinate => {
                            CalcValue::error(WorksheetErrorCode::NA)
                        }
                    })
                    .collect();
                Ok(CalcValue::array(
                    CalcArray::new(shape, mapped).expect("shape preserved"),
                ))
            } else {
                eval_concat_scalar_pair(&prepared[0], &prepared[1])
            }
        },
        OperatorCompareConcatError::Coercion,
    )
}

pub fn eval_op_equal_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, OperatorCompareConcatError> {
    eval_operator_compare_surface(args, resolver, CompareOp::Eq)
}

pub fn eval_op_not_equal_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, OperatorCompareConcatError> {
    eval_operator_compare_surface(args, resolver, CompareOp::Ne)
}

pub fn eval_op_less_than_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, OperatorCompareConcatError> {
    eval_operator_compare_surface(args, resolver, CompareOp::Lt)
}

pub fn eval_op_less_equal_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, OperatorCompareConcatError> {
    eval_operator_compare_surface(args, resolver, CompareOp::Le)
}

pub fn eval_op_greater_than_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, OperatorCompareConcatError> {
    eval_operator_compare_surface(args, resolver, CompareOp::Gt)
}

pub fn eval_op_greater_equal_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, OperatorCompareConcatError> {
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
    use crate::resolver::ReferenceSystemCapabilities;
    use crate::value::CalcValue;

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

    fn text_cell(s: &str) -> CalcValue {
        CalcValue::text(ExcelText::from_utf16_code_units(s.encode_utf16().collect()))
    }

    #[test]
    fn concat_matches_seed_scalar_lanes() {
        assert_eq!(
            eval_op_concat_surface(&[(txt("a")), (CalcValue::number(1.0))], &NoResolver,),
            Ok(txt("a1"))
        );
        assert_eq!(
            eval_op_concat_surface(
                &[(CalcValue::number(1.0)), (CalcValue::logical(true)),],
                &NoResolver,
            ),
            Ok(txt("1TRUE"))
        );
    }

    #[test]
    fn comparisons_match_case_mixed_and_type_ordering_lanes() {
        assert_eq!(
            eval_op_equal_surface(&[(txt("a")), (txt("A"))], &NoResolver,),
            Ok(CalcValue::logical(true))
        );
        assert_eq!(
            eval_op_equal_surface(&[(CalcValue::number(1.0)), (txt("1"))], &NoResolver,),
            Ok(CalcValue::logical(false))
        );
        assert_eq!(
            eval_op_less_than_surface(
                &[(CalcValue::logical(false)), (CalcValue::logical(true)),],
                &NoResolver,
            ),
            Ok(CalcValue::logical(true))
        );
        assert_eq!(
            eval_op_greater_than_surface(&[(txt("10")), (CalcValue::number(2.0))], &NoResolver,),
            Ok(CalcValue::logical(true))
        );
    }

    #[test]
    fn comparisons_handle_blank_against_number_text_and_logical() {
        assert_eq!(
            eval_op_equal_surface(&[CalcValue::empty(), (CalcValue::number(0.0))], &NoResolver,),
            Ok(CalcValue::logical(true))
        );
        assert_eq!(
            eval_op_equal_surface(&[CalcValue::empty(), (txt(""))], &NoResolver,),
            Ok(CalcValue::logical(true))
        );
        assert_eq!(
            eval_op_equal_surface(
                &[CalcValue::empty(), (CalcValue::logical(false))],
                &NoResolver,
            ),
            Ok(CalcValue::logical(true))
        );
        assert_eq!(
            eval_op_less_than_surface(&[CalcValue::empty(), (CalcValue::number(1.0))], &NoResolver,),
            Ok(CalcValue::logical(true))
        );
    }

    #[test]
    fn comparison_error_operand_propagates() {
        let got = eval_op_equal_surface(
            &[
                (CalcValue::error(WorksheetErrorCode::NA)),
                (CalcValue::number(1.0)),
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

    #[test]
    fn concat_surface_broadcasts_arrays() {
        let got = eval_op_concat_surface(
            &[
                (CalcValue::array(
                    CalcArray::from_rows(vec![vec![text_cell("a"), text_cell("b")]]).unwrap(),
                )),
                (CalcValue::array(
                    CalcArray::from_rows(vec![vec![text_cell("x")], vec![text_cell("y")]]).unwrap(),
                )),
            ],
            &NoResolver,
        )
        .unwrap();

        assert_eq!(
            got,
            CalcValue::array(
                CalcArray::from_rows(vec![
                    vec![text_cell("ax"), text_cell("bx")],
                    vec![text_cell("ay"), text_cell("by")],
                ])
                .unwrap()
            )
        );
    }

    #[test]
    fn concat_surface_marks_missing_broadcast_coordinates_as_na() {
        let got = eval_op_concat_surface(
            &[
                (CalcValue::array(
                    CalcArray::from_rows(vec![vec![text_cell("a"), text_cell("b")]]).unwrap(),
                )),
                (CalcValue::array(
                    CalcArray::from_rows(vec![vec![
                        text_cell("x"),
                        text_cell("y"),
                        text_cell("z"),
                    ]])
                    .unwrap(),
                )),
            ],
            &NoResolver,
        )
        .unwrap();

        assert_eq!(
            got,
            CalcValue::array(
                CalcArray::from_rows(vec![vec![
                    text_cell("ax"),
                    text_cell("by"),
                    CalcValue::error(WorksheetErrorCode::NA),
                ]])
                .unwrap()
            )
        );
    }

    #[test]
    fn compare_surface_broadcasts_arrays() {
        let got = eval_op_equal_surface(
            &[
                (CalcValue::array(
                    CalcArray::from_rows(vec![vec![
                        CalcValue::number(1.0),
                        CalcValue::number(2.0),
                    ]])
                    .unwrap(),
                )),
                (CalcValue::array(
                    CalcArray::from_rows(vec![
                        vec![CalcValue::number(1.0)],
                        vec![CalcValue::number(2.0)],
                    ])
                    .unwrap(),
                )),
            ],
            &NoResolver,
        )
        .unwrap();

        assert_eq!(
            got,
            CalcValue::array(
                CalcArray::from_rows(vec![
                    vec![CalcValue::logical(true), CalcValue::logical(false)],
                    vec![CalcValue::logical(false), CalcValue::logical(true)],
                ])
                .unwrap()
            )
        );
    }

    #[test]
    fn compare_surface_marks_missing_broadcast_coordinates_as_na() {
        let got = eval_op_equal_surface(
            &[
                (CalcValue::array(
                    CalcArray::from_rows(vec![vec![
                        CalcValue::number(1.0),
                        CalcValue::number(2.0),
                    ]])
                    .unwrap(),
                )),
                (CalcValue::array(
                    CalcArray::from_rows(vec![vec![
                        CalcValue::number(1.0),
                        CalcValue::number(2.0),
                        CalcValue::number(3.0),
                    ]])
                    .unwrap(),
                )),
            ],
            &NoResolver,
        )
        .unwrap();

        assert_eq!(
            got,
            CalcValue::array(
                CalcArray::from_rows(vec![vec![
                    CalcValue::logical(true),
                    CalcValue::logical(true),
                    CalcValue::error(WorksheetErrorCode::NA),
                ]])
                .unwrap()
            )
        );
    }

    #[test]
    fn comparisons_use_excel_near_equal_numeric_ordering() {
        let lhs = CalcValue::number(0.1 + 0.2);
        let rhs = CalcValue::number(0.3);
        assert_eq!(
            eval_op_equal_surface(&[lhs.clone(), rhs.clone()], &NoResolver),
            Ok(CalcValue::logical(true))
        );
        assert_eq!(
            eval_op_not_equal_surface(&[lhs.clone(), rhs.clone()], &NoResolver),
            Ok(CalcValue::logical(false))
        );
        assert_eq!(
            eval_op_less_than_surface(&[lhs.clone(), rhs.clone()], &NoResolver),
            Ok(CalcValue::logical(false))
        );
        assert_eq!(
            eval_op_less_equal_surface(&[lhs.clone(), rhs.clone()], &NoResolver),
            Ok(CalcValue::logical(true))
        );
        assert_eq!(
            eval_op_greater_than_surface(&[lhs.clone(), rhs.clone()], &NoResolver),
            Ok(CalcValue::logical(false))
        );
        assert_eq!(
            eval_op_greater_equal_surface(&[lhs, rhs], &NoResolver),
            Ok(CalcValue::logical(true))
        );
        assert_eq!(
            eval_op_equal_surface(
                &[(CalcValue::number(1.0 + 1.0e-14)), (CalcValue::number(1.0)),],
                &NoResolver,
            ),
            Ok(CalcValue::logical(false))
        );

        let boundary_lhs = CalcValue::number(((123_456_789_012_345_f64 * 10.0) + 5.0) / 1.0e25);
        let boundary_rhs = CalcValue::number(((123_456_789_012_345_f64 * 10.0) + 4.0) / 1.0e25);
        assert_eq!(
            eval_op_equal_surface(&[boundary_lhs.clone(), boundary_rhs.clone()], &NoResolver),
            Ok(CalcValue::logical(true))
        );
        assert_eq!(
            eval_op_not_equal_surface(&[boundary_lhs.clone(), boundary_rhs.clone()], &NoResolver),
            Ok(CalcValue::logical(false))
        );
        assert_eq!(
            eval_op_less_than_surface(&[boundary_lhs.clone(), boundary_rhs.clone()], &NoResolver),
            Ok(CalcValue::logical(false))
        );
        assert_eq!(
            eval_op_less_equal_surface(&[boundary_lhs.clone(), boundary_rhs.clone()], &NoResolver),
            Ok(CalcValue::logical(true))
        );
        assert_eq!(
            eval_op_greater_than_surface(
                &[boundary_lhs.clone(), boundary_rhs.clone()],
                &NoResolver,
            ),
            Ok(CalcValue::logical(false))
        );
        assert_eq!(
            eval_op_greater_equal_surface(&[boundary_lhs, boundary_rhs], &NoResolver),
            Ok(CalcValue::logical(true))
        );
    }
}

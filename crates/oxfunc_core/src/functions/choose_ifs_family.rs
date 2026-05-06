use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    PreparedArgValue, coerce_prepared_to_number, prepare_arg_values_only,
};
use crate::resolver::ReferenceResolver;
use crate::value::{
    ArrayCellValue, ArrayShape, CallArgValue, EvalArray, EvalValue, WorksheetErrorCode,
};

pub const CHOOSE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.CHOOSE",
    arity: Arity { min: 2, max: 255 },
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::RefsVisibleInAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
};

pub const IFS_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.IFS",
    arity: Arity { min: 2, max: 254 },
    ..CHOOSE_META
};

#[derive(Debug, Clone, PartialEq)]
pub enum ChooseIfsEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    PairStructureMismatch {
        actual: usize,
    },
    IndexCoercion(CoercionError),
    ConditionCoercion(CoercionError),
    SelectedPreparation(CoercionError),
}

fn materialize_selected(prepared: PreparedArgValue) -> EvalValue {
    match prepared {
        PreparedArgValue::Eval(value) => value,
        PreparedArgValue::MissingArg => EvalValue::Error(WorksheetErrorCode::Value),
        PreparedArgValue::EmptyCell => EvalValue::Number(0.0),
    }
}

fn choose_index_from_number(
    choice_count: usize,
    index_num: f64,
) -> Result<usize, WorksheetErrorCode> {
    if !index_num.is_finite() {
        return Err(WorksheetErrorCode::Value);
    }

    let truncated = index_num.trunc();
    if truncated < 1.0 || truncated > choice_count as f64 {
        return Err(WorksheetErrorCode::Value);
    }

    Ok(truncated as usize - 1)
}

fn prepared_from_array_cell(cell: &ArrayCellValue) -> PreparedArgValue {
    match cell {
        ArrayCellValue::Number(n) => PreparedArgValue::Eval(EvalValue::Number(*n)),
        ArrayCellValue::Text(t) => PreparedArgValue::Eval(EvalValue::Text(t.clone())),
        ArrayCellValue::Logical(b) => PreparedArgValue::Eval(EvalValue::Logical(*b)),
        ArrayCellValue::Error(code) => PreparedArgValue::Eval(EvalValue::Error(*code)),
        ArrayCellValue::EmptyCell => PreparedArgValue::EmptyCell,
    }
}

fn scalar_cell(prepared: &PreparedArgValue) -> ArrayCellValue {
    match prepared {
        PreparedArgValue::Eval(EvalValue::Number(n)) => ArrayCellValue::Number(*n),
        PreparedArgValue::Eval(EvalValue::Text(t)) => ArrayCellValue::Text(t.clone()),
        PreparedArgValue::Eval(EvalValue::Logical(b)) => ArrayCellValue::Logical(*b),
        PreparedArgValue::Eval(EvalValue::Error(code)) => ArrayCellValue::Error(*code),
        PreparedArgValue::Eval(EvalValue::Reference(_))
        | PreparedArgValue::Eval(EvalValue::Lambda(_)) => {
            ArrayCellValue::Error(WorksheetErrorCode::Value)
        }
        PreparedArgValue::Eval(EvalValue::Array(_)) => unreachable!(),
        PreparedArgValue::MissingArg => ArrayCellValue::Error(WorksheetErrorCode::Value),
        PreparedArgValue::EmptyCell => ArrayCellValue::Number(0.0),
    }
}

fn materialize_choice_array(prepared: PreparedArgValue) -> EvalArray {
    match prepared {
        PreparedArgValue::Eval(EvalValue::Array(array)) => array,
        other => EvalArray::from_scalar(scalar_cell(&other)),
    }
}

fn choose_index_from_cell(
    choice_count: usize,
    cell: &ArrayCellValue,
) -> Result<usize, WorksheetErrorCode> {
    match coerce_prepared_to_number(&prepared_from_array_cell(cell)) {
        Ok(index_num) => choose_index_from_number(choice_count, index_num),
        Err(CoercionError::WorksheetError(code)) => Err(code),
        Err(_) => Err(WorksheetErrorCode::Value),
    }
}

fn choose_array_surface(
    index_array: &EvalArray,
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, ChooseIfsEvalError> {
    let mut choice_arrays = Vec::with_capacity(args.len().saturating_sub(1));
    for arg in &args[1..] {
        let prepared = prepare_arg_values_only(arg, resolver)
            .map_err(ChooseIfsEvalError::SelectedPreparation)?;
        choice_arrays.push(materialize_choice_array(prepared));
    }

    let block_rows = choice_arrays
        .iter()
        .map(|array| array.shape().rows)
        .max()
        .unwrap_or(1);
    let block_cols = choice_arrays
        .iter()
        .map(|array| array.shape().cols)
        .max()
        .unwrap_or(1);
    let index_shape = index_array.shape();
    let mut cells =
        Vec::with_capacity(index_shape.rows * block_rows * index_shape.cols * block_cols);

    for index_row in 0..index_shape.rows {
        for block_row in 0..block_rows {
            for index_col in 0..index_shape.cols {
                let index_cell = index_array
                    .get(index_row, index_col)
                    .expect("index array bounds validated");
                match choose_index_from_cell(choice_arrays.len(), index_cell) {
                    Ok(selected_index) => {
                        let selected = &choice_arrays[selected_index];
                        for block_col in 0..block_cols {
                            cells.push(
                                selected
                                    .get(block_row, block_col)
                                    .cloned()
                                    .unwrap_or(ArrayCellValue::Error(WorksheetErrorCode::NA)),
                            );
                        }
                    }
                    Err(code) => {
                        for _ in 0..block_cols {
                            cells.push(ArrayCellValue::Error(code));
                        }
                    }
                }
            }
        }
    }

    Ok(EvalValue::Array(
        EvalArray::new(
            ArrayShape {
                rows: index_shape.rows * block_rows,
                cols: index_shape.cols * block_cols,
            },
            cells,
        )
        .expect("choose array output shape is computed"),
    ))
}

fn prepared_condition_truthy(prepared: &PreparedArgValue) -> Result<bool, CoercionError> {
    match prepared {
        PreparedArgValue::MissingArg | PreparedArgValue::EmptyCell => Ok(false),
        PreparedArgValue::Eval(EvalValue::Logical(value)) => Ok(*value),
        PreparedArgValue::Eval(EvalValue::Number(value)) => Ok(*value != 0.0),
        PreparedArgValue::Eval(EvalValue::Text(text)) => {
            Err(CoercionError::NonNumericText(text.to_string_lossy()))
        }
        PreparedArgValue::Eval(EvalValue::Error(code)) => Err(CoercionError::WorksheetError(*code)),
        _ => Ok(coerce_prepared_to_number(prepared)? != 0.0),
    }
}

pub fn eval_choose_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, ChooseIfsEvalError> {
    if !CHOOSE_META.arity.accepts(args.len()) {
        return Err(ChooseIfsEvalError::ArityMismatch {
            expected_min: CHOOSE_META.arity.min,
            expected_max: CHOOSE_META.arity.max,
            actual: args.len(),
        });
    }

    let prepared_index =
        prepare_arg_values_only(&args[0], resolver).map_err(ChooseIfsEvalError::IndexCoercion)?;
    if let PreparedArgValue::Eval(EvalValue::Array(index_array)) = &prepared_index {
        return choose_array_surface(index_array, args, resolver);
    }
    let index_num =
        coerce_prepared_to_number(&prepared_index).map_err(ChooseIfsEvalError::IndexCoercion)?;
    let selected_index = match choose_index_from_number(args.len() - 1, index_num) {
        Ok(index) => index,
        Err(code) => return Ok(EvalValue::Error(code)),
    };

    let selected = prepare_arg_values_only(&args[selected_index + 1], resolver)
        .map_err(ChooseIfsEvalError::SelectedPreparation)?;
    Ok(materialize_selected(selected))
}

pub fn eval_ifs_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, ChooseIfsEvalError> {
    if !IFS_META.arity.accepts(args.len()) {
        return Err(ChooseIfsEvalError::ArityMismatch {
            expected_min: IFS_META.arity.min,
            expected_max: IFS_META.arity.max,
            actual: args.len(),
        });
    }
    if args.len() % 2 != 0 {
        return Err(ChooseIfsEvalError::PairStructureMismatch { actual: args.len() });
    }

    for pair in args.chunks_exact(2) {
        let prepared_condition = prepare_arg_values_only(&pair[0], resolver)
            .map_err(ChooseIfsEvalError::ConditionCoercion)?;
        if prepared_condition_truthy(&prepared_condition)
            .map_err(ChooseIfsEvalError::ConditionCoercion)?
        {
            let selected = prepare_arg_values_only(&pair[1], resolver)
                .map_err(ChooseIfsEvalError::SelectedPreparation)?;
            return Ok(materialize_selected(selected));
        }
    }

    Ok(EvalValue::Error(WorksheetErrorCode::NA))
}

pub fn map_choose_ifs_error_to_ws(error: &ChooseIfsEvalError) -> WorksheetErrorCode {
    match error {
        ChooseIfsEvalError::ArityMismatch { .. }
        | ChooseIfsEvalError::PairStructureMismatch { .. } => WorksheetErrorCode::Value,
        ChooseIfsEvalError::IndexCoercion(CoercionError::WorksheetError(code))
        | ChooseIfsEvalError::ConditionCoercion(CoercionError::WorksheetError(code))
        | ChooseIfsEvalError::SelectedPreparation(CoercionError::WorksheetError(code)) => *code,
        ChooseIfsEvalError::IndexCoercion(_)
        | ChooseIfsEvalError::ConditionCoercion(_)
        | ChooseIfsEvalError::SelectedPreparation(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::{RefResolutionError, ResolverCapabilities};
    use crate::value::{ExcelText, ReferenceKind, ReferenceLike};

    struct MockResolver;

    impl ReferenceResolver for MockResolver {
        fn capabilities(&self) -> ResolverCapabilities {
            ResolverCapabilities::permissive_local()
        }

        fn resolve_reference(
            &self,
            reference: &ReferenceLike,
        ) -> Result<EvalValue, RefResolutionError> {
            match reference.target.as_str() {
                "BLANK" => Ok(EvalValue::Array(crate::value::EvalArray::from_scalar(
                    crate::value::ArrayCellValue::EmptyCell,
                ))),
                "POISON" => Err(RefResolutionError::UnresolvedReference {
                    target: reference.target.clone(),
                }),
                other => Err(RefResolutionError::UnresolvedReference {
                    target: other.to_string(),
                }),
            }
        }
    }

    fn text_arg(s: &str) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
            s.encode_utf16().collect(),
        )))
    }

    fn number_arg(n: f64) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Number(n))
    }

    #[test]
    fn choose_truncates_index_and_does_not_touch_unselected_poison() {
        let got = eval_choose_surface(
            &[
                number_arg(2.9),
                CallArgValue::Reference(ReferenceLike {
                    kind: ReferenceKind::A1,
                    target: "POISON".to_string(),
                }),
                text_arg("picked"),
                CallArgValue::Eval(EvalValue::Error(WorksheetErrorCode::Div0)),
            ],
            &MockResolver,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Text(ExcelText::from_utf16_code_units(
                "picked".encode_utf16().collect(),
            )))
        );
    }

    #[test]
    fn choose_rejects_zero_and_out_of_range_indices() {
        assert_eq!(
            eval_choose_surface(
                &[number_arg(0.9), number_arg(10.0), number_arg(20.0)],
                &MockResolver
            ),
            Ok(EvalValue::Error(WorksheetErrorCode::Value))
        );
        assert_eq!(
            eval_choose_surface(
                &[
                    number_arg(4.0),
                    number_arg(10.0),
                    number_arg(20.0),
                    number_arg(30.0)
                ],
                &MockResolver
            ),
            Ok(EvalValue::Error(WorksheetErrorCode::Value))
        );
    }

    #[test]
    fn choose_materializes_selected_blank_reference_as_zero() {
        let got = eval_choose_surface(
            &[
                number_arg(1.0),
                CallArgValue::Reference(ReferenceLike {
                    kind: ReferenceKind::A1,
                    target: "BLANK".to_string(),
                }),
                number_arg(7.0),
            ],
            &MockResolver,
        );
        assert_eq!(got, Ok(EvalValue::Number(0.0)));
    }

    #[test]
    fn choose_array_index_materializes_scalar_choices_as_row_vector() {
        let got = eval_choose_surface(
            &[
                CallArgValue::Eval(EvalValue::Array(
                    crate::value::EvalArray::from_rows(vec![vec![
                        crate::value::ArrayCellValue::Number(1.0),
                        crate::value::ArrayCellValue::Number(2.0),
                        crate::value::ArrayCellValue::Number(3.0),
                        crate::value::ArrayCellValue::Number(4.0),
                    ]])
                    .unwrap(),
                )),
                text_arg("Alpha"),
                text_arg("Bravo"),
                text_arg("Charlie"),
                text_arg("Delta"),
            ],
            &MockResolver,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Array(
                crate::value::EvalArray::from_rows(vec![vec![
                    crate::value::ArrayCellValue::Text(ExcelText::from_utf16_code_units(
                        "Alpha".encode_utf16().collect(),
                    )),
                    crate::value::ArrayCellValue::Text(ExcelText::from_utf16_code_units(
                        "Bravo".encode_utf16().collect(),
                    )),
                    crate::value::ArrayCellValue::Text(ExcelText::from_utf16_code_units(
                        "Charlie".encode_utf16().collect(),
                    )),
                    crate::value::ArrayCellValue::Text(ExcelText::from_utf16_code_units(
                        "Delta".encode_utf16().collect(),
                    )),
                ]])
                .unwrap()
            ))
        );
    }

    #[test]
    fn choose_array_index_concatenates_column_arrays_by_selected_position() {
        let got = eval_choose_surface(
            &[
                CallArgValue::Eval(EvalValue::Array(
                    crate::value::EvalArray::from_rows(vec![vec![
                        crate::value::ArrayCellValue::Number(1.0),
                        crate::value::ArrayCellValue::Number(2.0),
                        crate::value::ArrayCellValue::Number(3.0),
                    ]])
                    .unwrap(),
                )),
                CallArgValue::Eval(EvalValue::Array(
                    crate::value::EvalArray::from_rows(vec![
                        vec![crate::value::ArrayCellValue::Number(1.0)],
                        vec![crate::value::ArrayCellValue::Number(2.0)],
                        vec![crate::value::ArrayCellValue::Number(3.0)],
                    ])
                    .unwrap(),
                )),
                CallArgValue::Eval(EvalValue::Array(
                    crate::value::EvalArray::from_rows(vec![
                        vec![crate::value::ArrayCellValue::Number(10.0)],
                        vec![crate::value::ArrayCellValue::Number(20.0)],
                        vec![crate::value::ArrayCellValue::Number(30.0)],
                    ])
                    .unwrap(),
                )),
                CallArgValue::Eval(EvalValue::Array(
                    crate::value::EvalArray::from_rows(vec![
                        vec![crate::value::ArrayCellValue::Number(100.0)],
                        vec![crate::value::ArrayCellValue::Number(200.0)],
                        vec![crate::value::ArrayCellValue::Number(300.0)],
                    ])
                    .unwrap(),
                )),
            ],
            &MockResolver,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Array(
                crate::value::EvalArray::from_rows(vec![
                    vec![
                        crate::value::ArrayCellValue::Number(1.0),
                        crate::value::ArrayCellValue::Number(10.0),
                        crate::value::ArrayCellValue::Number(100.0),
                    ],
                    vec![
                        crate::value::ArrayCellValue::Number(2.0),
                        crate::value::ArrayCellValue::Number(20.0),
                        crate::value::ArrayCellValue::Number(200.0),
                    ],
                    vec![
                        crate::value::ArrayCellValue::Number(3.0),
                        crate::value::ArrayCellValue::Number(30.0),
                        crate::value::ArrayCellValue::Number(300.0),
                    ],
                ])
                .unwrap()
            ))
        );
    }

    #[test]
    fn ifs_scans_pairs_left_to_right_and_short_circuits() {
        let got = eval_ifs_surface(
            &[
                CallArgValue::Eval(EvalValue::Logical(false)),
                CallArgValue::Reference(ReferenceLike {
                    kind: ReferenceKind::A1,
                    target: "POISON".to_string(),
                }),
                number_arg(2.0),
                text_arg("hit"),
                CallArgValue::Reference(ReferenceLike {
                    kind: ReferenceKind::A1,
                    target: "POISON".to_string(),
                }),
                number_arg(99.0),
            ],
            &MockResolver,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Text(ExcelText::from_utf16_code_units(
                "hit".encode_utf16().collect(),
            )))
        );
    }

    #[test]
    fn ifs_returns_na_when_no_condition_matches() {
        let got = eval_ifs_surface(
            &[
                CallArgValue::Eval(EvalValue::Logical(false)),
                number_arg(1.0),
                number_arg(0.0),
                number_arg(2.0),
                CallArgValue::EmptyCell,
                number_arg(3.0),
            ],
            &MockResolver,
        );
        assert_eq!(got, Ok(EvalValue::Error(WorksheetErrorCode::NA)));
    }

    #[test]
    fn ifs_rejects_odd_pair_structure_and_propagates_condition_errors() {
        let odd = eval_ifs_surface(
            &[number_arg(1.0), number_arg(2.0), number_arg(3.0)],
            &MockResolver,
        );
        assert_eq!(
            odd,
            Err(ChooseIfsEvalError::PairStructureMismatch { actual: 3 })
        );

        let condition_error = eval_ifs_surface(
            &[
                CallArgValue::Eval(EvalValue::Error(WorksheetErrorCode::Div0)),
                number_arg(1.0),
                CallArgValue::Eval(EvalValue::Logical(true)),
                number_arg(2.0),
            ],
            &MockResolver,
        );
        assert_eq!(
            condition_error,
            Err(ChooseIfsEvalError::ConditionCoercion(
                CoercionError::WorksheetError(WorksheetErrorCode::Div0)
            ))
        );

        let text_condition = eval_ifs_surface(&[text_arg("2"), number_arg(1.0)], &MockResolver);
        assert_eq!(
            text_condition,
            Err(ChooseIfsEvalError::ConditionCoercion(
                CoercionError::NonNumericText("2".to_string())
            ))
        );
    }

    #[test]
    fn ifs_empty_text_condition_returns_value_error() {
        let got = eval_ifs_surface(&[text_arg(""), number_arg(1.0)], &MockResolver);
        assert_eq!(
            got,
            Err(ChooseIfsEvalError::ConditionCoercion(
                CoercionError::NonNumericText("".to_string())
            ))
        );
    }
}

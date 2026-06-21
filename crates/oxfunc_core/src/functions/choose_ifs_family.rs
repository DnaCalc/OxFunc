use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{coerce_prepared_to_number, prepare_arg_values_only};
use crate::resolver::ReferenceSystemProvider;
use crate::value::{ArrayShape, CalcArray, WorksheetErrorCode};
use crate::value::{CalcValue, CoreValue};

pub const CHOOSE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.CHOOSE",
    arity: Arity { min: 2, max: 255 },
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::RefsVisibleInAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    lift_broadcast_profile: FunctionMeta::DEFAULT_LIFT_BROADCAST_PROFILE,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
    real_result_policy: FunctionMeta::DEFAULT_REAL_RESULT_POLICY,
};

// IFS broadcasts its first condition/value pair plus the second condition (`[0,1,2]`) over an
// array; CHOOSE lifts natively (default). Verified live Excel 16.0 build 20026.
pub const IFS_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.IFS",
    arity: Arity { min: 2, max: 254 },
    lift_broadcast_profile: FunctionMeta::lift_at(&[0, 1, 2]),
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

fn materialize_selected(prepared: CalcValue) -> CalcValue {
    match prepared.core() {
        CoreValue::Missing => CalcValue::error(WorksheetErrorCode::Value),
        CoreValue::Empty => CalcValue::number(0.0),
        _ => prepared,
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

fn prepared_from_array_cell(cell: &CalcValue) -> CalcValue {
    match cell.core() {
        CoreValue::Number(n) => CalcValue::number(*n),
        CoreValue::Text(t) => CalcValue::text(t.clone()),
        CoreValue::Logical(b) => CalcValue::logical(*b),
        CoreValue::Error(code) => CalcValue::error(*code),
        CoreValue::Empty | CoreValue::Missing => CalcValue::empty(),
        CoreValue::Array(_) | CoreValue::Reference(_) => {
            CalcValue::error(WorksheetErrorCode::Value)
        }
    }
}

fn scalar_cell(prepared: &CalcValue) -> CalcValue {
    match prepared.core() {
        CoreValue::Number(n) => CalcValue::number(*n),
        CoreValue::Text(t) => CalcValue::text(t.clone()),
        CoreValue::Logical(b) => CalcValue::logical(*b),
        CoreValue::Error(code) => CalcValue::error(*code),
        CoreValue::Reference(_) => CalcValue::error(WorksheetErrorCode::Value),
        CoreValue::Array(_) => unreachable!(),
        CoreValue::Missing => CalcValue::error(WorksheetErrorCode::Value),
        CoreValue::Empty => CalcValue::number(0.0),
    }
}

fn materialize_choice_array(prepared: CalcValue) -> CalcArray {
    match prepared.core() {
        CoreValue::Array(array) => array.clone(),
        _ => CalcArray::from_scalar(scalar_cell(&prepared))
            .expect("scalar choice array has valid dimensions"),
    }
}

fn choose_index_from_cell(
    choice_count: usize,
    cell: &CalcValue,
) -> Result<usize, WorksheetErrorCode> {
    match coerce_prepared_to_number(&prepared_from_array_cell(cell)) {
        Ok(index_num) => choose_index_from_number(choice_count, index_num),
        Err(CoercionError::WorksheetError(code)) => Err(code),
        Err(_) => Err(WorksheetErrorCode::Value),
    }
}

fn choose_array_surface(
    index_array: &CalcArray,
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, ChooseIfsEvalError> {
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
                                    .unwrap_or(CalcValue::error(WorksheetErrorCode::NA)),
                            );
                        }
                    }
                    Err(code) => {
                        for _ in 0..block_cols {
                            cells.push(CalcValue::error(code));
                        }
                    }
                }
            }
        }
    }

    Ok(CalcValue::array(
        CalcArray::new(
            ArrayShape {
                rows: index_shape.rows * block_rows,
                cols: index_shape.cols * block_cols,
            },
            cells,
        )
        .expect("choose array output shape is computed"),
    ))
}

fn prepared_condition_truthy(prepared: &CalcValue) -> Result<bool, CoercionError> {
    match prepared.core() {
        CoreValue::Missing | CoreValue::Empty => Ok(false),
        CoreValue::Logical(value) => Ok(*value),
        CoreValue::Number(value) => Ok(*value != 0.0),
        CoreValue::Text(text) => Err(CoercionError::NonNumericText(text.to_string_lossy())),
        CoreValue::Error(code) => Err(CoercionError::WorksheetError(*code)),
        _ => Ok(coerce_prepared_to_number(prepared)? != 0.0),
    }
}

pub fn eval_choose_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, ChooseIfsEvalError> {
    if !CHOOSE_META.arity.accepts(args.len()) {
        return Err(ChooseIfsEvalError::ArityMismatch {
            expected_min: CHOOSE_META.arity.min,
            expected_max: CHOOSE_META.arity.max,
            actual: args.len(),
        });
    }

    let prepared_index =
        prepare_arg_values_only(&args[0], resolver).map_err(ChooseIfsEvalError::IndexCoercion)?;
    if let CoreValue::Array(index_array) = prepared_index.core() {
        return choose_array_surface(index_array, args, resolver);
    }
    let index_num =
        coerce_prepared_to_number(&prepared_index).map_err(ChooseIfsEvalError::IndexCoercion)?;
    let selected_index = match choose_index_from_number(args.len() - 1, index_num) {
        Ok(index) => index,
        Err(code) => return Ok(CalcValue::error(code)),
    };

    let selected = prepare_arg_values_only(&args[selected_index + 1], resolver)
        .map_err(ChooseIfsEvalError::SelectedPreparation)?;
    Ok(materialize_selected(selected))
}

pub fn eval_ifs_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, ChooseIfsEvalError> {
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

    Ok(CalcValue::error(WorksheetErrorCode::NA))
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
    use crate::resolver::ReferenceSystemCapabilities;
    use crate::value::{ExcelText, ReferenceKind, ReferenceLike};

    struct MockResolver;

    impl ReferenceSystemProvider for MockResolver {
        fn capabilities(&self) -> ReferenceSystemCapabilities {
            ReferenceSystemCapabilities::permissive_local()
        }

        fn dereference(
            &self,
            request: &crate::resolver::ReferenceDereferenceRequest,
        ) -> Result<CalcValue, crate::resolver::ReferenceResolutionError> {
            let reference = &request.reference;
            match reference.target() {
                "BLANK" => Ok(CalcValue::array(
                    crate::value::CalcArray::from_scalar(crate::value::CalcValue::empty())
                        .expect("scalar array"),
                )),
                "POISON" => Err(
                    crate::resolver::ReferenceResolutionError::UnresolvedReference {
                        target: reference.target().to_string(),
                    },
                ),
                other => Err(
                    crate::resolver::ReferenceResolutionError::UnresolvedReference {
                        target: other.to_string(),
                    },
                ),
            }
        }
    }

    fn text_arg(s: &str) -> CalcValue {
        CalcValue::text(ExcelText::from_utf16_code_units(s.encode_utf16().collect()))
    }

    fn number_arg(n: f64) -> CalcValue {
        CalcValue::number(n)
    }

    #[test]
    fn choose_truncates_index_and_does_not_touch_unselected_poison() {
        let got = eval_choose_surface(
            &[
                number_arg(2.9),
                CalcValue::reference(ReferenceLike::new(ReferenceKind::A1, "POISON".to_string())),
                text_arg("picked"),
                (CalcValue::error(WorksheetErrorCode::Div0)),
            ],
            &MockResolver,
        );
        assert_eq!(
            got,
            Ok(CalcValue::text(ExcelText::from_utf16_code_units(
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
            Ok(CalcValue::error(WorksheetErrorCode::Value))
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
            Ok(CalcValue::error(WorksheetErrorCode::Value))
        );
    }

    #[test]
    fn choose_materializes_selected_blank_reference_as_zero() {
        let got = eval_choose_surface(
            &[
                number_arg(1.0),
                CalcValue::reference(ReferenceLike::new(ReferenceKind::A1, "BLANK".to_string())),
                number_arg(7.0),
            ],
            &MockResolver,
        );
        assert_eq!(got, Ok(CalcValue::number(0.0)));
    }

    #[test]
    fn choose_array_index_materializes_scalar_choices_as_row_vector() {
        let got = eval_choose_surface(
            &[
                (CalcValue::array(
                    crate::value::CalcArray::from_rows(vec![vec![
                        crate::value::CalcValue::number(1.0),
                        crate::value::CalcValue::number(2.0),
                        crate::value::CalcValue::number(3.0),
                        crate::value::CalcValue::number(4.0),
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
            Ok(CalcValue::array(
                crate::value::CalcArray::from_rows(vec![vec![
                    crate::value::CalcValue::text(ExcelText::from_utf16_code_units(
                        "Alpha".encode_utf16().collect(),
                    )),
                    crate::value::CalcValue::text(ExcelText::from_utf16_code_units(
                        "Bravo".encode_utf16().collect(),
                    )),
                    crate::value::CalcValue::text(ExcelText::from_utf16_code_units(
                        "Charlie".encode_utf16().collect(),
                    )),
                    crate::value::CalcValue::text(ExcelText::from_utf16_code_units(
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
                (CalcValue::array(
                    crate::value::CalcArray::from_rows(vec![vec![
                        crate::value::CalcValue::number(1.0),
                        crate::value::CalcValue::number(2.0),
                        crate::value::CalcValue::number(3.0),
                    ]])
                    .unwrap(),
                )),
                (CalcValue::array(
                    crate::value::CalcArray::from_rows(vec![
                        vec![crate::value::CalcValue::number(1.0)],
                        vec![crate::value::CalcValue::number(2.0)],
                        vec![crate::value::CalcValue::number(3.0)],
                    ])
                    .unwrap(),
                )),
                (CalcValue::array(
                    crate::value::CalcArray::from_rows(vec![
                        vec![crate::value::CalcValue::number(10.0)],
                        vec![crate::value::CalcValue::number(20.0)],
                        vec![crate::value::CalcValue::number(30.0)],
                    ])
                    .unwrap(),
                )),
                (CalcValue::array(
                    crate::value::CalcArray::from_rows(vec![
                        vec![crate::value::CalcValue::number(100.0)],
                        vec![crate::value::CalcValue::number(200.0)],
                        vec![crate::value::CalcValue::number(300.0)],
                    ])
                    .unwrap(),
                )),
            ],
            &MockResolver,
        );
        assert_eq!(
            got,
            Ok(CalcValue::array(
                crate::value::CalcArray::from_rows(vec![
                    vec![
                        crate::value::CalcValue::number(1.0),
                        crate::value::CalcValue::number(10.0),
                        crate::value::CalcValue::number(100.0),
                    ],
                    vec![
                        crate::value::CalcValue::number(2.0),
                        crate::value::CalcValue::number(20.0),
                        crate::value::CalcValue::number(200.0),
                    ],
                    vec![
                        crate::value::CalcValue::number(3.0),
                        crate::value::CalcValue::number(30.0),
                        crate::value::CalcValue::number(300.0),
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
                (CalcValue::logical(false)),
                CalcValue::reference(ReferenceLike::new(ReferenceKind::A1, "POISON".to_string())),
                number_arg(2.0),
                text_arg("hit"),
                CalcValue::reference(ReferenceLike::new(ReferenceKind::A1, "POISON".to_string())),
                number_arg(99.0),
            ],
            &MockResolver,
        );
        assert_eq!(
            got,
            Ok(CalcValue::text(ExcelText::from_utf16_code_units(
                "hit".encode_utf16().collect(),
            )))
        );
    }

    #[test]
    fn ifs_returns_na_when_no_condition_matches() {
        let got = eval_ifs_surface(
            &[
                (CalcValue::logical(false)),
                number_arg(1.0),
                number_arg(0.0),
                number_arg(2.0),
                CalcValue::empty(),
                number_arg(3.0),
            ],
            &MockResolver,
        );
        assert_eq!(got, Ok(CalcValue::error(WorksheetErrorCode::NA)));
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
                (CalcValue::error(WorksheetErrorCode::Div0)),
                number_arg(1.0),
                (CalcValue::logical(true)),
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

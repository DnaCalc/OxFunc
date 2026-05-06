use std::cmp::Ordering;

use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{PreparedArgValue, prepare_args_values_only};
use crate::functions::callable_helpers::{
    CallableInvoker, LambdaHelperEvalError, map_lambda_helper_error_to_ws,
};
use crate::functions::group_pivot_common::{
    CellKey, FieldHeadersMode, default_column_field_headers, default_row_field_headers,
    default_value_headers, group_indices_by_key, invoke_group_aggregate, key_from_cells,
    parse_field_headers_mode, parse_filter_vector, parse_sort_orders, prepared_to_array,
    require_callable, row_as_cells, split_header_row, take_header_row, text_cell,
};
use crate::resolver::ReferenceResolver;
use crate::value::{ArrayCellValue, CallArgValue, EvalArray, EvalValue, WorksheetErrorCode};

pub const PIVOTBY_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.PIVOTBY",
    arity: Arity { min: 4, max: 255 },
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

fn surface_arity_error(actual: usize) -> LambdaHelperEvalError {
    LambdaHelperEvalError::ArityMismatch {
        expected_min: PIVOTBY_META.arity.min,
        expected_max: PIVOTBY_META.arity.max,
        actual,
    }
}

#[derive(Debug, Clone)]
struct AxisGroup {
    key_cells: Vec<ArrayCellValue>,
    key: Vec<CellKey>,
    row_indices: Vec<usize>,
}

fn header_mode_shows_output(mode: FieldHeadersMode, had_headers: bool) -> bool {
    match mode {
        FieldHeadersMode::YesShow | FieldHeadersMode::NoGenerate => true,
        FieldHeadersMode::YesHide | FieldHeadersMode::No => false,
        FieldHeadersMode::Auto => had_headers,
    }
}

fn parse_total_depth(prepared: Option<&PreparedArgValue>) -> Result<i32, LambdaHelperEvalError> {
    let depth = crate::functions::group_pivot_common::coerce_optional_i32(prepared)?.unwrap_or(1);
    if depth == 0 || depth == 1 {
        return Ok(depth);
    }
    Err(LambdaHelperEvalError::Invocation(
        crate::functions::callable_helpers::CallableInvocationError::Worksheet(
            WorksheetErrorCode::Value,
        ),
    ))
}

fn compare_cell(a: &ArrayCellValue, b: &ArrayCellValue) -> Ordering {
    match (a, b) {
        (ArrayCellValue::Number(x), ArrayCellValue::Number(y)) => {
            x.partial_cmp(y).unwrap_or(Ordering::Equal)
        }
        (ArrayCellValue::Text(x), ArrayCellValue::Text(y)) => {
            x.to_string_lossy().cmp(&y.to_string_lossy())
        }
        (ArrayCellValue::Logical(x), ArrayCellValue::Logical(y)) => x.cmp(y),
        (ArrayCellValue::EmptyCell, ArrayCellValue::EmptyCell) => Ordering::Equal,
        (ArrayCellValue::Error(x), ArrayCellValue::Error(y)) => (*x as u8).cmp(&(*y as u8)),
        _ => sort_rank(a).cmp(&sort_rank(b)),
    }
}

fn sort_rank(cell: &ArrayCellValue) -> u8 {
    match cell {
        ArrayCellValue::Number(_) => 0,
        ArrayCellValue::Text(_) => 1,
        ArrayCellValue::Logical(_) => 2,
        ArrayCellValue::Error(_) => 3,
        ArrayCellValue::EmptyCell => 4,
    }
}

fn apply_axis_sort(
    groups: &mut [AxisGroup],
    totals: &[(Vec<CellKey>, Vec<ArrayCellValue>)],
    sort_orders: &[i32],
    field_cols: usize,
) {
    if sort_orders.is_empty() {
        return;
    }
    groups.sort_by(|left, right| {
        let left_total = totals
            .iter()
            .find(|(key, _)| *key == left.key)
            .map(|(_, totals)| totals.as_slice())
            .unwrap_or(&[]);
        let right_total = totals
            .iter()
            .find(|(key, _)| *key == right.key)
            .map(|(_, totals)| totals.as_slice())
            .unwrap_or(&[]);

        for entry in sort_orders {
            let descending = *entry < 0;
            let ordinal = entry.unsigned_abs() as usize;
            let cmp = if ordinal == 0 {
                Ordering::Equal
            } else if ordinal <= field_cols {
                compare_cell(&left.key_cells[ordinal - 1], &right.key_cells[ordinal - 1])
            } else {
                let idx = ordinal - field_cols - 1;
                if idx >= left_total.len() || idx >= right_total.len() {
                    Ordering::Equal
                } else {
                    compare_cell(&left_total[idx], &right_total[idx])
                }
            };
            if cmp != Ordering::Equal {
                return if descending { cmp.reverse() } else { cmp };
            }
        }
        Ordering::Equal
    });
}

fn aggregate_totals_for_groups(
    groups: &[AxisGroup],
    value_rows: &[Vec<ArrayCellValue>],
    callable: &crate::value::LambdaValue,
    invoker: &(impl CallableInvoker + ?Sized),
) -> Result<Vec<(Vec<CellKey>, Vec<ArrayCellValue>)>, LambdaHelperEvalError> {
    let value_cols = value_rows.first().map_or(1, Vec::len);
    groups
        .iter()
        .map(|group| {
            let totals = (0..value_cols)
                .map(|value_col| {
                    let members = group
                        .row_indices
                        .iter()
                        .map(|row| value_rows[*row][value_col].clone())
                        .collect::<Vec<_>>();
                    invoke_group_aggregate(callable, &members, invoker)
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok((group.key.clone(), totals))
        })
        .collect()
}

fn build_axis_groups(rows: &[Vec<ArrayCellValue>]) -> Vec<AxisGroup> {
    let keys = rows
        .iter()
        .map(|row| key_from_cells(row))
        .collect::<Vec<_>>();
    group_indices_by_key(rows.len(), &keys)
        .into_iter()
        .map(|row_indices| {
            let first = row_indices[0];
            AxisGroup {
                key_cells: rows[first].clone(),
                key: keys[first].clone(),
                row_indices,
            }
        })
        .collect()
}

fn extract_filtered_rows(
    row_fields: &EvalArray,
    col_fields: &EvalArray,
    values: &EvalArray,
    filter: Option<&[bool]>,
) -> Result<
    (
        Vec<Vec<ArrayCellValue>>,
        Vec<Vec<ArrayCellValue>>,
        Vec<Vec<ArrayCellValue>>,
    ),
    LambdaHelperEvalError,
> {
    if row_fields.shape().rows != col_fields.shape().rows
        || row_fields.shape().rows != values.shape().rows
    {
        return Err(LambdaHelperEvalError::Invocation(
            crate::functions::callable_helpers::CallableInvocationError::Worksheet(
                WorksheetErrorCode::Value,
            ),
        ));
    }
    let mut row_rows = Vec::new();
    let mut col_rows = Vec::new();
    let mut value_rows = Vec::new();
    for row in 0..row_fields.shape().rows {
        if filter.is_some_and(|keep| !keep[row]) {
            continue;
        }
        row_rows.push(row_as_cells(row_fields, row));
        col_rows.push(row_as_cells(col_fields, row));
        value_rows.push(row_as_cells(values, row));
    }
    Ok((row_rows, col_rows, value_rows))
}

fn find_intersection_rows(row_group: &AxisGroup, col_group: &AxisGroup) -> Vec<usize> {
    let mut left = row_group.row_indices.clone();
    let mut right = col_group.row_indices.clone();
    left.sort_unstable();
    right.sort_unstable();
    let mut out = Vec::new();
    let (mut i, mut j) = (0, 0);
    while i < left.len() && j < right.len() {
        match left[i].cmp(&right[j]) {
            Ordering::Equal => {
                out.push(left[i]);
                i += 1;
                j += 1;
            }
            Ordering::Less => i += 1,
            Ordering::Greater => j += 1,
        }
    }
    out
}

pub fn eval_pivotby_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
    invoker: &(impl CallableInvoker + ?Sized),
) -> Result<EvalValue, LambdaHelperEvalError> {
    if !PIVOTBY_META.arity.accepts(args.len()) {
        return Err(surface_arity_error(args.len()));
    }

    let prepared =
        prepare_args_values_only(args, resolver).map_err(LambdaHelperEvalError::Preparation)?;
    let callable = require_callable(&prepared[3])?;
    if prepared.get(10).is_some_and(|arg| {
        !matches!(
            arg,
            PreparedArgValue::MissingArg | PreparedArgValue::EmptyCell
        )
    }) {
        return Err(LambdaHelperEvalError::Invocation(
            crate::functions::callable_helpers::CallableInvocationError::Worksheet(
                WorksheetErrorCode::Value,
            ),
        ));
    }

    let field_headers_mode = parse_field_headers_mode(prepared.get(4))?;
    let row_total_depth = parse_total_depth(prepared.get(5))?;
    let row_sort_orders = parse_sort_orders(prepared.get(6))?;
    let col_total_depth = parse_total_depth(prepared.get(7))?;
    let col_sort_orders = parse_sort_orders(prepared.get(8))?;

    let raw_row_fields = prepared_to_array(&prepared[0]);
    let raw_col_fields = prepared_to_array(&prepared[1]);
    let raw_values = prepared_to_array(&prepared[2]);

    let row_headers = take_header_row(&raw_row_fields);
    let col_headers = take_header_row(&raw_col_fields);
    let value_headers = take_header_row(&raw_values);

    let split_rows = split_header_row(&raw_row_fields, field_headers_mode)?;
    let split_cols = split_header_row(&raw_col_fields, field_headers_mode)?;
    let split_values = split_header_row(&raw_values, field_headers_mode)?;

    let filter = parse_filter_vector(prepared.get(9), split_rows.array.shape().rows)?;
    let (row_rows, col_rows, value_rows) = extract_filtered_rows(
        &split_rows.array,
        &split_cols.array,
        &split_values.array,
        filter.as_deref(),
    )?;

    if row_rows.is_empty() {
        return Ok(EvalValue::Array(
            EvalArray::from_rows(vec![vec![ArrayCellValue::Error(WorksheetErrorCode::Calc)]])
                .expect("scalar calc error"),
        ));
    }

    let mut row_groups = build_axis_groups(&row_rows);
    let mut col_groups = build_axis_groups(&col_rows);
    let row_totals = aggregate_totals_for_groups(&row_groups, &value_rows, callable, invoker)?;
    let col_totals = aggregate_totals_for_groups(&col_groups, &value_rows, callable, invoker)?;
    apply_axis_sort(
        &mut row_groups,
        &row_totals,
        &row_sort_orders,
        split_rows.array.shape().cols,
    );
    apply_axis_sort(
        &mut col_groups,
        &col_totals,
        &col_sort_orders,
        split_cols.array.shape().cols,
    );

    let mut rows = Vec::new();
    let show_headers = header_mode_shows_output(field_headers_mode, split_values.had_headers);

    if show_headers {
        let top_label = if matches!(field_headers_mode, FieldHeadersMode::NoGenerate) {
            default_column_field_headers(split_cols.array.shape().cols)
        } else if split_cols.had_headers {
            col_headers
        } else {
            default_column_field_headers(split_cols.array.shape().cols)
        };
        let row_label = if matches!(field_headers_mode, FieldHeadersMode::NoGenerate) {
            default_row_field_headers(split_rows.array.shape().cols)
        } else if split_rows.had_headers {
            row_headers
        } else {
            default_row_field_headers(split_rows.array.shape().cols)
        };
        let value_label = if matches!(field_headers_mode, FieldHeadersMode::NoGenerate) {
            default_value_headers(split_values.array.shape().cols)
        } else if split_values.had_headers {
            value_headers
        } else {
            default_value_headers(split_values.array.shape().cols)
        };

        if split_cols.array.shape().cols == 1 && split_rows.array.shape().cols == 1 {
            let mut row0 = vec![ArrayCellValue::EmptyCell];
            row0.push(top_label[0].clone());
            row0.extend((1..col_groups.len()).map(|_| ArrayCellValue::EmptyCell));
            if col_total_depth != 0 {
                row0.push(ArrayCellValue::EmptyCell);
            }
            rows.push(row0);

            let mut row1 = vec![ArrayCellValue::EmptyCell];
            row1.extend(col_groups.iter().map(|group| group.key_cells[0].clone()));
            if col_total_depth != 0 {
                row1.push(text_cell("Total"));
            }
            rows.push(row1);

            let mut row2 = vec![row_label[0].clone()];
            row2.extend((0..col_groups.len()).map(|_| value_label[0].clone()));
            if col_total_depth != 0 {
                row2.push(value_label[0].clone());
            }
            rows.push(row2);
        }
    } else {
        let mut header = vec![ArrayCellValue::EmptyCell; split_rows.array.shape().cols];
        header.extend(col_groups.iter().map(|group| group.key_cells[0].clone()));
        if col_total_depth != 0 {
            header.push(text_cell("Total"));
        }
        rows.push(header);
    }

    let value_cols = split_values.array.shape().cols;
    for row_group in &row_groups {
        let mut row = row_group.key_cells.clone();
        for col_group in &col_groups {
            let matches = find_intersection_rows(row_group, col_group);
            for value_col in 0..value_cols {
                let members = matches
                    .iter()
                    .map(|row_index| value_rows[*row_index][value_col].clone())
                    .collect::<Vec<_>>();
                row.push(invoke_group_aggregate(callable, &members, invoker)?);
            }
        }
        if row_total_depth != 0 {
            let members = row_group
                .row_indices
                .iter()
                .map(|row_index| value_rows[*row_index][0].clone())
                .collect::<Vec<_>>();
            row.push(invoke_group_aggregate(callable, &members, invoker)?);
        }
        rows.push(row);
    }

    if row_total_depth != 0 {
        let mut total_row = vec![ArrayCellValue::EmptyCell; split_rows.array.shape().cols];
        total_row[0] = text_cell("Total");
        for col_group in &col_groups {
            for value_col in 0..value_cols {
                let members = col_group
                    .row_indices
                    .iter()
                    .map(|row_index| value_rows[*row_index][value_col].clone())
                    .collect::<Vec<_>>();
                total_row.push(invoke_group_aggregate(callable, &members, invoker)?);
            }
        }
        if col_total_depth != 0 || row_total_depth != 0 {
            let members = value_rows
                .iter()
                .map(|row| row[0].clone())
                .collect::<Vec<_>>();
            total_row.push(invoke_group_aggregate(callable, &members, invoker)?);
        }
        rows.push(total_row);
    }

    Ok(EvalValue::Array(
        EvalArray::from_rows(rows).expect("pivotby output is rectangular"),
    ))
}

pub fn eval_pivotby_surface_ws(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
    invoker: &(impl CallableInvoker + ?Sized),
) -> Result<EvalValue, WorksheetErrorCode> {
    eval_pivotby_surface(args, resolver, invoker).map_err(|err| map_lambda_helper_error_to_ws(&err))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::functions::adapters::PreparedArgValue;
    use crate::functions::callable_helpers::{CallableInvocationError, CallableInvoker};
    use crate::resolver::{RefResolutionError, ResolverCapabilities};
    use crate::value::{
        CallableArityShape, CallableCaptureMode, ExcelText, LambdaValue, ReferenceLike,
    };

    struct NoResolver;
    struct TestInvoker;

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

    impl CallableInvoker for TestInvoker {
        fn invoke(
            &self,
            callable: &LambdaValue,
            args: &[PreparedArgValue],
        ) -> Result<PreparedArgValue, CallableInvocationError> {
            match callable.callable_token.as_str() {
                "helper.sum_array" => match &args[0] {
                    PreparedArgValue::Eval(EvalValue::Array(array)) => {
                        let total = array
                            .iter_row_major()
                            .filter_map(|cell| match cell {
                                ArrayCellValue::Number(n) => Some(*n),
                                _ => None,
                            })
                            .sum::<f64>();
                        Ok(PreparedArgValue::Eval(EvalValue::Number(total)))
                    }
                    _ => Err(CallableInvocationError::Worksheet(
                        WorksheetErrorCode::Value,
                    )),
                },
                other => Err(CallableInvocationError::UnsupportedCallableToken(
                    other.to_string(),
                )),
            }
        }
    }

    fn text(s: &str) -> ExcelText {
        ExcelText::from_utf16_code_units(s.encode_utf16().collect())
    }

    fn t(s: &str) -> ArrayCellValue {
        ArrayCellValue::Text(text(s))
    }

    fn callable() -> CallArgValue {
        CallArgValue::Eval(EvalValue::Lambda(LambdaValue::helper_lambda(
            "helper.sum_array",
            CallableArityShape::exact(1),
            CallableCaptureMode::NoCapture,
            "helper.sum_array.v1",
        )))
    }

    #[test]
    fn pivotby_meta_arity() {
        assert_eq!(PIVOTBY_META.arity.min, 4);
        assert_eq!(PIVOTBY_META.arity.max, 255);
    }

    #[test]
    fn pivotby_default_lane_matches_empirical_matrix() {
        let row_fields = EvalArray::from_rows(vec![
            vec![t("East")],
            vec![t("East")],
            vec![t("West")],
            vec![t("West")],
        ])
        .unwrap();
        let col_fields =
            EvalArray::from_rows(vec![vec![t("A")], vec![t("B")], vec![t("A")], vec![t("B")]])
                .unwrap();
        let values = EvalArray::from_rows(vec![
            vec![ArrayCellValue::Number(10.0)],
            vec![ArrayCellValue::Number(20.0)],
            vec![ArrayCellValue::Number(40.0)],
            vec![ArrayCellValue::Number(50.0)],
        ])
        .unwrap();
        let got = eval_pivotby_surface(
            &[
                CallArgValue::Eval(EvalValue::Array(row_fields)),
                CallArgValue::Eval(EvalValue::Array(col_fields)),
                CallArgValue::Eval(EvalValue::Array(values)),
                callable(),
            ],
            &NoResolver,
            &TestInvoker,
        )
        .unwrap();
        let EvalValue::Array(array) = got else {
            panic!("expected array");
        };
        let expected = EvalArray::from_rows(vec![
            vec![ArrayCellValue::EmptyCell, t("A"), t("B"), t("Total")],
            vec![
                t("East"),
                ArrayCellValue::Number(10.0),
                ArrayCellValue::Number(20.0),
                ArrayCellValue::Number(30.0),
            ],
            vec![
                t("West"),
                ArrayCellValue::Number(40.0),
                ArrayCellValue::Number(50.0),
                ArrayCellValue::Number(90.0),
            ],
            vec![
                t("Total"),
                ArrayCellValue::Number(50.0),
                ArrayCellValue::Number(70.0),
                ArrayCellValue::Number(120.0),
            ],
        ])
        .unwrap();
        assert_eq!(array, expected);
    }

    #[test]
    fn pivotby_supports_zero_totals_and_filter() {
        let row_fields = EvalArray::from_rows(vec![
            vec![t("East")],
            vec![t("East")],
            vec![t("West")],
            vec![t("West")],
        ])
        .unwrap();
        let col_fields =
            EvalArray::from_rows(vec![vec![t("A")], vec![t("B")], vec![t("A")], vec![t("B")]])
                .unwrap();
        let values = EvalArray::from_rows(vec![
            vec![ArrayCellValue::Number(10.0)],
            vec![ArrayCellValue::Number(20.0)],
            vec![ArrayCellValue::Number(40.0)],
            vec![ArrayCellValue::Number(50.0)],
        ])
        .unwrap();
        let filter = EvalArray::from_rows(vec![
            vec![ArrayCellValue::Logical(true)],
            vec![ArrayCellValue::Logical(false)],
            vec![ArrayCellValue::Logical(true)],
            vec![ArrayCellValue::Logical(false)],
        ])
        .unwrap();
        let got = eval_pivotby_surface(
            &[
                CallArgValue::Eval(EvalValue::Array(row_fields)),
                CallArgValue::Eval(EvalValue::Array(col_fields)),
                CallArgValue::Eval(EvalValue::Array(values)),
                callable(),
                CallArgValue::MissingArg,
                CallArgValue::Eval(EvalValue::Number(0.0)),
                CallArgValue::MissingArg,
                CallArgValue::Eval(EvalValue::Number(0.0)),
                CallArgValue::MissingArg,
                CallArgValue::Eval(EvalValue::Array(filter)),
            ],
            &NoResolver,
            &TestInvoker,
        )
        .unwrap();
        let EvalValue::Array(array) = got else {
            panic!("expected array");
        };
        let expected = EvalArray::from_rows(vec![
            vec![ArrayCellValue::EmptyCell, t("A")],
            vec![t("East"), ArrayCellValue::Number(10.0)],
            vec![t("West"), ArrayCellValue::Number(40.0)],
        ])
        .unwrap();
        assert_eq!(array, expected);
    }

    #[test]
    fn pivotby_supports_row_and_column_total_sort() {
        let row_fields = EvalArray::from_rows(vec![
            vec![t("East")],
            vec![t("East")],
            vec![t("West")],
            vec![t("West")],
        ])
        .unwrap();
        let col_fields =
            EvalArray::from_rows(vec![vec![t("A")], vec![t("B")], vec![t("A")], vec![t("B")]])
                .unwrap();
        let values = EvalArray::from_rows(vec![
            vec![ArrayCellValue::Number(10.0)],
            vec![ArrayCellValue::Number(20.0)],
            vec![ArrayCellValue::Number(40.0)],
            vec![ArrayCellValue::Number(50.0)],
        ])
        .unwrap();
        let got = eval_pivotby_surface(
            &[
                CallArgValue::Eval(EvalValue::Array(row_fields)),
                CallArgValue::Eval(EvalValue::Array(col_fields)),
                CallArgValue::Eval(EvalValue::Array(values)),
                callable(),
                CallArgValue::MissingArg,
                CallArgValue::MissingArg,
                CallArgValue::Eval(EvalValue::Number(-2.0)),
                CallArgValue::MissingArg,
                CallArgValue::Eval(EvalValue::Number(-2.0)),
            ],
            &NoResolver,
            &TestInvoker,
        )
        .unwrap();
        let EvalValue::Array(array) = got else {
            panic!("expected array");
        };
        let expected = EvalArray::from_rows(vec![
            vec![ArrayCellValue::EmptyCell, t("B"), t("A"), t("Total")],
            vec![
                t("West"),
                ArrayCellValue::Number(50.0),
                ArrayCellValue::Number(40.0),
                ArrayCellValue::Number(90.0),
            ],
            vec![
                t("East"),
                ArrayCellValue::Number(20.0),
                ArrayCellValue::Number(10.0),
                ArrayCellValue::Number(30.0),
            ],
            vec![
                t("Total"),
                ArrayCellValue::Number(70.0),
                ArrayCellValue::Number(50.0),
                ArrayCellValue::Number(120.0),
            ],
        ])
        .unwrap();
        assert_eq!(array, expected);
    }

    #[test]
    fn pivotby_visible_headers_emit_empirical_header_bands() {
        let row_fields =
            EvalArray::from_rows(vec![vec![t("Region")], vec![t("East")], vec![t("West")]])
                .unwrap();
        let col_fields =
            EvalArray::from_rows(vec![vec![t("Product")], vec![t("A")], vec![t("B")]]).unwrap();
        let values = EvalArray::from_rows(vec![
            vec![t("Sales")],
            vec![ArrayCellValue::Number(40.0)],
            vec![ArrayCellValue::Number(50.0)],
        ])
        .unwrap();
        let got = eval_pivotby_surface(
            &[
                CallArgValue::Eval(EvalValue::Array(row_fields)),
                CallArgValue::Eval(EvalValue::Array(col_fields)),
                CallArgValue::Eval(EvalValue::Array(values)),
                callable(),
                CallArgValue::Eval(EvalValue::Number(3.0)),
            ],
            &NoResolver,
            &TestInvoker,
        )
        .unwrap();
        let EvalValue::Array(array) = got else {
            panic!("expected array");
        };
        let expected = EvalArray::from_rows(vec![
            vec![
                ArrayCellValue::EmptyCell,
                t("Product"),
                ArrayCellValue::EmptyCell,
                ArrayCellValue::EmptyCell,
            ],
            vec![ArrayCellValue::EmptyCell, t("A"), t("B"), t("Total")],
            vec![t("Region"), t("Sales"), t("Sales"), t("Sales")],
            vec![
                t("East"),
                ArrayCellValue::Number(40.0),
                ArrayCellValue::Number(0.0),
                ArrayCellValue::Number(40.0),
            ],
            vec![
                t("West"),
                ArrayCellValue::Number(0.0),
                ArrayCellValue::Number(50.0),
                ArrayCellValue::Number(50.0),
            ],
            vec![
                t("Total"),
                ArrayCellValue::Number(40.0),
                ArrayCellValue::Number(50.0),
                ArrayCellValue::Number(90.0),
            ],
        ])
        .unwrap();
        assert_eq!(array, expected);
    }
}

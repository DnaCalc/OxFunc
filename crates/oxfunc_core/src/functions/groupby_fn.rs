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
    CellKey, FieldHeadersMode, FieldRelationship, default_row_field_headers, default_value_headers,
    group_indices_by_key, invoke_group_aggregate, key_from_cells, parse_field_headers_mode,
    parse_field_relationship, parse_filter_vector, parse_sort_orders, prepared_to_array,
    require_callable, row_as_cells, split_header_row, take_header_row, text_cell,
};
use crate::resolver::ReferenceResolver;
use crate::value::{ArrayCellValue, CallArgValue, EvalArray, EvalValue, WorksheetErrorCode};

pub const GROUPBY_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.GROUPBY",
    arity: Arity { min: 3, max: 255 },
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
        expected_min: GROUPBY_META.arity.min,
        expected_max: GROUPBY_META.arity.max,
        actual,
    }
}

#[derive(Debug, Clone)]
struct LeafGroup {
    key_cells: Vec<ArrayCellValue>,
    key: Vec<CellKey>,
    row_indices: Vec<usize>,
    aggregates: Vec<ArrayCellValue>,
}

fn header_mode_shows_output(
    mode: FieldHeadersMode,
    had_headers: bool,
    row_cols: usize,
    value_cols: usize,
) -> bool {
    match mode {
        FieldHeadersMode::YesShow | FieldHeadersMode::NoGenerate => true,
        FieldHeadersMode::YesHide | FieldHeadersMode::No => false,
        FieldHeadersMode::Auto => had_headers && (row_cols > 1 || value_cols > 1),
    }
}

fn parse_total_depth(
    prepared: Option<&PreparedArgValue>,
    relationship: FieldRelationship,
    row_cols: usize,
) -> Result<i32, LambdaHelperEvalError> {
    let depth = crate::functions::group_pivot_common::coerce_optional_i32(prepared)?.unwrap_or(1);
    if depth == 0 || depth == 1 || depth == 2 {
        if relationship == FieldRelationship::Tabular && depth > 1 && row_cols > 1 {
            return Err(LambdaHelperEvalError::Invocation(
                crate::functions::callable_helpers::CallableInvocationError::Worksheet(
                    WorksheetErrorCode::Value,
                ),
            ));
        }
        return Ok(depth);
    }
    Err(LambdaHelperEvalError::Invocation(
        crate::functions::callable_helpers::CallableInvocationError::Worksheet(
            WorksheetErrorCode::Value,
        ),
    ))
}

fn extract_filtered_rows(
    row_fields: &EvalArray,
    values: &EvalArray,
    filter: Option<&[bool]>,
) -> Result<(Vec<Vec<ArrayCellValue>>, Vec<Vec<ArrayCellValue>>), LambdaHelperEvalError> {
    if row_fields.shape().rows != values.shape().rows {
        return Err(LambdaHelperEvalError::Invocation(
            crate::functions::callable_helpers::CallableInvocationError::Worksheet(
                WorksheetErrorCode::Value,
            ),
        ));
    }
    let mut key_rows = Vec::new();
    let mut value_rows = Vec::new();
    for row in 0..row_fields.shape().rows {
        if filter.is_some_and(|keep| !keep[row]) {
            continue;
        }
        key_rows.push(row_as_cells(row_fields, row));
        value_rows.push(row_as_cells(values, row));
    }
    Ok((key_rows, value_rows))
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

fn apply_sort(groups: &mut [LeafGroup], sort_orders: &[i32], row_field_cols: usize) {
    if sort_orders.is_empty() {
        return;
    }
    groups.sort_by(|left, right| {
        for entry in sort_orders {
            let descending = *entry < 0;
            let ordinal = entry.unsigned_abs() as usize;
            let cmp = if ordinal == 0 {
                Ordering::Equal
            } else if ordinal <= row_field_cols {
                compare_cell(&left.key_cells[ordinal - 1], &right.key_cells[ordinal - 1])
            } else {
                let idx = ordinal - row_field_cols - 1;
                if idx >= left.aggregates.len() || idx >= right.aggregates.len() {
                    Ordering::Equal
                } else {
                    compare_cell(&left.aggregates[idx], &right.aggregates[idx])
                }
            };
            if cmp != Ordering::Equal {
                return if descending { cmp.reverse() } else { cmp };
            }
        }
        Ordering::Equal
    });
}

fn build_leaf_groups(
    key_rows: &[Vec<ArrayCellValue>],
    value_rows: &[Vec<ArrayCellValue>],
    callable: &crate::value::LambdaValue,
    invoker: &(impl CallableInvoker + ?Sized),
) -> Result<Vec<LeafGroup>, LambdaHelperEvalError> {
    if key_rows.is_empty() {
        return Ok(Vec::new());
    }

    let keys = key_rows
        .iter()
        .map(|row| key_from_cells(row))
        .collect::<Vec<_>>();
    let grouped = group_indices_by_key(key_rows.len(), &keys);
    let value_cols = value_rows.first().map_or(1, Vec::len);

    grouped
        .into_iter()
        .map(|row_indices| {
            let first = row_indices[0];
            let mut aggregates = Vec::with_capacity(value_cols);
            for value_col in 0..value_cols {
                let members = row_indices
                    .iter()
                    .map(|row| value_rows[*row][value_col].clone())
                    .collect::<Vec<_>>();
                aggregates.push(invoke_group_aggregate(callable, &members, invoker)?);
            }
            Ok(LeafGroup {
                key_cells: key_rows[first].clone(),
                key: keys[first].clone(),
                row_indices,
                aggregates,
            })
        })
        .collect()
}

fn subtotal_row(
    prefix: &[ArrayCellValue],
    total_cols: usize,
    aggregates: Vec<ArrayCellValue>,
) -> Vec<ArrayCellValue> {
    let mut row = vec![ArrayCellValue::EmptyCell; total_cols];
    for (idx, cell) in prefix.iter().enumerate() {
        row[idx] = cell.clone();
    }
    for (idx, cell) in aggregates.into_iter().enumerate() {
        row[total_cols - idx - 1] = cell;
    }
    row
}

fn build_output_rows(
    groups: &[LeafGroup],
    value_rows: &[Vec<ArrayCellValue>],
    callable: &crate::value::LambdaValue,
    invoker: &(impl CallableInvoker + ?Sized),
    total_depth: i32,
    relationship: FieldRelationship,
) -> Result<Vec<Vec<ArrayCellValue>>, LambdaHelperEvalError> {
    if groups.is_empty() {
        return Ok(vec![vec![ArrayCellValue::Error(WorksheetErrorCode::Calc)]]);
    }

    let row_field_cols = groups[0].key_cells.len();
    let value_cols = groups[0].aggregates.len();
    let total_cols = row_field_cols + value_cols;
    let mut rows = Vec::new();

    for group in groups {
        let mut row = group.key_cells.clone();
        row.extend(group.aggregates.clone());
        rows.push(row);
    }

    if total_depth >= 2 && relationship == FieldRelationship::Hierarchical && row_field_cols > 1 {
        let mut expanded = Vec::new();
        let mut cursor = 0;
        while cursor < groups.len() {
            let prefix = groups[cursor].key[..1].to_vec();
            let start = cursor;
            while cursor < groups.len() && groups[cursor].key[..1] == prefix[..] {
                let mut row = groups[cursor].key_cells.clone();
                row.extend(groups[cursor].aggregates.clone());
                expanded.push(row);
                cursor += 1;
            }

            let subtotal_indices = groups[start..cursor]
                .iter()
                .flat_map(|group| group.row_indices.iter().copied())
                .collect::<Vec<_>>();
            let subtotal_values = (0..value_cols)
                .map(|value_col| {
                    let members = subtotal_indices
                        .iter()
                        .map(|row| value_rows[*row][value_col].clone())
                        .collect::<Vec<_>>();
                    invoke_group_aggregate(callable, &members, invoker)
                })
                .collect::<Result<Vec<_>, _>>()?;
            let prefix_cells = vec![groups[start].key_cells[0].clone()];
            let mut subtotal = subtotal_row(&prefix_cells, total_cols, subtotal_values);
            if row_field_cols > 1 {
                subtotal[1] = ArrayCellValue::EmptyCell;
            }
            expanded.push(subtotal);
        }
        rows = expanded;
    }

    if total_depth != 0 {
        let grand_indices = groups
            .iter()
            .flat_map(|group| group.row_indices.iter().copied())
            .collect::<Vec<_>>();
        let grand_values = (0..value_cols)
            .map(|value_col| {
                let members = grand_indices
                    .iter()
                    .map(|row| value_rows[*row][value_col].clone())
                    .collect::<Vec<_>>();
                invoke_group_aggregate(callable, &members, invoker)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let label = if total_depth >= 2 && row_field_cols > 1 {
            "Grand Total"
        } else {
            "Total"
        };
        let mut grand = vec![ArrayCellValue::EmptyCell; total_cols];
        grand[0] = text_cell(label);
        for (idx, cell) in grand_values.into_iter().enumerate() {
            grand[row_field_cols + idx] = cell;
        }
        rows.push(grand);
    }

    Ok(rows)
}

pub fn eval_groupby_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
    invoker: &(impl CallableInvoker + ?Sized),
) -> Result<EvalValue, LambdaHelperEvalError> {
    if !GROUPBY_META.arity.accepts(args.len()) {
        return Err(surface_arity_error(args.len()));
    }

    let prepared =
        prepare_args_values_only(args, resolver).map_err(LambdaHelperEvalError::Preparation)?;
    let callable = require_callable(&prepared[2])?;

    let field_headers_mode = parse_field_headers_mode(prepared.get(3))?;
    let relationship = parse_field_relationship(prepared.get(7))?;

    let raw_row_fields = prepared_to_array(&prepared[0]);
    let raw_values = prepared_to_array(&prepared[1]);
    let row_headers = take_header_row(&raw_row_fields);
    let value_headers = take_header_row(&raw_values);
    let split_rows = split_header_row(&raw_row_fields, field_headers_mode)?;
    let split_values = split_header_row(&raw_values, field_headers_mode)?;

    let total_depth =
        parse_total_depth(prepared.get(4), relationship, split_rows.array.shape().cols)?;
    let sort_orders = parse_sort_orders(prepared.get(5))?;
    let filter = parse_filter_vector(prepared.get(6), split_rows.array.shape().rows)?;

    let (key_rows, value_rows) =
        extract_filtered_rows(&split_rows.array, &split_values.array, filter.as_deref())?;
    let mut groups = build_leaf_groups(&key_rows, &value_rows, callable, invoker)?;
    apply_sort(&mut groups, &sort_orders, split_rows.array.shape().cols);

    let mut rows = Vec::new();
    if header_mode_shows_output(
        field_headers_mode,
        split_rows.had_headers,
        split_rows.array.shape().cols,
        split_values.array.shape().cols,
    ) {
        let mut header = if matches!(field_headers_mode, FieldHeadersMode::NoGenerate) {
            default_row_field_headers(split_rows.array.shape().cols)
        } else if split_rows.had_headers {
            row_headers
        } else {
            default_row_field_headers(split_rows.array.shape().cols)
        };
        if matches!(field_headers_mode, FieldHeadersMode::NoGenerate) {
            header.extend(default_value_headers(split_values.array.shape().cols));
        } else if split_values.had_headers {
            header.extend(value_headers);
        } else {
            header.extend(default_value_headers(split_values.array.shape().cols));
        }
        rows.push(header);
    }

    rows.extend(build_output_rows(
        &groups,
        &value_rows,
        callable,
        invoker,
        total_depth,
        relationship,
    )?);

    Ok(EvalValue::Array(
        EvalArray::from_rows(rows).expect("groupby output is rectangular"),
    ))
}

pub fn eval_groupby_surface_ws(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
    invoker: &(impl CallableInvoker + ?Sized),
) -> Result<EvalValue, WorksheetErrorCode> {
    eval_groupby_surface(args, resolver, invoker).map_err(|err| map_lambda_helper_error_to_ws(&err))
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
                        let mut total = 0.0;
                        for cell in array.iter_row_major() {
                            match cell {
                                ArrayCellValue::Number(n) => total += n,
                                ArrayCellValue::Text(_) | ArrayCellValue::EmptyCell => {}
                                ArrayCellValue::Logical(_) => {
                                    return Err(CallableInvocationError::Worksheet(
                                        WorksheetErrorCode::Value,
                                    ));
                                }
                                ArrayCellValue::Error(code) => {
                                    return Err(CallableInvocationError::Worksheet(*code));
                                }
                            }
                        }
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

    fn callable() -> CallArgValue {
        CallArgValue::Eval(EvalValue::Lambda(LambdaValue::helper_lambda(
            "helper.sum_array",
            CallableArityShape::exact(1),
            CallableCaptureMode::NoCapture,
            "helper.sum_array.v1",
        )))
    }

    fn text_cell_value(s: &str) -> ArrayCellValue {
        ArrayCellValue::Text(text(s))
    }

    #[test]
    fn groupby_meta_arity() {
        assert_eq!(GROUPBY_META.arity.min, 3);
        assert_eq!(GROUPBY_META.arity.max, 255);
    }

    #[test]
    fn groupby_default_lane_matches_empirical_single_axis_sum() {
        let row_fields = EvalArray::from_rows(vec![
            vec![text_cell_value("2024")],
            vec![text_cell_value("2024")],
            vec![text_cell_value("2025")],
            vec![text_cell_value("2025")],
        ])
        .unwrap();
        let values = EvalArray::from_rows(vec![
            vec![ArrayCellValue::Number(10.0)],
            vec![ArrayCellValue::Number(20.0)],
            vec![ArrayCellValue::Number(30.0)],
            vec![ArrayCellValue::Number(40.0)],
        ])
        .unwrap();
        let got = eval_groupby_surface(
            &[
                CallArgValue::Eval(EvalValue::Array(row_fields)),
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
            vec![text_cell_value("2024"), ArrayCellValue::Number(30.0)],
            vec![text_cell_value("2025"), ArrayCellValue::Number(70.0)],
            vec![text_cell_value("Total"), ArrayCellValue::Number(100.0)],
        ])
        .unwrap();
        assert_eq!(array, expected);
    }

    #[test]
    fn groupby_subtotals_match_empirical_two_level_hierarchical_lane() {
        let row_fields = EvalArray::from_rows(vec![
            vec![text_cell_value("East"), text_cell_value("A")],
            vec![text_cell_value("East"), text_cell_value("B")],
            vec![text_cell_value("East"), text_cell_value("A")],
            vec![text_cell_value("West"), text_cell_value("A")],
            vec![text_cell_value("West"), text_cell_value("B")],
        ])
        .unwrap();
        let values = EvalArray::from_rows(vec![
            vec![ArrayCellValue::Number(10.0)],
            vec![ArrayCellValue::Number(20.0)],
            vec![ArrayCellValue::Number(30.0)],
            vec![ArrayCellValue::Number(40.0)],
            vec![ArrayCellValue::Number(50.0)],
        ])
        .unwrap();
        let got = eval_groupby_surface(
            &[
                CallArgValue::Eval(EvalValue::Array(row_fields)),
                CallArgValue::Eval(EvalValue::Array(values)),
                callable(),
                CallArgValue::MissingArg,
                CallArgValue::Eval(EvalValue::Number(2.0)),
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
                text_cell_value("East"),
                text_cell_value("A"),
                ArrayCellValue::Number(40.0),
            ],
            vec![
                text_cell_value("East"),
                text_cell_value("B"),
                ArrayCellValue::Number(20.0),
            ],
            vec![
                text_cell_value("East"),
                ArrayCellValue::EmptyCell,
                ArrayCellValue::Number(60.0),
            ],
            vec![
                text_cell_value("West"),
                text_cell_value("A"),
                ArrayCellValue::Number(40.0),
            ],
            vec![
                text_cell_value("West"),
                text_cell_value("B"),
                ArrayCellValue::Number(50.0),
            ],
            vec![
                text_cell_value("West"),
                ArrayCellValue::EmptyCell,
                ArrayCellValue::Number(90.0),
            ],
            vec![
                text_cell_value("Grand Total"),
                ArrayCellValue::EmptyCell,
                ArrayCellValue::Number(150.0),
            ],
        ])
        .unwrap();
        assert_eq!(array, expected);
    }

    #[test]
    fn groupby_supports_filter_and_descending_value_sort() {
        let row_fields = EvalArray::from_rows(vec![
            vec![text_cell_value("A")],
            vec![text_cell_value("B")],
            vec![text_cell_value("A")],
            vec![text_cell_value("B")],
        ])
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
        let got = eval_groupby_surface(
            &[
                CallArgValue::Eval(EvalValue::Array(row_fields)),
                CallArgValue::Eval(EvalValue::Array(values)),
                callable(),
                CallArgValue::MissingArg,
                CallArgValue::MissingArg,
                CallArgValue::Eval(EvalValue::Number(-2.0)),
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
            vec![text_cell_value("A"), ArrayCellValue::Number(50.0)],
            vec![text_cell_value("Total"), ArrayCellValue::Number(50.0)],
        ])
        .unwrap();
        assert_eq!(array, expected);
    }

    #[test]
    fn groupby_with_visible_headers_emits_header_row() {
        let row_fields = EvalArray::from_rows(vec![
            vec![text_cell_value("Region"), text_cell_value("Product")],
            vec![text_cell_value("East"), text_cell_value("A")],
            vec![text_cell_value("East"), text_cell_value("B")],
        ])
        .unwrap();
        let values = EvalArray::from_rows(vec![
            vec![text_cell_value("Sales")],
            vec![ArrayCellValue::Number(10.0)],
            vec![ArrayCellValue::Number(20.0)],
        ])
        .unwrap();
        let got = eval_groupby_surface(
            &[
                CallArgValue::Eval(EvalValue::Array(row_fields)),
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
                text_cell_value("Region"),
                text_cell_value("Product"),
                text_cell_value("Sales"),
            ],
            vec![
                text_cell_value("East"),
                text_cell_value("A"),
                ArrayCellValue::Number(10.0),
            ],
            vec![
                text_cell_value("East"),
                text_cell_value("B"),
                ArrayCellValue::Number(20.0),
            ],
            vec![
                text_cell_value("Total"),
                ArrayCellValue::EmptyCell,
                ArrayCellValue::Number(30.0),
            ],
        ])
        .unwrap();
        assert_eq!(array, expected);
    }

    #[test]
    fn groupby_tabular_rejects_subtotals() {
        let row_fields = EvalArray::from_rows(vec![
            vec![text_cell_value("East"), text_cell_value("A")],
            vec![text_cell_value("East"), text_cell_value("B")],
        ])
        .unwrap();
        let values = EvalArray::from_rows(vec![
            vec![ArrayCellValue::Number(10.0)],
            vec![ArrayCellValue::Number(20.0)],
        ])
        .unwrap();
        let got = eval_groupby_surface_ws(
            &[
                CallArgValue::Eval(EvalValue::Array(row_fields)),
                CallArgValue::Eval(EvalValue::Array(values)),
                callable(),
                CallArgValue::MissingArg,
                CallArgValue::Eval(EvalValue::Number(2.0)),
                CallArgValue::MissingArg,
                CallArgValue::MissingArg,
                CallArgValue::Eval(EvalValue::Number(1.0)),
            ],
            &NoResolver,
            &TestInvoker,
        );
        assert_eq!(got, Err(WorksheetErrorCode::Value));
    }
}

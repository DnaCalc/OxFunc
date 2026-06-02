use std::cmp::Ordering;

use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    PreparedValue, prepare_args_values_only, prepare_calc_values_only, prepared_from_calc_value,
};
use crate::functions::callable_helpers::{
    CallableInvoker, LambdaHelperEvalError, map_lambda_helper_error_to_ws,
};
use crate::functions::group_pivot_common::{
    CellKey, FieldHeadersMode, FieldRelationship, default_row_field_headers, default_value_headers,
    group_indices_by_key, invoke_group_aggregate, key_from_cells, parse_field_headers_mode,
    parse_field_relationship, parse_filter_vector, parse_sort_orders, prepared_to_array,
    require_calc_callable, require_callable, row_as_cells, split_header_row, take_header_row,
    text_cell,
};
use crate::resolver::ReferenceSystemProvider;
use crate::value::{
    CalcValue, CallableValue, FunctionArg, FunctionArray, FunctionArrayCell, FunctionValue,
    WorksheetErrorCode,
};

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
    key_cells: Vec<FunctionArrayCell>,
    key: Vec<CellKey>,
    row_indices: Vec<usize>,
    aggregates: Vec<FunctionArrayCell>,
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
    prepared: Option<&PreparedValue>,
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
    row_fields: &FunctionArray,
    values: &FunctionArray,
    filter: Option<&[bool]>,
) -> Result<(Vec<Vec<FunctionArrayCell>>, Vec<Vec<FunctionArrayCell>>), LambdaHelperEvalError> {
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

fn compare_cell(a: &FunctionArrayCell, b: &FunctionArrayCell) -> Ordering {
    match (a, b) {
        (FunctionArrayCell::Number(x), FunctionArrayCell::Number(y)) => {
            x.partial_cmp(y).unwrap_or(Ordering::Equal)
        }
        (FunctionArrayCell::Text(x), FunctionArrayCell::Text(y)) => {
            x.to_string_lossy().cmp(&y.to_string_lossy())
        }
        (FunctionArrayCell::Logical(x), FunctionArrayCell::Logical(y)) => x.cmp(y),
        (FunctionArrayCell::EmptyCell, FunctionArrayCell::EmptyCell) => Ordering::Equal,
        (FunctionArrayCell::Error(x), FunctionArrayCell::Error(y)) => (*x as u8).cmp(&(*y as u8)),
        _ => sort_rank(a).cmp(&sort_rank(b)),
    }
}

fn sort_rank(cell: &FunctionArrayCell) -> u8 {
    match cell {
        FunctionArrayCell::Number(_) => 0,
        FunctionArrayCell::Text(_) => 1,
        FunctionArrayCell::Logical(_) => 2,
        FunctionArrayCell::Error(_) => 3,
        FunctionArrayCell::EmptyCell => 4,
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
    key_rows: &[Vec<FunctionArrayCell>],
    value_rows: &[Vec<FunctionArrayCell>],
    callable: &CallableValue,
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
    prefix: &[FunctionArrayCell],
    total_cols: usize,
    aggregates: Vec<FunctionArrayCell>,
) -> Vec<FunctionArrayCell> {
    let mut row = vec![FunctionArrayCell::EmptyCell; total_cols];
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
    value_rows: &[Vec<FunctionArrayCell>],
    callable: &CallableValue,
    invoker: &(impl CallableInvoker + ?Sized),
    total_depth: i32,
    relationship: FieldRelationship,
) -> Result<Vec<Vec<FunctionArrayCell>>, LambdaHelperEvalError> {
    if groups.is_empty() {
        return Ok(vec![vec![FunctionArrayCell::Error(
            WorksheetErrorCode::Calc,
        )]]);
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
                subtotal[1] = FunctionArrayCell::EmptyCell;
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
        let mut grand = vec![FunctionArrayCell::EmptyCell; total_cols];
        grand[0] = text_cell(label);
        for (idx, cell) in grand_values.into_iter().enumerate() {
            grand[row_field_cols + idx] = cell;
        }
        rows.push(grand);
    }

    Ok(rows)
}

pub fn eval_groupby_calc_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
    invoker: &(impl CallableInvoker + ?Sized),
) -> Result<FunctionValue, LambdaHelperEvalError> {
    if !GROUPBY_META.arity.accepts(args.len()) {
        return Err(surface_arity_error(args.len()));
    }

    let prepared_calc =
        prepare_calc_values_only(args, resolver).map_err(LambdaHelperEvalError::Preparation)?;
    let callable = require_calc_callable(&prepared_calc[2])?;
    let prepared: Vec<PreparedValue> = prepared_calc.iter().map(prepared_from_calc_value).collect();

    eval_groupby_prepared(&prepared, &callable, invoker)
}

pub fn eval_groupby_surface(
    args: &[FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
    invoker: &(impl CallableInvoker + ?Sized),
) -> Result<FunctionValue, LambdaHelperEvalError> {
    if !GROUPBY_META.arity.accepts(args.len()) {
        return Err(surface_arity_error(args.len()));
    }

    let prepared =
        prepare_args_values_only(args, resolver).map_err(LambdaHelperEvalError::Preparation)?;
    let callable = require_callable(&prepared[2])?;

    eval_groupby_prepared(&prepared, &callable, invoker)
}

fn eval_groupby_prepared(
    prepared: &[PreparedValue],
    callable: &CallableValue,
    invoker: &(impl CallableInvoker + ?Sized),
) -> Result<FunctionValue, LambdaHelperEvalError> {
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
        &callable,
        invoker,
        total_depth,
        relationship,
    )?);

    Ok(FunctionValue::Array(
        FunctionArray::from_rows(rows).expect("groupby output is rectangular"),
    ))
}

pub fn eval_groupby_surface_ws(
    args: &[FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
    invoker: &(impl CallableInvoker + ?Sized),
) -> Result<FunctionValue, WorksheetErrorCode> {
    eval_groupby_surface(args, resolver, invoker).map_err(|err| map_lambda_helper_error_to_ws(&err))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;

    use crate::functions::adapters::PreparedValue;
    use crate::functions::callable_helpers::{CallableInvocationError, CallableInvoker};
    use crate::resolver::ReferenceSystemCapabilities;
    use crate::value::{CallableArityShape, ExcelText, OpaqueCallable};

    struct NoResolver;
    struct TestInvoker;
    #[derive(Debug)]
    struct TestCallableHandle;

    impl OpaqueCallable for TestCallableHandle {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

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

    impl CallableInvoker for TestInvoker {
        fn invoke(
            &self,
            callable: &CallableValue,
            args: &[PreparedValue],
        ) -> Result<PreparedValue, CallableInvocationError> {
            match callable.summary.as_str() {
                "helper.sum_array" => match &args[0] {
                    PreparedValue::Eval(FunctionValue::Array(array)) => {
                        let mut total = 0.0;
                        for cell in array.iter_row_major() {
                            match cell {
                                FunctionArrayCell::Number(n) => total += n,
                                FunctionArrayCell::Text(_) | FunctionArrayCell::EmptyCell => {}
                                FunctionArrayCell::Logical(_) => {
                                    return Err(CallableInvocationError::Worksheet(
                                        WorksheetErrorCode::Value,
                                    ));
                                }
                                FunctionArrayCell::Error(code) => {
                                    return Err(CallableInvocationError::Worksheet(*code));
                                }
                            }
                        }
                        Ok(PreparedValue::Eval(FunctionValue::Number(total)))
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

    fn callable_arg() -> CalcValue {
        CalcValue::callable(CallableValue {
            arity: CallableArityShape::exact(1),
            summary: "helper.sum_array".to_string(),
            handle: Rc::new(TestCallableHandle),
        })
    }

    fn text_cell_value(s: &str) -> FunctionArrayCell {
        FunctionArrayCell::Text(text(s))
    }

    #[test]
    fn groupby_meta_arity() {
        assert_eq!(GROUPBY_META.arity.min, 3);
        assert_eq!(GROUPBY_META.arity.max, 255);
    }

    #[test]
    fn groupby_default_lane_matches_empirical_single_axis_sum() {
        let row_fields = FunctionArray::from_rows(vec![
            vec![text_cell_value("2024")],
            vec![text_cell_value("2024")],
            vec![text_cell_value("2025")],
            vec![text_cell_value("2025")],
        ])
        .unwrap();
        let values = FunctionArray::from_rows(vec![
            vec![FunctionArrayCell::Number(10.0)],
            vec![FunctionArrayCell::Number(20.0)],
            vec![FunctionArrayCell::Number(30.0)],
            vec![FunctionArrayCell::Number(40.0)],
        ])
        .unwrap();
        let got = eval_groupby_calc_surface(
            &[
                CalcValue::from(FunctionValue::Array(row_fields)),
                CalcValue::from(FunctionValue::Array(values)),
                callable_arg(),
            ],
            &NoResolver,
            &TestInvoker,
        )
        .unwrap();
        let FunctionValue::Array(array) = got else {
            panic!("expected array");
        };
        let expected = FunctionArray::from_rows(vec![
            vec![text_cell_value("2024"), FunctionArrayCell::Number(30.0)],
            vec![text_cell_value("2025"), FunctionArrayCell::Number(70.0)],
            vec![text_cell_value("Total"), FunctionArrayCell::Number(100.0)],
        ])
        .unwrap();
        assert_eq!(array, expected);
    }

    #[test]
    fn groupby_subtotals_match_empirical_two_level_hierarchical_lane() {
        let row_fields = FunctionArray::from_rows(vec![
            vec![text_cell_value("East"), text_cell_value("A")],
            vec![text_cell_value("East"), text_cell_value("B")],
            vec![text_cell_value("East"), text_cell_value("A")],
            vec![text_cell_value("West"), text_cell_value("A")],
            vec![text_cell_value("West"), text_cell_value("B")],
        ])
        .unwrap();
        let values = FunctionArray::from_rows(vec![
            vec![FunctionArrayCell::Number(10.0)],
            vec![FunctionArrayCell::Number(20.0)],
            vec![FunctionArrayCell::Number(30.0)],
            vec![FunctionArrayCell::Number(40.0)],
            vec![FunctionArrayCell::Number(50.0)],
        ])
        .unwrap();
        let got = eval_groupby_calc_surface(
            &[
                CalcValue::from(FunctionValue::Array(row_fields)),
                CalcValue::from(FunctionValue::Array(values)),
                callable_arg(),
                CalcValue::missing(),
                CalcValue::number(2.0),
            ],
            &NoResolver,
            &TestInvoker,
        )
        .unwrap();
        let FunctionValue::Array(array) = got else {
            panic!("expected array");
        };
        let expected = FunctionArray::from_rows(vec![
            vec![
                text_cell_value("East"),
                text_cell_value("A"),
                FunctionArrayCell::Number(40.0),
            ],
            vec![
                text_cell_value("East"),
                text_cell_value("B"),
                FunctionArrayCell::Number(20.0),
            ],
            vec![
                text_cell_value("East"),
                FunctionArrayCell::EmptyCell,
                FunctionArrayCell::Number(60.0),
            ],
            vec![
                text_cell_value("West"),
                text_cell_value("A"),
                FunctionArrayCell::Number(40.0),
            ],
            vec![
                text_cell_value("West"),
                text_cell_value("B"),
                FunctionArrayCell::Number(50.0),
            ],
            vec![
                text_cell_value("West"),
                FunctionArrayCell::EmptyCell,
                FunctionArrayCell::Number(90.0),
            ],
            vec![
                text_cell_value("Grand Total"),
                FunctionArrayCell::EmptyCell,
                FunctionArrayCell::Number(150.0),
            ],
        ])
        .unwrap();
        assert_eq!(array, expected);
    }

    #[test]
    fn groupby_supports_filter_and_descending_value_sort() {
        let row_fields = FunctionArray::from_rows(vec![
            vec![text_cell_value("A")],
            vec![text_cell_value("B")],
            vec![text_cell_value("A")],
            vec![text_cell_value("B")],
        ])
        .unwrap();
        let values = FunctionArray::from_rows(vec![
            vec![FunctionArrayCell::Number(10.0)],
            vec![FunctionArrayCell::Number(20.0)],
            vec![FunctionArrayCell::Number(40.0)],
            vec![FunctionArrayCell::Number(50.0)],
        ])
        .unwrap();
        let filter = FunctionArray::from_rows(vec![
            vec![FunctionArrayCell::Logical(true)],
            vec![FunctionArrayCell::Logical(false)],
            vec![FunctionArrayCell::Logical(true)],
            vec![FunctionArrayCell::Logical(false)],
        ])
        .unwrap();
        let got = eval_groupby_calc_surface(
            &[
                CalcValue::from(FunctionValue::Array(row_fields)),
                CalcValue::from(FunctionValue::Array(values)),
                callable_arg(),
                CalcValue::missing(),
                CalcValue::missing(),
                CalcValue::number(-2.0),
                CalcValue::from(FunctionValue::Array(filter)),
            ],
            &NoResolver,
            &TestInvoker,
        )
        .unwrap();
        let FunctionValue::Array(array) = got else {
            panic!("expected array");
        };
        let expected = FunctionArray::from_rows(vec![
            vec![text_cell_value("A"), FunctionArrayCell::Number(50.0)],
            vec![text_cell_value("Total"), FunctionArrayCell::Number(50.0)],
        ])
        .unwrap();
        assert_eq!(array, expected);
    }

    #[test]
    fn groupby_with_visible_headers_emits_header_row() {
        let row_fields = FunctionArray::from_rows(vec![
            vec![text_cell_value("Region"), text_cell_value("Product")],
            vec![text_cell_value("East"), text_cell_value("A")],
            vec![text_cell_value("East"), text_cell_value("B")],
        ])
        .unwrap();
        let values = FunctionArray::from_rows(vec![
            vec![text_cell_value("Sales")],
            vec![FunctionArrayCell::Number(10.0)],
            vec![FunctionArrayCell::Number(20.0)],
        ])
        .unwrap();
        let got = eval_groupby_calc_surface(
            &[
                CalcValue::from(FunctionValue::Array(row_fields)),
                CalcValue::from(FunctionValue::Array(values)),
                callable_arg(),
                CalcValue::number(3.0),
            ],
            &NoResolver,
            &TestInvoker,
        )
        .unwrap();
        let FunctionValue::Array(array) = got else {
            panic!("expected array");
        };
        let expected = FunctionArray::from_rows(vec![
            vec![
                text_cell_value("Region"),
                text_cell_value("Product"),
                text_cell_value("Sales"),
            ],
            vec![
                text_cell_value("East"),
                text_cell_value("A"),
                FunctionArrayCell::Number(10.0),
            ],
            vec![
                text_cell_value("East"),
                text_cell_value("B"),
                FunctionArrayCell::Number(20.0),
            ],
            vec![
                text_cell_value("Total"),
                FunctionArrayCell::EmptyCell,
                FunctionArrayCell::Number(30.0),
            ],
        ])
        .unwrap();
        assert_eq!(array, expected);
    }

    #[test]
    fn groupby_tabular_rejects_subtotals() {
        let row_fields = FunctionArray::from_rows(vec![
            vec![text_cell_value("East"), text_cell_value("A")],
            vec![text_cell_value("East"), text_cell_value("B")],
        ])
        .unwrap();
        let values = FunctionArray::from_rows(vec![
            vec![FunctionArrayCell::Number(10.0)],
            vec![FunctionArrayCell::Number(20.0)],
        ])
        .unwrap();
        let got = eval_groupby_calc_surface(
            &[
                CalcValue::from(FunctionValue::Array(row_fields)),
                CalcValue::from(FunctionValue::Array(values)),
                callable_arg(),
                CalcValue::missing(),
                CalcValue::number(2.0),
                CalcValue::missing(),
                CalcValue::missing(),
                CalcValue::number(1.0),
            ],
            &NoResolver,
            &TestInvoker,
        )
        .map_err(|err| map_lambda_helper_error_to_ws(&err));
        assert_eq!(got, Err(WorksheetErrorCode::Value));
    }
}

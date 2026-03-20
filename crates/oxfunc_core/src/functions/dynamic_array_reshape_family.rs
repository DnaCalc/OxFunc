use std::cmp::Ordering;
use std::collections::HashMap;

use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    PreparedArgValue, coerce_prepared_to_number, run_values_only_prepared,
};
use crate::resolver::ReferenceResolver;
use crate::value::{
    ArrayCellValue, ArrayShape, CallArgValue, EvalArray, EvalValue, WorksheetErrorCode,
};

macro_rules! reshape_meta {
    ($id:literal, $min:expr, $max:expr) => {
        FunctionMeta {
            function_id: $id,
            arity: Arity {
                min: $min,
                max: $max,
            },
            determinism: DeterminismClass::Deterministic,
            volatility: VolatilityClass::NonVolatile,
            host_interaction: HostInteractionClass::None,
            thread_safety: ThreadSafetyClass::SafePure,
            arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
            coercion_lift_profile: CoercionLiftProfile::Custom,
            kernel_signature_class: KernelSignatureClass::Custom,
            fec_dependency_profile: FecDependencyProfile::None,
            surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
        }
    };
}

pub const CHOOSECOLS_META: FunctionMeta = reshape_meta!("FUNC.CHOOSECOLS", 2, 255);
pub const CHOOSEROWS_META: FunctionMeta = reshape_meta!("FUNC.CHOOSEROWS", 2, 255);
pub const DROP_META: FunctionMeta = reshape_meta!("FUNC.DROP", 2, 3);
pub const EXPAND_META: FunctionMeta = reshape_meta!("FUNC.EXPAND", 2, 4);
pub const FILTER_META: FunctionMeta = reshape_meta!("FUNC.FILTER", 2, 3);
pub const SORT_META: FunctionMeta = reshape_meta!("FUNC.SORT", 1, 4);
pub const SORTBY_META: FunctionMeta = reshape_meta!("FUNC.SORTBY", 2, 30);
pub const TAKE_META: FunctionMeta = reshape_meta!("FUNC.TAKE", 2, 3);
pub const TOCOL_META: FunctionMeta = reshape_meta!("FUNC.TOCOL", 1, 3);
pub const TOROW_META: FunctionMeta = reshape_meta!("FUNC.TOROW", 1, 3);
pub const TRANSPOSE_META: FunctionMeta = reshape_meta!("FUNC.TRANSPOSE", 1, 1);
pub const UNIQUE_META: FunctionMeta = reshape_meta!("FUNC.UNIQUE", 1, 3);
pub const VSTACK_META: FunctionMeta = reshape_meta!("FUNC.VSTACK", 1, 255);
pub const WRAPCOLS_META: FunctionMeta = reshape_meta!("FUNC.WRAPCOLS", 2, 3);
pub const WRAPROWS_META: FunctionMeta = reshape_meta!("FUNC.WRAPROWS", 2, 3);

#[derive(Debug, Clone, PartialEq)]
pub enum DynamicArrayReshapeEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Preparation(CoercionError),
    InvalidSelector,
    InvalidCount,
    EmptyArrayResult,
    DimensionTooSmall,
    InvalidIgnoreMode,
    InvalidSortOrder,
    InvalidSortIndex,
    InvalidIncludeShape,
}

fn scalar_cell(arg: &PreparedArgValue) -> ArrayCellValue {
    match arg {
        PreparedArgValue::Eval(EvalValue::Number(n)) => ArrayCellValue::Number(*n),
        PreparedArgValue::Eval(EvalValue::Text(t)) => ArrayCellValue::Text(t.clone()),
        PreparedArgValue::Eval(EvalValue::Logical(b)) => ArrayCellValue::Logical(*b),
        PreparedArgValue::Eval(EvalValue::Error(code)) => ArrayCellValue::Error(*code),
        PreparedArgValue::Eval(EvalValue::Reference(_))
        | PreparedArgValue::Eval(EvalValue::Lambda(_)) => {
            ArrayCellValue::Error(WorksheetErrorCode::Value)
        }
        PreparedArgValue::Eval(EvalValue::Array(_)) => unreachable!(),
        PreparedArgValue::MissingArg | PreparedArgValue::EmptyCell => ArrayCellValue::EmptyCell,
    }
}

fn materialize_array_arg(arg: &PreparedArgValue) -> EvalArray {
    match arg {
        PreparedArgValue::Eval(EvalValue::Array(array)) => array.clone(),
        other => EvalArray::from_scalar(scalar_cell(other)),
    }
}

fn parse_integer(prepared: &PreparedArgValue) -> Result<isize, DynamicArrayReshapeEvalError> {
    let raw =
        coerce_prepared_to_number(prepared).map_err(DynamicArrayReshapeEvalError::Preparation)?;
    if !raw.is_finite() {
        return Err(DynamicArrayReshapeEvalError::InvalidCount);
    }
    let truncated = raw.trunc();
    if (truncated - raw).abs() > f64::EPSILON {
        return Err(DynamicArrayReshapeEvalError::InvalidCount);
    }
    Ok(truncated as isize)
}

fn parse_positive_integer(
    prepared: &PreparedArgValue,
) -> Result<usize, DynamicArrayReshapeEvalError> {
    let value = parse_integer(prepared)?;
    if value < 1 {
        return Err(DynamicArrayReshapeEvalError::InvalidCount);
    }
    Ok(value as usize)
}

fn parse_bool_like(prepared: &PreparedArgValue) -> Result<bool, DynamicArrayReshapeEvalError> {
    match prepared {
        PreparedArgValue::Eval(EvalValue::Logical(b)) => Ok(*b),
        PreparedArgValue::Eval(EvalValue::Number(n)) => Ok(*n != 0.0),
        PreparedArgValue::EmptyCell | PreparedArgValue::MissingArg => Ok(false),
        PreparedArgValue::Eval(EvalValue::Error(code)) => Err(
            DynamicArrayReshapeEvalError::Preparation(CoercionError::WorksheetError(*code)),
        ),
        _ => Err(DynamicArrayReshapeEvalError::Preparation(
            CoercionError::UnsupportedValueKind("boolean_like"),
        )),
    }
}

fn resolve_selector(index: isize, len: usize) -> Result<usize, DynamicArrayReshapeEvalError> {
    if index == 0 {
        return Err(DynamicArrayReshapeEvalError::InvalidSelector);
    }
    let resolved = if index > 0 {
        (index - 1) as usize
    } else {
        let offset = len as isize + index;
        if offset < 0 {
            return Err(DynamicArrayReshapeEvalError::InvalidSelector);
        }
        offset as usize
    };
    if resolved >= len {
        return Err(DynamicArrayReshapeEvalError::InvalidSelector);
    }
    Ok(resolved)
}

fn build_array(
    rows: usize,
    cols: usize,
    cells: Vec<ArrayCellValue>,
) -> Result<EvalValue, DynamicArrayReshapeEvalError> {
    EvalArray::new(ArrayShape { rows, cols }, cells)
        .map(EvalValue::Array)
        .ok_or(DynamicArrayReshapeEvalError::EmptyArrayResult)
}

fn row_signature(array: &EvalArray, row: usize) -> Vec<ArrayCellValue> {
    array.row_slice(row).expect("validated row").to_vec()
}

fn column_signature(array: &EvalArray, col: usize) -> Vec<ArrayCellValue> {
    (0..array.shape().rows)
        .map(|row| array.get(row, col).expect("validated col").clone())
        .collect()
}

fn cell_signature(cell: &ArrayCellValue) -> String {
    match cell {
        ArrayCellValue::Number(n) => format!("n:{n:?}"),
        ArrayCellValue::Text(t) => format!("t:{}", t.to_string_lossy()),
        ArrayCellValue::Logical(b) => format!("b:{b}"),
        ArrayCellValue::Error(code) => format!("e:{code:?}"),
        ArrayCellValue::EmptyCell => "m:".to_string(),
    }
}

fn signature_key(cells: &[ArrayCellValue]) -> String {
    cells
        .iter()
        .map(cell_signature)
        .collect::<Vec<_>>()
        .join("\u{001f}")
}

fn flatten_cells(array: &EvalArray, by_col: bool) -> Vec<ArrayCellValue> {
    if !by_col {
        return array.iter_row_major().cloned().collect();
    }

    let mut cells = Vec::with_capacity(array.shape().rows * array.shape().cols);
    for col in 0..array.shape().cols {
        for row in 0..array.shape().rows {
            cells.push(array.get(row, col).expect("validated cell").clone());
        }
    }
    cells
}

fn compare_cell_values(lhs: &ArrayCellValue, rhs: &ArrayCellValue) -> Ordering {
    match (lhs, rhs) {
        (ArrayCellValue::Number(a), ArrayCellValue::Number(b)) => {
            a.partial_cmp(b).unwrap_or(Ordering::Equal)
        }
        (ArrayCellValue::Text(a), ArrayCellValue::Text(b)) => {
            a.to_string_lossy().cmp(&b.to_string_lossy())
        }
        (ArrayCellValue::Logical(a), ArrayCellValue::Logical(b)) => a.cmp(b),
        (ArrayCellValue::EmptyCell, ArrayCellValue::EmptyCell) => Ordering::Equal,
        (ArrayCellValue::Error(a), ArrayCellValue::Error(b)) => (*a as u8).cmp(&(*b as u8)),
        (ArrayCellValue::EmptyCell, _) => Ordering::Less,
        (_, ArrayCellValue::EmptyCell) => Ordering::Greater,
        (ArrayCellValue::Number(_), _) => Ordering::Less,
        (_, ArrayCellValue::Number(_)) => Ordering::Greater,
        (ArrayCellValue::Text(_), _) => Ordering::Less,
        (_, ArrayCellValue::Text(_)) => Ordering::Greater,
        (ArrayCellValue::Logical(_), _) => Ordering::Less,
        (_, ArrayCellValue::Logical(_)) => Ordering::Greater,
    }
}

pub fn eval_choosecols_prepared(
    args: &[PreparedArgValue],
) -> Result<EvalValue, DynamicArrayReshapeEvalError> {
    let array = materialize_array_arg(&args[0]);
    let cols: Vec<usize> = args[1..]
        .iter()
        .map(parse_integer)
        .map(|result| result.and_then(|idx| resolve_selector(idx, array.shape().cols)))
        .collect::<Result<_, _>>()?;

    let mut cells = Vec::with_capacity(array.shape().rows * cols.len());
    for row in 0..array.shape().rows {
        for col in &cols {
            cells.push(array.get(row, *col).expect("validated selector").clone());
        }
    }
    build_array(array.shape().rows, cols.len(), cells)
}

pub fn eval_chooserows_prepared(
    args: &[PreparedArgValue],
) -> Result<EvalValue, DynamicArrayReshapeEvalError> {
    let array = materialize_array_arg(&args[0]);
    let rows: Vec<usize> = args[1..]
        .iter()
        .map(parse_integer)
        .map(|result| result.and_then(|idx| resolve_selector(idx, array.shape().rows)))
        .collect::<Result<_, _>>()?;

    let mut cells = Vec::with_capacity(rows.len() * array.shape().cols);
    for row in &rows {
        cells.extend(
            array
                .row_slice(*row)
                .expect("validated selector")
                .iter()
                .cloned(),
        );
    }
    build_array(rows.len(), array.shape().cols, cells)
}

fn take_span(len: usize, count: isize) -> Result<(usize, usize), DynamicArrayReshapeEvalError> {
    if count == 0 {
        return Err(DynamicArrayReshapeEvalError::EmptyArrayResult);
    }
    if count > 0 {
        let take = usize::min(count as usize, len);
        Ok((0, take))
    } else {
        let take = usize::min((-count) as usize, len);
        Ok((len - take, len))
    }
}

fn drop_span(len: usize, count: isize) -> Result<(usize, usize), DynamicArrayReshapeEvalError> {
    let (start, end) = if count >= 0 {
        (usize::min(count as usize, len), len)
    } else {
        (0, len.saturating_sub((-count) as usize))
    };
    if start >= end {
        return Err(DynamicArrayReshapeEvalError::EmptyArrayResult);
    }
    Ok((start, end))
}

pub fn eval_take_prepared(
    args: &[PreparedArgValue],
) -> Result<EvalValue, DynamicArrayReshapeEvalError> {
    let array = materialize_array_arg(&args[0]);
    let row_count = parse_integer(&args[1])?;
    let col_count = if let Some(arg) = args.get(2) {
        parse_integer(arg)?
    } else {
        array.shape().cols as isize
    };
    let (row_start, row_end) = take_span(array.shape().rows, row_count)?;
    let (col_start, col_end) = take_span(array.shape().cols, col_count)?;
    let rows = row_end - row_start;
    let cols = col_end - col_start;
    let mut cells = Vec::with_capacity(rows * cols);
    for row in row_start..row_end {
        for col in col_start..col_end {
            cells.push(array.get(row, col).expect("validated slice").clone());
        }
    }
    build_array(rows, cols, cells)
}

pub fn eval_drop_prepared(
    args: &[PreparedArgValue],
) -> Result<EvalValue, DynamicArrayReshapeEvalError> {
    let array = materialize_array_arg(&args[0]);
    let row_count = parse_integer(&args[1])?;
    let col_count = if let Some(arg) = args.get(2) {
        parse_integer(arg)?
    } else {
        0
    };
    let (row_start, row_end) = drop_span(array.shape().rows, row_count)?;
    let (col_start, col_end) = drop_span(array.shape().cols, col_count)?;
    let rows = row_end - row_start;
    let cols = col_end - col_start;
    let mut cells = Vec::with_capacity(rows * cols);
    for row in row_start..row_end {
        for col in col_start..col_end {
            cells.push(array.get(row, col).expect("validated slice").clone());
        }
    }
    build_array(rows, cols, cells)
}

pub fn eval_expand_prepared(
    args: &[PreparedArgValue],
) -> Result<EvalValue, DynamicArrayReshapeEvalError> {
    let array = materialize_array_arg(&args[0]);
    let target_rows = parse_positive_integer(&args[1])?;
    let target_cols = if let Some(arg) = args.get(2) {
        parse_positive_integer(arg)?
    } else {
        array.shape().cols
    };
    if target_rows < array.shape().rows || target_cols < array.shape().cols {
        return Err(DynamicArrayReshapeEvalError::DimensionTooSmall);
    }

    let pad_cell = args
        .get(3)
        .map(scalar_cell)
        .unwrap_or(ArrayCellValue::Error(WorksheetErrorCode::NA));
    let mut cells = Vec::with_capacity(target_rows * target_cols);
    for row in 0..target_rows {
        for col in 0..target_cols {
            let cell = array
                .get(row, col)
                .cloned()
                .unwrap_or_else(|| pad_cell.clone());
            cells.push(cell);
        }
    }
    build_array(target_rows, target_cols, cells)
}

fn should_ignore_cell(cell: &ArrayCellValue, ignore_mode: usize) -> bool {
    match ignore_mode {
        0 => false,
        1 => matches!(cell, ArrayCellValue::EmptyCell),
        2 => matches!(cell, ArrayCellValue::Error(_)),
        3 => matches!(cell, ArrayCellValue::EmptyCell | ArrayCellValue::Error(_)),
        _ => false,
    }
}

fn parse_ignore_mode(
    arg: Option<&PreparedArgValue>,
) -> Result<usize, DynamicArrayReshapeEvalError> {
    let Some(arg) = arg else {
        return Ok(0);
    };
    let mode = parse_integer(arg)?;
    if !(0..=3).contains(&mode) {
        return Err(DynamicArrayReshapeEvalError::InvalidIgnoreMode);
    }
    Ok(mode as usize)
}

pub fn eval_tocol_prepared(
    args: &[PreparedArgValue],
) -> Result<EvalValue, DynamicArrayReshapeEvalError> {
    let array = materialize_array_arg(&args[0]);
    let ignore_mode = parse_ignore_mode(args.get(1))?;
    let by_col = args
        .get(2)
        .map(parse_bool_like)
        .transpose()?
        .unwrap_or(false);
    let cells: Vec<ArrayCellValue> = flatten_cells(&array, by_col)
        .into_iter()
        .filter(|cell| !should_ignore_cell(cell, ignore_mode))
        .collect();
    if cells.is_empty() {
        return Err(DynamicArrayReshapeEvalError::EmptyArrayResult);
    }
    build_array(cells.len(), 1, cells)
}

pub fn eval_torow_prepared(
    args: &[PreparedArgValue],
) -> Result<EvalValue, DynamicArrayReshapeEvalError> {
    let array = materialize_array_arg(&args[0]);
    let ignore_mode = parse_ignore_mode(args.get(1))?;
    let by_col = args
        .get(2)
        .map(parse_bool_like)
        .transpose()?
        .unwrap_or(false);
    let cells: Vec<ArrayCellValue> = flatten_cells(&array, by_col)
        .into_iter()
        .filter(|cell| !should_ignore_cell(cell, ignore_mode))
        .collect();
    if cells.is_empty() {
        return Err(DynamicArrayReshapeEvalError::EmptyArrayResult);
    }
    build_array(1, cells.len(), cells)
}

pub fn eval_transpose_prepared(
    args: &[PreparedArgValue],
) -> Result<EvalValue, DynamicArrayReshapeEvalError> {
    let array = materialize_array_arg(&args[0]);
    let mut cells = Vec::with_capacity(array.shape().rows * array.shape().cols);
    for row in 0..array.shape().cols {
        for col in 0..array.shape().rows {
            cells.push(array.get(col, row).expect("validated transpose").clone());
        }
    }
    build_array(array.shape().cols, array.shape().rows, cells)
}

pub fn eval_vstack_prepared(
    args: &[PreparedArgValue],
) -> Result<EvalValue, DynamicArrayReshapeEvalError> {
    let arrays: Vec<EvalArray> = args.iter().map(materialize_array_arg).collect();
    let rows: usize = arrays.iter().map(|array| array.shape().rows).sum();
    let cols = arrays
        .iter()
        .map(|array| array.shape().cols)
        .max()
        .unwrap_or(1);

    let mut cells = Vec::with_capacity(rows * cols);
    for array in &arrays {
        for row in 0..array.shape().rows {
            for col in 0..cols {
                cells.push(
                    array
                        .get(row, col)
                        .cloned()
                        .unwrap_or(ArrayCellValue::Error(WorksheetErrorCode::NA)),
                );
            }
        }
    }
    build_array(rows, cols, cells)
}

pub fn eval_wraprows_prepared(
    args: &[PreparedArgValue],
) -> Result<EvalValue, DynamicArrayReshapeEvalError> {
    let source = materialize_array_arg(&args[0]);
    let wrap_count = parse_positive_integer(&args[1])?;
    let pad_cell = args
        .get(2)
        .map(scalar_cell)
        .unwrap_or(ArrayCellValue::Error(WorksheetErrorCode::NA));
    let flat = flatten_cells(&source, false);
    let rows = flat.len().div_ceil(wrap_count);
    let cols = wrap_count;
    let mut cells = Vec::with_capacity(rows * cols);
    let mut iter = flat.into_iter();
    for _row in 0..rows {
        for _col in 0..cols {
            cells.push(iter.next().unwrap_or_else(|| pad_cell.clone()));
        }
    }
    build_array(rows, cols, cells)
}

pub fn eval_wrapcols_prepared(
    args: &[PreparedArgValue],
) -> Result<EvalValue, DynamicArrayReshapeEvalError> {
    let source = materialize_array_arg(&args[0]);
    let wrap_count = parse_positive_integer(&args[1])?;
    let pad_cell = args
        .get(2)
        .map(scalar_cell)
        .unwrap_or(ArrayCellValue::Error(WorksheetErrorCode::NA));
    let flat = flatten_cells(&source, false);
    let rows = wrap_count;
    let cols = flat.len().div_ceil(wrap_count);
    let mut grid = vec![pad_cell.clone(); rows * cols];
    for (index, cell) in flat.into_iter().enumerate() {
        let row = index % rows;
        let col = index / rows;
        grid[row * cols + col] = cell;
    }
    build_array(rows, cols, grid)
}

fn build_filter_mask(
    include: &EvalArray,
    target_shape: ArrayShape,
) -> Result<(bool, Vec<bool>), DynamicArrayReshapeEvalError> {
    if include.shape().cols == 1 && include.shape().rows == target_shape.rows {
        let mask = (0..include.shape().rows)
            .map(|row| {
                include.get(row, 0).map_or(Ok(false), |cell| match cell {
                    ArrayCellValue::Logical(b) => Ok(*b),
                    ArrayCellValue::Number(n) => Ok(*n != 0.0),
                    ArrayCellValue::EmptyCell => Ok(false),
                    ArrayCellValue::Error(code) => Err(DynamicArrayReshapeEvalError::Preparation(
                        CoercionError::WorksheetError(*code),
                    )),
                    _ => Err(DynamicArrayReshapeEvalError::InvalidIncludeShape),
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        return Ok((false, mask));
    }
    if include.shape().rows == 1 && include.shape().cols == target_shape.cols {
        let mask = (0..include.shape().cols)
            .map(|col| {
                include.get(0, col).map_or(Ok(false), |cell| match cell {
                    ArrayCellValue::Logical(b) => Ok(*b),
                    ArrayCellValue::Number(n) => Ok(*n != 0.0),
                    ArrayCellValue::EmptyCell => Ok(false),
                    ArrayCellValue::Error(code) => Err(DynamicArrayReshapeEvalError::Preparation(
                        CoercionError::WorksheetError(*code),
                    )),
                    _ => Err(DynamicArrayReshapeEvalError::InvalidIncludeShape),
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        return Ok((true, mask));
    }
    Err(DynamicArrayReshapeEvalError::InvalidIncludeShape)
}

pub fn eval_filter_prepared(
    args: &[PreparedArgValue],
) -> Result<EvalValue, DynamicArrayReshapeEvalError> {
    let array = materialize_array_arg(&args[0]);
    let include = materialize_array_arg(&args[1]);
    let (filter_cols, mask) = build_filter_mask(&include, array.shape())?;

    if !filter_cols {
        let selected_rows: Vec<usize> = mask
            .iter()
            .enumerate()
            .filter_map(|(idx, keep)| if *keep { Some(idx) } else { None })
            .collect();
        if selected_rows.is_empty() {
            return if let Some(if_empty) = args.get(2) {
                Ok(match if_empty {
                    PreparedArgValue::Eval(EvalValue::Array(array)) => {
                        EvalValue::Array(array.clone())
                    }
                    other => EvalValue::Array(materialize_array_arg(other)),
                })
            } else {
                Err(DynamicArrayReshapeEvalError::EmptyArrayResult)
            };
        }
        let mut cells = Vec::with_capacity(selected_rows.len() * array.shape().cols);
        for row in selected_rows {
            cells.extend(array.row_slice(row).expect("validated row").iter().cloned());
        }
        return build_array(cells.len() / array.shape().cols, array.shape().cols, cells);
    }

    let selected_cols: Vec<usize> = mask
        .iter()
        .enumerate()
        .filter_map(|(idx, keep)| if *keep { Some(idx) } else { None })
        .collect();
    if selected_cols.is_empty() {
        return if let Some(if_empty) = args.get(2) {
            Ok(match if_empty {
                PreparedArgValue::Eval(EvalValue::Array(array)) => EvalValue::Array(array.clone()),
                other => EvalValue::Array(materialize_array_arg(other)),
            })
        } else {
            Err(DynamicArrayReshapeEvalError::EmptyArrayResult)
        };
    }
    let mut cells = Vec::with_capacity(array.shape().rows * selected_cols.len());
    for row in 0..array.shape().rows {
        for col in &selected_cols {
            cells.push(array.get(row, *col).expect("validated col").clone());
        }
    }
    build_array(array.shape().rows, selected_cols.len(), cells)
}

fn parse_sort_order(arg: Option<&PreparedArgValue>) -> Result<bool, DynamicArrayReshapeEvalError> {
    let Some(arg) = arg else {
        return Ok(false);
    };
    match parse_integer(arg)? {
        1 => Ok(false),
        -1 => Ok(true),
        _ => Err(DynamicArrayReshapeEvalError::InvalidSortOrder),
    }
}

fn parse_sort_index(
    arg: Option<&PreparedArgValue>,
    len: usize,
) -> Result<usize, DynamicArrayReshapeEvalError> {
    let Some(arg) = arg else {
        return Ok(0);
    };
    let idx = parse_integer(arg)?;
    if idx < 1 || idx as usize > len {
        return Err(DynamicArrayReshapeEvalError::InvalidSortIndex);
    }
    Ok(idx as usize - 1)
}

pub fn eval_sort_prepared(
    args: &[PreparedArgValue],
) -> Result<EvalValue, DynamicArrayReshapeEvalError> {
    let array = materialize_array_arg(&args[0]);
    let by_col = args
        .get(3)
        .map(parse_bool_like)
        .transpose()?
        .unwrap_or(false);
    let descending = parse_sort_order(args.get(2))?;

    if by_col {
        let sort_index = parse_sort_index(args.get(1), array.shape().rows)?;
        let mut order: Vec<usize> = (0..array.shape().cols).collect();
        order.sort_by(|lhs, rhs| {
            let lhs_cell = array.get(sort_index, *lhs).expect("validated column");
            let rhs_cell = array.get(sort_index, *rhs).expect("validated column");
            let ord = compare_cell_values(lhs_cell, rhs_cell);
            if descending { ord.reverse() } else { ord }
        });
        let mut cells = Vec::with_capacity(array.shape().rows * array.shape().cols);
        for row in 0..array.shape().rows {
            for col in &order {
                cells.push(array.get(row, *col).expect("validated cell").clone());
            }
        }
        return build_array(array.shape().rows, array.shape().cols, cells);
    }

    let sort_index = parse_sort_index(args.get(1), array.shape().cols)?;
    let mut order: Vec<usize> = (0..array.shape().rows).collect();
    order.sort_by(|lhs, rhs| {
        let lhs_cell = array.get(*lhs, sort_index).expect("validated row");
        let rhs_cell = array.get(*rhs, sort_index).expect("validated row");
        let ord = compare_cell_values(lhs_cell, rhs_cell);
        if descending { ord.reverse() } else { ord }
    });
    let mut cells = Vec::with_capacity(array.shape().rows * array.shape().cols);
    for row in order {
        cells.extend(array.row_slice(row).expect("validated row").iter().cloned());
    }
    build_array(array.shape().rows, array.shape().cols, cells)
}

pub fn eval_sortby_prepared(
    args: &[PreparedArgValue],
) -> Result<EvalValue, DynamicArrayReshapeEvalError> {
    let array = materialize_array_arg(&args[0]);
    let by_array = materialize_array_arg(&args[1]);
    let descending = parse_sort_order(args.get(2))?;

    let row_keys: Vec<ArrayCellValue> =
        if by_array.shape().cols == 1 && by_array.shape().rows == array.shape().rows {
            (0..by_array.shape().rows)
                .map(|row| by_array.get(row, 0).expect("validated key").clone())
                .collect()
        } else if by_array.shape().rows == 1 && by_array.shape().cols == array.shape().rows {
            (0..by_array.shape().cols)
                .map(|col| by_array.get(0, col).expect("validated key").clone())
                .collect()
        } else {
            return Err(DynamicArrayReshapeEvalError::InvalidIncludeShape);
        };

    let mut order: Vec<usize> = (0..array.shape().rows).collect();
    order.sort_by(|lhs, rhs| {
        let ord = compare_cell_values(&row_keys[*lhs], &row_keys[*rhs]);
        if descending { ord.reverse() } else { ord }
    });

    let mut cells = Vec::with_capacity(array.shape().rows * array.shape().cols);
    for row in order {
        cells.extend(array.row_slice(row).expect("validated row").iter().cloned());
    }
    build_array(array.shape().rows, array.shape().cols, cells)
}

pub fn eval_unique_prepared(
    args: &[PreparedArgValue],
) -> Result<EvalValue, DynamicArrayReshapeEvalError> {
    let array = materialize_array_arg(&args[0]);
    let by_col = args
        .get(1)
        .map(parse_bool_like)
        .transpose()?
        .unwrap_or(false);
    let exactly_once = args
        .get(2)
        .map(parse_bool_like)
        .transpose()?
        .unwrap_or(false);

    if !by_col {
        let mut counts: HashMap<String, usize> = HashMap::new();
        let mut signatures = Vec::with_capacity(array.shape().rows);
        for row in 0..array.shape().rows {
            let sig = row_signature(&array, row);
            let key = signature_key(&sig);
            *counts.entry(key.clone()).or_insert(0) += 1;
            signatures.push((key, sig));
        }

        let mut seen: HashMap<String, bool> = HashMap::new();
        let mut cells = Vec::new();
        let mut out_rows = 0usize;
        for (key, sig) in signatures {
            let count = *counts.get(&key).expect("signature counted");
            if exactly_once && count != 1 {
                continue;
            }
            if !exactly_once && seen.insert(key, true).is_some() {
                continue;
            }
            out_rows += 1;
            cells.extend(sig);
        }
        if out_rows == 0 {
            return Err(DynamicArrayReshapeEvalError::EmptyArrayResult);
        }
        return build_array(out_rows, array.shape().cols, cells);
    }

    let mut counts: HashMap<String, usize> = HashMap::new();
    let mut signatures = Vec::with_capacity(array.shape().cols);
    for col in 0..array.shape().cols {
        let sig = column_signature(&array, col);
        let key = signature_key(&sig);
        *counts.entry(key.clone()).or_insert(0) += 1;
        signatures.push((key, sig));
    }

    let mut seen: HashMap<String, bool> = HashMap::new();
    let mut kept = Vec::new();
    for (key, sig) in signatures {
        let count = *counts.get(&key).expect("signature counted");
        if exactly_once && count != 1 {
            continue;
        }
        if !exactly_once && seen.insert(key, true).is_some() {
            continue;
        }
        kept.push(sig);
    }
    if kept.is_empty() {
        return Err(DynamicArrayReshapeEvalError::EmptyArrayResult);
    }
    let rows = kept[0].len();
    let cols = kept.len();
    let mut cells = Vec::with_capacity(rows * cols);
    for row in 0..rows {
        for col in 0..cols {
            cells.push(kept[col][row].clone());
        }
    }
    build_array(rows, cols, cells)
}

fn surface_arity_error(meta: &FunctionMeta, actual: usize) -> DynamicArrayReshapeEvalError {
    DynamicArrayReshapeEvalError::ArityMismatch {
        expected_min: meta.arity.min,
        expected_max: meta.arity.max,
        actual,
    }
}

fn eval_surface_common(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
    meta: &FunctionMeta,
    eval: impl FnOnce(&[PreparedArgValue]) -> Result<EvalValue, DynamicArrayReshapeEvalError>,
) -> Result<EvalValue, DynamicArrayReshapeEvalError> {
    if !meta.arity.accepts(args.len()) {
        return Err(surface_arity_error(meta, args.len()));
    }
    run_values_only_prepared(
        args,
        resolver,
        eval,
        DynamicArrayReshapeEvalError::Preparation,
    )
}

pub fn eval_choosecols_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DynamicArrayReshapeEvalError> {
    eval_surface_common(args, resolver, &CHOOSECOLS_META, eval_choosecols_prepared)
}

pub fn eval_chooserows_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DynamicArrayReshapeEvalError> {
    eval_surface_common(args, resolver, &CHOOSEROWS_META, eval_chooserows_prepared)
}

pub fn eval_drop_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DynamicArrayReshapeEvalError> {
    eval_surface_common(args, resolver, &DROP_META, eval_drop_prepared)
}

pub fn eval_expand_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DynamicArrayReshapeEvalError> {
    eval_surface_common(args, resolver, &EXPAND_META, eval_expand_prepared)
}

pub fn eval_filter_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DynamicArrayReshapeEvalError> {
    eval_surface_common(args, resolver, &FILTER_META, eval_filter_prepared)
}

pub fn eval_sort_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DynamicArrayReshapeEvalError> {
    eval_surface_common(args, resolver, &SORT_META, eval_sort_prepared)
}

pub fn eval_sortby_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DynamicArrayReshapeEvalError> {
    eval_surface_common(args, resolver, &SORTBY_META, eval_sortby_prepared)
}

pub fn eval_take_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DynamicArrayReshapeEvalError> {
    eval_surface_common(args, resolver, &TAKE_META, eval_take_prepared)
}

pub fn eval_tocol_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DynamicArrayReshapeEvalError> {
    eval_surface_common(args, resolver, &TOCOL_META, eval_tocol_prepared)
}

pub fn eval_torow_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DynamicArrayReshapeEvalError> {
    eval_surface_common(args, resolver, &TOROW_META, eval_torow_prepared)
}

pub fn eval_transpose_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DynamicArrayReshapeEvalError> {
    eval_surface_common(args, resolver, &TRANSPOSE_META, eval_transpose_prepared)
}

pub fn eval_unique_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DynamicArrayReshapeEvalError> {
    eval_surface_common(args, resolver, &UNIQUE_META, eval_unique_prepared)
}

pub fn eval_vstack_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DynamicArrayReshapeEvalError> {
    eval_surface_common(args, resolver, &VSTACK_META, eval_vstack_prepared)
}

pub fn eval_wrapcols_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DynamicArrayReshapeEvalError> {
    eval_surface_common(args, resolver, &WRAPCOLS_META, eval_wrapcols_prepared)
}

pub fn eval_wraprows_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DynamicArrayReshapeEvalError> {
    eval_surface_common(args, resolver, &WRAPROWS_META, eval_wraprows_prepared)
}

pub fn map_dynamic_array_reshape_error_to_ws(
    error: &DynamicArrayReshapeEvalError,
) -> WorksheetErrorCode {
    match error {
        DynamicArrayReshapeEvalError::ArityMismatch { .. }
        | DynamicArrayReshapeEvalError::InvalidSelector
        | DynamicArrayReshapeEvalError::InvalidCount
        | DynamicArrayReshapeEvalError::InvalidIgnoreMode
        | DynamicArrayReshapeEvalError::InvalidSortOrder
        | DynamicArrayReshapeEvalError::InvalidSortIndex
        | DynamicArrayReshapeEvalError::InvalidIncludeShape
        | DynamicArrayReshapeEvalError::Preparation(_) => WorksheetErrorCode::Value,
        DynamicArrayReshapeEvalError::EmptyArrayResult => WorksheetErrorCode::Calc,
        DynamicArrayReshapeEvalError::DimensionTooSmall => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::{RefResolutionError, ResolverCapabilities};
    use crate::value::{ExcelText, ReferenceLike};

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

    fn array(rows: Vec<Vec<ArrayCellValue>>) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Array(EvalArray::from_rows(rows).unwrap()))
    }

    fn num(n: f64) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Number(n))
    }

    #[test]
    fn choosecols_and_chooserows_preserve_order_and_duplicates() {
        let cols = eval_choosecols_surface(
            &[
                array(vec![
                    vec![
                        ArrayCellValue::Number(1.0),
                        ArrayCellValue::Number(2.0),
                        ArrayCellValue::Number(3.0),
                    ],
                    vec![
                        ArrayCellValue::Number(4.0),
                        ArrayCellValue::Number(5.0),
                        ArrayCellValue::Number(6.0),
                    ],
                ]),
                num(3.0),
                num(1.0),
                num(-1.0),
            ],
            &NoResolver,
        )
        .unwrap();
        assert_eq!(
            cols,
            EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![
                        ArrayCellValue::Number(3.0),
                        ArrayCellValue::Number(1.0),
                        ArrayCellValue::Number(3.0),
                    ],
                    vec![
                        ArrayCellValue::Number(6.0),
                        ArrayCellValue::Number(4.0),
                        ArrayCellValue::Number(6.0),
                    ],
                ])
                .unwrap()
            )
        );

        let rows = eval_chooserows_surface(
            &[
                array(vec![
                    vec![ArrayCellValue::Number(1.0)],
                    vec![ArrayCellValue::Number(2.0)],
                    vec![ArrayCellValue::Number(3.0)],
                ]),
                num(-1.0),
                num(1.0),
            ],
            &NoResolver,
        )
        .unwrap();
        assert_eq!(
            rows,
            EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Number(3.0)],
                    vec![ArrayCellValue::Number(1.0)],
                ])
                .unwrap()
            )
        );
    }

    #[test]
    fn take_drop_and_expand_match_seeded_slices() {
        let source = array(vec![
            vec![
                ArrayCellValue::Number(1.0),
                ArrayCellValue::Number(2.0),
                ArrayCellValue::Number(3.0),
            ],
            vec![
                ArrayCellValue::Number(4.0),
                ArrayCellValue::Number(5.0),
                ArrayCellValue::Number(6.0),
            ],
            vec![
                ArrayCellValue::Number(7.0),
                ArrayCellValue::Number(8.0),
                ArrayCellValue::Number(9.0),
            ],
        ]);
        let take = eval_take_surface(&[source.clone(), num(2.0), num(-2.0)], &NoResolver).unwrap();
        assert_eq!(
            take,
            EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Number(2.0), ArrayCellValue::Number(3.0)],
                    vec![ArrayCellValue::Number(5.0), ArrayCellValue::Number(6.0)],
                ])
                .unwrap()
            )
        );

        let drop = eval_drop_surface(&[source.clone(), num(1.0), num(-1.0)], &NoResolver).unwrap();
        assert_eq!(
            drop,
            EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Number(4.0), ArrayCellValue::Number(5.0)],
                    vec![ArrayCellValue::Number(7.0), ArrayCellValue::Number(8.0)],
                ])
                .unwrap()
            )
        );

        let expand = eval_expand_surface(
            &[
                source,
                num(4.0),
                num(4.0),
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_interop_assignment("x"))),
            ],
            &NoResolver,
        )
        .unwrap();
        assert_eq!(
            expand,
            EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![
                        ArrayCellValue::Number(1.0),
                        ArrayCellValue::Number(2.0),
                        ArrayCellValue::Number(3.0),
                        ArrayCellValue::Text(ExcelText::from_interop_assignment("x")),
                    ],
                    vec![
                        ArrayCellValue::Number(4.0),
                        ArrayCellValue::Number(5.0),
                        ArrayCellValue::Number(6.0),
                        ArrayCellValue::Text(ExcelText::from_interop_assignment("x")),
                    ],
                    vec![
                        ArrayCellValue::Number(7.0),
                        ArrayCellValue::Number(8.0),
                        ArrayCellValue::Number(9.0),
                        ArrayCellValue::Text(ExcelText::from_interop_assignment("x")),
                    ],
                    vec![
                        ArrayCellValue::Text(ExcelText::from_interop_assignment("x")),
                        ArrayCellValue::Text(ExcelText::from_interop_assignment("x")),
                        ArrayCellValue::Text(ExcelText::from_interop_assignment("x")),
                        ArrayCellValue::Text(ExcelText::from_interop_assignment("x")),
                    ],
                ])
                .unwrap()
            )
        );
    }

    #[test]
    fn tocol_torow_wraprows_wrapcols_and_transpose_match_seeded_shapes() {
        let source = array(vec![
            vec![
                ArrayCellValue::Number(1.0),
                ArrayCellValue::EmptyCell,
                ArrayCellValue::Number(3.0),
            ],
            vec![
                ArrayCellValue::Number(4.0),
                ArrayCellValue::Error(WorksheetErrorCode::NA),
                ArrayCellValue::Number(6.0),
            ],
        ]);
        let tocol = eval_tocol_surface(
            &[
                source.clone(),
                num(3.0),
                CallArgValue::Eval(EvalValue::Logical(false)),
            ],
            &NoResolver,
        )
        .unwrap();
        assert_eq!(
            tocol,
            EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Number(1.0)],
                    vec![ArrayCellValue::Number(3.0)],
                    vec![ArrayCellValue::Number(4.0)],
                    vec![ArrayCellValue::Number(6.0)],
                ])
                .unwrap()
            )
        );

        let torow = eval_torow_surface(
            &[
                source.clone(),
                num(0.0),
                CallArgValue::Eval(EvalValue::Logical(true)),
            ],
            &NoResolver,
        )
        .unwrap();
        assert_eq!(
            torow,
            EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Number(1.0),
                    ArrayCellValue::Number(4.0),
                    ArrayCellValue::EmptyCell,
                    ArrayCellValue::Error(WorksheetErrorCode::NA),
                    ArrayCellValue::Number(3.0),
                    ArrayCellValue::Number(6.0),
                ]])
                .unwrap()
            )
        );

        let wraprows = eval_wraprows_surface(
            &[
                array(vec![vec![
                    ArrayCellValue::Number(1.0),
                    ArrayCellValue::Number(2.0),
                    ArrayCellValue::Number(3.0),
                    ArrayCellValue::Number(4.0),
                    ArrayCellValue::Number(5.0),
                ]]),
                num(2.0),
            ],
            &NoResolver,
        )
        .unwrap();
        assert_eq!(
            wraprows,
            EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Number(1.0), ArrayCellValue::Number(2.0)],
                    vec![ArrayCellValue::Number(3.0), ArrayCellValue::Number(4.0)],
                    vec![
                        ArrayCellValue::Number(5.0),
                        ArrayCellValue::Error(WorksheetErrorCode::NA),
                    ],
                ])
                .unwrap()
            )
        );

        let wrapcols = eval_wrapcols_surface(
            &[
                array(vec![vec![
                    ArrayCellValue::Number(1.0),
                    ArrayCellValue::Number(2.0),
                    ArrayCellValue::Number(3.0),
                    ArrayCellValue::Number(4.0),
                    ArrayCellValue::Number(5.0),
                ]]),
                num(2.0),
            ],
            &NoResolver,
        )
        .unwrap();
        assert_eq!(
            wrapcols,
            EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![
                        ArrayCellValue::Number(1.0),
                        ArrayCellValue::Number(3.0),
                        ArrayCellValue::Number(5.0),
                    ],
                    vec![
                        ArrayCellValue::Number(2.0),
                        ArrayCellValue::Number(4.0),
                        ArrayCellValue::Error(WorksheetErrorCode::NA),
                    ],
                ])
                .unwrap()
            )
        );

        let transpose = eval_transpose_surface(&[source], &NoResolver).unwrap();
        assert_eq!(
            transpose,
            EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Number(1.0), ArrayCellValue::Number(4.0)],
                    vec![
                        ArrayCellValue::EmptyCell,
                        ArrayCellValue::Error(WorksheetErrorCode::NA)
                    ],
                    vec![ArrayCellValue::Number(3.0), ArrayCellValue::Number(6.0)],
                ])
                .unwrap()
            )
        );
    }

    #[test]
    fn filter_sort_sortby_unique_and_vstack_match_seeded_lanes() {
        let filter = eval_filter_surface(
            &[
                array(vec![
                    vec![ArrayCellValue::Number(1.0), ArrayCellValue::Number(10.0)],
                    vec![ArrayCellValue::Number(2.0), ArrayCellValue::Number(20.0)],
                    vec![ArrayCellValue::Number(3.0), ArrayCellValue::Number(30.0)],
                ]),
                array(vec![
                    vec![ArrayCellValue::Logical(true)],
                    vec![ArrayCellValue::Logical(false)],
                    vec![ArrayCellValue::Logical(true)],
                ]),
            ],
            &NoResolver,
        )
        .unwrap();
        assert_eq!(
            filter,
            EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Number(1.0), ArrayCellValue::Number(10.0)],
                    vec![ArrayCellValue::Number(3.0), ArrayCellValue::Number(30.0)],
                ])
                .unwrap()
            )
        );

        let sort = eval_sort_surface(
            &[
                array(vec![
                    vec![
                        ArrayCellValue::Number(3.0),
                        ArrayCellValue::Text(ExcelText::from_interop_assignment("c")),
                    ],
                    vec![
                        ArrayCellValue::Number(1.0),
                        ArrayCellValue::Text(ExcelText::from_interop_assignment("a")),
                    ],
                    vec![
                        ArrayCellValue::Number(2.0),
                        ArrayCellValue::Text(ExcelText::from_interop_assignment("b")),
                    ],
                ]),
                num(1.0),
                num(1.0),
            ],
            &NoResolver,
        )
        .unwrap();
        assert_eq!(
            sort,
            EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![
                        ArrayCellValue::Number(1.0),
                        ArrayCellValue::Text(ExcelText::from_interop_assignment("a")),
                    ],
                    vec![
                        ArrayCellValue::Number(2.0),
                        ArrayCellValue::Text(ExcelText::from_interop_assignment("b")),
                    ],
                    vec![
                        ArrayCellValue::Number(3.0),
                        ArrayCellValue::Text(ExcelText::from_interop_assignment("c")),
                    ],
                ])
                .unwrap()
            )
        );

        let sortby = eval_sortby_surface(
            &[
                array(vec![
                    vec![ArrayCellValue::Text(ExcelText::from_interop_assignment(
                        "alpha",
                    ))],
                    vec![ArrayCellValue::Text(ExcelText::from_interop_assignment(
                        "beta",
                    ))],
                    vec![ArrayCellValue::Text(ExcelText::from_interop_assignment(
                        "gamma",
                    ))],
                ]),
                array(vec![
                    vec![ArrayCellValue::Number(2.0)],
                    vec![ArrayCellValue::Number(3.0)],
                    vec![ArrayCellValue::Number(1.0)],
                ]),
                num(1.0),
            ],
            &NoResolver,
        )
        .unwrap();
        assert_eq!(
            sortby,
            EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Text(ExcelText::from_interop_assignment(
                        "gamma"
                    ))],
                    vec![ArrayCellValue::Text(ExcelText::from_interop_assignment(
                        "alpha"
                    ))],
                    vec![ArrayCellValue::Text(ExcelText::from_interop_assignment(
                        "beta"
                    ))],
                ])
                .unwrap()
            )
        );

        let unique = eval_unique_surface(
            &[
                array(vec![
                    vec![ArrayCellValue::Number(1.0), ArrayCellValue::Number(10.0)],
                    vec![ArrayCellValue::Number(1.0), ArrayCellValue::Number(10.0)],
                    vec![ArrayCellValue::Number(2.0), ArrayCellValue::Number(20.0)],
                ]),
                CallArgValue::Eval(EvalValue::Logical(false)),
                CallArgValue::Eval(EvalValue::Logical(false)),
            ],
            &NoResolver,
        )
        .unwrap();
        assert_eq!(
            unique,
            EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Number(1.0), ArrayCellValue::Number(10.0)],
                    vec![ArrayCellValue::Number(2.0), ArrayCellValue::Number(20.0)],
                ])
                .unwrap()
            )
        );

        let vstack = eval_vstack_surface(
            &[
                array(vec![vec![
                    ArrayCellValue::Number(1.0),
                    ArrayCellValue::Number(2.0),
                ]]),
                array(vec![
                    vec![ArrayCellValue::Number(3.0)],
                    vec![ArrayCellValue::Number(4.0)],
                ]),
            ],
            &NoResolver,
        )
        .unwrap();
        assert_eq!(
            vstack,
            EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Number(1.0), ArrayCellValue::Number(2.0)],
                    vec![
                        ArrayCellValue::Number(3.0),
                        ArrayCellValue::Error(WorksheetErrorCode::NA)
                    ],
                    vec![
                        ArrayCellValue::Number(4.0),
                        ArrayCellValue::Error(WorksheetErrorCode::NA)
                    ],
                ])
                .unwrap()
            )
        );
    }
}

use std::cmp::Ordering;
use std::collections::HashMap;

use crate::coercion::{CoercionError, coerce_calc_scalar_to_number};
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    PreparedValue, coerce_prepared_to_number, expand_arg_values_only, prepare_arg_values_only,
    prepare_calc_value_values_only, run_values_only_prepared,
};
use crate::resolver::ReferenceSystemProvider;
use crate::value::{
    ArrayShape, CalcArray, CalcValue, CoreValue, FunctionArg, FunctionArray, FunctionArrayCell,
    FunctionValue, WorksheetErrorCode,
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

fn scalar_cell(arg: &PreparedValue) -> FunctionArrayCell {
    match arg {
        PreparedValue::Eval(FunctionValue::Number(n)) => FunctionArrayCell::Number(*n),
        PreparedValue::Eval(FunctionValue::Text(t)) => FunctionArrayCell::Text(t.clone()),
        PreparedValue::Eval(FunctionValue::Logical(b)) => FunctionArrayCell::Logical(*b),
        PreparedValue::Eval(FunctionValue::Error(code)) => FunctionArrayCell::Error(*code),
        PreparedValue::Eval(FunctionValue::Reference(_)) => {
            FunctionArrayCell::Error(WorksheetErrorCode::Value)
        }
        PreparedValue::Eval(FunctionValue::Array(_)) => {
            FunctionArrayCell::Error(WorksheetErrorCode::Value)
        }
        PreparedValue::MissingArg | PreparedValue::EmptyCell => FunctionArrayCell::EmptyCell,
        _ => FunctionArrayCell::Error(WorksheetErrorCode::Value),
    }
}

fn materialize_array_arg(arg: &PreparedValue) -> FunctionArray {
    match arg {
        PreparedValue::Eval(FunctionValue::Array(array)) => array.clone(),
        other => FunctionArray::from_scalar(scalar_cell(other)),
    }
}

fn scalar_calc_value(value: &CalcValue) -> CalcValue {
    match value.core() {
        CoreValue::Array(_) | CoreValue::Reference(_) => {
            CalcValue::error(WorksheetErrorCode::Value)
        }
        CoreValue::Missing | CoreValue::Empty => CalcValue::empty(),
        _ => value.clone(),
    }
}

fn materialize_calc_array_arg(
    arg: &CalcValue,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcArray, DynamicArrayReshapeEvalError> {
    let prepared = prepare_calc_value_values_only(arg, resolver)
        .map_err(DynamicArrayReshapeEvalError::Preparation)?;
    Ok(match prepared.core() {
        CoreValue::Array(array) => array.clone(),
        _ => CalcArray::from_scalar(scalar_calc_value(&prepared))
            .expect("scalar materialization produces a non-empty 1x1 array"),
    })
}

enum StackArgSource<'a> {
    Array(&'a FunctionArray),
    Scalar(FunctionArrayCell),
}

impl<'a> StackArgSource<'a> {
    fn new(arg: &'a PreparedValue) -> Self {
        match arg {
            PreparedValue::Eval(FunctionValue::Array(array)) => Self::Array(array),
            other => Self::Scalar(scalar_cell(other)),
        }
    }

    fn shape(&self) -> ArrayShape {
        match self {
            Self::Array(array) => array.shape(),
            Self::Scalar(_) => ArrayShape { rows: 1, cols: 1 },
        }
    }

    fn get(&self, row: usize, col: usize) -> Option<&FunctionArrayCell> {
        match self {
            Self::Array(array) => array.get(row, col),
            Self::Scalar(cell) if row == 0 && col == 0 => Some(cell),
            Self::Scalar(_) => None,
        }
    }
}

fn prepared_from_array_cell(cell: &FunctionArrayCell) -> PreparedValue {
    match cell {
        FunctionArrayCell::Number(n) => PreparedValue::Eval(FunctionValue::Number(*n)),
        FunctionArrayCell::Text(t) => PreparedValue::Eval(FunctionValue::Text(t.clone())),
        FunctionArrayCell::Logical(b) => PreparedValue::Eval(FunctionValue::Logical(*b)),
        FunctionArrayCell::Error(code) => PreparedValue::Eval(FunctionValue::Error(*code)),
        FunctionArrayCell::EmptyCell => PreparedValue::EmptyCell,
    }
}

fn parse_integer(prepared: &PreparedValue) -> Result<isize, DynamicArrayReshapeEvalError> {
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

fn parse_integer_calc(value: &CalcValue) -> Result<isize, DynamicArrayReshapeEvalError> {
    let raw =
        coerce_calc_scalar_to_number(value).map_err(DynamicArrayReshapeEvalError::Preparation)?;
    if !raw.is_finite() {
        return Err(DynamicArrayReshapeEvalError::InvalidCount);
    }
    let truncated = raw.trunc();
    if (truncated - raw).abs() > f64::EPSILON {
        return Err(DynamicArrayReshapeEvalError::InvalidCount);
    }
    Ok(truncated as isize)
}

fn parse_positive_integer(prepared: &PreparedValue) -> Result<usize, DynamicArrayReshapeEvalError> {
    let value = parse_integer(prepared)?;
    if value < 1 {
        return Err(DynamicArrayReshapeEvalError::InvalidCount);
    }
    Ok(value as usize)
}

fn parse_bool_like(prepared: &PreparedValue) -> Result<bool, DynamicArrayReshapeEvalError> {
    match prepared {
        PreparedValue::Eval(FunctionValue::Logical(b)) => Ok(*b),
        PreparedValue::Eval(FunctionValue::Number(n)) => Ok(*n != 0.0),
        PreparedValue::EmptyCell | PreparedValue::MissingArg => Ok(false),
        PreparedValue::Eval(FunctionValue::Error(code)) => Err(
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
    cells: Vec<FunctionArrayCell>,
) -> Result<FunctionValue, DynamicArrayReshapeEvalError> {
    CalcArray::from_function_cells_iter(ArrayShape { rows, cols }, cells)
        .map(|array| FunctionValue::Array(array.to_function_array_lossy()))
        .ok_or(DynamicArrayReshapeEvalError::EmptyArrayResult)
}

fn build_calc_array(
    rows: usize,
    cols: usize,
    cells: Vec<CalcValue>,
) -> Result<CalcValue, DynamicArrayReshapeEvalError> {
    CalcArray::from_cells_iter(ArrayShape { rows, cols }, cells)
        .map(CalcValue::array)
        .ok_or(DynamicArrayReshapeEvalError::EmptyArrayResult)
}

fn value_from_if_empty_arg(arg: &PreparedValue) -> FunctionValue {
    match arg {
        PreparedValue::Eval(value) => value.clone(),
        PreparedValue::MissingArg | PreparedValue::EmptyCell => {
            FunctionValue::Text(crate::value::ExcelText::from_utf16_code_units(Vec::new()))
        }
    }
}

fn row_signature(array: &FunctionArray, row: usize) -> Vec<FunctionArrayCell> {
    array.row_slice(row).expect("validated row").to_vec()
}

fn column_signature(array: &FunctionArray, col: usize) -> Vec<FunctionArrayCell> {
    (0..array.shape().rows)
        .map(|row| array.get(row, col).expect("validated col").clone())
        .collect()
}

fn cell_signature(cell: &FunctionArrayCell) -> String {
    match cell {
        FunctionArrayCell::Number(n) => format!("n:{n:?}"),
        FunctionArrayCell::Text(t) => format!("t:{}", t.to_string_lossy()),
        FunctionArrayCell::Logical(b) => format!("b:{b}"),
        FunctionArrayCell::Error(code) => format!("e:{code:?}"),
        FunctionArrayCell::EmptyCell => "m:".to_string(),
    }
}

fn signature_key(cells: &[FunctionArrayCell]) -> String {
    cells
        .iter()
        .map(cell_signature)
        .collect::<Vec<_>>()
        .join("\u{001f}")
}

fn flatten_cells(array: &FunctionArray, by_col: bool) -> Vec<FunctionArrayCell> {
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

fn compare_cell_values(lhs: &FunctionArrayCell, rhs: &FunctionArrayCell) -> Ordering {
    match (lhs, rhs) {
        (FunctionArrayCell::Number(a), FunctionArrayCell::Number(b)) => {
            a.partial_cmp(b).unwrap_or(Ordering::Equal)
        }
        (FunctionArrayCell::Text(a), FunctionArrayCell::Text(b)) => {
            a.to_string_lossy().cmp(&b.to_string_lossy())
        }
        (FunctionArrayCell::Logical(a), FunctionArrayCell::Logical(b)) => a.cmp(b),
        (FunctionArrayCell::EmptyCell, FunctionArrayCell::EmptyCell) => Ordering::Equal,
        (FunctionArrayCell::Error(a), FunctionArrayCell::Error(b)) => (*a as u8).cmp(&(*b as u8)),
        (FunctionArrayCell::EmptyCell, _) => Ordering::Less,
        (_, FunctionArrayCell::EmptyCell) => Ordering::Greater,
        (FunctionArrayCell::Number(_), _) => Ordering::Less,
        (_, FunctionArrayCell::Number(_)) => Ordering::Greater,
        (FunctionArrayCell::Text(_), _) => Ordering::Less,
        (_, FunctionArrayCell::Text(_)) => Ordering::Greater,
        (FunctionArrayCell::Logical(_), _) => Ordering::Less,
        (_, FunctionArrayCell::Logical(_)) => Ordering::Greater,
    }
}

pub fn eval_choosecols_prepared(
    args: &[PreparedValue],
) -> Result<FunctionValue, DynamicArrayReshapeEvalError> {
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

fn expand_calc_selector_values(
    arg: &CalcValue,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<Vec<CalcValue>, DynamicArrayReshapeEvalError> {
    let prepared = prepare_calc_value_values_only(arg, resolver)
        .map_err(DynamicArrayReshapeEvalError::Preparation)?;
    Ok(match prepared.core() {
        CoreValue::Array(array) => array.iter_row_major().cloned().collect(),
        _ => vec![prepared],
    })
}

pub fn eval_choosecols_calc_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, DynamicArrayReshapeEvalError> {
    if !CHOOSECOLS_META.arity.accepts(args.len()) {
        return Err(surface_arity_error(&CHOOSECOLS_META, args.len()));
    }
    let array = materialize_calc_array_arg(&args[0], resolver)?;
    let selectors = args[1..]
        .iter()
        .map(|arg| expand_calc_selector_values(arg, resolver))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    let cols: Vec<usize> = selectors
        .iter()
        .map(parse_integer_calc)
        .map(|result| result.and_then(|idx| resolve_selector(idx, array.shape().cols)))
        .collect::<Result<_, _>>()?;

    let mut cells = Vec::with_capacity(array.shape().rows * cols.len());
    for row in 0..array.shape().rows {
        for col in &cols {
            cells.push(array.get(row, *col).expect("validated selector").clone());
        }
    }
    build_calc_array(array.shape().rows, cols.len(), cells)
}

pub fn eval_chooserows_prepared(
    args: &[PreparedValue],
) -> Result<FunctionValue, DynamicArrayReshapeEvalError> {
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

pub fn eval_chooserows_calc_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, DynamicArrayReshapeEvalError> {
    if !CHOOSEROWS_META.arity.accepts(args.len()) {
        return Err(surface_arity_error(&CHOOSEROWS_META, args.len()));
    }
    let array = materialize_calc_array_arg(&args[0], resolver)?;
    let selectors = args[1..]
        .iter()
        .map(|arg| expand_calc_selector_values(arg, resolver))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    let rows: Vec<usize> = selectors
        .iter()
        .map(parse_integer_calc)
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
    build_calc_array(rows.len(), array.shape().cols, cells)
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
    args: &[PreparedValue],
) -> Result<FunctionValue, DynamicArrayReshapeEvalError> {
    let array = materialize_array_arg(&args[0]);
    let row_count = match args.get(1) {
        Some(PreparedValue::MissingArg) if args.get(2).is_some() => array.shape().rows as isize,
        Some(arg) => parse_integer(arg)?,
        None => {
            return Err(DynamicArrayReshapeEvalError::ArityMismatch {
                expected_min: TAKE_META.arity.min,
                expected_max: TAKE_META.arity.max,
                actual: args.len(),
            });
        }
    };
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

pub fn eval_take_calc_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, DynamicArrayReshapeEvalError> {
    if !TAKE_META.arity.accepts(args.len()) {
        return Err(surface_arity_error(&TAKE_META, args.len()));
    }
    let array = materialize_calc_array_arg(&args[0], resolver)?;
    let row_count = match args.get(1) {
        Some(value) if matches!(value.core(), CoreValue::Missing) && args.get(2).is_some() => {
            array.shape().rows as isize
        }
        Some(value) => parse_integer_calc(value)?,
        None => return Err(surface_arity_error(&TAKE_META, args.len())),
    };
    let col_count = if let Some(value) = args.get(2) {
        parse_integer_calc(value)?
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
    build_calc_array(rows, cols, cells)
}

pub fn eval_drop_prepared(
    args: &[PreparedValue],
) -> Result<FunctionValue, DynamicArrayReshapeEvalError> {
    let array = materialize_array_arg(&args[0]);
    let row_count = match args.get(1) {
        Some(PreparedValue::MissingArg) if args.get(2).is_some() => 0,
        Some(arg) => parse_integer(arg)?,
        None => {
            return Err(DynamicArrayReshapeEvalError::ArityMismatch {
                expected_min: DROP_META.arity.min,
                expected_max: DROP_META.arity.max,
                actual: args.len(),
            });
        }
    };
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

pub fn eval_drop_calc_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, DynamicArrayReshapeEvalError> {
    if !DROP_META.arity.accepts(args.len()) {
        return Err(surface_arity_error(&DROP_META, args.len()));
    }
    let array = materialize_calc_array_arg(&args[0], resolver)?;
    let row_count = match args.get(1) {
        Some(value) if matches!(value.core(), CoreValue::Missing) && args.get(2).is_some() => 0,
        Some(value) => parse_integer_calc(value)?,
        None => return Err(surface_arity_error(&DROP_META, args.len())),
    };
    let col_count = if let Some(value) = args.get(2) {
        parse_integer_calc(value)?
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
    build_calc_array(rows, cols, cells)
}

pub fn eval_expand_prepared(
    args: &[PreparedValue],
) -> Result<FunctionValue, DynamicArrayReshapeEvalError> {
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

    let pad_cell = if let Some(arg) = args.get(3) {
        if matches!(arg, PreparedValue::Eval(FunctionValue::Array(_))) {
            return Err(DynamicArrayReshapeEvalError::Preparation(
                CoercionError::UnsupportedValueKind("scalar_pad"),
            ));
        }
        scalar_cell(arg)
    } else {
        FunctionArrayCell::Error(WorksheetErrorCode::NA)
    };
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

fn should_ignore_cell(cell: &FunctionArrayCell, ignore_mode: usize) -> bool {
    match ignore_mode {
        0 => false,
        1 => matches!(cell, FunctionArrayCell::EmptyCell),
        2 => matches!(cell, FunctionArrayCell::Error(_)),
        3 => matches!(
            cell,
            FunctionArrayCell::EmptyCell | FunctionArrayCell::Error(_)
        ),
        _ => false,
    }
}

fn parse_ignore_mode(arg: Option<&PreparedValue>) -> Result<usize, DynamicArrayReshapeEvalError> {
    let Some(arg) = arg else {
        return Ok(0);
    };
    if matches!(arg, PreparedValue::MissingArg | PreparedValue::EmptyCell) {
        return Ok(0);
    }
    let mode = parse_integer(arg)?;
    if !(0..=3).contains(&mode) {
        return Err(DynamicArrayReshapeEvalError::InvalidIgnoreMode);
    }
    Ok(mode as usize)
}

pub fn eval_tocol_prepared(
    args: &[PreparedValue],
) -> Result<FunctionValue, DynamicArrayReshapeEvalError> {
    let array = materialize_array_arg(&args[0]);
    let ignore_mode = parse_ignore_mode(args.get(1))?;
    let by_col = args
        .get(2)
        .map(parse_bool_like)
        .transpose()?
        .unwrap_or(false);
    let cells: Vec<FunctionArrayCell> = flatten_cells(&array, by_col)
        .into_iter()
        .filter(|cell| !should_ignore_cell(cell, ignore_mode))
        .collect();
    if cells.is_empty() {
        return Err(DynamicArrayReshapeEvalError::EmptyArrayResult);
    }
    build_array(cells.len(), 1, cells)
}

pub fn eval_torow_prepared(
    args: &[PreparedValue],
) -> Result<FunctionValue, DynamicArrayReshapeEvalError> {
    let array = materialize_array_arg(&args[0]);
    let ignore_mode = parse_ignore_mode(args.get(1))?;
    let by_col = args
        .get(2)
        .map(parse_bool_like)
        .transpose()?
        .unwrap_or(false);
    let cells: Vec<FunctionArrayCell> = flatten_cells(&array, by_col)
        .into_iter()
        .filter(|cell| !should_ignore_cell(cell, ignore_mode))
        .collect();
    if cells.is_empty() {
        return Err(DynamicArrayReshapeEvalError::EmptyArrayResult);
    }
    build_array(1, cells.len(), cells)
}

pub fn eval_transpose_prepared(
    args: &[PreparedValue],
) -> Result<FunctionValue, DynamicArrayReshapeEvalError> {
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
    args: &[PreparedValue],
) -> Result<FunctionValue, DynamicArrayReshapeEvalError> {
    let sources: Vec<StackArgSource<'_>> = args.iter().map(StackArgSource::new).collect();
    let rows: usize = sources.iter().map(|source| source.shape().rows).sum();
    let cols = sources
        .iter()
        .map(|source| source.shape().cols)
        .max()
        .unwrap_or(1);

    let cells = sources.iter().flat_map(|source| {
        (0..source.shape().rows).flat_map(move |row| {
            (0..cols).map(move |col| {
                source
                    .get(row, col)
                    .cloned()
                    .unwrap_or(FunctionArrayCell::Error(WorksheetErrorCode::NA))
            })
        })
    });
    build_array(rows, cols, cells.collect())
}

pub fn eval_wraprows_prepared(
    args: &[PreparedValue],
) -> Result<FunctionValue, DynamicArrayReshapeEvalError> {
    let wrap_count = parse_positive_integer(&args[1])?;
    if let PreparedValue::Eval(value) = &args[0] {
        match value {
            FunctionValue::Number(_)
            | FunctionValue::Text(_)
            | FunctionValue::Logical(_)
            | FunctionValue::Error(_) => return Ok(value.clone()),
            FunctionValue::Array(_) | FunctionValue::Reference(_) => {}
            _ => {}
        }
    }

    let source = materialize_array_arg(&args[0]);
    let pad_cell = args
        .get(2)
        .map(scalar_cell)
        .unwrap_or(FunctionArrayCell::Error(WorksheetErrorCode::NA));
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
    args: &[PreparedValue],
) -> Result<FunctionValue, DynamicArrayReshapeEvalError> {
    let source = materialize_array_arg(&args[0]);
    let wrap_count = parse_positive_integer(&args[1])?;
    let pad_cell = args
        .get(2)
        .map(scalar_cell)
        .unwrap_or(FunctionArrayCell::Error(WorksheetErrorCode::NA));
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
    include: &FunctionArray,
    target_shape: ArrayShape,
) -> Result<(bool, Vec<bool>), DynamicArrayReshapeEvalError> {
    if include.shape().cols == 1 && include.shape().rows == target_shape.rows {
        let mask = (0..include.shape().rows)
            .map(|row| {
                include.get(row, 0).map_or(Ok(false), |cell| match cell {
                    FunctionArrayCell::Logical(b) => Ok(*b),
                    FunctionArrayCell::Number(n) => Ok(*n != 0.0),
                    FunctionArrayCell::EmptyCell => Ok(false),
                    FunctionArrayCell::Error(code) => {
                        Err(DynamicArrayReshapeEvalError::Preparation(
                            CoercionError::WorksheetError(*code),
                        ))
                    }
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
                    FunctionArrayCell::Logical(b) => Ok(*b),
                    FunctionArrayCell::Number(n) => Ok(*n != 0.0),
                    FunctionArrayCell::EmptyCell => Ok(false),
                    FunctionArrayCell::Error(code) => {
                        Err(DynamicArrayReshapeEvalError::Preparation(
                            CoercionError::WorksheetError(*code),
                        ))
                    }
                    _ => Err(DynamicArrayReshapeEvalError::InvalidIncludeShape),
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        return Ok((true, mask));
    }
    Err(DynamicArrayReshapeEvalError::InvalidIncludeShape)
}

pub fn eval_filter_prepared(
    args: &[PreparedValue],
) -> Result<FunctionValue, DynamicArrayReshapeEvalError> {
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
                    PreparedValue::Eval(FunctionValue::Array(array)) => {
                        FunctionValue::Array(array.clone())
                    }
                    other => value_from_if_empty_arg(other),
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
                PreparedValue::Eval(FunctionValue::Array(array)) => {
                    FunctionValue::Array(array.clone())
                }
                other => value_from_if_empty_arg(other),
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

fn parse_sort_order(arg: Option<&PreparedValue>) -> Result<bool, DynamicArrayReshapeEvalError> {
    let Some(arg) = arg else {
        return Ok(false);
    };
    if matches!(arg, PreparedValue::MissingArg | PreparedValue::EmptyCell) {
        return Ok(false);
    }
    match parse_integer(arg)? {
        1 => Ok(false),
        -1 => Ok(true),
        _ => Err(DynamicArrayReshapeEvalError::InvalidSortOrder),
    }
}

fn parse_sort_index(
    arg: Option<&PreparedValue>,
    len: usize,
) -> Result<usize, DynamicArrayReshapeEvalError> {
    let Some(arg) = arg else {
        return Ok(0);
    };
    let scalarized;
    let arg = match arg {
        PreparedValue::Eval(FunctionValue::Array(array)) => {
            scalarized = array
                .get(0, 0)
                .map(prepared_from_array_cell)
                .ok_or(DynamicArrayReshapeEvalError::InvalidSortIndex)?;
            &scalarized
        }
        other => other,
    };
    if matches!(arg, PreparedValue::MissingArg | PreparedValue::EmptyCell) {
        return Ok(0);
    }
    let idx = parse_integer(arg)?;
    if idx < 1 || idx as usize > len {
        return Err(DynamicArrayReshapeEvalError::InvalidSortIndex);
    }
    Ok(idx as usize - 1)
}

pub fn eval_sort_prepared(
    args: &[PreparedValue],
) -> Result<FunctionValue, DynamicArrayReshapeEvalError> {
    let array = materialize_array_arg(&args[0]);
    let by_col = args
        .get(3)
        .map(parse_bool_like)
        .transpose()?
        .unwrap_or(false);
    let descending = parse_sort_order(args.get(2))?;
    let sort_across_columns = by_col;

    if sort_across_columns {
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

#[derive(Debug, Clone)]
struct SortbyKeySpec {
    keys: FunctionArray,
    descending: bool,
}

fn parse_sortby_key_specs(
    args: &[PreparedValue],
) -> Result<Vec<SortbyKeySpec>, DynamicArrayReshapeEvalError> {
    let mut specs = Vec::new();
    let mut idx = 1usize;
    while idx < args.len() {
        let keys = materialize_array_arg(&args[idx]);
        idx += 1;
        let descending = match args.get(idx) {
            None => false,
            Some(PreparedValue::Eval(FunctionValue::Array(_))) => false,
            Some(PreparedValue::MissingArg | PreparedValue::EmptyCell) => {
                idx += 1;
                false
            }
            Some(arg) => {
                let descending = parse_sort_order(Some(arg))?;
                idx += 1;
                descending
            }
        };
        specs.push(SortbyKeySpec { keys, descending });
    }
    Ok(specs)
}

fn sortby_column_keys(
    by_array: &FunctionArray,
    len: usize,
) -> Result<Vec<FunctionArrayCell>, DynamicArrayReshapeEvalError> {
    if by_array.shape().rows == 1 && by_array.shape().cols == len {
        return Ok((0..by_array.shape().cols)
            .map(|col| by_array.get(0, col).expect("validated key").clone())
            .collect());
    }
    if by_array.shape().cols == 1 && by_array.shape().rows == len {
        return Ok((0..by_array.shape().rows)
            .map(|row| by_array.get(row, 0).expect("validated key").clone())
            .collect());
    }
    Err(DynamicArrayReshapeEvalError::InvalidIncludeShape)
}

fn sortby_row_keys(
    by_array: &FunctionArray,
    len: usize,
) -> Result<Vec<FunctionArrayCell>, DynamicArrayReshapeEvalError> {
    if by_array.shape().cols == 1 && by_array.shape().rows == len {
        return Ok((0..by_array.shape().rows)
            .map(|row| by_array.get(row, 0).expect("validated key").clone())
            .collect());
    }
    if by_array.shape().rows == 1 && by_array.shape().cols == len {
        return Ok((0..by_array.shape().cols)
            .map(|col| by_array.get(0, col).expect("validated key").clone())
            .collect());
    }
    Err(DynamicArrayReshapeEvalError::InvalidIncludeShape)
}

pub fn eval_sortby_prepared(
    args: &[PreparedValue],
) -> Result<FunctionValue, DynamicArrayReshapeEvalError> {
    let array = materialize_array_arg(&args[0]);
    let specs = parse_sortby_key_specs(args)?;

    if array.shape().rows == 1 && array.shape().cols > 1 {
        let key_specs = specs
            .iter()
            .map(|spec| {
                sortby_column_keys(&spec.keys, array.shape().cols)
                    .map(|keys| (keys, spec.descending))
            })
            .collect::<Result<Vec<_>, _>>()?;

        let mut order: Vec<usize> = (0..array.shape().cols).collect();
        order.sort_by(|lhs, rhs| {
            for (keys, descending) in &key_specs {
                let ord = compare_cell_values(&keys[*lhs], &keys[*rhs]);
                if ord != Ordering::Equal {
                    return if *descending { ord.reverse() } else { ord };
                }
            }
            Ordering::Equal
        });

        let mut cells = Vec::with_capacity(array.shape().cols);
        for col in order {
            cells.push(array.get(0, col).expect("validated cell").clone());
        }
        return build_array(array.shape().rows, array.shape().cols, cells);
    }

    let key_specs = specs
        .iter()
        .map(|spec| {
            sortby_row_keys(&spec.keys, array.shape().rows).map(|keys| (keys, spec.descending))
        })
        .collect::<Result<Vec<_>, _>>()?;

    let mut order: Vec<usize> = (0..array.shape().rows).collect();
    order.sort_by(|lhs, rhs| {
        for (keys, descending) in &key_specs {
            let ord = compare_cell_values(&keys[*lhs], &keys[*rhs]);
            if ord != Ordering::Equal {
                return if *descending { ord.reverse() } else { ord };
            }
        }
        Ordering::Equal
    });

    let mut cells = Vec::with_capacity(array.shape().rows * array.shape().cols);
    for row in order {
        cells.extend(array.row_slice(row).expect("validated row").iter().cloned());
    }
    build_array(array.shape().rows, array.shape().cols, cells)
}

pub fn eval_unique_prepared(
    args: &[PreparedValue],
) -> Result<FunctionValue, DynamicArrayReshapeEvalError> {
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
    args: &[FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
    meta: &FunctionMeta,
    eval: impl FnOnce(&[PreparedValue]) -> Result<FunctionValue, DynamicArrayReshapeEvalError>,
) -> Result<FunctionValue, DynamicArrayReshapeEvalError> {
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

fn eval_choose_axes_surface(
    args: &[FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
    meta: &FunctionMeta,
    eval: impl FnOnce(&[PreparedValue]) -> Result<FunctionValue, DynamicArrayReshapeEvalError>,
) -> Result<FunctionValue, DynamicArrayReshapeEvalError> {
    if !meta.arity.accepts(args.len()) {
        return Err(surface_arity_error(meta, args.len()));
    }

    let mut prepared = Vec::new();
    prepared.push(
        prepare_arg_values_only(&args[0], resolver)
            .map_err(DynamicArrayReshapeEvalError::Preparation)?,
    );
    for arg in &args[1..] {
        prepared.extend(
            expand_arg_values_only(arg, resolver)
                .map_err(DynamicArrayReshapeEvalError::Preparation)?,
        );
    }
    eval(&prepared)
}

pub fn eval_choosecols_surface(
    args: &[FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<FunctionValue, DynamicArrayReshapeEvalError> {
    eval_choose_axes_surface(args, resolver, &CHOOSECOLS_META, eval_choosecols_prepared)
}

pub fn eval_chooserows_surface(
    args: &[FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<FunctionValue, DynamicArrayReshapeEvalError> {
    eval_choose_axes_surface(args, resolver, &CHOOSEROWS_META, eval_chooserows_prepared)
}

pub fn eval_drop_surface(
    args: &[FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<FunctionValue, DynamicArrayReshapeEvalError> {
    eval_surface_common(args, resolver, &DROP_META, eval_drop_prepared)
}

pub fn eval_expand_surface(
    args: &[FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<FunctionValue, DynamicArrayReshapeEvalError> {
    eval_surface_common(args, resolver, &EXPAND_META, eval_expand_prepared)
}

pub fn eval_filter_surface(
    args: &[FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<FunctionValue, DynamicArrayReshapeEvalError> {
    eval_surface_common(args, resolver, &FILTER_META, eval_filter_prepared)
}

pub fn eval_sort_surface(
    args: &[FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<FunctionValue, DynamicArrayReshapeEvalError> {
    eval_surface_common(args, resolver, &SORT_META, eval_sort_prepared)
}

pub fn eval_sortby_surface(
    args: &[FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<FunctionValue, DynamicArrayReshapeEvalError> {
    eval_surface_common(args, resolver, &SORTBY_META, eval_sortby_prepared)
}

pub fn eval_take_surface(
    args: &[FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<FunctionValue, DynamicArrayReshapeEvalError> {
    eval_surface_common(args, resolver, &TAKE_META, eval_take_prepared)
}

pub fn eval_tocol_surface(
    args: &[FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<FunctionValue, DynamicArrayReshapeEvalError> {
    eval_surface_common(args, resolver, &TOCOL_META, eval_tocol_prepared)
}

pub fn eval_torow_surface(
    args: &[FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<FunctionValue, DynamicArrayReshapeEvalError> {
    eval_surface_common(args, resolver, &TOROW_META, eval_torow_prepared)
}

pub fn eval_transpose_surface(
    args: &[FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<FunctionValue, DynamicArrayReshapeEvalError> {
    eval_surface_common(args, resolver, &TRANSPOSE_META, eval_transpose_prepared)
}

pub fn eval_unique_surface(
    args: &[FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<FunctionValue, DynamicArrayReshapeEvalError> {
    eval_surface_common(args, resolver, &UNIQUE_META, eval_unique_prepared)
}

pub fn eval_vstack_surface(
    args: &[FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<FunctionValue, DynamicArrayReshapeEvalError> {
    eval_surface_common(args, resolver, &VSTACK_META, eval_vstack_prepared)
}

pub fn eval_wrapcols_surface(
    args: &[FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<FunctionValue, DynamicArrayReshapeEvalError> {
    eval_surface_common(args, resolver, &WRAPCOLS_META, eval_wrapcols_prepared)
}

pub fn eval_wraprows_surface(
    args: &[FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<FunctionValue, DynamicArrayReshapeEvalError> {
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
    use crate::resolver::ReferenceSystemCapabilities;
    use crate::value::ExcelText;

    struct NoResolver;

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

    fn array(rows: Vec<Vec<FunctionArrayCell>>) -> FunctionArg {
        FunctionArg::Eval(FunctionValue::Array(
            FunctionArray::from_rows(rows).unwrap(),
        ))
    }

    fn num(n: f64) -> FunctionArg {
        FunctionArg::Eval(FunctionValue::Number(n))
    }

    #[test]
    fn expand_rejects_array_valued_pad_with_as_value_error() {
        let err = eval_expand_surface(
            &[
                array(vec![
                    vec![
                        FunctionArrayCell::Number(1.0),
                        FunctionArrayCell::Number(2.0),
                    ],
                    vec![
                        FunctionArrayCell::Number(3.0),
                        FunctionArrayCell::Number(4.0),
                    ],
                ]),
                num(3.0),
                num(4.0),
                array(vec![vec![
                    FunctionArrayCell::Text(ExcelText::from_interop_assignment("x")),
                    FunctionArrayCell::Text(ExcelText::from_interop_assignment("x")),
                ]]),
            ],
            &NoResolver,
        )
        .unwrap_err();

        assert_eq!(
            map_dynamic_array_reshape_error_to_ws(&err),
            WorksheetErrorCode::Value
        );
    }

    #[test]
    fn sort_uses_top_left_cell_for_array_valued_sort_index() {
        let got = eval_sort_surface(
            &[
                array(vec![
                    vec![
                        FunctionArrayCell::Number(3.0),
                        FunctionArrayCell::Text(ExcelText::from_interop_assignment("c")),
                    ],
                    vec![
                        FunctionArrayCell::Number(1.0),
                        FunctionArrayCell::Text(ExcelText::from_interop_assignment("a")),
                    ],
                    vec![
                        FunctionArrayCell::Number(2.0),
                        FunctionArrayCell::Text(ExcelText::from_interop_assignment("b")),
                    ],
                ]),
                array(vec![vec![
                    FunctionArrayCell::Number(1.0),
                    FunctionArrayCell::Number(1.0),
                ]]),
                num(1.0),
            ],
            &NoResolver,
        )
        .unwrap();

        assert_eq!(
            got,
            FunctionValue::Array(
                FunctionArray::from_rows(vec![
                    vec![
                        FunctionArrayCell::Number(1.0),
                        FunctionArrayCell::Text(ExcelText::from_interop_assignment("a")),
                    ],
                    vec![
                        FunctionArrayCell::Number(2.0),
                        FunctionArrayCell::Text(ExcelText::from_interop_assignment("b")),
                    ],
                    vec![
                        FunctionArrayCell::Number(3.0),
                        FunctionArrayCell::Text(ExcelText::from_interop_assignment("c")),
                    ],
                ])
                .unwrap()
            )
        );
    }

    #[test]
    fn choosecols_and_chooserows_preserve_order_and_duplicates() {
        let cols = eval_choosecols_surface(
            &[
                array(vec![
                    vec![
                        FunctionArrayCell::Number(1.0),
                        FunctionArrayCell::Number(2.0),
                        FunctionArrayCell::Number(3.0),
                    ],
                    vec![
                        FunctionArrayCell::Number(4.0),
                        FunctionArrayCell::Number(5.0),
                        FunctionArrayCell::Number(6.0),
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
            FunctionValue::Array(
                FunctionArray::from_rows(vec![
                    vec![
                        FunctionArrayCell::Number(3.0),
                        FunctionArrayCell::Number(1.0),
                        FunctionArrayCell::Number(3.0),
                    ],
                    vec![
                        FunctionArrayCell::Number(6.0),
                        FunctionArrayCell::Number(4.0),
                        FunctionArrayCell::Number(6.0),
                    ],
                ])
                .unwrap()
            )
        );

        let rows = eval_chooserows_surface(
            &[
                array(vec![
                    vec![FunctionArrayCell::Number(1.0)],
                    vec![FunctionArrayCell::Number(2.0)],
                    vec![FunctionArrayCell::Number(3.0)],
                ]),
                num(-1.0),
                num(1.0),
            ],
            &NoResolver,
        )
        .unwrap();
        assert_eq!(
            rows,
            FunctionValue::Array(
                FunctionArray::from_rows(vec![
                    vec![FunctionArrayCell::Number(3.0)],
                    vec![FunctionArrayCell::Number(1.0)],
                ])
                .unwrap()
            )
        );
    }

    #[test]
    fn choosecols_and_chooserows_flatten_selector_array_arguments() {
        let cols = eval_choosecols_surface(
            &[
                array(vec![vec![
                    FunctionArrayCell::Number(10.0),
                    FunctionArrayCell::Number(20.0),
                    FunctionArrayCell::Number(30.0),
                    FunctionArrayCell::Number(40.0),
                    FunctionArrayCell::Number(50.0),
                ]]),
                array(vec![vec![
                    FunctionArrayCell::Number(3.0),
                    FunctionArrayCell::Number(1.0),
                    FunctionArrayCell::Number(5.0),
                ]]),
            ],
            &NoResolver,
        )
        .unwrap();
        assert_eq!(
            cols,
            FunctionValue::Array(
                FunctionArray::from_rows(vec![vec![
                    FunctionArrayCell::Number(30.0),
                    FunctionArrayCell::Number(10.0),
                    FunctionArrayCell::Number(50.0),
                ]])
                .unwrap()
            )
        );

        let rows = eval_chooserows_surface(
            &[
                array(vec![
                    vec![FunctionArrayCell::Text(ExcelText::from_interop_assignment(
                        "a",
                    ))],
                    vec![FunctionArrayCell::Text(ExcelText::from_interop_assignment(
                        "b",
                    ))],
                    vec![FunctionArrayCell::Text(ExcelText::from_interop_assignment(
                        "c",
                    ))],
                ]),
                array(vec![
                    vec![FunctionArrayCell::Number(3.0)],
                    vec![FunctionArrayCell::Number(1.0)],
                ]),
            ],
            &NoResolver,
        )
        .unwrap();
        assert_eq!(
            rows,
            FunctionValue::Array(
                FunctionArray::from_rows(vec![
                    vec![FunctionArrayCell::Text(ExcelText::from_interop_assignment(
                        "c"
                    ))],
                    vec![FunctionArrayCell::Text(ExcelText::from_interop_assignment(
                        "a"
                    ))],
                ])
                .unwrap()
            )
        );
    }

    #[test]
    fn take_drop_and_expand_match_seeded_slices() {
        let source = array(vec![
            vec![
                FunctionArrayCell::Number(1.0),
                FunctionArrayCell::Number(2.0),
                FunctionArrayCell::Number(3.0),
            ],
            vec![
                FunctionArrayCell::Number(4.0),
                FunctionArrayCell::Number(5.0),
                FunctionArrayCell::Number(6.0),
            ],
            vec![
                FunctionArrayCell::Number(7.0),
                FunctionArrayCell::Number(8.0),
                FunctionArrayCell::Number(9.0),
            ],
        ]);
        let take = eval_take_surface(&[source.clone(), num(2.0), num(-2.0)], &NoResolver).unwrap();
        assert_eq!(
            take,
            FunctionValue::Array(
                FunctionArray::from_rows(vec![
                    vec![
                        FunctionArrayCell::Number(2.0),
                        FunctionArrayCell::Number(3.0)
                    ],
                    vec![
                        FunctionArrayCell::Number(5.0),
                        FunctionArrayCell::Number(6.0)
                    ],
                ])
                .unwrap()
            )
        );

        let drop = eval_drop_surface(&[source.clone(), num(1.0), num(-1.0)], &NoResolver).unwrap();
        assert_eq!(
            drop,
            FunctionValue::Array(
                FunctionArray::from_rows(vec![
                    vec![
                        FunctionArrayCell::Number(4.0),
                        FunctionArrayCell::Number(5.0)
                    ],
                    vec![
                        FunctionArrayCell::Number(7.0),
                        FunctionArrayCell::Number(8.0)
                    ],
                ])
                .unwrap()
            )
        );

        let take_omitted_rows = eval_take_surface(
            &[source.clone(), FunctionArg::MissingArg, num(1.0)],
            &NoResolver,
        )
        .unwrap();
        assert_eq!(
            take_omitted_rows,
            FunctionValue::Array(
                FunctionArray::from_rows(vec![
                    vec![FunctionArrayCell::Number(1.0)],
                    vec![FunctionArrayCell::Number(4.0)],
                    vec![FunctionArrayCell::Number(7.0)],
                ])
                .unwrap()
            )
        );

        let take_omitted_rows_negative = eval_take_surface(
            &[source.clone(), FunctionArg::MissingArg, num(-1.0)],
            &NoResolver,
        )
        .unwrap();
        assert_eq!(
            take_omitted_rows_negative,
            FunctionValue::Array(
                FunctionArray::from_rows(vec![
                    vec![FunctionArrayCell::Number(3.0)],
                    vec![FunctionArrayCell::Number(6.0)],
                    vec![FunctionArrayCell::Number(9.0)],
                ])
                .unwrap()
            )
        );

        let drop_omitted_rows = eval_drop_surface(
            &[source.clone(), FunctionArg::MissingArg, num(1.0)],
            &NoResolver,
        )
        .unwrap();
        assert_eq!(
            drop_omitted_rows,
            FunctionValue::Array(
                FunctionArray::from_rows(vec![
                    vec![
                        FunctionArrayCell::Number(2.0),
                        FunctionArrayCell::Number(3.0)
                    ],
                    vec![
                        FunctionArrayCell::Number(5.0),
                        FunctionArrayCell::Number(6.0)
                    ],
                    vec![
                        FunctionArrayCell::Number(8.0),
                        FunctionArrayCell::Number(9.0)
                    ],
                ])
                .unwrap()
            )
        );

        let drop_omitted_rows_negative = eval_drop_surface(
            &[source.clone(), FunctionArg::MissingArg, num(-1.0)],
            &NoResolver,
        )
        .unwrap();
        assert_eq!(
            drop_omitted_rows_negative,
            FunctionValue::Array(
                FunctionArray::from_rows(vec![
                    vec![
                        FunctionArrayCell::Number(1.0),
                        FunctionArrayCell::Number(2.0)
                    ],
                    vec![
                        FunctionArrayCell::Number(4.0),
                        FunctionArrayCell::Number(5.0)
                    ],
                    vec![
                        FunctionArrayCell::Number(7.0),
                        FunctionArrayCell::Number(8.0)
                    ],
                ])
                .unwrap()
            )
        );

        let expand = eval_expand_surface(
            &[
                source,
                num(4.0),
                num(4.0),
                FunctionArg::Eval(FunctionValue::Text(ExcelText::from_interop_assignment("x"))),
            ],
            &NoResolver,
        )
        .unwrap();
        assert_eq!(
            expand,
            FunctionValue::Array(
                FunctionArray::from_rows(vec![
                    vec![
                        FunctionArrayCell::Number(1.0),
                        FunctionArrayCell::Number(2.0),
                        FunctionArrayCell::Number(3.0),
                        FunctionArrayCell::Text(ExcelText::from_interop_assignment("x")),
                    ],
                    vec![
                        FunctionArrayCell::Number(4.0),
                        FunctionArrayCell::Number(5.0),
                        FunctionArrayCell::Number(6.0),
                        FunctionArrayCell::Text(ExcelText::from_interop_assignment("x")),
                    ],
                    vec![
                        FunctionArrayCell::Number(7.0),
                        FunctionArrayCell::Number(8.0),
                        FunctionArrayCell::Number(9.0),
                        FunctionArrayCell::Text(ExcelText::from_interop_assignment("x")),
                    ],
                    vec![
                        FunctionArrayCell::Text(ExcelText::from_interop_assignment("x")),
                        FunctionArrayCell::Text(ExcelText::from_interop_assignment("x")),
                        FunctionArrayCell::Text(ExcelText::from_interop_assignment("x")),
                        FunctionArrayCell::Text(ExcelText::from_interop_assignment("x")),
                    ],
                ])
                .unwrap()
            )
        );
    }

    #[test]
    fn take_one_by_one_slice_preserves_array_value() {
        let source = array(vec![
            vec![
                FunctionArrayCell::Number(1.0),
                FunctionArrayCell::Number(2.0),
            ],
            vec![
                FunctionArrayCell::Number(3.0),
                FunctionArrayCell::Number(4.0),
            ],
        ]);

        let got = eval_take_surface(&[source, num(1.0), num(1.0)], &NoResolver).unwrap();

        assert_eq!(
            got,
            FunctionValue::Array(
                FunctionArray::from_rows(vec![vec![FunctionArrayCell::Number(1.0)]],).unwrap()
            )
        );
    }

    #[test]
    fn filter_if_empty_scalar_text_returns_scalar_text() {
        let source = array(vec![
            vec![FunctionArrayCell::Number(1.0)],
            vec![FunctionArrayCell::Number(2.0)],
        ]);
        let include = array(vec![
            vec![FunctionArrayCell::Logical(false)],
            vec![FunctionArrayCell::Logical(false)],
        ]);
        let got = eval_filter_surface(
            &[
                source,
                include,
                FunctionArg::Eval(FunctionValue::Text(ExcelText::from_interop_assignment(
                    "none",
                ))),
            ],
            &NoResolver,
        )
        .unwrap();
        assert_eq!(
            got,
            FunctionValue::Text(ExcelText::from_interop_assignment("none"))
        );
    }

    #[test]
    fn tocol_torow_wraprows_wrapcols_and_transpose_match_seeded_shapes() {
        let source = array(vec![
            vec![
                FunctionArrayCell::Number(1.0),
                FunctionArrayCell::EmptyCell,
                FunctionArrayCell::Number(3.0),
            ],
            vec![
                FunctionArrayCell::Number(4.0),
                FunctionArrayCell::Error(WorksheetErrorCode::NA),
                FunctionArrayCell::Number(6.0),
            ],
        ]);
        let tocol = eval_tocol_surface(
            &[
                source.clone(),
                num(3.0),
                FunctionArg::Eval(FunctionValue::Logical(false)),
            ],
            &NoResolver,
        )
        .unwrap();
        assert_eq!(
            tocol,
            FunctionValue::Array(
                FunctionArray::from_rows(vec![
                    vec![FunctionArrayCell::Number(1.0)],
                    vec![FunctionArrayCell::Number(3.0)],
                    vec![FunctionArrayCell::Number(4.0)],
                    vec![FunctionArrayCell::Number(6.0)],
                ])
                .unwrap()
            )
        );

        let torow = eval_torow_surface(
            &[
                source.clone(),
                num(0.0),
                FunctionArg::Eval(FunctionValue::Logical(true)),
            ],
            &NoResolver,
        )
        .unwrap();
        assert_eq!(
            torow,
            FunctionValue::Array(
                FunctionArray::from_rows(vec![vec![
                    FunctionArrayCell::Number(1.0),
                    FunctionArrayCell::Number(4.0),
                    FunctionArrayCell::EmptyCell,
                    FunctionArrayCell::Error(WorksheetErrorCode::NA),
                    FunctionArrayCell::Number(3.0),
                    FunctionArrayCell::Number(6.0),
                ]])
                .unwrap()
            )
        );

        let wraprows = eval_wraprows_surface(
            &[
                array(vec![vec![
                    FunctionArrayCell::Number(1.0),
                    FunctionArrayCell::Number(2.0),
                    FunctionArrayCell::Number(3.0),
                    FunctionArrayCell::Number(4.0),
                    FunctionArrayCell::Number(5.0),
                ]]),
                num(2.0),
            ],
            &NoResolver,
        )
        .unwrap();
        assert_eq!(
            wraprows,
            FunctionValue::Array(
                FunctionArray::from_rows(vec![
                    vec![
                        FunctionArrayCell::Number(1.0),
                        FunctionArrayCell::Number(2.0)
                    ],
                    vec![
                        FunctionArrayCell::Number(3.0),
                        FunctionArrayCell::Number(4.0)
                    ],
                    vec![
                        FunctionArrayCell::Number(5.0),
                        FunctionArrayCell::Error(WorksheetErrorCode::NA),
                    ],
                ])
                .unwrap()
            )
        );

        let wraprows_scalar = eval_wraprows_surface(&[num(0.0), num(7.0)], &NoResolver).unwrap();
        assert_eq!(wraprows_scalar, FunctionValue::Number(0.0));

        let wrapcols = eval_wrapcols_surface(
            &[
                array(vec![vec![
                    FunctionArrayCell::Number(1.0),
                    FunctionArrayCell::Number(2.0),
                    FunctionArrayCell::Number(3.0),
                    FunctionArrayCell::Number(4.0),
                    FunctionArrayCell::Number(5.0),
                ]]),
                num(2.0),
            ],
            &NoResolver,
        )
        .unwrap();
        assert_eq!(
            wrapcols,
            FunctionValue::Array(
                FunctionArray::from_rows(vec![
                    vec![
                        FunctionArrayCell::Number(1.0),
                        FunctionArrayCell::Number(3.0),
                        FunctionArrayCell::Number(5.0),
                    ],
                    vec![
                        FunctionArrayCell::Number(2.0),
                        FunctionArrayCell::Number(4.0),
                        FunctionArrayCell::Error(WorksheetErrorCode::NA),
                    ],
                ])
                .unwrap()
            )
        );

        let transpose = eval_transpose_surface(&[source], &NoResolver).unwrap();
        assert_eq!(
            transpose,
            FunctionValue::Array(
                FunctionArray::from_rows(vec![
                    vec![
                        FunctionArrayCell::Number(1.0),
                        FunctionArrayCell::Number(4.0)
                    ],
                    vec![
                        FunctionArrayCell::EmptyCell,
                        FunctionArrayCell::Error(WorksheetErrorCode::NA)
                    ],
                    vec![
                        FunctionArrayCell::Number(3.0),
                        FunctionArrayCell::Number(6.0)
                    ],
                ])
                .unwrap()
            )
        );
    }

    #[test]
    fn ftc_0917_sort_row_vector_default_axis_direct_call_preserves_row_order() {
        let sort_0917 = eval_sort_surface(
            &[
                array(vec![vec![
                    FunctionArrayCell::Number(3.0),
                    FunctionArrayCell::Number(1.0),
                    FunctionArrayCell::Number(4.0),
                    FunctionArrayCell::Number(1.0),
                    FunctionArrayCell::Number(5.0),
                    FunctionArrayCell::Number(9.0),
                    FunctionArrayCell::Number(2.0),
                    FunctionArrayCell::Number(6.0),
                ]]),
                num(1.0),
                num(1.0),
            ],
            &NoResolver,
        )
        .unwrap();
        assert_eq!(
            sort_0917,
            FunctionValue::Array(
                FunctionArray::from_rows(vec![vec![
                    FunctionArrayCell::Number(3.0),
                    FunctionArrayCell::Number(1.0),
                    FunctionArrayCell::Number(4.0),
                    FunctionArrayCell::Number(1.0),
                    FunctionArrayCell::Number(5.0),
                    FunctionArrayCell::Number(9.0),
                    FunctionArrayCell::Number(2.0),
                    FunctionArrayCell::Number(6.0),
                ]])
                .unwrap()
            )
        );
    }

    #[test]
    fn ftc_0836_sortby_row_vector_multi_key_direct_call_matches_witness() {
        let got = eval_sortby_surface(
            &[
                array(vec![vec![
                    FunctionArrayCell::Text(ExcelText::from_interop_assignment("a")),
                    FunctionArrayCell::Text(ExcelText::from_interop_assignment("b")),
                    FunctionArrayCell::Text(ExcelText::from_interop_assignment("c")),
                    FunctionArrayCell::Text(ExcelText::from_interop_assignment("d")),
                ]]),
                array(vec![vec![
                    FunctionArrayCell::Number(2.0),
                    FunctionArrayCell::Number(1.0),
                    FunctionArrayCell::Number(2.0),
                    FunctionArrayCell::Number(1.0),
                ]]),
                array(vec![vec![
                    FunctionArrayCell::Number(1.0),
                    FunctionArrayCell::Number(2.0),
                    FunctionArrayCell::Number(1.0),
                    FunctionArrayCell::Number(2.0),
                ]]),
            ],
            &NoResolver,
        )
        .unwrap();
        assert_eq!(
            got,
            FunctionValue::Array(
                FunctionArray::from_rows(vec![vec![
                    FunctionArrayCell::Text(ExcelText::from_interop_assignment("b")),
                    FunctionArrayCell::Text(ExcelText::from_interop_assignment("d")),
                    FunctionArrayCell::Text(ExcelText::from_interop_assignment("a")),
                    FunctionArrayCell::Text(ExcelText::from_interop_assignment("c")),
                ]])
                .unwrap()
            )
        );
    }

    #[test]
    fn filter_sort_sortby_unique_and_vstack_match_seeded_lanes() {
        let filter = eval_filter_surface(
            &[
                array(vec![
                    vec![
                        FunctionArrayCell::Number(1.0),
                        FunctionArrayCell::Number(10.0),
                    ],
                    vec![
                        FunctionArrayCell::Number(2.0),
                        FunctionArrayCell::Number(20.0),
                    ],
                    vec![
                        FunctionArrayCell::Number(3.0),
                        FunctionArrayCell::Number(30.0),
                    ],
                ]),
                array(vec![
                    vec![FunctionArrayCell::Logical(true)],
                    vec![FunctionArrayCell::Logical(false)],
                    vec![FunctionArrayCell::Logical(true)],
                ]),
            ],
            &NoResolver,
        )
        .unwrap();
        assert_eq!(
            filter,
            FunctionValue::Array(
                FunctionArray::from_rows(vec![
                    vec![
                        FunctionArrayCell::Number(1.0),
                        FunctionArrayCell::Number(10.0)
                    ],
                    vec![
                        FunctionArrayCell::Number(3.0),
                        FunctionArrayCell::Number(30.0)
                    ],
                ])
                .unwrap()
            )
        );

        let sort = eval_sort_surface(
            &[
                array(vec![
                    vec![
                        FunctionArrayCell::Number(3.0),
                        FunctionArrayCell::Text(ExcelText::from_interop_assignment("c")),
                    ],
                    vec![
                        FunctionArrayCell::Number(1.0),
                        FunctionArrayCell::Text(ExcelText::from_interop_assignment("a")),
                    ],
                    vec![
                        FunctionArrayCell::Number(2.0),
                        FunctionArrayCell::Text(ExcelText::from_interop_assignment("b")),
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
            FunctionValue::Array(
                FunctionArray::from_rows(vec![
                    vec![
                        FunctionArrayCell::Number(1.0),
                        FunctionArrayCell::Text(ExcelText::from_interop_assignment("a")),
                    ],
                    vec![
                        FunctionArrayCell::Number(2.0),
                        FunctionArrayCell::Text(ExcelText::from_interop_assignment("b")),
                    ],
                    vec![
                        FunctionArrayCell::Number(3.0),
                        FunctionArrayCell::Text(ExcelText::from_interop_assignment("c")),
                    ],
                ])
                .unwrap()
            )
        );

        let sort_missing_index = eval_sort_surface(
            &[
                array(vec![
                    vec![FunctionArrayCell::Number(2.0)],
                    vec![FunctionArrayCell::Number(3.0)],
                    vec![FunctionArrayCell::Number(7.0)],
                    vec![FunctionArrayCell::Number(5.0)],
                ]),
                FunctionArg::MissingArg,
                num(-1.0),
            ],
            &NoResolver,
        )
        .unwrap();
        assert_eq!(
            sort_missing_index,
            FunctionValue::Array(
                FunctionArray::from_rows(vec![
                    vec![FunctionArrayCell::Number(7.0)],
                    vec![FunctionArrayCell::Number(5.0)],
                    vec![FunctionArrayCell::Number(3.0)],
                    vec![FunctionArrayCell::Number(2.0)],
                ])
                .unwrap()
            )
        );

        let sortby = eval_sortby_surface(
            &[
                array(vec![
                    vec![FunctionArrayCell::Text(ExcelText::from_interop_assignment(
                        "alpha",
                    ))],
                    vec![FunctionArrayCell::Text(ExcelText::from_interop_assignment(
                        "beta",
                    ))],
                    vec![FunctionArrayCell::Text(ExcelText::from_interop_assignment(
                        "gamma",
                    ))],
                ]),
                array(vec![
                    vec![FunctionArrayCell::Number(2.0)],
                    vec![FunctionArrayCell::Number(3.0)],
                    vec![FunctionArrayCell::Number(1.0)],
                ]),
                num(1.0),
            ],
            &NoResolver,
        )
        .unwrap();
        assert_eq!(
            sortby,
            FunctionValue::Array(
                FunctionArray::from_rows(vec![
                    vec![FunctionArrayCell::Text(ExcelText::from_interop_assignment(
                        "gamma"
                    ))],
                    vec![FunctionArrayCell::Text(ExcelText::from_interop_assignment(
                        "alpha"
                    ))],
                    vec![FunctionArrayCell::Text(ExcelText::from_interop_assignment(
                        "beta"
                    ))],
                ])
                .unwrap()
            )
        );

        let sortby_missing_order = eval_sortby_surface(
            &[
                array(vec![
                    vec![FunctionArrayCell::Text(ExcelText::from_interop_assignment(
                        "alpha",
                    ))],
                    vec![FunctionArrayCell::Text(ExcelText::from_interop_assignment(
                        "beta",
                    ))],
                    vec![FunctionArrayCell::Text(ExcelText::from_interop_assignment(
                        "gamma",
                    ))],
                ]),
                array(vec![
                    vec![FunctionArrayCell::Number(2.0)],
                    vec![FunctionArrayCell::Number(3.0)],
                    vec![FunctionArrayCell::Number(1.0)],
                ]),
                FunctionArg::MissingArg,
            ],
            &NoResolver,
        )
        .unwrap();
        assert_eq!(
            sortby_missing_order,
            FunctionValue::Array(
                FunctionArray::from_rows(vec![
                    vec![FunctionArrayCell::Text(ExcelText::from_interop_assignment(
                        "gamma"
                    ))],
                    vec![FunctionArrayCell::Text(ExcelText::from_interop_assignment(
                        "alpha"
                    ))],
                    vec![FunctionArrayCell::Text(ExcelText::from_interop_assignment(
                        "beta"
                    ))],
                ])
                .unwrap()
            )
        );

        let unique = eval_unique_surface(
            &[
                array(vec![
                    vec![
                        FunctionArrayCell::Number(1.0),
                        FunctionArrayCell::Number(10.0),
                    ],
                    vec![
                        FunctionArrayCell::Number(1.0),
                        FunctionArrayCell::Number(10.0),
                    ],
                    vec![
                        FunctionArrayCell::Number(2.0),
                        FunctionArrayCell::Number(20.0),
                    ],
                ]),
                FunctionArg::Eval(FunctionValue::Logical(false)),
                FunctionArg::Eval(FunctionValue::Logical(false)),
            ],
            &NoResolver,
        )
        .unwrap();
        assert_eq!(
            unique,
            FunctionValue::Array(
                FunctionArray::from_rows(vec![
                    vec![
                        FunctionArrayCell::Number(1.0),
                        FunctionArrayCell::Number(10.0)
                    ],
                    vec![
                        FunctionArrayCell::Number(2.0),
                        FunctionArrayCell::Number(20.0)
                    ],
                ])
                .unwrap()
            )
        );

        let vstack = eval_vstack_surface(
            &[
                array(vec![vec![
                    FunctionArrayCell::Number(1.0),
                    FunctionArrayCell::Number(2.0),
                ]]),
                array(vec![
                    vec![FunctionArrayCell::Number(3.0)],
                    vec![FunctionArrayCell::Number(4.0)],
                ]),
            ],
            &NoResolver,
        )
        .unwrap();
        assert_eq!(
            vstack,
            FunctionValue::Array(
                FunctionArray::from_rows(vec![
                    vec![
                        FunctionArrayCell::Number(1.0),
                        FunctionArrayCell::Number(2.0)
                    ],
                    vec![
                        FunctionArrayCell::Number(3.0),
                        FunctionArrayCell::Error(WorksheetErrorCode::NA)
                    ],
                    vec![
                        FunctionArrayCell::Number(4.0),
                        FunctionArrayCell::Error(WorksheetErrorCode::NA)
                    ],
                ])
                .unwrap()
            )
        );
    }
}

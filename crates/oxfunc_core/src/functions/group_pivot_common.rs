use std::collections::HashMap;

use crate::functions::adapters::{coerce_prepared_to_number, prepared_arg_to_calc_value_lossy};
use crate::functions::callable_helpers::{
    CallableInvocationError, CallableInvoker, LambdaHelperEvalError, invoke_callable_prepared,
};
use crate::value::{CalcArray, CalcValue, CallableValue, CoreValue, ExcelText, WorksheetErrorCode};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum CellKey {
    Number(u64),
    Text(Vec<u16>),
    Logical(bool),
    Error(u8),
    EmptyCell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FieldHeadersMode {
    Auto,
    No,
    YesHide,
    NoGenerate,
    YesShow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FieldRelationship {
    Hierarchical,
    Tabular,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct MatrixArg {
    pub array: CalcArray,
    pub had_headers: bool,
}

pub(crate) fn require_callable(
    prepared: &CalcValue,
) -> Result<CallableValue, LambdaHelperEvalError> {
    match prepared.core() {
        CoreValue::Error(code) => Err(LambdaHelperEvalError::Invocation(
            CallableInvocationError::Worksheet(*code),
        )),
        _ => prepared_arg_to_calc_value_lossy(prepared)
            .callable_value()
            .cloned()
            .ok_or_else(|| {
                LambdaHelperEvalError::Invocation(CallableInvocationError::Worksheet(
                    WorksheetErrorCode::Value,
                ))
            }),
    }
}

pub(crate) fn require_calc_callable(
    value: &CalcValue,
) -> Result<CallableValue, LambdaHelperEvalError> {
    match value.core() {
        CoreValue::Error(code) if value.callable_value().is_none() => Err(
            LambdaHelperEvalError::Invocation(CallableInvocationError::Worksheet(*code)),
        ),
        _ => value.callable_value().cloned().ok_or_else(|| {
            LambdaHelperEvalError::Invocation(CallableInvocationError::Worksheet(
                WorksheetErrorCode::Value,
            ))
        }),
    }
}

pub(crate) fn scalar_cell_from_prepared(
    prepared: &CalcValue,
) -> Result<CalcValue, LambdaHelperEvalError> {
    match prepared.core() {
        CoreValue::Number(n) => Ok(CalcValue::number(*n)),
        CoreValue::Text(t) => Ok(CalcValue::text(t.clone())),
        CoreValue::Logical(b) => Ok(CalcValue::logical(*b)),
        CoreValue::Error(code) => Ok(CalcValue::error(*code)),
        CoreValue::Missing | CoreValue::Empty => Ok(CalcValue::empty()),
        CoreValue::Array(_) => Err(LambdaHelperEvalError::NonScalarHelperResult),
        CoreValue::Reference(_) => Err(LambdaHelperEvalError::Invocation(
            CallableInvocationError::Worksheet(WorksheetErrorCode::Value),
        )),
    }
}

pub(crate) fn prepared_to_array(prepared: &CalcValue) -> CalcArray {
    match prepared.core() {
        CoreValue::Array(array) => array.clone(),
        CoreValue::Number(n) => CalcArray::from_scalar(CalcValue::number(*n))
            .expect("scalar number array has valid dimensions"),
        CoreValue::Text(t) => CalcArray::from_scalar(CalcValue::text(t.clone()))
            .expect("scalar text array has valid dimensions"),
        CoreValue::Logical(b) => CalcArray::from_scalar(CalcValue::logical(*b))
            .expect("scalar logical array has valid dimensions"),
        CoreValue::Error(code) => CalcArray::from_scalar(CalcValue::error(*code))
            .expect("scalar error array has valid dimensions"),
        CoreValue::Missing | CoreValue::Empty => CalcArray::from_scalar(CalcValue::empty())
            .expect("scalar empty array has valid dimensions"),
        CoreValue::Reference(_) => {
            CalcArray::from_scalar(CalcValue::error(WorksheetErrorCode::Value))
                .expect("scalar error array has valid dimensions")
        }
    }
}

pub(crate) fn coerce_optional_i32(
    prepared: Option<&CalcValue>,
) -> Result<Option<i32>, LambdaHelperEvalError> {
    let Some(prepared) = prepared else {
        return Ok(None);
    };
    if matches!(prepared.core(), CoreValue::Missing | CoreValue::Empty) {
        return Ok(None);
    }
    let raw = coerce_prepared_to_number(prepared).map_err(LambdaHelperEvalError::Preparation)?;
    if !raw.is_finite() || raw.fract() != 0.0 {
        return Err(LambdaHelperEvalError::Invocation(
            CallableInvocationError::Worksheet(WorksheetErrorCode::Value),
        ));
    }
    Ok(Some(raw as i32))
}

pub(crate) fn parse_field_headers_mode(
    prepared: Option<&CalcValue>,
) -> Result<FieldHeadersMode, LambdaHelperEvalError> {
    match coerce_optional_i32(prepared)? {
        None => Ok(FieldHeadersMode::Auto),
        Some(0) => Ok(FieldHeadersMode::No),
        Some(1) => Ok(FieldHeadersMode::YesHide),
        Some(2) => Ok(FieldHeadersMode::NoGenerate),
        Some(3) => Ok(FieldHeadersMode::YesShow),
        _ => Err(LambdaHelperEvalError::Invocation(
            CallableInvocationError::Worksheet(WorksheetErrorCode::Value),
        )),
    }
}

pub(crate) fn parse_field_relationship(
    prepared: Option<&CalcValue>,
) -> Result<FieldRelationship, LambdaHelperEvalError> {
    match coerce_optional_i32(prepared)? {
        None | Some(0) => Ok(FieldRelationship::Hierarchical),
        Some(1) => Ok(FieldRelationship::Tabular),
        _ => Err(LambdaHelperEvalError::Invocation(
            CallableInvocationError::Worksheet(WorksheetErrorCode::Value),
        )),
    }
}

fn cell_looks_numeric(cell: &CalcValue) -> bool {
    matches!(cell.core(), CoreValue::Number(_))
}

fn cell_looks_text(cell: &CalcValue) -> bool {
    matches!(cell.core(), CoreValue::Text(_))
}

pub(crate) fn detect_headers(values: &CalcArray) -> bool {
    if values.shape().rows < 2 || values.shape().cols == 0 {
        return false;
    }
    let Some(first) = values.get(0, 0) else {
        return false;
    };
    let Some(second) = values.get(1, 0) else {
        return false;
    };
    cell_looks_text(first) && cell_looks_numeric(second)
}

pub(crate) fn split_header_row(
    array: &CalcArray,
    mode: FieldHeadersMode,
) -> Result<MatrixArg, LambdaHelperEvalError> {
    let inferred_headers = detect_headers(array);
    let had_headers = match mode {
        FieldHeadersMode::Auto => inferred_headers,
        FieldHeadersMode::YesHide | FieldHeadersMode::YesShow => true,
        FieldHeadersMode::No | FieldHeadersMode::NoGenerate => false,
    };

    if had_headers {
        if array.shape().rows < 2 {
            return Err(LambdaHelperEvalError::Invocation(
                CallableInvocationError::Worksheet(WorksheetErrorCode::Value),
            ));
        }
        let rows = (1..array.shape().rows)
            .map(|row| {
                (0..array.shape().cols)
                    .map(|col| array.get(row, col).cloned().expect("validated cell"))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let array = CalcArray::from_rows(rows).expect("header split preserves rectangular shape");
        Ok(MatrixArg {
            array,
            had_headers: true,
        })
    } else {
        Ok(MatrixArg {
            array: array.clone(),
            had_headers: false,
        })
    }
}

pub(crate) fn row_as_cells(array: &CalcArray, row: usize) -> Vec<CalcValue> {
    (0..array.shape().cols)
        .map(|col| array.get(row, col).cloned().expect("validated row cell"))
        .collect()
}

pub(crate) fn key_from_cells(cells: &[CalcValue]) -> Vec<CellKey> {
    cells.iter().map(cell_key).collect()
}

pub(crate) fn cell_key(cell: &CalcValue) -> CellKey {
    match cell.core() {
        CoreValue::Number(n) => CellKey::Number(n.to_bits()),
        CoreValue::Text(t) => CellKey::Text(t.utf16_code_units().to_vec()),
        CoreValue::Logical(b) => CellKey::Logical(*b),
        CoreValue::Error(code) => CellKey::Error(*code as u8),
        CoreValue::Empty | CoreValue::Missing => CellKey::EmptyCell,
        CoreValue::Array(_) | CoreValue::Reference(_) => CellKey::EmptyCell,
    }
}

pub(crate) fn default_row_field_headers(cols: usize) -> Vec<CalcValue> {
    (1..=cols)
        .map(|i| {
            CalcValue::text(ExcelText::from_utf16_code_units(
                format!("Row Field {i}").encode_utf16().collect(),
            ))
        })
        .collect()
}

pub(crate) fn default_column_field_headers(cols: usize) -> Vec<CalcValue> {
    (1..=cols)
        .map(|i| {
            CalcValue::text(ExcelText::from_utf16_code_units(
                format!("Column Field {i}").encode_utf16().collect(),
            ))
        })
        .collect()
}

pub(crate) fn default_value_headers(cols: usize) -> Vec<CalcValue> {
    (1..=cols)
        .map(|i| {
            CalcValue::text(ExcelText::from_utf16_code_units(
                format!("Value {i}").encode_utf16().collect(),
            ))
        })
        .collect()
}

pub(crate) fn take_header_row(array: &CalcArray) -> Vec<CalcValue> {
    row_as_cells(array, 0)
}

pub(crate) fn parse_filter_vector(
    prepared: Option<&CalcValue>,
    expected_rows: usize,
) -> Result<Option<Vec<bool>>, LambdaHelperEvalError> {
    let Some(prepared) = prepared else {
        return Ok(None);
    };
    if matches!(prepared.core(), CoreValue::Missing | CoreValue::Empty) {
        return Ok(None);
    }
    let array = prepared_to_array(prepared);
    let shape = array.shape();
    if shape.cols != 1 && shape.rows != 1 {
        return Err(LambdaHelperEvalError::Invocation(
            CallableInvocationError::Worksheet(WorksheetErrorCode::Value),
        ));
    }
    let items = array
        .iter_row_major()
        .map(coerce_cell_to_bool)
        .collect::<Result<Vec<_>, _>>()?;
    if items.len() != expected_rows {
        return Err(LambdaHelperEvalError::Invocation(
            CallableInvocationError::Worksheet(WorksheetErrorCode::Value),
        ));
    }
    Ok(Some(items))
}

fn coerce_cell_to_bool(cell: &CalcValue) -> Result<bool, LambdaHelperEvalError> {
    match cell.core() {
        CoreValue::Logical(b) => Ok(*b),
        CoreValue::Number(n) => Ok(*n != 0.0),
        CoreValue::Empty | CoreValue::Missing => Ok(false),
        CoreValue::Error(code) => Err(LambdaHelperEvalError::Invocation(
            CallableInvocationError::Worksheet(*code),
        )),
        CoreValue::Text(_) | CoreValue::Array(_) | CoreValue::Reference(_) => {
            Err(LambdaHelperEvalError::Invocation(
                CallableInvocationError::Worksheet(WorksheetErrorCode::Value),
            ))
        }
    }
}

pub(crate) fn parse_sort_orders(
    prepared: Option<&CalcValue>,
) -> Result<Vec<i32>, LambdaHelperEvalError> {
    let Some(prepared) = prepared else {
        return Ok(Vec::new());
    };
    if matches!(prepared.core(), CoreValue::Missing | CoreValue::Empty) {
        return Ok(Vec::new());
    }
    let array = prepared_to_array(prepared);
    let shape = array.shape();
    if shape.rows != 1 && shape.cols != 1 {
        return Err(LambdaHelperEvalError::Invocation(
            CallableInvocationError::Worksheet(WorksheetErrorCode::Value),
        ));
    }
    array
        .iter_row_major()
        .map(|cell| match cell.core() {
            CoreValue::Number(n) if n.is_finite() && n.fract() == 0.0 && *n != 0.0 => Ok(*n as i32),
            _ => Err(LambdaHelperEvalError::Invocation(
                CallableInvocationError::Worksheet(WorksheetErrorCode::Value),
            )),
        })
        .collect()
}

pub(crate) fn invoke_group_aggregate(
    callable: &CallableValue,
    values: &[CalcValue],
    invoker: &(impl CallableInvoker + ?Sized),
) -> Result<CalcValue, LambdaHelperEvalError> {
    let column = if values.is_empty() {
        vec![vec![CalcValue::empty()]]
    } else {
        values
            .iter()
            .cloned()
            .map(|cell| vec![cell])
            .collect::<Vec<_>>()
    };
    let arg = CalcValue::array(CalcArray::from_rows(column).expect("column vector shape is valid"));
    let prepared = invoke_callable_prepared(callable, &[arg], invoker)
        .map_err(LambdaHelperEvalError::Invocation)?;
    scalar_cell_from_prepared(&prepared)
}

pub(crate) fn text_cell(text: &str) -> CalcValue {
    CalcValue::text(ExcelText::from_utf16_code_units(
        text.encode_utf16().collect(),
    ))
}

pub(crate) fn group_indices_by_key(rows: usize, keys: &[Vec<CellKey>]) -> Vec<Vec<usize>> {
    let mut order: Vec<Vec<usize>> = Vec::new();
    let mut seen = HashMap::<Vec<CellKey>, usize>::new();
    for row in 0..rows {
        let key = keys[row].clone();
        if let Some(index) = seen.get(&key) {
            order[*index].push(row);
        } else {
            seen.insert(key, order.len());
            order.push(vec![row]);
        }
    }
    order
}

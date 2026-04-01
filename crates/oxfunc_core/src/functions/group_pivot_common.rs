use std::collections::HashMap;

use crate::functions::adapters::{PreparedArgValue, coerce_prepared_to_number};
use crate::functions::callable_helpers::{
    CallableInvocationError, CallableInvoker, LambdaHelperEvalError, invoke_callable_prepared,
};
use crate::value::{
    ArrayCellValue, EvalArray, EvalValue, ExcelText, LambdaValue, WorksheetErrorCode,
};

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
    pub array: EvalArray,
    pub had_headers: bool,
}

pub(crate) fn require_callable(
    prepared: &PreparedArgValue,
) -> Result<&LambdaValue, LambdaHelperEvalError> {
    match prepared {
        PreparedArgValue::Eval(EvalValue::Lambda(callable)) => Ok(callable),
        PreparedArgValue::Eval(EvalValue::Error(code)) => Err(LambdaHelperEvalError::Invocation(
            CallableInvocationError::Worksheet(*code),
        )),
        _ => Err(LambdaHelperEvalError::Invocation(
            CallableInvocationError::Worksheet(WorksheetErrorCode::Value),
        )),
    }
}

pub(crate) fn scalar_cell_from_prepared(
    prepared: &PreparedArgValue,
) -> Result<ArrayCellValue, LambdaHelperEvalError> {
    match prepared {
        PreparedArgValue::Eval(EvalValue::Number(n)) => Ok(ArrayCellValue::Number(*n)),
        PreparedArgValue::Eval(EvalValue::Text(t)) => Ok(ArrayCellValue::Text(t.clone())),
        PreparedArgValue::Eval(EvalValue::Logical(b)) => Ok(ArrayCellValue::Logical(*b)),
        PreparedArgValue::Eval(EvalValue::Error(code)) => Ok(ArrayCellValue::Error(*code)),
        PreparedArgValue::MissingArg | PreparedArgValue::EmptyCell => Ok(ArrayCellValue::EmptyCell),
        PreparedArgValue::Eval(EvalValue::Array(_)) => {
            Err(LambdaHelperEvalError::NonScalarHelperResult)
        }
        PreparedArgValue::Eval(EvalValue::Reference(_))
        | PreparedArgValue::Eval(EvalValue::Lambda(_)) => Err(LambdaHelperEvalError::Invocation(
            CallableInvocationError::Worksheet(WorksheetErrorCode::Value),
        )),
    }
}

pub(crate) fn prepared_to_array(prepared: &PreparedArgValue) -> EvalArray {
    match prepared {
        PreparedArgValue::Eval(EvalValue::Array(array)) => array.clone(),
        PreparedArgValue::Eval(EvalValue::Number(n)) => {
            EvalArray::from_scalar(ArrayCellValue::Number(*n))
        }
        PreparedArgValue::Eval(EvalValue::Text(t)) => {
            EvalArray::from_scalar(ArrayCellValue::Text(t.clone()))
        }
        PreparedArgValue::Eval(EvalValue::Logical(b)) => {
            EvalArray::from_scalar(ArrayCellValue::Logical(*b))
        }
        PreparedArgValue::Eval(EvalValue::Error(code)) => {
            EvalArray::from_scalar(ArrayCellValue::Error(*code))
        }
        PreparedArgValue::MissingArg | PreparedArgValue::EmptyCell => {
            EvalArray::from_scalar(ArrayCellValue::EmptyCell)
        }
        PreparedArgValue::Eval(EvalValue::Reference(_))
        | PreparedArgValue::Eval(EvalValue::Lambda(_)) => {
            EvalArray::from_scalar(ArrayCellValue::Error(WorksheetErrorCode::Value))
        }
    }
}

pub(crate) fn coerce_optional_i32(
    prepared: Option<&PreparedArgValue>,
) -> Result<Option<i32>, LambdaHelperEvalError> {
    let Some(prepared) = prepared else {
        return Ok(None);
    };
    if matches!(
        prepared,
        PreparedArgValue::MissingArg | PreparedArgValue::EmptyCell
    ) {
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
    prepared: Option<&PreparedArgValue>,
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
    prepared: Option<&PreparedArgValue>,
) -> Result<FieldRelationship, LambdaHelperEvalError> {
    match coerce_optional_i32(prepared)? {
        None | Some(0) => Ok(FieldRelationship::Hierarchical),
        Some(1) => Ok(FieldRelationship::Tabular),
        _ => Err(LambdaHelperEvalError::Invocation(
            CallableInvocationError::Worksheet(WorksheetErrorCode::Value),
        )),
    }
}

fn cell_looks_numeric(cell: &ArrayCellValue) -> bool {
    matches!(cell, ArrayCellValue::Number(_))
}

fn cell_looks_text(cell: &ArrayCellValue) -> bool {
    matches!(cell, ArrayCellValue::Text(_))
}

pub(crate) fn detect_headers(values: &EvalArray) -> bool {
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
    array: &EvalArray,
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
        let array = EvalArray::from_rows(rows).expect("header split preserves rectangular shape");
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

pub(crate) fn row_as_cells(array: &EvalArray, row: usize) -> Vec<ArrayCellValue> {
    (0..array.shape().cols)
        .map(|col| array.get(row, col).cloned().expect("validated row cell"))
        .collect()
}

pub(crate) fn key_from_cells(cells: &[ArrayCellValue]) -> Vec<CellKey> {
    cells.iter().map(cell_key).collect()
}

pub(crate) fn cell_key(cell: &ArrayCellValue) -> CellKey {
    match cell {
        ArrayCellValue::Number(n) => CellKey::Number(n.to_bits()),
        ArrayCellValue::Text(t) => CellKey::Text(t.utf16_code_units().to_vec()),
        ArrayCellValue::Logical(b) => CellKey::Logical(*b),
        ArrayCellValue::Error(code) => CellKey::Error(*code as u8),
        ArrayCellValue::EmptyCell => CellKey::EmptyCell,
    }
}

pub(crate) fn default_row_field_headers(cols: usize) -> Vec<ArrayCellValue> {
    (1..=cols)
        .map(|i| {
            ArrayCellValue::Text(ExcelText::from_utf16_code_units(
                format!("Row Field {i}").encode_utf16().collect(),
            ))
        })
        .collect()
}

pub(crate) fn default_column_field_headers(cols: usize) -> Vec<ArrayCellValue> {
    (1..=cols)
        .map(|i| {
            ArrayCellValue::Text(ExcelText::from_utf16_code_units(
                format!("Column Field {i}").encode_utf16().collect(),
            ))
        })
        .collect()
}

pub(crate) fn default_value_headers(cols: usize) -> Vec<ArrayCellValue> {
    (1..=cols)
        .map(|i| {
            ArrayCellValue::Text(ExcelText::from_utf16_code_units(
                format!("Value {i}").encode_utf16().collect(),
            ))
        })
        .collect()
}

pub(crate) fn take_header_row(array: &EvalArray) -> Vec<ArrayCellValue> {
    row_as_cells(array, 0)
}

pub(crate) fn parse_filter_vector(
    prepared: Option<&PreparedArgValue>,
    expected_rows: usize,
) -> Result<Option<Vec<bool>>, LambdaHelperEvalError> {
    let Some(prepared) = prepared else {
        return Ok(None);
    };
    if matches!(
        prepared,
        PreparedArgValue::MissingArg | PreparedArgValue::EmptyCell
    ) {
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

fn coerce_cell_to_bool(cell: &ArrayCellValue) -> Result<bool, LambdaHelperEvalError> {
    match cell {
        ArrayCellValue::Logical(b) => Ok(*b),
        ArrayCellValue::Number(n) => Ok(*n != 0.0),
        ArrayCellValue::EmptyCell => Ok(false),
        ArrayCellValue::Error(code) => Err(LambdaHelperEvalError::Invocation(
            CallableInvocationError::Worksheet(*code),
        )),
        ArrayCellValue::Text(_) => Err(LambdaHelperEvalError::Invocation(
            CallableInvocationError::Worksheet(WorksheetErrorCode::Value),
        )),
    }
}

pub(crate) fn parse_sort_orders(
    prepared: Option<&PreparedArgValue>,
) -> Result<Vec<i32>, LambdaHelperEvalError> {
    let Some(prepared) = prepared else {
        return Ok(Vec::new());
    };
    if matches!(
        prepared,
        PreparedArgValue::MissingArg | PreparedArgValue::EmptyCell
    ) {
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
        .map(|cell| match cell {
            ArrayCellValue::Number(n) if n.is_finite() && n.fract() == 0.0 && *n != 0.0 => {
                Ok(*n as i32)
            }
            _ => Err(LambdaHelperEvalError::Invocation(
                CallableInvocationError::Worksheet(WorksheetErrorCode::Value),
            )),
        })
        .collect()
}

pub(crate) fn invoke_group_aggregate(
    callable: &LambdaValue,
    values: &[ArrayCellValue],
    invoker: &(impl CallableInvoker + ?Sized),
) -> Result<ArrayCellValue, LambdaHelperEvalError> {
    let column = if values.is_empty() {
        vec![vec![ArrayCellValue::EmptyCell]]
    } else {
        values
            .iter()
            .cloned()
            .map(|cell| vec![cell])
            .collect::<Vec<_>>()
    };
    let arg = PreparedArgValue::Eval(EvalValue::Array(
        EvalArray::from_rows(column).expect("column vector shape is valid"),
    ));
    let prepared = invoke_callable_prepared(callable, &[arg], invoker)
        .map_err(LambdaHelperEvalError::Invocation)?;
    scalar_cell_from_prepared(&prepared)
}

pub(crate) fn text_cell(text: &str) -> ArrayCellValue {
    ArrayCellValue::Text(ExcelText::from_utf16_code_units(
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

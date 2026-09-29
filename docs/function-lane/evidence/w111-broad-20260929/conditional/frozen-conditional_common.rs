//! Prepared-value selection shared by IF, IFERROR and IFNA.
//! W111 COM observations bind selection and spill shape, not evaluator scheduling.
use crate::coercion::CoercionError;
use crate::value::{ArrayShape, CalcArray, CalcValue, CoreValue, WorksheetErrorCode};

pub(super) fn shape(value: &CalcValue) -> ArrayShape {
    match value.core() {
        CoreValue::Array(array) => array.shape(),
        _ => ArrayShape { rows: 1, cols: 1 },
    }
}

pub(super) fn union_shape(values: &[&CalcValue]) -> ArrayShape {
    values
        .iter()
        .fold(ArrayShape { rows: 1, cols: 1 }, |out, value| {
            let current = shape(value);
            ArrayShape {
                rows: out.rows.max(current.rows),
                cols: out.cols.max(current.cols),
            }
        })
}

/// Singleton dimensions broadcast; absent coordinates are #N/A values. Selectors
/// inspect that value themselves: IFERROR/IFNA can replace a padded #N/A.
pub(super) fn at(value: &CalcValue, row: usize, col: usize) -> CalcValue {
    match value.core() {
        CoreValue::Array(array) => {
            let shape = array.shape();
            array
                .get(
                    if shape.rows == 1 { 0 } else { row },
                    if shape.cols == 1 { 0 } else { col },
                )
                .cloned()
                .unwrap_or_else(|| CalcValue::error(WorksheetErrorCode::NA))
        }
        _ => value.clone(),
    }
}

/// Explicit missing selected arguments and blank cells publish numeric zero.
/// IF's omitted false argument is supplied separately as logical FALSE.
pub(super) fn selected(value: CalcValue) -> CalcValue {
    match value.core() {
        CoreValue::Empty | CoreValue::Missing => CalcValue::number(0.0),
        CoreValue::Array(array) => CalcValue::array(
            CalcArray::new(
                array.shape(),
                array.iter_row_major().cloned().map(selected).collect(),
            )
            .expect("same shape"),
        ),
        _ => value,
    }
}

pub(super) fn fallback(
    primary: CalcValue,
    prepare_fallback: impl FnOnce() -> Result<CalcValue, CoercionError>,
    na_only: bool,
) -> Result<CalcValue, CoercionError> {
    let catches = |value: &CalcValue| match value.core() {
        CoreValue::Error(code) => !na_only || *code == WorksheetErrorCode::NA,
        _ => false,
    };
    if matches!(primary.core(), CoreValue::Array(_)) {
        let fallback = prepare_fallback()?;
        let shape = union_shape(&[&primary, &fallback]);
        let mut cells = Vec::with_capacity(shape.cell_count());
        for row in 0..shape.rows {
            for col in 0..shape.cols {
                let value = at(&primary, row, col);
                cells.push(selected(if catches(&value) {
                    at(&fallback, row, col)
                } else {
                    value
                }));
            }
        }
        Ok(CalcValue::array(
            CalcArray::new(shape, cells).expect("same shape"),
        ))
    } else if catches(&primary) {
        prepare_fallback().map(selected)
    } else {
        Ok(selected(primary))
    }
}

//! Observed prepared-value policy for LOG and POWER; generic adapters are separate.
use crate::coercion::CoercionError;
use crate::functions::adapters::coerce_prepared_to_number;
use crate::functions::binary_numeric::{BinaryNumericSurfaceError, map_binary_numeric_error_to_ws};
use crate::value::{ArrayShape, CalcArray, CalcValue, CoreValue, WorksheetErrorCode};

fn number(arg: &CalcValue) -> Result<f64, CoercionError> {
    if matches!(arg.core(), CoreValue::Missing) {
        Ok(0.0)
    } else {
        coerce_prepared_to_number(arg)
    }
}

/// Missing is zero. Padding is an NA value in its original argument position,
/// so scalar coercion proceeds left to right even on unequal array dimensions.
pub(crate) fn ordered_binary(
    args: &[CalcValue],
    kernel: impl Fn(f64, f64) -> Result<f64, WorksheetErrorCode>,
) -> Result<CalcValue, BinaryNumericSurfaceError> {
    if args.len() != 2 {
        return Err(BinaryNumericSurfaceError::ArityMismatch {
            expected: 2,
            actual: args.len(),
        });
    }
    let on_cell = |lhs: &CalcValue, rhs: &CalcValue| {
        let lhs = number(lhs).map_err(BinaryNumericSurfaceError::Coercion)?;
        let rhs = number(rhs).map_err(BinaryNumericSurfaceError::Coercion)?;
        kernel(lhs, rhs)
            .map(CalcValue::number)
            .map_err(BinaryNumericSurfaceError::Domain)
    };
    let shape = args
        .iter()
        .fold(ArrayShape { rows: 1, cols: 1 }, |shape, arg| {
            if let CoreValue::Array(array) = arg.core() {
                ArrayShape {
                    rows: shape.rows.max(array.shape().rows),
                    cols: shape.cols.max(array.shape().cols),
                }
            } else {
                shape
            }
        });
    if shape == (ArrayShape { rows: 1, cols: 1 }) {
        return on_cell(&args[0], &args[1]);
    }
    let mut cells = Vec::with_capacity(shape.cell_count());
    for row in 0..shape.rows {
        for col in 0..shape.cols {
            let at = |arg: &CalcValue| match arg.core() {
                CoreValue::Array(array) => array
                    .get(
                        if array.shape().rows == 1 { 0 } else { row },
                        if array.shape().cols == 1 { 0 } else { col },
                    )
                    .cloned()
                    .unwrap_or_else(|| CalcValue::error(WorksheetErrorCode::NA)),
                _ => arg.clone(),
            };
            cells.push(
                on_cell(&at(&args[0]), &at(&args[1])).unwrap_or_else(|error| {
                    CalcValue::error(map_binary_numeric_error_to_ws(&error))
                }),
            );
        }
    }
    Ok(CalcValue::array(
        CalcArray::new(shape, cells).expect("elementary broadcast shape preserved"),
    ))
}

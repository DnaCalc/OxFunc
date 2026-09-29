//! Prepared NORM/EXPON lifting observed through public Excel array formulas.
use crate::coercion::CoercionError;
use crate::functions::adapters::prepare_args_values_only;
use crate::resolver::ReferenceSystemProvider;
use crate::value::{ArrayShape, CalcArray, CalcValue, CoreValue, WorksheetErrorCode};

/// Preserve each argument position when a coordinate is absent. A padded #N/A
/// participates in the same left-to-right coercion as an explicit error; it
/// must not overwrite an earlier present error in a different argument.
pub(super) fn run_distribution_lifted<E>(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
    on_cell: impl Fn(&[CalcValue]) -> Result<CalcValue, E>,
    map_error: impl Fn(&E) -> WorksheetErrorCode,
    map_preparation_error: impl FnOnce(CoercionError) -> E,
) -> Result<CalcValue, E> {
    let prepared = prepare_args_values_only(args, resolver).map_err(map_preparation_error)?;
    let shape = prepared
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
        return on_cell(&prepared);
    }
    let mut cells = Vec::with_capacity(shape.cell_count());
    for row in 0..shape.rows {
        for col in 0..shape.cols {
            let values: Vec<_> = prepared
                .iter()
                .map(|arg| match arg.core() {
                    CoreValue::Array(array) => array
                        .get(
                            if array.shape().rows == 1 { 0 } else { row },
                            if array.shape().cols == 1 { 0 } else { col },
                        )
                        .cloned()
                        .unwrap_or_else(|| CalcValue::error(WorksheetErrorCode::NA)),
                    _ => arg.clone(),
                })
                .collect();
            cells
                .push(on_cell(&values).unwrap_or_else(|error| CalcValue::error(map_error(&error))));
        }
    }
    Ok(CalcValue::array(
        CalcArray::new(shape, cells).expect("distribution shape preserved"),
    ))
}

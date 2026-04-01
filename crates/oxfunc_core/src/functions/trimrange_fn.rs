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

pub const TRIMRANGE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.TRIMRANGE",
    arity: Arity { min: 1, max: 4 },
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::WorkbookState,
    thread_safety: ThreadSafetyClass::HostSerialized,
    arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
};

/// Trim direction: which edges to strip.
/// 0 = none, 1 = trailing only (default), 2 = leading only, 3 = both.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TrimType {
    None,
    Trailing,
    Leading,
    Both,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TrimRangeEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
    InvalidTrimType(f64),
    InvalidHeadersCount(f64),
    EmptyResult,
}

fn parse_trim_type(prepared: Option<&PreparedArgValue>) -> Result<TrimType, TrimRangeEvalError> {
    match prepared {
        None | Some(PreparedArgValue::MissingArg) | Some(PreparedArgValue::EmptyCell) => {
            Ok(TrimType::Trailing)
        }
        Some(arg) => {
            let raw = coerce_prepared_to_number(arg).map_err(TrimRangeEvalError::Coercion)?;
            if !raw.is_finite() {
                return Err(TrimRangeEvalError::InvalidTrimType(raw));
            }
            match raw.trunc() as i64 {
                0 => Ok(TrimType::None),
                1 => Ok(TrimType::Trailing),
                2 => Ok(TrimType::Leading),
                3 => Ok(TrimType::Both),
                _ => Err(TrimRangeEvalError::InvalidTrimType(raw)),
            }
        }
    }
}

fn parse_headers_count(prepared: Option<&PreparedArgValue>) -> Result<usize, TrimRangeEvalError> {
    match prepared {
        None | Some(PreparedArgValue::MissingArg) | Some(PreparedArgValue::EmptyCell) => Ok(0),
        Some(arg) => {
            let raw = coerce_prepared_to_number(arg).map_err(TrimRangeEvalError::Coercion)?;
            if !raw.is_finite() || raw < 0.0 {
                return Err(TrimRangeEvalError::InvalidHeadersCount(raw));
            }
            Ok(raw.trunc() as usize)
        }
    }
}

fn is_blank_cell(cell: &ArrayCellValue) -> bool {
    matches!(cell, ArrayCellValue::EmptyCell)
}

fn is_row_blank(array: &EvalArray, row: usize) -> bool {
    let cols = array.shape().cols;
    (0..cols).all(|col| array.get(row, col).map_or(true, is_blank_cell))
}

fn is_col_blank(array: &EvalArray, col: usize) -> bool {
    let rows = array.shape().rows;
    (0..rows).all(|row| array.get(row, col).map_or(true, is_blank_cell))
}

fn materialize_input(prepared: &PreparedArgValue) -> EvalArray {
    match prepared {
        PreparedArgValue::Eval(EvalValue::Array(arr)) => arr.clone(),
        PreparedArgValue::Eval(v) => {
            let cell = match v {
                EvalValue::Number(n) => ArrayCellValue::Number(*n),
                EvalValue::Text(t) => ArrayCellValue::Text(t.clone()),
                EvalValue::Logical(b) => ArrayCellValue::Logical(*b),
                EvalValue::Error(code) => ArrayCellValue::Error(*code),
                _ => ArrayCellValue::EmptyCell,
            };
            EvalArray::from_scalar(cell)
        }
        PreparedArgValue::EmptyCell | PreparedArgValue::MissingArg => {
            EvalArray::from_scalar(ArrayCellValue::EmptyCell)
        }
    }
}

pub(crate) fn trimrange_kernel(
    array: &EvalArray,
    trim_rows: TrimType,
    trim_cols: TrimType,
    headers_count: usize,
) -> Result<EvalValue, TrimRangeEvalError> {
    let total_rows = array.shape().rows;
    let total_cols = array.shape().cols;

    // Determine the row range to keep.
    let data_start = headers_count.min(total_rows);
    let mut first_data_row = data_start;
    let mut last_data_row = if total_rows > 0 { total_rows - 1 } else { 0 };

    if total_rows > data_start {
        match trim_rows {
            TrimType::None => {}
            TrimType::Leading => {
                while first_data_row <= last_data_row && is_row_blank(array, first_data_row) {
                    first_data_row += 1;
                }
            }
            TrimType::Trailing => {
                while last_data_row > first_data_row && is_row_blank(array, last_data_row) {
                    if last_data_row == 0 {
                        break;
                    }
                    last_data_row -= 1;
                }
                // Check if the last remaining data row is also blank.
                if last_data_row == first_data_row
                    && is_row_blank(array, last_data_row)
                    && data_start == 0
                {
                    // All rows blank — will produce empty result.
                }
            }
            TrimType::Both => {
                while first_data_row <= last_data_row && is_row_blank(array, first_data_row) {
                    first_data_row += 1;
                }
                while last_data_row > first_data_row && is_row_blank(array, last_data_row) {
                    last_data_row -= 1;
                }
            }
        }
    }

    // Determine the column range to keep.
    let mut first_col = 0usize;
    let mut last_col = if total_cols > 0 { total_cols - 1 } else { 0 };

    match trim_cols {
        TrimType::None => {}
        TrimType::Leading => {
            while first_col <= last_col && is_col_blank(array, first_col) {
                first_col += 1;
            }
        }
        TrimType::Trailing => {
            while last_col > first_col && is_col_blank(array, last_col) {
                last_col -= 1;
            }
        }
        TrimType::Both => {
            while first_col <= last_col && is_col_blank(array, first_col) {
                first_col += 1;
            }
            while last_col > first_col && is_col_blank(array, last_col) {
                last_col -= 1;
            }
        }
    }

    // Build the header rows (always preserved) + trimmed data rows.
    let header_end = data_start; // exclusive
    let kept_rows: Vec<usize> = (0..header_end)
        .chain(first_data_row..=last_data_row)
        .collect();
    let kept_cols: Vec<usize> = (first_col..=last_col).collect();

    // Check for degenerate cases.
    if kept_rows.is_empty() || kept_cols.is_empty() {
        return Err(TrimRangeEvalError::EmptyResult);
    }

    // If first_data_row > last_data_row and there are no headers, result is empty.
    if first_data_row > last_data_row && header_end == 0 {
        return Err(TrimRangeEvalError::EmptyResult);
    }

    let out_rows = kept_rows.len();
    let out_cols = kept_cols.len();

    let mut cells = Vec::with_capacity(out_rows * out_cols);
    for &r in &kept_rows {
        for &c in &kept_cols {
            cells.push(
                array
                    .get(r, c)
                    .cloned()
                    .unwrap_or(ArrayCellValue::EmptyCell),
            );
        }
    }

    if out_rows == 1 && out_cols == 1 {
        return Ok(match &cells[0] {
            ArrayCellValue::Number(n) => EvalValue::Number(*n),
            ArrayCellValue::Text(t) => EvalValue::Text(t.clone()),
            ArrayCellValue::Logical(b) => EvalValue::Logical(*b),
            ArrayCellValue::Error(code) => EvalValue::Error(*code),
            ArrayCellValue::EmptyCell => {
                EvalValue::Text(crate::value::ExcelText::from_utf16_code_units(Vec::new()))
            }
        });
    }

    Ok(EvalValue::Array(
        EvalArray::new(
            ArrayShape {
                rows: out_rows,
                cols: out_cols,
            },
            cells,
        )
        .expect("trimrange shape valid"),
    ))
}

pub fn eval_trimrange_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, TrimRangeEvalError> {
    if !TRIMRANGE_META.arity.accepts(args.len()) {
        return Err(TrimRangeEvalError::ArityMismatch {
            expected_min: TRIMRANGE_META.arity.min,
            expected_max: TRIMRANGE_META.arity.max,
            actual: args.len(),
        });
    }

    run_values_only_prepared(
        args,
        resolver,
        |prepared| {
            let array = materialize_input(&prepared[0]);
            let trim_rows = parse_trim_type(prepared.get(1))?;
            let trim_cols = parse_trim_type(prepared.get(2))?;
            let headers_count = parse_headers_count(prepared.get(3))?;
            trimrange_kernel(&array, trim_rows, trim_cols, headers_count)
        },
        TrimRangeEvalError::Coercion,
    )
}

pub fn map_trimrange_error_to_ws(e: &TrimRangeEvalError) -> WorksheetErrorCode {
    match e {
        TrimRangeEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        TrimRangeEvalError::Coercion(_) => WorksheetErrorCode::Value,
        TrimRangeEvalError::InvalidTrimType(_) => WorksheetErrorCode::Value,
        TrimRangeEvalError::InvalidHeadersCount(_) => WorksheetErrorCode::Value,
        TrimRangeEvalError::EmptyResult => WorksheetErrorCode::Calc,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::{CallerContext, RefResolutionError, ResolverCapabilities};
    use crate::value::ReferenceLike;

    struct MockResolver;
    impl ReferenceResolver for MockResolver {
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
        fn caller_context(&self) -> Option<CallerContext> {
            None
        }
    }

    fn make_array(rows: usize, cols: usize, cells: Vec<ArrayCellValue>) -> EvalArray {
        EvalArray::new(ArrayShape { rows, cols }, cells).unwrap()
    }

    fn n(v: f64) -> ArrayCellValue {
        ArrayCellValue::Number(v)
    }

    fn e() -> ArrayCellValue {
        ArrayCellValue::EmptyCell
    }

    // --- Meta tests ---

    #[test]
    fn trimrange_meta_arity() {
        assert_eq!(TRIMRANGE_META.arity.min, 1);
        assert_eq!(TRIMRANGE_META.arity.max, 4);
    }

    #[test]
    fn trimrange_meta_deterministic() {
        assert_eq!(TRIMRANGE_META.determinism, DeterminismClass::Deterministic);
    }

    // --- Arity tests ---

    #[test]
    fn trimrange_rejects_zero_args() {
        let got = eval_trimrange_surface(&[], &MockResolver);
        assert!(matches!(got, Err(TrimRangeEvalError::ArityMismatch { .. })));
    }

    // --- Trailing trim (default) ---

    #[test]
    fn trimrange_trims_trailing_blank_rows_and_cols() {
        // 3x3 grid: data in top-left 2x2, blank right col and bottom row
        let arr = make_array(
            3,
            3,
            vec![n(1.0), n(2.0), e(), n(3.0), n(4.0), e(), e(), e(), e()],
        );
        let got = trimrange_kernel(&arr, TrimType::Trailing, TrimType::Trailing, 0).unwrap();
        let expected = EvalValue::Array(make_array(2, 2, vec![n(1.0), n(2.0), n(3.0), n(4.0)]));
        assert_eq!(got, expected);
    }

    // --- Leading trim ---

    #[test]
    fn trimrange_trims_leading_blank_rows() {
        let arr = make_array(3, 2, vec![e(), e(), n(1.0), n(2.0), n(3.0), n(4.0)]);
        let got = trimrange_kernel(&arr, TrimType::Leading, TrimType::None, 0).unwrap();
        let expected = EvalValue::Array(make_array(2, 2, vec![n(1.0), n(2.0), n(3.0), n(4.0)]));
        assert_eq!(got, expected);
    }

    // --- Both trim ---

    #[test]
    fn trimrange_trims_both_edges() {
        let arr = make_array(
            4,
            4,
            vec![
                e(),
                e(),
                e(),
                e(),
                e(),
                n(1.0),
                n(2.0),
                e(),
                e(),
                n(3.0),
                n(4.0),
                e(),
                e(),
                e(),
                e(),
                e(),
            ],
        );
        let got = trimrange_kernel(&arr, TrimType::Both, TrimType::Both, 0).unwrap();
        let expected = EvalValue::Array(make_array(2, 2, vec![n(1.0), n(2.0), n(3.0), n(4.0)]));
        assert_eq!(got, expected);
    }

    // --- No trim ---

    #[test]
    fn trimrange_no_trim_preserves_all() {
        let arr = make_array(2, 2, vec![n(1.0), e(), e(), n(2.0)]);
        let got = trimrange_kernel(&arr, TrimType::None, TrimType::None, 0).unwrap();
        let expected = EvalValue::Array(make_array(2, 2, vec![n(1.0), e(), e(), n(2.0)]));
        assert_eq!(got, expected);
    }

    // --- Headers preserved ---

    #[test]
    fn trimrange_preserves_header_rows() {
        // Header row is blank but should not be trimmed with leading trim.
        let arr = make_array(
            3,
            2,
            vec![
                e(),
                e(), // header row (blank)
                n(1.0),
                n(2.0),
                e(),
                e(), // trailing blank
            ],
        );
        let got = trimrange_kernel(&arr, TrimType::Both, TrimType::None, 1).unwrap();
        // Header row kept + data row kept, trailing blank trimmed.
        let expected = EvalValue::Array(make_array(2, 2, vec![e(), e(), n(1.0), n(2.0)]));
        assert_eq!(got, expected);
    }

    // --- All-blank returns #CALC! ---

    #[test]
    fn trimrange_all_blank_returns_calc_error() {
        let arr = make_array(2, 2, vec![e(), e(), e(), e()]);
        let got = trimrange_kernel(&arr, TrimType::Both, TrimType::Both, 0);
        assert_eq!(got, Err(TrimRangeEvalError::EmptyResult));
    }

    // --- Already trimmed passthrough ---

    #[test]
    fn trimrange_already_trimmed_returns_same() {
        let arr = make_array(2, 2, vec![n(1.0), n(2.0), n(3.0), n(4.0)]);
        let got = trimrange_kernel(&arr, TrimType::Both, TrimType::Both, 0).unwrap();
        let expected = EvalValue::Array(make_array(2, 2, vec![n(1.0), n(2.0), n(3.0), n(4.0)]));
        assert_eq!(got, expected);
    }

    // --- Single cell result ---

    #[test]
    fn trimrange_single_cell_result_returns_scalar() {
        let arr = make_array(2, 2, vec![n(42.0), e(), e(), e()]);
        let got = trimrange_kernel(&arr, TrimType::Trailing, TrimType::Trailing, 0).unwrap();
        assert_eq!(got, EvalValue::Number(42.0));
    }

    // --- Invalid trim type ---

    #[test]
    fn trimrange_invalid_trim_type() {
        let got = parse_trim_type(Some(&PreparedArgValue::Eval(EvalValue::Number(5.0))));
        assert!(matches!(got, Err(TrimRangeEvalError::InvalidTrimType(_))));
    }

    // --- Error mapping ---

    #[test]
    fn trimrange_error_mapping() {
        assert_eq!(
            map_trimrange_error_to_ws(&TrimRangeEvalError::EmptyResult),
            WorksheetErrorCode::Calc
        );
        assert_eq!(
            map_trimrange_error_to_ws(&TrimRangeEvalError::InvalidTrimType(5.0)),
            WorksheetErrorCode::Value
        );
    }
}

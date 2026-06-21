use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::a1_refs::parse_a1_reference;
use crate::resolver::{
    ReferenceResolutionError, ReferenceSystemProvider, enumerate_reference_values,
};
use crate::value::{CalcValue, CoreValue, WorksheetErrorCode};

pub const ROWS_META: FunctionMeta = function_spec! {
    function_id: "FUNC.ROWS",
    arity: Arity { min: 1, max: 1 },
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::WorkbookState,
    thread_safety: ThreadSafetyClass::HostSerialized,
    arg_preparation_profile: ArgPreparationProfile::RefsVisibleInAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::RefOnly,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
};

#[derive(Debug, Clone, PartialEq)]
pub enum RowsEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    InvalidReferenceArg,
    RefResolution(ReferenceResolutionError),
}

pub fn eval_rows_surface(args: &[CalcValue]) -> Result<CalcValue, RowsEvalError> {
    if !ROWS_META.arity.accepts(args.len()) {
        return Err(RowsEvalError::ArityMismatch {
            expected_min: ROWS_META.arity.min,
            expected_max: ROWS_META.arity.max,
            actual: args.len(),
        });
    }

    let arg = &args[0];
    if let CoreValue::Array(arr) = arg.core() {
        return Ok(CalcValue::number(arr.shape().rows as f64));
    }

    let reference = arg
        .as_reference()
        .ok_or(RowsEvalError::InvalidReferenceArg)?;
    let parsed =
        parse_a1_reference(reference.target()).ok_or(RowsEvalError::InvalidReferenceArg)?;
    let count = parsed.end_row - parsed.start_row + 1;
    Ok(CalcValue::number(count as f64))
}

pub fn eval_rows_surface_with_resolver(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, RowsEvalError> {
    if !ROWS_META.arity.accepts(args.len()) {
        return Err(RowsEvalError::ArityMismatch {
            expected_min: ROWS_META.arity.min,
            expected_max: ROWS_META.arity.max,
            actual: args.len(),
        });
    }

    let arg = &args[0];

    // Array argument: return row count directly.
    if let CoreValue::Array(arr) = arg.core() {
        return Ok(CalcValue::number(arr.shape().rows as f64));
    }

    // Reference argument: parse and compute row span.
    let reference = arg
        .as_reference()
        .ok_or(RowsEvalError::InvalidReferenceArg)?;
    let count = if let Some(parsed) = parse_a1_reference(reference.target()) {
        parsed.end_row - parsed.start_row + 1
    } else {
        enumerate_reference_values(resolver, reference)
            .map_err(RowsEvalError::RefResolution)?
            .map(|values| values.declared_extent.rows)
            .ok_or(RowsEvalError::InvalidReferenceArg)?
    };
    Ok(CalcValue::number(count as f64))
}

pub fn map_rows_error_to_ws(e: &RowsEvalError) -> WorksheetErrorCode {
    match e {
        RowsEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        RowsEvalError::InvalidReferenceArg => WorksheetErrorCode::Value,
        RowsEvalError::RefResolution(_) => WorksheetErrorCode::Ref,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::function::{
        ArgPreparationProfile, DeterminismClass, FecDependencyProfile, VolatilityClass,
    };
    use crate::value::{ArrayShape, CalcArray, ReferenceKind, ReferenceLike};

    fn ref_arg(target: &str) -> CalcValue {
        CalcValue::reference(ReferenceLike::new(ReferenceKind::Area, target.to_string()))
    }

    // --- Meta property tests ---

    #[test]
    fn rows_meta_arity_is_exact_one() {
        assert_eq!(ROWS_META.arity.min, 1);
        assert_eq!(ROWS_META.arity.max, 1);
    }

    #[test]
    fn rows_meta_determinism() {
        assert_eq!(ROWS_META.determinism, DeterminismClass::Deterministic);
    }

    #[test]
    fn rows_meta_volatility() {
        assert_eq!(ROWS_META.volatility, VolatilityClass::NonVolatile);
    }

    #[test]
    fn rows_meta_refs_visible() {
        assert_eq!(
            ROWS_META.arg_preparation_profile,
            ArgPreparationProfile::RefsVisibleInAdapter
        );
    }

    #[test]
    fn rows_meta_fec_ref_only() {
        assert_eq!(
            ROWS_META.fec_dependency_profile,
            FecDependencyProfile::RefOnly
        );
        assert_eq!(
            ROWS_META.surface_fec_dependency_profile,
            FecDependencyProfile::RefOnly
        );
    }

    // --- Arity tests ---

    #[test]
    fn rows_rejects_zero_args() {
        let got = eval_rows_surface(&[]);
        assert!(matches!(got, Err(RowsEvalError::ArityMismatch { .. })));
    }

    #[test]
    fn rows_rejects_two_args() {
        let got = eval_rows_surface(&[ref_arg("A1"), ref_arg("B1")]);
        assert!(matches!(got, Err(RowsEvalError::ArityMismatch { .. })));
    }

    // --- Reference tests ---

    #[test]
    fn rows_single_cell_returns_one() {
        assert_eq!(
            eval_rows_surface(&[ref_arg("B2")]),
            Ok(CalcValue::number(1.0))
        );
    }

    #[test]
    fn rows_area_reference_returns_row_count() {
        assert_eq!(
            eval_rows_surface(&[ref_arg("A1:C5")]),
            Ok(CalcValue::number(5.0))
        );
    }

    #[test]
    fn rows_single_row_area_returns_one() {
        assert_eq!(
            eval_rows_surface(&[ref_arg("B2:D2")]),
            Ok(CalcValue::number(1.0))
        );
    }

    #[test]
    fn rows_whole_column_returns_max_rows() {
        assert_eq!(
            eval_rows_surface(&[ref_arg("A:A")]),
            Ok(CalcValue::number(1_048_576.0))
        );
    }

    #[test]
    fn rows_whole_row_returns_one() {
        assert_eq!(
            eval_rows_surface(&[ref_arg("1:1")]),
            Ok(CalcValue::number(1.0))
        );
    }

    #[test]
    fn rows_multi_whole_row_returns_count() {
        assert_eq!(
            eval_rows_surface(&[ref_arg("2:5")]),
            Ok(CalcValue::number(4.0))
        );
    }

    #[test]
    fn rows_cross_sheet_reference() {
        assert_eq!(
            eval_rows_surface(&[ref_arg("Sheet1!A1:A10")]),
            Ok(CalcValue::number(10.0))
        );
    }

    // --- Array argument tests ---

    #[test]
    fn rows_array_arg_returns_row_count() {
        let arr = CalcArray::new(
            ArrayShape { rows: 3, cols: 2 },
            vec![
                CalcValue::number(1.0),
                CalcValue::number(2.0),
                CalcValue::number(3.0),
                CalcValue::number(4.0),
                CalcValue::number(5.0),
                CalcValue::number(6.0),
            ],
        )
        .unwrap();
        let got = eval_rows_surface(&[(CalcValue::array(arr))]);
        assert_eq!(got, Ok(CalcValue::number(3.0)));
    }

    #[test]
    fn rows_single_cell_array_returns_one() {
        let arr = CalcArray::new(
            ArrayShape { rows: 1, cols: 1 },
            vec![CalcValue::number(42.0)],
        )
        .unwrap();
        let got = eval_rows_surface(&[(CalcValue::array(arr))]);
        assert_eq!(got, Ok(CalcValue::number(1.0)));
    }

    // --- Error tests ---

    #[test]
    fn rows_non_reference_non_array_returns_error() {
        let got = eval_rows_surface(&[(CalcValue::number(42.0))]);
        assert_eq!(got, Err(RowsEvalError::InvalidReferenceArg));
    }

    #[test]
    fn rows_text_arg_returns_error() {
        let got = eval_rows_surface(&[(CalcValue::text(
            crate::value::ExcelText::from_interop_assignment("hello"),
        ))]);
        assert_eq!(got, Err(RowsEvalError::InvalidReferenceArg));
    }

    #[test]
    fn rows_error_maps_to_value() {
        assert_eq!(
            map_rows_error_to_ws(&RowsEvalError::InvalidReferenceArg),
            WorksheetErrorCode::Value
        );
        assert_eq!(
            map_rows_error_to_ws(&RowsEvalError::ArityMismatch {
                expected_min: 1,
                expected_max: 1,
                actual: 0,
            }),
            WorksheetErrorCode::Value
        );
    }
}

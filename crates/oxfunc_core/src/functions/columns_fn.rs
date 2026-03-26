use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::a1_refs::parse_a1_reference;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

pub const COLUMNS_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.COLUMNS",
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
pub enum ColumnsEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    InvalidReferenceArg,
}

pub fn eval_columns_surface(args: &[CallArgValue]) -> Result<EvalValue, ColumnsEvalError> {
    if !COLUMNS_META.arity.accepts(args.len()) {
        return Err(ColumnsEvalError::ArityMismatch {
            expected_min: COLUMNS_META.arity.min,
            expected_max: COLUMNS_META.arity.max,
            actual: args.len(),
        });
    }

    let arg = &args[0];

    // Array argument: return column count directly.
    if let CallArgValue::Eval(EvalValue::Array(arr)) = arg {
        return Ok(EvalValue::Number(arr.shape().cols as f64));
    }

    // Reference argument: parse and compute column span.
    let reference = match arg {
        CallArgValue::Reference(r) | CallArgValue::Eval(EvalValue::Reference(r)) => r,
        _ => return Err(ColumnsEvalError::InvalidReferenceArg),
    };
    let parsed =
        parse_a1_reference(&reference.target).ok_or(ColumnsEvalError::InvalidReferenceArg)?;
    let count = parsed.end_col - parsed.start_col + 1;
    Ok(EvalValue::Number(count as f64))
}

pub fn map_columns_error_to_ws(e: &ColumnsEvalError) -> WorksheetErrorCode {
    match e {
        ColumnsEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        ColumnsEvalError::InvalidReferenceArg => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::function::{
        ArgPreparationProfile, DeterminismClass, FecDependencyProfile, VolatilityClass,
    };
    use crate::value::{ArrayCellValue, ArrayShape, EvalArray, ReferenceKind, ReferenceLike};

    fn ref_arg(target: &str) -> CallArgValue {
        CallArgValue::Reference(ReferenceLike {
            kind: ReferenceKind::Area,
            target: target.to_string(),
        })
    }

    // --- Meta property tests ---

    #[test]
    fn columns_meta_arity_is_exact_one() {
        assert_eq!(COLUMNS_META.arity.min, 1);
        assert_eq!(COLUMNS_META.arity.max, 1);
    }

    #[test]
    fn columns_meta_determinism() {
        assert_eq!(COLUMNS_META.determinism, DeterminismClass::Deterministic);
    }

    #[test]
    fn columns_meta_volatility() {
        assert_eq!(COLUMNS_META.volatility, VolatilityClass::NonVolatile);
    }

    #[test]
    fn columns_meta_refs_visible() {
        assert_eq!(
            COLUMNS_META.arg_preparation_profile,
            ArgPreparationProfile::RefsVisibleInAdapter
        );
    }

    #[test]
    fn columns_meta_fec_ref_only() {
        assert_eq!(
            COLUMNS_META.fec_dependency_profile,
            FecDependencyProfile::RefOnly
        );
        assert_eq!(
            COLUMNS_META.surface_fec_dependency_profile,
            FecDependencyProfile::RefOnly
        );
    }

    // --- Arity tests ---

    #[test]
    fn columns_rejects_zero_args() {
        let got = eval_columns_surface(&[]);
        assert!(matches!(got, Err(ColumnsEvalError::ArityMismatch { .. })));
    }

    #[test]
    fn columns_rejects_two_args() {
        let got = eval_columns_surface(&[ref_arg("A1"), ref_arg("B1")]);
        assert!(matches!(got, Err(ColumnsEvalError::ArityMismatch { .. })));
    }

    // --- Reference tests ---

    #[test]
    fn columns_single_cell_returns_one() {
        assert_eq!(
            eval_columns_surface(&[ref_arg("B2")]),
            Ok(EvalValue::Number(1.0))
        );
    }

    #[test]
    fn columns_area_reference_returns_col_count() {
        assert_eq!(
            eval_columns_surface(&[ref_arg("A1:C5")]),
            Ok(EvalValue::Number(3.0))
        );
    }

    #[test]
    fn columns_single_column_area_returns_one() {
        assert_eq!(
            eval_columns_surface(&[ref_arg("B2:B5")]),
            Ok(EvalValue::Number(1.0))
        );
    }

    #[test]
    fn columns_whole_row_returns_max_cols() {
        assert_eq!(
            eval_columns_surface(&[ref_arg("1:1")]),
            Ok(EvalValue::Number(16_384.0))
        );
    }

    #[test]
    fn columns_whole_column_returns_one() {
        assert_eq!(
            eval_columns_surface(&[ref_arg("A:A")]),
            Ok(EvalValue::Number(1.0))
        );
    }

    #[test]
    fn columns_multi_whole_column_returns_count() {
        assert_eq!(
            eval_columns_surface(&[ref_arg("B:D")]),
            Ok(EvalValue::Number(3.0))
        );
    }

    #[test]
    fn columns_cross_sheet_reference() {
        assert_eq!(
            eval_columns_surface(&[ref_arg("Sheet1!A1:E1")]),
            Ok(EvalValue::Number(5.0))
        );
    }

    // --- Array argument tests ---

    #[test]
    fn columns_array_arg_returns_col_count() {
        let arr = EvalArray::new(
            ArrayShape { rows: 3, cols: 2 },
            vec![
                ArrayCellValue::Number(1.0),
                ArrayCellValue::Number(2.0),
                ArrayCellValue::Number(3.0),
                ArrayCellValue::Number(4.0),
                ArrayCellValue::Number(5.0),
                ArrayCellValue::Number(6.0),
            ],
        )
        .unwrap();
        let got = eval_columns_surface(&[CallArgValue::Eval(EvalValue::Array(arr))]);
        assert_eq!(got, Ok(EvalValue::Number(2.0)));
    }

    #[test]
    fn columns_single_cell_array_returns_one() {
        let arr = EvalArray::new(
            ArrayShape { rows: 1, cols: 1 },
            vec![ArrayCellValue::Number(42.0)],
        )
        .unwrap();
        let got = eval_columns_surface(&[CallArgValue::Eval(EvalValue::Array(arr))]);
        assert_eq!(got, Ok(EvalValue::Number(1.0)));
    }

    // --- Error tests ---

    #[test]
    fn columns_non_reference_non_array_returns_error() {
        let got = eval_columns_surface(&[CallArgValue::Eval(EvalValue::Number(42.0))]);
        assert_eq!(got, Err(ColumnsEvalError::InvalidReferenceArg));
    }

    #[test]
    fn columns_error_maps_to_value() {
        assert_eq!(
            map_columns_error_to_ws(&ColumnsEvalError::InvalidReferenceArg),
            WorksheetErrorCode::Value
        );
    }
}

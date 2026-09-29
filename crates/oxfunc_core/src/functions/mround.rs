use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile, FunctionMeta,
    HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::binary_numeric::{
    BinaryNumericSurfaceError, map_binary_numeric_error_to_ws,
};
use crate::functions::adapters::prepare_args_values_only;
use crate::resolver::ReferenceSystemProvider;
use crate::value::{CalcArray, CalcValue, CoreValue};
use crate::value::WorksheetErrorCode;

pub const MROUND_META: FunctionMeta = function_spec! {
    function_id: "FUNC.MROUND",
    arity: Arity::exact(2),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::RefsVisibleInAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::NumsToNum,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
};

pub fn mround_kernel(number: f64, multiple: f64) -> Result<f64, WorksheetErrorCode> {
    if multiple == 0.0 {
        return Ok(0.0);
    }
    // Zero has no sign; Excel returns 0 for MROUND(0, any_nonzero_multiple).
    if number == 0.0 {
        return Ok(0.0);
    }
    if number.signum() != multiple.signum() {
        return Err(WorksheetErrorCode::Num);
    }
    // W111 public Excel controls bracket the fractional transition at these
    // adjacent binary64 values: ...ffa5 rounds down, ...ffa6 rounds up.
    // This is an observed uniform cutoff, not a claim about Excel's internals.
    // Each division/product is rounded to 64 significand bits before its
    // binary64 publication; ordinary binary64-only operations differ here.
    let quotient = crate::excel_numeric::excel_x87_div(number.abs(), multiple.abs());
    if !quotient.is_finite() {
        return Err(WorksheetErrorCode::Num);
    }
    let lower = quotient.floor();
    let count = lower
        + f64::from(quotient - lower >= f64::from_bits(0x3fdf_ffff_ffff_ffa6));
    let result = crate::excel_numeric::excel_x87_mul(count, multiple);
    if !result.is_finite() {
        Err(WorksheetErrorCode::Num)
    } else {
        Ok(if result == 0.0 { 0.0 } else { result })
    }
}

pub fn eval_mround_surface(
    args: &[crate::value::CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, BinaryNumericSurfaceError> {
    // W111: logicals are VALUE, explicit omitted slots are NA, and blanks
    // remain zero. Convert these kinds before ordered scalar coercion so
    // an earlier error still wins over a later missing array coordinate.
    fn prepare_kind(value: &CalcValue) -> CalcValue {
        match value.core() {
            CoreValue::Logical(_) => CalcValue::error(WorksheetErrorCode::Value),
            CoreValue::Missing => CalcValue::error(WorksheetErrorCode::NA),
            CoreValue::Array(array) => CalcValue::array(
                CalcArray::new(array.shape(), array.iter_row_major().map(prepare_kind).collect())
                    .expect("MROUND preparation preserves shape"),
            ),
            _ => value.clone(),
        }
    }
    let prepared = prepare_args_values_only(args, resolver)
        .map_err(BinaryNumericSurfaceError::Coercion)?;
    // W111 distinguishes a multi-cell reference from its materialized array,
    // even when the caller is aligned with a row/column of that reference.
    // Keep the rejection at this argument position for ordered error selection.
    let prepared = prepared.iter().zip(args).map(|(value, original)| {
        if matches!(original.core(), CoreValue::Reference(_))
            && matches!(value.core(), CoreValue::Array(array) if array.shape().cell_count() > 1)
        {
            CalcValue::error(WorksheetErrorCode::Value)
        } else {
            prepare_kind(value)
        }
    }).collect::<Vec<_>>();
    crate::functions::elementary_prepared::ordered_binary(&prepared, mround_kernel)
}

pub fn map_mround_error_to_ws(e: &BinaryNumericSurfaceError) -> WorksheetErrorCode {
    map_binary_numeric_error_to_ws(e)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mround_meta_function_id_is_stable() {
        assert_eq!(MROUND_META.function_id, "FUNC.MROUND");
    }

    #[test]
    fn mround_kernel_matches_excel_sign_and_midpoint_lanes() {
        assert_eq!(mround_kernel(10.0, 3.0), Ok(9.0));
        assert_eq!(mround_kernel(-10.0, -3.0), Ok(-9.0));
        assert_eq!(mround_kernel(10.0, -3.0), Err(WorksheetErrorCode::Num));
        assert_eq!(mround_kernel(-10.0, 3.0), Err(WorksheetErrorCode::Num));
        assert_eq!(mround_kernel(1.25, 0.5), Ok(1.5));
        assert_eq!(mround_kernel(5.0, 0.0), Ok(0.0));
    }

    // BUG-FUNC-039 item 5: MROUND(0, negative) -> 0, not #NUM!.
    #[test]
    fn mround_zero_number_with_negative_multiple_returns_zero() {
        assert_eq!(mround_kernel(0.0, -3.0), Ok(0.0));
        assert_eq!(mround_kernel(0.0, -1.0), Ok(0.0));
        assert_eq!(mround_kernel(0.0, 3.0), Ok(0.0));
        // Sign-mismatch for non-zero number still errors.
        assert_eq!(mround_kernel(1.0, -3.0), Err(WorksheetErrorCode::Num));
        assert_eq!(mround_kernel(-1.0, 3.0), Err(WorksheetErrorCode::Num));
    }
}

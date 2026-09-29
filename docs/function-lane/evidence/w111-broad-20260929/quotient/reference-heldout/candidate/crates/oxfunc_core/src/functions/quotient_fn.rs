use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::prepare_args_values_only;
use crate::functions::binary_numeric::{
    BinaryNumericSurfaceError, eval_binary_numeric_prepared, map_binary_numeric_error_to_ws,
};
use crate::resolver::ReferenceSystemProvider;
use crate::value::WorksheetErrorCode;
use crate::value::{CalcArray, CalcValue, CoreValue};

pub const QUOTIENT_META: FunctionMeta = function_spec! {
    function_id: "FUNC.QUOTIENT",
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

pub fn quotient_kernel(numerator: f64, denominator: f64) -> Result<f64, WorksheetErrorCode> {
    if denominator == 0.0 {
        return Err(WorksheetErrorCode::Div0);
    }
    let quotient = numerator / denominator;
    if !quotient.is_finite() {
        return Err(WorksheetErrorCode::Num);
    }
    let integer = quotient.trunc();
    // Excel publishes integer zero without an IEEE sign, including negative
    // fractions and finite division underflow. W111 retains exact-bit probes.
    Ok(if integer == 0.0 { 0.0 } else { integer })
}

pub fn eval_quotient_surface(
    args: &[crate::value::CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, BinaryNumericSurfaceError> {
    // QUOTIENT rejects logicals even through a reference or lifted array cell;
    // an explicitly omitted argument is #N/A. Prepared blanks still coerce to
    // zero. Convert only these family-specific kinds before reusing the common
    // broadcast and left-to-right coercion machinery.
    fn prepare_kind(value: &CalcValue) -> CalcValue {
        match value.core() {
            CoreValue::Logical(_) => CalcValue::error(WorksheetErrorCode::Value),
            CoreValue::Missing => CalcValue::error(WorksheetErrorCode::NA),
            CoreValue::Array(array) => CalcValue::array(
                CalcArray::new(
                    array.shape(),
                    array.iter_row_major().map(prepare_kind).collect(),
                )
                .expect("shape preserved"),
            ),
            _ => value.clone(),
        }
    }
    let prepared =
        prepare_args_values_only(args, resolver).map_err(BinaryNumericSurfaceError::Coercion)?;
    // A multi-cell reference is rejected even when its caller is aligned.
    // An explicitly materialized array remains liftable. Preserve the original
    // kind until this point and leave the error at its own argument position.
    let prepared = prepared
        .iter()
        .zip(args)
        .map(|(value, original)| {
            if matches!(original.core(), CoreValue::Reference(_))
                && matches!(value.core(), CoreValue::Array(array) if array.shape().cell_count() > 1)
            {
                CalcValue::error(WorksheetErrorCode::Value)
            } else {
                prepare_kind(value)
            }
        })
        .collect::<Vec<_>>();
    eval_binary_numeric_prepared(&prepared, quotient_kernel)
}

pub fn map_quotient_error_to_ws(e: &BinaryNumericSurfaceError) -> WorksheetErrorCode {
    map_binary_numeric_error_to_ws(e)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quotient_meta_function_id_is_stable() {
        assert_eq!(QUOTIENT_META.function_id, "FUNC.QUOTIENT");
    }

    #[test]
    fn quotient_kernel_truncates_toward_zero() {
        assert_eq!(quotient_kernel(7.0, 3.0), Ok(2.0));
        assert_eq!(quotient_kernel(-7.0, 3.0), Ok(-2.0));
        assert_eq!(quotient_kernel(7.0, -3.0), Ok(-2.0));
        assert_eq!(quotient_kernel(-7.0, -3.0), Ok(2.0));
        assert_eq!(quotient_kernel(1.0, 0.0), Err(WorksheetErrorCode::Div0));
    }
}

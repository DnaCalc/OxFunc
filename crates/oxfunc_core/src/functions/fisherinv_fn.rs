use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::unary_numeric::{
    UnaryNumericSurfaceError, eval_unary_numeric_surface, map_unary_numeric_error_to_ws,
};
use crate::resolver::ReferenceSystemProvider;
use crate::value::CalcValue;
use crate::value::WorksheetErrorCode;

pub const FISHERINV_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.FISHERINV",
    arity: Arity::exact(1),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
    coercion_lift_profile: CoercionLiftProfile::UnaryNumericScalarOrArrayElementwise,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
};

pub fn fisherinv_kernel(y: f64) -> Result<f64, WorksheetErrorCode> {
    let e = (2.0 * y).exp();
    // BUG-FUNC-027 CLASS-A6: for large positive y, e=e^(2y) overflows to +Inf and
    // the direct form gives Inf/Inf = NaN; Excel saturates to +1. The large
    // negative side already saturates to -1 via e -> 0, so only +Inf needs a guard.
    // The interior formula is preserved bit-for-bit.
    if e.is_infinite() {
        return Ok(1.0);
    }
    Ok((e - 1.0) / (e + 1.0))
}

pub fn eval_fisherinv_surface(
    args: &[crate::value::CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, UnaryNumericSurfaceError> {
    eval_unary_numeric_surface(args, resolver, fisherinv_kernel)
}

pub fn map_fisherinv_error_to_ws(e: &UnaryNumericSurfaceError) -> WorksheetErrorCode {
    map_unary_numeric_error_to_ws(e)
}

#[cfg(test)]
mod tests {
    use super::*;

    // BUG-FUNC-027 CLASS-A6: live Excel 16.0 b20026 FISHERINV(817.81)=1.
    #[test]
    fn fisherinv_saturates_at_large_argument() {
        assert_eq!(fisherinv_kernel(817.81), Ok(1.0));
        assert_eq!(fisherinv_kernel(714.11), Ok(1.0));
        // Large negative saturates to -1 through the existing formula (e -> 0).
        assert_eq!(fisherinv_kernel(-817.81), Ok(-1.0));
    }

    #[test]
    fn fisherinv_interior_is_unchanged() {
        let y = 0.5_f64;
        let e = (2.0 * y).exp();
        assert_eq!(fisherinv_kernel(y), Ok((e - 1.0) / (e + 1.0)));
    }
}

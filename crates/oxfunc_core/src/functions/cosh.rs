use crate::function::{
    Arity, CoercionLiftProfile, DeterminismClass, ExcelRealPolicy, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::unary_numeric::{
    UnaryNumericExecSpec, UnaryNumericSurfaceError, eval_unary_numeric_via_executor,
    map_unary_numeric_error_to_ws,
};
use crate::resolver::ReferenceSystemProvider;
use crate::value::CalcValue;
use crate::value::WorksheetErrorCode;

pub const COSH_META: FunctionMeta = function_spec! {
    function_id: "FUNC.COSH",
    arity: Arity::exact(1),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    coercion_lift_profile: CoercionLiftProfile::UnaryNumericScalarOrArrayElementwise,
    kernel_signature_class: KernelSignatureClass::NumToNum,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
    // BUG-FUNC-027 CLASS-A3: COSH overflows to `#NUM!` in Excel, not `+Inf`.
    real_result_policy: ExcelRealPolicy::FINITE,
};

pub fn cosh_kernel(n: f64) -> f64 {
    // Live Excel 16.0 b20326 Range.Value2: COSH(x)=(EXP(x)+EXP(-x))/2
    // 40/40, including spilled EXP cells and E/2+EXP(-x)/2. libm cosh is
    // 1 ULP off Excel at 0.001, 0.01, and 10.
    let e = crate::excel_numeric::excel_exp(n);
    let em = crate::excel_numeric::excel_exp(-n);
    (e + em) / 2.0
}

pub fn eval_cosh_surface(
    args: &[crate::value::CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, UnaryNumericSurfaceError> {
    eval_unary_numeric_via_executor(
        args,
        resolver,
        UnaryNumericExecSpec::raw(cosh_kernel, COSH_META.real_result_policy),
    )
}

pub fn map_cosh_error_to_ws(e: &UnaryNumericSurfaceError) -> WorksheetErrorCode {
    map_unary_numeric_error_to_ws(e)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cosh_meta_function_id_is_stable() {
        assert_eq!(COSH_META.function_id, "FUNC.COSH");
    }

    #[test]
    fn cosh_kernel_matches_std_at_one() {
        // x=1 is on the EXP identity and also matches libm.
        assert_eq!(cosh_kernel(1.0), 1.0f64.cosh());
    }

    #[test]
    fn cosh_matches_live_excel_exp_pair_pins() {
        // Live Excel 16.0 b20326: COSH=(EXP+EXP(-))/2. These include the
        // three libm misses (0.001, 0.01, 10).
        assert_eq!(cosh_kernel(0.0).to_bits(), 0x3ff0000000000000);
        assert_eq!(cosh_kernel(0.5).to_bits(), 0x3ff20ac1862ae8d0);
        assert_eq!(cosh_kernel(1.0).to_bits(), 0x3ff8b07551d9f550);
        assert_eq!(cosh_kernel(0.001).to_bits(), 0x3ff000008637bdc2);
        assert_eq!(cosh_kernel(0.01).to_bits(), 0x3ff000346de27852);
        assert_eq!(cosh_kernel(10.0).to_bits(), 0x40c5829dd053712e);
        assert_eq!(cosh_kernel(-10.0).to_bits(), 0x40c5829dd053712e);
        assert_ne!(cosh_kernel(0.001).to_bits(), 0.001_f64.cosh().to_bits());
        assert_ne!(cosh_kernel(10.0).to_bits(), 10.0_f64.cosh().to_bits());
    }

    // BUG-FUNC-027 CLASS-A3: live Excel 16.0 b20026 COSH(-24230)=#NUM!.
    #[test]
    fn cosh_overflow_maps_to_num() {
        assert_eq!(
            COSH_META
                .real_result_policy
                .publish(-24230.0, cosh_kernel(-24230.0)),
            Err(WorksheetErrorCode::Num)
        );
        assert_eq!(
            COSH_META.real_result_policy.publish(1.0, cosh_kernel(1.0)),
            Ok(1.0f64.cosh())
        );
    }
}

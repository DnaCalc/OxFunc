use crate::function::{
    Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile, FunctionMeta,
    HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::unary_numeric::{
    UnaryNumericExecSpec, UnaryNumericSurfaceError, eval_unary_numeric_via_executor,
    map_unary_numeric_error_to_ws,
};
use crate::resolver::ReferenceSystemProvider;
use crate::value::CalcValue;
use crate::value::WorksheetErrorCode;

pub const ACOSH_META: FunctionMeta = function_spec! {
    function_id: "FUNC.ACOSH",
    arity: Arity::exact(1),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    coercion_lift_profile: CoercionLiftProfile::UnaryNumericScalarOrArrayElementwise,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
};

pub fn acosh_kernel(n: f64) -> Result<f64, WorksheetErrorCode> {
    if n < 1.0 {
        return Err(WorksheetErrorCode::Num);
    }
    // Inverse-problem identity, live Excel 16.0 b20228 Range.Value2:
    // ACOSH(x) == LN(x+SQRT(x*x-1)) on 29/29 including 1, 1+1ulp, and
    // large finite x. SQRT((x-1)*(x+1)) is not the graph (18/24).
    // When x*x overflows, Excel publishes #NUM! (ACOSH(1e200)), the same
    // intermediate overflow ASINH uses.
    if !(n * n).is_finite() {
        return Err(WorksheetErrorCode::Num);
    }
    Ok(crate::excel_numeric::excel_log(n + (n * n - 1.0).sqrt()))
}

pub fn eval_acosh_surface(
    args: &[crate::value::CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, UnaryNumericSurfaceError> {
    eval_unary_numeric_via_executor(
        args,
        resolver,
        UnaryNumericExecSpec::fallible(acosh_kernel, ACOSH_META.real_result_policy),
    )
}

pub fn map_acosh_error_to_ws(e: &UnaryNumericSurfaceError) -> WorksheetErrorCode {
    map_unary_numeric_error_to_ws(e)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acosh_meta_function_id_is_stable() {
        assert_eq!(ACOSH_META.function_id, "FUNC.ACOSH");
    }

    #[test]
    fn acosh_kernel_rejects_values_below_one() {
        assert_eq!(acosh_kernel(0.5), Err(WorksheetErrorCode::Num));
    }

    #[test]
    fn acosh_just_above_one_is_nonzero_matching_excel() {
        // BUG-FUNC-027 C5: the "Excel collapses to 0" near-1 witness was a
        // formula-literal artifact (the parser rounded 1+1e-15 down to 1.0).
        // Range.Value2 on Excel 16.0 b20228 publishes the LN identity at
        // 1+1ulp (0x3e56a09e67ffffff) and at 1+1e-15 (0x3e694c5839fffffe).
        let one_ulp = f64::from_bits(0x3ff0000000000001);
        assert_eq!(
            acosh_kernel(one_ulp).unwrap().to_bits(),
            0x3e56a09e67ffffff
        );
        let y = acosh_kernel(1.0 + 1e-15).expect("non-zero just above 1");
        assert_eq!(y.to_bits(), 0x3e694c5839fffffe);
    }

    #[test]
    fn acosh_follows_worksheet_ln_identity() {
        for x in [1.0, 1.0 + f64::EPSILON, 1.1, 2.0, 10.0, 1e6, 1e150] {
            let got = acosh_kernel(x).unwrap();
            let expect = crate::excel_numeric::excel_log(x + (x * x - 1.0).sqrt());
            assert_eq!(got.to_bits(), expect.to_bits(), "x={x}");
        }
        assert_eq!(acosh_kernel(1e200), Err(WorksheetErrorCode::Num));
    }
}

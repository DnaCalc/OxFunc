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

pub const SQRTPI_META: FunctionMeta = function_spec! {
    function_id: "FUNC.SQRTPI",
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

pub fn sqrtpi_kernel(n: f64) -> Result<f64, WorksheetErrorCode> {
    if n < 0.0 {
        return Err(WorksheetErrorCode::Num);
    }
    // Excel computes `sqrt(n·π)` and overflows when the `n·π` intermediate overflows f64,
    // publishing `#NUM!` rather than the (finite, representable) square root for such large n.
    // Reproduce that boundary *bit-exactly* by forming the same `n*π` intermediate and
    // rejecting when it is non-finite: this flips to `#NUM!` at exactly the input Excel does
    // (n > f64::MAX/π = 5.7222122209853366e307, verified live against Excel 16.0 build 20026
    // around 0x7fd45f306dc9c882↔0x7fd45f306dc9c883). No guessed magic constant — identical
    // f64 arithmetic yields Excel's exact threshold.
    let product = n * std::f64::consts::PI;
    if !product.is_finite() {
        return Err(WorksheetErrorCode::Num);
    }
    Ok(product.sqrt())
}

pub fn eval_sqrtpi_surface(
    args: &[crate::value::CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, UnaryNumericSurfaceError> {
    eval_unary_numeric_via_executor(
        args,
        resolver,
        UnaryNumericExecSpec::fallible(sqrtpi_kernel, SQRTPI_META.real_result_policy),
    )
}

pub fn map_sqrtpi_error_to_ws(e: &UnaryNumericSurfaceError) -> WorksheetErrorCode {
    map_unary_numeric_error_to_ws(e)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sqrtpi_meta_function_id_is_stable() {
        assert_eq!(SQRTPI_META.function_id, "FUNC.SQRTPI");
    }

    #[test]
    fn sqrtpi_kernel_negative_is_num() {
        assert_eq!(sqrtpi_kernel(-1.0), Err(WorksheetErrorCode::Num));
    }

    /// Excel publishes `#NUM!` once the internal `n·π` intermediate overflows f64, and the
    /// finite square root below that. The boundary is the EXACT `n*π`-overflow point,
    /// `n = f64::MAX/π` which rounds to `0x7fd45f306dc9c882`; the next f64
    /// (`0x7fd45f306dc9c883`) overflows. Probed live against Excel 16.0 build 20026
    /// (.tmp/asinh-sqrtpi-oracle4.ps1): Excel returns finite at and below `0x7fd45f306dc9c882`
    /// and `#NUM!` at and above `0x7fd45f306dc9c883`. OxFunc must flip at the SAME input.
    #[test]
    fn sqrtpi_kernel_matches_excel_num_on_internal_n_pi_overflow() {
        // Last finite input (n*π == f64::MAX, finite).
        let last_finite = f64::from_bits(0x7fd4_5f30_6dc9_c882);
        assert!((last_finite * std::f64::consts::PI).is_finite());
        // Stays finite (matching Excel's finite/#NUM! classification at this input). NOTE: at
        // this exact boundary input OxFunc's value is 0x5fefffffffffffff while Excel publishes
        // 0x5ff0000000000000 — a pre-existing 1-ULP finite-range gap (n*π rounds to exactly
        // f64::MAX; sqrt rounds down 1 ULP vs Excel's internal computation). Out of scope for
        // this overflow-guard fix; reported separately. We pin only that it stays finite here.
        assert!(sqrtpi_kernel(last_finite).is_ok());

        // First input whose `n*π` overflows -> Excel #NUM!.
        let first_num = f64::from_bits(0x7fd4_5f30_6dc9_c883);
        assert!(!(first_num * std::f64::consts::PI).is_finite());
        assert_eq!(sqrtpi_kernel(first_num), Err(WorksheetErrorCode::Num));

        // The range OxFunc historically published +Inf for (Excel #NUM!): 5.8e307 .. f64::MAX.
        for n in [5.8e307, 1.0e308, f64::MAX] {
            assert_eq!(
                sqrtpi_kernel(n),
                Err(WorksheetErrorCode::Num),
                "SQRTPI({n:e})"
            );
        }

        // Finite values below the threshold stay finite and bit-exact to Excel (oracle2.ps1).
        assert_eq!(
            sqrtpi_kernel(1.0e10).unwrap().to_bits(),
            0x4105_a2eb_14aa_5ae9
        );
        assert_eq!(
            sqrtpi_kernel(1.0e300).unwrap().to_bits(),
            0x5f21_53bf_f724_c583
        );
        assert_eq!(
            sqrtpi_kernel(1.0e307).unwrap().to_bits(),
            0x5fda_c128_1685_1ee0
        );
        assert_eq!(
            sqrtpi_kernel(5.0e307).unwrap().to_bits(),
            0x5fed_e997_0174_6551
        );
    }
}

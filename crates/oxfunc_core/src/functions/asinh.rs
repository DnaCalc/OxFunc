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

pub const ASINH_META: FunctionMeta = function_spec! {
    function_id: "FUNC.ASINH",
    arity: Arity::exact(1),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    coercion_lift_profile: CoercionLiftProfile::UnaryNumericScalarOrArrayElementwise,
    kernel_signature_class: KernelSignatureClass::NumToNum,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
};

pub fn asinh_kernel(n: f64) -> Result<f64, WorksheetErrorCode> {
    // Excel computes `ln(x + sqrt(x² + 1))` and overflows when the `x²` intermediate
    // overflows f64, publishing `#NUM!` rather than the (finite, representable) analytic
    // value for such large |x|. Reproduce that boundary *bit-exactly* by forming the same
    // `x*x` intermediate Excel forms and rejecting when it is non-finite: this flips to
    // `#NUM!` at exactly the input Excel does (|x| > sqrt(f64::MAX) = 1.3407807929942596e154,
    // verified live against Excel 16.0 build 20026 around 0x5fefffffffffffff↔0x5ff0000000000000).
    // No guessed magic constant — identical f64 arithmetic yields Excel's exact threshold.
    if !(n * n).is_finite() {
        return Err(WorksheetErrorCode::Num);
    }
    // Below the threshold Excel publishes the finite value sign(x) * ln(|x| + hypot(x, 1))
    // (bit-exact on the disputed lanes where platform libm `asinh` differs by 1+ ULP).
    Ok(n.signum() * (n.abs() + n.hypot(1.0)).ln())
}

pub fn eval_asinh_surface(
    args: &[crate::value::CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, UnaryNumericSurfaceError> {
    eval_unary_numeric_via_executor(
        args,
        resolver,
        UnaryNumericExecSpec::fallible(asinh_kernel, ASINH_META.real_result_policy),
    )
}

pub fn map_asinh_error_to_ws(e: &UnaryNumericSurfaceError) -> WorksheetErrorCode {
    map_unary_numeric_error_to_ws(e)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asinh_meta_function_id_is_stable() {
        assert_eq!(ASINH_META.function_id, "FUNC.ASINH");
    }

    #[test]
    fn asinh_kernel_matches_std() {
        assert_eq!(asinh_kernel(0.5), Ok(0.5f64.asinh()));
    }

    #[test]
    fn asinh_kernel_matches_pinned_excel_publication_on_disputed_rows() {
        assert_eq!(asinh_kernel(1.0), Ok(0.8813735870195429));
        assert_eq!(asinh_kernel(-1.0), Ok(-0.8813735870195429));
        assert_eq!(asinh_kernel(1.0e-10), Ok(1.000000082690371e-10));
        assert_eq!(asinh_kernel(-1.0e-10), Ok(-1.000000082690371e-10));
    }

    /// Excel publishes `#NUM!` once the internal `x²` intermediate overflows f64, and the
    /// finite analytic value below that. The boundary is the EXACT `x*x`-overflow point,
    /// `|x| = sqrt(f64::MAX) = 0x5fefffffffffffff`; the next f64 (`0x5ff0000000000000`)
    /// overflows. Probed live against Excel 16.0 build 20026 (.tmp/asinh-sqrtpi-oracle4.ps1):
    /// Excel returns finite at and below `0x5fefffffffffffff` and `#NUM!` at and above
    /// `0x5ff0000000000000`, on both signs. OxFunc must flip at the SAME input.
    #[test]
    fn asinh_kernel_matches_excel_num_on_internal_x_squared_overflow() {
        // Last finite input (|x| = sqrt(f64::MAX)); `x*x` is finite (== f64::MAX).
        let last_finite = f64::from_bits(0x5fef_ffff_ffff_ffff);
        assert_eq!(last_finite, last_finite.abs());
        assert!((last_finite * last_finite).is_finite());
        // Finite, and bit-exact to Excel's published value at this boundary input.
        assert_eq!(asinh_kernel(last_finite), Ok(355.58450362725193));
        assert_eq!(
            asinh_kernel(last_finite).unwrap().to_bits(),
            0x4076_395a_2079_b70c
        );
        assert_eq!(asinh_kernel(-last_finite), Ok(-355.58450362725193));

        // First input whose `x*x` overflows -> Excel #NUM!, both signs.
        let first_num = f64::from_bits(0x5ff0_0000_0000_0000);
        assert!(!(first_num * first_num).is_finite());
        assert_eq!(asinh_kernel(first_num), Err(WorksheetErrorCode::Num));
        assert_eq!(asinh_kernel(-first_num), Err(WorksheetErrorCode::Num));

        // The wide range the previous `|x| + hypot(x,1)` guard drifted across (Excel #NUM!,
        // OxFunc historically finite/+Inf): 1e155 .. f64::MAX.
        for x in [1.0e155, 1.0e200, 1.0e300, 1.0e308, f64::MAX] {
            assert_eq!(
                asinh_kernel(x),
                Err(WorksheetErrorCode::Num),
                "ASINH({x:e})"
            );
            assert_eq!(
                asinh_kernel(-x),
                Err(WorksheetErrorCode::Num),
                "ASINH(-{x:e})"
            );
        }

        // Mid-range finite values stay finite and bit-exact to Excel (oracle2.ps1).
        assert_eq!(
            asinh_kernel(1.0e150).unwrap().to_bits(),
            0x4075_a14b_6977_fcd2
        );
        assert_eq!(
            asinh_kernel(1.3e154).unwrap().to_bits(),
            0x4076_38db_9c34_9fe9
        );
    }
}

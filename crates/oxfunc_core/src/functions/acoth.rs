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

pub const ACOTH_META: FunctionMeta = function_spec! {
    function_id: "FUNC.ACOTH",
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

/// Above this `|x|`, Excel's ACOTH switches from the direct ratio-log to the
/// reciprocal ln1p-pair `ATANH(1/x)` path (W109). The switch is bracketed by the
/// live corpus near `3.5` (rows just below want the ratio, just above want the
/// pair); the exact double is not yet pinned (needs dense probes — see W109 ACOTH).
const ACOTH_PAIR_FLOOR: f64 = 3.5;

pub fn acoth_kernel(n: f64) -> Result<f64, WorksheetErrorCode> {
    let a = n.abs();
    if a <= 1.0 {
        return Err(WorksheetErrorCode::Num);
    }
    // W109 identification (2026-07-12): Excel's ACOTH is exactly ODD (compute on
    // |x|, restore the sign) and splits into two regimes, mirroring ATANH:
    //   * `|x| < ~3.5`: the direct ratio `0.5·ln((|x|+1)/(|x|-1))` with the x87
    //     CRT ln (the binary64 ratio's rounding is load-bearing), i.e. ATANH's
    //     region-C form on `1/|x|` but with the ratio formed directly.
    //   * `|x| >= ~3.5`: the reciprocal ln1p pair `0.5·(ln1p(1/|x|)−ln1p(−1/|x|))`
    //     via the x87 `fyl2xp1` pair — i.e. `ATANH(1/|x|)` on the region-B path.
    // Strictly dominates the prior platform-ln1p form (0 regressions, +19 rows on
    // the 56-row live corpus; 53/56). Residual: 3 pair-branch rows (±5, +8.1) at
    // +-1..2 ULP and the exact switch double remain open (dense probes — see W109).
    let mag = if a < ACOTH_PAIR_FLOOR {
        0.5 * crate::excel_numeric::excel_log((a + 1.0) / (a - 1.0))
    } else {
        crate::excel_numeric::excel_atanh_small(1.0 / a)
    };
    Ok(mag.copysign(n))
}

pub fn eval_acoth_surface(
    args: &[crate::value::CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, UnaryNumericSurfaceError> {
    eval_unary_numeric_via_executor(
        args,
        resolver,
        UnaryNumericExecSpec::fallible(acoth_kernel, ACOTH_META.real_result_policy),
    )
}

pub fn map_acoth_error_to_ws(e: &UnaryNumericSurfaceError) -> WorksheetErrorCode {
    map_unary_numeric_error_to_ws(e)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acoth_meta_function_id_is_stable() {
        assert_eq!(ACOTH_META.function_id, "FUNC.ACOTH");
    }

    #[test]
    fn acoth_kernel_rejects_abs_one() {
        assert_eq!(acoth_kernel(1.0), Err(WorksheetErrorCode::Num));
        assert_eq!(acoth_kernel(-1.0), Err(WorksheetErrorCode::Num));
    }

    #[test]
    fn acoth_just_above_one_is_finite_matching_excel() {
        // BUG-FUNC-027 C5: the "Excel #NUM!" near-1 witness was a formula-literal
        // artifact (the parser rounded 1+ULP down to 1.0). With the exact input
        // 1 + 2^-52, live Excel 16.0 b20026 returns 18.36840028483855, which this
        // kernel matches bit-for-bit — so it must stay finite, not collapse to #NUM!.
        let y = acoth_kernel(1.0 + f64::EPSILON).expect("finite just above 1");
        assert!((y - 18.36840028483855).abs() < 1e-10, "got {y}");
    }

    #[test]
    fn acoth_large_and_negative_args_bit_exact() {
        // BUG-FUNC-027 C5: the direct 0.5*ln((x+1)/(x-1)) form drifted up to ~1.2e14
        // ULP for large |x|. The odd-symmetric ln1p form matches live Excel 16.0
        // b20026 bit-for-bit across the range (a 1-ULP x87-ln residual remains at a
        // few scattered mid-range points, tracked on catalog G4). Bits from elem-probe.
        assert_eq!(
            acoth_kernel(1_000_000.0).unwrap().to_bits(),
            0x3eb0_c6f7_a0b5_f3b3
        );
        assert_eq!(
            acoth_kernel(1.001).unwrap().to_bits(),
            0x400e_67d6_037b_1a46
        );
        // Excel's ACOTH is exactly odd-symmetric.
        assert_eq!(acoth_kernel(-2.0).unwrap(), -acoth_kernel(2.0).unwrap());
        assert_eq!(
            acoth_kernel(-1_000_000.0).unwrap(),
            -acoth_kernel(1_000_000.0).unwrap()
        );
    }

    /// W109 (2026-07-12): the two-regime identification. ACOTH(2) takes the
    /// ratio branch (|x| < 3.5) and equals ATANH(0.5) bit-for-bit (the ACOTH(x) =
    /// ATANH(1/x) identity — 0x3fe193ea7aad030b); ACOTH(20) takes the reciprocal
    /// ln1p-pair branch (|x| >= 3.5). Both pinned from live Excel 16.0 build 20131.
    #[test]
    fn acoth_two_regime_matches_live_excel_pinned_witnesses() {
        assert_eq!(acoth_kernel(2.0).unwrap().to_bits(), 0x3fe1_93ea_7aad_030b); // ratio branch
        assert_eq!(acoth_kernel(20.0).unwrap().to_bits(), 0x3fa9_9f11_cd5f_7091); // pair branch
        assert_eq!(acoth_kernel(-20.0).unwrap().to_bits(), 0xbfa9_9f11_cd5f_7091); // odd
    }
}

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

pub const ATANH_META: FunctionMeta = function_spec! {
    function_id: "FUNC.ATANH",
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

/// Below this |x|, Excel switches ATANH off the naive log-ratio onto a distinct
/// accurate small-argument path (an x87 `fyl2xp1` ln1p-difference, per W109);
/// the exact switch and a non-odd transition band near `1e-4` are not yet pinned,
/// so the retained platform path below the boundary is unchanged.
const ATANH_RATIO_FLOOR: f64 = 1.25e-4;

pub fn atanh_kernel(n: f64) -> Result<f64, WorksheetErrorCode> {
    let a = n.abs();
    if a >= 1.0 {
        return Err(WorksheetErrorCode::Num);
    }
    // W109 identification (2026-07-12): for |x| >= ~1.25e-4 — which covers the
    // entire catalog band (mid-small witnesses) AND the near-1 rows — Excel's
    // ATANH is the naive log-ratio 0.5·ln((1+x)/(1-x)) evaluated with the x87
    // CRT ln, bit-for-bit (163/163 live rows). The double-rounding of the
    // binary64 ratio is load-bearing; a higher-precision ratio does NOT match.
    // The form is naturally odd (x -> -x maps the ratio to its reciprocal, ln
    // negates), so no abs/copysign is needed here.
    if a >= ATANH_RATIO_FLOOR {
        return Ok(0.5 * crate::excel_numeric::excel_log((1.0 + n) / (1.0 - n)));
    }
    // Small-|x| region: distinct accurate path in Excel (ln1p-difference). The
    // platform atanh already matches there for tiny x (and atanh(x)->x as x->0);
    // computing on |x| and restoring the sign preserves the odd symmetry the
    // platform libm otherwise loses near the domain edge. Region-B bit-exactness
    // (x87 ln1p-pair) and the ~1e-4 transition band remain open — see W109.
    Ok(a.atanh().copysign(n))
}

pub fn eval_atanh_surface(
    args: &[crate::value::CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, UnaryNumericSurfaceError> {
    eval_unary_numeric_via_executor(
        args,
        resolver,
        UnaryNumericExecSpec::fallible(atanh_kernel, ATANH_META.real_result_policy),
    )
}

pub fn map_atanh_error_to_ws(e: &UnaryNumericSurfaceError) -> WorksheetErrorCode {
    map_unary_numeric_error_to_ws(e)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atanh_meta_function_id_is_stable() {
        assert_eq!(ATANH_META.function_id, "FUNC.ATANH");
    }

    #[test]
    fn atanh_kernel_rejects_abs_one() {
        assert_eq!(atanh_kernel(1.0), Err(WorksheetErrorCode::Num));
        assert_eq!(atanh_kernel(-1.0), Err(WorksheetErrorCode::Num));
    }

    #[test]
    fn atanh_is_odd_symmetric_and_bit_exact_near_minus_one() {
        // BUG-FUNC-027 C4: the platform libm broke odd symmetry near -1 (up to
        // ~1.5e13 ULP). The |x|-then-copysign form matches live Excel 16.0 b20026
        // bit-for-bit; bits pinned from the elem-probe ledger.
        assert_eq!(
            atanh_kernel(-0.9999999999999990).unwrap().to_bits(),
            0xc031_9dc9_df78_50b1
        );
        assert_eq!(
            atanh_kernel(-0.999999999).unwrap().to_bits(),
            0xc025_6a9a_0b9b_2416
        );
        let x = 0.9999999999999990_f64;
        assert_eq!(atanh_kernel(-x).unwrap(), -atanh_kernel(x).unwrap());
    }

    #[test]
    fn atanh_x87_log_candidate_is_not_global() {
        // The ratio-log is the production path only for |x| >= the floor; below
        // it the small-arg path must be retained. The subnormal input is the
        // canonical witness: the ratio rounds to 1 and collapses to zero, while
        // live Excel (and the retained path) preserve the input.
        let candidate = |x: f64| 0.5 * crate::excel_numeric::excel_log((1.0 + x) / (1.0 - x));
        let tiny = f64::from_bits(1);
        assert_eq!(candidate(tiny).to_bits(), 0.0_f64.to_bits());
        assert_eq!(atanh_kernel(tiny).unwrap().to_bits(), tiny.to_bits());
    }

    /// W109 (2026-07-12): region-C ratio-log is bit-exact to live Excel 16.0
    /// build 20131 across the catalog mid-small band and up to the domain edge
    /// (163/163 answered rows; pins from G4-hyp / G4-02 answer sets).
    #[test]
    fn atanh_region_c_matches_live_excel_pinned_witnesses() {
        let pins: [(u64, u64); 5] = [
            (0x3fb9_9999_9999_999a, 0x3fb9_af93_cd23_4415), // 0.1
            (0x3fc9_9999_9999_999a, 0x3fc9_f323_ecbf_9849), // 0.2
            (0x3f94_7ae1_47ae_147b, 0x3f94_7b94_47a9_a9f8), // 0.02 (band floor)
            (0x3fe0_0000_0000_0000, 0x3fe1_93ea_7aad_030b), // 0.5
            (0x3fde_147a_e147_ae14, 0x3fe0_527f_06cd_3e63), // 0.47
        ];
        for (xb, want) in pins {
            let x = f64::from_bits(xb);
            assert_eq!(atanh_kernel(x).unwrap().to_bits(), want, "x={x}");
        }
    }

    /// W109 (2026-07-12): Excel ATANH is NOT exactly odd in region C — it
    /// evaluates the signed ratio 0.5·ln((1+x)/(1-x)) directly, and the negative
    /// argument rounds independently. Live Excel: ATANH(-0.2) = 0xbfc9f323ecbf984a,
    /// which is 1 ULP away from -ATANH(0.2) = 0xbfc9f323ecbf9849. The prior
    /// copysign-forced oddness therefore introduced a divergence; the signed ratio
    /// reproduces Excel's actual (non-odd) value.
    #[test]
    fn atanh_region_c_is_not_odd_and_matches_signed_ratio() {
        let x = f64::from_bits(0x3fc9_9999_9999_999a); // 0.2
        assert_eq!(atanh_kernel(-x).unwrap().to_bits(), 0xbfc9_f323_ecbf_984a);
        assert_ne!(
            atanh_kernel(-x).unwrap().to_bits(),
            (-atanh_kernel(x).unwrap()).to_bits()
        );
    }
}

use crate::function::{
    Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile, FunctionMeta,
    HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::binary_numeric::{
    BinaryNumericSurfaceError, eval_binary_numeric_surface, map_binary_numeric_error_to_ws,
};
use crate::functions::factorial_common::trunc_nonnegative;
use crate::resolver::ReferenceSystemProvider;
use crate::value::CalcValue;
use crate::value::WorksheetErrorCode;

pub const PERMUT_META: FunctionMeta = function_spec! {
    function_id: "FUNC.PERMUT",
    arity: Arity::exact(2),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    coercion_lift_profile: CoercionLiftProfile::UnaryNumericScalarOnly,
    kernel_signature_class: KernelSignatureClass::NumsToNum,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
};

/// Worksheet `PERMUT` — bit-exact to 64-bit Excel on `x86_64` (W109 Phase 2,
/// unique surviving candidate over 402 live witnesses, build 20131): the
/// ASCENDING legacy x87 spill-loop product
/// `acc = RN53(RN64(acc · f))` for `f = n-k+1 ..= n` — not a factorial
/// ratio (the former `n!/(n-k)!` staging is 1 ULP off on the catalog witness
/// and overflows spuriously for large `n`).
pub fn permut_kernel(n: f64, k: f64) -> Result<f64, WorksheetErrorCode> {
    use crate::excel_numeric::excel_x87_mul;
    let n = trunc_nonnegative(n)?;
    let k = trunc_nonnegative(k)?;
    if k > n {
        return Err(WorksheetErrorCode::Num);
    }
    let mut acc = 1.0f64;
    for f in (n - k + 1)..=n {
        acc = excel_x87_mul(acc, f as f64);
    }
    if acc.is_finite() {
        Ok(acc)
    } else {
        Err(WorksheetErrorCode::Num)
    }
}

pub fn eval_permut_surface(
    args: &[crate::value::CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, BinaryNumericSurfaceError> {
    eval_binary_numeric_surface(args, resolver, permut_kernel)
}

pub fn map_permut_error_to_ws(e: &BinaryNumericSurfaceError) -> WorksheetErrorCode {
    map_binary_numeric_error_to_ws(e)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_bits(actual: f64, expected: f64) {
        assert_eq!(
            actual.to_bits(),
            expected.to_bits(),
            "{actual} vs {expected}"
        );
    }

    #[test]
    fn permut_meta_function_id_is_stable() {
        assert_eq!(PERMUT_META.function_id, "FUNC.PERMUT");
    }

    #[test]
    fn permut_kernel_matches_excel_lanes() {
        assert_eq!(permut_kernel(10.0, 3.0), Ok(720.0));
        assert_eq!(permut_kernel(10.9, 3.2), Ok(720.0));
        assert_eq!(permut_kernel(0.0, 0.0), Ok(1.0));
        assert_eq!(permut_kernel(3.0, 4.0), Err(WorksheetErrorCode::Num));
        assert_eq!(permut_kernel(-1.0, 1.0), Err(WorksheetErrorCode::Num));
    }

    #[test]
    fn permut_exact_publication_controls_remain_exact() {
        assert_bits(permut_kernel(10.0, 3.0).expect("permut(10,3)"), 720.0_f64);
        assert_bits(permut_kernel(9.0, 2.0).expect("permut(9,2)"), 72.0_f64);
    }

    /// W109 PERMUT identification pin — live Excel 16.0 build 20131. The
    /// former catalog witness PERMUT(61,20) was 1 ULP off under the factorial
    /// ratio; the ascending spill-loop product matches Excel's bits.
    #[test]
    fn permut_matches_live_excel_pinned_witnesses() {
        assert_eq!(
            permut_kernel(61.0, 20.0).unwrap().to_bits(),
            0x470760c0a63908aa
        );
        assert_eq!(
            permut_kernel(500.0, 1.0).unwrap().to_bits(),
            500.0f64.to_bits()
        );
    }
}

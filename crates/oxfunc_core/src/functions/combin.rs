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

pub const COMBIN_META: FunctionMeta = function_spec! {
    function_id: "FUNC.COMBIN",
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

/// Worksheet `COMBIN` on the current x64 Excel reference profile.
///
/// W109 identified a cyclic rising-factor graph rather than the conventional
/// multiply-first recurrence. After `k = min(k, n-k)`, Excel evaluates
///
/// `product(i=2..=k, (n-k+i-1)/i) * n`
///
/// in ascending `i`, storing every quotient and accumulator product from x87
/// PC64 to binary64 (`RN53(RN64(op))`), then multiplying `n` last through the
/// same stored-x87 operation. This graph is exact on 505 legacy rows, 20,713
/// current-build discovery rows, and a frozen 1,024-row publication heldout.
pub fn combin_kernel(n: f64, k: f64) -> Result<f64, WorksheetErrorCode> {
    use crate::excel_numeric::{excel_x87_div, excel_x87_mul};

    let n = trunc_nonnegative(n)?;
    let raw_k = trunc_nonnegative(k)?;
    if raw_k > n {
        return Err(WorksheetErrorCode::Num);
    }
    let k = raw_k.min(n - raw_k);
    if k == 0 {
        return Ok(1.0);
    }

    let mut acc = 1.0;
    for i in 2..=k {
        let numerator = (n - k + i - 1) as f64;
        let factor = excel_x87_div(numerator, i as f64);
        acc = excel_x87_mul(acc, factor);
    }
    let result = excel_x87_mul(acc, n as f64);
    if result.is_finite() {
        Ok(result)
    } else {
        Err(WorksheetErrorCode::Num)
    }
}

pub fn eval_combin_surface(
    args: &[crate::value::CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, BinaryNumericSurfaceError> {
    eval_binary_numeric_surface(args, resolver, combin_kernel)
}

pub fn map_combin_error_to_ws(e: &BinaryNumericSurfaceError) -> WorksheetErrorCode {
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
    fn combin_meta_function_id_is_stable() {
        assert_eq!(COMBIN_META.function_id, "FUNC.COMBIN");
    }

    #[test]
    fn combin_kernel_matches_excel_truncation_and_num_lanes() {
        assert_eq!(combin_kernel(5.0, 2.0), Ok(10.0));
        assert_eq!(combin_kernel(5.9, 2.2), Ok(10.0));
        assert_eq!(combin_kernel(5.0, 6.0), Err(WorksheetErrorCode::Num));
        assert_eq!(combin_kernel(-1.0, 1.0), Err(WorksheetErrorCode::Num));
    }

    #[test]
    fn combin_exact_publication_controls_remain_exact() {
        assert_bits(combin_kernel(10.0, 3.0).expect("combin(10,3)"), 120.0_f64);
        assert_bits(combin_kernel(9.0, 2.0).expect("combin(9,2)"), 36.0_f64);
    }

    /// W109 current-build discovery pins. These representable integers are
    /// published one ULP away from the correctly rounded value, which rules
    /// out the former multiply-first kernel.
    #[test]
    fn combin_matches_live_excel_cyclic_spill_pins() {
        assert_eq!(
            combin_kernel(9.0, 3.0).unwrap().to_bits(),
            0x4054_ffff_ffff_ffff
        );
        assert_eq!(
            combin_kernel(23.0, 10.0).unwrap().to_bits(),
            0x4131_7502_0000_0001
        );
        assert_eq!(
            combin_kernel(200.0, 3.0).unwrap().to_bits(),
            0x4134_0a77_ffff_ffff
        );
        assert_eq!(
            combin_kernel(23.0, 13.0).unwrap().to_bits(),
            0x4131_7502_0000_0001
        );
    }

    /// Frozen 1,024-row publication-heldout discriminator. The plain RN53
    /// recurrence is one ULP low here; the selected stored-x87 graph matches.
    #[test]
    fn combin_matches_frozen_publication_heldout_pin() {
        assert_eq!(
            combin_kernel(258.0, 49.0).unwrap().to_bits(),
            0x4afe_ff3e_f80e_71b4
        );
    }
}

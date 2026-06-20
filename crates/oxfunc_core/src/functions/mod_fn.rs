use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::binary_numeric::{
    BinaryNumericSurfaceError, eval_binary_numeric_surface, map_binary_numeric_error_to_ws,
};
use crate::resolver::ReferenceSystemProvider;
use crate::value::CalcValue;
use crate::value::WorksheetErrorCode;

pub const MOD_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.MOD",
    arity: Arity::exact(2),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
    coercion_lift_profile: CoercionLiftProfile::UnaryNumericScalarOnly,
    kernel_signature_class: KernelSignatureClass::NumsToNum,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
    real_result_policy: FunctionMeta::DEFAULT_REAL_RESULT_POLICY,
};

/// Excel `MOD` returns `#NUM!` once the implicit quotient `|number/divisor|`
/// reaches an internal magnitude limit (above it Excel cannot recover the
/// remainder from `INT(number/divisor)`). The boundary is on the *quotient*
/// (not `|number|`) and is d-independent; it was bisected against live Excel
/// 16.0 build 20026 to the exact double below (`MOD(q,1)` flips from a number to
/// `#NUM!` between `1125899999999.9998` and `1125900000000`). See BUG-FUNC-027 B1.
const MOD_QUOTIENT_NUM_LIMIT: f64 = 1_125_900_000_000.0;

pub fn mod_kernel(number: f64, divisor: f64) -> Result<f64, WorksheetErrorCode> {
    if divisor == 0.0 {
        return Err(WorksheetErrorCode::Div0);
    }
    if (number / divisor).abs() >= MOD_QUOTIENT_NUM_LIMIT {
        return Err(WorksheetErrorCode::Num);
    }
    // Excel's MOD is the floored modulo (sign of the divisor). Compute it from the
    // exact IEEE remainder (`%` is fmod — no rounding) and adjust the sign, rather than
    // `number - divisor*floor(number/divisor)`, which catastrophically cancels for large
    // quotients (the two terms are ~equal and ~1e10, so the remainder loses up to
    // ~9.5e10 ULP). BUG-FUNC-027 C2.
    let remainder = number % divisor;
    if remainder != 0.0 && (remainder < 0.0) != (divisor < 0.0) {
        Ok(remainder + divisor)
    } else {
        Ok(remainder)
    }
}

pub fn eval_mod_surface(
    args: &[crate::value::CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, BinaryNumericSurfaceError> {
    eval_binary_numeric_surface(args, resolver, mod_kernel)
}

pub fn map_mod_error_to_ws(e: &BinaryNumericSurfaceError) -> WorksheetErrorCode {
    map_binary_numeric_error_to_ws(e)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mod_meta_function_id_is_stable() {
        assert_eq!(MOD_META.function_id, "FUNC.MOD");
    }

    #[test]
    fn mod_kernel_matches_excel_sign_behavior() {
        assert_eq!(mod_kernel(3.0, 2.0), Ok(1.0));
        assert_eq!(mod_kernel(-3.0, 2.0), Ok(1.0));
        assert_eq!(mod_kernel(3.0, -2.0), Ok(-1.0));
        assert_eq!(mod_kernel(-3.0, -2.0), Ok(-1.0));
        assert_eq!(mod_kernel(3.0, 0.0), Err(WorksheetErrorCode::Div0));
    }

    #[test]
    fn mod_large_quotient_matches_excel_num_threshold() {
        // BUG-FUNC-027 B1: Excel returns #NUM! once the *quotient* |n/d| reaches the
        // internal limit (d-independent), bisected against live Excel 16.0 b20026.
        assert_eq!(mod_kernel(1.005e14, 1.0), Err(WorksheetErrorCode::Num)); // witness
        assert_eq!(mod_kernel(-4.44e14, 0.288), Err(WorksheetErrorCode::Num)); // witness
        assert_eq!(
            mod_kernel(MOD_QUOTIENT_NUM_LIMIT, 1.0),
            Err(WorksheetErrorCode::Num)
        ); // at limit
        assert!(mod_kernel(2f64.powi(40) + 2f64.powi(34), 1.0).is_ok()); // just below
        assert_eq!(
            mod_kernel(2f64.powi(40) + 2f64.powi(35), 1.0),
            Err(WorksheetErrorCode::Num)
        ); // just above
        assert!(mod_kernel(2f64.powi(50), 2f64.powi(10)).is_ok()); // quotient 2^40, |n|=2^50
        assert_eq!(
            mod_kernel(2f64.powi(51), 2f64.powi(10)),
            Err(WorksheetErrorCode::Num)
        ); // quotient 2^41
    }

    #[test]
    fn mod_large_quotient_is_bit_exact_via_fmod() {
        // BUG-FUNC-027 C2: `n - d*floor(n/d)` catastrophically cancels for large
        // quotients (up to ~9.5e10 ULP). The fmod form is bit-exact vs live Excel
        // 16.0 b20026; bits pinned from the elem-probe ledger.
        assert_eq!(
            mod_kernel(-9.26e9, 1.86).unwrap().to_bits(),
            0x3fe4_7ae2_4ccf_fbc0
        );
        assert_eq!(
            mod_kernel(-78170.05, 1.0).unwrap().to_bits(),
            0x3fee_6666_6666_0000
        );
        assert_eq!(
            mod_kernel(9.65e9, -0.374).unwrap().to_bits(),
            0xbfcb_22d0_b429_8e28
        );
    }
}

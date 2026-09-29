use crate::function::{
    Arity, CoercionLiftProfile, DeterminismClass, ExcelRealPolicy, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::binary_numeric::{
    BinaryNumericSurfaceError, map_binary_numeric_error_to_ws,
};
use crate::functions::adapters::coerce_prepared_to_number;
use crate::functions::distribution_common::run_distribution_lifted;
use crate::resolver::ReferenceSystemProvider;
use crate::value::{CalcValue, CoreValue};
use crate::value::WorksheetErrorCode;

pub const PERMUTATIONA_META: FunctionMeta = function_spec! {
    function_id: "FUNC.PERMUTATIONA",
    arity: Arity::exact(2),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::NumsToNum,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
    // BUG-FUNC-027 CLASS-A5: n^k overflows to `#NUM!` in Excel, not `+Inf`.
    real_result_policy: ExcelRealPolicy::FINITE,
};

pub fn permutationa_kernel(n: f64, k: f64) -> Result<f64, WorksheetErrorCode> {
    // W111 distinguishes raw-base admission from exponent truncation:
    // a negative fractional base is invalid, but -1 < k < 0 truncates to zero.
    if !n.is_finite() || !k.is_finite() || n < 0.0 || n >= 2147483647.0
        || k >= 2147483647.0 || k.trunc() < 0.0 {
        return Err(WorksheetErrorCode::Num);
    }
    let n = n.trunc();
    let mut k = k.trunc() as u32;
    // The base-ten branch agrees with decimal power conversion, distinguishable
    // from repeated squaring at 10^99, 10^100 and later powers. Fresh validation
    // covers the full finite decimal exponent range before any parity promotion.
    let result = if n == 10.0 {
        if k > 308 { f64::INFINITY }
        else { crate::coercion::scale_decimal_pair_to_binary(1, k as i32).unwrap_or(f64::INFINITY) }
    } else {
        let mut result = 1.0;
        let mut base = n;
        while k > 0 {
            if k & 1 != 0 { result = crate::excel_numeric::excel_x87_mul(result, base); }
            k >>= 1;
            if k > 0 { base = crate::excel_numeric::excel_x87_mul(base, base); }
        }
        result
    };
    PERMUTATIONA_META
        .real_result_policy
        .publish(n, result)
}

pub fn eval_permutationa_surface(
    args: &[crate::value::CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, BinaryNumericSurfaceError> {
    run_distribution_lifted(args, resolver, |values| {
        if values.len() != 2 {
            return Err(BinaryNumericSurfaceError::ArityMismatch { expected: 2, actual: values.len() });
        }
        let number = |value: &CalcValue| {
            if matches!(value.core(), CoreValue::Missing) { Ok(0.0) }
            else { coerce_prepared_to_number(value).map_err(BinaryNumericSurfaceError::Coercion) }
        };
        let n = number(&values[0])?;
        let k = number(&values[1])?;
        permutationa_kernel(n, k).map(CalcValue::number).map_err(BinaryNumericSurfaceError::Domain)
    }, map_binary_numeric_error_to_ws, BinaryNumericSurfaceError::Coercion)
}

pub fn map_permutationa_error_to_ws(e: &BinaryNumericSurfaceError) -> WorksheetErrorCode {
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
    fn permutationa_meta_function_id_is_stable() {
        assert_eq!(PERMUTATIONA_META.function_id, "FUNC.PERMUTATIONA");
    }

    #[test]
    fn permutationa_kernel_matches_excel_lanes() {
        assert_eq!(permutationa_kernel(3.0, 2.0), Ok(9.0));
        assert_eq!(permutationa_kernel(3.9, 2.1), Ok(9.0));
        assert_eq!(permutationa_kernel(0.0, 0.0), Ok(1.0));
        assert_eq!(permutationa_kernel(0.0, 1.0), Ok(0.0));
        assert_eq!(permutationa_kernel(-1.0, 1.0), Err(WorksheetErrorCode::Num));
        // BUG-FUNC-027 CLASS-A5: live Excel 16.0 b20026 PERMUTATIONA(163,150)=#NUM!.
        assert_eq!(
            permutationa_kernel(163.0, 150.0),
            Err(WorksheetErrorCode::Num)
        );
    }

    #[test]
    fn permutationa_exact_publication_controls_remain_exact() {
        assert_bits(
            permutationa_kernel(3.0, 2.0).expect("permutationa(3,2)"),
            9.0_f64,
        );
        assert_bits(
            permutationa_kernel(4.0, 3.0).expect("permutationa(4,3)"),
            64.0_f64,
        );
    }
}

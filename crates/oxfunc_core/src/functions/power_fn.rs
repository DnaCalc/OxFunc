use crate::function::{
    Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile, FunctionMeta,
    HostInteractionClass, KernelSignatureClass, PrecisionRoundingProfile, ThreadSafetyClass,
    VolatilityClass,
};
use crate::functions::binary_numeric::{
    BinaryNumericSurfaceError, eval_binary_numeric_surface, map_binary_numeric_error_to_ws,
};
use crate::functions::excel_numeric::excel_underflow_to_zero;
use crate::resolver::ReferenceSystemProvider;
use crate::value::CalcValue;
use crate::value::WorksheetErrorCode;

pub const POWER_META: FunctionMeta = function_spec! {
    function_id: "FUNC.POWER",
    arity: Arity::exact(2),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    coercion_lift_profile: CoercionLiftProfile::UnaryNumericScalarOnly,
    kernel_signature_class: KernelSignatureClass::NumsToNum,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
    // POWER (and the `^` operator and the financial growth callers, which all share `power_kernel`)
    // publishes an exact-integer exponent via repeated multiplication rather than `powf` — a real,
    // separable precision deviation, declared once here. `power_kernel` reads this policy instead of
    // unconditionally hand-coding the rule; the integer-detection tolerance and the
    // binary-exponentiation algorithm live in this module (the impl that interprets the variant),
    // never as data on the meta. Verified live Excel 16.0 build 20026.
    precision_rounding_profile: PrecisionRoundingProfile::IntegerExponentPublication,
};

fn exact_integer_exponent(power: f64) -> Option<i64> {
    if !power.is_finite() {
        return None;
    }
    let truncated = power.trunc();
    if power != truncated || truncated < i64::MIN as f64 || truncated > i64::MAX as f64 {
        None
    } else {
        Some(truncated as i64)
    }
}

fn detect_reciprocal_odd_integer(power: f64) -> Option<i64> {
    if !power.is_finite() || power <= 0.0 || power >= 1.0 {
        return None;
    }
    let tolerance = 32.0 * f64::EPSILON * power.abs().max(1.0);
    let mut best_q: Option<i64> = None;
    let mut best_diff = f64::INFINITY;
    let mut q = 3_i64;
    while q <= 255 {
        let recip = 1.0_f64 / q as f64;
        let diff = (power - recip).abs();
        if diff < best_diff {
            best_diff = diff;
            best_q = Some(q);
        }
        q += 2;
    }
    if best_diff <= tolerance { best_q } else { None }
}

fn powi_excel_publication(number: f64, power: i64) -> f64 {
    if power == 0 {
        return 1.0;
    }

    let negative = power < 0;
    let mut exponent = power.unsigned_abs();
    let mut base = number;
    let mut result = 1.0;

    while exponent > 0 {
        if exponent & 1 == 1 {
            result *= base;
        }
        exponent >>= 1;
        if exponent > 0 {
            base *= base;
        }
    }

    if negative { 1.0 / result } else { result }
}

pub fn power_kernel(number: f64, power: f64) -> Result<f64, WorksheetErrorCode> {
    if number == 0.0 && power == 0.0 {
        return Err(WorksheetErrorCode::Num);
    }

    if number == 0.0 && power < 0.0 {
        return Err(WorksheetErrorCode::Div0);
    }

    // The exact-integer-exponent publication is the declared precision quirk on `POWER_META`
    // (shared by the `^` operator and the financial growth callers, which all funnel through this
    // kernel). The kernel CONSULTS the declared variant rather than carrying its own copy of the
    // rule; the integer-detection tolerance and the binary-exponentiation algorithm below are this
    // module's interpretation of that variant. The policy is `IntegerExponentPublication` today, so
    // this gate is active and the published result is bit-identical to the prior hand-coded path.
    let integer_publication = POWER_META
        .precision_rounding_profile
        .uses_integer_exponent_publication();
    let result = if let Some(integer_power) =
        exact_integer_exponent(power).filter(|_| integer_publication)
    {
        powi_excel_publication(number, integer_power)
    } else if number < 0.0 {
        if detect_reciprocal_odd_integer(power).is_some() {
            -((power * (-number).ln()).exp())
        } else {
            number.powf(power)
        }
    } else {
        number.powf(power)
    };
    if result.is_nan() {
        Err(WorksheetErrorCode::Num)
    } else if result.is_infinite() {
        // BUG-FUNC-027 CLASS-A4: a finite-input overflow is #NUM!; a +-Inf produced
        // by a negative exponent over a sub-unit base (1 / underflowed-to-zero) is
        // #DIV/0! in Excel, consistent with the 0^negative path above.
        if power < 0.0 {
            Err(WorksheetErrorCode::Div0)
        } else {
            Err(WorksheetErrorCode::Num)
        }
    } else {
        Ok(excel_underflow_to_zero(result))
    }
}

pub fn eval_power_surface(
    args: &[crate::value::CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, BinaryNumericSurfaceError> {
    eval_binary_numeric_surface(args, resolver, power_kernel)
}

pub fn map_power_error_to_ws(e: &BinaryNumericSurfaceError) -> WorksheetErrorCode {
    map_binary_numeric_error_to_ws(e)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn power_meta_function_id_is_stable() {
        assert_eq!(POWER_META.function_id, "FUNC.POWER");
    }

    #[test]
    fn power_kernel_matches_excel_domain_lanes() {
        assert_eq!(power_kernel(2.0, 3.0), Ok(8.0));
        assert_eq!(power_kernel(2.0, -3.0), Ok(0.125));
        assert_eq!(power_kernel(2.0, -1023.0), Ok(0.0));
        assert_eq!(power_kernel(2.0, -1022.0), Ok(f64::MIN_POSITIVE));
        assert_eq!(power_kernel(0.0, 0.0), Err(WorksheetErrorCode::Num));
        assert_eq!(power_kernel(-0.0, 0.0), Err(WorksheetErrorCode::Num));
        assert_eq!(power_kernel(0.0, -1.0), Err(WorksheetErrorCode::Div0));
        assert_eq!(power_kernel(-1.0, 0.5), Err(WorksheetErrorCode::Num));
    }

    // BUG-FUNC-027 CLASS-A4: live Excel 16.0 b20026 POWER(10,700)=#NUM!,
    // POWER(0.001,-700)=#DIV/0!, POWER(10,-700)=0.
    #[test]
    fn power_kernel_overflow_maps_to_excel_error_codes() {
        assert_eq!(power_kernel(10.0, 700.0), Err(WorksheetErrorCode::Num));
        assert_eq!(power_kernel(0.001, -700.0), Err(WorksheetErrorCode::Div0));
        assert_eq!(power_kernel(10.0, -700.0), Ok(0.0));
    }

    #[test]
    fn power_kernel_matches_excel_integer_publication_rows() {
        assert_eq!(power_kernel(1.05, 10.0), Ok(1.6288946267774416));
        assert_eq!(power_kernel(1.01, 48.0), Ok(1.6122260776824653));
        assert_eq!(
            power_kernel(1.0 + 0.08 / 12.0, 10.0),
            Ok(1.0687026403740616)
        );
    }

    #[test]
    fn power_kernel_matches_excel_negative_base_reciprocal_odd_root_rows() {
        // Reciprocal odd-integer roots of negative bases: bit-exact Excel publication
        // via -exp(power * ln(-base)). 1/3 and its 16/17-digit decimal literals
        // round to the same f64, so all three reach the exp/ln path identically.
        assert_eq!(power_kernel(-8.0, 1.0 / 3.0), Ok(-1.9999999999999998));
        assert_eq!(
            power_kernel(-8.0, 0.3333333333333333),
            Ok(-1.9999999999999998)
        );
        assert_eq!(
            power_kernel(-8.0, 0.33333333333333331),
            Ok(-1.9999999999999998)
        );
        assert_eq!(power_kernel(-27.0, 1.0 / 3.0), Ok(-2.9999999999999996));
        assert_eq!(power_kernel(-32.0, 1.0 / 5.0), Ok(-2.0));

        // Non-reciprocal-odd negative-base exponents fall through to #NUM!.
        assert_eq!(power_kernel(-8.0, 2.0 / 3.0), Err(WorksheetErrorCode::Num));
        assert_eq!(power_kernel(-8.0, 0.5), Err(WorksheetErrorCode::Num));
    }
}

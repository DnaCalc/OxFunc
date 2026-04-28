use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::binary_numeric::{
    BinaryNumericSurfaceError, eval_binary_numeric_surface, map_binary_numeric_error_to_ws,
};
use crate::functions::excel_numeric::excel_underflow_to_zero;
use crate::resolver::ReferenceResolver;
use crate::value::{EvalValue, WorksheetErrorCode};

pub const POWER_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.POWER",
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

    let result = if let Some(integer_power) = exact_integer_exponent(power) {
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
    } else {
        Ok(excel_underflow_to_zero(result))
    }
}

pub fn eval_power_surface(
    args: &[crate::value::CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, BinaryNumericSurfaceError> {
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

use crate::coercion::CoercionError;
use crate::function::{
    Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile, FunctionMeta,
    HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::coerce_prepared_to_number;
use crate::functions::binary_numeric::{BinaryNumericSurfaceError, eval_binary_numeric_surface};
use crate::resolver::ReferenceSystemProvider;
use crate::value::CalcValue;
use crate::value::WorksheetErrorCode;

pub const ROUND_META: FunctionMeta = function_spec! {
    function_id: "FUNC.ROUND",
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

#[derive(Debug, Clone, PartialEq)]
pub enum RoundEvalError {
    ArityMismatch { expected: usize, actual: usize },
    Coercion(CoercionError),
    Domain(WorksheetErrorCode),
}

impl From<BinaryNumericSurfaceError> for RoundEvalError {
    fn from(value: BinaryNumericSurfaceError) -> Self {
        match value {
            BinaryNumericSurfaceError::ArityMismatch { expected, actual } => {
                Self::ArityMismatch { expected, actual }
            }
            BinaryNumericSurfaceError::Coercion(error) => Self::Coercion(error),
            BinaryNumericSurfaceError::Domain(code) => Self::Domain(code),
        }
    }
}

fn parse_digits(arg: &CalcValue) -> Result<i32, RoundEvalError> {
    let digits = coerce_prepared_to_number(arg).map_err(RoundEvalError::Coercion)?;
    Ok(digits.trunc() as i32)
}

/// Rounding direction for [`excel_decimal_round`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecimalRoundMode {
    /// ROUND: half away from zero.
    HalfAwayFromZero,
    /// ROUNDUP: away from zero whenever anything is dropped.
    AwayFromZero,
    /// ROUNDDOWN: toward zero.
    TowardZero,
}

/// `|n|` taken to 15 significant digits as `(significand, exponent)` with
/// `|n| = significand * 10^exponent`. An exact tie at the 16th digit goes up in magnitude when
/// `ties_away` (ROUNDUP, ROUNDDOWN: 283025097725723.5 -> ...724, -8026461466170265 -> ...270)
/// and down otherwise (ROUND: 2664283639896305 -> ...300). Live Excel 20430, W111 G8-14.
fn fifteen_significant_digits(n: f64, ties_away: bool) -> (u128, i32) {
    let text = format!("{:.30e}", n.abs());
    let (mantissa, exponent) = text.split_once('e').expect("exponent form");
    let exponent: i32 = exponent.parse().expect("exponent");
    let digits = mantissa.replace('.', "");
    let (head, tail) = digits.split_at(15);
    let mut significand: u128 = head.parse().expect("15 digits");
    let half = format!("5{}", "0".repeat(tail.len() - 1));
    let round_up = match tail.cmp(half.as_str()) {
        std::cmp::Ordering::Greater => true,
        std::cmp::Ordering::Less => false,
        std::cmp::Ordering::Equal => ties_away,
    };
    if round_up {
        significand += 1;
    }
    (significand, exponent - 14)
}

fn decimal_to_f64(significand: u128, scale: i32) -> f64 {
    format!("{significand}e{scale}").parse().expect("decimal literal")
}

/// Excel's ROUND / ROUNDUP / ROUNDDOWN. The value is first taken to 15 significant digits
/// (exact ties: see [`fifteen_significant_digits`]); then ROUND rounds half away from zero and ROUNDDOWN
/// truncates at `digits` decimal places, in decimal, converting the decimal result to the
/// nearest double; ROUNDUP is the ROUNDDOWN result plus one unit of 10^-digits, added in binary
/// (`ROUNDUP(1.35, 1)` is 1.3 + 0.1 = 1.4000000000000001). Binary `(n * 10^d).round() / 10^d`
/// differed on half-way decimals (`ROUND(1.005, 2)` is 1.01), on digits beyond 15 significant,
/// and in last bits. Subnormal inputs flush to 0 and a zero result is +0. Live Excel 20430 on
/// two fresh corpora, W111 G8-14.
pub fn excel_decimal_round(n: f64, digits: i32, mode: DecimalRoundMode) -> f64 {
    if !n.is_finite() {
        return n;
    }
    if n == 0.0 || n.abs() < f64::MIN_POSITIVE {
        return 0.0;
    }
    let ties_away = mode != DecimalRoundMode::HalfAwayFromZero;
    let (significand, scale) = fifteen_significant_digits(n, ties_away);
    let dropped = -(scale + digits);
    let signed = |m: f64| if n < 0.0 { -m } else { m };
    if dropped <= 0 {
        return signed(decimal_to_f64(significand, scale));
    }
    let (quotient, remainder, half_or_more) = if dropped > 30 {
        (0u128, significand, false)
    } else {
        let divisor = 10u128.pow(dropped as u32);
        let (q, r) = (significand / divisor, significand % divisor);
        (q, r, r * 2 >= divisor)
    };
    let kept_scale = scale + dropped;
    let magnitude = match mode {
        DecimalRoundMode::HalfAwayFromZero => {
            decimal_to_f64(quotient + u128::from(half_or_more), kept_scale)
        }
        DecimalRoundMode::TowardZero => decimal_to_f64(quotient, kept_scale),
        DecimalRoundMode::AwayFromZero => {
            let down = decimal_to_f64(quotient, kept_scale);
            if remainder == 0 {
                down
            } else {
                down + decimal_to_f64(1, kept_scale)
            }
        }
    };
    if magnitude == 0.0 { 0.0 } else { signed(magnitude) }
}

pub fn round_kernel(n: f64, digits: i32) -> f64 {
    excel_decimal_round(n, digits, DecimalRoundMode::HalfAwayFromZero)
}

pub fn eval_round_adapter_prepared(args: &[CalcValue]) -> Result<CalcValue, RoundEvalError> {
    if !ROUND_META.arity.accepts(args.len()) {
        return Err(RoundEvalError::ArityMismatch {
            expected: ROUND_META.arity.min,
            actual: args.len(),
        });
    }

    let value = coerce_prepared_to_number(&args[0]).map_err(RoundEvalError::Coercion)?;
    let digits = parse_digits(&args[1])?;
    Ok(CalcValue::number(round_kernel(value, digits)))
}

pub fn eval_round_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, RoundEvalError> {
    eval_binary_numeric_surface(args, resolver, |value, digits| {
        Ok(round_kernel(value, digits.trunc() as i32))
    })
    .map_err(RoundEvalError::from)
}

pub fn map_round_error_to_ws(e: &RoundEvalError) -> WorksheetErrorCode {
    match e {
        RoundEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        RoundEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        RoundEvalError::Coercion(_) => WorksheetErrorCode::Value,
        RoundEvalError::Domain(code) => *code,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::ReferenceSystemCapabilities;
    use crate::value::CalcArray;

    struct NoResolver;

    impl ReferenceSystemProvider for NoResolver {
        fn capabilities(&self) -> ReferenceSystemCapabilities {
            ReferenceSystemCapabilities::permissive_local()
        }

        fn dereference(
            &self,
            request: &crate::resolver::ReferenceDereferenceRequest,
        ) -> Result<CalcValue, crate::resolver::ReferenceResolutionError> {
            let reference = &request.reference;
            Err(
                crate::resolver::ReferenceResolutionError::UnresolvedReference {
                    target: reference.target().to_string(),
                },
            )
        }
    }

    #[test]
    fn round_kernel_rounds_half_away_from_zero() {
        assert_eq!(round_kernel(1.25, 1), 1.3);
        assert_eq!(round_kernel(-1.25, 1), -1.3);
    }

    #[test]
    fn eval_round_supports_negative_digits() {
        let got = eval_round_surface(
            &[(CalcValue::number(123.0)), (CalcValue::number(-1.0))],
            &NoResolver,
        );
        assert_eq!(got, Ok(CalcValue::number(120.0)));
    }

    #[test]
    fn eval_round_truncates_digits_toward_zero() {
        let got = eval_round_surface(
            &[(CalcValue::number(1.5)), (CalcValue::number(0.9))],
            &NoResolver,
        );
        assert_eq!(got, Ok(CalcValue::number(2.0)));
    }

    #[test]
    fn eval_round_spills_array_arguments() {
        let got = eval_round_surface(
            &[
                (CalcValue::array(
                    CalcArray::from_rows(vec![
                        vec![CalcValue::number(1.234)],
                        vec![CalcValue::number(2.345)],
                    ])
                    .unwrap(),
                )),
                (CalcValue::array(
                    CalcArray::from_rows(vec![
                        vec![CalcValue::number(0.0)],
                        vec![CalcValue::number(1.0)],
                    ])
                    .unwrap(),
                )),
            ],
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(CalcValue::array(
                CalcArray::from_rows(vec![
                    vec![CalcValue::number(1.0)],
                    vec![CalcValue::number(2.3)],
                ])
                .unwrap()
            ))
        );
    }

    /// W111 G8-14, live Excel 20430: decimal rounding on the 15-significant-digit value.
    #[test]
    fn excel_decimal_rounding_matches_excel_rows() {
        use super::DecimalRoundMode::*;
        let r = super::excel_decimal_round;
        assert_eq!(round_kernel(1.005, 2), 1.01);
        assert_eq!(round_kernel(-1.005, 2), -1.01);
        assert_eq!(round_kernel(0.285, 2), 0.29);
        assert_eq!(round_kernel(0.49999999999999994, 0), 1.0);
        assert_eq!(round_kernel(106.16375978790555, 16), 106.163759787906);
        assert_eq!(round_kernel(2664283639896305.0, 0), 2664283639896300.0);
        assert_eq!(r(1.35, 1, AwayFromZero), 1.3 + 0.1);
        assert_eq!(r(283025097725723.5, 0, TowardZero), 283025097725724.0);
        assert_eq!(r(-8026461466170265.0, 7, TowardZero), -8026461466170270.0);
        assert_eq!(round_kernel(1e-308, 308), 0.0);
    }

}

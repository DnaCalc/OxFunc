use crate::coercion::CoercionError;
use crate::function::{
    Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile, FunctionMeta,
    HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{coerce_prepared_to_number, prepare_args_values_only};
use crate::functions::binary_numeric::BinaryNumericSurfaceError;
use crate::resolver::ReferenceSystemProvider;
use crate::value::{ArrayShape, CalcArray, CalcValue, CoreValue, WorksheetErrorCode};

pub const ROUND_META: FunctionMeta = function_spec! {
    function_id: "FUNC.ROUND",
    arity: Arity::exact(2),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    coercion_lift_profile: CoercionLiftProfile::Custom,
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

fn rounding_number(arg: &CalcValue) -> Result<f64, CoercionError> {
    if matches!(arg.core(), CoreValue::Missing) {
        Ok(0.0)
    } else {
        coerce_prepared_to_number(arg)
    }
}

/// Rounding-local preparation: Missing is zero; array padding supplies NA at
/// its own argument position. Earlier coercion errors remain visible.
pub(crate) fn eval_rounding_prepared(
    args: &[CalcValue],
    kernel: impl Fn(f64, f64) -> Result<f64, WorksheetErrorCode>,
) -> Result<CalcValue, BinaryNumericSurfaceError> {
    if args.len() != 2 {
        return Err(BinaryNumericSurfaceError::ArityMismatch {
            expected: 2,
            actual: args.len(),
        });
    }
    let on_cell = |lhs: &CalcValue, rhs: &CalcValue| {
        let number = rounding_number(lhs).map_err(BinaryNumericSurfaceError::Coercion)?;
        let count = rounding_number(rhs).map_err(BinaryNumericSurfaceError::Coercion)?;
        kernel(number, count)
            .map(CalcValue::number)
            .map_err(BinaryNumericSurfaceError::Domain)
    };
    let shape = args
        .iter()
        .fold(ArrayShape { rows: 1, cols: 1 }, |shape, arg| {
            if let CoreValue::Array(array) = arg.core() {
                ArrayShape {
                    rows: shape.rows.max(array.shape().rows),
                    cols: shape.cols.max(array.shape().cols),
                }
            } else {
                shape
            }
        });
    if shape == (ArrayShape { rows: 1, cols: 1 }) {
        return on_cell(&args[0], &args[1]);
    }
    let mut cells = Vec::with_capacity(shape.cell_count());
    for row in 0..shape.rows {
        for col in 0..shape.cols {
            let at = |arg: &CalcValue| match arg.core() {
                CoreValue::Array(array) => array
                    .get(
                        if array.shape().rows == 1 { 0 } else { row },
                        if array.shape().cols == 1 { 0 } else { col },
                    )
                    .cloned()
                    .unwrap_or_else(|| CalcValue::error(WorksheetErrorCode::NA)),
                _ => arg.clone(),
            };
            let result = on_cell(&at(&args[0]), &at(&args[1])).unwrap_or_else(|error| {
                let code = match error {
                    BinaryNumericSurfaceError::Coercion(CoercionError::WorksheetError(code))
                    | BinaryNumericSurfaceError::Domain(code) => code,
                    _ => WorksheetErrorCode::Value,
                };
                CalcValue::error(code)
            });
            cells.push(result);
        }
    }
    Ok(CalcValue::array(
        CalcArray::new(shape, cells).expect("rounding shape preserved"),
    ))
}

pub(crate) fn eval_rounding_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
    kernel: impl Fn(f64, f64) -> Result<f64, WorksheetErrorCode>,
) -> Result<CalcValue, BinaryNumericSurfaceError> {
    let prepared =
        prepare_args_values_only(args, resolver).map_err(BinaryNumericSurfaceError::Coercion)?;
    eval_rounding_prepared(&prepared, kernel)
}

// Black-box digit conversion distinguishes ROUND's signed conversion from
// the magnitude/sign preparation used by TRUNC, ROUNDUP and ROUNDDOWN.
pub(crate) fn round_digit_count(value: f64) -> i32 {
    let magnitude = value.abs().trunc() as i32;
    if value < 0.0 {
        -magnitude
    } else {
        magnitude
    }
}

pub(crate) fn directed_digit_count(value: f64) -> i32 {
    let magnitude = ((value.abs().trunc() as u32) & 0xffff) as i32;
    if value < 0.0 {
        -magnitude
    } else {
        magnitude
    }
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
/// the rounded approximation `significand * 10^exponent`. An exact tie at the 16th digit goes up in magnitude when
/// `ties_away` (ROUNDUP, ROUNDDOWN: 283025097725723.5 -> ...724, -8026461466170265 -> ...270)
/// and down otherwise (ROUND: 2664283639896305 -> ...300). Live Excel 20430, W111 G8-14.
pub(crate) fn fifteen_significant_digits(n: f64, ties_away: bool) -> (u128, i32) {
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

fn decimal_to_f64(significand: u128, scale: i64) -> f64 {
    if significand == 0 || scale < -323 {
        return 0.0;
    }
    if scale > 308 {
        return f64::INFINITY;
    }
    crate::coercion::scale_decimal_pair_to_binary(significand as u64, scale as i32)
        .unwrap_or(f64::INFINITY)
}

/// Excel's ROUND / ROUNDUP / ROUNDDOWN. The value is first taken to 15 significant digits
/// (exact ties: see [`fifteen_significant_digits`]); then ROUND rounds half away from zero and ROUNDDOWN
/// truncates at `digits` decimal places, in decimal, converting each decimal pair through
/// the evidenced RN64 nibble graph; ROUNDUP adds one unit of 10^-digits in binary64
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
    let signed = |m: f64| if n < 0.0 { -m } else { m };
    if ties_away {
        if let Some(initial) =
            crate::coercion::rounding_initial_subnormal(significand as u64, scale)
        {
            return signed(initial);
        }
    }
    let dropped = if mode == DecimalRoundMode::HalfAwayFromZero {
        // ROUND combines the signed count and decimal point in a wrapping i32
        // carrier. W111 width discriminators exercise both overflow directions.
        15_i64 - i64::from(digits.wrapping_add(scale + 15))
    } else {
        // Directed modes retain the unsigned low16 count, with its separate
        // sign. The decimal truncation stage substitutes 100 above i16::MAX;
        // ROUNDUP's added unit below still uses the original converted count.
        let decimal_digits = if digits.unsigned_abs() > 32767 {
            if digits < 0 {
                -100
            } else {
                100
            }
        } else {
            digits
        };
        -(i64::from(scale) + i64::from(decimal_digits))
    };
    if dropped <= 0 {
        let magnitude = decimal_to_f64(significand, i64::from(scale));
        return if magnitude == 0.0 {
            0.0
        } else {
            signed(magnitude)
        };
    }
    let (quotient, remainder, half_or_more) = if dropped > 30 {
        (0u128, significand, false)
    } else {
        let divisor = 10u128.pow(dropped as u32);
        let (q, r) = (significand / divisor, significand % divisor);
        (q, r, r * 2 >= divisor)
    };
    let kept_scale = i64::from(scale) + dropped;
    let magnitude = match mode {
        DecimalRoundMode::HalfAwayFromZero => {
            decimal_to_f64(quotient + u128::from(half_or_more), kept_scale)
        }
        DecimalRoundMode::TowardZero => decimal_to_f64(quotient, kept_scale),
        DecimalRoundMode::AwayFromZero => {
            let down = decimal_to_f64(quotient, kept_scale);
            // W111 signed residual ladders distinguish this publication test
            // from normal subtraction underflow: positive residuals below
            // 2^-1026 have a zero upper16, while negatives retain their sign.
            // The subtraction uses the normalized initial15 value, not n.
            let initial = decimal_to_f64(significand, i64::from(scale));
            let difference = signed(initial) - signed(down);
            if remainder == 0 || difference.to_bits() >> 48 == 0 {
                down
            } else {
                down + decimal_to_f64(1, -i64::from(digits))
            }
        }
    };
    if magnitude == 0.0 {
        0.0
    } else {
        signed(magnitude)
    }
}

pub fn round_kernel(n: f64, digits: i32) -> f64 {
    excel_decimal_round(n, digits, DecimalRoundMode::HalfAwayFromZero)
}

pub fn eval_round_adapter_prepared(args: &[CalcValue]) -> Result<CalcValue, RoundEvalError> {
    eval_rounding_prepared(args, |value, digits| {
        let result = round_kernel(value, round_digit_count(digits));
        if result.is_finite() {
            Ok(result)
        } else {
            Err(WorksheetErrorCode::Num)
        }
    })
    .map_err(RoundEvalError::from)
}

pub fn eval_round_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, RoundEvalError> {
    eval_rounding_surface(args, resolver, |value, digits| {
        let result = round_kernel(value, round_digit_count(digits));
        if result.is_finite() {
            Ok(result)
        } else {
            Err(WorksheetErrorCode::Num)
        }
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

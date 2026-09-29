use crate::coercion::CoercionError;
use crate::function::{
    Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile, FunctionMeta,
    HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    coerce_prepared_to_number, run_values_only_prepared_lifted,
};
use crate::resolver::ReferenceSystemProvider;
use crate::value::{CalcValue, CoreValue};
use crate::value::WorksheetErrorCode;

pub const DATE_META: FunctionMeta = function_spec! {
    function_id: "FUNC.DATE",
    arity: Arity::exact(3),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
};

#[derive(Debug, Clone, PartialEq)]
pub enum DateEvalError {
    ArityMismatch { expected: usize, actual: usize },
    Coercion(CoercionError),
    NumericDomain,
}

/// Observed date-argument conversion on the current reference. The inclusive
/// boundary is the same two-step binary rounding rule characterized for ADDRESS.
/// Keep the result floating until the caller applies its own integer-width rule.
pub(crate) fn date_integer_floor(number: f64) -> f64 {
    let upper = number.ceil();
    let threshold = if upper > 0.0 {
        2.0_f64.powi(-22) + 2.0_f64.powi(-33)
    } else {
        2.0_f64.powi(-23) + 2.0_f64.powi(-34)
    };
    if upper - number <= threshold { upper } else { number.floor() }
}

fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = year - i64::from(month <= 2);
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let yoe = year - era * 400;
    let mp = month + if month > 2 { -3 } else { 9 };
    let doy = (153 * mp + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

fn excel_serial_from_ymd_unbounded_1900(year: i64, month: i64, day: i64) -> i64 {
    let base = days_from_civil(1899, 12, 31);
    let month_start = days_from_civil(year, month, 1) - base;
    let month_start = if month_start >= 60 { month_start + 1 } else { month_start };
    month_start + day - 1
}

pub fn eval_date_adapter_prepared(args: &[CalcValue]) -> Result<CalcValue, DateEvalError> {
    if !DATE_META.arity.accepts(args.len()) {
        return Err(DateEvalError::ArityMismatch {
            expected: DATE_META.arity.min,
            actual: args.len(),
        });
    }

    let number = |arg: &CalcValue| match arg.core() {
        CoreValue::Missing | CoreValue::Empty => Ok(0.0),
        _ => coerce_prepared_to_number(arg).map_err(DateEvalError::Coercion),
    };
    let raw_year = number(&args[0])?;
    let raw_month = number(&args[1])?;
    let raw_day = number(&args[2])?;
    if !raw_year.is_finite() || !raw_month.is_finite() || !raw_day.is_finite() {
        return Err(DateEvalError::NumericDomain);
    }
    let original_year = date_integer_floor(raw_year).min(10_000.0);
    // Black-box probes expose a positive cap and signed 16-bit year conversion.
    // Apply the short-year offset after conversion; a negative resulting year
    // may still become an admitted year when a large month is normalized.
    let year_bits = if (-2_147_483_648.0..2_147_483_648.0).contains(&original_year) {
        original_year as i64
    } else { 0 };
    let year = year_bits as i16 as i64;
    let year = year + if year < 1900 { 1900 } else { 0 };
    let month = date_integer_floor(raw_month);
    if !(-32_767.0..32_767.0).contains(&month) {
        return Err(DateEvalError::NumericDomain);
    }
    let month = month as i64;
    let day = date_integer_floor(raw_day);
    let day = if (-32_768.0..=32_767.0).contains(&day) { day as i64 } else { 32_767 };

    let month_index = year
        .checked_mul(12)
        .and_then(|v| v.checked_add(month - 1))
        .ok_or(DateEvalError::NumericDomain)?;
    let normalized_year = month_index.div_euclid(12);
    let normalized_month = month_index.rem_euclid(12) + 1;

    if !(1900..=9999).contains(&normalized_year) {
        return Err(DateEvalError::NumericDomain);
    }

    let serial = excel_serial_from_ymd_unbounded_1900(normalized_year, normalized_month, day);
    if !(0..=2_958_465).contains(&serial) {
        return Err(DateEvalError::NumericDomain);
    }

    Ok(CalcValue::number(serial as f64))
}

pub fn eval_date_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, DateEvalError> {
    run_values_only_prepared_lifted(
        args,
        resolver,
        eval_date_adapter_prepared,
        map_date_error_to_ws,
        DateEvalError::Coercion,
    )
}

pub fn eval_date_calc_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, DateEvalError> {
    eval_date_surface(args, resolver)
}

pub fn map_date_error_to_ws(e: &DateEvalError) -> WorksheetErrorCode {
    match e {
        DateEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        DateEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        DateEvalError::Coercion(_) => WorksheetErrorCode::Value,
        DateEvalError::NumericDomain => WorksheetErrorCode::Num,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::ReferenceSystemCapabilities;

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
    fn eval_date_handles_1900_serial_baselines() {
        let got = eval_date_surface(
            &[
                (CalcValue::number(1900.0)),
                (CalcValue::number(1.0)),
                (CalcValue::number(1.0)),
            ],
            &NoResolver,
        );
        assert_eq!(got, Ok(CalcValue::number(1.0)));
    }

    #[test]
    fn eval_date_preserves_excel_1900_leap_bug_seed() {
        let got = eval_date_surface(
            &[
                (CalcValue::number(1900.0)),
                (CalcValue::number(2.0)),
                (CalcValue::number(29.0)),
            ],
            &NoResolver,
        );
        assert_eq!(got, Ok(CalcValue::number(60.0)));
    }

    #[test]
    fn eval_date_normalizes_month_overflow() {
        let got = eval_date_surface(
            &[
                (CalcValue::number(2024.0)),
                (CalcValue::number(14.0)),
                (CalcValue::number(1.0)),
            ],
            &NoResolver,
        );
        assert_eq!(got, Ok(CalcValue::number(45689.0)));
    }

    #[test]
    fn eval_date_allows_serial_zero_boundary() {
        let got = eval_date_surface(
            &[
                (CalcValue::number(1900.0)),
                (CalcValue::number(1.0)),
                (CalcValue::number(0.0)),
            ],
            &NoResolver,
        );
        assert_eq!(got, Ok(CalcValue::number(0.0)));
    }

    #[test]
    fn eval_date_rejects_month_zero_boundary() {
        let got = eval_date_surface(
            &[
                (CalcValue::number(1900.0)),
                (CalcValue::number(0.0)),
                (CalcValue::number(1.0)),
            ],
            &NoResolver,
        );
        assert_eq!(got, Err(DateEvalError::NumericDomain));
    }

    #[test]
    fn eval_date_normalizes_march_zero_to_excel_1900_leap_bug_day() {
        let got = eval_date_surface(
            &[
                (CalcValue::number(1900.0)),
                (CalcValue::number(3.0)),
                (CalcValue::number(0.0)),
            ],
            &NoResolver,
        );
        assert_eq!(got, Ok(CalcValue::number(60.0)));
    }

    #[test]
    fn eval_date_normalizes_january_sixtieth_to_excel_1900_leap_bug_day() {
        let got = eval_date_surface(
            &[
                (CalcValue::number(1900.0)),
                (CalcValue::number(1.0)),
                (CalcValue::number(60.0)),
            ],
            &NoResolver,
        );
        assert_eq!(got, Ok(CalcValue::number(60.0)));
    }

    #[test]
    fn eval_date_normalizes_short_year_and_truncates_day() {
        let got_short_year = eval_date_surface(
            &[
                (CalcValue::number(0.0)),
                (CalcValue::number(1.0)),
                (CalcValue::number(1.0)),
            ],
            &NoResolver,
        );
        assert_eq!(got_short_year, Ok(CalcValue::number(1.0)));

        let got_truncated_day = eval_date_surface(
            &[
                (CalcValue::number(2008.0)),
                (CalcValue::number(1.0)),
                (CalcValue::number(2.9)),
            ],
            &NoResolver,
        );
        assert_eq!(got_truncated_day, Ok(CalcValue::number(39449.0)));
    }
}

use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    coerce_prepared_to_number, prepare_calc_values_only, prepared_from_calc_value,
    run_values_only_prepared,
};
use crate::resolver::ReferenceSystemProvider;
use crate::value::CalcValue;
use crate::value::WorksheetErrorCode;

pub const DATE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.DATE",
    arity: Arity::exact(3),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
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

fn truncate_to_i64(arg: &CalcValue) -> Result<i64, DateEvalError> {
    Ok(coerce_prepared_to_number(arg)
        .map_err(DateEvalError::Coercion)?
        .trunc() as i64)
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

fn excel_1900_ordinal(month: i64, day: i64) -> i64 {
    let prefix = match month {
        1 => 0,
        2 => 31,
        3 => 60,
        4 => 91,
        5 => 121,
        6 => 152,
        7 => 182,
        8 => 213,
        9 => 244,
        10 => 274,
        11 => 305,
        12 => 335,
        _ => unreachable!("month normalized to 1..=12"),
    };
    prefix + day
}

fn excel_serial_from_ymd_unbounded_1900(year: i64, month: i64, day: i64) -> i64 {
    if year == 1900 && excel_1900_ordinal(month, day) == 60 {
        return 60;
    }

    let base = days_from_civil(1899, 12, 31);
    let days = days_from_civil(year, month, 1) - base + (day - 1);
    if days >= 60 { days + 1 } else { days }
}

pub fn eval_date_adapter_prepared(args: &[CalcValue]) -> Result<CalcValue, DateEvalError> {
    if !DATE_META.arity.accepts(args.len()) {
        return Err(DateEvalError::ArityMismatch {
            expected: DATE_META.arity.min,
            actual: args.len(),
        });
    }

    let mut year = truncate_to_i64(&args[0])?;
    if (0..=1899).contains(&year) {
        year += 1900;
    }
    let month = truncate_to_i64(&args[1])?;
    let day = truncate_to_i64(&args[2])?;

    let month_index = year
        .checked_mul(12)
        .and_then(|v| v.checked_add(month - 1))
        .ok_or(DateEvalError::NumericDomain)?;
    let normalized_year = month_index.div_euclid(12);
    let normalized_month = month_index.rem_euclid(12) + 1;

    if normalized_year < 0 || normalized_year > 9999 {
        return Err(DateEvalError::NumericDomain);
    }

    let serial = excel_serial_from_ymd_unbounded_1900(normalized_year, normalized_month, day);
    if serial < 0 {
        return Err(DateEvalError::NumericDomain);
    }

    Ok(CalcValue::number(serial as f64))
}

pub fn eval_date_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, DateEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_date_adapter_prepared,
        DateEvalError::Coercion,
    )
}

pub fn eval_date_calc_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, DateEvalError> {
    let prepared_calc =
        prepare_calc_values_only(args, resolver).map_err(DateEvalError::Coercion)?;
    let prepared = prepared_calc
        .iter()
        .map(prepared_from_calc_value)
        .collect::<Vec<_>>();
    eval_date_adapter_prepared(&prepared).map(CalcValue::from)
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

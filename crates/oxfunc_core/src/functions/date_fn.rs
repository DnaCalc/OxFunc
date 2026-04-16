use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    PreparedArgValue, coerce_prepared_to_number, run_values_only_prepared,
};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

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

fn truncate_to_i64(arg: &PreparedArgValue) -> Result<i64, DateEvalError> {
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

fn excel_serial_from_ymd_unbounded_1900(year: i64, month: i64, day: i64) -> i64 {
    if year == 1900 && month == 2 && day == 29 {
        return 60;
    }

    let base = days_from_civil(1899, 12, 31);
    let days = days_from_civil(year, month, 1) - base + (day - 1);
    if days >= 60 { days + 1 } else { days }
}

pub fn eval_date_adapter_prepared(args: &[PreparedArgValue]) -> Result<EvalValue, DateEvalError> {
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

    Ok(EvalValue::Number(serial as f64))
}

pub fn eval_date_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DateEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_date_adapter_prepared,
        DateEvalError::Coercion,
    )
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
    use crate::resolver::{RefResolutionError, ResolverCapabilities};
    use crate::value::ReferenceLike;

    struct NoResolver;

    impl ReferenceResolver for NoResolver {
        fn capabilities(&self) -> ResolverCapabilities {
            ResolverCapabilities::permissive_local()
        }

        fn resolve_reference(
            &self,
            reference: &ReferenceLike,
        ) -> Result<EvalValue, RefResolutionError> {
            Err(RefResolutionError::UnresolvedReference {
                target: reference.target.clone(),
            })
        }
    }

    #[test]
    fn eval_date_handles_1900_serial_baselines() {
        let got = eval_date_surface(
            &[
                CallArgValue::Eval(EvalValue::Number(1900.0)),
                CallArgValue::Eval(EvalValue::Number(1.0)),
                CallArgValue::Eval(EvalValue::Number(1.0)),
            ],
            &NoResolver,
        );
        assert_eq!(got, Ok(EvalValue::Number(1.0)));
    }

    #[test]
    fn eval_date_preserves_excel_1900_leap_bug_seed() {
        let got = eval_date_surface(
            &[
                CallArgValue::Eval(EvalValue::Number(1900.0)),
                CallArgValue::Eval(EvalValue::Number(2.0)),
                CallArgValue::Eval(EvalValue::Number(29.0)),
            ],
            &NoResolver,
        );
        assert_eq!(got, Ok(EvalValue::Number(60.0)));
    }

    #[test]
    fn eval_date_normalizes_month_overflow() {
        let got = eval_date_surface(
            &[
                CallArgValue::Eval(EvalValue::Number(2024.0)),
                CallArgValue::Eval(EvalValue::Number(14.0)),
                CallArgValue::Eval(EvalValue::Number(1.0)),
            ],
            &NoResolver,
        );
        assert_eq!(got, Ok(EvalValue::Number(45689.0)));
    }

    #[test]
    fn eval_date_allows_serial_zero_boundary() {
        let got = eval_date_surface(
            &[
                CallArgValue::Eval(EvalValue::Number(1900.0)),
                CallArgValue::Eval(EvalValue::Number(1.0)),
                CallArgValue::Eval(EvalValue::Number(0.0)),
            ],
            &NoResolver,
        );
        assert_eq!(got, Ok(EvalValue::Number(0.0)));
    }

    #[test]
    fn eval_date_rejects_month_zero_boundary() {
        let got = eval_date_surface(
            &[
                CallArgValue::Eval(EvalValue::Number(1900.0)),
                CallArgValue::Eval(EvalValue::Number(0.0)),
                CallArgValue::Eval(EvalValue::Number(1.0)),
            ],
            &NoResolver,
        );
        assert_eq!(got, Err(DateEvalError::NumericDomain));
    }

    #[test]
    fn eval_date_normalizes_march_zero_to_february_twenty_eight() {
        let got = eval_date_surface(
            &[
                CallArgValue::Eval(EvalValue::Number(1900.0)),
                CallArgValue::Eval(EvalValue::Number(3.0)),
                CallArgValue::Eval(EvalValue::Number(0.0)),
            ],
            &NoResolver,
        );
        assert_eq!(got, Ok(EvalValue::Number(59.0)));
    }

    #[test]
    fn eval_date_normalizes_short_year_and_truncates_day() {
        let got_short_year = eval_date_surface(
            &[
                CallArgValue::Eval(EvalValue::Number(0.0)),
                CallArgValue::Eval(EvalValue::Number(1.0)),
                CallArgValue::Eval(EvalValue::Number(1.0)),
            ],
            &NoResolver,
        );
        assert_eq!(got_short_year, Ok(EvalValue::Number(1.0)));

        let got_truncated_day = eval_date_surface(
            &[
                CallArgValue::Eval(EvalValue::Number(2008.0)),
                CallArgValue::Eval(EvalValue::Number(1.0)),
                CallArgValue::Eval(EvalValue::Number(2.9)),
            ],
            &NoResolver,
        );
        assert_eq!(got_truncated_day, Ok(EvalValue::Number(39449.0)));
    }
}

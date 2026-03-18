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

const EPSILON: f64 = 1.0e-12;

const CUMULATIVE_FINANCE_META_BASE: FunctionMeta = FunctionMeta {
    function_id: "FUNC.CUMULATIVE_FINANCE_BASE",
    arity: Arity::exact(6),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::None,
};

pub const CUMIPMT_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.CUMIPMT",
    ..CUMULATIVE_FINANCE_META_BASE
};

pub const CUMPRINC_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.CUMPRINC",
    ..CUMULATIVE_FINANCE_META_BASE
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaymentTiming {
    EndOfPeriod,
    BeginningOfPeriod,
}

impl PaymentTiming {
    fn factor(self, rate: f64) -> f64 {
        match self {
            Self::EndOfPeriod => 1.0,
            Self::BeginningOfPeriod => 1.0 + rate,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CumulativeFinanceEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
    Domain(WorksheetErrorCode),
}

fn arity_error(meta: &FunctionMeta, actual: usize) -> CumulativeFinanceEvalError {
    CumulativeFinanceEvalError::ArityMismatch {
        expected_min: meta.arity.min,
        expected_max: meta.arity.max,
        actual,
    }
}

fn number_arg(args: &[PreparedArgValue], idx: usize) -> Result<f64, CumulativeFinanceEvalError> {
    args.get(idx)
        .ok_or(CumulativeFinanceEvalError::Domain(
            WorksheetErrorCode::Value,
        ))
        .and_then(|value| {
            coerce_prepared_to_number(value).map_err(CumulativeFinanceEvalError::Coercion)
        })
}

fn validate_finite(values: &[f64]) -> Result<(), CumulativeFinanceEvalError> {
    if values.iter().all(|value| value.is_finite()) {
        Ok(())
    } else {
        Err(CumulativeFinanceEvalError::Domain(
            WorksheetErrorCode::Value,
        ))
    }
}

fn trunc_positive_period(value: f64) -> Result<i32, CumulativeFinanceEvalError> {
    if !value.is_finite() {
        return Err(CumulativeFinanceEvalError::Domain(
            WorksheetErrorCode::Value,
        ));
    }
    let truncated = value.trunc();
    if truncated < 1.0 || truncated > i32::MAX as f64 {
        return Err(CumulativeFinanceEvalError::Domain(WorksheetErrorCode::Num));
    }
    Ok(truncated as i32)
}

fn payment_timing_arg(value: f64) -> Result<PaymentTiming, CumulativeFinanceEvalError> {
    validate_finite(&[value])?;
    if (value - 0.0).abs() < EPSILON {
        Ok(PaymentTiming::EndOfPeriod)
    } else if (value - 1.0).abs() < EPSILON {
        Ok(PaymentTiming::BeginningOfPeriod)
    } else {
        Err(CumulativeFinanceEvalError::Domain(WorksheetErrorCode::Num))
    }
}

fn growth(rate: f64, periods: f64) -> Result<f64, CumulativeFinanceEvalError> {
    validate_finite(&[rate, periods])?;
    let base = 1.0 + rate;
    if base <= 0.0 {
        return Err(CumulativeFinanceEvalError::Domain(WorksheetErrorCode::Num));
    }
    let factor = base.powf(periods);
    if factor.is_finite() {
        Ok(factor)
    } else {
        Err(CumulativeFinanceEvalError::Domain(WorksheetErrorCode::Num))
    }
}

fn annuity_term(
    rate: f64,
    periods: f64,
    timing: PaymentTiming,
) -> Result<f64, CumulativeFinanceEvalError> {
    if rate.abs() < EPSILON {
        return Ok(periods);
    }
    let factor = growth(rate, periods)?;
    let term = timing.factor(rate) * (factor - 1.0) / rate;
    if term.is_finite() {
        Ok(term)
    } else {
        Err(CumulativeFinanceEvalError::Domain(WorksheetErrorCode::Num))
    }
}

fn fv(
    rate: f64,
    periods: f64,
    payment: f64,
    present_value: f64,
    timing: PaymentTiming,
) -> Result<f64, CumulativeFinanceEvalError> {
    validate_finite(&[rate, periods, payment, present_value])?;
    let factor = growth(rate, periods)?;
    let term = annuity_term(rate, periods, timing)?;
    let result = -(present_value * factor + payment * term);
    if result.is_finite() {
        Ok(result)
    } else {
        Err(CumulativeFinanceEvalError::Domain(WorksheetErrorCode::Num))
    }
}

fn pmt(
    rate: f64,
    periods: f64,
    present_value: f64,
    future_value: f64,
    timing: PaymentTiming,
) -> Result<f64, CumulativeFinanceEvalError> {
    validate_finite(&[rate, periods, present_value, future_value])?;
    if periods <= 0.0 {
        return Err(CumulativeFinanceEvalError::Domain(WorksheetErrorCode::Num));
    }
    if rate.abs() < EPSILON {
        return Ok(-(future_value + present_value) / periods);
    }
    let factor = growth(rate, periods)?;
    let term = annuity_term(rate, periods, timing)?;
    if term.abs() < EPSILON {
        return Err(CumulativeFinanceEvalError::Domain(WorksheetErrorCode::Num));
    }
    let result = -(future_value + present_value * factor) / term;
    if result.is_finite() {
        Ok(result)
    } else {
        Err(CumulativeFinanceEvalError::Domain(WorksheetErrorCode::Num))
    }
}

fn ipmt(
    rate: f64,
    period_index: i32,
    periods: i32,
    present_value: f64,
    timing: PaymentTiming,
) -> Result<f64, CumulativeFinanceEvalError> {
    validate_finite(&[rate, present_value])?;
    if period_index < 1 || period_index > periods || periods < 1 {
        return Err(CumulativeFinanceEvalError::Domain(WorksheetErrorCode::Num));
    }
    if rate <= 0.0 || present_value <= 0.0 {
        return Err(CumulativeFinanceEvalError::Domain(WorksheetErrorCode::Num));
    }
    if rate.abs() < EPSILON {
        return Ok(0.0);
    }
    let payment = pmt(rate, periods as f64, present_value, 0.0, timing)?;
    let result = match timing {
        PaymentTiming::EndOfPeriod => {
            fv(
                rate,
                period_index as f64 - 1.0,
                payment,
                present_value,
                timing,
            )? * rate
        }
        PaymentTiming::BeginningOfPeriod => {
            if period_index == 1 {
                0.0
            } else {
                fv(
                    rate,
                    period_index as f64 - 2.0,
                    payment,
                    present_value,
                    timing,
                )? * rate
            }
        }
    };
    if result.is_finite() {
        Ok(result)
    } else {
        Err(CumulativeFinanceEvalError::Domain(WorksheetErrorCode::Num))
    }
}

fn ppmt(
    rate: f64,
    period_index: i32,
    periods: i32,
    present_value: f64,
    timing: PaymentTiming,
) -> Result<f64, CumulativeFinanceEvalError> {
    let payment = pmt(rate, periods as f64, present_value, 0.0, timing)?;
    let interest = ipmt(rate, period_index, periods, present_value, timing)?;
    Ok(payment - interest)
}

fn validate_cumulative_inputs(
    rate: f64,
    periods: i32,
    pv: f64,
    start_period: i32,
    end_period: i32,
    timing: PaymentTiming,
) -> Result<(), CumulativeFinanceEvalError> {
    let _ = timing;
    validate_finite(&[rate, pv])?;
    if rate <= 0.0 || periods < 1 || pv <= 0.0 {
        return Err(CumulativeFinanceEvalError::Domain(WorksheetErrorCode::Num));
    }
    if start_period < 1 || end_period < 1 || start_period > end_period || end_period > periods {
        return Err(CumulativeFinanceEvalError::Domain(WorksheetErrorCode::Num));
    }
    Ok(())
}

pub fn cumipmt_kernel(
    rate: f64,
    periods: f64,
    pv: f64,
    start_period: f64,
    end_period: f64,
    type_number: f64,
) -> Result<f64, CumulativeFinanceEvalError> {
    let periods = trunc_positive_period(periods)?;
    let start_period = trunc_positive_period(start_period)?;
    let end_period = trunc_positive_period(end_period)?;
    let timing = payment_timing_arg(type_number)?;
    validate_cumulative_inputs(rate, periods, pv, start_period, end_period, timing)?;

    let mut total = 0.0;
    for period in start_period..=end_period {
        total += ipmt(rate, period, periods, pv, timing)?;
    }
    if total.is_finite() {
        Ok(total)
    } else {
        Err(CumulativeFinanceEvalError::Domain(WorksheetErrorCode::Num))
    }
}

pub fn cumprinc_kernel(
    rate: f64,
    periods: f64,
    pv: f64,
    start_period: f64,
    end_period: f64,
    type_number: f64,
) -> Result<f64, CumulativeFinanceEvalError> {
    let periods = trunc_positive_period(periods)?;
    let start_period = trunc_positive_period(start_period)?;
    let end_period = trunc_positive_period(end_period)?;
    let timing = payment_timing_arg(type_number)?;
    validate_cumulative_inputs(rate, periods, pv, start_period, end_period, timing)?;

    let mut total = 0.0;
    for period in start_period..=end_period {
        total += ppmt(rate, period, periods, pv, timing)?;
    }
    if total.is_finite() {
        Ok(total)
    } else {
        Err(CumulativeFinanceEvalError::Domain(WorksheetErrorCode::Num))
    }
}

fn eval_numeric(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
    meta: &FunctionMeta,
    kernel: impl FnOnce(&[PreparedArgValue]) -> Result<f64, CumulativeFinanceEvalError>,
) -> Result<EvalValue, CumulativeFinanceEvalError> {
    if !meta.arity.accepts(args.len()) {
        return Err(arity_error(meta, args.len()));
    }
    run_values_only_prepared(
        args,
        resolver,
        |prepared| kernel(prepared).map(EvalValue::Number),
        CumulativeFinanceEvalError::Coercion,
    )
}

pub fn eval_cumipmt_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, CumulativeFinanceEvalError> {
    eval_numeric(args, resolver, &CUMIPMT_META, |prepared| {
        cumipmt_kernel(
            number_arg(prepared, 0)?,
            number_arg(prepared, 1)?,
            number_arg(prepared, 2)?,
            number_arg(prepared, 3)?,
            number_arg(prepared, 4)?,
            number_arg(prepared, 5)?,
        )
    })
}

pub fn eval_cumprinc_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, CumulativeFinanceEvalError> {
    eval_numeric(args, resolver, &CUMPRINC_META, |prepared| {
        cumprinc_kernel(
            number_arg(prepared, 0)?,
            number_arg(prepared, 1)?,
            number_arg(prepared, 2)?,
            number_arg(prepared, 3)?,
            number_arg(prepared, 4)?,
            number_arg(prepared, 5)?,
        )
    })
}

pub fn map_cumulative_finance_error_to_ws(
    error: &CumulativeFinanceEvalError,
) -> WorksheetErrorCode {
    match error {
        CumulativeFinanceEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        CumulativeFinanceEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        CumulativeFinanceEvalError::Coercion(_) => WorksheetErrorCode::Value,
        CumulativeFinanceEvalError::Domain(code) => *code,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_close(actual: f64, expected: f64, tolerance: f64) {
        assert!(
            (actual - expected).abs() <= tolerance,
            "expected {expected}, got {actual}"
        );
    }

    #[test]
    fn meta_ids_match_expected_function_ids() {
        assert_eq!(CUMIPMT_META.function_id, "FUNC.CUMIPMT");
        assert_eq!(CUMPRINC_META.function_id, "FUNC.CUMPRINC");
    }

    #[test]
    fn cumipmt_matches_microsoft_seed_rows() {
        let rate = 0.09 / 12.0;
        let periods = 30.0 * 12.0;
        let pv = 125000.0;
        assert_close(
            cumipmt_kernel(rate, periods, pv, 13.0, 24.0, 0.0).expect("cumipmt second year"),
            -11135.23213,
            1.0e-5,
        );
        assert_close(
            cumipmt_kernel(rate, periods, pv, 1.0, 1.0, 0.0).expect("cumipmt first month"),
            -937.5,
            1.0e-9,
        );
    }

    #[test]
    fn cumprinc_matches_microsoft_seed_rows() {
        let rate = 0.09 / 12.0;
        let periods = 30.0 * 12.0;
        let pv = 125000.0;
        assert_close(
            cumprinc_kernel(rate, periods, pv, 13.0, 24.0, 0.0).expect("cumprinc second year"),
            -934.1071234,
            1.0e-6,
        );
        assert_close(
            cumprinc_kernel(rate, periods, pv, 1.0, 1.0, 0.0).expect("cumprinc first month"),
            -68.27827118,
            1.0e-8,
        );
    }

    #[test]
    fn cumulative_interest_and_principal_partition_total_payment() {
        let rate = 0.09 / 12.0;
        let periods = 30.0 * 12.0;
        let pv = 125000.0;
        let payment = pmt(rate, periods, pv, 0.0, PaymentTiming::EndOfPeriod)
            .expect("payment should succeed");
        let interest =
            cumipmt_kernel(rate, periods, pv, 13.0, 24.0, 0.0).expect("cumipmt should succeed");
        let principal =
            cumprinc_kernel(rate, periods, pv, 13.0, 24.0, 0.0).expect("cumprinc should succeed");
        assert_close(interest + principal, payment * 12.0, 1.0e-8);
    }

    #[test]
    fn type_one_has_zero_interest_in_first_period() {
        let rate = 0.09 / 12.0;
        let periods = 30.0 * 12.0;
        let pv = 125000.0;
        assert_close(
            cumipmt_kernel(rate, periods, pv, 1.0, 1.0, 1.0).expect("type one cumipmt"),
            0.0,
            1.0e-12,
        );
    }

    #[test]
    fn invalid_type_is_num_error() {
        assert_eq!(
            cumipmt_kernel(0.09 / 12.0, 360.0, 125000.0, 1.0, 1.0, 2.0),
            Err(CumulativeFinanceEvalError::Domain(WorksheetErrorCode::Num))
        );
    }

    #[test]
    fn period_arguments_truncate_toward_zero() {
        let exact = cumipmt_kernel(0.09 / 12.0, 360.8, 125000.0, 13.9, 24.9, 0.0)
            .expect("truncated periods should succeed");
        let seeded = cumipmt_kernel(0.09 / 12.0, 360.0, 125000.0, 13.0, 24.0, 0.0)
            .expect("seeded periods should succeed");
        assert_close(exact, seeded, 1.0e-8);
    }
}

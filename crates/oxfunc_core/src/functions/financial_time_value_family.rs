use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    PreparedArgValue, coerce_prepared_to_number, prepare_arg_values_only,
};
use crate::functions::power_fn::power_kernel;
use crate::resolver::{ReferenceResolver, resolve_eval_value};
use crate::value::{ArrayCellValue, CallArgValue, EvalValue, WorksheetErrorCode};
use std::fmt;

const EPSILON: f64 = 1e-12;
const RATE_TOLERANCE: f64 = 1e-7;
const RATE_MAX_ITERATIONS: usize = 20;
const RATE_BRACKET_MAX_ITERATIONS: usize = 64;
const MIN_VALID_RATE: f64 = -0.999_999_999;

const FINANCIAL_META_BASE: FunctionMeta = FunctionMeta {
    function_id: "FUNC.FINANCIAL_TIME_VALUE_BASE",
    arity: Arity::exact(1),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::RefsVisibleInAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
};

pub const PV_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.PV",
    arity: Arity { min: 3, max: 5 },
    ..FINANCIAL_META_BASE
};
pub const FV_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.FV",
    arity: Arity { min: 3, max: 5 },
    ..FINANCIAL_META_BASE
};
pub const PMT_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.PMT",
    arity: Arity { min: 3, max: 5 },
    ..FINANCIAL_META_BASE
};
pub const NPER_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.NPER",
    arity: Arity { min: 3, max: 5 },
    ..FINANCIAL_META_BASE
};
pub const NPV_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.NPV",
    arity: Arity {
        min: 2,
        max: usize::MAX,
    },
    ..FINANCIAL_META_BASE
};
pub const RATE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.RATE",
    arity: Arity { min: 3, max: 6 },
    ..FINANCIAL_META_BASE
};
pub const IPMT_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.IPMT",
    arity: Arity { min: 4, max: 6 },
    ..FINANCIAL_META_BASE
};
pub const PPMT_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.PPMT",
    arity: Arity { min: 4, max: 6 },
    ..FINANCIAL_META_BASE
};
pub const ISPMT_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.ISPMT",
    arity: Arity::exact(4),
    ..FINANCIAL_META_BASE
};
pub const MIRR_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.MIRR",
    arity: Arity::exact(3),
    ..FINANCIAL_META_BASE
};
pub const FVSCHEDULE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.FVSCHEDULE",
    arity: Arity::exact(2),
    ..FINANCIAL_META_BASE
};
pub const PDURATION_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.PDURATION",
    arity: Arity::exact(3),
    ..FINANCIAL_META_BASE
};
pub const RRI_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.RRI",
    arity: Arity::exact(3),
    ..FINANCIAL_META_BASE
};
pub const NOMINAL_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.NOMINAL",
    arity: Arity::exact(2),
    ..FINANCIAL_META_BASE
};
pub const EFFECT_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.EFFECT",
    arity: Arity::exact(2),
    ..FINANCIAL_META_BASE
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FinancialError {
    Num,
    Div0,
    Value,
    NoConvergence,
}

impl fmt::Display for FinancialError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for FinancialError {}

fn validate_finite(values: &[f64]) -> Result<(), FinancialError> {
    if values.iter().all(|value| value.is_finite()) {
        Ok(())
    } else {
        Err(FinancialError::Value)
    }
}

fn growth(periodic_rate: f64, periods: f64) -> Result<f64, FinancialError> {
    validate_finite(&[periodic_rate, periods])?;
    let base = 1.0 + periodic_rate;
    if base <= 0.0 {
        return Err(FinancialError::Num);
    }
    let factor = match power_kernel(base, periods) {
        Ok(factor) => factor,
        Err(WorksheetErrorCode::Div0) => return Err(FinancialError::Div0),
        Err(WorksheetErrorCode::Num) => return Err(FinancialError::Num),
        Err(_) => return Err(FinancialError::Value),
    };
    if factor.is_finite() {
        Ok(factor)
    } else {
        Err(FinancialError::Num)
    }
}

fn annuity_term(
    periodic_rate: f64,
    periods: f64,
    timing: PaymentTiming,
) -> Result<f64, FinancialError> {
    if periodic_rate.abs() < EPSILON {
        return Ok(periods);
    }
    let factor = growth(periodic_rate, periods)?;
    let term = timing.factor(periodic_rate) * (factor - 1.0) / periodic_rate;
    if term.is_finite() {
        Ok(term)
    } else {
        Err(FinancialError::Num)
    }
}

fn balance_equation(
    periodic_rate: f64,
    periods: f64,
    payment_value: f64,
    present_value: f64,
    future_value: f64,
    timing: PaymentTiming,
) -> Result<f64, FinancialError> {
    validate_finite(&[
        periodic_rate,
        periods,
        payment_value,
        present_value,
        future_value,
    ])?;
    if periodic_rate.abs() < EPSILON {
        return Ok(future_value + present_value + payment_value * periods);
    }
    let factor = growth(periodic_rate, periods)?;
    let term = annuity_term(periodic_rate, periods, timing)?;
    let value = future_value + present_value * factor + payment_value * term;
    if value.is_finite() {
        Ok(value)
    } else {
        Err(FinancialError::Num)
    }
}

pub fn pv(
    periodic_rate: f64,
    periods: f64,
    payment_value: f64,
    future_value: f64,
    timing: PaymentTiming,
) -> Result<f64, FinancialError> {
    validate_finite(&[periodic_rate, periods, payment_value, future_value])?;
    if periodic_rate.abs() < EPSILON {
        return Ok(-(future_value + payment_value * periods));
    }
    let factor = growth(periodic_rate, periods)?;
    let term = annuity_term(periodic_rate, periods, timing)?;
    let result = -(future_value + payment_value * term) / factor;
    if result.is_finite() {
        Ok(result)
    } else {
        Err(FinancialError::Num)
    }
}

pub fn fv(
    periodic_rate: f64,
    periods: f64,
    payment_value: f64,
    present_value: f64,
    timing: PaymentTiming,
) -> Result<f64, FinancialError> {
    validate_finite(&[periodic_rate, periods, payment_value, present_value])?;
    let factor = growth(periodic_rate, periods)?;
    let term = annuity_term(periodic_rate, periods, timing)?;
    let result = -(present_value * factor + payment_value * term);
    if result.is_finite() {
        Ok(result)
    } else {
        Err(FinancialError::Num)
    }
}

pub fn pmt(
    periodic_rate: f64,
    periods: f64,
    present_value: f64,
    future_value: f64,
    timing: PaymentTiming,
) -> Result<f64, FinancialError> {
    validate_finite(&[periodic_rate, periods, present_value, future_value])?;
    if periods.abs() < EPSILON {
        return Err(FinancialError::Num);
    }
    if periodic_rate.abs() < EPSILON {
        return Ok(-(future_value + present_value) / periods);
    }
    let factor = growth(periodic_rate, periods)?;
    let term = annuity_term(periodic_rate, periods, timing)?;
    if term.abs() < EPSILON {
        return Err(FinancialError::Num);
    }
    let result = -(future_value + present_value * factor) / term;
    if result.is_finite() {
        Ok(result)
    } else {
        Err(FinancialError::Num)
    }
}

pub fn nper(
    periodic_rate: f64,
    payment_value: f64,
    present_value: f64,
    future_value: f64,
    timing: PaymentTiming,
) -> Result<f64, FinancialError> {
    validate_finite(&[periodic_rate, payment_value, present_value, future_value])?;
    if periodic_rate.abs() < EPSILON {
        if payment_value.abs() < EPSILON {
            return Err(FinancialError::Num);
        }
        let result = -(future_value + present_value) / payment_value;
        return if result.is_finite() {
            Ok(result)
        } else {
            Err(FinancialError::Num)
        };
    }
    let adjust = payment_value * timing.factor(periodic_rate) / periodic_rate;
    let numerator = adjust - future_value;
    let denominator = present_value + adjust;
    let ratio = numerator / denominator;
    if ratio <= 0.0 {
        return Err(FinancialError::Num);
    }
    let result = ratio.ln() / (1.0 + periodic_rate).ln();
    if result.is_finite() {
        Ok(result)
    } else {
        Err(FinancialError::Num)
    }
}

fn values_bracket_root(lower_value: f64, upper_value: f64) -> bool {
    (lower_value < 0.0 && upper_value > 0.0) || (lower_value > 0.0 && upper_value < 0.0)
}

fn bisect_rate_bracket(
    mut lower: f64,
    mut lower_value: f64,
    mut upper: f64,
    upper_value: f64,
    periods: f64,
    payment_value: f64,
    present_value: f64,
    future_value: f64,
    timing: PaymentTiming,
) -> Result<f64, FinancialError> {
    if lower_value.abs() <= RATE_TOLERANCE {
        return Ok(lower);
    }
    if upper_value.abs() <= RATE_TOLERANCE {
        return Ok(upper);
    }
    if !values_bracket_root(lower_value, upper_value) {
        return Err(FinancialError::NoConvergence);
    }

    for _ in 0..RATE_BRACKET_MAX_ITERATIONS {
        let midpoint = (lower + upper) / 2.0;
        let midpoint_value = balance_equation(
            midpoint,
            periods,
            payment_value,
            present_value,
            future_value,
            timing,
        )?;
        if midpoint_value.abs() <= RATE_TOLERANCE {
            return Ok(midpoint);
        }

        if values_bracket_root(lower_value, midpoint_value) {
            upper = midpoint;
        } else {
            lower = midpoint;
            lower_value = midpoint_value;
        }
    }

    let midpoint = (lower + upper) / 2.0;
    let midpoint_value = balance_equation(
        midpoint,
        periods,
        payment_value,
        present_value,
        future_value,
        timing,
    )?;
    if midpoint_value.abs() <= RATE_TOLERANCE || (upper - lower).abs() <= EPSILON {
        Ok(midpoint)
    } else {
        Err(FinancialError::NoConvergence)
    }
}

fn search_rate_bracket_from_zero(
    zero_value: f64,
    initial_rate: f64,
    periods: f64,
    payment_value: f64,
    present_value: f64,
    future_value: f64,
    timing: PaymentTiming,
) -> Result<Option<f64>, FinancialError> {
    let mut positive_rate = initial_rate.abs().max(0.1);
    for _ in 0..RATE_BRACKET_MAX_ITERATIONS {
        let positive_value = balance_equation(
            positive_rate,
            periods,
            payment_value,
            present_value,
            future_value,
            timing,
        )?;
        if positive_value.abs() <= RATE_TOLERANCE {
            return Ok(Some(positive_rate));
        }
        if values_bracket_root(zero_value, positive_value) {
            return bisect_rate_bracket(
                0.0,
                zero_value,
                positive_rate,
                positive_value,
                periods,
                payment_value,
                present_value,
                future_value,
                timing,
            )
            .map(Some);
        }

        positive_rate *= 2.0;
        if !positive_rate.is_finite() {
            break;
        }
    }

    let mut negative_rate = if initial_rate < 0.0 {
        initial_rate
    } else {
        -0.1
    };
    if negative_rate <= MIN_VALID_RATE {
        negative_rate = -0.1;
    }
    for _ in 0..RATE_BRACKET_MAX_ITERATIONS {
        let negative_value = balance_equation(
            negative_rate,
            periods,
            payment_value,
            present_value,
            future_value,
            timing,
        )?;
        if negative_value.abs() <= RATE_TOLERANCE {
            return Ok(Some(negative_rate));
        }
        if values_bracket_root(negative_value, zero_value) {
            return bisect_rate_bracket(
                negative_rate,
                negative_value,
                0.0,
                zero_value,
                periods,
                payment_value,
                present_value,
                future_value,
                timing,
            )
            .map(Some);
        }

        let margin = 1.0 + negative_rate;
        if margin <= EPSILON {
            break;
        }
        negative_rate = -1.0 + margin / 2.0;
        if negative_rate <= MIN_VALID_RATE {
            negative_rate = MIN_VALID_RATE + EPSILON;
        }
    }

    Ok(None)
}

pub fn rate(
    periods: f64,
    payment_value: f64,
    present_value: f64,
    future_value: f64,
    timing: PaymentTiming,
    guess: Option<f64>,
) -> Result<f64, FinancialError> {
    validate_finite(&[periods, payment_value, present_value, future_value])?;
    let mut prev_rate = 0.0;
    let mut prev_value = balance_equation(
        prev_rate,
        periods,
        payment_value,
        present_value,
        future_value,
        timing,
    )?;
    let mut current_rate = guess.unwrap_or(0.1);
    if current_rate <= MIN_VALID_RATE {
        current_rate = MIN_VALID_RATE;
    }
    let mut current_value = balance_equation(
        current_rate,
        periods,
        payment_value,
        present_value,
        future_value,
        timing,
    )?;

    if current_value.abs() <= RATE_TOLERANCE {
        return Ok(current_rate);
    }
    if values_bracket_root(prev_value, current_value) {
        return bisect_rate_bracket(
            prev_rate,
            prev_value,
            current_rate,
            current_value,
            periods,
            payment_value,
            present_value,
            future_value,
            timing,
        );
    }

    for _ in 0..RATE_MAX_ITERATIONS {
        if current_value.abs() <= RATE_TOLERANCE {
            return Ok(current_rate);
        }

        let denominator = current_value - prev_value;
        let next_rate = if denominator.abs() > EPSILON {
            current_rate - current_value * (current_rate - prev_rate) / denominator
        } else {
            let h = current_rate.abs().max(1.0) * 1e-6;
            let lower = (current_rate - h).max(-0.999_999_999);
            let upper = current_rate + h;
            let lower_value = balance_equation(
                lower,
                periods,
                payment_value,
                present_value,
                future_value,
                timing,
            )?;
            let upper_value = balance_equation(
                upper,
                periods,
                payment_value,
                present_value,
                future_value,
                timing,
            )?;
            let derivative = (upper_value - lower_value) / (upper - lower);
            if derivative.abs() <= EPSILON {
                return Err(FinancialError::NoConvergence);
            }
            current_rate - current_value / derivative
        };

        if !next_rate.is_finite() || next_rate <= MIN_VALID_RATE {
            break;
        }

        prev_rate = current_rate;
        prev_value = current_value;
        current_rate = next_rate;
        current_value = balance_equation(
            current_rate,
            periods,
            payment_value,
            present_value,
            future_value,
            timing,
        )?;

        if values_bracket_root(prev_value, current_value) {
            return bisect_rate_bracket(
                prev_rate,
                prev_value,
                current_rate,
                current_value,
                periods,
                payment_value,
                present_value,
                future_value,
                timing,
            );
        }
    }

    if current_value.abs() <= RATE_TOLERANCE {
        Ok(current_rate)
    } else if let Some(root) = search_rate_bracket_from_zero(
        balance_equation(
            0.0,
            periods,
            payment_value,
            present_value,
            future_value,
            timing,
        )?,
        guess.unwrap_or(0.1),
        periods,
        payment_value,
        present_value,
        future_value,
        timing,
    )? {
        Ok(root)
    } else {
        Err(FinancialError::NoConvergence)
    }
}

pub fn npv(periodic_rate: f64, cashflows: &[f64]) -> Result<f64, FinancialError> {
    validate_finite(&[periodic_rate])?;
    validate_finite(cashflows)?;
    if cashflows.is_empty() {
        return Err(FinancialError::Value);
    }
    let base = 1.0 + periodic_rate;
    if base <= 0.0 {
        return Err(FinancialError::Num);
    }
    let mut total = 0.0;
    let mut discount = 1.0;
    for cashflow in cashflows {
        discount *= base;
        total += *cashflow / discount;
    }
    if total.is_finite() {
        Ok(total)
    } else {
        Err(FinancialError::Num)
    }
}

pub fn ipmt(
    periodic_rate: f64,
    period_index: f64,
    periods: f64,
    present_value: f64,
    future_value: f64,
    timing: PaymentTiming,
) -> Result<f64, FinancialError> {
    validate_finite(&[
        periodic_rate,
        period_index,
        periods,
        present_value,
        future_value,
    ])?;
    if period_index < 1.0 || period_index > periods {
        return Err(FinancialError::Num);
    }
    if periodic_rate.abs() < EPSILON {
        return Ok(0.0);
    }
    let periodic_payment = pmt(periodic_rate, periods, present_value, future_value, timing)?;
    let result = match timing {
        PaymentTiming::EndOfPeriod => {
            fv(
                periodic_rate,
                period_index - 1.0,
                periodic_payment,
                present_value,
                timing,
            )? * periodic_rate
        }
        PaymentTiming::BeginningOfPeriod => {
            if period_index <= 1.0 {
                0.0
            } else {
                fv(
                    periodic_rate,
                    period_index - 2.0,
                    periodic_payment,
                    present_value,
                    timing,
                )? * periodic_rate
            }
        }
    };
    if result.is_finite() {
        Ok(result)
    } else {
        Err(FinancialError::Num)
    }
}

pub fn ppmt(
    periodic_rate: f64,
    period_index: f64,
    periods: f64,
    present_value: f64,
    future_value: f64,
    timing: PaymentTiming,
) -> Result<f64, FinancialError> {
    let payment = pmt(periodic_rate, periods, present_value, future_value, timing)?;
    let interest = ipmt(
        periodic_rate,
        period_index,
        periods,
        present_value,
        future_value,
        timing,
    )?;
    Ok(payment - interest)
}

pub fn ispmt(
    periodic_rate: f64,
    period_index: f64,
    periods: f64,
    present_value: f64,
) -> Result<f64, FinancialError> {
    validate_finite(&[periodic_rate, period_index, periods, present_value])?;
    if periods <= 0.0 {
        return Err(FinancialError::Num);
    }
    let result = present_value * periodic_rate * (period_index / periods - 1.0);
    if result.is_finite() {
        Ok(result)
    } else {
        Err(FinancialError::Num)
    }
}

pub fn mirr(
    cashflows: &[f64],
    finance_rate: f64,
    reinvest_rate: f64,
) -> Result<f64, FinancialError> {
    validate_finite(&[finance_rate, reinvest_rate])?;
    validate_finite(cashflows)?;
    if cashflows.len() < 2 {
        return Err(FinancialError::Div0);
    }
    let has_positive = cashflows.iter().any(|value| *value > 0.0);
    let has_negative = cashflows.iter().any(|value| *value < 0.0);
    if !has_positive || !has_negative {
        return Err(FinancialError::Div0);
    }
    if (1.0 + finance_rate).abs() < EPSILON || (1.0 + reinvest_rate).abs() < EPSILON {
        return Err(FinancialError::Div0);
    }
    let periods = (cashflows.len() - 1) as f64;
    let mut finance_pv = 0.0;
    let mut reinvest_fv = 0.0;
    for (idx, cashflow) in cashflows.iter().enumerate() {
        let idxf = idx as f64;
        if *cashflow < 0.0 {
            finance_pv += *cashflow / (1.0 + finance_rate).powf(idxf);
        } else if *cashflow > 0.0 {
            reinvest_fv += *cashflow * (1.0 + reinvest_rate).powf(periods - idxf);
        }
    }
    if finance_pv >= 0.0 || reinvest_fv <= 0.0 {
        return Err(FinancialError::Div0);
    }
    let result = (-reinvest_fv / finance_pv).powf(1.0 / periods) - 1.0;
    if result.is_finite() {
        Ok(result)
    } else {
        Err(FinancialError::Num)
    }
}

pub fn fvschedule(principal: f64, schedule: &[f64]) -> Result<f64, FinancialError> {
    validate_finite(&[principal])?;
    validate_finite(schedule)?;
    let mut result = principal;
    for rate in schedule {
        result *= 1.0 + rate;
    }
    if result.is_finite() {
        Ok(result)
    } else {
        Err(FinancialError::Num)
    }
}

pub fn pduration(
    periodic_rate: f64,
    present_value: f64,
    future_value: f64,
) -> Result<f64, FinancialError> {
    validate_finite(&[periodic_rate, present_value, future_value])?;
    if periodic_rate <= 0.0 || present_value <= 0.0 || future_value <= 0.0 {
        return Err(FinancialError::Num);
    }
    let result = (future_value / present_value).ln() / (1.0 + periodic_rate).ln();
    if result.is_finite() {
        Ok(result)
    } else {
        Err(FinancialError::Num)
    }
}

pub fn rri(periods: f64, present_value: f64, future_value: f64) -> Result<f64, FinancialError> {
    validate_finite(&[periods, present_value, future_value])?;
    if periods <= 0.0 || present_value <= 0.0 || future_value <= 0.0 {
        return Err(FinancialError::Num);
    }
    let result = (future_value / present_value).powf(1.0 / periods) - 1.0;
    if result.is_finite() {
        Ok(result)
    } else {
        Err(FinancialError::Num)
    }
}

pub fn nominal(effect_rate: f64, periods_per_year: f64) -> Result<f64, FinancialError> {
    validate_finite(&[effect_rate, periods_per_year])?;
    let periods = periods_per_year.trunc();
    if effect_rate <= 0.0 || periods < 1.0 {
        return Err(FinancialError::Num);
    }
    let result = periods * ((1.0 + effect_rate).powf(1.0 / periods) - 1.0);
    if result.is_finite() {
        Ok(result)
    } else {
        Err(FinancialError::Num)
    }
}

pub fn effect(nominal_rate: f64, periods_per_year: f64) -> Result<f64, FinancialError> {
    validate_finite(&[nominal_rate, periods_per_year])?;
    let periods = periods_per_year.trunc();
    if nominal_rate <= 0.0 || periods < 1.0 {
        return Err(FinancialError::Num);
    }
    let result = (1.0 + nominal_rate / periods).powf(periods) - 1.0;
    if result.is_finite() {
        Ok(result)
    } else {
        Err(FinancialError::Num)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FinancialSurfaceEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
    Kernel(FinancialError),
}

fn scalar_number(
    arg: &CallArgValue,
    resolver: &impl ReferenceResolver,
) -> Result<f64, FinancialSurfaceEvalError> {
    let prepared =
        prepare_arg_values_only(arg, resolver).map_err(FinancialSurfaceEvalError::Coercion)?;
    match prepared {
        PreparedArgValue::MissingArg | PreparedArgValue::EmptyCell => Ok(0.0),
        other => coerce_prepared_to_number(&other).map_err(FinancialSurfaceEvalError::Coercion),
    }
}

fn payment_timing_arg(
    arg: Option<&CallArgValue>,
    resolver: &impl ReferenceResolver,
) -> Result<PaymentTiming, FinancialSurfaceEvalError> {
    let value = match arg {
        None => 0.0,
        Some(inner) => scalar_number(inner, resolver)?,
    };
    Ok(if value.trunc() == 0.0 {
        PaymentTiming::EndOfPeriod
    } else {
        PaymentTiming::BeginningOfPeriod
    })
}

fn resolve_eval(
    arg: &CallArgValue,
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, FinancialSurfaceEvalError> {
    match arg {
        CallArgValue::Reference(reference)
        | CallArgValue::Eval(EvalValue::Reference(reference)) => {
            let resolved = resolve_eval_value(resolver, reference).map_err(|e| {
                FinancialSurfaceEvalError::Coercion(CoercionError::RefResolution(e))
            })?;
            resolve_eval(&CallArgValue::Eval(resolved), resolver)
        }
        CallArgValue::Eval(value) => Ok(value.clone()),
        CallArgValue::MissingArg | CallArgValue::EmptyCell => Ok(EvalValue::Number(0.0)),
    }
}

fn eval_to_numeric_sequence(
    value: EvalValue,
    out: &mut Vec<f64>,
) -> Result<(), FinancialSurfaceEvalError> {
    match value {
        EvalValue::Array(array) => {
            for cell in array.iter_row_major() {
                match cell {
                    ArrayCellValue::Number(n) => out.push(*n),
                    ArrayCellValue::Text(t) => {
                        let prepared = PreparedArgValue::Eval(EvalValue::Text(t.clone()));
                        out.push(
                            coerce_prepared_to_number(&prepared)
                                .map_err(FinancialSurfaceEvalError::Coercion)?,
                        );
                    }
                    ArrayCellValue::Logical(b) => out.push(if *b { 1.0 } else { 0.0 }),
                    ArrayCellValue::Error(code) => {
                        return Err(FinancialSurfaceEvalError::Coercion(
                            CoercionError::WorksheetError(*code),
                        ));
                    }
                    ArrayCellValue::EmptyCell => {}
                }
            }
            Ok(())
        }
        EvalValue::Number(n) => {
            out.push(n);
            Ok(())
        }
        EvalValue::Text(t) => {
            let prepared = PreparedArgValue::Eval(EvalValue::Text(t));
            out.push(
                coerce_prepared_to_number(&prepared)
                    .map_err(FinancialSurfaceEvalError::Coercion)?,
            );
            Ok(())
        }
        EvalValue::Logical(b) => {
            out.push(if b { 1.0 } else { 0.0 });
            Ok(())
        }
        EvalValue::Error(code) => Err(FinancialSurfaceEvalError::Coercion(
            CoercionError::WorksheetError(code),
        )),
        EvalValue::Reference(_) | EvalValue::Lambda(_) => Err(FinancialSurfaceEvalError::Coercion(
            CoercionError::UnsupportedValueKind("reference_like"),
        )),
    }
}

fn numeric_sequence_from_args(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<Vec<f64>, FinancialSurfaceEvalError> {
    let mut out = Vec::new();
    for arg in args {
        let value = resolve_eval(arg, resolver)?;
        eval_to_numeric_sequence(value, &mut out)?;
    }
    Ok(out)
}

fn numeric_result(
    value: Result<f64, FinancialError>,
) -> Result<EvalValue, FinancialSurfaceEvalError> {
    value
        .map(EvalValue::Number)
        .map_err(FinancialSurfaceEvalError::Kernel)
}

pub fn eval_pv_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, FinancialSurfaceEvalError> {
    if !PV_META.arity.accepts(args.len()) {
        return Err(FinancialSurfaceEvalError::ArityMismatch {
            expected_min: PV_META.arity.min,
            expected_max: PV_META.arity.max,
            actual: args.len(),
        });
    }
    numeric_result(pv(
        scalar_number(&args[0], resolver)?,
        scalar_number(&args[1], resolver)?,
        scalar_number(&args[2], resolver)?,
        args.get(3)
            .map(|arg| scalar_number(arg, resolver))
            .transpose()?
            .unwrap_or(0.0),
        payment_timing_arg(args.get(4), resolver)?,
    ))
}

pub fn eval_fv_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, FinancialSurfaceEvalError> {
    if !FV_META.arity.accepts(args.len()) {
        return Err(FinancialSurfaceEvalError::ArityMismatch {
            expected_min: FV_META.arity.min,
            expected_max: FV_META.arity.max,
            actual: args.len(),
        });
    }
    numeric_result(fv(
        scalar_number(&args[0], resolver)?,
        scalar_number(&args[1], resolver)?,
        scalar_number(&args[2], resolver)?,
        args.get(3)
            .map(|arg| scalar_number(arg, resolver))
            .transpose()?
            .unwrap_or(0.0),
        payment_timing_arg(args.get(4), resolver)?,
    ))
}

pub fn eval_pmt_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, FinancialSurfaceEvalError> {
    if !PMT_META.arity.accepts(args.len()) {
        return Err(FinancialSurfaceEvalError::ArityMismatch {
            expected_min: PMT_META.arity.min,
            expected_max: PMT_META.arity.max,
            actual: args.len(),
        });
    }
    numeric_result(pmt(
        scalar_number(&args[0], resolver)?,
        scalar_number(&args[1], resolver)?,
        scalar_number(&args[2], resolver)?,
        args.get(3)
            .map(|arg| scalar_number(arg, resolver))
            .transpose()?
            .unwrap_or(0.0),
        payment_timing_arg(args.get(4), resolver)?,
    ))
}

pub fn eval_nper_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, FinancialSurfaceEvalError> {
    if !NPER_META.arity.accepts(args.len()) {
        return Err(FinancialSurfaceEvalError::ArityMismatch {
            expected_min: NPER_META.arity.min,
            expected_max: NPER_META.arity.max,
            actual: args.len(),
        });
    }
    numeric_result(nper(
        scalar_number(&args[0], resolver)?,
        scalar_number(&args[1], resolver)?,
        scalar_number(&args[2], resolver)?,
        args.get(3)
            .map(|arg| scalar_number(arg, resolver))
            .transpose()?
            .unwrap_or(0.0),
        payment_timing_arg(args.get(4), resolver)?,
    ))
}

pub fn eval_rate_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, FinancialSurfaceEvalError> {
    if !RATE_META.arity.accepts(args.len()) {
        return Err(FinancialSurfaceEvalError::ArityMismatch {
            expected_min: RATE_META.arity.min,
            expected_max: RATE_META.arity.max,
            actual: args.len(),
        });
    }
    numeric_result(rate(
        scalar_number(&args[0], resolver)?,
        scalar_number(&args[1], resolver)?,
        scalar_number(&args[2], resolver)?,
        args.get(3)
            .map(|arg| scalar_number(arg, resolver))
            .transpose()?
            .unwrap_or(0.0),
        payment_timing_arg(args.get(4), resolver)?,
        args.get(5)
            .map(|arg| scalar_number(arg, resolver))
            .transpose()?,
    ))
}

pub fn eval_npv_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, FinancialSurfaceEvalError> {
    if !NPV_META.arity.accepts(args.len()) {
        return Err(FinancialSurfaceEvalError::ArityMismatch {
            expected_min: NPV_META.arity.min,
            expected_max: NPV_META.arity.max,
            actual: args.len(),
        });
    }
    let rate = scalar_number(&args[0], resolver)?;
    let values = numeric_sequence_from_args(&args[1..], resolver)?;
    numeric_result(npv(rate, &values))
}

pub fn eval_ipmt_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, FinancialSurfaceEvalError> {
    if !IPMT_META.arity.accepts(args.len()) {
        return Err(FinancialSurfaceEvalError::ArityMismatch {
            expected_min: IPMT_META.arity.min,
            expected_max: IPMT_META.arity.max,
            actual: args.len(),
        });
    }
    numeric_result(ipmt(
        scalar_number(&args[0], resolver)?,
        scalar_number(&args[1], resolver)?,
        scalar_number(&args[2], resolver)?,
        scalar_number(&args[3], resolver)?,
        args.get(4)
            .map(|arg| scalar_number(arg, resolver))
            .transpose()?
            .unwrap_or(0.0),
        payment_timing_arg(args.get(5), resolver)?,
    ))
}

pub fn eval_ppmt_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, FinancialSurfaceEvalError> {
    if !PPMT_META.arity.accepts(args.len()) {
        return Err(FinancialSurfaceEvalError::ArityMismatch {
            expected_min: PPMT_META.arity.min,
            expected_max: PPMT_META.arity.max,
            actual: args.len(),
        });
    }
    numeric_result(ppmt(
        scalar_number(&args[0], resolver)?,
        scalar_number(&args[1], resolver)?,
        scalar_number(&args[2], resolver)?,
        scalar_number(&args[3], resolver)?,
        args.get(4)
            .map(|arg| scalar_number(arg, resolver))
            .transpose()?
            .unwrap_or(0.0),
        payment_timing_arg(args.get(5), resolver)?,
    ))
}

pub fn eval_ispmt_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, FinancialSurfaceEvalError> {
    if !ISPMT_META.arity.accepts(args.len()) {
        return Err(FinancialSurfaceEvalError::ArityMismatch {
            expected_min: ISPMT_META.arity.min,
            expected_max: ISPMT_META.arity.max,
            actual: args.len(),
        });
    }
    numeric_result(ispmt(
        scalar_number(&args[0], resolver)?,
        scalar_number(&args[1], resolver)?,
        scalar_number(&args[2], resolver)?,
        scalar_number(&args[3], resolver)?,
    ))
}

pub fn eval_mirr_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, FinancialSurfaceEvalError> {
    if !MIRR_META.arity.accepts(args.len()) {
        return Err(FinancialSurfaceEvalError::ArityMismatch {
            expected_min: MIRR_META.arity.min,
            expected_max: MIRR_META.arity.max,
            actual: args.len(),
        });
    }
    let values = numeric_sequence_from_args(&args[..1], resolver)?;
    numeric_result(mirr(
        &values,
        scalar_number(&args[1], resolver)?,
        scalar_number(&args[2], resolver)?,
    ))
}

pub fn eval_fvschedule_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, FinancialSurfaceEvalError> {
    if !FVSCHEDULE_META.arity.accepts(args.len()) {
        return Err(FinancialSurfaceEvalError::ArityMismatch {
            expected_min: FVSCHEDULE_META.arity.min,
            expected_max: FVSCHEDULE_META.arity.max,
            actual: args.len(),
        });
    }
    let schedule = numeric_sequence_from_args(&args[1..2], resolver)?;
    numeric_result(fvschedule(scalar_number(&args[0], resolver)?, &schedule))
}

pub fn eval_pduration_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, FinancialSurfaceEvalError> {
    if !PDURATION_META.arity.accepts(args.len()) {
        return Err(FinancialSurfaceEvalError::ArityMismatch {
            expected_min: PDURATION_META.arity.min,
            expected_max: PDURATION_META.arity.max,
            actual: args.len(),
        });
    }
    numeric_result(pduration(
        scalar_number(&args[0], resolver)?,
        scalar_number(&args[1], resolver)?,
        scalar_number(&args[2], resolver)?,
    ))
}

pub fn eval_rri_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, FinancialSurfaceEvalError> {
    if !RRI_META.arity.accepts(args.len()) {
        return Err(FinancialSurfaceEvalError::ArityMismatch {
            expected_min: RRI_META.arity.min,
            expected_max: RRI_META.arity.max,
            actual: args.len(),
        });
    }
    numeric_result(rri(
        scalar_number(&args[0], resolver)?,
        scalar_number(&args[1], resolver)?,
        scalar_number(&args[2], resolver)?,
    ))
}

pub fn eval_nominal_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, FinancialSurfaceEvalError> {
    if !NOMINAL_META.arity.accepts(args.len()) {
        return Err(FinancialSurfaceEvalError::ArityMismatch {
            expected_min: NOMINAL_META.arity.min,
            expected_max: NOMINAL_META.arity.max,
            actual: args.len(),
        });
    }
    numeric_result(nominal(
        scalar_number(&args[0], resolver)?,
        scalar_number(&args[1], resolver)?,
    ))
}

pub fn eval_effect_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, FinancialSurfaceEvalError> {
    if !EFFECT_META.arity.accepts(args.len()) {
        return Err(FinancialSurfaceEvalError::ArityMismatch {
            expected_min: EFFECT_META.arity.min,
            expected_max: EFFECT_META.arity.max,
            actual: args.len(),
        });
    }
    numeric_result(effect(
        scalar_number(&args[0], resolver)?,
        scalar_number(&args[1], resolver)?,
    ))
}

pub fn map_financial_time_value_error_to_ws(
    error: &FinancialSurfaceEvalError,
) -> WorksheetErrorCode {
    match error {
        FinancialSurfaceEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        FinancialSurfaceEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        FinancialSurfaceEvalError::Coercion(_) => WorksheetErrorCode::Value,
        FinancialSurfaceEvalError::Kernel(FinancialError::Num)
        | FinancialSurfaceEvalError::Kernel(FinancialError::NoConvergence) => {
            WorksheetErrorCode::Num
        }
        FinancialSurfaceEvalError::Kernel(FinancialError::Div0) => WorksheetErrorCode::Div0,
        FinancialSurfaceEvalError::Kernel(FinancialError::Value) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_close(actual: f64, expected: f64, tolerance: f64) {
        let delta = (actual - expected).abs();
        assert!(
            delta <= tolerance,
            "expected {expected}, got {actual}, delta {delta}, tolerance {tolerance}"
        );
    }

    fn assert_bits(actual: f64, expected: f64) {
        assert_eq!(
            actual.to_bits(),
            expected.to_bits(),
            "{actual} vs {expected}"
        );
    }

    #[test]
    fn pmt_and_npv_exactness_witness_rows_are_pinned() {
        assert_bits(
            pmt(
                0.05 / 12.0,
                360.0,
                200000.0,
                0.0,
                PaymentTiming::EndOfPeriod,
            )
            .expect("pmt witness"),
            -1073.6432460242763,
        );
        assert_bits(
            npv(0.1, &[-10000.0, 3000.0, 4200.0, 6800.0]).expect("npv witness"),
            1188.4434123352207,
        );
    }

    #[test]
    fn rate_exactness_witness_pins_current_local_value_and_excel_gap() {
        let actual = rate(
            360.0,
            -1073.64,
            200000.0,
            0.0,
            PaymentTiming::EndOfPeriod,
            None,
        )
        .expect("rate witness");
        let current_local = 0.0041666445363460975_f64;
        let excel_target = 0.004166644536345589_f64;

        assert_bits(actual, current_local);
        assert_ne!(actual.to_bits(), excel_target.to_bits());
    }

    #[test]
    fn core_annuity_functions_match_known_examples() {
        let monthly_rate = 0.08 / 12.0;
        let periods = 10.0;
        let principal = 10_000.0;
        let payment = pmt(
            monthly_rate,
            periods,
            principal,
            0.0,
            PaymentTiming::EndOfPeriod,
        )
        .expect("pmt");
        assert_close(payment, -1037.0320893591607, 1e-9);

        let future = fv(
            monthly_rate,
            periods,
            payment,
            principal,
            PaymentTiming::EndOfPeriod,
        )
        .expect("fv");
        assert_close(future, 0.0, 1e-7);

        let present = pv(
            monthly_rate,
            periods,
            payment,
            0.0,
            PaymentTiming::EndOfPeriod,
        )
        .expect("pv");
        assert_close(present, principal, 1e-7);

        let recovered_nper = nper(
            monthly_rate,
            payment,
            principal,
            0.0,
            PaymentTiming::EndOfPeriod,
        )
        .expect("nper");
        assert_close(recovered_nper, periods, 1e-9);
    }

    #[test]
    fn core_annuity_functions_match_pinned_excel_publication_rows() {
        assert_eq!(
            pv(0.05, 10.0, -100.0, 0.0, PaymentTiming::EndOfPeriod),
            Ok(772.1734929184813)
        );
        assert_eq!(
            fv(0.05, 10.0, -100.0, 0.0, PaymentTiming::EndOfPeriod),
            Ok(1257.789253554883)
        );
        assert_eq!(
            pmt(0.05, 10.0, 1000.0, 0.0, PaymentTiming::EndOfPeriod),
            Ok(-129.50457496545667)
        );
    }

    #[test]
    fn rate_recovers_known_periodic_rate_from_annuity_identity() {
        let expected_rate = 0.01;
        let payment = pmt(expected_rate, 48.0, 8000.0, 0.0, PaymentTiming::EndOfPeriod)
            .expect("pmt for rate inversion");
        let recovered = rate(
            48.0,
            payment,
            8000.0,
            0.0,
            PaymentTiming::EndOfPeriod,
            Some(0.1),
        )
        .expect("rate");
        assert_close(recovered, expected_rate, 1e-7);
    }

    #[test]
    fn rate_default_guess_converges_on_mortgage_style_lane() {
        let expected_rate = 0.004166644536345589;
        let omitted_guess = rate(
            360.0,
            -1073.64,
            200000.0,
            0.0,
            PaymentTiming::EndOfPeriod,
            None,
        )
        .expect("rate with omitted guess");
        assert_close(omitted_guess, expected_rate, 1e-12);

        let default_guess = rate(
            360.0,
            -1073.64,
            200000.0,
            0.0,
            PaymentTiming::EndOfPeriod,
            Some(0.1),
        )
        .expect("rate with default 0.1 guess");
        assert_close(default_guess, expected_rate, 1e-12);
    }

    #[test]
    fn npv_matches_microsoft_sample() {
        let got = npv(0.1, &[-10000.0, 3000.0, 4200.0, 6800.0]).expect("npv");
        assert_close(got, 1188.4434123352216, 1e-9);
    }

    #[test]
    fn ipmt_and_ppmt_partition_payment() {
        let monthly_rate = 0.10 / 12.0;
        let periods = 3.0 * 12.0;
        let present_value = 8000.0;
        let payment = pmt(
            monthly_rate,
            periods,
            present_value,
            0.0,
            PaymentTiming::EndOfPeriod,
        )
        .expect("payment");
        let interest = ipmt(
            monthly_rate,
            1.0,
            periods,
            present_value,
            0.0,
            PaymentTiming::EndOfPeriod,
        )
        .expect("ipmt");
        let principal = ppmt(
            monthly_rate,
            1.0,
            periods,
            present_value,
            0.0,
            PaymentTiming::EndOfPeriod,
        )
        .expect("ppmt");
        assert_close(interest + principal, payment, 1e-9);
        assert_close(interest, -66.66666666666667, 1e-9);
    }

    #[test]
    fn ispmt_uses_even_principal_schedule() {
        let first = ispmt(0.1, 1.0, 4.0, 1000.0).expect("ispmt first");
        assert_close(first, -75.0, 1e-9);

        let later = ispmt(0.1, 4.0, 4.0, 1000.0).expect("ispmt later");
        assert_close(later, 0.0, 1e-9);

        let pre_schedule = ispmt(0.1, 0.0, 4.0, 1000.0).expect("ispmt zero");
        assert_close(pre_schedule, -100.0, 1e-9);

        let beyond_schedule = ispmt(0.1, 5.0, 4.0, 1000.0).expect("ispmt beyond");
        assert_close(beyond_schedule, 25.0, 1e-9);
    }

    #[test]
    fn mirr_matches_manual_compounding_identity() {
        let cashflows = [-120000.0, 39000.0, 30000.0, 21000.0, 37000.0, 46000.0];
        let got = mirr(&cashflows, 0.10, 0.12).expect("mirr");
        assert_close(got, 0.1260941303659051, 1e-9);
    }

    #[test]
    fn fvschedule_matches_microsoft_sample() {
        let got = fvschedule(1.0, &[0.09, 0.11, 0.10]).expect("fvschedule");
        assert_close(got, 1.33089, 1e-12);
    }

    #[test]
    fn pduration_and_rri_match_documented_examples() {
        let pd = pduration(0.025, 2000.0, 2200.0).expect("pduration");
        assert_close(pd, 3.859866162622662, 1e-9);

        let rr = rri(96.0, 10000.0, 11000.0).expect("rri");
        assert_close(rr, 0.0009933073762913303, 1e-12);
    }

    #[test]
    fn nominal_and_effect_are_consistent_with_microsoft_example() {
        let nominal_rate = nominal(0.053543, 4.0).expect("nominal");
        assert_close(nominal_rate, 0.052500319868356016, 1e-12);

        let effect_rate = effect(nominal_rate, 4.0).expect("effect");
        assert_close(effect_rate, 0.053543, 1e-9);
    }

    #[test]
    fn numeric_domain_errors_are_reported() {
        assert_eq!(pduration(0.0, 100.0, 120.0), Err(FinancialError::Num));
        assert_eq!(nominal(0.05, 0.0), Err(FinancialError::Num));
        assert_eq!(npv(-1.0, &[1.0]), Err(FinancialError::Num));
        assert_eq!(mirr(&[1.0, 2.0], 0.1, 0.1), Err(FinancialError::Div0));
        assert_eq!(
            ipmt(0.1, 0.0, 10.0, 1000.0, 0.0, PaymentTiming::EndOfPeriod),
            Err(FinancialError::Num)
        );
    }
}

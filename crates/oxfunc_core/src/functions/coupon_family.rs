use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{coerce_prepared_to_number, run_values_only_prepared};
use crate::locale_format::{WorkbookDateSystem, excel_serial_from_ymd, ymd_from_excel_serial};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

const COUPON_BASE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.COUPON_BASE",
    arity: Arity { min: 3, max: 4 },
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

pub const COUPDAYBS_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.COUPDAYBS",
    ..COUPON_BASE_META
};

pub const COUPDAYS_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.COUPDAYS",
    ..COUPON_BASE_META
};

pub const COUPDAYSNC_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.COUPDAYSNC",
    ..COUPON_BASE_META
};

pub const COUPNCD_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.COUPNCD",
    ..COUPON_BASE_META
};

pub const COUPNUM_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.COUPNUM",
    ..COUPON_BASE_META
};

pub const COUPPCD_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.COUPPCD",
    ..COUPON_BASE_META
};

#[derive(Debug, Clone, PartialEq)]
pub enum CouponEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
    Domain(WorksheetErrorCode),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CouponBasis {
    Us30_360,
    ActualActual,
    Actual360,
    Actual365,
    European30_360,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CouponContext {
    settlement: i64,
    maturity: i64,
    frequency: i64,
    basis: CouponBasis,
    maturity_anchor_day: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CouponPeriod {
    previous: i64,
    next: i64,
    remaining_coupons: i64,
    raw_previous: i64,
}

fn max_excel_serial() -> i64 {
    excel_serial_from_ymd(WorkbookDateSystem::System1900, 9999, 12, 31).unwrap() as i64
}

fn days_in_month(year: i64, month: i64) -> i64 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
            if leap { 29 } else { 28 }
        }
        _ => 30,
    }
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

fn add_months_clamped(serial: i64, months: i64) -> Option<i64> {
    let (year, month, day) = ymd_from_excel_serial(WorkbookDateSystem::System1900, serial as f64)?;
    let month_index = year
        .checked_mul(12)?
        .checked_add(month - 1)?
        .checked_add(months)?;
    let target_year = month_index.div_euclid(12);
    let target_month = month_index.rem_euclid(12) + 1;
    let source_is_month_end = day == days_in_month(year, month);
    let target_day = if source_is_month_end {
        days_in_month(target_year, target_month)
    } else {
        day.min(days_in_month(target_year, target_month))
    };
    Some(excel_serial_from_ymd_unbounded_1900(
        target_year,
        target_month,
        target_day,
    ))
}

fn add_months_with_anchor_day(serial: i64, months: i64, anchor_day: i64) -> Option<i64> {
    let (year, month, _day) = ymd_from_excel_serial(WorkbookDateSystem::System1900, serial as f64)?;
    let month_index = year
        .checked_mul(12)?
        .checked_add(month - 1)?
        .checked_add(months)?;
    let target_year = month_index.div_euclid(12);
    let target_month = month_index.rem_euclid(12) + 1;
    let target_day = anchor_day.min(days_in_month(target_year, target_month));
    Some(excel_serial_from_ymd_unbounded_1900(
        target_year,
        target_month,
        target_day,
    ))
}

fn days360_us(start: i64, end: i64) -> Result<f64, WorksheetErrorCode> {
    let (sy, sm, mut sd) = ymd_from_excel_serial(WorkbookDateSystem::System1900, start as f64)
        .ok_or(WorksheetErrorCode::Value)?;
    let (ey, em, mut ed) = ymd_from_excel_serial(WorkbookDateSystem::System1900, end as f64)
        .ok_or(WorksheetErrorCode::Value)?;

    let start_last_feb = sm == 2 && sd == days_in_month(sy, sm);
    let end_last_feb = em == 2 && ed == days_in_month(ey, em);

    if sd == 31 || start_last_feb {
        sd = 30;
    }
    if ed == 31 {
        if sd < 30 {
            ed = 1;
        } else {
            ed = 30;
        }
    }
    if end_last_feb && start_last_feb {
        ed = 30;
    }

    Ok(((ey - sy) * 360 + (em - sm) * 30 + (ed - sd)) as f64)
}

fn days360_eu(start: i64, end: i64) -> Result<f64, WorksheetErrorCode> {
    let (sy, sm, mut sd) = ymd_from_excel_serial(WorkbookDateSystem::System1900, start as f64)
        .ok_or(WorksheetErrorCode::Value)?;
    let (ey, em, mut ed) = ymd_from_excel_serial(WorkbookDateSystem::System1900, end as f64)
        .ok_or(WorksheetErrorCode::Value)?;
    if sd == 31 {
        sd = 30;
    }
    if ed == 31 {
        ed = 30;
    }
    Ok(((ey - sy) * 360 + (em - sm) * 30 + (ed - sd)) as f64)
}

fn actual_days(start: i64, end: i64) -> f64 {
    (end - start) as f64
}

fn coupon_day_count(
    start: i64,
    end: i64,
    basis: CouponBasis,
    frequency: i64,
) -> Result<f64, WorksheetErrorCode> {
    match basis {
        CouponBasis::Us30_360 => Ok(360.0 / frequency as f64),
        CouponBasis::ActualActual => Ok(actual_days(start, end)),
        CouponBasis::Actual360 => Ok(360.0 / frequency as f64),
        CouponBasis::Actual365 => Ok(365.0 / frequency as f64),
        CouponBasis::European30_360 => Ok(360.0 / frequency as f64),
    }
}

fn accrued_or_remaining_days(
    start: i64,
    end: i64,
    basis: CouponBasis,
) -> Result<f64, WorksheetErrorCode> {
    match basis {
        CouponBasis::Us30_360 => days360_us(start, end),
        CouponBasis::ActualActual | CouponBasis::Actual360 | CouponBasis::Actual365 => {
            Ok(actual_days(start, end))
        }
        CouponBasis::European30_360 => days360_eu(start, end),
    }
}

fn parse_basis(value: f64) -> Result<CouponBasis, WorksheetErrorCode> {
    if !value.is_finite() {
        return Err(WorksheetErrorCode::Num);
    }
    match value.trunc() as i64 {
        0 => Ok(CouponBasis::Us30_360),
        1 => Ok(CouponBasis::ActualActual),
        2 => Ok(CouponBasis::Actual360),
        3 => Ok(CouponBasis::Actual365),
        4 => Ok(CouponBasis::European30_360),
        _ => Err(WorksheetErrorCode::Num),
    }
}

fn parse_frequency(value: f64) -> Result<i64, WorksheetErrorCode> {
    if !value.is_finite() {
        return Err(WorksheetErrorCode::Num);
    }
    match value.trunc() as i64 {
        1 | 2 | 4 => Ok(value.trunc() as i64),
        _ => Err(WorksheetErrorCode::Num),
    }
}

fn parse_coupon_date(value: f64) -> Result<i64, WorksheetErrorCode> {
    if !value.is_finite() {
        return Err(WorksheetErrorCode::Num);
    }
    let serial = value.trunc() as i64;
    if serial < 0 || serial > max_excel_serial() {
        return Err(WorksheetErrorCode::Num);
    }
    ymd_from_excel_serial(WorkbookDateSystem::System1900, serial as f64)
        .ok_or(WorksheetErrorCode::Num)?;
    Ok(serial)
}

fn parse_coupon_context(
    settlement: f64,
    maturity: f64,
    frequency: f64,
    basis: Option<f64>,
) -> Result<CouponContext, WorksheetErrorCode> {
    let settlement = parse_coupon_date(settlement)?;
    let maturity = parse_coupon_date(maturity)?;
    let frequency = parse_frequency(frequency)?;
    let basis = parse_basis(basis.unwrap_or(0.0))?;
    let (_, _, maturity_anchor_day) =
        ymd_from_excel_serial(WorkbookDateSystem::System1900, maturity as f64)
            .ok_or(WorksheetErrorCode::Num)?;
    if settlement >= maturity {
        return Err(WorksheetErrorCode::Num);
    }
    Ok(CouponContext {
        settlement,
        maturity,
        frequency,
        basis,
        maturity_anchor_day,
    })
}

fn locate_coupon_period(ctx: CouponContext) -> Result<CouponPeriod, WorksheetErrorCode> {
    let months_per_coupon = 12 / ctx.frequency;
    let mut next = ctx.maturity;
    let mut coupons = 1i64;
    loop {
        let previous =
            add_months_clamped(next, -months_per_coupon).ok_or(WorksheetErrorCode::Num)?;
        if previous <= ctx.settlement {
            let raw_previous =
                add_months_with_anchor_day(next, -months_per_coupon, ctx.maturity_anchor_day)
                    .ok_or(WorksheetErrorCode::Num)?;
            let mut period = CouponPeriod {
                previous: previous.max(0),
                next,
                remaining_coupons: coupons,
                raw_previous,
            };
            if ctx.settlement == period.next {
                let forward_next = add_months_clamped(period.next, months_per_coupon)
                    .ok_or(WorksheetErrorCode::Num)?;
                period.previous = period.next;
                period.raw_previous = period.next;
                period.next = forward_next;
                period.remaining_coupons -= 1;
            }
            return Ok(period);
        }
        next = previous;
        coupons += 1;
    }
}

pub fn coupdaybs_kernel(
    settlement: f64,
    maturity: f64,
    frequency: f64,
    basis: Option<f64>,
) -> Result<f64, WorksheetErrorCode> {
    let ctx = parse_coupon_context(settlement, maturity, frequency, basis)?;
    let period = locate_coupon_period(ctx)?;
    accrued_or_remaining_days(period.previous, ctx.settlement, ctx.basis)
}

pub fn coupdays_kernel(
    settlement: f64,
    maturity: f64,
    frequency: f64,
    basis: Option<f64>,
) -> Result<f64, WorksheetErrorCode> {
    let ctx = parse_coupon_context(settlement, maturity, frequency, basis)?;
    let period = locate_coupon_period(ctx)?;
    let period_start = match ctx.basis {
        CouponBasis::ActualActual => period.raw_previous,
        _ => period.previous,
    };
    coupon_day_count(period_start, period.next, ctx.basis, ctx.frequency)
}

pub fn coupdaysnc_kernel(
    settlement: f64,
    maturity: f64,
    frequency: f64,
    basis: Option<f64>,
) -> Result<f64, WorksheetErrorCode> {
    let ctx = parse_coupon_context(settlement, maturity, frequency, basis)?;
    let period = locate_coupon_period(ctx)?;
    accrued_or_remaining_days(ctx.settlement, period.next, ctx.basis)
}

pub fn coupncd_kernel(
    settlement: f64,
    maturity: f64,
    frequency: f64,
    basis: Option<f64>,
) -> Result<f64, WorksheetErrorCode> {
    let ctx = parse_coupon_context(settlement, maturity, frequency, basis)?;
    let period = locate_coupon_period(ctx)?;
    Ok(period.next as f64)
}

pub fn coupnum_kernel(
    settlement: f64,
    maturity: f64,
    frequency: f64,
    basis: Option<f64>,
) -> Result<f64, WorksheetErrorCode> {
    let ctx = parse_coupon_context(settlement, maturity, frequency, basis)?;
    let period = locate_coupon_period(ctx)?;
    Ok(period.remaining_coupons as f64)
}

pub fn couppcd_kernel(
    settlement: f64,
    maturity: f64,
    frequency: f64,
    basis: Option<f64>,
) -> Result<f64, WorksheetErrorCode> {
    let ctx = parse_coupon_context(settlement, maturity, frequency, basis)?;
    let period = locate_coupon_period(ctx)?;
    Ok(period.previous as f64)
}

fn eval_coupon_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
    meta: &FunctionMeta,
    kernel: fn(f64, f64, f64, Option<f64>) -> Result<f64, WorksheetErrorCode>,
) -> Result<EvalValue, CouponEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| {
            if !meta.arity.accepts(prepared.len()) {
                return Err(CouponEvalError::ArityMismatch {
                    expected_min: meta.arity.min,
                    expected_max: meta.arity.max,
                    actual: prepared.len(),
                });
            }
            let settlement =
                coerce_prepared_to_number(&prepared[0]).map_err(CouponEvalError::Coercion)?;
            let maturity =
                coerce_prepared_to_number(&prepared[1]).map_err(CouponEvalError::Coercion)?;
            let frequency =
                coerce_prepared_to_number(&prepared[2]).map_err(CouponEvalError::Coercion)?;
            let basis = if prepared.len() > 3 {
                Some(coerce_prepared_to_number(&prepared[3]).map_err(CouponEvalError::Coercion)?)
            } else {
                None
            };
            kernel(settlement, maturity, frequency, basis)
                .map(EvalValue::Number)
                .map_err(CouponEvalError::Domain)
        },
        CouponEvalError::Coercion,
    )
}

pub fn eval_coupdaybs_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, CouponEvalError> {
    eval_coupon_surface(args, resolver, &COUPDAYBS_META, coupdaybs_kernel)
}

pub fn eval_coupdays_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, CouponEvalError> {
    eval_coupon_surface(args, resolver, &COUPDAYS_META, coupdays_kernel)
}

pub fn eval_coupdaysnc_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, CouponEvalError> {
    eval_coupon_surface(args, resolver, &COUPDAYSNC_META, coupdaysnc_kernel)
}

pub fn eval_coupncd_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, CouponEvalError> {
    eval_coupon_surface(args, resolver, &COUPNCD_META, coupncd_kernel)
}

pub fn eval_coupnum_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, CouponEvalError> {
    eval_coupon_surface(args, resolver, &COUPNUM_META, coupnum_kernel)
}

pub fn eval_couppcd_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, CouponEvalError> {
    eval_coupon_surface(args, resolver, &COUPPCD_META, couppcd_kernel)
}

pub fn map_coupon_error_to_ws(error: &CouponEvalError) -> WorksheetErrorCode {
    match error {
        CouponEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        CouponEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        CouponEvalError::Coercion(_) => WorksheetErrorCode::Value,
        CouponEvalError::Domain(code) => *code,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::{RefResolutionError, ReferenceResolver, ResolverCapabilities};
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

    fn serial(year: i64, month: i64, day: i64) -> f64 {
        excel_serial_from_ymd(WorkbookDateSystem::System1900, year, month, day).unwrap()
    }

    fn num(n: f64) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Number(n))
    }

    #[test]
    fn regular_semiannual_actual_actual_matches_known_example() {
        let settlement = serial(2011, 1, 25);
        let maturity = serial(2011, 11, 15);
        assert_eq!(
            couppcd_kernel(settlement, maturity, 2.0, Some(1.0)),
            Ok(serial(2010, 11, 15))
        );
        assert_eq!(
            coupncd_kernel(settlement, maturity, 2.0, Some(1.0)),
            Ok(serial(2011, 5, 15))
        );
        assert_eq!(
            coupdaybs_kernel(settlement, maturity, 2.0, Some(1.0)),
            Ok(71.0)
        );
        assert_eq!(
            coupdaysnc_kernel(settlement, maturity, 2.0, Some(1.0)),
            Ok(110.0)
        );
        assert_eq!(
            coupdays_kernel(settlement, maturity, 2.0, Some(1.0)),
            Ok(181.0)
        );
        assert_eq!(
            coupnum_kernel(settlement, maturity, 2.0, Some(1.0)),
            Ok(2.0)
        );
    }

    #[test]
    fn basis_zero_two_three_and_four_follow_bounded_contract() {
        let settlement = serial(2011, 1, 25);
        let maturity = serial(2011, 11, 15);
        assert_eq!(
            coupdaybs_kernel(settlement, maturity, 2.0, Some(0.0)),
            Ok(70.0)
        );
        assert_eq!(
            coupdaysnc_kernel(settlement, maturity, 2.0, Some(0.0)),
            Ok(110.0)
        );
        assert_eq!(
            coupdays_kernel(settlement, maturity, 2.0, Some(0.0)),
            Ok(180.0)
        );
        assert_eq!(
            coupdays_kernel(settlement, maturity, 2.0, Some(2.0)),
            Ok(180.0)
        );
        assert_eq!(
            coupdays_kernel(settlement, maturity, 2.0, Some(3.0)),
            Ok(182.5)
        );
        assert_eq!(
            coupdays_kernel(settlement, maturity, 2.0, Some(4.0)),
            Ok(180.0)
        );
    }

    #[test]
    fn quarterly_end_of_month_schedule_clamps_coupon_dates() {
        let settlement = serial(2024, 2, 15);
        let maturity = serial(2024, 11, 30);
        assert_eq!(
            couppcd_kernel(settlement, maturity, 4.0, Some(1.0)),
            Ok(serial(2023, 11, 30))
        );
        assert_eq!(
            coupncd_kernel(settlement, maturity, 4.0, Some(1.0)),
            Ok(serial(2024, 2, 29))
        );
        assert_eq!(
            coupnum_kernel(settlement, maturity, 4.0, Some(1.0)),
            Ok(4.0)
        );
        assert_eq!(
            coupdays_kernel(settlement, maturity, 4.0, Some(1.0)),
            Ok(91.0)
        );
    }

    #[test]
    fn surface_eval_and_mapping_follow_house_style() {
        let resolver = NoResolver;
        let got = eval_coupdays_surface(
            &[
                num(serial(2011, 1, 25)),
                num(serial(2011, 11, 15)),
                num(2.0),
                num(1.0),
            ],
            &resolver,
        );
        assert_eq!(got, Ok(EvalValue::Number(181.0)));
        assert_eq!(
            map_coupon_error_to_ws(&CouponEvalError::ArityMismatch {
                expected_min: 4,
                expected_max: 4,
                actual: 3,
            }),
            WorksheetErrorCode::Value
        );
    }

    #[test]
    fn invalid_frequency_basis_and_order_are_rejected() {
        let settlement = serial(2011, 1, 25);
        let maturity = serial(2011, 11, 15);
        assert_eq!(
            coupnum_kernel(settlement, maturity, 3.0, Some(1.0)),
            Err(WorksheetErrorCode::Num)
        );
        assert_eq!(
            coupnum_kernel(settlement, maturity, 2.0, Some(9.0)),
            Err(WorksheetErrorCode::Num)
        );
        assert_eq!(
            coupnum_kernel(maturity, settlement, 2.0, Some(1.0)),
            Err(WorksheetErrorCode::Num)
        );
    }

    #[test]
    fn serial_zero_is_admitted_but_negative_or_nonfinite_dates_are_num_errors() {
        let maturity = serial(2011, 11, 15);
        assert_eq!(coupdaybs_kernel(0.0, maturity, 2.0, Some(1.0)), Ok(0.0));
        assert_eq!(couppcd_kernel(0.0, maturity, 2.0, Some(1.0)), Ok(0.0));
        assert_eq!(
            coupdaybs_kernel(-1.0, maturity, 2.0, Some(1.0)),
            Err(WorksheetErrorCode::Num)
        );
        assert_eq!(
            coupdaybs_kernel(f64::NAN, maturity, 2.0, Some(1.0)),
            Err(WorksheetErrorCode::Num)
        );
    }

    #[test]
    fn settlement_on_coupon_date_advances_to_following_period() {
        let maturity = serial(2011, 11, 15);
        assert_eq!(couppcd_kernel(136.0, maturity, 2.0, Some(1.0)), Ok(136.0));
        assert_eq!(coupncd_kernel(136.0, maturity, 2.0, Some(1.0)), Ok(320.0));
        assert_eq!(coupnum_kernel(136.0, maturity, 2.0, Some(1.0)), Ok(223.0));
    }
}

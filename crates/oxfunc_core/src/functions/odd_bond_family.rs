use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    PreparedArgValue, coerce_prepared_to_number, run_values_only_prepared,
};
use crate::locale_format::{WorkbookDateSystem, excel_serial_from_ymd, ymd_from_excel_serial};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

const ODD_BOND_META_BASE: FunctionMeta = FunctionMeta {
    function_id: "FUNC.ODD_BOND_BASE",
    arity: Arity::exact(7),
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

pub const ODDFPRICE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.ODDFPRICE",
    arity: Arity { min: 8, max: 9 },
    ..ODD_BOND_META_BASE
};
pub const ODDFYIELD_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.ODDFYIELD",
    arity: Arity { min: 8, max: 9 },
    ..ODD_BOND_META_BASE
};
pub const ODDLPRICE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.ODDLPRICE",
    arity: Arity { min: 7, max: 8 },
    ..ODD_BOND_META_BASE
};
pub const ODDLYIELD_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.ODDLYIELD",
    arity: Arity { min: 7, max: 8 },
    ..ODD_BOND_META_BASE
};

#[derive(Debug, Clone, PartialEq)]
pub enum OddBondEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
    Domain(WorksheetErrorCode),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DayCountBasis {
    Us30_360,
    ActualActual,
    Actual360,
    Actual365,
    European30_360,
}

fn arity_error(meta: &FunctionMeta, actual: usize) -> OddBondEvalError {
    OddBondEvalError::ArityMismatch {
        expected_min: meta.arity.min,
        expected_max: meta.arity.max,
        actual,
    }
}

fn number_arg(args: &[PreparedArgValue], idx: usize) -> Result<f64, OddBondEvalError> {
    args.get(idx)
        .ok_or(OddBondEvalError::Domain(WorksheetErrorCode::Value))
        .and_then(|value| coerce_prepared_to_number(value).map_err(OddBondEvalError::Coercion))
}

fn parse_basis(value: f64) -> Result<DayCountBasis, OddBondEvalError> {
    if !value.is_finite() {
        return Err(OddBondEvalError::Domain(WorksheetErrorCode::Num));
    }
    match value.trunc() as i64 {
        0 => Ok(DayCountBasis::Us30_360),
        1 => Ok(DayCountBasis::ActualActual),
        2 => Ok(DayCountBasis::Actual360),
        3 => Ok(DayCountBasis::Actual365),
        4 => Ok(DayCountBasis::European30_360),
        _ => Err(OddBondEvalError::Domain(WorksheetErrorCode::Num)),
    }
}

fn parse_frequency(value: f64) -> Result<i64, OddBondEvalError> {
    if !value.is_finite() {
        return Err(OddBondEvalError::Domain(WorksheetErrorCode::Num));
    }
    match value.trunc() as i64 {
        1 | 2 | 4 => Ok(value.trunc() as i64),
        _ => Err(OddBondEvalError::Domain(WorksheetErrorCode::Num)),
    }
}

fn max_excel_serial() -> i64 {
    excel_serial_from_ymd(WorkbookDateSystem::System1900, 9999, 12, 31).unwrap() as i64
}

fn parse_date_serial(value: f64) -> Result<i64, OddBondEvalError> {
    if !value.is_finite() {
        return Err(OddBondEvalError::Domain(WorksheetErrorCode::Value));
    }
    let serial = value.trunc() as i64;
    if serial < 1 || serial > max_excel_serial() {
        return Err(OddBondEvalError::Domain(WorksheetErrorCode::Value));
    }
    ymd_from_excel_serial(WorkbookDateSystem::System1900, serial as f64)
        .ok_or(OddBondEvalError::Domain(WorksheetErrorCode::Value))?;
    Ok(serial)
}

fn days_in_year(year: i64) -> f64 {
    if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 {
        366.0
    } else {
        365.0
    }
}

fn days_in_month(year: i64, month: i64) -> i64 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if days_in_year(year) == 366.0 {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
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
    excel_serial_from_ymd(
        WorkbookDateSystem::System1900,
        target_year,
        target_month,
        target_day,
    )
    .map(|v| v as i64)
}

fn days360_us(start: i64, end: i64) -> Result<f64, OddBondEvalError> {
    let (sy, sm, mut sd) = ymd_from_excel_serial(WorkbookDateSystem::System1900, start as f64)
        .ok_or(OddBondEvalError::Domain(WorksheetErrorCode::Value))?;
    let (ey, em, mut ed) = ymd_from_excel_serial(WorkbookDateSystem::System1900, end as f64)
        .ok_or(OddBondEvalError::Domain(WorksheetErrorCode::Value))?;

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

fn days360_eu(start: i64, end: i64) -> Result<f64, OddBondEvalError> {
    let (sy, sm, mut sd) = ymd_from_excel_serial(WorkbookDateSystem::System1900, start as f64)
        .ok_or(OddBondEvalError::Domain(WorksheetErrorCode::Value))?;
    let (ey, em, mut ed) = ymd_from_excel_serial(WorkbookDateSystem::System1900, end as f64)
        .ok_or(OddBondEvalError::Domain(WorksheetErrorCode::Value))?;
    if sd == 31 {
        sd = 30;
    }
    if ed == 31 {
        ed = 30;
    }
    Ok(((ey - sy) * 360 + (em - sm) * 30 + (ed - sd)) as f64)
}

fn day_count(start: i64, end: i64, basis: DayCountBasis) -> Result<f64, OddBondEvalError> {
    match basis {
        DayCountBasis::Us30_360 => days360_us(start, end),
        DayCountBasis::ActualActual | DayCountBasis::Actual360 | DayCountBasis::Actual365 => {
            Ok((end - start) as f64)
        }
        DayCountBasis::European30_360 => days360_eu(start, end),
    }
}

fn count_regular_periods(
    first_coupon: i64,
    maturity: i64,
    months_per_coupon: i64,
) -> Result<i32, OddBondEvalError> {
    let mut count = 0i32;
    let mut cursor = first_coupon;
    while cursor < maturity {
        cursor = add_months_clamped(cursor, months_per_coupon)
            .ok_or(OddBondEvalError::Domain(WorksheetErrorCode::Num))?;
        count += 1;
    }
    if cursor != maturity {
        return Err(OddBondEvalError::Domain(WorksheetErrorCode::Num));
    }
    Ok(count)
}

fn coupon_amount(rate: f64, frequency: i64, redemption: f64) -> f64 {
    redemption * rate / frequency as f64
}

fn validate_positive_inputs(values: &[f64]) -> Result<(), OddBondEvalError> {
    if values.iter().all(|v| v.is_finite()) {
        Ok(())
    } else {
        Err(OddBondEvalError::Domain(WorksheetErrorCode::Value))
    }
}

pub fn oddfprice_kernel(
    settlement: f64,
    maturity: f64,
    issue: f64,
    first_coupon: f64,
    rate: f64,
    yld: f64,
    redemption: f64,
    frequency: f64,
    basis: Option<f64>,
) -> Result<f64, OddBondEvalError> {
    validate_positive_inputs(&[
        settlement,
        maturity,
        issue,
        first_coupon,
        rate,
        yld,
        redemption,
        frequency,
    ])?;
    if rate < 0.0 || yld < 0.0 || redemption <= 0.0 {
        return Err(OddBondEvalError::Domain(WorksheetErrorCode::Num));
    }
    let settlement = parse_date_serial(settlement)?;
    let maturity = parse_date_serial(maturity)?;
    let issue = parse_date_serial(issue)?;
    let first_coupon = parse_date_serial(first_coupon)?;
    let frequency = parse_frequency(frequency)?;
    let basis = parse_basis(basis.unwrap_or(0.0))?;
    if !(issue < settlement && settlement < first_coupon && first_coupon <= maturity) {
        return Err(OddBondEvalError::Domain(WorksheetErrorCode::Num));
    }

    let months_per_coupon = 12 / frequency;
    let prev_coupon = add_months_clamped(first_coupon, -months_per_coupon)
        .ok_or(OddBondEvalError::Domain(WorksheetErrorCode::Num))?;
    if issue <= prev_coupon {
        return Err(OddBondEvalError::Domain(WorksheetErrorCode::Num));
    }
    let period_days = day_count(prev_coupon, first_coupon, basis)?;
    if period_days <= 0.0 {
        return Err(OddBondEvalError::Domain(WorksheetErrorCode::Num));
    }

    let odd_coupon_fraction = day_count(issue, first_coupon, basis)? / period_days;
    let accrued_fraction = day_count(issue, settlement, basis)? / period_days;
    let discount_fraction = day_count(settlement, first_coupon, basis)? / period_days;
    let periods = count_regular_periods(first_coupon, maturity, months_per_coupon)?;
    let coupon = coupon_amount(rate, frequency, redemption);
    let ypf = yld / frequency as f64;
    let base = 1.0 + ypf;
    if base <= 0.0 {
        return Err(OddBondEvalError::Domain(WorksheetErrorCode::Num));
    }

    let mut future_value = 0.0;
    for k in 1..=periods {
        future_value += coupon / base.powf(k as f64);
    }
    future_value += redemption / base.powf(periods as f64);

    let price = (coupon * odd_coupon_fraction + future_value) / base.powf(discount_fraction)
        - coupon * accrued_fraction;
    if price.is_finite() {
        Ok(price)
    } else {
        Err(OddBondEvalError::Domain(WorksheetErrorCode::Num))
    }
}

fn odd_last_boundaries(
    last_interest: i64,
    maturity: i64,
    months_per_coupon: i64,
) -> Result<Vec<i64>, OddBondEvalError> {
    let mut boundaries = vec![last_interest];
    let mut cursor = last_interest;
    loop {
        let next = add_months_clamped(cursor, months_per_coupon)
            .ok_or(OddBondEvalError::Domain(WorksheetErrorCode::Num))?;
        boundaries.push(next);
        if next >= maturity {
            break;
        }
        cursor = next;
        if boundaries.len() > 64 {
            return Err(OddBondEvalError::Domain(WorksheetErrorCode::Num));
        }
    }
    Ok(boundaries)
}

fn fractional_periods_between(
    start: i64,
    end: i64,
    boundaries: &[i64],
    basis: DayCountBasis,
) -> Result<f64, OddBondEvalError> {
    if end <= start {
        return Ok(0.0);
    }
    let mut total = 0.0;
    for window in boundaries.windows(2) {
        let left = window[0];
        let right = window[1];
        if end <= left || start >= right {
            continue;
        }
        let seg_start = start.max(left);
        let seg_end = end.min(right);
        let normal_len = day_count(left, right, basis)?;
        if normal_len <= 0.0 {
            return Err(OddBondEvalError::Domain(WorksheetErrorCode::Num));
        }
        total += day_count(seg_start, seg_end, basis)? / normal_len;
    }
    Ok(total)
}

pub fn oddlprice_kernel(
    settlement: f64,
    maturity: f64,
    last_interest: f64,
    rate: f64,
    yld: f64,
    redemption: f64,
    frequency: f64,
    basis: Option<f64>,
) -> Result<f64, OddBondEvalError> {
    validate_positive_inputs(&[
        settlement,
        maturity,
        last_interest,
        rate,
        yld,
        redemption,
        frequency,
    ])?;
    if rate < 0.0 || yld < 0.0 || redemption <= 0.0 {
        return Err(OddBondEvalError::Domain(WorksheetErrorCode::Num));
    }
    let settlement = parse_date_serial(settlement)?;
    let maturity = parse_date_serial(maturity)?;
    let last_interest = parse_date_serial(last_interest)?;
    let frequency = parse_frequency(frequency)?;
    let basis = parse_basis(basis.unwrap_or(0.0))?;
    if !(last_interest < settlement && settlement < maturity) {
        return Err(OddBondEvalError::Domain(WorksheetErrorCode::Num));
    }

    let months_per_coupon = 12 / frequency;
    let boundaries = odd_last_boundaries(last_interest, maturity, months_per_coupon)?;
    let coupon = coupon_amount(rate, frequency, redemption);
    let ypf = yld / frequency as f64;
    let base = 1.0 + ypf;
    if base <= 0.0 {
        return Err(OddBondEvalError::Domain(WorksheetErrorCode::Num));
    }

    let accrued =
        coupon * fractional_periods_between(last_interest, settlement, &boundaries, basis)?;
    let mut pv = 0.0;
    for window in boundaries.windows(2) {
        let left = window[0];
        let right = window[1];
        if right <= settlement {
            continue;
        }
        if right < maturity {
            let exponent = fractional_periods_between(settlement, right, &boundaries, basis)?;
            pv += coupon / base.powf(exponent);
        } else {
            let final_coupon_fraction =
                fractional_periods_between(left, maturity, &boundaries, basis)?;
            let exponent = fractional_periods_between(settlement, maturity, &boundaries, basis)?;
            pv += (coupon * final_coupon_fraction + redemption) / base.powf(exponent);
            break;
        }
    }
    let price = pv - accrued;
    if price.is_finite() {
        Ok(price)
    } else {
        Err(OddBondEvalError::Domain(WorksheetErrorCode::Num))
    }
}

fn solve_yield_bisection(
    target_price: f64,
    price_at_yield: impl Fn(f64) -> Result<f64, OddBondEvalError>,
) -> Result<f64, OddBondEvalError> {
    if target_price <= 0.0 || !target_price.is_finite() {
        return Err(OddBondEvalError::Domain(WorksheetErrorCode::Num));
    }
    let low = 0.0;
    let price_low = price_at_yield(low)?;
    if target_price > price_low {
        return Err(OddBondEvalError::Domain(WorksheetErrorCode::Num));
    }
    let mut hi = 0.1;
    let mut price_hi = price_at_yield(hi)?;
    for _ in 0..32 {
        if price_hi <= target_price {
            break;
        }
        hi *= 2.0;
        price_hi = price_at_yield(hi)?;
    }
    if price_hi > target_price {
        return Err(OddBondEvalError::Domain(WorksheetErrorCode::Num));
    }

    let mut lo = low;
    let mut hi_mut = hi;
    for _ in 0..100 {
        let mid = (lo + hi_mut) / 2.0;
        let price_mid = price_at_yield(mid)?;
        if (price_mid - target_price).abs() < 1.0e-12 {
            return Ok(mid);
        }
        if price_mid > target_price {
            lo = mid;
        } else {
            hi_mut = mid;
        }
    }
    Ok((lo + hi_mut) / 2.0)
}

pub fn oddfyield_kernel(
    settlement: f64,
    maturity: f64,
    issue: f64,
    first_coupon: f64,
    rate: f64,
    pr: f64,
    redemption: f64,
    frequency: f64,
    basis: Option<f64>,
) -> Result<f64, OddBondEvalError> {
    solve_yield_bisection(pr, |y| {
        oddfprice_kernel(
            settlement,
            maturity,
            issue,
            first_coupon,
            rate,
            y,
            redemption,
            frequency,
            basis,
        )
    })
}

pub fn oddlyield_kernel(
    settlement: f64,
    maturity: f64,
    last_interest: f64,
    rate: f64,
    pr: f64,
    redemption: f64,
    frequency: f64,
    basis: Option<f64>,
) -> Result<f64, OddBondEvalError> {
    solve_yield_bisection(pr, |y| {
        oddlprice_kernel(
            settlement,
            maturity,
            last_interest,
            rate,
            y,
            redemption,
            frequency,
            basis,
        )
    })
}

fn eval_numeric(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
    meta: &FunctionMeta,
    kernel: impl FnOnce(&[PreparedArgValue]) -> Result<f64, OddBondEvalError>,
) -> Result<EvalValue, OddBondEvalError> {
    if !meta.arity.accepts(args.len()) {
        return Err(arity_error(meta, args.len()));
    }
    run_values_only_prepared(
        args,
        resolver,
        |prepared| kernel(prepared).map(EvalValue::Number),
        OddBondEvalError::Coercion,
    )
}

pub fn eval_oddfprice_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, OddBondEvalError> {
    eval_numeric(args, resolver, &ODDFPRICE_META, |prepared| {
        oddfprice_kernel(
            number_arg(prepared, 0)?,
            number_arg(prepared, 1)?,
            number_arg(prepared, 2)?,
            number_arg(prepared, 3)?,
            number_arg(prepared, 4)?,
            number_arg(prepared, 5)?,
            number_arg(prepared, 6)?,
            number_arg(prepared, 7)?,
            prepared
                .get(8)
                .map(|_| number_arg(prepared, 8))
                .transpose()?,
        )
    })
}

pub fn eval_oddfyield_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, OddBondEvalError> {
    eval_numeric(args, resolver, &ODDFYIELD_META, |prepared| {
        oddfyield_kernel(
            number_arg(prepared, 0)?,
            number_arg(prepared, 1)?,
            number_arg(prepared, 2)?,
            number_arg(prepared, 3)?,
            number_arg(prepared, 4)?,
            number_arg(prepared, 5)?,
            number_arg(prepared, 6)?,
            number_arg(prepared, 7)?,
            prepared
                .get(8)
                .map(|_| number_arg(prepared, 8))
                .transpose()?,
        )
    })
}

pub fn eval_oddlprice_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, OddBondEvalError> {
    eval_numeric(args, resolver, &ODDLPRICE_META, |prepared| {
        oddlprice_kernel(
            number_arg(prepared, 0)?,
            number_arg(prepared, 1)?,
            number_arg(prepared, 2)?,
            number_arg(prepared, 3)?,
            number_arg(prepared, 4)?,
            number_arg(prepared, 5)?,
            number_arg(prepared, 6)?,
            prepared
                .get(7)
                .map(|_| number_arg(prepared, 7))
                .transpose()?,
        )
    })
}

pub fn eval_oddlyield_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, OddBondEvalError> {
    eval_numeric(args, resolver, &ODDLYIELD_META, |prepared| {
        oddlyield_kernel(
            number_arg(prepared, 0)?,
            number_arg(prepared, 1)?,
            number_arg(prepared, 2)?,
            number_arg(prepared, 3)?,
            number_arg(prepared, 4)?,
            number_arg(prepared, 5)?,
            number_arg(prepared, 6)?,
            prepared
                .get(7)
                .map(|_| number_arg(prepared, 7))
                .transpose()?,
        )
    })
}

pub fn map_odd_bond_error_to_ws(error: &OddBondEvalError) -> WorksheetErrorCode {
    match error {
        OddBondEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        OddBondEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        OddBondEvalError::Coercion(_) => WorksheetErrorCode::Value,
        OddBondEvalError::Domain(code) => *code,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn serial(year: i64, month: i64, day: i64) -> f64 {
        excel_serial_from_ymd(WorkbookDateSystem::System1900, year, month, day).unwrap()
    }

    fn assert_close(actual: f64, expected: f64, tolerance: f64) {
        assert!(
            (actual - expected).abs() <= tolerance,
            "expected {expected}, got {actual}"
        );
    }

    #[test]
    fn meta_ids_match_expected_function_ids() {
        assert_eq!(ODDFPRICE_META.function_id, "FUNC.ODDFPRICE");
        assert_eq!(ODDFYIELD_META.function_id, "FUNC.ODDFYIELD");
        assert_eq!(ODDLPRICE_META.function_id, "FUNC.ODDLPRICE");
        assert_eq!(ODDLYIELD_META.function_id, "FUNC.ODDLYIELD");
    }

    #[test]
    fn oddfprice_matches_bounded_microsoft_lane() {
        let got = oddfprice_kernel(
            serial(2008, 11, 11),
            serial(2021, 3, 1),
            serial(2008, 10, 15),
            serial(2009, 3, 1),
            0.0785,
            0.0625,
            100.0,
            2.0,
            Some(1.0),
        )
        .expect("oddfprice should succeed");
        assert_close(got, 113.597_717_474_079, 1.0e-9);
    }

    #[test]
    fn oddfyield_inverts_bounded_price_lane() {
        let got = oddfyield_kernel(
            serial(2008, 11, 11),
            serial(2021, 3, 1),
            serial(2008, 10, 15),
            serial(2009, 3, 1),
            0.0785,
            113.597_717_474_079,
            100.0,
            2.0,
            Some(1.0),
        )
        .expect("oddfyield should succeed");
        assert_close(got, 0.0625, 1.0e-10);
    }

    #[test]
    fn oddlprice_matches_bounded_example_lane() {
        let got = oddlprice_kernel(
            serial(2008, 2, 7),
            serial(2008, 6, 15),
            serial(2007, 10, 15),
            0.0375,
            0.0405,
            100.0,
            2.0,
            Some(0.0),
        )
        .expect("oddlprice should succeed");
        assert_close(got, 99.894_839_513_695_3, 1.0e-10);
    }

    #[test]
    fn oddlyield_inverts_bounded_price_lane() {
        let got = oddlyield_kernel(
            serial(2008, 2, 7),
            serial(2008, 6, 15),
            serial(2007, 10, 15),
            0.0375,
            99.894_839_513_695_3,
            100.0,
            2.0,
            Some(0.0),
        )
        .expect("oddlyield should succeed");
        assert_close(got, 0.0405, 1.0e-10);
    }

    #[test]
    fn long_odd_first_is_currently_rejected() {
        assert_eq!(
            oddfprice_kernel(
                serial(2008, 11, 11),
                serial(2021, 3, 1),
                serial(2008, 8, 15),
                serial(2009, 3, 1),
                0.0785,
                0.0625,
                100.0,
                2.0,
                Some(1.0),
            ),
            Err(OddBondEvalError::Domain(WorksheetErrorCode::Num))
        );
    }

    #[test]
    fn invalid_frequency_and_basis_are_rejected() {
        assert_eq!(
            oddlprice_kernel(
                serial(2008, 2, 7),
                serial(2008, 6, 15),
                serial(2007, 10, 15),
                0.0375,
                0.0405,
                100.0,
                3.0,
                Some(0.0),
            ),
            Err(OddBondEvalError::Domain(WorksheetErrorCode::Num))
        );
        assert_eq!(
            oddlprice_kernel(
                serial(2008, 2, 7),
                serial(2008, 6, 15),
                serial(2007, 10, 15),
                0.0375,
                0.0405,
                100.0,
                2.0,
                Some(9.0),
            ),
            Err(OddBondEvalError::Domain(WorksheetErrorCode::Num))
        );
    }
}

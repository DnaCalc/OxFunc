use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{coerce_prepared_to_number, run_values_only_prepared};
use crate::locale_format::{WorkbookDateSystem, excel_serial_from_ymd, ymd_from_excel_serial};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

const DATE_WEEK_BASE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.DATE_WEEK_BASE",
    arity: Arity::exact(1),
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

pub const EDATE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.EDATE",
    arity: Arity::exact(2),
    ..DATE_WEEK_BASE_META
};

pub const EOMONTH_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.EOMONTH",
    arity: Arity::exact(2),
    ..DATE_WEEK_BASE_META
};

pub const WEEKDAY_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.WEEKDAY",
    arity: Arity { min: 1, max: 2 },
    ..DATE_WEEK_BASE_META
};

pub const WEEKNUM_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.WEEKNUM",
    arity: Arity { min: 1, max: 2 },
    ..DATE_WEEK_BASE_META
};

pub const ISOWEEKNUM_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.ISOWEEKNUM",
    ..DATE_WEEK_BASE_META
};

#[derive(Debug, Clone, PartialEq)]
pub enum DateWeekEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
    Domain(WorksheetErrorCode),
}

fn domain_num_error() -> DateWeekEvalError {
    DateWeekEvalError::Domain(WorksheetErrorCode::Num)
}

fn truncate_number_to_i64(n: f64) -> Result<i64, DateWeekEvalError> {
    if !n.is_finite() {
        return Err(domain_num_error());
    }
    Ok(n.trunc() as i64)
}

fn coerce_serial(n: f64) -> Result<i64, DateWeekEvalError> {
    let serial = truncate_number_to_i64(n)?;
    if serial < 0 {
        return Err(domain_num_error());
    }
    Ok(serial)
}

fn normalize_year_month(
    year: i64,
    month: i64,
    offset: i64,
) -> Result<(i64, i64), DateWeekEvalError> {
    let month_index = year
        .checked_mul(12)
        .and_then(|v| v.checked_add(month - 1))
        .and_then(|v| v.checked_add(offset))
        .ok_or_else(domain_num_error)?;
    Ok((month_index.div_euclid(12), month_index.rem_euclid(12) + 1))
}

fn is_gregorian_leap_year(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
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

fn excel_like_ymd_from_serial(serial: i64) -> Option<(i64, i64, i64)> {
    if serial == 0 {
        return Some((1900, 1, 0));
    }
    if serial == 60 {
        return Some((1900, 2, 29));
    }
    ymd_from_excel_serial(WorkbookDateSystem::System1900, serial as f64)
}

fn days_in_month(year: i64, month: i64) -> i64 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_gregorian_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}

pub fn edate_kernel(serial: f64, months: f64) -> Result<f64, WorksheetErrorCode> {
    let serial = coerce_serial(serial).map_err(|e| map_date_week_error_to_ws(&e))?;
    let months = truncate_number_to_i64(months).map_err(|e| map_date_week_error_to_ws(&e))?;
    let (year, month, day) = excel_like_ymd_from_serial(serial).ok_or(WorksheetErrorCode::Num)?;
    let (target_year, target_month) =
        normalize_year_month(year, month, months).map_err(|e| map_date_week_error_to_ws(&e))?;
    let target_day = day.min(days_in_month(target_year, target_month));
    excel_serial_from_ymd(
        WorkbookDateSystem::System1900,
        target_year,
        target_month,
        target_day,
    )
    .ok_or(WorksheetErrorCode::Num)
}

pub fn eomonth_kernel(serial: f64, months: f64) -> Result<f64, WorksheetErrorCode> {
    let serial = coerce_serial(serial).map_err(|e| map_date_week_error_to_ws(&e))?;
    let months = truncate_number_to_i64(months).map_err(|e| map_date_week_error_to_ws(&e))?;
    let (year, month, _) = excel_like_ymd_from_serial(serial).ok_or(WorksheetErrorCode::Num)?;
    let (target_year, target_month) =
        normalize_year_month(year, month, months).map_err(|e| map_date_week_error_to_ws(&e))?;
    excel_serial_from_ymd(
        WorkbookDateSystem::System1900,
        target_year,
        target_month,
        days_in_month(target_year, target_month),
    )
    .ok_or(WorksheetErrorCode::Num)
}

fn weekday_sunday_one_based(serial: i64) -> i64 {
    (serial - 1).rem_euclid(7) + 1
}

fn remap_weekday_for_return_type(sunday_one_based: i64, return_type: i64) -> Option<f64> {
    match return_type {
        1 => Some(sunday_one_based as f64),
        2 | 11 => Some(((sunday_one_based + 5) % 7 + 1) as f64),
        3 => Some(((sunday_one_based + 5) % 7) as f64),
        12..=17 => {
            let start_day = (return_type - 10).rem_euclid(7) + 1;
            Some(((sunday_one_based - start_day).rem_euclid(7) + 1) as f64)
        }
        _ => None,
    }
}

fn weekday_unbounded(serial: i64, return_type: i64) -> Result<f64, WorksheetErrorCode> {
    remap_weekday_for_return_type(weekday_sunday_one_based(serial), return_type)
        .ok_or(WorksheetErrorCode::Num)
}

pub fn weekday_kernel(serial: f64, return_type: Option<f64>) -> Result<f64, WorksheetErrorCode> {
    let serial = coerce_serial(serial).map_err(|e| map_date_week_error_to_ws(&e))?;
    let return_type = return_type.unwrap_or(1.0);
    let return_type =
        truncate_number_to_i64(return_type).map_err(|e| map_date_week_error_to_ws(&e))?;
    weekday_unbounded(serial, return_type)
}

fn iso_weeknum_serial(serial: i64) -> Result<f64, WorksheetErrorCode> {
    let weekday_monday = weekday_unbounded(serial, 2)? as i64;
    let thursday_serial = serial + (4 - weekday_monday);
    let (iso_year, _, _) =
        ymd_from_excel_serial(WorkbookDateSystem::System1900, thursday_serial as f64)
            .ok_or(WorksheetErrorCode::Num)?;
    let jan4_serial = excel_serial_from_ymd_unbounded_1900(iso_year, 1, 4);
    let jan4_weekday = weekday_unbounded(jan4_serial, 2)? as i64;
    let first_monday = jan4_serial - (jan4_weekday - 1);
    Ok(((serial - first_monday).div_euclid(7) + 1) as f64)
}

pub fn weeknum_kernel(serial: f64, return_type: Option<f64>) -> Result<f64, WorksheetErrorCode> {
    let serial = coerce_serial(serial).map_err(|e| map_date_week_error_to_ws(&e))?;
    let return_type = return_type.unwrap_or(1.0);
    let return_type =
        truncate_number_to_i64(return_type).map_err(|e| map_date_week_error_to_ws(&e))?;
    if return_type == 21 {
        return iso_weeknum_serial(serial);
    }
    let Some(_) = remap_weekday_for_return_type(1, return_type) else {
        return Err(WorksheetErrorCode::Num);
    };
    if return_type == 3 {
        return Err(WorksheetErrorCode::Num);
    }
    let (year, _, _) = excel_like_ymd_from_serial(serial).ok_or(WorksheetErrorCode::Num)?;
    let jan1_serial = excel_serial_from_ymd_unbounded_1900(year, 1, 1);
    let jan1_weekday = weekday_kernel(jan1_serial as f64, Some(return_type as f64))? as i64;
    let week1_start = jan1_serial - (jan1_weekday - 1);
    Ok(((serial - week1_start).div_euclid(7) + 1) as f64)
}

pub fn isoweeknum_kernel(serial: f64) -> Result<f64, WorksheetErrorCode> {
    let serial = coerce_serial(serial).map_err(|e| map_date_week_error_to_ws(&e))?;
    iso_weeknum_serial(serial)
}

pub fn eval_edate_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DateWeekEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| {
            if !EDATE_META.arity.accepts(prepared.len()) {
                return Err(DateWeekEvalError::ArityMismatch {
                    expected_min: EDATE_META.arity.min,
                    expected_max: EDATE_META.arity.max,
                    actual: prepared.len(),
                });
            }
            let serial =
                coerce_prepared_to_number(&prepared[0]).map_err(DateWeekEvalError::Coercion)?;
            let months =
                coerce_prepared_to_number(&prepared[1]).map_err(DateWeekEvalError::Coercion)?;
            edate_kernel(serial, months)
                .map(EvalValue::Number)
                .map_err(DateWeekEvalError::Domain)
        },
        DateWeekEvalError::Coercion,
    )
}

pub fn eval_eomonth_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DateWeekEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| {
            if !EOMONTH_META.arity.accepts(prepared.len()) {
                return Err(DateWeekEvalError::ArityMismatch {
                    expected_min: EOMONTH_META.arity.min,
                    expected_max: EOMONTH_META.arity.max,
                    actual: prepared.len(),
                });
            }
            let serial =
                coerce_prepared_to_number(&prepared[0]).map_err(DateWeekEvalError::Coercion)?;
            let months =
                coerce_prepared_to_number(&prepared[1]).map_err(DateWeekEvalError::Coercion)?;
            eomonth_kernel(serial, months)
                .map(EvalValue::Number)
                .map_err(DateWeekEvalError::Domain)
        },
        DateWeekEvalError::Coercion,
    )
}

pub fn eval_weekday_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DateWeekEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| {
            if !WEEKDAY_META.arity.accepts(prepared.len()) {
                return Err(DateWeekEvalError::ArityMismatch {
                    expected_min: WEEKDAY_META.arity.min,
                    expected_max: WEEKDAY_META.arity.max,
                    actual: prepared.len(),
                });
            }
            let serial =
                coerce_prepared_to_number(&prepared[0]).map_err(DateWeekEvalError::Coercion)?;
            let return_type = if prepared.len() > 1 {
                Some(coerce_prepared_to_number(&prepared[1]).map_err(DateWeekEvalError::Coercion)?)
            } else {
                None
            };
            weekday_kernel(serial, return_type)
                .map(EvalValue::Number)
                .map_err(DateWeekEvalError::Domain)
        },
        DateWeekEvalError::Coercion,
    )
}

pub fn eval_weeknum_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DateWeekEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| {
            if !WEEKNUM_META.arity.accepts(prepared.len()) {
                return Err(DateWeekEvalError::ArityMismatch {
                    expected_min: WEEKNUM_META.arity.min,
                    expected_max: WEEKNUM_META.arity.max,
                    actual: prepared.len(),
                });
            }
            let serial =
                coerce_prepared_to_number(&prepared[0]).map_err(DateWeekEvalError::Coercion)?;
            let return_type = if prepared.len() > 1 {
                Some(coerce_prepared_to_number(&prepared[1]).map_err(DateWeekEvalError::Coercion)?)
            } else {
                None
            };
            weeknum_kernel(serial, return_type)
                .map(EvalValue::Number)
                .map_err(DateWeekEvalError::Domain)
        },
        DateWeekEvalError::Coercion,
    )
}

pub fn eval_isoweeknum_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DateWeekEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| {
            if !ISOWEEKNUM_META.arity.accepts(prepared.len()) {
                return Err(DateWeekEvalError::ArityMismatch {
                    expected_min: ISOWEEKNUM_META.arity.min,
                    expected_max: ISOWEEKNUM_META.arity.max,
                    actual: prepared.len(),
                });
            }
            let serial =
                coerce_prepared_to_number(&prepared[0]).map_err(DateWeekEvalError::Coercion)?;
            isoweeknum_kernel(serial)
                .map(EvalValue::Number)
                .map_err(DateWeekEvalError::Domain)
        },
        DateWeekEvalError::Coercion,
    )
}

pub fn map_date_week_error_to_ws(e: &DateWeekEvalError) -> WorksheetErrorCode {
    match e {
        DateWeekEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        DateWeekEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        DateWeekEvalError::Coercion(_) => WorksheetErrorCode::Value,
        DateWeekEvalError::Domain(code) => *code,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edate_kernel_matches_seed_rows() {
        assert_eq!(edate_kernel(45322.0, 1.0), Ok(45351.0));
        assert_eq!(edate_kernel(45322.0, -1.0), Ok(45291.0));
        assert_eq!(edate_kernel(0.0, 0.0), Ok(0.0));
        assert_eq!(edate_kernel(60.0, 0.0), Ok(59.0));
        assert_eq!(edate_kernel(60.0, 1.0), Ok(89.0));
        assert_eq!(edate_kernel(-1.0, 1.0), Err(WorksheetErrorCode::Num));
    }

    #[test]
    fn eomonth_kernel_matches_seed_rows() {
        assert_eq!(eomonth_kernel(45306.0, 0.0), Ok(45322.0));
        assert_eq!(eomonth_kernel(45322.0, 1.0), Ok(45351.0));
        assert_eq!(eomonth_kernel(0.0, 0.0), Ok(31.0));
        assert_eq!(eomonth_kernel(60.0, 0.0), Ok(59.0));
        assert_eq!(eomonth_kernel(-1.0, 1.0), Err(WorksheetErrorCode::Num));
    }

    #[test]
    fn weekday_kernel_matches_seed_rows() {
        assert_eq!(weekday_kernel(1.0, None), Ok(1.0));
        assert_eq!(weekday_kernel(0.0, None), Ok(7.0));
        assert_eq!(weekday_kernel(60.0, None), Ok(4.0));
        assert_eq!(weekday_kernel(45292.0, Some(2.0)), Ok(1.0));
        assert_eq!(weekday_kernel(45292.0, Some(3.0)), Ok(0.0));
        assert_eq!(weekday_kernel(45292.0, Some(12.0)), Ok(7.0));
        assert_eq!(weekday_kernel(45292.0, Some(17.0)), Ok(2.0));
        assert_eq!(
            weekday_kernel(45292.0, Some(0.0)),
            Err(WorksheetErrorCode::Num)
        );
    }

    #[test]
    fn weeknum_kernel_matches_seed_rows() {
        assert_eq!(weeknum_kernel(45292.0, None), Ok(1.0));
        assert_eq!(weeknum_kernel(45292.0, Some(2.0)), Ok(1.0));
        assert_eq!(weeknum_kernel(45292.0, Some(17.0)), Ok(1.0));
        assert_eq!(weeknum_kernel(45292.0, Some(21.0)), Ok(1.0));
        assert_eq!(weeknum_kernel(0.0, None), Ok(0.0));
        assert_eq!(weeknum_kernel(0.0, Some(2.0)), Ok(1.0));
        assert_eq!(weeknum_kernel(60.0, None), Ok(9.0));
        assert_eq!(
            weeknum_kernel(45292.0, Some(3.0)),
            Err(WorksheetErrorCode::Num)
        );
    }

    #[test]
    fn isoweeknum_kernel_matches_seed_rows() {
        assert_eq!(isoweeknum_kernel(45292.0), Ok(1.0));
        assert_eq!(isoweeknum_kernel(44196.0), Ok(53.0));
        assert_eq!(isoweeknum_kernel(60.0), Ok(9.0));
        assert_eq!(isoweeknum_kernel(0.0), Ok(52.0));
        assert_eq!(isoweeknum_kernel(-1.0), Err(WorksheetErrorCode::Num));
    }
}

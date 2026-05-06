use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    PreparedArgValue, coerce_prepared_to_number, coerce_prepared_to_text, run_values_only_prepared,
};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

const DATE_VALUE_FAMILY_BASE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.DATE_VALUE_FAMILY_BASE",
    arity: Arity::exact(1),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::LocaleProfile,
    surface_fec_dependency_profile: FecDependencyProfile::Composite,
};

pub const DATEVALUE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.DATEVALUE",
    ..DATE_VALUE_FAMILY_BASE_META
};

pub const TIMEVALUE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.TIMEVALUE",
    ..DATE_VALUE_FAMILY_BASE_META
};

pub const DAYS360_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.DAYS360",
    arity: Arity { min: 2, max: 3 },
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
    ..DATE_VALUE_FAMILY_BASE_META
};

pub const DATEDIF_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.DATEDIF",
    arity: Arity::exact(3),
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
    ..DATE_VALUE_FAMILY_BASE_META
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateValueFamilyError {
    Arity {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion,
    Value,
    Num,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateDiffUnit {
    Y,
    M,
    D,
    Ym,
    Yd,
    Md,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ParsedDateTime {
    date: Option<(i64, i64, i64)>,
    time_fraction: Option<f64>,
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

fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = mp + if mp < 10 { 3 } else { -9 };
    let year = y + i64::from(m <= 2);
    (year, m, d)
}

fn excel_serial_from_ymd_1900(
    year: i64,
    month: i64,
    day: i64,
) -> Result<f64, DateValueFamilyError> {
    if year == 1900 && month == 2 && day == 29 {
        return Ok(60.0);
    }
    let base = days_from_civil(1899, 12, 31);
    let days = days_from_civil(year, month, 1) - base + (day - 1);
    if days < 0 {
        return Err(DateValueFamilyError::Num);
    }
    Ok(if days >= 60 {
        (days + 1) as f64
    } else {
        days as f64
    })
}

fn ymd_from_excel_serial_1900(serial: f64) -> Result<(i64, i64, i64), DateValueFamilyError> {
    if !serial.is_finite() {
        return Err(DateValueFamilyError::Num);
    }
    let whole = serial.trunc() as i64;
    if whole < 0 {
        return Err(DateValueFamilyError::Num);
    }
    if whole == 0 {
        return Ok((1900, 1, 0));
    }
    if whole == 60 {
        return Ok((1900, 2, 29));
    }
    let adjusted = if whole >= 60 { whole - 1 } else { whole };
    let base = days_from_civil(1899, 12, 31);
    Ok(civil_from_days(base + adjusted))
}

fn is_gregorian_leap_year(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
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
        _ => 0,
    }
}

fn last_day_of_previous_month(year: i64, month: i64) -> i64 {
    let (prev_year, prev_month) = if month == 1 {
        (year - 1, 12)
    } else {
        (year, month - 1)
    };
    days_in_month(prev_year, prev_month)
}

fn is_last_day_of_february(year: i64, month: i64, day: i64) -> bool {
    month == 2 && day == days_in_month(year, month)
}

fn parse_iso_ymd(text: &str) -> Option<(i64, i64, i64)> {
    let parts: Vec<&str> = text.split('-').collect();
    if parts.len() != 3 {
        return None;
    }
    let year = parts[0].parse::<i64>().ok()?;
    let month = parts[1].parse::<i64>().ok()?;
    let day = parts[2].parse::<i64>().ok()?;
    Some((year, month, day))
}

fn month_from_abbrev(s: &str) -> Option<i64> {
    match s.to_ascii_lowercase().as_str() {
        "jan" => Some(1),
        "feb" => Some(2),
        "mar" => Some(3),
        "apr" => Some(4),
        "may" => Some(5),
        "jun" => Some(6),
        "jul" => Some(7),
        "aug" => Some(8),
        "sep" => Some(9),
        "oct" => Some(10),
        "nov" => Some(11),
        "dec" => Some(12),
        _ => None,
    }
}

fn parse_text_month_date(text: &str) -> Option<(i64, i64, i64)> {
    let parts: Vec<&str> = text.split('-').collect();
    if parts.len() != 3 {
        return None;
    }
    let day = parts[0].parse::<i64>().ok()?;
    let month = month_from_abbrev(parts[1].trim())?;
    let year = parts[2].parse::<i64>().ok()?;
    Some((year, month, day))
}

fn parse_time_text(text: &str) -> Option<f64> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }

    let upper = trimmed.to_ascii_uppercase();
    let (body, ampm) = if let Some(rest) = upper.strip_suffix(" AM") {
        (rest.trim_end(), Some("AM"))
    } else if let Some(rest) = upper.strip_suffix(" PM") {
        (rest.trim_end(), Some("PM"))
    } else if let Some(rest) = upper.strip_suffix("AM") {
        (rest.trim_end(), Some("AM"))
    } else if let Some(rest) = upper.strip_suffix("PM") {
        (rest.trim_end(), Some("PM"))
    } else {
        (upper.as_str(), None)
    };

    let parts: Vec<&str> = body.split(':').collect();
    if parts.len() < 2 || parts.len() > 3 {
        return None;
    }

    let mut hour = parts[0].trim().parse::<i64>().ok()?;
    let minute = parts[1].trim().parse::<i64>().ok()?;
    let second = if parts.len() == 3 {
        parts[2].trim().parse::<i64>().ok()?
    } else {
        0
    };

    if minute < 0 || minute >= 60 || second < 0 || second >= 60 {
        return None;
    }

    match ampm {
        Some("AM") => {
            if hour < 1 || hour > 12 {
                return None;
            }
            if hour == 12 {
                hour = 0;
            }
        }
        Some("PM") => {
            if hour < 1 || hour > 12 {
                return None;
            }
            if hour != 12 {
                hour += 12;
            }
        }
        _ => {
            if !(0..=23).contains(&hour) {
                return None;
            }
        }
    }

    let total_seconds = hour * 3600 + minute * 60 + second;
    Some(total_seconds as f64 / 86_400.0)
}

fn split_time_suffix(text: &str) -> (&str, Option<&str>) {
    let trimmed = text.trim();
    if let Some((lhs1, rhs1)) = trimmed.rsplit_once(' ') {
        if parse_time_text(rhs1).is_some() {
            return (lhs1.trim_end(), Some(rhs1));
        }
        if let Some((lhs2, _)) = lhs1.rsplit_once(' ') {
            let candidate = &trimmed[lhs2.len() + 1..];
            if parse_time_text(candidate).is_some() {
                return (lhs2.trim_end(), Some(candidate));
            }
        }
    }
    (trimmed, None)
}

fn parse_date_time_text(text: &str) -> Result<ParsedDateTime, DateValueFamilyError> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(DateValueFamilyError::Value);
    }

    if let Some(time_fraction) = parse_time_text(trimmed) {
        return Ok(ParsedDateTime {
            date: None,
            time_fraction: Some(time_fraction),
        });
    }

    let (date_part, trailing_time) = split_time_suffix(trimmed);
    let date = parse_iso_ymd(date_part).or_else(|| parse_text_month_date(date_part));

    match (date, trailing_time) {
        (Some(date), Some(time_text)) => Ok(ParsedDateTime {
            date: Some(date),
            time_fraction: parse_time_text(time_text),
        }),
        (Some(date), None) => Ok(ParsedDateTime {
            date: Some(date),
            time_fraction: None,
        }),
        (None, Some(_)) => Err(DateValueFamilyError::Value),
        (None, None) => Err(DateValueFamilyError::Value),
    }
}

pub fn datevalue_kernel(text: &str) -> Result<f64, DateValueFamilyError> {
    let parsed = parse_date_time_text(text)?;
    match parsed.date {
        Some((year, month, day)) => excel_serial_from_ymd_1900(year, month, day),
        None => Ok(0.0),
    }
}

pub fn timevalue_kernel(text: &str) -> Result<f64, DateValueFamilyError> {
    let parsed = parse_date_time_text(text)?;
    Ok(parsed.time_fraction.unwrap_or(0.0))
}

pub fn days360_kernel(
    start_serial: f64,
    end_serial: f64,
    european: bool,
) -> Result<f64, DateValueFamilyError> {
    let (sy, sm, mut sd) = ymd_from_excel_serial_1900(start_serial)?;
    let (ey, em, mut ed) = ymd_from_excel_serial_1900(end_serial)?;

    if european {
        if sd == 31 {
            sd = 30;
        }
        if ed == 31 {
            ed = 30;
        }
    } else {
        if sd == 31 || is_last_day_of_february(sy, sm, sd) {
            sd = 30;
        }
        if ed == 31 {
            if sd < 30 {
                let (next_year, next_month) = if em == 12 { (ey + 1, 1) } else { (ey, em + 1) };
                return Ok((next_year - sy) as f64 * 360.0
                    + (next_month - sm) as f64 * 30.0
                    + (1 - sd) as f64);
            }
            ed = 30;
        }
        if is_last_day_of_february(ey, em, ed) && is_last_day_of_february(sy, sm, sd) {
            ed = 30;
        }
    }

    Ok((ey - sy) as f64 * 360.0 + (em - sm) as f64 * 30.0 + (ed - sd) as f64)
}

pub fn parse_datedif_unit(unit: &str) -> Result<DateDiffUnit, DateValueFamilyError> {
    match unit.trim().to_ascii_uppercase().as_str() {
        "Y" => Ok(DateDiffUnit::Y),
        "M" => Ok(DateDiffUnit::M),
        "D" => Ok(DateDiffUnit::D),
        "YM" => Ok(DateDiffUnit::Ym),
        "YD" => Ok(DateDiffUnit::Yd),
        "MD" => Ok(DateDiffUnit::Md),
        _ => Err(DateValueFamilyError::Num),
    }
}

fn compare_month_day(lhs: (i64, i64), rhs: (i64, i64)) -> std::cmp::Ordering {
    lhs.cmp(&rhs)
}

fn clamp_day(year: i64, month: i64, day: i64) -> i64 {
    day.min(days_in_month(year, month))
}

pub fn datedif_kernel(
    start_serial: f64,
    end_serial: f64,
    unit: DateDiffUnit,
) -> Result<f64, DateValueFamilyError> {
    let start = start_serial.trunc();
    let end = end_serial.trunc();
    if !start.is_finite() || !end.is_finite() || start < 0.0 || end < 0.0 {
        return Err(DateValueFamilyError::Num);
    }
    if end < start {
        return Err(DateValueFamilyError::Num);
    }

    let (sy, sm, sd) = ymd_from_excel_serial_1900(start)?;
    let (ey, em, ed) = ymd_from_excel_serial_1900(end)?;

    match unit {
        DateDiffUnit::D => Ok(end - start),
        DateDiffUnit::Y => {
            let mut years = ey - sy;
            if compare_month_day((em, ed), (sm, sd)) == std::cmp::Ordering::Less {
                years -= 1;
            }
            Ok(years as f64)
        }
        DateDiffUnit::M => {
            let mut months = (ey - sy) * 12 + (em - sm);
            if ed < sd {
                months -= 1;
            }
            Ok(months as f64)
        }
        DateDiffUnit::Ym => {
            let mut months = (em - sm).rem_euclid(12);
            if ed < sd {
                months = (months - 1).rem_euclid(12);
            }
            Ok(months as f64)
        }
        DateDiffUnit::Yd => {
            let anniversary_day = clamp_day(ey, sm, sd);
            let candidate = excel_serial_from_ymd_1900(ey, sm, anniversary_day)?;
            if candidate <= end {
                Ok(end - candidate)
            } else {
                let prev_anniversary_day = clamp_day(ey - 1, sm, sd);
                let prev = excel_serial_from_ymd_1900(ey - 1, sm, prev_anniversary_day)?;
                Ok(end - prev)
            }
        }
        DateDiffUnit::Md => {
            let value = if ed >= sd {
                ed - sd
            } else {
                ed + last_day_of_previous_month(ey, em) - sd
            };
            Ok(value as f64)
        }
    }
}

fn prep_len_error(meta: &FunctionMeta, actual: usize) -> DateValueFamilyError {
    DateValueFamilyError::Arity {
        expected_min: meta.arity.min,
        expected_max: meta.arity.max,
        actual,
    }
}

fn map_family_err_to_eval(err: DateValueFamilyError) -> EvalValue {
    match err {
        DateValueFamilyError::Value | DateValueFamilyError::Coercion => {
            EvalValue::Error(WorksheetErrorCode::Value)
        }
        DateValueFamilyError::Num => EvalValue::Error(WorksheetErrorCode::Num),
        DateValueFamilyError::Arity { .. } => EvalValue::Error(WorksheetErrorCode::Value),
    }
}

fn coerce_prepared_to_date_serial(
    prepared: &PreparedArgValue,
) -> Result<f64, DateValueFamilyError> {
    match prepared {
        PreparedArgValue::Eval(EvalValue::Text(text)) => {
            let parsed = parse_date_time_text(&text.to_string_lossy())?;
            let Some((year, month, day)) = parsed.date else {
                return Err(DateValueFamilyError::Value);
            };
            excel_serial_from_ymd_1900(year, month, day)
        }
        _ => coerce_prepared_to_number(prepared).map_err(|_| DateValueFamilyError::Coercion),
    }
}

pub fn eval_datevalue_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, DateValueFamilyError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared: &[PreparedArgValue]| {
            if !DATEVALUE_META.arity.accepts(prepared.len()) {
                return Ok(map_family_err_to_eval(prep_len_error(
                    &DATEVALUE_META,
                    prepared.len(),
                )));
            }
            let text =
                coerce_prepared_to_text(&prepared[0]).map_err(|_| DateValueFamilyError::Coercion);
            Ok(
                match text.and_then(|t| datevalue_kernel(&t.to_string_lossy())) {
                    Ok(value) => EvalValue::Number(value),
                    Err(err) => map_family_err_to_eval(err),
                },
            )
        },
        |_| DateValueFamilyError::Coercion,
    )
}

pub fn eval_timevalue_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, DateValueFamilyError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared: &[PreparedArgValue]| {
            if !TIMEVALUE_META.arity.accepts(prepared.len()) {
                return Ok(map_family_err_to_eval(prep_len_error(
                    &TIMEVALUE_META,
                    prepared.len(),
                )));
            }
            let text =
                coerce_prepared_to_text(&prepared[0]).map_err(|_| DateValueFamilyError::Coercion);
            Ok(
                match text.and_then(|t| timevalue_kernel(&t.to_string_lossy())) {
                    Ok(value) => EvalValue::Number(value),
                    Err(err) => map_family_err_to_eval(err),
                },
            )
        },
        |_| DateValueFamilyError::Coercion,
    )
}

pub fn eval_days360_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, DateValueFamilyError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared: &[PreparedArgValue]| {
            if !DAYS360_META.arity.accepts(prepared.len()) {
                return Ok(map_family_err_to_eval(prep_len_error(
                    &DAYS360_META,
                    prepared.len(),
                )));
            }
            let start = coerce_prepared_to_number(&prepared[0])
                .map_err(|_| DateValueFamilyError::Coercion)?;
            let end = coerce_prepared_to_number(&prepared[1])
                .map_err(|_| DateValueFamilyError::Coercion)?;
            let european = if prepared.len() >= 3 {
                coerce_prepared_to_number(&prepared[2])
                    .map_err(|_| DateValueFamilyError::Coercion)?
                    != 0.0
            } else {
                false
            };
            Ok(match days360_kernel(start, end, european) {
                Ok(value) => EvalValue::Number(value),
                Err(err) => map_family_err_to_eval(err),
            })
        },
        |_| DateValueFamilyError::Coercion,
    )
}

pub fn eval_datedif_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, DateValueFamilyError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared: &[PreparedArgValue]| {
            if !DATEDIF_META.arity.accepts(prepared.len()) {
                return Ok(map_family_err_to_eval(prep_len_error(
                    &DATEDIF_META,
                    prepared.len(),
                )));
            }
            let start = coerce_prepared_to_date_serial(&prepared[0])?;
            let end = coerce_prepared_to_date_serial(&prepared[1])?;
            let unit_text = coerce_prepared_to_text(&prepared[2])
                .map_err(|_| DateValueFamilyError::Coercion)?;
            let unit = parse_datedif_unit(&unit_text.to_string_lossy())?;
            Ok(match datedif_kernel(start, end, unit) {
                Ok(value) => EvalValue::Number(value),
                Err(err) => map_family_err_to_eval(err),
            })
        },
        |_| DateValueFamilyError::Coercion,
    )
}

pub fn map_date_value_family_error_to_ws(error: &DateValueFamilyError) -> WorksheetErrorCode {
    match error {
        DateValueFamilyError::Arity { .. } => WorksheetErrorCode::Value,
        DateValueFamilyError::Coercion | DateValueFamilyError::Value => WorksheetErrorCode::Value,
        DateValueFamilyError::Num => WorksheetErrorCode::Num,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::{RefResolutionError, ReferenceResolver, ResolverCapabilities};
    use crate::value::ReferenceLike;

    fn assert_close(actual: f64, expected: f64) {
        let delta = (actual - expected).abs();
        assert!(
            delta < 1.0e-12,
            "expected {expected}, got {actual}, delta {delta}"
        );
    }

    fn serial(year: i64, month: i64, day: i64) -> f64 {
        excel_serial_from_ymd_1900(year, month, day).unwrap()
    }

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
    fn datevalue_matches_seed_rows() {
        assert_eq!(datevalue_kernel("2024-02-03"), Ok(45325.0));
        assert_eq!(datevalue_kernel("2024-02-03 6:35 AM"), Ok(45325.0));
        assert_eq!(datevalue_kernel("6:35 AM"), Ok(0.0));
        assert_eq!(datevalue_kernel("22-Aug-2008"), Ok(39682.0));
        assert_eq!(
            datevalue_kernel("1/2/2024"),
            Err(DateValueFamilyError::Value)
        );
    }

    #[test]
    fn timevalue_matches_seed_rows() {
        assert_close(timevalue_kernel("2:24 AM").unwrap(), 0.1);
        assert_close(
            timevalue_kernel("22-Aug-2008 6:35 AM").unwrap(),
            0.2743055555555556,
        );
        assert_eq!(timevalue_kernel("2024-02-03"), Ok(0.0));
        assert_eq!(timevalue_kernel("22-Aug-2008"), Ok(0.0));
        assert_eq!(
            timevalue_kernel("1/2/2024 6:35 AM"),
            Err(DateValueFamilyError::Value)
        );
    }

    #[test]
    fn days360_matches_seed_rows() {
        assert_eq!(
            days360_kernel(serial(2011, 1, 30), serial(2011, 2, 1), false),
            Ok(1.0)
        );
        assert_eq!(
            days360_kernel(serial(2011, 1, 1), serial(2011, 12, 31), false),
            Ok(360.0)
        );
        assert_eq!(
            days360_kernel(serial(2011, 1, 31), serial(2011, 2, 28), false),
            Ok(28.0)
        );
        assert_eq!(
            days360_kernel(serial(2011, 1, 31), serial(2011, 2, 28), true),
            Ok(28.0)
        );
        assert_eq!(
            days360_kernel(serial(2024, 2, 29), serial(2024, 3, 31), false),
            Ok(30.0)
        );
        assert_eq!(
            days360_kernel(serial(2024, 2, 29), serial(2024, 3, 31), true),
            Ok(31.0)
        );
        assert_eq!(
            days360_kernel(serial(2011, 2, 28), serial(2011, 3, 31), false),
            Ok(30.0)
        );
        assert_eq!(
            days360_kernel(serial(2011, 2, 28), serial(2011, 3, 31), true),
            Ok(32.0)
        );
    }

    #[test]
    fn datedif_matches_seed_rows() {
        assert_eq!(
            datedif_kernel(serial(2001, 1, 1), serial(2003, 1, 1), DateDiffUnit::Y),
            Ok(2.0)
        );
        assert_eq!(
            datedif_kernel(serial(2001, 6, 1), serial(2002, 8, 15), DateDiffUnit::D),
            Ok(440.0)
        );
        assert_eq!(
            datedif_kernel(serial(2001, 6, 1), serial(2002, 8, 15), DateDiffUnit::Yd),
            Ok(75.0)
        );
        assert_eq!(
            datedif_kernel(serial(2001, 6, 1), serial(2002, 8, 15), DateDiffUnit::M),
            Ok(14.0)
        );
        assert_eq!(
            datedif_kernel(serial(2001, 6, 1), serial(2002, 8, 15), DateDiffUnit::Ym),
            Ok(2.0)
        );
        assert_eq!(
            datedif_kernel(serial(2001, 1, 31), serial(2001, 2, 28), DateDiffUnit::Md),
            Ok(28.0)
        );
        assert_eq!(
            datedif_kernel(serial(2001, 1, 31), serial(2001, 3, 1), DateDiffUnit::Md),
            Ok(-2.0)
        );
        assert_eq!(
            datedif_kernel(serial(2001, 1, 30), serial(2001, 3, 1), DateDiffUnit::Md),
            Ok(-1.0)
        );
        assert_eq!(
            datedif_kernel(serial(2000, 2, 29), serial(2001, 2, 28), DateDiffUnit::Y),
            Ok(0.0)
        );
        assert_eq!(
            datedif_kernel(serial(2000, 2, 29), serial(2001, 2, 28), DateDiffUnit::Ym),
            Ok(11.0)
        );
        assert_eq!(
            datedif_kernel(serial(2000, 2, 29), serial(2001, 2, 28), DateDiffUnit::Md),
            Ok(30.0)
        );
    }

    #[test]
    fn datedif_rejects_invalid_order_and_unit() {
        assert_eq!(
            datedif_kernel(serial(2001, 2, 28), serial(2001, 1, 31), DateDiffUnit::D),
            Err(DateValueFamilyError::Num)
        );
        assert_eq!(parse_datedif_unit("Q"), Err(DateValueFamilyError::Num));
    }

    #[test]
    fn eval_datedif_surface_accepts_iso_date_text_args() {
        let got = eval_datedif_surface(
            &[
                CallArgValue::Eval(EvalValue::Text(
                    crate::value::ExcelText::from_interop_assignment("2020-01-15"),
                )),
                CallArgValue::Eval(EvalValue::Text(
                    crate::value::ExcelText::from_interop_assignment("2024-03-20"),
                )),
                CallArgValue::Eval(EvalValue::Text(
                    crate::value::ExcelText::from_interop_assignment("Y"),
                )),
            ],
            &NoResolver,
        );
        assert_eq!(got, Ok(EvalValue::Number(4.0)));
    }

    #[test]
    fn eval_datedif_surface_keeps_direct_serial_control() {
        let got = eval_datedif_surface(
            &[
                CallArgValue::Eval(EvalValue::Number(serial(2020, 1, 15))),
                CallArgValue::Eval(EvalValue::Number(serial(2024, 3, 20))),
                CallArgValue::Eval(EvalValue::Text(
                    crate::value::ExcelText::from_interop_assignment("Y"),
                )),
            ],
            &NoResolver,
        );
        assert_eq!(got, Ok(EvalValue::Number(4.0)));
    }

    #[test]
    fn eval_datedif_surface_rejects_slash_date_text_args() {
        let got = eval_datedif_surface(
            &[
                CallArgValue::Eval(EvalValue::Text(
                    crate::value::ExcelText::from_interop_assignment("1/15/2020"),
                )),
                CallArgValue::Eval(EvalValue::Text(
                    crate::value::ExcelText::from_interop_assignment("3/20/2024"),
                )),
                CallArgValue::Eval(EvalValue::Text(
                    crate::value::ExcelText::from_interop_assignment("Y"),
                )),
            ],
            &NoResolver,
        );
        assert_eq!(got, Err(DateValueFamilyError::Value));
    }
}

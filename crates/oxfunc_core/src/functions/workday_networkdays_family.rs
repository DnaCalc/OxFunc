use crate::coercion::CoercionError;
use crate::function::{
    Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile, FunctionMeta,
    HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    BroadcastPreparedGroup, expand_aggregate_arg, expand_prepared_broadcast_grid,
    prepare_arg_values_only,
};
use crate::functions::aggregate_common::average_argument_value;
use crate::resolver::ReferenceSystemProvider;
use crate::value::CalcValue;
use crate::value::{CalcArray, CoreValue, WorksheetErrorCode};
use std::collections::BTreeSet;

const MAX_EXCEL_1900_SERIAL: i64 = 2_958_465;

const WORKDAY_NETWORKDAYS_BASE_META: FunctionMeta = function_spec! {
    function_id: "FUNC.WORKDAY_NETWORKDAYS_BASE",
    arity: Arity::exact(1),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
};

pub const WORKDAY_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.WORKDAY",
    arity: Arity { min: 2, max: 3 },
    ..WORKDAY_NETWORKDAYS_BASE_META
};

pub const WORKDAY_INTL_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.WORKDAY.INTL",
    arity: Arity { min: 2, max: 4 },
    ..WORKDAY_NETWORKDAYS_BASE_META
};

pub const NETWORKDAYS_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.NETWORKDAYS",
    arity: Arity { min: 2, max: 3 },
    ..WORKDAY_NETWORKDAYS_BASE_META
};

pub const NETWORKDAYS_INTL_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.NETWORKDAYS.INTL",
    arity: Arity { min: 2, max: 4 },
    ..WORKDAY_NETWORKDAYS_BASE_META
};

#[derive(Debug, Clone, PartialEq)]
pub enum WorkdayNetworkdaysEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
    Domain(WorksheetErrorCode),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WeekendParseMode {
    WorkdayIntl,
    NetworkdaysIntl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WeekendMask {
    monday_first: [bool; 7],
}

impl WeekendMask {
    pub const fn saturday_sunday() -> Self {
        Self {
            monday_first: [false, false, false, false, false, true, true],
        }
    }

    pub const fn all_days() -> Self {
        Self {
            monday_first: [true, true, true, true, true, true, true],
        }
    }

    pub fn from_number(code: i64) -> Option<Self> {
        let monday_first = match code {
            1 => [false, false, false, false, false, true, true],
            2 => [true, false, false, false, false, false, true],
            3 => [true, true, false, false, false, false, false],
            4 => [false, true, true, false, false, false, false],
            5 => [false, false, true, true, false, false, false],
            6 => [false, false, false, true, true, false, false],
            7 => [false, false, false, false, true, true, false],
            11 => [false, false, false, false, false, false, true],
            12 => [true, false, false, false, false, false, false],
            13 => [false, true, false, false, false, false, false],
            14 => [false, false, true, false, false, false, false],
            15 => [false, false, false, true, false, false, false],
            16 => [false, false, false, false, true, false, false],
            17 => [false, false, false, false, false, true, false],
            _ => return None,
        };
        Some(Self { monday_first })
    }

    pub fn from_mask_text(text: &str) -> Result<Self, WorksheetErrorCode> {
        if text.len() != 7 || !text.chars().all(|ch| ch == '0' || ch == '1') {
            return Err(WorksheetErrorCode::Value);
        }
        let mut monday_first = [false; 7];
        for (idx, ch) in text.chars().enumerate() {
            monday_first[idx] = ch == '1';
        }
        Ok(Self { monday_first })
    }

    pub fn is_all_days_weekend(&self) -> bool {
        self.monday_first.iter().all(|bit| *bit)
    }

    fn contains_serial(&self, serial: i64) -> bool {
        let sunday_one_based = weekday_sunday_one_based(serial);
        let monday_idx = ((sunday_one_based + 5).rem_euclid(7)) as usize;
        self.monday_first[monday_idx]
    }
}

fn weekday_sunday_one_based(serial: i64) -> i64 {
    (serial - 1).rem_euclid(7) + 1
}

fn guard_arity(meta: &FunctionMeta, args: &[CalcValue]) -> Result<(), WorkdayNetworkdaysEvalError> {
    if meta.arity.accepts(args.len()) {
        Ok(())
    } else {
        Err(WorkdayNetworkdaysEvalError::ArityMismatch {
            expected_min: meta.arity.min,
            expected_max: meta.arity.max,
            actual: args.len(),
        })
    }
}

fn optional_prepared_arg(
    args: &[CalcValue],
    index: usize,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<Option<CalcValue>, WorkdayNetworkdaysEvalError> {
    args.get(index)
        .map(|arg| {
            prepare_arg_values_only(arg, resolver).map_err(WorkdayNetworkdaysEvalError::Coercion)
        })
        .transpose()
}

struct DatePairPreparedArg {
    prepared: CalcValue,
    direct_array_lift_allowed: bool,
}

fn optional_date_pair_arg(
    args: &[CalcValue],
    index: usize,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<Option<DatePairPreparedArg>, WorkdayNetworkdaysEvalError> {
    let Some(arg) = args.get(index) else {
        return Ok(None);
    };
    let direct_array_lift_allowed = matches!(arg.core(), CoreValue::Array(_));
    let prepared =
        prepare_arg_values_only(arg, resolver).map_err(WorkdayNetworkdaysEvalError::Coercion)?;
    Ok(Some(DatePairPreparedArg {
        prepared,
        direct_array_lift_allowed,
    }))
}

fn serial_from_number(value: f64) -> Result<i64, WorksheetErrorCode> {
    if !value.is_finite() {
        return Err(WorksheetErrorCode::Num);
    }
    let serial = value.trunc() as i64;
    if !(0..=MAX_EXCEL_1900_SERIAL).contains(&serial) {
        return Err(WorksheetErrorCode::Num);
    }
    Ok(serial)
}

fn scalar_serial_arg(prepared: &CalcValue) -> Result<i64, WorkdayNetworkdaysEvalError> {
    let value = match prepared.core() {
        CoreValue::Missing | CoreValue::Empty => {
            return Err(WorkdayNetworkdaysEvalError::Domain(
                WorksheetErrorCode::Value,
            ));
        }
        _ => crate::functions::adapters::coerce_prepared_to_number(prepared)
            .map_err(WorkdayNetworkdaysEvalError::Coercion)?,
    };
    serial_from_number(value).map_err(WorkdayNetworkdaysEvalError::Domain)
}

fn scalar_truncated_i64_arg(prepared: &CalcValue) -> Result<i64, WorkdayNetworkdaysEvalError> {
    let value = match prepared.core() {
        CoreValue::Missing | CoreValue::Empty => {
            return Err(WorkdayNetworkdaysEvalError::Domain(
                WorksheetErrorCode::Value,
            ));
        }
        _ => crate::functions::adapters::coerce_prepared_to_number(prepared)
            .map_err(WorkdayNetworkdaysEvalError::Coercion)?,
    };
    if !value.is_finite() {
        return Err(WorkdayNetworkdaysEvalError::Domain(WorksheetErrorCode::Num));
    }
    Ok(value.trunc() as i64)
}

fn parse_weekend_arg(
    prepared: Option<&CalcValue>,
    mode: WeekendParseMode,
) -> Result<WeekendMask, WorkdayNetworkdaysEvalError> {
    let Some(prepared) = prepared else {
        return Ok(WeekendMask::saturday_sunday());
    };
    match prepared.core() {
        CoreValue::Missing | CoreValue::Empty => Ok(WeekendMask::saturday_sunday()),
        CoreValue::Text(text) => {
            let raw = text.to_string_lossy();
            if raw.len() == 7 && raw.chars().all(|ch| ch == '0' || ch == '1') {
                let mask = WeekendMask::from_mask_text(&raw)
                    .map_err(WorkdayNetworkdaysEvalError::Domain)?;
                if mask.is_all_days_weekend() && matches!(mode, WeekendParseMode::WorkdayIntl) {
                    return Err(WorkdayNetworkdaysEvalError::Domain(
                        WorksheetErrorCode::Value,
                    ));
                }
                return Ok(mask);
            }
            let code = scalar_truncated_i64_arg(prepared)?;
            WeekendMask::from_number(code)
                .ok_or(WorkdayNetworkdaysEvalError::Domain(WorksheetErrorCode::Num))
        }
        _ => {
            let code = scalar_truncated_i64_arg(prepared)?;
            WeekendMask::from_number(code)
                .ok_or(WorkdayNetworkdaysEvalError::Domain(WorksheetErrorCode::Num))
        }
    }
}

fn collect_holiday_serials(
    arg: Option<&CalcValue>,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<BTreeSet<i64>, WorkdayNetworkdaysEvalError> {
    let Some(arg) = arg else {
        return Ok(BTreeSet::new());
    };
    let mut serials = BTreeSet::new();
    for item in
        expand_aggregate_arg(arg, resolver).map_err(WorkdayNetworkdaysEvalError::Coercion)?
    {
        if let Some(value) =
            average_argument_value(&item).map_err(WorkdayNetworkdaysEvalError::Coercion)?
        {
            serials.insert(serial_from_number(value).map_err(WorkdayNetworkdaysEvalError::Domain)?);
        }
    }
    Ok(serials)
}

fn is_business_day(serial: i64, weekend: WeekendMask, holidays: &BTreeSet<i64>) -> bool {
    !weekend.contains_serial(serial) && !holidays.contains(&serial)
}

pub fn workday_intl_kernel(
    start_serial: f64,
    days: f64,
    weekend: WeekendMask,
    holidays: &BTreeSet<i64>,
) -> Result<f64, WorksheetErrorCode> {
    if weekend.is_all_days_weekend() {
        return Err(WorksheetErrorCode::Value);
    }
    let start = serial_from_number(start_serial)?;
    if !days.is_finite() {
        return Err(WorksheetErrorCode::Num);
    }
    let mut remaining = days.trunc() as i64;
    if remaining == 0 {
        return Ok(start as f64);
    }
    let step = if remaining > 0 { 1 } else { -1 };
    let mut current = start;
    while remaining != 0 {
        current += step;
        if !(0..=MAX_EXCEL_1900_SERIAL).contains(&current) {
            return Err(WorksheetErrorCode::Num);
        }
        if is_business_day(current, weekend, holidays) {
            remaining -= step;
        }
    }
    Ok(current as f64)
}

pub fn workday_kernel(
    start_serial: f64,
    days: f64,
    holidays: &BTreeSet<i64>,
) -> Result<f64, WorksheetErrorCode> {
    workday_intl_kernel(start_serial, days, WeekendMask::saturday_sunday(), holidays)
}

pub fn networkdays_intl_kernel(
    start_serial: f64,
    end_serial: f64,
    weekend: WeekendMask,
    holidays: &BTreeSet<i64>,
) -> Result<f64, WorksheetErrorCode> {
    let start = serial_from_number(start_serial)?;
    let end = serial_from_number(end_serial)?;
    if weekend.is_all_days_weekend() {
        return Ok(0.0);
    }
    let (lo, hi, sign) = if start <= end {
        (start, end, 1.0)
    } else {
        (end, start, -1.0)
    };
    let mut count = 0i64;
    for serial in lo..=hi {
        if is_business_day(serial, weekend, holidays) {
            count += 1;
        }
    }
    Ok(sign * count as f64)
}

pub fn networkdays_kernel(
    start_serial: f64,
    end_serial: f64,
    holidays: &BTreeSet<i64>,
) -> Result<f64, WorksheetErrorCode> {
    networkdays_intl_kernel(
        start_serial,
        end_serial,
        WeekendMask::saturday_sunday(),
        holidays,
    )
}

/// Partial array-lift for the WORKDAY/NETWORKDAYS family: broadcast the first
/// two arguments (start + days/end) elementwise (Excel spill) while the weekend
/// and holiday arguments are held fixed (they are aggregates, not broadcast).
/// Returns `None` when neither of the first two arguments is an array.
fn lift_date_pair(
    start_arg: &DatePairPreparedArg,
    second_arg: &DatePairPreparedArg,
    coerce_second: impl Fn(&CalcValue) -> Result<i64, WorkdayNetworkdaysEvalError>,
    kernel: impl Fn(f64, f64) -> Result<f64, WorkdayNetworkdaysEvalError>,
) -> Option<CalcValue> {
    let pair = [start_arg.prepared.clone(), second_arg.prepared.clone()];
    let range_derived_array_present = [&start_arg, &second_arg].iter().any(|arg| {
        matches!(arg.prepared.core(), CoreValue::Array(_)) && !arg.direct_array_lift_allowed
    });
    if range_derived_array_present {
        return None;
    }
    let (shape, groups) = expand_prepared_broadcast_grid(&pair)?;
    let cells = groups
        .into_iter()
        .map(|group| match group {
            BroadcastPreparedGroup::Values(values) => {
                let start = match scalar_serial_arg(&values[0]) {
                    Ok(s) => s,
                    Err(e) => {
                        return CalcValue::error(map_workday_networkdays_error_to_ws(&e));
                    }
                };
                let second = match coerce_second(&values[1]) {
                    Ok(s) => s,
                    Err(e) => {
                        return CalcValue::error(map_workday_networkdays_error_to_ws(&e));
                    }
                };
                match kernel(start as f64, second as f64) {
                    Ok(v) => CalcValue::number(v),
                    Err(error) => CalcValue::error(map_workday_networkdays_error_to_ws(&error)),
                }
            }
            BroadcastPreparedGroup::MissingCoordinate => CalcValue::error(WorksheetErrorCode::NA),
        })
        .collect();
    Some(CalcValue::array(
        CalcArray::new(shape, cells).expect("broadcast shape preserved"),
    ))
}

pub fn eval_workday_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, WorkdayNetworkdaysEvalError> {
    guard_arity(&WORKDAY_META, args)?;
    let start_prepared = optional_date_pair_arg(args, 0, resolver)?;
    let days_prepared = optional_date_pair_arg(args, 1, resolver)?;
    let holidays_result = collect_holiday_serials(args.get(2), resolver);
    let start_p = start_prepared.as_ref().unwrap();
    let days_p = days_prepared.as_ref().unwrap();
    let holidays_for_lift = holidays_result.clone();
    if let Some(array) = lift_date_pair(start_p, days_p, scalar_truncated_i64_arg, |s, d| {
        let holidays = holidays_for_lift.as_ref().map_err(Clone::clone)?;
        workday_kernel(s, d, holidays).map_err(WorkdayNetworkdaysEvalError::Domain)
    }) {
        return Ok(array);
    }
    let holidays = holidays_result?;
    let start = scalar_serial_arg(&start_p.prepared)?;
    let days = scalar_truncated_i64_arg(&days_p.prepared)?;
    workday_kernel(start as f64, days as f64, &holidays)
        .map(CalcValue::number)
        .map_err(WorkdayNetworkdaysEvalError::Domain)
}

pub fn eval_workday_intl_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, WorkdayNetworkdaysEvalError> {
    guard_arity(&WORKDAY_INTL_META, args)?;
    let start_prepared = optional_date_pair_arg(args, 0, resolver)?;
    let days_prepared = optional_date_pair_arg(args, 1, resolver)?;
    let weekend_prepared = optional_prepared_arg(args, 2, resolver)?;
    let weekend_result =
        parse_weekend_arg(weekend_prepared.as_ref(), WeekendParseMode::WorkdayIntl);
    let holidays_result = collect_holiday_serials(args.get(3), resolver);
    let start_p = start_prepared.as_ref().unwrap();
    let days_p = days_prepared.as_ref().unwrap();
    let weekend_for_lift = weekend_result.clone();
    let holidays_for_lift = holidays_result.clone();
    if let Some(array) = lift_date_pair(start_p, days_p, scalar_truncated_i64_arg, |s, d| {
        let weekend = *weekend_for_lift.as_ref().map_err(Clone::clone)?;
        let holidays = holidays_for_lift.as_ref().map_err(Clone::clone)?;
        workday_intl_kernel(s, d, weekend, holidays).map_err(WorkdayNetworkdaysEvalError::Domain)
    }) {
        return Ok(array);
    }
    let weekend = weekend_result?;
    let holidays = holidays_result?;
    let start = scalar_serial_arg(&start_p.prepared)?;
    let days = scalar_truncated_i64_arg(&days_p.prepared)?;
    workday_intl_kernel(start as f64, days as f64, weekend, &holidays)
        .map(CalcValue::number)
        .map_err(WorkdayNetworkdaysEvalError::Domain)
}

pub fn eval_networkdays_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, WorkdayNetworkdaysEvalError> {
    guard_arity(&NETWORKDAYS_META, args)?;
    let start_prepared = optional_date_pair_arg(args, 0, resolver)?;
    let end_prepared = optional_date_pair_arg(args, 1, resolver)?;
    let holidays_result = collect_holiday_serials(args.get(2), resolver);
    let start_p = start_prepared.as_ref().unwrap();
    let end_p = end_prepared.as_ref().unwrap();
    let holidays_for_lift = holidays_result.clone();
    if let Some(array) = lift_date_pair(start_p, end_p, scalar_serial_arg, |s, e| {
        let holidays = holidays_for_lift.as_ref().map_err(Clone::clone)?;
        networkdays_kernel(s, e, holidays).map_err(WorkdayNetworkdaysEvalError::Domain)
    }) {
        return Ok(array);
    }
    let holidays = holidays_result?;
    let start = scalar_serial_arg(&start_p.prepared)?;
    let end = scalar_serial_arg(&end_p.prepared)?;
    networkdays_kernel(start as f64, end as f64, &holidays)
        .map(CalcValue::number)
        .map_err(WorkdayNetworkdaysEvalError::Domain)
}

pub fn eval_networkdays_intl_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, WorkdayNetworkdaysEvalError> {
    guard_arity(&NETWORKDAYS_INTL_META, args)?;
    let start_prepared = optional_date_pair_arg(args, 0, resolver)?;
    let end_prepared = optional_date_pair_arg(args, 1, resolver)?;
    let weekend_prepared = optional_prepared_arg(args, 2, resolver)?;
    let weekend_result =
        parse_weekend_arg(weekend_prepared.as_ref(), WeekendParseMode::NetworkdaysIntl);
    let holidays_result = collect_holiday_serials(args.get(3), resolver);
    let start_p = start_prepared.as_ref().unwrap();
    let end_p = end_prepared.as_ref().unwrap();
    let weekend_for_lift = weekend_result.clone();
    let holidays_for_lift = holidays_result.clone();
    if let Some(array) = lift_date_pair(start_p, end_p, scalar_serial_arg, |s, e| {
        let weekend = *weekend_for_lift.as_ref().map_err(Clone::clone)?;
        let holidays = holidays_for_lift.as_ref().map_err(Clone::clone)?;
        networkdays_intl_kernel(s, e, weekend, holidays)
            .map_err(WorkdayNetworkdaysEvalError::Domain)
    }) {
        return Ok(array);
    }
    let weekend = weekend_result?;
    let holidays = holidays_result?;
    let start = scalar_serial_arg(&start_p.prepared)?;
    let end = scalar_serial_arg(&end_p.prepared)?;
    networkdays_intl_kernel(start as f64, end as f64, weekend, &holidays)
        .map(CalcValue::number)
        .map_err(WorkdayNetworkdaysEvalError::Domain)
}

pub fn map_workday_networkdays_error_to_ws(
    error: &WorkdayNetworkdaysEvalError,
) -> WorksheetErrorCode {
    match error {
        WorkdayNetworkdaysEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        WorkdayNetworkdaysEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        WorkdayNetworkdaysEvalError::Coercion(_) => WorksheetErrorCode::Value,
        WorkdayNetworkdaysEvalError::Domain(code) => *code,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::locale_format::{WorkbookDateSystem, excel_serial_from_ymd};
    use crate::resolver::{ReferenceSystemCapabilities, ReferenceSystemProvider};
    use crate::value::{CalcArray, ExcelText, ReferenceKind, ReferenceLike};
    use std::collections::BTreeMap;

    struct MockResolver {
        cells: BTreeMap<String, CalcValue>,
    }

    impl ReferenceSystemProvider for MockResolver {
        fn capabilities(&self) -> ReferenceSystemCapabilities {
            ReferenceSystemCapabilities::permissive_local()
        }

        fn dereference(
            &self,
            request: &crate::resolver::ReferenceDereferenceRequest,
        ) -> Result<CalcValue, crate::resolver::ReferenceResolutionError> {
            let reference = &request.reference;
            self.cells.get(reference.target()).cloned().ok_or_else(|| {
                crate::resolver::ReferenceResolutionError::UnresolvedReference {
                    target: reference.target().to_string(),
                }
            })
        }
    }

    fn serial(year: i64, month: i64, day: i64) -> f64 {
        excel_serial_from_ymd(WorkbookDateSystem::System1900, year, month, day).unwrap()
    }

    fn num(n: f64) -> CalcValue {
        CalcValue::number(n)
    }

    fn txt(s: &str) -> CalcValue {
        CalcValue::text(ExcelText::from_utf16_code_units(s.encode_utf16().collect()))
    }

    fn prepared_text(s: &str) -> CalcValue {
        CalcValue::text(ExcelText::from_utf16_code_units(s.encode_utf16().collect()))
    }

    fn ref_arg(target: &str) -> CalcValue {
        CalcValue::reference(ReferenceLike::new(ReferenceKind::Area, target.to_string()))
    }

    fn mixed_row(cells: Vec<CalcValue>) -> CalcValue {
        CalcValue::array(CalcArray::from_rows(vec![cells]).unwrap())
    }

    fn number_column(values: &[f64]) -> CalcValue {
        CalcValue::array(
            CalcArray::from_rows(
                values
                    .iter()
                    .map(|value| vec![CalcValue::number(*value)])
                    .collect(),
            )
            .unwrap(),
        )
    }

    fn expected_number_column(values: &[f64]) -> CalcValue {
        CalcValue::array(
            CalcArray::from_rows(
                values
                    .iter()
                    .map(|value| vec![CalcValue::number(*value)])
                    .collect(),
            )
            .unwrap(),
        )
    }

    fn expected_error_column(code: WorksheetErrorCode, len: usize) -> CalcValue {
        CalcValue::array(
            CalcArray::from_rows((0..len).map(|_| vec![CalcValue::error(code)]).collect()).unwrap(),
        )
    }

    #[test]
    fn metadata_matches_expected_shape() {
        assert_eq!(WORKDAY_META.arity.min, 2);
        assert_eq!(WORKDAY_INTL_META.arity.max, 4);
        assert_eq!(NETWORKDAYS_META.function_id, "FUNC.NETWORKDAYS");
        assert_eq!(
            NETWORKDAYS_INTL_META.surface_fec_dependency_profile,
            FecDependencyProfile::RefOnly
        );
    }

    #[test]
    fn workday_and_networkdays_default_weekend_match_baseline_examples() {
        let holidays = BTreeSet::from([serial(2024, 1, 1).trunc() as i64]);
        assert_eq!(
            workday_kernel(serial(2024, 1, 2), 5.0, &holidays),
            Ok(serial(2024, 1, 9))
        );
        assert_eq!(
            networkdays_kernel(serial(2024, 1, 1), serial(2024, 1, 10), &holidays),
            Ok(7.0)
        );
        assert_eq!(
            networkdays_kernel(serial(2024, 1, 10), serial(2024, 1, 1), &holidays),
            Ok(-7.0)
        );
    }

    #[test]
    fn intl_weekend_numbers_and_masks_are_honored() {
        let empty = BTreeSet::new();
        assert_eq!(
            workday_intl_kernel(
                serial(2012, 1, 1),
                30.0,
                WeekendMask::from_number(17).unwrap(),
                &empty,
            ),
            Ok(40944.0)
        );
        assert_eq!(
            workday_intl_kernel(
                serial(2012, 1, 1),
                90.0,
                WeekendMask::from_number(11).unwrap(),
                &empty,
            ),
            Ok(41013.0)
        );
        assert_eq!(
            networkdays_intl_kernel(
                serial(2006, 1, 1),
                serial(2006, 2, 1),
                WeekendMask::from_number(7).unwrap(),
                &BTreeSet::from([
                    serial(2006, 1, 2).trunc() as i64,
                    serial(2006, 1, 16).trunc() as i64,
                ]),
            ),
            Ok(22.0)
        );
        assert_eq!(
            networkdays_intl_kernel(
                serial(2006, 1, 1),
                serial(2006, 2, 1),
                WeekendMask::from_mask_text("0010001").unwrap(),
                &BTreeSet::from([
                    serial(2006, 1, 2).trunc() as i64,
                    serial(2006, 1, 16).trunc() as i64,
                ]),
            ),
            Ok(20.0)
        );
    }

    #[test]
    fn direct_array_date_pairs_spill_like_excel() {
        let resolver = MockResolver {
            cells: BTreeMap::new(),
        };
        assert_eq!(
            eval_workday_surface(
                &[
                    number_column(&[45292.0, 45293.0]),
                    number_column(&[1.0, 2.0]),
                ],
                &resolver,
            ),
            Ok(expected_number_column(&[45293.0, 45295.0]))
        );
        assert_eq!(
            eval_networkdays_surface(
                &[
                    number_column(&[45292.0, 45293.0]),
                    number_column(&[45297.0, 45298.0]),
                ],
                &resolver,
            ),
            Ok(expected_number_column(&[5.0, 4.0]))
        );
        assert_eq!(
            eval_workday_intl_surface(
                &[
                    number_column(&[45292.0, 45293.0]),
                    number_column(&[1.0, 2.0]),
                    num(1.0),
                ],
                &resolver,
            ),
            Ok(expected_number_column(&[45293.0, 45295.0]))
        );
        assert_eq!(
            eval_networkdays_intl_surface(
                &[
                    number_column(&[45292.0, 45293.0]),
                    number_column(&[45297.0, 45298.0]),
                    num(1.0),
                ],
                &resolver,
            ),
            Ok(expected_number_column(&[5.0, 4.0]))
        );
    }

    #[test]
    fn fixed_control_errors_repeat_over_direct_array_spill_shape() {
        let resolver = MockResolver {
            cells: BTreeMap::new(),
        };
        assert_eq!(
            eval_workday_intl_surface(
                &[
                    number_column(&[45292.0, 45293.0]),
                    number_column(&[1.0, 2.0]),
                    num(99.0),
                ],
                &resolver,
            ),
            Ok(expected_error_column(WorksheetErrorCode::Num, 2))
        );
        assert_eq!(
            eval_networkdays_intl_surface(
                &[
                    number_column(&[45292.0, 45293.0]),
                    number_column(&[45297.0, 45298.0]),
                    txt("1111111"),
                ],
                &resolver,
            ),
            Ok(expected_number_column(&[0.0, 0.0]))
        );
        assert_eq!(
            eval_workday_surface(
                &[
                    number_column(&[45292.0, 45293.0]),
                    number_column(&[1.0, 2.0]),
                    txt("bad"),
                ],
                &resolver,
            ),
            Ok(expected_error_column(WorksheetErrorCode::Value, 2))
        );
    }

    #[test]
    fn range_derived_date_arrays_do_not_use_direct_array_spill_path() {
        let resolver = MockResolver {
            cells: BTreeMap::from([
                (
                    "A1:A2".to_string(),
                    CalcValue::array(
                        CalcArray::from_rows(vec![
                            vec![CalcValue::number(45292.0)],
                            vec![CalcValue::number(45293.0)],
                        ])
                        .unwrap(),
                    ),
                ),
                (
                    "B1:B2".to_string(),
                    CalcValue::array(
                        CalcArray::from_rows(vec![
                            vec![CalcValue::number(1.0)],
                            vec![CalcValue::number(2.0)],
                        ])
                        .unwrap(),
                    ),
                ),
            ]),
        };
        let workday = eval_workday_surface(&[ref_arg("A1:A2"), ref_arg("B1:B2")], &resolver);
        assert_eq!(
            workday
                .as_ref()
                .map_err(map_workday_networkdays_error_to_ws),
            Err(WorksheetErrorCode::Value)
        );
        let networkdays =
            eval_networkdays_surface(&[ref_arg("A1:A2"), ref_arg("A1:A2")], &resolver);
        assert_eq!(
            networkdays
                .as_ref()
                .map_err(map_workday_networkdays_error_to_ws),
            Err(WorksheetErrorCode::Value)
        );
    }

    #[test]
    fn zero_days_returns_the_start_date_for_workday_variants() {
        let holidays = BTreeSet::from([serial(2024, 1, 8).trunc() as i64]);
        assert_eq!(
            workday_kernel(serial(2024, 1, 6), 0.0, &holidays),
            Ok(serial(2024, 1, 6))
        );
        assert_eq!(
            workday_intl_kernel(
                serial(2024, 1, 6),
                0.0,
                WeekendMask::from_number(1).unwrap(),
                &holidays,
            ),
            Ok(serial(2024, 1, 6))
        );
    }

    #[test]
    fn weekend_parser_distinguishes_workday_and_networkdays_all_weekend_mask() {
        let all_days = parse_weekend_arg(
            Some(&prepared_text("1111111")),
            WeekendParseMode::NetworkdaysIntl,
        );
        assert_eq!(all_days, Ok(WeekendMask::all_days()));
        assert_eq!(
            parse_weekend_arg(
                Some(&prepared_text("1111111")),
                WeekendParseMode::WorkdayIntl,
            ),
            Err(WorkdayNetworkdaysEvalError::Domain(
                WorksheetErrorCode::Value
            ))
        );
        assert_eq!(
            networkdays_intl_kernel(
                serial(2024, 1, 1),
                serial(2024, 1, 31),
                WeekendMask::all_days(),
                &BTreeSet::new(),
            ),
            Ok(0.0)
        );
    }

    #[test]
    fn surface_holiday_ranges_ignore_reference_text_but_error_on_direct_invalid_text() {
        let resolver = MockResolver {
            cells: BTreeMap::from([(
                "H1:H4".to_string(),
                mixed_row(vec![
                    CalcValue::number(serial(2024, 1, 1)),
                    CalcValue::text(ExcelText::from_utf16_code_units(
                        "x".encode_utf16().collect(),
                    )),
                    CalcValue::number(serial(2024, 1, 15)),
                    CalcValue::logical(true),
                ]),
            )]),
        };
        assert_eq!(
            eval_networkdays_surface(
                &[
                    num(serial(2024, 1, 1)),
                    num(serial(2024, 1, 16)),
                    ref_arg("H1:H4"),
                ],
                &resolver,
            ),
            Ok(CalcValue::number(10.0))
        );
        assert_eq!(
            eval_networkdays_surface(
                &[
                    num(serial(2024, 1, 1)),
                    num(serial(2024, 1, 16)),
                    txt("bad"),
                ],
                &resolver,
            ),
            Err(WorkdayNetworkdaysEvalError::Coercion(
                CoercionError::NonNumericText("bad".to_string())
            ))
        );
    }

    #[test]
    fn weekend_numeric_text_and_invalid_number_lanes_are_exercised() {
        let resolver = MockResolver {
            cells: BTreeMap::new(),
        };
        assert_eq!(
            eval_workday_intl_surface(&[num(serial(2024, 1, 1)), num(1.0), txt("2")], &resolver,),
            Ok(CalcValue::number(serial(2024, 1, 2)))
        );
        assert_eq!(
            eval_workday_intl_surface(&[num(serial(2024, 1, 1)), num(1.0), num(99.0)], &resolver,),
            Err(WorkdayNetworkdaysEvalError::Domain(WorksheetErrorCode::Num))
        );
        assert_eq!(
            eval_networkdays_intl_surface(
                &[
                    num(serial(2024, 1, 1)),
                    num(serial(2024, 1, 5)),
                    txt("0000011"),
                ],
                &resolver,
            ),
            Ok(CalcValue::number(5.0))
        );
    }

    #[test]
    fn domain_and_mapping_lanes_are_exercised() {
        let resolver = MockResolver {
            cells: BTreeMap::new(),
        };
        assert_eq!(
            eval_workday_surface(&[num(-1.0), num(1.0)], &resolver),
            Err(WorkdayNetworkdaysEvalError::Domain(WorksheetErrorCode::Num))
        );
        assert_eq!(
            eval_workday_intl_surface(&[num(serial(2024, 1, 1)), num(1.0), txt("abc")], &resolver,),
            Err(WorkdayNetworkdaysEvalError::Coercion(
                CoercionError::NonNumericText("abc".to_string())
            ))
        );
        assert_eq!(
            map_workday_networkdays_error_to_ws(&WorkdayNetworkdaysEvalError::ArityMismatch {
                expected_min: 2,
                expected_max: 4,
                actual: 1,
            }),
            WorksheetErrorCode::Value
        );
    }
}

use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{PreparedArgValue, expand_aggregate_arg, prepare_arg_values_only};
use crate::functions::aggregate_common::average_argument_value;
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};
use std::collections::BTreeSet;

const MAX_EXCEL_1900_SERIAL: i64 = 2_958_465;

const WORKDAY_NETWORKDAYS_BASE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.WORKDAY_NETWORKDAYS_BASE",
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

fn guard_arity(
    meta: &FunctionMeta,
    args: &[CallArgValue],
) -> Result<(), WorkdayNetworkdaysEvalError> {
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
    args: &[CallArgValue],
    index: usize,
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<Option<PreparedArgValue>, WorkdayNetworkdaysEvalError> {
    args.get(index)
        .map(|arg| {
            prepare_arg_values_only(arg, resolver).map_err(WorkdayNetworkdaysEvalError::Coercion)
        })
        .transpose()
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

fn scalar_serial_arg(prepared: &PreparedArgValue) -> Result<i64, WorkdayNetworkdaysEvalError> {
    let value = match prepared {
        PreparedArgValue::MissingArg | PreparedArgValue::EmptyCell => {
            return Err(WorkdayNetworkdaysEvalError::Domain(
                WorksheetErrorCode::Value,
            ));
        }
        other => crate::functions::adapters::coerce_prepared_to_number(other)
            .map_err(WorkdayNetworkdaysEvalError::Coercion)?,
    };
    serial_from_number(value).map_err(WorkdayNetworkdaysEvalError::Domain)
}

fn scalar_truncated_i64_arg(
    prepared: &PreparedArgValue,
) -> Result<i64, WorkdayNetworkdaysEvalError> {
    let value = match prepared {
        PreparedArgValue::MissingArg | PreparedArgValue::EmptyCell => {
            return Err(WorkdayNetworkdaysEvalError::Domain(
                WorksheetErrorCode::Value,
            ));
        }
        other => crate::functions::adapters::coerce_prepared_to_number(other)
            .map_err(WorkdayNetworkdaysEvalError::Coercion)?,
    };
    if !value.is_finite() {
        return Err(WorkdayNetworkdaysEvalError::Domain(WorksheetErrorCode::Num));
    }
    Ok(value.trunc() as i64)
}

fn parse_weekend_arg(
    prepared: Option<&PreparedArgValue>,
    mode: WeekendParseMode,
) -> Result<WeekendMask, WorkdayNetworkdaysEvalError> {
    let Some(prepared) = prepared else {
        return Ok(WeekendMask::saturday_sunday());
    };
    match prepared {
        PreparedArgValue::MissingArg | PreparedArgValue::EmptyCell => {
            Ok(WeekendMask::saturday_sunday())
        }
        PreparedArgValue::Eval(EvalValue::Text(text)) => {
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
    arg: Option<&CallArgValue>,
    resolver: &(impl ReferenceResolver + ?Sized),
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

pub fn eval_workday_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, WorkdayNetworkdaysEvalError> {
    guard_arity(&WORKDAY_META, args)?;
    let start_prepared = optional_prepared_arg(args, 0, resolver)?;
    let days_prepared = optional_prepared_arg(args, 1, resolver)?;
    let start = scalar_serial_arg(start_prepared.as_ref().unwrap())?;
    let days = scalar_truncated_i64_arg(days_prepared.as_ref().unwrap())?;
    let holidays = collect_holiday_serials(args.get(2), resolver)?;
    workday_kernel(start as f64, days as f64, &holidays)
        .map(EvalValue::Number)
        .map_err(WorkdayNetworkdaysEvalError::Domain)
}

pub fn eval_workday_intl_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, WorkdayNetworkdaysEvalError> {
    guard_arity(&WORKDAY_INTL_META, args)?;
    let start_prepared = optional_prepared_arg(args, 0, resolver)?;
    let days_prepared = optional_prepared_arg(args, 1, resolver)?;
    let weekend_prepared = optional_prepared_arg(args, 2, resolver)?;
    let start = scalar_serial_arg(start_prepared.as_ref().unwrap())?;
    let days = scalar_truncated_i64_arg(days_prepared.as_ref().unwrap())?;
    let weekend = parse_weekend_arg(weekend_prepared.as_ref(), WeekendParseMode::WorkdayIntl)?;
    let holidays = collect_holiday_serials(args.get(3), resolver)?;
    workday_intl_kernel(start as f64, days as f64, weekend, &holidays)
        .map(EvalValue::Number)
        .map_err(WorkdayNetworkdaysEvalError::Domain)
}

pub fn eval_networkdays_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, WorkdayNetworkdaysEvalError> {
    guard_arity(&NETWORKDAYS_META, args)?;
    let start_prepared = optional_prepared_arg(args, 0, resolver)?;
    let end_prepared = optional_prepared_arg(args, 1, resolver)?;
    let start = scalar_serial_arg(start_prepared.as_ref().unwrap())?;
    let end = scalar_serial_arg(end_prepared.as_ref().unwrap())?;
    let holidays = collect_holiday_serials(args.get(2), resolver)?;
    networkdays_kernel(start as f64, end as f64, &holidays)
        .map(EvalValue::Number)
        .map_err(WorkdayNetworkdaysEvalError::Domain)
}

pub fn eval_networkdays_intl_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, WorkdayNetworkdaysEvalError> {
    guard_arity(&NETWORKDAYS_INTL_META, args)?;
    let start_prepared = optional_prepared_arg(args, 0, resolver)?;
    let end_prepared = optional_prepared_arg(args, 1, resolver)?;
    let weekend_prepared = optional_prepared_arg(args, 2, resolver)?;
    let start = scalar_serial_arg(start_prepared.as_ref().unwrap())?;
    let end = scalar_serial_arg(end_prepared.as_ref().unwrap())?;
    let weekend = parse_weekend_arg(weekend_prepared.as_ref(), WeekendParseMode::NetworkdaysIntl)?;
    let holidays = collect_holiday_serials(args.get(3), resolver)?;
    networkdays_intl_kernel(start as f64, end as f64, weekend, &holidays)
        .map(EvalValue::Number)
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
    use crate::resolver::{RefResolutionError, ReferenceResolver, ResolverCapabilities};
    use crate::value::{ArrayCellValue, EvalArray, ExcelText, ReferenceKind, ReferenceLike};
    use std::collections::BTreeMap;

    struct MockResolver {
        cells: BTreeMap<String, EvalValue>,
    }

    impl ReferenceResolver for MockResolver {
        fn capabilities(&self) -> ResolverCapabilities {
            ResolverCapabilities::permissive_local()
        }

        fn resolve_reference(
            &self,
            reference: &ReferenceLike,
        ) -> Result<EvalValue, RefResolutionError> {
            self.cells.get(&reference.target).cloned().ok_or_else(|| {
                RefResolutionError::UnresolvedReference {
                    target: reference.target.clone(),
                }
            })
        }
    }

    fn serial(year: i64, month: i64, day: i64) -> f64 {
        excel_serial_from_ymd(WorkbookDateSystem::System1900, year, month, day).unwrap()
    }

    fn num(n: f64) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Number(n))
    }

    fn txt(s: &str) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
            s.encode_utf16().collect(),
        )))
    }

    fn prepared_text(s: &str) -> PreparedArgValue {
        PreparedArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
            s.encode_utf16().collect(),
        )))
    }

    fn ref_arg(target: &str) -> CallArgValue {
        CallArgValue::Reference(ReferenceLike {
            kind: ReferenceKind::Area,
            target: target.to_string(),
        })
    }

    fn mixed_row(cells: Vec<ArrayCellValue>) -> EvalValue {
        EvalValue::Array(EvalArray::from_rows(vec![cells]).unwrap())
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
                    ArrayCellValue::Number(serial(2024, 1, 1)),
                    ArrayCellValue::Text(ExcelText::from_utf16_code_units(
                        "x".encode_utf16().collect(),
                    )),
                    ArrayCellValue::Number(serial(2024, 1, 15)),
                    ArrayCellValue::Logical(true),
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
            Ok(EvalValue::Number(10.0))
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
            Ok(EvalValue::Number(serial(2024, 1, 2)))
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
            Ok(EvalValue::Number(5.0))
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

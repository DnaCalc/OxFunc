use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    PreparedArgValue, coerce_prepared_to_number, run_values_only_prepared,
};
use crate::resolver::ReferenceResolver;
use crate::value::{ArrayCellValue, CallArgValue, EvalArray, EvalValue, WorksheetErrorCode};

const DATE_PART_BASE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.DATE_PART_BASE",
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

pub const DAY_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.DAY",
    ..DATE_PART_BASE_META
};

pub const MONTH_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.MONTH",
    ..DATE_PART_BASE_META
};

pub const YEAR_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.YEAR",
    ..DATE_PART_BASE_META
};

pub const DAYS_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.DAYS",
    arity: Arity::exact(2),
    ..DATE_PART_BASE_META
};

pub const HOUR_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.HOUR",
    ..DATE_PART_BASE_META
};

pub const MINUTE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.MINUTE",
    ..DATE_PART_BASE_META
};

pub const SECOND_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.SECOND",
    ..DATE_PART_BASE_META
};

pub const TIME_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.TIME",
    arity: Arity::exact(3),
    ..DATE_PART_BASE_META
};

#[derive(Debug, Clone, PartialEq)]
pub enum DatePartsEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
    Domain(WorksheetErrorCode),
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

fn coerce_serial_arg(arg: &PreparedArgValue) -> Result<f64, DatePartsEvalError> {
    match arg {
        PreparedArgValue::MissingArg | PreparedArgValue::EmptyCell => Ok(0.0),
        _ => coerce_prepared_to_number(arg).map_err(DatePartsEvalError::Coercion),
    }
}

fn serial_to_ymd(serial: f64) -> Result<(i64, i64, i64), WorksheetErrorCode> {
    if !serial.is_finite() {
        return Err(WorksheetErrorCode::Num);
    }
    let serial = serial.trunc() as i64;
    if serial < 0 {
        return Err(WorksheetErrorCode::Num);
    }
    if serial == 0 {
        return Ok((1900, 1, 0));
    }
    if serial == 60 {
        return Ok((1900, 2, 29));
    }
    let adjusted = if serial > 60 { serial - 1 } else { serial };
    let base = days_from_civil(1899, 12, 31);
    Ok(civil_from_days(base + adjusted))
}

pub fn day_kernel(serial: f64) -> Result<f64, WorksheetErrorCode> {
    serial_to_ymd(serial).map(|(_, _, day)| day as f64)
}

pub fn month_kernel(serial: f64) -> Result<f64, WorksheetErrorCode> {
    serial_to_ymd(serial).map(|(_, month, _)| month as f64)
}

pub fn year_kernel(serial: f64) -> Result<f64, WorksheetErrorCode> {
    serial_to_ymd(serial).map(|(year, _, _)| year as f64)
}

pub fn days_kernel(end_serial: f64, start_serial: f64) -> Result<f64, WorksheetErrorCode> {
    if !end_serial.is_finite() || !start_serial.is_finite() {
        return Err(WorksheetErrorCode::Num);
    }
    let end_serial = end_serial.trunc();
    let start_serial = start_serial.trunc();
    if end_serial < 0.0 || start_serial < 0.0 {
        return Err(WorksheetErrorCode::Num);
    }
    Ok(end_serial - start_serial)
}

fn serial_fraction_to_hms(serial: f64) -> Result<(f64, f64, f64), WorksheetErrorCode> {
    if !serial.is_finite() {
        return Err(WorksheetErrorCode::Num);
    }
    if serial < 0.0 {
        return Err(WorksheetErrorCode::Num);
    }

    let fractional = serial - serial.floor();
    let total_seconds = (fractional * 86_400.0).round().rem_euclid(86_400.0);
    let hour = (total_seconds / 3600.0).floor();
    let minute = ((total_seconds - hour * 3600.0) / 60.0).floor();
    let second = total_seconds - hour * 3600.0 - minute * 60.0;
    Ok((hour, minute, second))
}

pub fn hour_kernel(serial: f64) -> Result<f64, WorksheetErrorCode> {
    serial_fraction_to_hms(serial).map(|(hour, _, _)| hour)
}

pub fn minute_kernel(serial: f64) -> Result<f64, WorksheetErrorCode> {
    serial_fraction_to_hms(serial).map(|(_, minute, _)| minute)
}

pub fn second_kernel(serial: f64) -> Result<f64, WorksheetErrorCode> {
    serial_fraction_to_hms(serial).map(|(_, _, second)| second)
}

fn truncate_time_component(arg: &PreparedArgValue) -> Result<f64, DatePartsEvalError> {
    match arg {
        PreparedArgValue::MissingArg | PreparedArgValue::EmptyCell => Ok(0.0),
        _ => coerce_prepared_to_number(arg)
            .map(|n| n.trunc())
            .map_err(DatePartsEvalError::Coercion),
    }
}

pub fn time_kernel(hour: f64, minute: f64, second: f64) -> Result<f64, WorksheetErrorCode> {
    if !hour.is_finite() || !minute.is_finite() || !second.is_finite() {
        return Err(WorksheetErrorCode::Num);
    }
    if hour.abs() > 32767.0 || minute.abs() > 32767.0 || second.abs() > 32767.0 {
        return Err(WorksheetErrorCode::Num);
    }

    let total_seconds = hour * 3600.0 + minute * 60.0 + second;
    if total_seconds < 0.0 {
        return Err(WorksheetErrorCode::Num);
    }

    Ok((total_seconds / 86_400.0).rem_euclid(1.0))
}

fn eval_date_part_unary_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
    meta: &FunctionMeta,
    kernel: fn(f64) -> Result<f64, WorksheetErrorCode>,
) -> Result<EvalValue, DatePartsEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| {
            if !meta.arity.accepts(prepared.len()) {
                return Err(DatePartsEvalError::ArityMismatch {
                    expected_min: meta.arity.min,
                    expected_max: meta.arity.max,
                    actual: prepared.len(),
                });
            }
            match &prepared[0] {
                PreparedArgValue::Eval(EvalValue::Array(array)) => {
                    let cells = array
                        .iter_row_major()
                        .map(|cell| {
                            let serial = match cell {
                                ArrayCellValue::Number(n) => *n,
                                ArrayCellValue::Text(text) => coerce_prepared_to_number(
                                    &PreparedArgValue::Eval(EvalValue::Text(text.clone())),
                                )
                                .map_err(DatePartsEvalError::Coercion)?,
                                ArrayCellValue::Logical(value) => {
                                    if *value { 1.0 } else { 0.0 }
                                }
                                ArrayCellValue::Error(code) => {
                                    return Ok(ArrayCellValue::Error(*code));
                                }
                                ArrayCellValue::EmptyCell => 0.0,
                            };
                            kernel(serial)
                                .map(ArrayCellValue::Number)
                                .map_err(DatePartsEvalError::Domain)
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    Ok(EvalValue::Array(
                        EvalArray::new(array.shape(), cells)
                            .expect("date-part array lift preserves input shape"),
                    ))
                }
                _ => {
                    let serial = coerce_serial_arg(&prepared[0])?;
                    kernel(serial)
                        .map(EvalValue::Number)
                        .map_err(DatePartsEvalError::Domain)
                }
            }
        },
        DatePartsEvalError::Coercion,
    )
}

pub fn eval_day_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DatePartsEvalError> {
    eval_date_part_unary_surface(args, resolver, &DAY_META, day_kernel)
}

pub fn eval_month_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DatePartsEvalError> {
    eval_date_part_unary_surface(args, resolver, &MONTH_META, month_kernel)
}

pub fn eval_year_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DatePartsEvalError> {
    eval_date_part_unary_surface(args, resolver, &YEAR_META, year_kernel)
}

pub fn eval_days_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DatePartsEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| {
            if !DAYS_META.arity.accepts(prepared.len()) {
                return Err(DatePartsEvalError::ArityMismatch {
                    expected_min: DAYS_META.arity.min,
                    expected_max: DAYS_META.arity.max,
                    actual: prepared.len(),
                });
            }
            let end_serial = coerce_serial_arg(&prepared[0])?;
            let start_serial = coerce_serial_arg(&prepared[1])?;
            days_kernel(end_serial, start_serial)
                .map(EvalValue::Number)
                .map_err(DatePartsEvalError::Domain)
        },
        DatePartsEvalError::Coercion,
    )
}

pub fn eval_hour_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DatePartsEvalError> {
    eval_date_part_unary_surface(args, resolver, &HOUR_META, hour_kernel)
}

pub fn eval_minute_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DatePartsEvalError> {
    eval_date_part_unary_surface(args, resolver, &MINUTE_META, minute_kernel)
}

pub fn eval_second_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DatePartsEvalError> {
    eval_date_part_unary_surface(args, resolver, &SECOND_META, second_kernel)
}

pub fn eval_time_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DatePartsEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| {
            if !TIME_META.arity.accepts(prepared.len()) {
                return Err(DatePartsEvalError::ArityMismatch {
                    expected_min: TIME_META.arity.min,
                    expected_max: TIME_META.arity.max,
                    actual: prepared.len(),
                });
            }
            let hour = truncate_time_component(&prepared[0])?;
            let minute = truncate_time_component(&prepared[1])?;
            let second = truncate_time_component(&prepared[2])?;
            time_kernel(hour, minute, second)
                .map(EvalValue::Number)
                .map_err(DatePartsEvalError::Domain)
        },
        DatePartsEvalError::Coercion,
    )
}

pub fn map_date_parts_error_to_ws(e: &DatePartsEvalError) -> WorksheetErrorCode {
    match e {
        DatePartsEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        DatePartsEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        DatePartsEvalError::Coercion(_) => WorksheetErrorCode::Value,
        DatePartsEvalError::Domain(code) => *code,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::{RefResolutionError, ResolverCapabilities};
    use crate::value::{ArrayCellValue, EvalArray, ExcelText, ReferenceLike};

    struct NoResolver;

    fn txt(s: &str) -> ExcelText {
        ExcelText::from_utf16_code_units(s.encode_utf16().collect())
    }

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
    fn serial_parts_match_excel_probe_rows() {
        assert_eq!(day_kernel(1.0), Ok(1.0));
        assert_eq!(month_kernel(1.0), Ok(1.0));
        assert_eq!(year_kernel(1.0), Ok(1900.0));
        assert_eq!(day_kernel(60.0), Ok(29.0));
        assert_eq!(month_kernel(60.0), Ok(2.0));
        assert_eq!(year_kernel(60.0), Ok(1900.0));
        assert_eq!(day_kernel(61.9), Ok(1.0));
        assert_eq!(month_kernel(61.9), Ok(3.0));
        assert_eq!(year_kernel(61.9), Ok(1900.0));
        assert_eq!(day_kernel(-1.0), Err(WorksheetErrorCode::Num));
    }

    #[test]
    fn days_kernel_matches_excel_probe_rows() {
        assert_eq!(days_kernel(61.0, 60.0), Ok(1.0));
        assert_eq!(days_kernel(1.0, 2.0), Ok(-1.0));
        assert_eq!(days_kernel(61.9, 60.1), Ok(1.0));
        assert_eq!(days_kernel(1.0, -1.0), Err(WorksheetErrorCode::Num));
    }

    #[test]
    fn blank_and_missing_inputs_coerce_to_zero_like_excel() {
        assert_eq!(
            eval_day_surface(&[CallArgValue::EmptyCell], &NoResolver),
            Ok(EvalValue::Number(0.0))
        );
        assert_eq!(
            eval_days_surface(
                &[
                    CallArgValue::MissingArg,
                    CallArgValue::Eval(EvalValue::Number(1.0))
                ],
                &NoResolver
            ),
            Ok(EvalValue::Number(-1.0))
        );
        assert_eq!(
            eval_days_surface(
                &[
                    CallArgValue::Eval(EvalValue::Number(1.0)),
                    CallArgValue::EmptyCell
                ],
                &NoResolver
            ),
            Ok(EvalValue::Number(1.0))
        );
    }

    #[test]
    fn logical_inputs_follow_numeric_coercion() {
        assert_eq!(
            eval_day_surface(&[CallArgValue::Eval(EvalValue::Logical(true))], &NoResolver),
            Ok(EvalValue::Number(1.0))
        );
        assert_eq!(
            eval_days_surface(
                &[
                    CallArgValue::Eval(EvalValue::Logical(true)),
                    CallArgValue::Eval(EvalValue::Number(1.0)),
                ],
                &NoResolver,
            ),
            Ok(EvalValue::Number(0.0))
        );
    }

    #[test]
    fn month_surface_lifts_array_inputs_elementwise() {
        let got = eval_month_surface(
            &[CallArgValue::Eval(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Number(45291.0),
                    ArrayCellValue::Number(45292.0),
                    ArrayCellValue::Number(45322.0),
                ]])
                .unwrap(),
            ))],
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Number(12.0),
                    ArrayCellValue::Number(1.0),
                    ArrayCellValue::Number(1.0),
                ]])
                .unwrap()
            ))
        );
    }

    #[test]
    fn time_parts_match_excel_probe_rows() {
        assert_eq!(hour_kernel(0.0), Ok(0.0));
        assert_eq!(minute_kernel(0.0), Ok(0.0));
        assert_eq!(second_kernel(0.0), Ok(0.0));
        assert_eq!(hour_kernel(1.9), Ok(21.0));
        assert_eq!(minute_kernel(1.9), Ok(36.0));
        assert_eq!(second_kernel(1.9), Ok(0.0));
        assert_eq!(hour_kernel(-0.1), Err(WorksheetErrorCode::Num));
    }

    #[test]
    fn time_kernel_matches_excel_probe_rows() {
        assert_eq!(time_kernel(1.0, 2.0, 3.0), Ok(0.043090277777777776));
        assert_eq!(time_kernel(27.0, 0.0, 0.0), Ok(0.125));
        assert_eq!(time_kernel(0.0, 120.0, 0.0), Ok(0.08333333333333333));
        assert_eq!(time_kernel(0.0, 0.0, 120.0), Ok(0.001388888888888889));
        assert_eq!(time_kernel(-1.0, 60.0, 0.0), Ok(0.0));
        assert_eq!(time_kernel(-1.0, 0.0, 1.0), Err(WorksheetErrorCode::Num));
        assert_eq!(time_kernel(32768.0, 0.0, 0.0), Err(WorksheetErrorCode::Num));
    }

    #[test]
    fn time_surface_blanks_missing_text_and_logicals_match_excel() {
        assert_eq!(
            eval_time_surface(
                &[
                    CallArgValue::MissingArg,
                    CallArgValue::Eval(EvalValue::Number(2.0)),
                    CallArgValue::Eval(EvalValue::Number(3.0)),
                ],
                &NoResolver,
            ),
            Ok(EvalValue::Number(0.0014236111111111112))
        );
        assert_eq!(
            eval_time_surface(
                &[
                    CallArgValue::Eval(EvalValue::Logical(true)),
                    CallArgValue::Eval(EvalValue::Number(2.0)),
                    CallArgValue::Eval(EvalValue::Number(3.0)),
                ],
                &NoResolver,
            ),
            Ok(EvalValue::Number(0.043090277777777776))
        );
        assert_eq!(
            eval_time_surface(
                &[
                    CallArgValue::Eval(EvalValue::Text(txt("1"))),
                    CallArgValue::Eval(EvalValue::Text(txt("2"))),
                    CallArgValue::Eval(EvalValue::Text(txt("3"))),
                ],
                &NoResolver,
            ),
            Ok(EvalValue::Number(0.043090277777777776))
        );
    }
}

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

const DISCOUNT_BILL_YEARFRAC_META_BASE: FunctionMeta = FunctionMeta {
    function_id: "FUNC.DISCOUNT_BILL_YEARFRAC_BASE",
    arity: Arity::exact(3),
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

pub const DISC_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.DISC",
    arity: Arity { min: 4, max: 5 },
    ..DISCOUNT_BILL_YEARFRAC_META_BASE
};
pub const INTRATE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.INTRATE",
    arity: Arity { min: 4, max: 5 },
    ..DISCOUNT_BILL_YEARFRAC_META_BASE
};
pub const RECEIVED_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.RECEIVED",
    arity: Arity { min: 4, max: 5 },
    ..DISCOUNT_BILL_YEARFRAC_META_BASE
};
pub const PRICEDISC_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.PRICEDISC",
    arity: Arity { min: 4, max: 5 },
    ..DISCOUNT_BILL_YEARFRAC_META_BASE
};
pub const TBILLEQ_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.TBILLEQ",
    ..DISCOUNT_BILL_YEARFRAC_META_BASE
};
pub const TBILLPRICE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.TBILLPRICE",
    ..DISCOUNT_BILL_YEARFRAC_META_BASE
};
pub const TBILLYIELD_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.TBILLYIELD",
    ..DISCOUNT_BILL_YEARFRAC_META_BASE
};
pub const YEARFRAC_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.YEARFRAC",
    arity: Arity { min: 2, max: 3 },
    ..DISCOUNT_BILL_YEARFRAC_META_BASE
};

#[derive(Debug, Clone, PartialEq)]
pub enum DiscountBillYearfracEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
    Domain(WorksheetErrorCode),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DayCountBasis {
    Us30_360,
    ActualActual,
    Actual360,
    Actual365,
    European30_360,
}

fn arity_error(meta: &FunctionMeta, actual: usize) -> DiscountBillYearfracEvalError {
    DiscountBillYearfracEvalError::ArityMismatch {
        expected_min: meta.arity.min,
        expected_max: meta.arity.max,
        actual,
    }
}

fn max_excel_serial() -> i64 {
    excel_serial_from_ymd(WorkbookDateSystem::System1900, 9999, 12, 31).unwrap() as i64
}

fn number_arg(args: &[PreparedArgValue], idx: usize) -> Result<f64, DiscountBillYearfracEvalError> {
    args.get(idx)
        .ok_or(DiscountBillYearfracEvalError::Domain(
            WorksheetErrorCode::Value,
        ))
        .and_then(|value| {
            coerce_prepared_to_number(value).map_err(DiscountBillYearfracEvalError::Coercion)
        })
}

fn parse_basis(value: f64) -> Result<DayCountBasis, DiscountBillYearfracEvalError> {
    if !value.is_finite() {
        return Err(DiscountBillYearfracEvalError::Domain(
            WorksheetErrorCode::Num,
        ));
    }
    match value.trunc() as i64 {
        0 => Ok(DayCountBasis::Us30_360),
        1 => Ok(DayCountBasis::ActualActual),
        2 => Ok(DayCountBasis::Actual360),
        3 => Ok(DayCountBasis::Actual365),
        4 => Ok(DayCountBasis::European30_360),
        _ => Err(DiscountBillYearfracEvalError::Domain(
            WorksheetErrorCode::Num,
        )),
    }
}

fn parse_date_serial(value: f64) -> Result<i64, DiscountBillYearfracEvalError> {
    if !value.is_finite() {
        return Err(DiscountBillYearfracEvalError::Domain(
            WorksheetErrorCode::Value,
        ));
    }
    let serial = value.trunc() as i64;
    if serial < 1 || serial > max_excel_serial() {
        return Err(DiscountBillYearfracEvalError::Domain(
            WorksheetErrorCode::Value,
        ));
    }
    ymd_from_excel_serial(WorkbookDateSystem::System1900, serial as f64).ok_or(
        DiscountBillYearfracEvalError::Domain(WorksheetErrorCode::Value),
    )?;
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

fn actual_days(start: i64, end: i64) -> f64 {
    (end - start) as f64
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

fn days360_us(start: i64, end: i64) -> Result<f64, DiscountBillYearfracEvalError> {
    let (sy, sm, mut sd) = ymd_from_excel_serial(WorkbookDateSystem::System1900, start as f64)
        .ok_or(DiscountBillYearfracEvalError::Domain(
            WorksheetErrorCode::Value,
        ))?;
    let (ey, em, mut ed) = ymd_from_excel_serial(WorkbookDateSystem::System1900, end as f64)
        .ok_or(DiscountBillYearfracEvalError::Domain(
            WorksheetErrorCode::Value,
        ))?;

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

fn days360_eu(start: i64, end: i64) -> Result<f64, DiscountBillYearfracEvalError> {
    let (sy, sm, mut sd) = ymd_from_excel_serial(WorkbookDateSystem::System1900, start as f64)
        .ok_or(DiscountBillYearfracEvalError::Domain(
            WorksheetErrorCode::Value,
        ))?;
    let (ey, em, mut ed) = ymd_from_excel_serial(WorkbookDateSystem::System1900, end as f64)
        .ok_or(DiscountBillYearfracEvalError::Domain(
            WorksheetErrorCode::Value,
        ))?;
    if sd == 31 {
        sd = 30;
    }
    if ed == 31 {
        ed = 30;
    }
    Ok(((ey - sy) * 360 + (em - sm) * 30 + (ed - sd)) as f64)
}

fn actual_actual_positive(start: i64, end: i64) -> Result<f64, DiscountBillYearfracEvalError> {
    if end <= start {
        return Ok(0.0);
    }
    let (sy, _, _) = ymd_from_excel_serial(WorkbookDateSystem::System1900, start as f64).ok_or(
        DiscountBillYearfracEvalError::Domain(WorksheetErrorCode::Value),
    )?;
    let (ey, _, _) = ymd_from_excel_serial(WorkbookDateSystem::System1900, end as f64).ok_or(
        DiscountBillYearfracEvalError::Domain(WorksheetErrorCode::Value),
    )?;
    if sy == ey {
        return Ok(actual_days(start, end) / days_in_year(sy));
    }

    let start_next_year = excel_serial_from_ymd(WorkbookDateSystem::System1900, sy + 1, 1, 1)
        .ok_or(DiscountBillYearfracEvalError::Domain(
            WorksheetErrorCode::Value,
        ))? as i64;
    let end_year_start = excel_serial_from_ymd(WorkbookDateSystem::System1900, ey, 1, 1).ok_or(
        DiscountBillYearfracEvalError::Domain(WorksheetErrorCode::Value),
    )? as i64;

    let mut total = actual_days(start, start_next_year) / days_in_year(sy);
    for _year in (sy + 1)..ey {
        total += 1.0;
    }
    total += actual_days(end_year_start, end) / days_in_year(ey);
    Ok(total)
}

fn yearfrac_positive(
    start: i64,
    end: i64,
    basis: DayCountBasis,
) -> Result<f64, DiscountBillYearfracEvalError> {
    match basis {
        DayCountBasis::Us30_360 => Ok(days360_us(start, end)? / 360.0),
        DayCountBasis::ActualActual => actual_actual_positive(start, end),
        DayCountBasis::Actual360 => Ok(actual_days(start, end) / 360.0),
        DayCountBasis::Actual365 => Ok(actual_days(start, end) / 365.0),
        DayCountBasis::European30_360 => Ok(days360_eu(start, end)? / 360.0),
    }
}

pub fn yearfrac_kernel(
    start_date: f64,
    end_date: f64,
    basis: Option<f64>,
) -> Result<f64, DiscountBillYearfracEvalError> {
    let start = parse_date_serial(start_date)?;
    let end = parse_date_serial(end_date)?;
    let basis = parse_basis(basis.unwrap_or(0.0))?;
    if end >= start {
        yearfrac_positive(start, end, basis)
    } else {
        Ok(-yearfrac_positive(end, start, basis)?)
    }
}

fn security_fraction(
    settlement: f64,
    maturity: f64,
    basis: Option<f64>,
) -> Result<f64, DiscountBillYearfracEvalError> {
    let settlement = parse_date_serial(settlement)?;
    let maturity = parse_date_serial(maturity)?;
    let basis = parse_basis(basis.unwrap_or(0.0))?;
    if settlement >= maturity {
        return Err(DiscountBillYearfracEvalError::Domain(
            WorksheetErrorCode::Num,
        ));
    }
    yearfrac_positive(settlement, maturity, basis)
}

pub fn disc_kernel(
    settlement: f64,
    maturity: f64,
    pr: f64,
    redemption: f64,
    basis: Option<f64>,
) -> Result<f64, DiscountBillYearfracEvalError> {
    if !pr.is_finite() || !redemption.is_finite() {
        return Err(DiscountBillYearfracEvalError::Domain(
            WorksheetErrorCode::Value,
        ));
    }
    if pr <= 0.0 || redemption <= 0.0 {
        return Err(DiscountBillYearfracEvalError::Domain(
            WorksheetErrorCode::Num,
        ));
    }
    let frac = security_fraction(settlement, maturity, basis)?;
    Ok((1.0 - pr / redemption) / frac)
}

pub fn intrate_kernel(
    settlement: f64,
    maturity: f64,
    investment: f64,
    redemption: f64,
    basis: Option<f64>,
) -> Result<f64, DiscountBillYearfracEvalError> {
    if !investment.is_finite() || !redemption.is_finite() {
        return Err(DiscountBillYearfracEvalError::Domain(
            WorksheetErrorCode::Value,
        ));
    }
    if investment <= 0.0 || redemption <= 0.0 {
        return Err(DiscountBillYearfracEvalError::Domain(
            WorksheetErrorCode::Num,
        ));
    }
    let frac = security_fraction(settlement, maturity, basis)?;
    Ok(((redemption - investment) / investment) / frac)
}

pub fn received_kernel(
    settlement: f64,
    maturity: f64,
    investment: f64,
    discount: f64,
    basis: Option<f64>,
) -> Result<f64, DiscountBillYearfracEvalError> {
    if !investment.is_finite() || !discount.is_finite() {
        return Err(DiscountBillYearfracEvalError::Domain(
            WorksheetErrorCode::Value,
        ));
    }
    if investment <= 0.0 || discount <= 0.0 {
        return Err(DiscountBillYearfracEvalError::Domain(
            WorksheetErrorCode::Num,
        ));
    }
    let frac = security_fraction(settlement, maturity, basis)?;
    let denom = 1.0 - discount * frac;
    if denom <= 0.0 {
        return Err(DiscountBillYearfracEvalError::Domain(
            WorksheetErrorCode::Num,
        ));
    }
    Ok(investment / denom)
}

pub fn pricedisc_kernel(
    settlement: f64,
    maturity: f64,
    discount: f64,
    redemption: f64,
    basis: Option<f64>,
) -> Result<f64, DiscountBillYearfracEvalError> {
    if !discount.is_finite() || !redemption.is_finite() {
        return Err(DiscountBillYearfracEvalError::Domain(
            WorksheetErrorCode::Value,
        ));
    }
    if discount <= 0.0 || redemption <= 0.0 {
        return Err(DiscountBillYearfracEvalError::Domain(
            WorksheetErrorCode::Num,
        ));
    }
    let frac = security_fraction(settlement, maturity, basis)?;
    Ok(redemption * (1.0 - discount * frac))
}

fn tbill_days(settlement: f64, maturity: f64) -> Result<f64, DiscountBillYearfracEvalError> {
    let settlement = parse_date_serial(settlement)?;
    let maturity = parse_date_serial(maturity)?;
    if settlement >= maturity {
        return Err(DiscountBillYearfracEvalError::Domain(
            WorksheetErrorCode::Num,
        ));
    }
    let one_year_out = add_months_clamped(settlement, 12).ok_or(
        DiscountBillYearfracEvalError::Domain(WorksheetErrorCode::Num),
    )?;
    if maturity > one_year_out {
        return Err(DiscountBillYearfracEvalError::Domain(
            WorksheetErrorCode::Num,
        ));
    }
    Ok(actual_days(settlement, maturity))
}

pub fn tbillprice_kernel(
    settlement: f64,
    maturity: f64,
    discount: f64,
) -> Result<f64, DiscountBillYearfracEvalError> {
    if !discount.is_finite() {
        return Err(DiscountBillYearfracEvalError::Domain(
            WorksheetErrorCode::Value,
        ));
    }
    if discount <= 0.0 {
        return Err(DiscountBillYearfracEvalError::Domain(
            WorksheetErrorCode::Num,
        ));
    }
    let dsm = tbill_days(settlement, maturity)?;
    Ok(100.0 * (1.0 - discount * dsm / 360.0))
}

pub fn tbillyield_kernel(
    settlement: f64,
    maturity: f64,
    pr: f64,
) -> Result<f64, DiscountBillYearfracEvalError> {
    if !pr.is_finite() {
        return Err(DiscountBillYearfracEvalError::Domain(
            WorksheetErrorCode::Value,
        ));
    }
    if pr <= 0.0 {
        return Err(DiscountBillYearfracEvalError::Domain(
            WorksheetErrorCode::Num,
        ));
    }
    let dsm = tbill_days(settlement, maturity)?;
    Ok((100.0 - pr) / pr * 360.0 / dsm)
}

pub fn tbilleq_kernel(
    settlement: f64,
    maturity: f64,
    discount: f64,
) -> Result<f64, DiscountBillYearfracEvalError> {
    if !discount.is_finite() {
        return Err(DiscountBillYearfracEvalError::Domain(
            WorksheetErrorCode::Value,
        ));
    }
    if discount <= 0.0 {
        return Err(DiscountBillYearfracEvalError::Domain(
            WorksheetErrorCode::Num,
        ));
    }
    let dsm = tbill_days(settlement, maturity)?;
    let denom = 360.0 - discount * dsm;
    if denom <= 0.0 {
        return Err(DiscountBillYearfracEvalError::Domain(
            WorksheetErrorCode::Num,
        ));
    }
    Ok(365.0 * discount / denom)
}

fn eval_numeric(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
    meta: &FunctionMeta,
    kernel: impl FnOnce(&[PreparedArgValue]) -> Result<f64, DiscountBillYearfracEvalError>,
) -> Result<EvalValue, DiscountBillYearfracEvalError> {
    if !meta.arity.accepts(args.len()) {
        return Err(arity_error(meta, args.len()));
    }
    run_values_only_prepared(
        args,
        resolver,
        |prepared| kernel(prepared).map(EvalValue::Number),
        DiscountBillYearfracEvalError::Coercion,
    )
}

pub fn eval_disc_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DiscountBillYearfracEvalError> {
    eval_numeric(args, resolver, &DISC_META, |prepared| {
        disc_kernel(
            number_arg(prepared, 0)?,
            number_arg(prepared, 1)?,
            number_arg(prepared, 2)?,
            number_arg(prepared, 3)?,
            prepared
                .get(4)
                .map(|_| number_arg(prepared, 4))
                .transpose()?,
        )
    })
}

pub fn eval_intrate_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DiscountBillYearfracEvalError> {
    eval_numeric(args, resolver, &INTRATE_META, |prepared| {
        intrate_kernel(
            number_arg(prepared, 0)?,
            number_arg(prepared, 1)?,
            number_arg(prepared, 2)?,
            number_arg(prepared, 3)?,
            prepared
                .get(4)
                .map(|_| number_arg(prepared, 4))
                .transpose()?,
        )
    })
}

pub fn eval_received_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DiscountBillYearfracEvalError> {
    eval_numeric(args, resolver, &RECEIVED_META, |prepared| {
        received_kernel(
            number_arg(prepared, 0)?,
            number_arg(prepared, 1)?,
            number_arg(prepared, 2)?,
            number_arg(prepared, 3)?,
            prepared
                .get(4)
                .map(|_| number_arg(prepared, 4))
                .transpose()?,
        )
    })
}

pub fn eval_pricedisc_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DiscountBillYearfracEvalError> {
    eval_numeric(args, resolver, &PRICEDISC_META, |prepared| {
        pricedisc_kernel(
            number_arg(prepared, 0)?,
            number_arg(prepared, 1)?,
            number_arg(prepared, 2)?,
            number_arg(prepared, 3)?,
            prepared
                .get(4)
                .map(|_| number_arg(prepared, 4))
                .transpose()?,
        )
    })
}

pub fn eval_tbilleq_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DiscountBillYearfracEvalError> {
    eval_numeric(args, resolver, &TBILLEQ_META, |prepared| {
        tbilleq_kernel(
            number_arg(prepared, 0)?,
            number_arg(prepared, 1)?,
            number_arg(prepared, 2)?,
        )
    })
}

pub fn eval_tbillprice_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DiscountBillYearfracEvalError> {
    eval_numeric(args, resolver, &TBILLPRICE_META, |prepared| {
        tbillprice_kernel(
            number_arg(prepared, 0)?,
            number_arg(prepared, 1)?,
            number_arg(prepared, 2)?,
        )
    })
}

pub fn eval_tbillyield_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DiscountBillYearfracEvalError> {
    eval_numeric(args, resolver, &TBILLYIELD_META, |prepared| {
        tbillyield_kernel(
            number_arg(prepared, 0)?,
            number_arg(prepared, 1)?,
            number_arg(prepared, 2)?,
        )
    })
}

pub fn eval_yearfrac_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DiscountBillYearfracEvalError> {
    eval_numeric(args, resolver, &YEARFRAC_META, |prepared| {
        yearfrac_kernel(
            number_arg(prepared, 0)?,
            number_arg(prepared, 1)?,
            prepared
                .get(2)
                .map(|_| number_arg(prepared, 2))
                .transpose()?,
        )
    })
}

pub fn map_discount_bill_yearfrac_error_to_ws(
    error: &DiscountBillYearfracEvalError,
) -> WorksheetErrorCode {
    match error {
        DiscountBillYearfracEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        DiscountBillYearfracEvalError::Coercion(coercion) => match coercion {
            CoercionError::WorksheetError(code) => *code,
            CoercionError::RefResolution(_) => WorksheetErrorCode::Ref,
            CoercionError::MissingArg
            | CoercionError::EmptyCell
            | CoercionError::NonNumericText(_)
            | CoercionError::UnsupportedValueKind(_) => WorksheetErrorCode::Value,
        },
        DiscountBillYearfracEvalError::Domain(code) => *code,
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

    fn assert_bits(actual: f64, expected: f64) {
        assert_eq!(
            actual.to_bits(),
            expected.to_bits(),
            "{actual} vs {expected}"
        );
    }

    #[test]
    fn meta_ids_match_expected_function_ids() {
        assert_eq!(DISC_META.function_id, "FUNC.DISC");
        assert_eq!(INTRATE_META.function_id, "FUNC.INTRATE");
        assert_eq!(RECEIVED_META.function_id, "FUNC.RECEIVED");
        assert_eq!(PRICEDISC_META.function_id, "FUNC.PRICEDISC");
        assert_eq!(TBILLEQ_META.function_id, "FUNC.TBILLEQ");
        assert_eq!(TBILLPRICE_META.function_id, "FUNC.TBILLPRICE");
        assert_eq!(TBILLYIELD_META.function_id, "FUNC.TBILLYIELD");
        assert_eq!(YEARFRAC_META.function_id, "FUNC.YEARFRAC");
    }

    #[test]
    fn yearfrac_matches_microsoft_examples() {
        let start = serial(2012, 1, 1);
        let end = serial(2012, 7, 30);
        assert_close(
            yearfrac_kernel(start, end, None).unwrap(),
            0.580_555_56,
            1.0e-8,
        );
        assert_close(
            yearfrac_kernel(start, end, Some(1.0)).unwrap(),
            0.576_502_73,
            1.0e-8,
        );
        assert_close(
            yearfrac_kernel(start, end, Some(3.0)).unwrap(),
            0.578_082_19,
            1.0e-8,
        );
    }

    #[test]
    fn yearfrac_is_sign_symmetric_in_bounded_slice() {
        let start = serial(2012, 1, 1);
        let end = serial(2012, 7, 30);
        let forward = yearfrac_kernel(start, end, Some(1.0)).unwrap();
        let reverse = yearfrac_kernel(end, start, Some(1.0)).unwrap();
        assert_close(forward, -reverse, 1.0e-12);
    }

    #[test]
    fn pricedisc_matches_microsoft_example() {
        let got = pricedisc_kernel(
            serial(2008, 2, 16),
            serial(2008, 3, 1),
            0.0525,
            100.0,
            Some(2.0),
        )
        .unwrap();
        assert_close(got, 99.795_833_333_333_33, 1.0e-10);
    }

    #[test]
    fn intrate_matches_microsoft_example() {
        let got = intrate_kernel(
            serial(2008, 2, 15),
            serial(2008, 5, 15),
            1_000_000.0,
            1_014_420.0,
            Some(2.0),
        )
        .unwrap();
        assert_close(got, 0.057_68, 1.0e-5);
    }

    #[test]
    fn received_matches_microsoft_example() {
        let got = received_kernel(
            serial(2008, 2, 15),
            serial(2008, 5, 15),
            1_000_000.0,
            0.0575,
            Some(2.0),
        )
        .unwrap();
        assert_close(got, 1_014_584.654_407_102, 1.0e-6);
    }

    #[test]
    fn tbill_functions_match_microsoft_examples() {
        let settlement = serial(2008, 3, 31);
        let maturity = serial(2008, 6, 1);
        assert_close(
            tbillprice_kernel(settlement, maturity, 0.09).unwrap(),
            98.45,
            1.0e-10,
        );
        assert_close(
            tbillyield_kernel(settlement, maturity, 98.45).unwrap(),
            0.091_416_962_925_342_6,
            1.0e-12,
        );
        assert_close(
            tbilleq_kernel(settlement, maturity, 0.0914).unwrap(),
            0.094_151_493_565_943,
            1.0e-12,
        );
    }

    #[test]
    fn disc_round_trips_pricedisc_in_bounded_slice() {
        let settlement = serial(2008, 2, 16);
        let maturity = serial(2008, 3, 1);
        let price = pricedisc_kernel(settlement, maturity, 0.0525, 100.0, Some(2.0)).unwrap();
        let recovered = disc_kernel(settlement, maturity, price, 100.0, Some(2.0)).unwrap();
        assert_close(recovered, 0.0525, 1.0e-12);
    }

    #[test]
    fn disc_exactness_witness_matches_excel_target_and_fraction_is_unity() {
        let settlement = 44927.0_f64;
        let maturity = 45292.0_f64;
        let actual = disc_kernel(settlement, maturity, 97.0, 100.0, None).expect("disc witness");
        let prior_local = f64::from_bits(0x3f9eb851eb851eb8);
        let excel_target = f64::from_bits(0x3f9eb851eb851ec0);
        let fraction = security_fraction(settlement, maturity, None).expect("disc witness frac");

        assert_bits(fraction, 1.0_f64);
        assert_bits(actual, excel_target);
        assert_ne!(actual.to_bits(), prior_local.to_bits());
    }

    #[test]
    fn tbill_rejects_maturities_more_than_one_calendar_year_out() {
        assert_eq!(
            tbillprice_kernel(serial(2024, 1, 1), serial(2025, 1, 2), 0.05),
            Err(DiscountBillYearfracEvalError::Domain(
                WorksheetErrorCode::Num
            ))
        );
    }

    #[test]
    fn invalid_basis_is_num_error() {
        assert_eq!(
            yearfrac_kernel(serial(2012, 1, 1), serial(2012, 7, 30), Some(9.0)),
            Err(DiscountBillYearfracEvalError::Domain(
                WorksheetErrorCode::Num
            ))
        );
    }
}

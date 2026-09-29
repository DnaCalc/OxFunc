use crate::coercion::CoercionError;
use crate::function::{
    Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile, FunctionMeta,
    HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::coerce_prepared_to_number;
// The observed financial padding rule is the same positional-error rule as
// NORM/EXPON: a later absent coordinate must not replace an earlier error.
use crate::functions::distribution_common::run_distribution_lifted as run_depreciation_lifted;
use crate::resolver::ReferenceSystemProvider;
use crate::value::{CalcValue, CoreValue, WorksheetErrorCode};

const OPTIONAL_ARITY_5: Arity = Arity { min: 4, max: 5 };
const OPTIONAL_ARITY_7: Arity = Arity { min: 5, max: 7 };

const DEPRECIATION_BASE_META: FunctionMeta = function_spec! {
    function_id: "FUNC.DEPRECIATION_BASE",
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

pub const SLN_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.SLN",
    arity: Arity::exact(3),
    ..DEPRECIATION_BASE_META
};
pub const SYD_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.SYD",
    arity: Arity::exact(4),
    ..DEPRECIATION_BASE_META
};
pub const DB_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.DB",
    arity: OPTIONAL_ARITY_5,
    ..DEPRECIATION_BASE_META
};
pub const DDB_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.DDB",
    arity: OPTIONAL_ARITY_5,
    ..DEPRECIATION_BASE_META
};
pub const VDB_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.VDB",
    arity: OPTIONAL_ARITY_7,
    ..DEPRECIATION_BASE_META
};

#[derive(Debug, Clone, PartialEq)]
pub enum DepreciationEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
}

fn arity_error(meta: &FunctionMeta, actual: usize) -> DepreciationEvalError {
    DepreciationEvalError::ArityMismatch {
        expected_min: meta.arity.min,
        expected_max: meta.arity.max,
        actual,
    }
}

fn required_number(arg: &CalcValue) -> Result<f64, DepreciationEvalError> {
    coerce_prepared_to_number(arg).map_err(DepreciationEvalError::Coercion)
}

fn optional_number(
    args: &[CalcValue],
    idx: usize,
    default: f64,
) -> Result<f64, DepreciationEvalError> {
    match args.get(idx) {
        None => Ok(default),
        Some(arg) => depreciation_number(arg),
    }
}

fn validate_finite(values: &[f64]) -> Result<(), WorksheetErrorCode> {
    if values.iter().all(|value| value.is_finite()) {
        Ok(())
    } else {
        Err(WorksheetErrorCode::Num)
    }
}

pub fn sln_kernel(cost: f64, salvage: f64, life: f64) -> Result<f64, WorksheetErrorCode> {
    validate_finite(&[cost, salvage, life])?;
    if life == 0.0 {
        return Err(WorksheetErrorCode::Div0);
    }
    // SLN admits either sign and every nonzero life. The observed quotient is
    // rounded through the existing extended-precision division substrate.
    // Preserve subnormal differences until division: only the published result
    // is flushed to positive zero (Excel 16.0 build 20430, CV2, W111 probes).
    let result = crate::excel_numeric::excel_x87_div(cost - salvage, life);
    if !result.is_finite() {
        Err(WorksheetErrorCode::Num)
    } else if result.abs() < f64::MIN_POSITIVE {
        Ok(0.0)
    } else {
        Ok(result)
    }
}

pub fn syd_kernel(cost: f64, salvage: f64, life: f64, per: f64) -> Result<f64, WorksheetErrorCode> {
    validate_finite(&[cost, salvage, life, per])?;
    if salvage < 0.0 || life <= 0.0 || per <= 0.0 || (per > life && per - life >= f64::MIN_POSITIVE)
    {
        return Err(WorksheetErrorCode::Num);
    }
    // SYD admits negative cost. Its denominator publishes nonfinite/subnormal
    // values as zero; a zero denominator is DIV/0. Basis overflow remains NUM.
    // Keep (life + 1) - period in that order, including lost low bits for
    // large life. Selected store discriminators require x87 multiply/divide.
    let stage_or_zero = |value: f64| {
        if !value.is_finite() || value.abs() < f64::MIN_POSITIVE {
            0.0
        } else {
            value
        }
    };
    let life_plus_one = crate::excel_numeric::excel_x87_add(life, 1.0);
    let denominator = stage_or_zero(crate::excel_numeric::excel_x87_mul(life, life_plus_one) / 2.0);
    if denominator == 0.0 {
        return Err(WorksheetErrorCode::Div0);
    }
    let basis = publish_depreciation(crate::excel_numeric::excel_x87_sub(cost, salvage))?;
    let remaining = crate::excel_numeric::excel_x87_sub(life_plus_one, per);
    let numerator = crate::excel_numeric::excel_x87_mul(basis, remaining);
    publish_depreciation(crate::excel_numeric::excel_x87_div(numerator, denominator))
}

fn publish_depreciation(value: f64) -> Result<f64, WorksheetErrorCode> {
    if !value.is_finite() {
        Err(WorksheetErrorCode::Num)
    } else if value.abs() < f64::MIN_POSITIVE {
        Ok(0.0)
    } else {
        Ok(value)
    }
}

fn depreciation_integer_power(mut base: f64, mut exponent: u32) -> f64 {
    let mut product = 1.0;
    while exponent > 0 {
        if exponent & 1 != 0 {
            product = crate::excel_numeric::excel_x87_mul(product, base);
        }
        exponent >>= 1;
        if exponent > 0 {
            base = crate::excel_numeric::excel_x87_mul(base, base);
        }
    }
    product
}

fn db_rate(cost: f64, salvage: f64, life: f64) -> Result<f64, WorksheetErrorCode> {
    let ratio = crate::excel_numeric::excel_x87_div(salvage, cost);
    // W111 ratio/power overflow and subnormal probes publish rate=1, rather
    // than propagating an intermediate numeric error. This is internal DB
    // arithmetic; the source Value2 inputs remain exactly as captured.
    let ratio = if !ratio.is_finite() || ratio.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        ratio
    };
    let exponent = crate::excel_numeric::excel_x87_div(1.0, life);
    let integer_power = exponent.fract() == 0.0 && exponent < u32::MAX as f64;
    let power = if integer_power {
        // DB's integer dispatch is narrower than the worksheet POWER wrapper.
        let value = depreciation_integer_power(ratio, exponent as u32);
        if value.is_finite() {
            value
        } else {
            0.0
        }
    } else if ratio == 0.0 {
        0.0
    } else {
        // Above the unsigned-count sentinel, even exact integer exponents use
        // the positive power chain. Rate-half probes distinguish it from the
        // worksheet wrapper's wider repeated-multiplication integer branch.
        crate::excel_numeric::excel_pow_positive(ratio, exponent)
    };
    // An overflowing exp/log product reaches NaN in the internal power chain.
    // DB publishes a zero rate on that route, distinct from an ordinary power
    // overflow/error or an underflowed finite power (W111 extreme-life probes).
    if power.is_nan() {
        return Ok(0.0);
    }
    if power.is_infinite() {
        return Err(WorksheetErrorCode::Num);
    }
    let power = if power.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        power
    };
    // DB uses Excel decimal ROUND, including its 15-significant-digit input
    // staging, not binary (rate*1000+0.5).floor()/1000.
    Ok(crate::functions::round_fn::round_kernel(1.0 - power, 3))
}

pub fn db_kernel(
    cost: f64,
    salvage: f64,
    life: f64,
    period: f64,
    month: f64,
) -> Result<f64, WorksheetErrorCode> {
    validate_finite(&[cost, salvage, life, period, month])?;
    // The period and month are truncated, but life remains fractional. Original
    // positive periods below one still request the initial depreciation.
    let month = month.trunc();
    // The host first saturates a nonnegative count to u32, then interprets that
    // word as signed. W111 boundary probes distinguish this from an i32 cast:
    // 2^31 becomes MIN, while counts above u32::MAX become -1.
    let whole_period = (period as u32) as i32;
    let whole_life = (life as u32) as i32;
    if cost < 0.0
        || salvage < 0.0
        || life <= 0.0
        || period <= 0.0
        || month < 1.0
        || month > 12.0
        || i64::from(whole_period) > i64::from(whole_life) + if month < 12.0 { 1 } else { 0 }
    {
        return Err(WorksheetErrorCode::Num);
    }
    if cost == 0.0 {
        return Ok(0.0);
    }
    let rate = db_rate(cost, salvage, life)?;
    if rate == 0.0 {
        return Ok(0.0);
    }
    // These store boundaries are observed by W111 DB probes. Replacing the
    // book recurrence with cost minus a running total changes final bits.
    let mut dep = crate::excel_numeric::excel_x87_mul(
        crate::excel_numeric::excel_x87_mul(cost, rate),
        month / 12.0,
    );
    // Each depreciation amount is published before it changes the book.
    // A subnormal first/regular amount is zero even if subsequent negative-rate
    // growth could otherwise amplify that difference (W111 minimum-normal inputs).
    dep = publish_depreciation(dep)?;
    let mut book = crate::excel_numeric::excel_x87_sub(cost, dep);
    let regular_periods = whole_period.min(whole_life).max(0) as usize;
    for p in 2..=regular_periods {
        dep = publish_depreciation(crate::excel_numeric::excel_x87_mul(book, rate))?;
        if dep == 0.0 {
            break;
        }
        book = crate::excel_numeric::excel_x87_sub(book, dep);
        if book == 0.0 {
            if p < regular_periods {
                dep = 0.0;
            }
            break;
        }
        if !book.is_finite() {
            return Err(WorksheetErrorCode::Num);
        }
    }
    if month < 12.0 && i64::from(whole_period) == i64::from(whole_life) + 1 {
        // The final partial year's quotient and rate product are separate
        // published amounts too: later multiplication must not recover a
        // subnormal intermediate (W111 minimum-normal final-year probes).
        let quotient = publish_depreciation(book / 12.0)?;
        let product = publish_depreciation(crate::excel_numeric::excel_x87_mul(quotient, rate))?;
        dep = product * (12.0 - month);
    }
    publish_depreciation(dep)
}

fn declining_interval_depreciation(
    cost: f64,
    salvage: f64,
    life: f64,
    start_period: f64,
    end_period: f64,
    factor: f64,
    no_switch: bool,
) -> Result<f64, WorksheetErrorCode> {
    validate_finite(&[cost, salvage, life, start_period, end_period, factor])?;
    if cost < 0.0 || salvage < 0.0 || life <= 0.0 || factor < 0.0 {
        return Err(WorksheetErrorCode::Num);
    }
    if start_period < 0.0 || end_period < start_period || end_period > life {
        return Err(WorksheetErrorCode::Num);
    }
    if no_switch {
        if start_period == end_period {
            return Ok(0.0);
        }
        let mut year = start_period.floor();
        let mut total = 0.0;
        while year < end_period {
            let dep = ddb_period_depreciation(cost, salvage, life, year + 1.0, factor)?;
            // Excel rounds these partial products separately, even when both
            // bounds lie in the same year. The final fraction is formed by
            // subtracting the unused tail from one; end-year loses observed
            // cancellation, including tiny negative intervals. The final year uses the same
            // annual amount, without rescaling it by the remaining year length.
            let final_fraction = 1.0 - ((year + 1.0) - end_period).max(0.0);
            // Both products pass through an extended-precision store. Their
            // separate rounding is visible when adjacent endpoints cancel.
            let final_part =
                publish_depreciation(crate::excel_numeric::excel_x87_mul(dep, final_fraction))?;
            let initial_part = publish_depreciation(crate::excel_numeric::excel_x87_mul(
                dep,
                (start_period - year).max(0.0),
            ))?;
            total += final_part - initial_part;
            if dep == 0.0 {
                break;
            }
            year += 1.0;
        }
        // The partial products publish subnormals as zero, but subtraction
        // of two normal products may return a nonzero subnormal Value2.
        return if total.is_finite() {
            Ok(total)
        } else {
            Err(WorksheetErrorCode::Num)
        };
    }
    switched_interval_depreciation(cost, salvage, life, start_period, end_period, factor)
}

// W111 frozen switched-VDB graph: separate advance/requested book stores,
// absolute stored period cursor, and operation-specific numeric publication.
// The bounded evidence is retained in VDB_SWITCHED_GRAPH_RESEARCH.md. Large
// period progress remains a separate open lane; no iteration cap or invented
// worksheet error is imposed here.
fn switched_interval_depreciation(
    cost: f64,
    salvage: f64,
    life: f64,
    start_period: f64,
    end_period: f64,
    factor: f64,
) -> Result<f64, WorksheetErrorCode> {
    use crate::excel_numeric::{
        excel_x87_add as add, excel_x87_div as div, excel_x87_mul as mul, excel_x87_sub as sub,
    };
    if start_period == end_period {
        return Ok(0.0);
    }
    // Exact neighboring probes pin an inclusive absolute 2^-1026 boundary.
    // This is a VDB switch rule, not a general numeric comparison policy.
    let switch_exceeds = |straight: f64, declining: f64| {
        straight > declining && straight - declining >= f64::MIN_POSITIVE / 16.0
    };
    let rate = publish_depreciation(div(factor, life))?;
    let initial_declining = publish_depreciation(mul(cost, rate))?;
    let initial_straight = publish_depreciation(div(cost - salvage, life))?;
    if cost >= salvage && switch_exceeds(initial_straight, initial_declining.min(cost - salvage)) {
        return publish_depreciation(mul(initial_straight, end_period - start_period));
    }
    let fraction = start_period - start_period.floor();
    let initial_amount = initial_declining.min(cost - salvage).max(0.0);
    let initial_part = publish_depreciation(mul(initial_amount, fraction))?;
    let mut book = sub(cost, initial_part);
    let mut remaining_life = life - fraction;
    let mut straight_line = None;
    let mut accumulate = |duration: f64, requested: bool| -> Result<f64, WorksheetErrorCode> {
        let mut total = 0.0;
        let mut cursor = start_period;
        let mut year = 0.0;
        loop {
            if if requested {
                cursor >= end_period
            } else {
                year >= duration
            } {
                break;
            }
            let declining = publish_depreciation(mul(book, rate))?;
            let available = book - salvage;
            let denominator = if declining > available {
                remaining_life.max(1.0)
            } else {
                remaining_life
            };
            let straight = match straight_line {
                Some(value) => value,
                None => publish_depreciation(div(available, denominator))?,
            };
            // Even clipping does not hide overflow of this remaining-life
            // division. A fixed straight-line amount does not waive it either.
            publish_depreciation(div(available, remaining_life))?;
            let amount = if straight_line.is_none() && declining > available {
                available
            } else if switch_exceeds(straight, declining.min(available)) {
                straight_line = Some(straight);
                straight
            } else {
                declining.min(available)
            };
            let remaining = if requested {
                end_period - cursor
            } else {
                duration - year
            };
            let take = if straight_line.is_some() {
                remaining
            } else {
                remaining.min(1.0)
            };
            // A whole clipped year preserves a subnormal amount. Fractional
            // products publish separately, so a blanket final flush is wrong.
            let part = if take == 1.0 {
                amount
            } else {
                publish_depreciation(mul(amount, take))?
            };
            total = add(total, part);
            book = if requested {
                book - part
            } else {
                sub(book, part)
            };
            remaining_life -= take;
            // Use the already stored remaining length: cursor+1 can round
            // below end even when this subtraction has rounded to exactly one.
            if requested && remaining <= 1.0 {
                break;
            }
            cursor += 1.0;
            if straight_line.is_some() {
                break;
            }
            year += 1.0;
        }
        if !total.is_finite() {
            Err(WorksheetErrorCode::Num)
        } else {
            Ok(if total == 0.0 { 0.0 } else { total })
        }
    };
    accumulate(start_period.floor(), false)?;
    accumulate(end_period - start_period, true)
}

pub fn ddb_kernel(
    cost: f64,
    salvage: f64,
    life: f64,
    period: f64,
    factor: f64,
) -> Result<f64, WorksheetErrorCode> {
    validate_finite(&[cost, salvage, life, period, factor])?;
    if cost < 0.0 || salvage < 0.0 || life <= 0.0 || period <= 0.0 || period > life || factor <= 0.0
    {
        return Err(WorksheetErrorCode::Num);
    }
    ddb_period_depreciation(cost, salvage, life, period, factor)
}

// Admitted DDB arithmetic is also VDB's no-switch annual amount. VDB permits
// a fractional final year, whose annual index can exceed the fractional life.
fn ddb_period_depreciation(
    cost: f64,
    salvage: f64,
    life: f64,
    period: f64,
    factor: f64,
) -> Result<f64, WorksheetErrorCode> {
    let rate = crate::excel_numeric::excel_x87_div(factor, life);
    // DDB publishes its division stage before the cost product. A subnormal
    // rate is zero even if multiplication by cost could make it normal again.
    let rate = if rate.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        rate
    };
    // The financial power dispatch shares POWER's bounded integer substrate,
    // but its count sentinel is u32::MAX rather than the worksheet wrapper's
    // wider integer range (W111 large-period discriminator).
    let power = if period <= 1.0 {
        1.0
    } else {
        let base = crate::excel_numeric::excel_x87_sub(1.0, rate).max(0.0);
        let exponent = period - 1.0;
        if exponent.fract() == 0.0 && exponent < u32::MAX as f64 {
            // Each accumulator product is rounded through the x87 store. The
            // ordinary POWER binary64 body differs on large integer periods.
            let value = depreciation_integer_power(base, exponent as u32);
            if value.abs() < f64::MIN_POSITIVE {
                0.0
            } else {
                value
            }
        } else if exponent >= u32::MAX as f64 && base != 0.0 {
            crate::excel_numeric::excel_pow_positive(base, exponent)
        } else {
            crate::functions::power_fn::power_kernel(base, exponent)?
        }
    };
    let book = crate::excel_numeric::excel_x87_mul(cost, power);
    let declining = crate::excel_numeric::excel_x87_mul(book, rate);
    // Overflow is observable before salvage clipping, including cost<salvage.
    if !declining.is_finite() {
        return Err(WorksheetErrorCode::Num);
    }
    publish_depreciation(declining.min(book - salvage).max(0.0))
}

pub fn vdb_kernel(
    cost: f64,
    salvage: f64,
    life: f64,
    start_period: f64,
    end_period: f64,
    factor: f64,
    no_switch: bool,
) -> Result<f64, WorksheetErrorCode> {
    declining_interval_depreciation(
        cost,
        salvage,
        life,
        start_period,
        end_period,
        factor,
        no_switch,
    )
}

fn eval_sln_prepared(args: &[CalcValue]) -> Result<CalcValue, DepreciationEvalError> {
    if !SLN_META.arity.accepts(args.len()) {
        return Err(arity_error(&SLN_META, args.len()));
    }
    let sln_number = |arg: &CalcValue| {
        if matches!(arg.core(), CoreValue::Missing) {
            Ok(0.0)
        } else {
            required_number(arg)
        }
    };
    let cost = sln_number(&args[0])?;
    let salvage = sln_number(&args[1])?;
    let life = sln_number(&args[2])?;
    Ok(match sln_kernel(cost, salvage, life) {
        Ok(value) => CalcValue::number(value),
        Err(code) => CalcValue::error(code),
    })
}

fn eval_syd_prepared(args: &[CalcValue]) -> Result<CalcValue, DepreciationEvalError> {
    if !SYD_META.arity.accepts(args.len()) {
        return Err(arity_error(&SYD_META, args.len()));
    }
    let cost = depreciation_number(&args[0])?;
    let salvage = depreciation_number(&args[1])?;
    let life = depreciation_number(&args[2])?;
    let per = depreciation_number(&args[3])?;
    Ok(match syd_kernel(cost, salvage, life, per) {
        Ok(value) => CalcValue::number(value),
        Err(code) => CalcValue::error(code),
    })
}

// Explicit missing required parameters are numeric zero on these Excel surfaces.
fn depreciation_number(arg: &CalcValue) -> Result<f64, DepreciationEvalError> {
    if matches!(arg.core(), CoreValue::Missing) {
        Ok(0.0)
    } else {
        required_number(arg)
    }
}

fn eval_db_prepared(args: &[CalcValue]) -> Result<CalcValue, DepreciationEvalError> {
    if !DB_META.arity.accepts(args.len()) {
        return Err(arity_error(&DB_META, args.len()));
    }
    let cost = depreciation_number(&args[0])?;
    let salvage = depreciation_number(&args[1])?;
    let life = depreciation_number(&args[2])?;
    let period = depreciation_number(&args[3])?;
    let month = optional_number(args, 4, 12.0)?;
    Ok(match db_kernel(cost, salvage, life, period, month) {
        Ok(value) => CalcValue::number(value),
        Err(code) => CalcValue::error(code),
    })
}

fn eval_ddb_prepared(args: &[CalcValue]) -> Result<CalcValue, DepreciationEvalError> {
    if !DDB_META.arity.accepts(args.len()) {
        return Err(arity_error(&DDB_META, args.len()));
    }
    let cost = depreciation_number(&args[0])?;
    let salvage = depreciation_number(&args[1])?;
    let life = depreciation_number(&args[2])?;
    let period = depreciation_number(&args[3])?;
    let factor = optional_number(args, 4, 2.0)?;
    Ok(match ddb_kernel(cost, salvage, life, period, factor) {
        Ok(value) => CalcValue::number(value),
        Err(code) => CalcValue::error(code),
    })
}

fn eval_vdb_prepared(args: &[CalcValue]) -> Result<CalcValue, DepreciationEvalError> {
    if !VDB_META.arity.accepts(args.len()) {
        return Err(arity_error(&VDB_META, args.len()));
    }
    let cost = depreciation_number(&args[0])?;
    let salvage = depreciation_number(&args[1])?;
    let life = depreciation_number(&args[2])?;
    let start_period = depreciation_number(&args[3])?;
    let end_period = depreciation_number(&args[4])?;
    let factor = optional_number(args, 5, 2.0)?;
    let no_switch = match args.get(6).map(CalcValue::core) {
        None | Some(CoreValue::Missing | CoreValue::Empty) => false,
        Some(CoreValue::Logical(value)) => *value,
        Some(CoreValue::Text(text)) => {
            crate::coercion::parse_excel_logical_text(&text.to_string_lossy()).ok_or_else(|| {
                DepreciationEvalError::Coercion(CoercionError::NonNumericText(
                    text.to_string_lossy(),
                ))
            })?
        }
        _ => depreciation_number(&args[6])? != 0.0,
    };
    Ok(
        match vdb_kernel(
            cost,
            salvage,
            life,
            start_period,
            end_period,
            factor,
            no_switch,
        ) {
            Ok(value) => CalcValue::number(value),
            Err(code) => CalcValue::error(code),
        },
    )
}

pub fn eval_sln_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, DepreciationEvalError> {
    run_depreciation_lifted(
        args,
        resolver,
        eval_sln_prepared,
        map_depreciation_error_to_ws,
        DepreciationEvalError::Coercion,
    )
}

pub fn eval_syd_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, DepreciationEvalError> {
    run_depreciation_lifted(
        args,
        resolver,
        eval_syd_prepared,
        map_depreciation_error_to_ws,
        DepreciationEvalError::Coercion,
    )
}

pub fn eval_db_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, DepreciationEvalError> {
    run_depreciation_lifted(
        args,
        resolver,
        eval_db_prepared,
        map_depreciation_error_to_ws,
        DepreciationEvalError::Coercion,
    )
}

pub fn eval_ddb_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, DepreciationEvalError> {
    run_depreciation_lifted(
        args,
        resolver,
        eval_ddb_prepared,
        map_depreciation_error_to_ws,
        DepreciationEvalError::Coercion,
    )
}

pub fn eval_vdb_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, DepreciationEvalError> {
    run_depreciation_lifted(
        args,
        resolver,
        eval_vdb_prepared,
        map_depreciation_error_to_ws,
        DepreciationEvalError::Coercion,
    )
}

pub fn map_depreciation_error_to_ws(error: &DepreciationEvalError) -> WorksheetErrorCode {
    match error {
        DepreciationEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        DepreciationEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        DepreciationEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::ReferenceSystemCapabilities;
    use crate::value::{ExcelText, ReferenceKind, ReferenceLike};

    struct NoRefResolver;

    impl ReferenceSystemProvider for NoRefResolver {
        fn capabilities(&self) -> ReferenceSystemCapabilities {
            ReferenceSystemCapabilities::permissive_local()
        }

        fn dereference(
            &self,
            request: &crate::resolver::ReferenceDereferenceRequest,
        ) -> Result<CalcValue, crate::resolver::ReferenceResolutionError> {
            let reference = &request.reference;
            Err(
                crate::resolver::ReferenceResolutionError::UnresolvedReference {
                    target: reference.target().to_string(),
                },
            )
        }
    }

    fn num(n: f64) -> CalcValue {
        CalcValue::number(n)
    }

    fn text(s: &str) -> CalcValue {
        CalcValue::text(ExcelText::from_utf16_code_units(s.encode_utf16().collect()))
    }

    #[test]
    fn depreciation_metadata_matches_expected_shape() {
        assert_eq!(SLN_META.arity, Arity::exact(3));
        assert_eq!(DB_META.arity.max, 5);
        assert_eq!(
            DDB_META.arg_preparation_profile,
            FunctionMeta::DEFAULT_ARG_PREPARATION_PROFILE
        );
        assert_eq!(VDB_META.function_id, "FUNC.VDB");
    }

    #[test]
    fn sln_and_syd_match_support_examples() {
        assert_eq!(sln_kernel(30000.0, 7500.0, 10.0), Ok(2250.0));
        match syd_kernel(30000.0, 7500.0, 10.0, 1.0).unwrap() {
            n => assert!((n - 4090.909090909091).abs() < 1.0e-9),
        }
        match syd_kernel(30000.0, 7500.0, 10.0, 10.0).unwrap() {
            n => assert!((n - 409.09090909090907).abs() < 1.0e-9),
        }
    }

    #[test]
    fn db_matches_support_examples() {
        assert!(
            (db_kernel(1_000_000.0, 100_000.0, 6.0, 1.0, 7.0).unwrap() - 186083.33333333334).abs()
                < 1.0e-9
        );
        assert!(
            (db_kernel(1_000_000.0, 100_000.0, 6.0, 2.0, 7.0).unwrap() - 259639.41666666666).abs()
                < 1.0e-9
        );
        assert!(
            (db_kernel(1_000_000.0, 100_000.0, 6.0, 7.0, 7.0).unwrap() - 15845.098473848071).abs()
                < 1.0e-9
        );
    }

    #[test]
    fn ddb_and_vdb_match_seeded_examples() {
        assert_eq!(ddb_kernel(2400.0, 300.0, 10.0, 1.0, 2.0), Ok(480.0));
        assert_eq!(ddb_kernel(2400.0, 300.0, 10.0, 2.0, 2.0), Ok(384.0));
        assert!(
            (vdb_kernel(2400.0, 300.0, 3650.0, 0.0, 1.0, 2.0, false).unwrap() - 1.3150684931506849)
                .abs()
                < 1.0e-12
        );
        assert!(
            (vdb_kernel(2400.0, 300.0, 120.0, 0.0, 1.0, 2.0, false).unwrap() - 40.0).abs()
                < 1.0e-12
        );
        assert!(
            vdb_kernel(2400.0, 300.0, 120.0, 6.0, 18.0, 2.0, false)
                .unwrap()
                .to_bits()
                == 0x4078_c4e5_981b_af06
        );
        assert!(
            (vdb_kernel(2400.0, 300.0, 120.0, 6.0, 18.0, 1.5, false).unwrap() - 311.8089366582341)
                .abs()
                < 1.0e-9
        );
        assert!(
            (vdb_kernel(2400.0, 300.0, 10.0, 0.0, 0.875, 1.5, false).unwrap() - 315.0).abs()
                < 1.0e-12
        );
    }

    #[test]
    fn surface_evaluators_apply_defaults_and_numeric_coercion() {
        let resolver = NoRefResolver;
        assert_eq!(
            eval_sln_surface(&[num(30000.0), num(7500.0), num(10.0)], &resolver),
            Ok(CalcValue::number(2250.0))
        );
        assert_eq!(
            eval_ddb_surface(&[num(2400.0), num(300.0), num(10.0), num(1.0)], &resolver),
            Ok(CalcValue::number(480.0))
        );
        let got = eval_vdb_surface(
            &[
                num(2400.0),
                num(300.0),
                num(10.0),
                num(0.0),
                num(0.875),
                num(1.5),
            ],
            &resolver,
        )
        .unwrap();
        match got.core() {
            CoreValue::Number(n) => assert!((*n - 315.0).abs() < 1.0e-12),
            other => panic!("expected number, got {other:?}"),
        }
        assert_eq!(
            eval_db_surface(
                &[
                    num(1_000_000.0),
                    num(100_000.0),
                    num(6.0),
                    num(1.0),
                    text("7")
                ],
                &resolver,
            ),
            Ok(CalcValue::number(186083.33333333334))
        );
    }

    #[test]
    fn domain_and_mapping_lanes_are_pinned() {
        let resolver = NoRefResolver;
        assert_eq!(
            sln_kernel(30000.0, 7500.0, 0.0),
            Err(WorksheetErrorCode::Div0)
        );
        assert_eq!(
            syd_kernel(30000.0, 7500.0, 10.0, 11.0),
            Err(WorksheetErrorCode::Num)
        );
        assert_eq!(
            db_kernel(1000.0, 100.0, 6.0, 1.0, 13.0),
            Err(WorksheetErrorCode::Num)
        );
        assert_eq!(
            vdb_kernel(2400.0, 300.0, 10.0, 2.0, 1.0, 2.0, false),
            Err(WorksheetErrorCode::Num)
        );
        assert_eq!(
            map_depreciation_error_to_ws(&DepreciationEvalError::ArityMismatch {
                expected_min: 3,
                expected_max: 3,
                actual: 2,
            }),
            WorksheetErrorCode::Value
        );
        assert_eq!(
            eval_sln_surface(&[num(30000.0), num(7500.0)], &resolver),
            Err(DepreciationEvalError::ArityMismatch {
                expected_min: 3,
                expected_max: 3,
                actual: 2,
            })
        );
    }

    #[test]
    fn no_switch_flag_seed_lane_is_currently_equal() {
        let switched = vdb_kernel(2400.0, 300.0, 10.0, 6.0, 8.0, 2.0, false).unwrap();
        let pure_declining = vdb_kernel(2400.0, 300.0, 10.0, 6.0, 8.0, 2.0, true).unwrap();
        assert!((switched - pure_declining).abs() < 1.0e-12);
    }

    #[test]
    fn no_ref_resolver_reference_lane_stays_unresolved() {
        let resolver = NoRefResolver;
        let got = eval_sln_surface(
            &[
                CalcValue::reference(ReferenceLike::new(ReferenceKind::A1, "A1".to_string())),
                num(0.0),
                num(10.0),
            ],
            &resolver,
        );
        assert_eq!(
            got,
            Err(DepreciationEvalError::Coercion(
                CoercionError::RefResolution(
                    crate::resolver::ReferenceResolutionError::UnresolvedReference {
                        target: "A1".to_string(),
                    }
                )
            ))
        );
    }
}

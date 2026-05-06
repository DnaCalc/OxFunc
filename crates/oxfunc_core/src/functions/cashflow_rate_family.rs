use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::resolver::{ReferenceResolver, resolve_eval_value};
use crate::value::{ArrayCellValue, CallArgValue, EvalValue, WorksheetErrorCode};

const ROOT_TOLERANCE: f64 = 1e-8;
const ROOT_DERIVATIVE_EPS: f64 = 1e-12;
const ROOT_MAX_ITERATIONS: usize = 100;
const MIN_VALID_RATE: f64 = -0.999_999_999;
const IRR_PUBLICATION_ULP_SCAN_RADIUS: usize = 16;
const XIRR_TWO_CASHFLOW_RELATIVE_BRACKET_TOLERANCE: f64 = 2e-8;
const XIRR_TWO_CASHFLOW_MAX_BRACKET_EXPANSIONS: usize = 128;
const XIRR_GENERAL_POSITIVE_ROOT_BRENT_EPS: f64 = 1e-8;
const XIRR_GENERAL_POSITIVE_ROOT_MAX_BRACKET_EXPANSIONS: usize = 128;

const CASHFLOW_RATE_BASE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.CASHFLOW_RATE_BASE",
    arity: Arity::exact(1),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::RefsVisibleInAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::RefOnly,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
};

pub const IRR_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.IRR",
    arity: Arity { min: 1, max: 2 },
    ..CASHFLOW_RATE_BASE_META
};

pub const XNPV_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.XNPV",
    arity: Arity::exact(3),
    ..CASHFLOW_RATE_BASE_META
};

pub const XIRR_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.XIRR",
    arity: Arity { min: 2, max: 3 },
    ..CASHFLOW_RATE_BASE_META
};

#[derive(Debug, Clone, PartialEq)]
pub enum CashflowRateEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
    Domain(WorksheetErrorCode),
}

fn arity_error(meta: &FunctionMeta, actual: usize) -> CashflowRateEvalError {
    CashflowRateEvalError::ArityMismatch {
        expected_min: meta.arity.min,
        expected_max: meta.arity.max,
        actual,
    }
}

fn resolve_arg_eval(
    arg: &CallArgValue,
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, CashflowRateEvalError> {
    match arg {
        CallArgValue::Reference(reference)
        | CallArgValue::Eval(EvalValue::Reference(reference)) => {
            resolve_eval_value(resolver, reference)
                .map_err(CoercionError::RefResolution)
                .map_err(CashflowRateEvalError::Coercion)
        }
        CallArgValue::Eval(value) => Ok(value.clone()),
        CallArgValue::MissingArg => Err(CashflowRateEvalError::Coercion(CoercionError::MissingArg)),
        CallArgValue::EmptyCell => Err(CashflowRateEvalError::Domain(WorksheetErrorCode::Value)),
    }
}

fn optional_arg_value(
    arg: Option<&CallArgValue>,
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<Option<EvalValue>, CashflowRateEvalError> {
    match arg {
        None | Some(CallArgValue::MissingArg) => Ok(None),
        Some(other) => Ok(Some(resolve_arg_eval(other, resolver)?)),
    }
}

fn numeric_from_cell(cell: &ArrayCellValue) -> Result<f64, CashflowRateEvalError> {
    match cell {
        ArrayCellValue::Number(n) => Ok(*n),
        ArrayCellValue::Error(code) => Err(CashflowRateEvalError::Domain(*code)),
        ArrayCellValue::Text(_) | ArrayCellValue::Logical(_) | ArrayCellValue::EmptyCell => {
            Err(CashflowRateEvalError::Domain(WorksheetErrorCode::Value))
        }
    }
}

fn scalar_number_from_eval(value: &EvalValue) -> Result<f64, CashflowRateEvalError> {
    match value {
        EvalValue::Number(n) => Ok(*n),
        EvalValue::Logical(flag) => Ok(if *flag { 1.0 } else { 0.0 }),
        EvalValue::Error(code) => Err(CashflowRateEvalError::Domain(*code)),
        EvalValue::Array(array) => {
            let shape = array.shape();
            if shape.rows == 1 && shape.cols == 1 {
                numeric_from_cell(array.get(0, 0).expect("single cell"))
            } else {
                Err(CashflowRateEvalError::Domain(WorksheetErrorCode::Value))
            }
        }
        EvalValue::Text(_) | EvalValue::Reference(_) | EvalValue::Lambda(_) => {
            Err(CashflowRateEvalError::Domain(WorksheetErrorCode::Value))
        }
    }
}

fn collect_numeric_vector_from_eval(value: &EvalValue) -> Result<Vec<f64>, CashflowRateEvalError> {
    match value {
        EvalValue::Number(n) => Ok(vec![*n]),
        EvalValue::Error(code) => Err(CashflowRateEvalError::Domain(*code)),
        EvalValue::Array(array) => {
            let shape = array.shape();
            if shape.rows > 1 && shape.cols > 1 {
                return Err(CashflowRateEvalError::Domain(WorksheetErrorCode::Ref));
            }
            let mut out = Vec::with_capacity(shape.rows * shape.cols);
            for cell in array.iter_row_major() {
                out.push(numeric_from_cell(cell)?);
            }
            Ok(out)
        }
        EvalValue::Text(_)
        | EvalValue::Logical(_)
        | EvalValue::Reference(_)
        | EvalValue::Lambda(_) => Err(CashflowRateEvalError::Domain(WorksheetErrorCode::Value)),
    }
}

fn serial_from_number(value: f64) -> Result<i64, CashflowRateEvalError> {
    if !value.is_finite() {
        return Err(CashflowRateEvalError::Domain(WorksheetErrorCode::Num));
    }
    let serial = value.trunc() as i64;
    if serial < 0 {
        return Err(CashflowRateEvalError::Domain(WorksheetErrorCode::Num));
    }
    Ok(serial)
}

fn collect_serial_vector_from_eval(value: &EvalValue) -> Result<Vec<i64>, CashflowRateEvalError> {
    match value {
        EvalValue::Number(n) => Ok(vec![serial_from_number(*n)?]),
        EvalValue::Error(code) => Err(CashflowRateEvalError::Domain(*code)),
        EvalValue::Array(array) => {
            let shape = array.shape();
            if shape.rows > 1 && shape.cols > 1 {
                return Err(CashflowRateEvalError::Domain(WorksheetErrorCode::Ref));
            }
            let mut out = Vec::with_capacity(shape.rows * shape.cols);
            for cell in array.iter_row_major() {
                out.push(serial_from_number(numeric_from_cell(cell)?)?);
            }
            Ok(out)
        }
        EvalValue::Text(_)
        | EvalValue::Logical(_)
        | EvalValue::Reference(_)
        | EvalValue::Lambda(_) => Err(CashflowRateEvalError::Domain(WorksheetErrorCode::Value)),
    }
}

fn collect_numeric_vector_arg(
    arg: &CallArgValue,
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<Vec<f64>, CashflowRateEvalError> {
    let eval = resolve_arg_eval(arg, resolver)?;
    collect_numeric_vector_from_eval(&eval)
}

fn collect_serial_vector_arg(
    arg: &CallArgValue,
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<Vec<i64>, CashflowRateEvalError> {
    let eval = resolve_arg_eval(arg, resolver)?;
    collect_serial_vector_from_eval(&eval)
}

fn validate_cashflows(cashflows: &[f64]) -> Result<(), WorksheetErrorCode> {
    if cashflows.len() < 2 {
        return Err(WorksheetErrorCode::Num);
    }
    if !cashflows.iter().all(|value| value.is_finite()) {
        return Err(WorksheetErrorCode::Value);
    }
    let has_positive = cashflows.iter().any(|value| *value > 0.0);
    let has_negative = cashflows.iter().any(|value| *value < 0.0);
    if !has_positive || !has_negative {
        return Err(WorksheetErrorCode::Num);
    }
    Ok(())
}

fn validate_xcashflow_inputs(values: &[f64], dates: &[i64]) -> Result<(), WorksheetErrorCode> {
    validate_cashflows(values)?;
    if values.len() != dates.len() || dates.is_empty() {
        return Err(WorksheetErrorCode::Num);
    }
    let anchor = dates[0];
    if dates.iter().any(|date| *date < anchor) {
        return Err(WorksheetErrorCode::Num);
    }
    Ok(())
}

fn periodic_npv_with_t0(rate: f64, cashflows: &[f64]) -> Result<f64, WorksheetErrorCode> {
    validate_cashflows(cashflows)?;
    if !rate.is_finite() || rate <= MIN_VALID_RATE {
        return Err(WorksheetErrorCode::Num);
    }
    let base = 1.0 + rate;
    let mut total = 0.0;
    for (idx, cashflow) in cashflows.iter().enumerate() {
        total += *cashflow / base.powf(idx as f64);
    }
    if total.is_finite() {
        Ok(total)
    } else {
        Err(WorksheetErrorCode::Num)
    }
}

fn periodic_npv_derivative(rate: f64, cashflows: &[f64]) -> Result<f64, WorksheetErrorCode> {
    validate_cashflows(cashflows)?;
    if !rate.is_finite() || rate <= MIN_VALID_RATE {
        return Err(WorksheetErrorCode::Num);
    }
    let base = 1.0 + rate;
    let mut total = 0.0;
    for (idx, cashflow) in cashflows.iter().enumerate().skip(1) {
        let period = idx as f64;
        total += -period * *cashflow / base.powf(period + 1.0);
    }
    if total.is_finite() {
        Ok(total)
    } else {
        Err(WorksheetErrorCode::Num)
    }
}

fn xnpv_kernel_raw(rate: f64, values: &[f64], dates: &[i64]) -> Result<f64, WorksheetErrorCode> {
    validate_xcashflow_inputs(values, dates)?;
    if !rate.is_finite() || rate <= MIN_VALID_RATE {
        return Err(WorksheetErrorCode::Num);
    }
    let base = 1.0 + rate;
    let anchor = dates[0];
    let mut total = 0.0;
    for (value, date) in values.iter().zip(dates.iter()) {
        let years = (*date - anchor) as f64 / 365.0;
        total += *value / base.powf(years);
    }
    if total.is_finite() {
        Ok(total)
    } else {
        Err(WorksheetErrorCode::Num)
    }
}

fn xnpv_derivative(rate: f64, values: &[f64], dates: &[i64]) -> Result<f64, WorksheetErrorCode> {
    validate_xcashflow_inputs(values, dates)?;
    if !rate.is_finite() || rate <= MIN_VALID_RATE {
        return Err(WorksheetErrorCode::Num);
    }
    let base = 1.0 + rate;
    let anchor = dates[0];
    let mut total = 0.0;
    for (value, date) in values.iter().zip(dates.iter()) {
        let years = (*date - anchor) as f64 / 365.0;
        if years == 0.0 {
            continue;
        }
        total += -years * *value / base.powf(years + 1.0);
    }
    if total.is_finite() {
        Ok(total)
    } else {
        Err(WorksheetErrorCode::Num)
    }
}

fn next_up(rate: f64) -> f64 {
    if rate.is_nan() || rate == f64::INFINITY {
        rate
    } else if rate == -0.0 {
        0.0
    } else {
        let bits = rate.to_bits();
        if rate >= 0.0 {
            f64::from_bits(bits + 1)
        } else {
            f64::from_bits(bits - 1)
        }
    }
}

fn next_down(rate: f64) -> f64 {
    if rate.is_nan() || rate == f64::NEG_INFINITY {
        rate
    } else if rate == 0.0 {
        -0.0
    } else {
        let bits = rate.to_bits();
        if rate > 0.0 {
            f64::from_bits(bits - 1)
        } else {
            f64::from_bits(bits + 1)
        }
    }
}

fn midpoint_of_local_min_abs_residual_plateau<F>(
    root: f64,
    mut f: F,
) -> Result<f64, WorksheetErrorCode>
where
    F: FnMut(f64) -> Result<f64, WorksheetErrorCode>,
{
    let mut best_rate = root;
    let mut best_abs_bits = f(root)?.abs().to_bits();

    let mut lower_probe = root;
    let mut upper_probe = root;
    for _ in 0..IRR_PUBLICATION_ULP_SCAN_RADIUS {
        lower_probe = next_down(lower_probe);
        if lower_probe.is_finite() {
            let abs_bits = f(lower_probe)?.abs().to_bits();
            if abs_bits < best_abs_bits {
                best_abs_bits = abs_bits;
                best_rate = lower_probe;
            }
        }

        upper_probe = next_up(upper_probe);
        if upper_probe.is_finite() {
            let abs_bits = f(upper_probe)?.abs().to_bits();
            if abs_bits < best_abs_bits {
                best_abs_bits = abs_bits;
                best_rate = upper_probe;
            }
        }
    }

    let mut lower = best_rate;
    loop {
        let candidate = next_down(lower);
        if !candidate.is_finite() || f(candidate)?.abs().to_bits() != best_abs_bits {
            break;
        }
        lower = candidate;
    }

    let mut upper = best_rate;
    loop {
        let candidate = next_up(upper);
        if !candidate.is_finite() || f(candidate)?.abs().to_bits() != best_abs_bits {
            break;
        }
        upper = candidate;
    }

    let mut width = 0usize;
    let mut cursor = lower;
    while cursor.to_bits() != upper.to_bits() {
        cursor = next_up(cursor);
        width += 1;
    }

    let mut midpoint = lower;
    for _ in 0..(width / 2) {
        midpoint = next_up(midpoint);
    }
    Ok(midpoint)
}

fn bounded_rate_solve<F, D>(
    guess: f64,
    mut f: F,
    mut derivative: D,
) -> Result<f64, WorksheetErrorCode>
where
    F: FnMut(f64) -> Result<f64, WorksheetErrorCode>,
    D: FnMut(f64) -> Result<f64, WorksheetErrorCode>,
{
    let mut prev_rate = 0.0;
    let mut prev_value = f(prev_rate)?;
    let mut current_rate = if guess <= MIN_VALID_RATE { 0.1 } else { guess };
    let mut current_value = f(current_rate)?;

    for _ in 0..ROOT_MAX_ITERATIONS {
        if current_value.abs() <= ROOT_TOLERANCE {
            return Ok(current_rate);
        }

        let secant_next = if (current_value - prev_value).abs() > ROOT_DERIVATIVE_EPS {
            let next = current_rate
                - current_value * (current_rate - prev_rate) / (current_value - prev_value);
            (next.is_finite() && next > MIN_VALID_RATE).then_some(next)
        } else {
            None
        };

        let newton_next = if let Ok(deriv) = derivative(current_rate) {
            if deriv.abs() > ROOT_DERIVATIVE_EPS {
                let next = current_rate - current_value / deriv;
                (next.is_finite() && next > MIN_VALID_RATE).then_some(next)
            } else {
                None
            }
        } else {
            None
        };

        let next_rate = secant_next.or(newton_next).unwrap_or_else(|| {
            if current_rate > 0.0 {
                current_rate / 2.0
            } else {
                0.1
            }
        });

        prev_rate = current_rate;
        prev_value = current_value;
        current_rate = if next_rate <= MIN_VALID_RATE {
            0.1
        } else {
            next_rate
        };
        current_value = f(current_rate)?;
    }

    if current_value.abs() <= ROOT_TOLERANCE {
        Ok(current_rate)
    } else {
        Err(WorksheetErrorCode::Num)
    }
}

fn irr_kernel(cashflows: &[f64], guess: Option<f64>) -> Result<f64, WorksheetErrorCode> {
    validate_cashflows(cashflows)?;
    let root = bounded_rate_solve(
        guess.unwrap_or(0.1),
        |rate| periodic_npv_with_t0(rate, cashflows),
        |rate| periodic_npv_derivative(rate, cashflows),
    )?;
    midpoint_of_local_min_abs_residual_plateau(root, |rate| periodic_npv_with_t0(rate, cashflows))
}

pub fn xnpv_kernel(rate: f64, values: &[f64], dates: &[i64]) -> Result<f64, WorksheetErrorCode> {
    if rate < 0.0 {
        return Err(WorksheetErrorCode::Num);
    }
    xnpv_kernel_raw(rate, values, dates)
}

fn xirr_two_cashflow_root(values: &[f64], dates: &[i64]) -> Result<f64, WorksheetErrorCode> {
    if values.len() != 2 || dates.len() != 2 {
        return Err(WorksheetErrorCode::Num);
    }
    let first = values[0];
    let second = values[1];
    let years = (dates[1] - dates[0]) as f64 / 365.0;
    if years <= 0.0 || first == 0.0 {
        return Err(WorksheetErrorCode::Num);
    }
    let ratio = -second / first;
    if !ratio.is_finite() || ratio <= 0.0 {
        return Err(WorksheetErrorCode::Num);
    }
    let root = ratio.powf(1.0 / years) - 1.0;
    if root.is_finite() && root > MIN_VALID_RATE {
        Ok(root)
    } else {
        Err(WorksheetErrorCode::Num)
    }
}

fn xirr_two_cashflow_positive_root_excel_like(
    values: &[f64],
    dates: &[i64],
    guess: f64,
) -> Result<f64, WorksheetErrorCode> {
    if !guess.is_finite() || guess <= 0.0 {
        return Err(WorksheetErrorCode::Num);
    }

    let mut lower = 0.0;
    let mut lower_value = xnpv_kernel_raw(lower, values, dates)?;
    let mut upper = guess;
    let mut upper_value = xnpv_kernel_raw(upper, values, dates)?;

    if lower_value == 0.0 {
        return Ok(lower);
    }
    if upper_value == 0.0 {
        return Ok(upper);
    }

    let mut expansions = 0usize;
    while lower_value.signum() == upper_value.signum() {
        lower = upper;
        lower_value = upper_value;
        upper *= 2.0;
        if !upper.is_finite() || upper <= MIN_VALID_RATE {
            return Err(WorksheetErrorCode::Num);
        }
        upper_value = xnpv_kernel_raw(upper, values, dates)?;
        expansions += 1;
        if expansions > XIRR_TWO_CASHFLOW_MAX_BRACKET_EXPANSIONS {
            return Err(WorksheetErrorCode::Num);
        }
    }

    for _ in 0..ROOT_MAX_ITERATIONS {
        let midpoint = (lower + upper) / 2.0;
        let relative_width = (upper - lower) / midpoint.abs();
        if relative_width <= XIRR_TWO_CASHFLOW_RELATIVE_BRACKET_TOLERANCE {
            return Ok(midpoint);
        }

        let midpoint_value = xnpv_kernel_raw(midpoint, values, dates)?;
        if midpoint_value == 0.0 {
            return Ok(midpoint);
        }
        if lower_value.signum() == midpoint_value.signum() {
            lower = midpoint;
            lower_value = midpoint_value;
        } else {
            upper = midpoint;
        }
    }

    Err(WorksheetErrorCode::Num)
}

fn xirr_general_positive_root_excel_like(
    values: &[f64],
    dates: &[i64],
    guess: f64,
) -> Result<f64, WorksheetErrorCode> {
    if !guess.is_finite() || guess <= 0.0 {
        return Err(WorksheetErrorCode::Num);
    }

    let zero = 0.0;
    let zero_value = xnpv_kernel_raw(zero, values, dates)?;
    if zero_value == 0.0 {
        return Ok(zero);
    }

    let mut lower = guess;
    let mut lower_value = xnpv_kernel_raw(lower, values, dates)?;
    if lower_value == 0.0 {
        return Ok(lower);
    }

    let mut upper;

    if zero_value.signum() != lower_value.signum() {
        lower = zero;
        lower_value = zero_value;
        upper = guess;
        let upper_value = xnpv_kernel_raw(upper, values, dates)?;
        if upper_value == 0.0 {
            return Ok(upper);
        }
    } else {
        upper = guess * 2.0;
        if !upper.is_finite() || upper <= MIN_VALID_RATE {
            return Err(WorksheetErrorCode::Num);
        }
        let mut upper_value = xnpv_kernel_raw(upper, values, dates)?;

        let mut expansions = 0usize;
        while lower_value.signum() == upper_value.signum() {
            lower = upper;
            lower_value = upper_value;
            upper *= 2.0;
            if !upper.is_finite() || upper <= MIN_VALID_RATE {
                return Err(WorksheetErrorCode::Num);
            }
            upper_value = xnpv_kernel_raw(upper, values, dates)?;
            expansions += 1;
            if expansions > XIRR_GENERAL_POSITIVE_ROOT_MAX_BRACKET_EXPANSIONS {
                return Err(WorksheetErrorCode::Num);
            }
        }
    }

    for _ in 0..ROOT_MAX_ITERATIONS {
        let midpoint = (lower + upper) / 2.0;
        let bracket_width = upper - lower;
        let brent_tol = XIRR_GENERAL_POSITIVE_ROOT_BRENT_EPS * (1.0 + 2.0 * midpoint.abs());
        if bracket_width <= brent_tol {
            return Ok(midpoint);
        }

        let midpoint_value = xnpv_kernel_raw(midpoint, values, dates)?;
        if midpoint_value == 0.0 {
            return Ok(midpoint);
        }
        if lower_value.signum() == midpoint_value.signum() {
            lower = midpoint;
            lower_value = midpoint_value;
        } else {
            upper = midpoint;
        }
    }

    Err(WorksheetErrorCode::Num)
}

fn xirr_kernel(
    values: &[f64],
    dates: &[i64],
    guess: Option<f64>,
) -> Result<f64, WorksheetErrorCode> {
    validate_xcashflow_inputs(values, dates)?;
    let guess = guess.unwrap_or(0.1);
    if values.len() == 2 && dates.len() == 2 {
        let root = xirr_two_cashflow_root(values, dates)?;
        if root >= 0.0 && guess < 0.0 {
            return Err(WorksheetErrorCode::Num);
        }
        if root >= 0.0 {
            return xirr_two_cashflow_positive_root_excel_like(values, dates, guess);
        }
        return Ok(root);
    }

    if guess > 0.0 {
        if let Ok(root) = xirr_general_positive_root_excel_like(values, dates, guess) {
            return Ok(root);
        }
    }

    bounded_rate_solve(
        guess,
        |rate| xnpv_kernel_raw(rate, values, dates),
        |rate| xnpv_derivative(rate, values, dates),
    )
}

pub fn eval_irr_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, CashflowRateEvalError> {
    if !IRR_META.arity.accepts(args.len()) {
        return Err(arity_error(&IRR_META, args.len()));
    }
    let cashflows = collect_numeric_vector_arg(&args[0], resolver)?;
    let guess = optional_arg_value(args.get(1), resolver)?
        .map(|value| scalar_number_from_eval(&value))
        .transpose()?;
    irr_kernel(&cashflows, guess)
        .map(EvalValue::Number)
        .map_err(CashflowRateEvalError::Domain)
}

pub fn eval_xnpv_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, CashflowRateEvalError> {
    if !XNPV_META.arity.accepts(args.len()) {
        return Err(arity_error(&XNPV_META, args.len()));
    }
    let rate = scalar_number_from_eval(&resolve_arg_eval(&args[0], resolver)?)?;
    let values = collect_numeric_vector_arg(&args[1], resolver)?;
    let dates = collect_serial_vector_arg(&args[2], resolver)?;
    xnpv_kernel(rate, &values, &dates)
        .map(EvalValue::Number)
        .map_err(CashflowRateEvalError::Domain)
}

pub fn eval_xirr_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, CashflowRateEvalError> {
    if !XIRR_META.arity.accepts(args.len()) {
        return Err(arity_error(&XIRR_META, args.len()));
    }
    let values = collect_numeric_vector_arg(&args[0], resolver)?;
    let dates = collect_serial_vector_arg(&args[1], resolver)?;
    let guess = optional_arg_value(args.get(2), resolver)?
        .map(|value| scalar_number_from_eval(&value))
        .transpose()?;
    xirr_kernel(&values, &dates, guess)
        .map(EvalValue::Number)
        .map_err(CashflowRateEvalError::Domain)
}

pub fn map_cashflow_rate_error_to_ws(error: &CashflowRateEvalError) -> WorksheetErrorCode {
    match error {
        CashflowRateEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        CashflowRateEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        CashflowRateEvalError::Coercion(_) => WorksheetErrorCode::Value,
        CashflowRateEvalError::Domain(code) => *code,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::{RefResolutionError, ResolverCapabilities};
    use crate::value::{EvalArray, ReferenceKind, ReferenceLike};
    use std::collections::BTreeMap;

    struct MockResolver {
        map: BTreeMap<String, EvalValue>,
    }

    impl ReferenceResolver for MockResolver {
        fn capabilities(&self) -> ResolverCapabilities {
            ResolverCapabilities::permissive_local()
        }

        fn resolve_reference(
            &self,
            reference: &ReferenceLike,
        ) -> Result<EvalValue, RefResolutionError> {
            self.map.get(&reference.target).cloned().ok_or_else(|| {
                RefResolutionError::UnresolvedReference {
                    target: reference.target.clone(),
                }
            })
        }
    }

    fn col(values: &[f64]) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Array(
            EvalArray::from_rows(
                values
                    .iter()
                    .copied()
                    .map(|n| vec![ArrayCellValue::Number(n)])
                    .collect(),
            )
            .unwrap(),
        ))
    }

    fn num(n: f64) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Number(n))
    }

    fn ref_arg(target: &str) -> CallArgValue {
        CallArgValue::Reference(ReferenceLike {
            kind: ReferenceKind::Area,
            target: target.to_string(),
        })
    }

    fn assert_close(left: f64, right: f64) {
        assert!((left - right).abs() < 1e-8, "left={left}, right={right}");
    }

    fn assert_bits(actual: f64, expected: f64) {
        assert_eq!(
            actual.to_bits(),
            expected.to_bits(),
            "{actual} vs {expected}"
        );
    }

    #[test]
    fn metadata_matches_batch_shape() {
        assert_eq!(IRR_META.function_id, "FUNC.IRR");
        assert_eq!(IRR_META.arity, Arity { min: 1, max: 2 });
        assert_eq!(XNPV_META.arity, Arity::exact(3));
        assert_eq!(
            XIRR_META.surface_fec_dependency_profile,
            FecDependencyProfile::RefOnly
        );
    }

    #[test]
    fn irr_exactness_witness_matches_excel_target() {
        let actual = irr_kernel(&[-10000.0, 3000.0, 4200.0, 6800.0], None).expect("irr witness");
        let prior_local = 0.1634056006889894_f64;
        let excel_target = 0.16340560068898924_f64;

        assert_bits(actual, excel_target);
        assert_ne!(actual.to_bits(), prior_local.to_bits());
    }

    #[test]
    fn irr_matches_simple_two_cashflow_identity() {
        let got = irr_kernel(&[-100.0, 121.0], None).unwrap();
        assert_close(got, 0.21);
    }

    #[test]
    fn xnpv_matches_one_year_discount_identity() {
        let got = xnpv_kernel(0.1, &[-100.0, 121.0], &[45000, 45365]).unwrap();
        assert_close(got, 10.0);
    }

    #[test]
    fn xnpv_rejects_negative_rate_on_surface() {
        assert_eq!(
            xnpv_kernel(
                -0.1,
                &[206_101_714.849_377, -156_650_972.542_65],
                &[36584, 36615]
            ),
            Err(WorksheetErrorCode::Num)
        );
    }

    #[test]
    fn xirr_matches_simple_one_year_identity() {
        let got = xirr_kernel(&[-100.0, 121.0], &[45000, 45365], None).unwrap();
        assert_close(got, 0.21);
    }

    #[test]
    fn xirr_simple_positive_root_matches_exact_excel_publication_witnesses() {
        assert_close(
            xirr_kernel(&[-100.0, 121.0], &[45000, 45365], None).unwrap(),
            0.209_999_996_423_721_44,
        );
        assert_close(
            xirr_kernel(&[-100.0, 121.0], &[45000, 45365], Some(0.5)).unwrap(),
            0.209_999_997_168_779,
        );
    }

    #[test]
    fn xnpv_rejects_dates_before_anchor() {
        assert_eq!(
            xnpv_kernel(0.1, &[-100.0, 121.0], &[45000, 44999]),
            Err(WorksheetErrorCode::Num)
        );
    }

    #[test]
    fn xirr_requires_sign_change() {
        assert_eq!(
            xirr_kernel(&[100.0, 121.0], &[45000, 45365], None),
            Err(WorksheetErrorCode::Num)
        );
    }

    #[test]
    fn xirr_finds_negative_root_when_it_is_the_only_root() {
        let got = xirr_kernel(
            &[206_101_714.849_377, -156_650_972.542_65],
            &[36585, 36616],
            Some(-0.1),
        )
        .unwrap();
        assert_close(got, -0.960_452_189_296_483_9);
    }

    #[test]
    fn xirr_negative_guess_does_not_jump_to_positive_root_only_domain() {
        assert_eq!(
            xirr_kernel(
                &[15_108_163.384_092_3, -75_382_259.662_842_4],
                &[36585, 36616],
                Some(-0.1),
            ),
            Err(WorksheetErrorCode::Num)
        );
    }

    #[test]
    fn xirr_two_cashflow_positive_root_matches_excel_guess_matrix() {
        let values = [15_108_163.384_092_3, -75_382_259.662_842_4];
        let dates = [36585, 36616];
        let cases = [
            (0.0001, 165_601_347.174_400_03),
            (0.01, 165_601_345.280_000_06),
            (0.1, 165_601_345.600_000_05),
            (1.0, 165_601_347.0),
            (10.0, 165_601_346.25),
            (100.0, 165_601_345.312_5),
            (1000.0, 165_601_346.679_687_5),
        ];

        for (guess, expected) in cases {
            let got = xirr_kernel(&values, &dates, Some(guess)).unwrap();
            assert_close(got, expected);
        }
    }

    #[test]
    fn xirr_general_positive_root_matches_excel_guess_matrix() {
        let values = [-10_000.0, 2_750.0, 4_250.0, 3_250.0, 2_750.0];
        let dates = [44927, 45108, 45292, 45473, 45658];
        let cases = [
            (0.01, 0.244_491_829_872_131),
            (0.1, 0.244_491_833_448_41),
            (0.5, 0.244_491_834_193_468),
            (1.0, 0.244_491_834_193_468),
        ];

        for (guess, expected) in cases {
            assert_close(xirr_kernel(&values, &dates, Some(guess)).unwrap(), expected);
        }
    }

    fn assert_exact(got: f64, expected: f64, label: &str) {
        let diff = (got - expected).abs();
        assert!(
            diff < 1e-14,
            "{label}: got={got:.17e}, expected={expected:.17e}, diff={diff:.3e}"
        );
    }

    #[test]
    fn xirr_general_positive_root_matches_widened_excel_value2_matrix() {
        let cases: &[(&[f64], &[i64], f64, f64, &str)] = &[
            (
                &[-10_000.0, 2_750.0, 4_250.0, 3_250.0, 2_750.0],
                &[44927, 45108, 45292, 45473, 45658],
                0.01,
                0.244_491_829_872_131,
                "w087_seed g=0.01",
            ),
            (
                &[-10_000.0, 2_750.0, 4_250.0, 3_250.0, 2_750.0],
                &[44927, 45108, 45292, 45473, 45658],
                0.1,
                0.244_491_833_448_41,
                "w087_seed g=0.1",
            ),
            (
                &[-10_000.0, 2_750.0, 4_250.0, 3_250.0, 2_750.0],
                &[44927, 45108, 45292, 45473, 45658],
                0.5,
                0.244_491_834_193_468,
                "w087_seed g=0.5",
            ),
            (
                &[-10_000.0, 2_750.0, 4_250.0, 3_250.0, 2_750.0],
                &[44927, 45108, 45292, 45473, 45658],
                1.0,
                0.244_491_834_193_468,
                "w087_seed g=1.0",
            ),
            (
                &[-1_000.0, 300.0, 400.0, 500.0],
                &[45000, 45100, 45200, 45365],
                0.01,
                0.320_753_083_229_065,
                "adjacent_a g=0.01",
            ),
            (
                &[-1_000.0, 300.0, 400.0, 500.0],
                &[45000, 45100, 45200, 45365],
                0.1,
                0.320_753_079_652_786,
                "adjacent_a g=0.1",
            ),
            (
                &[-1_000.0, 300.0, 400.0, 500.0],
                &[45000, 45100, 45200, 45365],
                0.5,
                0.320_753_075_182_438,
                "adjacent_a g=0.5",
            ),
            (
                &[-1_000.0, 300.0, 400.0, 500.0],
                &[45000, 45100, 45200, 45365],
                1.0,
                0.320_753_075_182_438,
                "adjacent_a g=1.0",
            ),
            (
                &[-5_000.0, 1_500.0, 1_500.0, 1_500.0, 1_500.0],
                &[45000, 45090, 45180, 45270, 45360],
                0.01,
                0.351_695_961_952_21,
                "adjacent_b g=0.01",
            ),
            (
                &[-5_000.0, 1_500.0, 1_500.0, 1_500.0, 1_500.0],
                &[45000, 45090, 45180, 45270, 45360],
                0.1,
                0.351_695_960_760_117,
                "adjacent_b g=0.1",
            ),
            (
                &[-5_000.0, 1_500.0, 1_500.0, 1_500.0, 1_500.0],
                &[45000, 45090, 45180, 45270, 45360],
                0.5,
                0.351_695_962_250_233,
                "adjacent_b g=0.5",
            ),
            (
                &[-5_000.0, 1_500.0, 1_500.0, 1_500.0, 1_500.0],
                &[45000, 45090, 45180, 45270, 45360],
                1.0,
                0.351_695_962_250_233,
                "adjacent_b g=1.0",
            ),
            (
                &[-12_000.0, 1_000.0, 2_000.0, 3_000.0, 7_000.0],
                &[45000, 45150, 45300, 45450, 45600],
                0.01,
                0.062_376_437_187_194_8,
                "adjacent_c g=0.01",
            ),
            (
                &[-12_000.0, 1_000.0, 2_000.0, 3_000.0, 7_000.0],
                &[45000, 45150, 45300, 45450, 45600],
                0.1,
                0.062_376_442_551_612_9,
                "adjacent_c g=0.1",
            ),
            (
                &[-12_000.0, 1_000.0, 2_000.0, 3_000.0, 7_000.0],
                &[45000, 45150, 45300, 45450, 45600],
                0.5,
                0.062_376_443_296_670_9,
                "adjacent_c g=0.5",
            ),
            (
                &[-12_000.0, 1_000.0, 2_000.0, 3_000.0, 7_000.0],
                &[45000, 45150, 45300, 45450, 45600],
                1.0,
                0.062_376_443_296_670_9,
                "adjacent_c g=1.0",
            ),
            (
                &[-2_000.0, 2_500.0, 100.0],
                &[45000, 45180, 45365],
                0.01,
                0.672_046_403_884_887,
                "adjacent_d g=0.01",
            ),
            (
                &[-2_000.0, 2_500.0, 100.0],
                &[45000, 45180, 45365],
                0.1,
                0.672_046_405_076_981,
                "adjacent_d g=0.1",
            ),
            (
                &[-2_000.0, 2_500.0, 100.0],
                &[45000, 45180, 45365],
                0.5,
                0.672_046_400_606_632,
                "adjacent_d g=0.5",
            ),
            (
                &[-2_000.0, 2_500.0, 100.0],
                &[45000, 45180, 45365],
                1.0,
                0.672_046_400_606_632,
                "adjacent_d g=1.0",
            ),
            (
                &[-4_000.0, 500.0, 800.0, 1_200.0, 2_200.0],
                &[45000, 45045, 45120, 45210, 45400],
                0.01,
                0.253_582_987_785_339,
                "adjacent_e g=0.01",
            ),
            (
                &[-4_000.0, 500.0, 800.0, 1_200.0, 2_200.0],
                &[45000, 45045, 45120, 45210, 45400],
                0.1,
                0.253_582_996_129_99,
                "adjacent_e g=0.1",
            ),
            (
                &[-4_000.0, 500.0, 800.0, 1_200.0, 2_200.0],
                &[45000, 45045, 45120, 45210, 45400],
                0.5,
                0.253_582_991_659_641,
                "adjacent_e g=0.5",
            ),
            (
                &[-4_000.0, 500.0, 800.0, 1_200.0, 2_200.0],
                &[45000, 45045, 45120, 45210, 45400],
                1.0,
                0.253_582_991_659_641,
                "adjacent_e g=1.0",
            ),
        ];

        for &(values, dates, guess, expected, label) in cases {
            let got = xirr_kernel(values, dates, Some(guess)).unwrap();
            assert_exact(got, expected, label);
        }
    }

    #[test]
    fn xnpv_rejects_length_mismatch() {
        assert_eq!(
            xnpv_kernel(0.1, &[-100.0, 121.0], &[45000]),
            Err(WorksheetErrorCode::Num)
        );
    }

    #[test]
    fn surface_eval_resolves_reference_vectors() {
        let mut map = BTreeMap::new();
        map.insert(
            "A1:A2".to_string(),
            EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Number(-100.0)],
                    vec![ArrayCellValue::Number(121.0)],
                ])
                .unwrap(),
            ),
        );
        map.insert(
            "B1:B2".to_string(),
            EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Number(45000.0)],
                    vec![ArrayCellValue::Number(45365.0)],
                ])
                .unwrap(),
            ),
        );
        let got = eval_xirr_surface(&[ref_arg("A1:A2"), ref_arg("B1:B2")], &MockResolver { map })
            .unwrap();
        match got {
            EvalValue::Number(n) => assert_close(n, 0.21),
            other => panic!("expected scalar, got {other:?}"),
        }
    }

    #[test]
    fn matrix_input_is_explicitly_out_of_slice() {
        let matrix = CallArgValue::Eval(EvalValue::Array(
            EvalArray::from_rows(vec![
                vec![
                    ArrayCellValue::Number(-100.0),
                    ArrayCellValue::Number(121.0),
                ],
                vec![ArrayCellValue::Number(5.0), ArrayCellValue::Number(6.0)],
            ])
            .unwrap(),
        ));
        let error = eval_irr_surface(
            &[matrix],
            &MockResolver {
                map: BTreeMap::new(),
            },
        )
        .unwrap_err();
        assert_eq!(
            map_cashflow_rate_error_to_ws(&error),
            WorksheetErrorCode::Ref
        );
    }

    #[test]
    fn optional_guess_is_admitted_on_surface() {
        let got = eval_irr_surface(
            &[col(&[-100.0, 121.0]), num(0.5)],
            &MockResolver {
                map: BTreeMap::new(),
            },
        )
        .unwrap();
        match got {
            EvalValue::Number(n) => assert_close(n, 0.21),
            other => panic!("expected scalar, got {other:?}"),
        }
    }

    #[test]
    fn arity_errors_map_to_value() {
        let error = eval_xnpv_surface(
            &[],
            &MockResolver {
                map: BTreeMap::new(),
            },
        )
        .unwrap_err();
        assert_eq!(
            map_cashflow_rate_error_to_ws(&error),
            WorksheetErrorCode::Value
        );
    }
}

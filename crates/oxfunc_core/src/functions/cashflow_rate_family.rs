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
    resolver: &impl ReferenceResolver,
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
    resolver: &impl ReferenceResolver,
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
    resolver: &impl ReferenceResolver,
) -> Result<Vec<f64>, CashflowRateEvalError> {
    let eval = resolve_arg_eval(arg, resolver)?;
    collect_numeric_vector_from_eval(&eval)
}

fn collect_serial_vector_arg(
    arg: &CallArgValue,
    resolver: &impl ReferenceResolver,
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
    bounded_rate_solve(
        guess.unwrap_or(0.1),
        |rate| periodic_npv_with_t0(rate, cashflows),
        |rate| periodic_npv_derivative(rate, cashflows),
    )
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
        return Ok(root);
    }
    bounded_rate_solve(
        guess,
        |rate| xnpv_kernel_raw(rate, values, dates),
        |rate| xnpv_derivative(rate, values, dates),
    )
}

pub fn eval_irr_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
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
    resolver: &impl ReferenceResolver,
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
    resolver: &impl ReferenceResolver,
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

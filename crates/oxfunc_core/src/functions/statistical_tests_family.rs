use crate::coercion::{CoercionError, coerce_eval_to_number};
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{AggregateArgOrigin, AggregatePreparedItem, expand_aggregate_arg};
use crate::functions::chi_f_t_family::{chisq_dist_rt_kernel, f_dist_rt_kernel};
use crate::functions::special_math_common::regularized_beta;
use crate::resolver::{ReferenceSystemProvider, resolve_eval_value};
use crate::value::CalcValue;
use crate::value::{CalcArray, CoreValue, WorksheetErrorCode};

const BASE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.STATISTICAL.TESTS.BASE",
    arity: Arity::exact(2),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::RefsVisibleInAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    lift_broadcast_profile: FunctionMeta::DEFAULT_LIFT_BROADCAST_PROFILE,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::RefOnly,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
    real_result_policy: FunctionMeta::DEFAULT_REAL_RESULT_POLICY,
};

pub const CHISQ_TEST_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.CHISQ.TEST",
    arity: Arity::exact(2),
    ..BASE_META
};
pub const CHITEST_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.CHITEST",
    arity: Arity::exact(2),
    ..BASE_META
};
pub const F_TEST_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.F.TEST",
    arity: Arity::exact(2),
    ..BASE_META
};
pub const FTEST_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.FTEST",
    arity: Arity::exact(2),
    ..BASE_META
};
pub const T_TEST_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.T.TEST",
    arity: Arity::exact(4),
    ..BASE_META
};
pub const TTEST_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.TTEST",
    arity: Arity::exact(4),
    ..BASE_META
};

#[derive(Debug, Clone, PartialEq)]
pub enum StatisticalTestsEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
    Domain(WorksheetErrorCode),
}

fn arity_error(meta: &FunctionMeta, actual: usize) -> StatisticalTestsEvalError {
    StatisticalTestsEvalError::ArityMismatch {
        expected_min: meta.arity.min,
        expected_max: meta.arity.max,
        actual,
    }
}

fn resolve_arg_eval(
    arg: &CalcValue,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, StatisticalTestsEvalError> {
    match arg.core() {
        CoreValue::Reference(r) => resolve_eval_value(resolver, r)
            .map_err(CoercionError::RefResolution)
            .map_err(StatisticalTestsEvalError::Coercion),
        CoreValue::Missing => Err(StatisticalTestsEvalError::Coercion(
            CoercionError::MissingArg,
        )),
        CoreValue::Empty => Err(StatisticalTestsEvalError::Domain(WorksheetErrorCode::Value)),
        _ => Ok(arg.clone()),
    }
}

fn scalar_number_from_cell(cell: &CalcValue) -> Result<f64, StatisticalTestsEvalError> {
    match cell.core() {
        CoreValue::Number(n) => Ok(*n),
        CoreValue::Error(code) => Err(StatisticalTestsEvalError::Domain(*code)),
        CoreValue::Text(_)
        | CoreValue::Logical(_)
        | CoreValue::Empty
        | CoreValue::Missing
        | CoreValue::Array(_)
        | CoreValue::Reference(_) => {
            Err(StatisticalTestsEvalError::Domain(WorksheetErrorCode::Value))
        }
    }
}

fn scalar_number_from_eval(value: &CalcValue) -> Result<f64, StatisticalTestsEvalError> {
    match value.core() {
        CoreValue::Array(array) if array.shape().rows == 1 && array.shape().cols == 1 => {
            scalar_number_from_cell(array.get(0, 0).expect("single cell"))
        }
        CoreValue::Array(_) => Err(StatisticalTestsEvalError::Domain(WorksheetErrorCode::Value)),
        _ => coerce_eval_to_number(value, &NoResolver).map_err(StatisticalTestsEvalError::Coercion),
    }
}

fn truncated_flag_arg(
    arg: &CalcValue,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<i32, StatisticalTestsEvalError> {
    let number = scalar_number_from_eval(&resolve_arg_eval(arg, resolver)?)?;
    if !number.is_finite() {
        return Err(StatisticalTestsEvalError::Domain(WorksheetErrorCode::Num));
    }
    Ok(number.trunc() as i32)
}

fn eval_to_numeric_matrix(value: &CalcValue) -> Result<CalcArray, StatisticalTestsEvalError> {
    match value.core() {
        CoreValue::Number(n) => Ok(CalcArray::from_scalar(CalcValue::number(*n))
            .expect("scalar number array has valid dimensions")),
        CoreValue::Error(code) => Err(StatisticalTestsEvalError::Domain(*code)),
        CoreValue::Array(array) => {
            let mut rows = Vec::with_capacity(array.shape().rows);
            for row in 0..array.shape().rows {
                let mut out = Vec::with_capacity(array.shape().cols);
                for cell in array.row_slice(row).expect("row") {
                    out.push(CalcValue::number(scalar_number_from_cell(cell)?));
                }
                rows.push(out);
            }
            Ok(CalcArray::from_rows(rows).expect("shape"))
        }
        CoreValue::Text(_) | CoreValue::Logical(_) | CoreValue::Reference(_) => {
            Err(StatisticalTestsEvalError::Domain(WorksheetErrorCode::Value))
        }
        _ => Err(StatisticalTestsEvalError::Domain(WorksheetErrorCode::Value)),
    }
}

fn numeric_matrices_from_args(
    actual: &CalcValue,
    expected: &CalcValue,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<(CalcArray, CalcArray), StatisticalTestsEvalError> {
    let actual = eval_to_numeric_matrix(&resolve_arg_eval(actual, resolver)?)?;
    let expected = eval_to_numeric_matrix(&resolve_arg_eval(expected, resolver)?)?;
    let actual_cells = actual.shape().rows * actual.shape().cols;
    let expected_cells = expected.shape().rows * expected.shape().cols;
    if actual_cells != expected_cells || actual_cells == 1 {
        return Err(StatisticalTestsEvalError::Domain(WorksheetErrorCode::NA));
    }
    if actual.shape() == expected.shape() {
        return Ok((actual, expected));
    }
    let reshaped_expected =
        CalcArray::new(actual.shape(), expected.iter_row_major().cloned().collect())
            .expect("equal cardinality reshape");
    Ok((actual, reshaped_expected))
}

fn aggregate_item_number(
    item: &AggregatePreparedItem,
) -> Result<Option<f64>, StatisticalTestsEvalError> {
    match item.1 {
        AggregateArgOrigin::DirectScalar => match item.0.core() {
            CoreValue::Number(n) => Ok(Some(*n)),
            CoreValue::Error(code) => Err(StatisticalTestsEvalError::Domain(*code)),
            CoreValue::Array(_) => Err(StatisticalTestsEvalError::Coercion(
                CoercionError::UnsupportedValueKind("array"),
            )),
            CoreValue::Reference(_) => Err(StatisticalTestsEvalError::Coercion(
                CoercionError::UnsupportedValueKind("reference_like"),
            )),
            CoreValue::Text(_) | CoreValue::Logical(_) | CoreValue::Missing | CoreValue::Empty => {
                Err(StatisticalTestsEvalError::Domain(WorksheetErrorCode::Value))
            }
        },
        AggregateArgOrigin::ArrayLike(_) => match item.0.core() {
            CoreValue::Number(n) => Ok(Some(*n)),
            CoreValue::Error(code) => Err(StatisticalTestsEvalError::Domain(*code)),
            CoreValue::Text(_) | CoreValue::Logical(_) | CoreValue::Missing | CoreValue::Empty => {
                Ok(None)
            }
            CoreValue::Array(_) => Err(StatisticalTestsEvalError::Coercion(
                CoercionError::UnsupportedValueKind("array"),
            )),
            CoreValue::Reference(_) => Err(StatisticalTestsEvalError::Coercion(
                CoercionError::UnsupportedValueKind("reference_like"),
            )),
        },
    }
}
fn collect_numeric_sample_arg(
    arg: &CalcValue,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<Vec<f64>, StatisticalTestsEvalError> {
    let expanded =
        expand_aggregate_arg(arg, resolver).map_err(StatisticalTestsEvalError::Coercion)?;
    let mut out = Vec::new();
    for item in &expanded {
        if let Some(n) = aggregate_item_number(item)? {
            out.push(n);
        }
    }
    Ok(out)
}

fn collect_paired_numeric_samples(
    x_arg: &CalcValue,
    y_arg: &CalcValue,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<Vec<(f64, f64)>, StatisticalTestsEvalError> {
    let xs = expand_aggregate_arg(x_arg, resolver).map_err(StatisticalTestsEvalError::Coercion)?;
    let ys = expand_aggregate_arg(y_arg, resolver).map_err(StatisticalTestsEvalError::Coercion)?;
    if xs.len() != ys.len() {
        return Err(StatisticalTestsEvalError::Domain(WorksheetErrorCode::NA));
    }
    let mut pairs = Vec::new();
    for (x_item, y_item) in xs.iter().zip(ys.iter()) {
        let x = aggregate_item_number(x_item)?;
        let y = aggregate_item_number(y_item)?;
        if let (Some(x), Some(y)) = (x, y) {
            pairs.push((x, y));
        }
    }
    Ok(pairs)
}

fn sample_mean(values: &[f64]) -> Result<f64, WorksheetErrorCode> {
    if values.is_empty() {
        return Err(WorksheetErrorCode::Div0);
    }
    Ok(values.iter().sum::<f64>() / values.len() as f64)
}

fn sample_variance(values: &[f64]) -> Result<f64, WorksheetErrorCode> {
    if values.len() < 2 {
        return Err(WorksheetErrorCode::Div0);
    }
    let mean = sample_mean(values)?;
    Ok(values
        .iter()
        .map(|v| {
            let d = *v - mean;
            d * d
        })
        .sum::<f64>()
        / (values.len() - 1) as f64)
}

fn validate_positive_df(df: f64) -> Result<f64, WorksheetErrorCode> {
    if !df.is_finite() || df <= 0.0 {
        Err(WorksheetErrorCode::Num)
    } else {
        Ok(df)
    }
}

fn t_probability(abs_t: f64, df: f64, tails: i32) -> Result<f64, WorksheetErrorCode> {
    if !abs_t.is_finite() || abs_t < 0.0 {
        return Err(WorksheetErrorCode::Num);
    }
    let z = validate_positive_df(df)? / (validate_positive_df(df)? + abs_t * abs_t);
    let two_tail = regularized_beta(z, df / 2.0, 0.5);
    match tails {
        1 => Ok(two_tail / 2.0),
        2 => Ok(two_tail),
        _ => Err(WorksheetErrorCode::Num),
    }
}

fn chisq_test_kernel(actual: &CalcArray, expected: &CalcArray) -> Result<f64, WorksheetErrorCode> {
    let shape = actual.shape();
    let df = if shape.rows > 1 && shape.cols > 1 {
        (shape.rows - 1) * (shape.cols - 1)
    } else if shape.rows == 1 {
        shape.cols - 1
    } else {
        shape.rows - 1
    };
    if df == 0 {
        return Err(WorksheetErrorCode::NA);
    }
    let mut statistic = 0.0;
    for (a_cell, e_cell) in actual.iter_row_major().zip(expected.iter_row_major()) {
        let actual = scalar_number_from_cell(a_cell).map_err(|_| WorksheetErrorCode::Value)?;
        let expected = scalar_number_from_cell(e_cell).map_err(|_| WorksheetErrorCode::Value)?;
        if !actual.is_finite() || !expected.is_finite() || expected <= 0.0 {
            return Err(WorksheetErrorCode::Num);
        }
        let delta = actual - expected;
        statistic += delta * delta / expected;
    }
    chisq_dist_rt_kernel(statistic, df as f64)
}

fn f_test_kernel(array1: &[f64], array2: &[f64]) -> Result<f64, WorksheetErrorCode> {
    if array1.len() < 2 || array2.len() < 2 {
        return Err(WorksheetErrorCode::Div0);
    }
    let var1 = sample_variance(array1)?;
    let var2 = sample_variance(array2)?;
    if var1 == 0.0 || var2 == 0.0 {
        return Err(WorksheetErrorCode::Div0);
    }
    let (ratio, df1, df2) = if var1 >= var2 {
        (
            var1 / var2,
            (array1.len() - 1) as f64,
            (array2.len() - 1) as f64,
        )
    } else {
        (
            var2 / var1,
            (array2.len() - 1) as f64,
            (array1.len() - 1) as f64,
        )
    };
    Ok((2.0 * f_dist_rt_kernel(ratio, df1, df2)?).min(1.0))
}

fn t_test_paired_kernel(pairs: &[(f64, f64)], tails: i32) -> Result<f64, WorksheetErrorCode> {
    if pairs.len() < 2 {
        return Err(WorksheetErrorCode::Div0);
    }
    let diffs: Vec<f64> = pairs.iter().map(|(x, y)| x - y).collect();
    let mean = sample_mean(&diffs)?;
    let se = (sample_variance(&diffs)? / diffs.len() as f64).sqrt();
    if se == 0.0 {
        return Err(WorksheetErrorCode::Div0);
    }
    t_probability((mean / se).abs(), (diffs.len() - 1) as f64, tails)
}

fn t_test_equal_variance_kernel(
    array1: &[f64],
    array2: &[f64],
    tails: i32,
) -> Result<f64, WorksheetErrorCode> {
    if array1.len() < 2 || array2.len() < 2 {
        return Err(WorksheetErrorCode::Div0);
    }
    let mean1 = sample_mean(array1)?;
    let mean2 = sample_mean(array2)?;
    let var1 = sample_variance(array1)?;
    let var2 = sample_variance(array2)?;
    let df = (array1.len() + array2.len() - 2) as f64;
    let pooled = (((array1.len() - 1) as f64) * var1 + ((array2.len() - 1) as f64) * var2) / df;
    let se = (pooled * (1.0 / array1.len() as f64 + 1.0 / array2.len() as f64)).sqrt();
    if se == 0.0 {
        return Err(WorksheetErrorCode::Div0);
    }
    t_probability(((mean1 - mean2) / se).abs(), df, tails)
}

fn t_test_unequal_variance_kernel(
    array1: &[f64],
    array2: &[f64],
    tails: i32,
) -> Result<f64, WorksheetErrorCode> {
    if array1.len() < 2 || array2.len() < 2 {
        return Err(WorksheetErrorCode::Div0);
    }
    let mean1 = sample_mean(array1)?;
    let mean2 = sample_mean(array2)?;
    let var1 = sample_variance(array1)?;
    let var2 = sample_variance(array2)?;
    let term1 = var1 / array1.len() as f64;
    let term2 = var2 / array2.len() as f64;
    let se = (term1 + term2).sqrt();
    if se == 0.0 {
        return Err(WorksheetErrorCode::Div0);
    }
    let denom =
        term1 * term1 / (array1.len() - 1) as f64 + term2 * term2 / (array2.len() - 1) as f64;
    if denom == 0.0 {
        return Err(WorksheetErrorCode::Div0);
    }
    let df = (term1 + term2) * (term1 + term2) / denom;
    t_probability(((mean1 - mean2) / se).abs(), df, tails)
}

fn eval_chisq_test_prepared(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, StatisticalTestsEvalError> {
    if !CHISQ_TEST_META.arity.accepts(args.len()) {
        return Err(arity_error(&CHISQ_TEST_META, args.len()));
    }
    let (actual, expected) = numeric_matrices_from_args(&args[0], &args[1], resolver)?;
    Ok(match chisq_test_kernel(&actual, &expected) {
        Ok(v) => CalcValue::number(v),
        Err(code) => CalcValue::error(code),
    })
}

fn eval_f_test_prepared(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, StatisticalTestsEvalError> {
    if !F_TEST_META.arity.accepts(args.len()) {
        return Err(arity_error(&F_TEST_META, args.len()));
    }
    let array1 = collect_numeric_sample_arg(&args[0], resolver)?;
    let array2 = collect_numeric_sample_arg(&args[1], resolver)?;
    Ok(match f_test_kernel(&array1, &array2) {
        Ok(v) => CalcValue::number(v),
        Err(code) => CalcValue::error(code),
    })
}

fn eval_t_test_prepared(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, StatisticalTestsEvalError> {
    if !T_TEST_META.arity.accepts(args.len()) {
        return Err(arity_error(&T_TEST_META, args.len()));
    }
    let tails = truncated_flag_arg(&args[2], resolver)?;
    let test_type = truncated_flag_arg(&args[3], resolver)?;
    let result = match test_type {
        1 => t_test_paired_kernel(
            &collect_paired_numeric_samples(&args[0], &args[1], resolver)?,
            tails,
        ),
        2 => t_test_equal_variance_kernel(
            &collect_numeric_sample_arg(&args[0], resolver)?,
            &collect_numeric_sample_arg(&args[1], resolver)?,
            tails,
        ),
        3 => t_test_unequal_variance_kernel(
            &collect_numeric_sample_arg(&args[0], resolver)?,
            &collect_numeric_sample_arg(&args[1], resolver)?,
            tails,
        ),
        _ => Err(WorksheetErrorCode::Num),
    };
    Ok(match result {
        Ok(v) => CalcValue::number(v),
        Err(code) => CalcValue::error(code),
    })
}
pub fn eval_chisq_test_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, StatisticalTestsEvalError> {
    surface_domain(eval_chisq_test_prepared(args, resolver))
}

pub fn eval_chitest_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, StatisticalTestsEvalError> {
    if !CHITEST_META.arity.accepts(args.len()) {
        return Err(arity_error(&CHITEST_META, args.len()));
    }
    surface_domain(eval_chisq_test_prepared(args, resolver))
}

pub fn eval_f_test_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, StatisticalTestsEvalError> {
    surface_domain(eval_f_test_prepared(args, resolver))
}

pub fn eval_ftest_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, StatisticalTestsEvalError> {
    if !FTEST_META.arity.accepts(args.len()) {
        return Err(arity_error(&FTEST_META, args.len()));
    }
    surface_domain(eval_f_test_prepared(args, resolver))
}

pub fn eval_t_test_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, StatisticalTestsEvalError> {
    surface_domain(eval_t_test_prepared(args, resolver))
}

pub fn eval_ttest_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, StatisticalTestsEvalError> {
    if !TTEST_META.arity.accepts(args.len()) {
        return Err(arity_error(&TTEST_META, args.len()));
    }
    surface_domain(eval_t_test_prepared(args, resolver))
}

fn surface_domain(
    result: Result<CalcValue, StatisticalTestsEvalError>,
) -> Result<CalcValue, StatisticalTestsEvalError> {
    match result {
        Err(StatisticalTestsEvalError::Domain(code)) => Ok(CalcValue::error(code)),
        other => other,
    }
}

pub fn map_statistical_tests_error_to_ws(error: &StatisticalTestsEvalError) -> WorksheetErrorCode {
    match error {
        StatisticalTestsEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        StatisticalTestsEvalError::Coercion(coercion) => match coercion {
            CoercionError::WorksheetError(code) => *code,
            CoercionError::RefResolution(_) => WorksheetErrorCode::Ref,
            CoercionError::MissingArg
            | CoercionError::EmptyCell
            | CoercionError::NonNumericText(_)
            | CoercionError::UnsupportedValueKind(_) => WorksheetErrorCode::Value,
        },
        StatisticalTestsEvalError::Domain(code) => *code,
    }
}

struct NoResolver;

impl ReferenceSystemProvider for NoResolver {
    fn capabilities(&self) -> crate::resolver::ReferenceSystemCapabilities {
        crate::resolver::ReferenceSystemCapabilities::permissive_local()
    }
    fn dereference(
        &self,
        request: &crate::resolver::ReferenceDereferenceRequest,
    ) -> Result<CalcValue, crate::resolver::ReferenceResolutionError> {
        Err(
            crate::resolver::ReferenceResolutionError::UnresolvedReference {
                target: request.reference.target().to_string(),
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::ReferenceSystemCapabilities;
    use crate::value::{CoreValue, ExcelText, ReferenceKind, ReferenceLike};
    use std::collections::HashMap;

    #[derive(Default)]
    struct MockResolver {
        values: HashMap<String, CalcValue>,
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
            self.values.get(reference.target()).cloned().ok_or_else(|| {
                crate::resolver::ReferenceResolutionError::UnresolvedReference {
                    target: reference.target().to_string(),
                }
            })
        }
    }

    fn num(n: f64) -> CalcValue {
        CalcValue::number(n)
    }
    fn txt(s: &str) -> CalcValue {
        CalcValue::text(ExcelText::from_interop_assignment(s))
    }
    fn col(values: &[f64]) -> CalcValue {
        CalcValue::array(
            CalcArray::from_rows(values.iter().map(|n| vec![CalcValue::number(*n)]).collect())
                .unwrap(),
        )
    }
    fn array(rows: Vec<Vec<CalcValue>>) -> CalcValue {
        CalcValue::array(CalcArray::from_rows(rows).unwrap())
    }
    fn ref_arg(target: &str) -> CalcValue {
        CalcValue::reference(ReferenceLike::new(ReferenceKind::Area, target.to_string()))
    }

    fn assert_number_close(value: CalcValue, expected: f64, tolerance: f64) {
        match value.core() {
            CoreValue::Number(v) => assert!((*v - expected).abs() < tolerance),
            other => panic!("unexpected result: {:?}", other),
        }
    }

    #[test]
    fn metadata_shapes_are_frozen() {
        assert_eq!(CHISQ_TEST_META.arity, Arity::exact(2));
        assert_eq!(F_TEST_META.arity, Arity::exact(2));
        assert_eq!(T_TEST_META.arity, Arity::exact(4));
        assert_eq!(TTEST_META.function_id, "FUNC.TTEST");
        assert_eq!(
            CHISQ_TEST_META.arg_preparation_profile,
            ArgPreparationProfile::RefsVisibleInAdapter
        );
    }

    #[test]
    fn chisq_test_matches_official_example() {
        let resolver = MockResolver {
            values: HashMap::from([
                (
                    "A2:B4".to_string(),
                    array(vec![
                        vec![CalcValue::number(58.0), CalcValue::number(35.0)],
                        vec![CalcValue::number(11.0), CalcValue::number(25.0)],
                        vec![CalcValue::number(10.0), CalcValue::number(23.0)],
                    ]),
                ),
                (
                    "A6:B8".to_string(),
                    array(vec![
                        vec![CalcValue::number(45.35), CalcValue::number(47.65)],
                        vec![CalcValue::number(17.56), CalcValue::number(18.44)],
                        vec![CalcValue::number(16.09), CalcValue::number(16.91)],
                    ]),
                ),
            ]),
        };
        assert_number_close(
            eval_chisq_test_surface(&[ref_arg("A2:B4"), ref_arg("A6:B8")], &resolver).unwrap(),
            0.000_308_2,
            1e-7,
        );
    }

    #[test]
    fn chisq_test_rejects_single_cell_shape() {
        assert_eq!(
            eval_chisq_test_surface(&[num(1.0), num(1.0)], &MockResolver::default()).unwrap(),
            CalcValue::error(WorksheetErrorCode::NA)
        );
    }

    #[test]
    fn chisq_test_reshapes_expected_by_actual_layout_when_cardinality_matches() {
        let resolver = MockResolver {
            values: HashMap::from([
                (
                    "A2:B4".to_string(),
                    array(vec![
                        vec![CalcValue::number(58.0), CalcValue::number(35.0)],
                        vec![CalcValue::number(11.0), CalcValue::number(25.0)],
                        vec![CalcValue::number(10.0), CalcValue::number(23.0)],
                    ]),
                ),
                (
                    "C1:H1".to_string(),
                    array(vec![vec![
                        CalcValue::number(45.35),
                        CalcValue::number(47.65),
                        CalcValue::number(17.56),
                        CalcValue::number(18.44),
                        CalcValue::number(16.09),
                        CalcValue::number(16.91),
                    ]]),
                ),
            ]),
        };
        assert_number_close(
            eval_chisq_test_surface(&[ref_arg("A2:B4"), ref_arg("C1:H1")], &resolver).unwrap(),
            0.000_308_192_017_008_309,
            1e-15,
        );
    }

    #[test]
    fn chitest_delegates_to_modern_function() {
        let resolver = MockResolver {
            values: HashMap::from([
                ("A1:A2".to_string(), col(&[10.0, 20.0])),
                ("B1:B2".to_string(), col(&[12.0, 18.0])),
            ]),
        };
        let args = [ref_arg("A1:A2"), ref_arg("B1:B2")];
        assert_eq!(
            eval_chitest_surface(&args, &resolver),
            eval_chisq_test_surface(&args, &resolver)
        );
    }

    #[test]
    fn f_test_matches_official_example() {
        let resolver = MockResolver {
            values: HashMap::from([
                ("A2:A6".to_string(), col(&[6.0, 7.0, 9.0, 15.0, 21.0])),
                ("B2:B6".to_string(), col(&[20.0, 28.0, 31.0, 38.0, 40.0])),
            ]),
        };
        assert_number_close(
            eval_f_test_surface(&[ref_arg("A2:A6"), ref_arg("B2:B6")], &resolver).unwrap(),
            0.648_317_85,
            1e-8,
        );
    }
    #[test]
    fn f_test_ignores_text_logical_and_empty_cells_in_array_inputs() {
        let resolver = MockResolver {
            values: HashMap::from([
                (
                    "A1:A7".to_string(),
                    array(vec![
                        vec![CalcValue::number(6.0)],
                        vec![txt("skip")],
                        vec![CalcValue::logical(true)],
                        vec![CalcValue::number(7.0)],
                        vec![CalcValue::empty()],
                        vec![CalcValue::number(9.0)],
                        vec![CalcValue::number(15.0)],
                    ]),
                ),
                (
                    "B1:B6".to_string(),
                    array(vec![
                        vec![CalcValue::number(20.0)],
                        vec![CalcValue::number(28.0)],
                        vec![CalcValue::number(31.0)],
                        vec![CalcValue::number(38.0)],
                        vec![CalcValue::number(40.0)],
                        vec![CalcValue::empty()],
                    ]),
                ),
            ]),
        };
        let clean = f_test_kernel(&[6.0, 7.0, 9.0, 15.0], &[20.0, 28.0, 31.0, 38.0, 40.0]).unwrap();
        assert_number_close(
            eval_f_test_surface(&[ref_arg("A1:A7"), ref_arg("B1:B6")], &resolver).unwrap(),
            clean,
            1e-12,
        );
    }

    #[test]
    fn ftest_delegates_to_modern_function() {
        let resolver = MockResolver {
            values: HashMap::from([
                ("A1:A5".to_string(), col(&[6.0, 7.0, 9.0, 15.0, 21.0])),
                ("B1:B5".to_string(), col(&[20.0, 28.0, 31.0, 38.0, 40.0])),
            ]),
        };
        let args = [ref_arg("A1:A5"), ref_arg("B1:B5")];
        assert_eq!(
            eval_ftest_surface(&args, &resolver),
            eval_f_test_surface(&args, &resolver)
        );
    }

    #[test]
    fn t_test_paired_matches_official_example() {
        let resolver = MockResolver {
            values: HashMap::from([
                (
                    "A2:A10".to_string(),
                    col(&[3.0, 4.0, 5.0, 8.0, 9.0, 1.0, 2.0, 4.0, 5.0]),
                ),
                (
                    "B2:B10".to_string(),
                    col(&[6.0, 19.0, 3.0, 2.0, 14.0, 4.0, 5.0, 17.0, 1.0]),
                ),
            ]),
        };
        assert_number_close(
            eval_t_test_surface(
                &[ref_arg("A2:A10"), ref_arg("B2:B10"), num(2.0), num(1.0)],
                &resolver,
            )
            .unwrap(),
            0.196_016,
            1e-6,
        );
    }

    #[test]
    fn t_test_type2_and_type3_support_both_tail_modes() {
        let resolver = MockResolver {
            values: HashMap::from([
                ("A1:A6".to_string(), col(&[3.0, 4.0, 5.0, 8.0, 9.0, 1.0])),
                ("B1:B6".to_string(), col(&[6.0, 19.0, 3.0, 2.0, 14.0, 4.0])),
            ]),
        };
        for test_type in [2.0, 3.0] {
            let one = eval_t_test_surface(
                &[ref_arg("A1:A6"), ref_arg("B1:B6"), num(1.0), num(test_type)],
                &resolver,
            )
            .unwrap();
            let two = eval_t_test_surface(
                &[ref_arg("A1:A6"), ref_arg("B1:B6"), num(2.0), num(test_type)],
                &resolver,
            )
            .unwrap();
            let (one, two) = match (one.core(), two.core()) {
                (CoreValue::Number(a), CoreValue::Number(b)) => (*a, *b),
                _ => panic!("unexpected"),
            };
            assert!(one > 0.0 && one < 1.0);
            assert!(two > 0.0 && two <= 1.0);
            assert!((two - 2.0 * one).abs() < 1e-12);
        }
    }

    #[test]
    fn t_test_paired_mismatched_expanded_cardinality_returns_na() {
        let resolver = MockResolver {
            values: HashMap::from([
                ("A1:A3".to_string(), col(&[1.0, 2.0, 3.0])),
                ("B1:B2".to_string(), col(&[1.0, 2.0])),
            ]),
        };
        assert_eq!(
            eval_t_test_surface(
                &[ref_arg("A1:A3"), ref_arg("B1:B2"), num(2.0), num(1.0)],
                &resolver
            )
            .unwrap(),
            CalcValue::error(WorksheetErrorCode::NA)
        );
    }

    #[test]
    fn t_test_invalid_tails_and_type_return_num() {
        let resolver = MockResolver {
            values: HashMap::from([
                ("A1:A3".to_string(), col(&[1.0, 2.0, 3.0])),
                ("B1:B3".to_string(), col(&[2.0, 3.0, 4.0])),
            ]),
        };
        assert_eq!(
            eval_t_test_surface(
                &[ref_arg("A1:A3"), ref_arg("B1:B3"), num(3.0), num(2.0)],
                &resolver
            )
            .unwrap(),
            CalcValue::error(WorksheetErrorCode::Num)
        );
        assert_eq!(
            eval_t_test_surface(
                &[ref_arg("A1:A3"), ref_arg("B1:B3"), num(2.0), num(9.0)],
                &resolver
            )
            .unwrap(),
            CalcValue::error(WorksheetErrorCode::Num)
        );
    }

    #[test]
    fn ttest_delegates_to_modern_function() {
        let resolver = MockResolver {
            values: HashMap::from([
                ("A1:A4".to_string(), col(&[1.0, 3.0, 5.0, 7.0])),
                ("B1:B4".to_string(), col(&[2.0, 4.0, 6.0, 8.0])),
            ]),
        };
        let args = [ref_arg("A1:A4"), ref_arg("B1:B4"), num(2.0), num(2.0)];
        assert_eq!(
            eval_ttest_surface(&args, &resolver),
            eval_t_test_surface(&args, &resolver)
        );
    }

    #[test]
    fn explicit_error_mapping_is_stable() {
        assert_eq!(
            map_statistical_tests_error_to_ws(&StatisticalTestsEvalError::ArityMismatch {
                expected_min: 2,
                expected_max: 2,
                actual: 1
            }),
            WorksheetErrorCode::Value
        );
        assert_eq!(
            map_statistical_tests_error_to_ws(&StatisticalTestsEvalError::Coercion(
                CoercionError::RefResolution(
                    crate::resolver::ReferenceResolutionError::EvalTimeDerefNotAllowed
                )
            )),
            WorksheetErrorCode::Ref
        );
        assert_eq!(
            map_statistical_tests_error_to_ws(&StatisticalTestsEvalError::Domain(
                WorksheetErrorCode::Div0
            )),
            WorksheetErrorCode::Div0
        );
    }

    #[test]
    fn aggregate_numeric_collection_rejects_direct_text_scalar_but_ignores_array_text() {
        let resolver = MockResolver::default();
        assert_eq!(
            collect_numeric_sample_arg(
                &(CalcValue::text(ExcelText::from_interop_assignment("3"))),
                &resolver
            ),
            Err(StatisticalTestsEvalError::Domain(WorksheetErrorCode::Value))
        );
        assert_eq!(
            collect_numeric_sample_arg(
                &(array(vec![
                    vec![CalcValue::number(1.0)],
                    vec![txt("skip")],
                    vec![CalcValue::number(2.0)]
                ])),
                &resolver
            )
            .unwrap(),
            vec![1.0, 2.0]
        );
    }
}

use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{AggregatePreparedValue, expand_aggregate_arg};
use crate::functions::aggregate_common::average_argument_value;
use crate::functions::paired_stats_common::collect_paired_values;
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

const MOMENT_STATS_BASE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.MOMENT_STATS_BASE",
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

pub const KURT_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.KURT",
    arity: Arity { min: 1, max: 255 },
    coercion_lift_profile: CoercionLiftProfile::AggregateDirectAndRangeDualPolicy,
    kernel_signature_class: KernelSignatureClass::NumsToNum,
    ..MOMENT_STATS_BASE_META
};

pub const SKEW_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.SKEW",
    arity: Arity { min: 1, max: 255 },
    coercion_lift_profile: CoercionLiftProfile::AggregateDirectAndRangeDualPolicy,
    kernel_signature_class: KernelSignatureClass::NumsToNum,
    ..MOMENT_STATS_BASE_META
};

pub const SKEW_P_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.SKEW.P",
    arity: Arity { min: 1, max: 255 },
    coercion_lift_profile: CoercionLiftProfile::AggregateDirectAndRangeDualPolicy,
    kernel_signature_class: KernelSignatureClass::NumsToNum,
    ..MOMENT_STATS_BASE_META
};

pub const STEYX_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.STEYX",
    arity: Arity::exact(2),
    ..MOMENT_STATS_BASE_META
};

pub const TRIMMEAN_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.TRIMMEAN",
    arity: Arity::exact(2),
    coercion_lift_profile: CoercionLiftProfile::AggregateDirectAndRangeDualPolicy,
    kernel_signature_class: KernelSignatureClass::Custom,
    ..MOMENT_STATS_BASE_META
};

#[derive(Debug, Clone, PartialEq)]
pub enum MomentStatsEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
}

fn collect_moment_values(args: &[AggregatePreparedValue]) -> Result<Vec<f64>, CoercionError> {
    let mut values = Vec::new();
    for arg in args {
        if let Some(value) = average_argument_value(arg)? {
            values.push(value);
        }
    }
    Ok(values)
}

fn mean(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len() as f64
}

fn sample_standardized_moment_sum(
    values: &[f64],
    power: i32,
) -> Result<(usize, f64), WorksheetErrorCode> {
    let n = values.len();
    let avg = mean(values);
    let mut sumsq = 0.0;
    let mut sump = 0.0;
    for value in values {
        let delta = *value - avg;
        sumsq += delta * delta;
        sump += delta.powi(power);
    }
    if sumsq == 0.0 {
        return Err(WorksheetErrorCode::Div0);
    }
    let sample_std = (sumsq / (n as f64 - 1.0)).sqrt();
    Ok((n, sump / sample_std.powi(power)))
}

fn population_standardized_moment_sum(
    values: &[f64],
    power: i32,
) -> Result<(usize, f64), WorksheetErrorCode> {
    let n = values.len();
    let avg = mean(values);
    let mut sumsq = 0.0;
    let mut sump = 0.0;
    for value in values {
        let delta = *value - avg;
        sumsq += delta * delta;
        sump += delta.powi(power);
    }
    if sumsq == 0.0 {
        return Err(WorksheetErrorCode::Div0);
    }
    let pop_std = (sumsq / n as f64).sqrt();
    Ok((n, sump / pop_std.powi(power)))
}

fn kurt_kernel(values: &[f64]) -> Result<f64, WorksheetErrorCode> {
    if values.len() < 4 {
        return Err(WorksheetErrorCode::Div0);
    }
    let (n, standardized_sum) = sample_standardized_moment_sum(values, 4)?;
    let n = n as f64;
    Ok(
        (n * (n + 1.0) * standardized_sum) / ((n - 1.0) * (n - 2.0) * (n - 3.0))
            - (3.0 * (n - 1.0).powi(2)) / ((n - 2.0) * (n - 3.0)),
    )
}

fn skew_kernel(values: &[f64]) -> Result<f64, WorksheetErrorCode> {
    if values.len() < 3 {
        return Err(WorksheetErrorCode::Div0);
    }

    let n = values.len();
    let avg = mean(values);
    let mut sumsq = 0.0;
    let mut deltas = Vec::with_capacity(n);
    for value in values {
        let delta = *value - avg;
        sumsq += delta * delta;
        deltas.push(delta);
    }
    if sumsq == 0.0 {
        return Err(WorksheetErrorCode::Div0);
    }

    let sample_std = (sumsq / (n as f64 - 1.0)).sqrt();
    let standardized_sum = deltas
        .iter()
        .map(|delta| {
            let z = *delta / sample_std;
            z * z * z
        })
        .sum::<f64>();
    let n = n as f64;
    Ok(n * standardized_sum / ((n - 1.0) * (n - 2.0)))
}

fn skew_p_kernel(values: &[f64]) -> Result<f64, WorksheetErrorCode> {
    if values.is_empty() {
        return Err(WorksheetErrorCode::Div0);
    }
    let (n, standardized_sum) = population_standardized_moment_sum(values, 3)?;
    Ok(standardized_sum / n as f64)
}

fn steyx_kernel(pairs: &[(f64, f64)]) -> Result<f64, WorksheetErrorCode> {
    if pairs.len() < 3 {
        return Err(WorksheetErrorCode::Div0);
    }
    let n = pairs.len() as f64;
    let mean_x = pairs.iter().map(|(x, _)| *x).sum::<f64>() / n;
    let mean_y = pairs.iter().map(|(_, y)| *y).sum::<f64>() / n;
    let mut sum_x2 = 0.0;
    let mut sum_y2 = 0.0;
    let mut sum_xy = 0.0;
    for (x, y) in pairs {
        let dx = *x - mean_x;
        let dy = *y - mean_y;
        sum_x2 += dx * dx;
        sum_y2 += dy * dy;
        sum_xy += dx * dy;
    }
    if sum_x2 == 0.0 {
        return Err(WorksheetErrorCode::Div0);
    }
    let sse = sum_y2 - (sum_xy * sum_xy) / sum_x2;
    let residual = if sse < 0.0 && sse.abs() < 1.0e-12 {
        0.0
    } else {
        sse
    };
    Ok((residual / (n - 2.0)).sqrt())
}

fn trimmean_kernel(values: &mut [f64], percent: f64) -> Result<f64, WorksheetErrorCode> {
    if !percent.is_finite() {
        return Err(WorksheetErrorCode::Value);
    }
    if !(0.0..=1.0).contains(&percent) {
        return Err(WorksheetErrorCode::Num);
    }
    if values.is_empty() {
        return Err(WorksheetErrorCode::Num);
    }
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let to_trim = (percent * values.len() as f64).floor() as usize;
    let each_side = to_trim / 2;
    if each_side * 2 >= values.len() {
        return Err(WorksheetErrorCode::Num);
    }
    let kept = &values[each_side..values.len() - each_side];
    Ok(kept.iter().sum::<f64>() / kept.len() as f64)
}

fn guard_arity(meta: &FunctionMeta, args: &[CallArgValue]) -> Result<(), MomentStatsEvalError> {
    if meta.arity.accepts(args.len()) {
        Ok(())
    } else {
        Err(MomentStatsEvalError::ArityMismatch {
            expected_min: meta.arity.min,
            expected_max: meta.arity.max,
            actual: args.len(),
        })
    }
}

pub fn eval_kurt_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, MomentStatsEvalError> {
    guard_arity(&KURT_META, args)?;
    let mut prepared = Vec::new();
    for arg in args {
        prepared
            .extend(expand_aggregate_arg(arg, resolver).map_err(MomentStatsEvalError::Coercion)?);
    }
    let values = collect_moment_values(&prepared).map_err(MomentStatsEvalError::Coercion)?;
    match kurt_kernel(&values) {
        Ok(value) => Ok(EvalValue::Number(value)),
        Err(code) => Ok(EvalValue::Error(code)),
    }
}

pub fn eval_skew_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, MomentStatsEvalError> {
    guard_arity(&SKEW_META, args)?;
    let mut prepared = Vec::new();
    for arg in args {
        prepared
            .extend(expand_aggregate_arg(arg, resolver).map_err(MomentStatsEvalError::Coercion)?);
    }
    let values = collect_moment_values(&prepared).map_err(MomentStatsEvalError::Coercion)?;
    match skew_kernel(&values) {
        Ok(value) => Ok(EvalValue::Number(value)),
        Err(code) => Ok(EvalValue::Error(code)),
    }
}

pub fn eval_skew_p_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, MomentStatsEvalError> {
    guard_arity(&SKEW_P_META, args)?;
    let mut prepared = Vec::new();
    for arg in args {
        prepared
            .extend(expand_aggregate_arg(arg, resolver).map_err(MomentStatsEvalError::Coercion)?);
    }
    let values = collect_moment_values(&prepared).map_err(MomentStatsEvalError::Coercion)?;
    match skew_p_kernel(&values) {
        Ok(value) => Ok(EvalValue::Number(value)),
        Err(code) => Ok(EvalValue::Error(code)),
    }
}

pub fn eval_steyx_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, MomentStatsEvalError> {
    guard_arity(&STEYX_META, args)?;
    let ys = expand_aggregate_arg(&args[0], resolver).map_err(MomentStatsEvalError::Coercion)?;
    let xs = expand_aggregate_arg(&args[1], resolver).map_err(MomentStatsEvalError::Coercion)?;
    let pairs = collect_paired_values(&xs, &ys).map_err(MomentStatsEvalError::Coercion)?;
    match steyx_kernel(&pairs) {
        Ok(value) => Ok(EvalValue::Number(value)),
        Err(code) => Ok(EvalValue::Error(code)),
    }
}

pub fn eval_trimmean_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, MomentStatsEvalError> {
    guard_arity(&TRIMMEAN_META, args)?;
    let prepared =
        expand_aggregate_arg(&args[0], resolver).map_err(MomentStatsEvalError::Coercion)?;
    let mut values = collect_moment_values(&prepared).map_err(MomentStatsEvalError::Coercion)?;
    let percent_items =
        expand_aggregate_arg(&args[1], resolver).map_err(MomentStatsEvalError::Coercion)?;
    let percent = match percent_items.as_slice() {
        [item] => average_argument_value(item).map_err(MomentStatsEvalError::Coercion)?,
        _ => None,
    };
    let Some(percent) = percent else {
        return Ok(EvalValue::Error(WorksheetErrorCode::Value));
    };
    match trimmean_kernel(&mut values, percent) {
        Ok(value) => Ok(EvalValue::Number(value)),
        Err(code) => Ok(EvalValue::Error(code)),
    }
}

pub fn map_moment_stats_error_to_ws(error: &MomentStatsEvalError) -> WorksheetErrorCode {
    match error {
        MomentStatsEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        MomentStatsEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        MomentStatsEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::{RefResolutionError, ResolverCapabilities};
    use crate::value::{ArrayCellValue, EvalArray, ExcelText, ReferenceKind, ReferenceLike};
    use std::collections::HashMap;

    struct MockResolver {
        cells: HashMap<String, EvalValue>,
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

    fn ref_arg(target: &str) -> CallArgValue {
        CallArgValue::Reference(ReferenceLike {
            kind: ReferenceKind::Area,
            target: target.to_string(),
        })
    }

    fn num(n: f64) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Number(n))
    }

    fn text(s: &str) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
            s.encode_utf16().collect(),
        )))
    }

    fn array_row(cells: Vec<ArrayCellValue>) -> EvalValue {
        EvalValue::Array(EvalArray::from_rows(vec![cells]).unwrap())
    }

    fn number_row(values: &[f64]) -> EvalValue {
        EvalValue::Array(
            EvalArray::from_rows(vec![
                values.iter().copied().map(ArrayCellValue::Number).collect(),
            ])
            .unwrap(),
        )
    }

    fn assert_close(value: EvalValue, expected: f64) {
        match value {
            EvalValue::Number(n) => assert!((n - expected).abs() < 1.0e-9, "{n} vs {expected}"),
            other => panic!("expected number, got {other:?}"),
        }
    }

    #[test]
    fn metadata_matches_expected_shapes() {
        assert_eq!(KURT_META.arity.min, 1);
        assert_eq!(SKEW_META.arity.max, 255);
        assert_eq!(SKEW_P_META.function_id, "FUNC.SKEW.P");
        assert_eq!(STEYX_META.arity, Arity::exact(2));
        assert_eq!(
            TRIMMEAN_META.surface_fec_dependency_profile,
            FecDependencyProfile::RefOnly
        );
    }

    #[test]
    fn kurt_skew_and_skew_p_match_seed_values() {
        let resolver = MockResolver {
            cells: HashMap::from([(
                "A1:A8".to_string(),
                number_row(&[3.0, 4.0, 5.0, 2.0, 3.0, 4.0, 6.0, 9.0]),
            )]),
        };
        assert_close(
            eval_kurt_surface(&[ref_arg("A1:A8")], &resolver).unwrap(),
            1.85051903114187,
        );
        assert_close(
            eval_skew_surface(&[ref_arg("A1:A8")], &resolver).unwrap(),
            1.28115559478545,
        );
        assert_close(
            eval_skew_p_surface(&[ref_arg("A1:A8")], &resolver).unwrap(),
            1.02720970603623,
        );
    }

    #[test]
    fn trimmean_uses_even_trim_rounding_rule() {
        let resolver = MockResolver {
            cells: HashMap::from([(
                "A1:A10".to_string(),
                number_row(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 100.0]),
            )]),
        };
        assert_close(
            eval_trimmean_surface(&[ref_arg("A1:A10"), num(0.2)], &resolver).unwrap(),
            5.5,
        );
        assert_close(
            eval_trimmean_surface(&[ref_arg("A1:A10"), num(0.19)], &resolver).unwrap(),
            14.5,
        );
    }

    #[test]
    fn steyx_matches_linear_regression_seed_and_zero_residual_case() {
        let resolver = MockResolver {
            cells: HashMap::from([
                ("Y1:Y5".to_string(), number_row(&[2.0, 4.1, 5.9, 8.2, 9.8])),
                ("X1:X5".to_string(), number_row(&[1.0, 2.0, 3.0, 4.0, 5.0])),
                ("YZ1:YZ4".to_string(), number_row(&[2.0, 4.0, 6.0, 8.0])),
                ("XZ1:XZ4".to_string(), number_row(&[1.0, 2.0, 3.0, 4.0])),
            ]),
        };
        assert_close(
            eval_steyx_surface(&[ref_arg("Y1:Y5"), ref_arg("X1:X5")], &resolver).unwrap(),
            0.174164673034836,
        );
        assert_close(
            eval_steyx_surface(&[ref_arg("YZ1:YZ4"), ref_arg("XZ1:XZ4")], &resolver).unwrap(),
            0.0,
        );
    }

    #[test]
    fn direct_numeric_text_counts_but_reference_text_is_ignored() {
        let resolver = MockResolver {
            cells: HashMap::from([(
                "A1:A4".to_string(),
                array_row(vec![
                    ArrayCellValue::Text(ExcelText::from_utf16_code_units(
                        "5".encode_utf16().collect(),
                    )),
                    ArrayCellValue::Number(7.0),
                    ArrayCellValue::Number(9.0),
                    ArrayCellValue::Number(11.0),
                ]),
            )]),
        };
        assert_close(
            eval_skew_surface(&[num(1.0), text("3"), num(5.0)], &resolver).unwrap(),
            0.0,
        );
        assert_close(
            eval_trimmean_surface(&[ref_arg("A1:A4"), num(0.0)], &resolver).unwrap(),
            9.0,
        );
    }

    #[test]
    fn worksheet_error_and_domain_lanes_are_exercised() {
        let resolver = MockResolver {
            cells: HashMap::from([
                (
                    "CONST1:CONST4".to_string(),
                    number_row(&[4.0, 4.0, 4.0, 4.0]),
                ),
                ("X1:X2".to_string(), number_row(&[1.0, 2.0])),
                ("Y1:Y2".to_string(), number_row(&[1.0, 3.0])),
                (
                    "ERR1:ERR3".to_string(),
                    array_row(vec![
                        ArrayCellValue::Number(1.0),
                        ArrayCellValue::Error(WorksheetErrorCode::NA),
                        ArrayCellValue::Number(3.0),
                    ]),
                ),
            ]),
        };
        assert_eq!(
            eval_kurt_surface(&[ref_arg("CONST1:CONST4")], &resolver),
            Ok(EvalValue::Error(WorksheetErrorCode::Div0))
        );
        assert_eq!(
            eval_steyx_surface(&[ref_arg("Y1:Y2"), ref_arg("X1:X2")], &resolver),
            Ok(EvalValue::Error(WorksheetErrorCode::Div0))
        );
        assert_eq!(
            eval_trimmean_surface(&[ref_arg("CONST1:CONST4"), num(1.0)], &resolver),
            Ok(EvalValue::Error(WorksheetErrorCode::Num))
        );
        assert_eq!(
            eval_skew_surface(&[ref_arg("ERR1:ERR3")], &resolver),
            Err(MomentStatsEvalError::Coercion(
                CoercionError::WorksheetError(WorksheetErrorCode::NA)
            ))
        );
    }

    #[test]
    fn trimmean_rejects_non_scalar_percent_argument() {
        let resolver = MockResolver {
            cells: HashMap::from([("A1:A5".to_string(), number_row(&[1.0, 2.0, 3.0, 4.0, 5.0]))]),
        };
        let percent = CallArgValue::Eval(EvalValue::Array(
            EvalArray::from_rows(vec![vec![
                ArrayCellValue::Number(0.2),
                ArrayCellValue::Number(0.4),
            ]])
            .unwrap(),
        ));
        assert_eq!(
            eval_trimmean_surface(&[ref_arg("A1:A5"), percent], &resolver),
            Ok(EvalValue::Error(WorksheetErrorCode::Value))
        );
    }

    #[test]
    fn error_mapping_matches_house_style() {
        assert_eq!(
            map_moment_stats_error_to_ws(&MomentStatsEvalError::ArityMismatch {
                expected_min: 2,
                expected_max: 2,
                actual: 1,
            }),
            WorksheetErrorCode::Value
        );
        assert_eq!(
            map_moment_stats_error_to_ws(&MomentStatsEvalError::Coercion(
                CoercionError::WorksheetError(WorksheetErrorCode::Div0)
            )),
            WorksheetErrorCode::Div0
        );
    }
}

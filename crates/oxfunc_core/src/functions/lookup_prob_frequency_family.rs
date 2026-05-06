use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::resolver::{ReferenceResolver, resolve_eval_value};
use crate::value::{ArrayCellValue, CallArgValue, EvalValue, WorksheetErrorCode};
use std::collections::BTreeMap;

const PROB_SUM_TOLERANCE: f64 = 1e-7;

const LOOKUP_PROB_FREQUENCY_BASE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.LOOKUP_PROB_FREQUENCY_BASE",
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

pub const LOOKUP_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.LOOKUP",
    arity: Arity { min: 2, max: 3 },
    ..LOOKUP_PROB_FREQUENCY_BASE_META
};

pub const FREQUENCY_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.FREQUENCY",
    arity: Arity::exact(2),
    ..LOOKUP_PROB_FREQUENCY_BASE_META
};

pub const PROB_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.PROB",
    arity: Arity { min: 3, max: 4 },
    ..LOOKUP_PROB_FREQUENCY_BASE_META
};

pub const MODE_MULT_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.MODE.MULT",
    arity: Arity { min: 1, max: 255 },
    ..LOOKUP_PROB_FREQUENCY_BASE_META
};

#[derive(Debug, Clone, PartialEq)]
pub enum LookupProbFrequencyEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
    Domain(WorksheetErrorCode),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VectorOrientation {
    Row,
    Column,
}

#[derive(Debug, Clone, PartialEq)]
struct ScalarVector {
    lookup_keys: Vec<f64>,
    result_values: Vec<EvalValue>,
}

fn arity_error(meta: &FunctionMeta, actual: usize) -> LookupProbFrequencyEvalError {
    LookupProbFrequencyEvalError::ArityMismatch {
        expected_min: meta.arity.min,
        expected_max: meta.arity.max,
        actual,
    }
}

fn resolve_arg_eval(
    arg: &CallArgValue,
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, LookupProbFrequencyEvalError> {
    match arg {
        CallArgValue::Reference(reference)
        | CallArgValue::Eval(EvalValue::Reference(reference)) => {
            resolve_eval_value(resolver, reference)
                .map_err(CoercionError::RefResolution)
                .map_err(LookupProbFrequencyEvalError::Coercion)
        }
        CallArgValue::Eval(value) => Ok(value.clone()),
        CallArgValue::MissingArg => Err(LookupProbFrequencyEvalError::Coercion(
            CoercionError::MissingArg,
        )),
        CallArgValue::EmptyCell => Err(LookupProbFrequencyEvalError::Domain(
            WorksheetErrorCode::Value,
        )),
    }
}

fn optional_arg_value(
    arg: Option<&CallArgValue>,
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<Option<EvalValue>, LookupProbFrequencyEvalError> {
    match arg {
        None | Some(CallArgValue::MissingArg) => Ok(None),
        Some(other) => Ok(Some(resolve_arg_eval(other, resolver)?)),
    }
}

fn scalar_number_from_eval(value: &EvalValue) -> Result<f64, LookupProbFrequencyEvalError> {
    match value {
        EvalValue::Number(n) => Ok(*n),
        EvalValue::Logical(flag) => Ok(if *flag { 1.0 } else { 0.0 }),
        EvalValue::Error(code) => Err(LookupProbFrequencyEvalError::Domain(*code)),
        EvalValue::Array(array) => {
            let shape = array.shape();
            if shape.rows == 1 && shape.cols == 1 {
                number_from_cell(array.get(0, 0).expect("single cell"), false)
            } else {
                Err(LookupProbFrequencyEvalError::Domain(
                    WorksheetErrorCode::Value,
                ))
            }
        }
        EvalValue::Text(_) | EvalValue::Reference(_) | EvalValue::Lambda(_) => Err(
            LookupProbFrequencyEvalError::Domain(WorksheetErrorCode::Value),
        ),
    }
}

fn number_from_cell(
    cell: &ArrayCellValue,
    ignore_non_numeric: bool,
) -> Result<f64, LookupProbFrequencyEvalError> {
    match cell {
        ArrayCellValue::Number(n) => Ok(*n),
        ArrayCellValue::Error(code) => Err(LookupProbFrequencyEvalError::Domain(*code)),
        ArrayCellValue::Text(_) | ArrayCellValue::Logical(_) | ArrayCellValue::EmptyCell => {
            if ignore_non_numeric {
                Err(LookupProbFrequencyEvalError::Coercion(
                    CoercionError::UnsupportedValueKind("ignored_non_numeric"),
                ))
            } else {
                Err(LookupProbFrequencyEvalError::Domain(
                    WorksheetErrorCode::Value,
                ))
            }
        }
    }
}

fn eval_from_cell(cell: &ArrayCellValue) -> EvalValue {
    match cell {
        ArrayCellValue::Number(n) => EvalValue::Number(*n),
        ArrayCellValue::Text(t) => EvalValue::Text(t.clone()),
        ArrayCellValue::Logical(b) => EvalValue::Logical(*b),
        ArrayCellValue::Error(code) => EvalValue::Error(*code),
        ArrayCellValue::EmptyCell => EvalValue::Number(0.0),
    }
}

fn collect_numeric_vector(
    value: &EvalValue,
    ignore_non_numeric: bool,
) -> Result<Vec<f64>, LookupProbFrequencyEvalError> {
    match value {
        EvalValue::Number(n) => Ok(vec![*n]),
        EvalValue::Logical(_) | EvalValue::Text(_) => {
            if ignore_non_numeric {
                Ok(Vec::new())
            } else {
                Err(LookupProbFrequencyEvalError::Domain(
                    WorksheetErrorCode::Value,
                ))
            }
        }
        EvalValue::Error(code) => Err(LookupProbFrequencyEvalError::Domain(*code)),
        EvalValue::Array(array) => {
            let shape = array.shape();
            if shape.rows > 1 && shape.cols > 1 {
                return Err(LookupProbFrequencyEvalError::Domain(
                    WorksheetErrorCode::Ref,
                ));
            }
            let mut out = Vec::with_capacity(shape.rows * shape.cols);
            for cell in array.iter_row_major() {
                match number_from_cell(cell, ignore_non_numeric) {
                    Ok(n) => out.push(n),
                    Err(LookupProbFrequencyEvalError::Coercion(
                        CoercionError::UnsupportedValueKind("ignored_non_numeric"),
                    )) => {}
                    Err(err) => return Err(err),
                }
            }
            Ok(out)
        }
        EvalValue::Reference(_) | EvalValue::Lambda(_) => Err(
            LookupProbFrequencyEvalError::Domain(WorksheetErrorCode::Value),
        ),
    }
}

fn collect_numeric_vector_arg(
    arg: &CallArgValue,
    resolver: &(impl ReferenceResolver + ?Sized),
    ignore_non_numeric: bool,
) -> Result<Vec<f64>, LookupProbFrequencyEvalError> {
    let eval = resolve_arg_eval(arg, resolver)?;
    collect_numeric_vector(&eval, ignore_non_numeric)
}

fn extract_lookup_vector(value: &EvalValue) -> Result<ScalarVector, LookupProbFrequencyEvalError> {
    match value {
        EvalValue::Number(n) => Ok(ScalarVector {
            lookup_keys: vec![*n],
            result_values: vec![EvalValue::Number(*n)],
        }),
        EvalValue::Array(array) => {
            let shape = array.shape();
            if shape.rows == 1 || shape.cols == 1 {
                let mut keys = Vec::with_capacity(shape.rows * shape.cols);
                let mut values = Vec::with_capacity(shape.rows * shape.cols);
                for cell in array.iter_row_major() {
                    keys.push(number_from_cell(cell, false)?);
                    values.push(eval_from_cell(cell));
                }
                Ok(ScalarVector {
                    lookup_keys: keys,
                    result_values: values,
                })
            } else {
                let orientation = if shape.rows > shape.cols {
                    VectorOrientation::Column
                } else {
                    VectorOrientation::Row
                };
                let mut keys = Vec::new();
                let mut values = Vec::new();
                match orientation {
                    VectorOrientation::Column => {
                        for row in 0..shape.rows {
                            let key_cell = array.get(row, 0).expect("bounded");
                            let result_cell = array.get(row, shape.cols - 1).expect("bounded");
                            keys.push(number_from_cell(key_cell, false)?);
                            values.push(eval_from_cell(result_cell));
                        }
                    }
                    VectorOrientation::Row => {
                        for col in 0..shape.cols {
                            let key_cell = array.get(0, col).expect("bounded");
                            let result_cell = array.get(shape.rows - 1, col).expect("bounded");
                            keys.push(number_from_cell(key_cell, false)?);
                            values.push(eval_from_cell(result_cell));
                        }
                    }
                }
                Ok(ScalarVector {
                    lookup_keys: keys,
                    result_values: values,
                })
            }
        }
        EvalValue::Error(code) => Err(LookupProbFrequencyEvalError::Domain(*code)),
        EvalValue::Text(_)
        | EvalValue::Logical(_)
        | EvalValue::Reference(_)
        | EvalValue::Lambda(_) => Err(LookupProbFrequencyEvalError::Domain(
            WorksheetErrorCode::Value,
        )),
    }
}

fn extract_result_vector(
    value: &EvalValue,
    expected_len: usize,
) -> Result<Vec<EvalValue>, LookupProbFrequencyEvalError> {
    match value {
        EvalValue::Number(n) => {
            if expected_len == 1 {
                Ok(vec![EvalValue::Number(*n)])
            } else {
                Err(LookupProbFrequencyEvalError::Domain(
                    WorksheetErrorCode::Ref,
                ))
            }
        }
        EvalValue::Array(array) => {
            let shape = array.shape();
            if shape.rows > 1 && shape.cols > 1 {
                return Err(LookupProbFrequencyEvalError::Domain(
                    WorksheetErrorCode::Ref,
                ));
            }
            let mut out = Vec::with_capacity(shape.rows * shape.cols);
            for cell in array.iter_row_major() {
                out.push(eval_from_cell(cell));
            }
            if out.len() == expected_len {
                Ok(out)
            } else {
                Err(LookupProbFrequencyEvalError::Domain(
                    WorksheetErrorCode::Ref,
                ))
            }
        }
        EvalValue::Error(code) => Err(LookupProbFrequencyEvalError::Domain(*code)),
        EvalValue::Text(t) => {
            if expected_len == 1 {
                Ok(vec![EvalValue::Text(t.clone())])
            } else {
                Err(LookupProbFrequencyEvalError::Domain(
                    WorksheetErrorCode::Ref,
                ))
            }
        }
        EvalValue::Logical(b) => {
            if expected_len == 1 {
                Ok(vec![EvalValue::Logical(*b)])
            } else {
                Err(LookupProbFrequencyEvalError::Domain(
                    WorksheetErrorCode::Ref,
                ))
            }
        }
        EvalValue::Reference(_) | EvalValue::Lambda(_) => Err(
            LookupProbFrequencyEvalError::Domain(WorksheetErrorCode::Value),
        ),
    }
}

fn sorted_non_decreasing(values: &[f64]) -> bool {
    values.windows(2).all(|w| w[0] <= w[1])
}

fn lookup_kernel(
    lookup_value: f64,
    lookup_vector: &[f64],
    result_vector: &[EvalValue],
) -> Result<EvalValue, WorksheetErrorCode> {
    if lookup_vector.is_empty()
        || result_vector.is_empty()
        || lookup_vector.len() != result_vector.len()
    {
        return Err(WorksheetErrorCode::Ref);
    }
    if !sorted_non_decreasing(lookup_vector) {
        return Err(WorksheetErrorCode::NA);
    }
    let mut best: Option<usize> = None;
    for (idx, candidate) in lookup_vector.iter().enumerate() {
        if *candidate <= lookup_value {
            best = Some(idx);
        } else {
            break;
        }
    }
    best.map(|idx| result_vector[idx].clone())
        .ok_or(WorksheetErrorCode::NA)
}

fn vertical_number_array(values: &[f64]) -> EvalValue {
    EvalValue::Array(
        crate::value::EvalArray::from_rows(
            values
                .iter()
                .copied()
                .map(|n| vec![ArrayCellValue::Number(n)])
                .collect(),
        )
        .expect("non-empty vertical array"),
    )
}

fn frequency_kernel(data: &[f64], bins: &[f64]) -> Result<EvalValue, WorksheetErrorCode> {
    if !sorted_non_decreasing(bins) {
        return Err(WorksheetErrorCode::Num);
    }
    let mut counts = vec![0.0; bins.len() + 1];
    for value in data {
        let mut placed = false;
        for (idx, bin) in bins.iter().enumerate() {
            if *value <= *bin {
                counts[idx] += 1.0;
                placed = true;
                break;
            }
        }
        if !placed {
            counts[bins.len()] += 1.0;
        }
    }
    Ok(vertical_number_array(&counts))
}

fn prob_kernel(
    x_values: &[f64],
    probabilities: &[f64],
    lower: f64,
    upper: Option<f64>,
) -> Result<f64, WorksheetErrorCode> {
    if x_values.len() != probabilities.len() || x_values.is_empty() {
        return Err(WorksheetErrorCode::NA);
    }
    if probabilities
        .iter()
        .any(|p| !p.is_finite() || *p < 0.0 || *p > 1.0)
    {
        return Err(WorksheetErrorCode::Num);
    }
    let sum = probabilities.iter().sum::<f64>();
    if (sum - 1.0).abs() > PROB_SUM_TOLERANCE {
        return Err(WorksheetErrorCode::Num);
    }
    let upper = upper.unwrap_or(lower);
    if upper < lower || !lower.is_finite() || !upper.is_finite() {
        return Err(WorksheetErrorCode::Num);
    }
    let mut total = 0.0;
    for (x, p) in x_values.iter().zip(probabilities.iter()) {
        if *x >= lower && *x <= upper {
            total += *p;
        }
    }
    Ok(total)
}

fn mode_mult_kernel(values: &[f64]) -> Result<EvalValue, WorksheetErrorCode> {
    let mut counts: BTreeMap<u64, (f64, usize)> = BTreeMap::new();
    for value in values {
        let entry = counts.entry(value.to_bits()).or_insert((*value, 0));
        entry.1 += 1;
    }
    let max_count = counts.values().map(|(_, c)| *c).max().unwrap_or(0);
    if max_count < 2 {
        return Err(WorksheetErrorCode::NA);
    }
    let mut modes = counts
        .values()
        .filter(|(_, c)| *c == max_count)
        .map(|(v, _)| *v)
        .collect::<Vec<_>>();
    modes.sort_by(|a, b| a.partial_cmp(b).expect("finite values"));
    Ok(vertical_number_array(&modes))
}

pub fn eval_lookup_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, LookupProbFrequencyEvalError> {
    if !LOOKUP_META.arity.accepts(args.len()) {
        return Err(arity_error(&LOOKUP_META, args.len()));
    }
    let lookup_value = scalar_number_from_eval(&resolve_arg_eval(&args[0], resolver)?)?;
    let lookup_eval = resolve_arg_eval(&args[1], resolver)?;
    let lookup_vector = extract_lookup_vector(&lookup_eval)?;
    let result_vector = match args.get(2) {
        Some(arg) => extract_result_vector(
            &resolve_arg_eval(arg, resolver)?,
            lookup_vector.lookup_keys.len(),
        )?,
        None => lookup_vector.result_values.clone(),
    };
    lookup_kernel(lookup_value, &lookup_vector.lookup_keys, &result_vector)
        .map_err(LookupProbFrequencyEvalError::Domain)
}

pub fn eval_frequency_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, LookupProbFrequencyEvalError> {
    if !FREQUENCY_META.arity.accepts(args.len()) {
        return Err(arity_error(&FREQUENCY_META, args.len()));
    }
    let data = collect_numeric_vector_arg(&args[0], resolver, true)?;
    let bins = collect_numeric_vector_arg(&args[1], resolver, true)?;
    frequency_kernel(&data, &bins).map_err(LookupProbFrequencyEvalError::Domain)
}

pub fn eval_prob_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, LookupProbFrequencyEvalError> {
    if !PROB_META.arity.accepts(args.len()) {
        return Err(arity_error(&PROB_META, args.len()));
    }
    let x_values = collect_numeric_vector_arg(&args[0], resolver, false)?;
    let probabilities = collect_numeric_vector_arg(&args[1], resolver, false)?;
    let lower = scalar_number_from_eval(&resolve_arg_eval(&args[2], resolver)?)?;
    let upper = optional_arg_value(args.get(3), resolver)?
        .map(|value| scalar_number_from_eval(&value))
        .transpose()?;
    prob_kernel(&x_values, &probabilities, lower, upper)
        .map(EvalValue::Number)
        .map_err(LookupProbFrequencyEvalError::Domain)
}

pub fn eval_mode_mult_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, LookupProbFrequencyEvalError> {
    if !MODE_MULT_META.arity.accepts(args.len()) {
        return Err(arity_error(&MODE_MULT_META, args.len()));
    }
    let mut values = Vec::new();
    for arg in args {
        values.extend(collect_numeric_vector_arg(arg, resolver, true)?);
    }
    mode_mult_kernel(&values).map_err(LookupProbFrequencyEvalError::Domain)
}

pub fn map_lookup_prob_frequency_error_to_ws(
    error: &LookupProbFrequencyEvalError,
) -> WorksheetErrorCode {
    match error {
        LookupProbFrequencyEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        LookupProbFrequencyEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        LookupProbFrequencyEvalError::Coercion(_) => WorksheetErrorCode::Value,
        LookupProbFrequencyEvalError::Domain(code) => *code,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::{RefResolutionError, ResolverCapabilities};
    use crate::value::{EvalArray, ExcelText, ReferenceKind, ReferenceLike};
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

    fn num(n: f64) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Number(n))
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

    fn row(values: &[f64]) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Array(
            EvalArray::from_rows(vec![
                values.iter().copied().map(ArrayCellValue::Number).collect(),
            ])
            .unwrap(),
        ))
    }

    fn ref_arg(target: &str) -> CallArgValue {
        CallArgValue::Reference(ReferenceLike {
            kind: ReferenceKind::Area,
            target: target.to_string(),
        })
    }

    fn expect_vertical_numbers(value: EvalValue) -> Vec<f64> {
        match value {
            EvalValue::Array(array) => array
                .iter_row_major()
                .map(|cell| match cell {
                    ArrayCellValue::Number(n) => *n,
                    other => panic!("unexpected cell: {other:?}"),
                })
                .collect(),
            other => panic!("expected array, got {other:?}"),
        }
    }

    #[test]
    fn metadata_matches_batch_shape() {
        assert_eq!(LOOKUP_META.function_id, "FUNC.LOOKUP");
        assert_eq!(FREQUENCY_META.arity, Arity::exact(2));
        assert_eq!(PROB_META.arity, Arity { min: 3, max: 4 });
        assert_eq!(
            MODE_MULT_META.surface_fec_dependency_profile,
            FecDependencyProfile::RefOnly
        );
    }

    #[test]
    fn lookup_vector_form_uses_approximate_last_less_or_equal() {
        let got = eval_lookup_surface(
            &[num(2.9), row(&[1.0, 2.0, 3.0]), row(&[10.0, 20.0, 30.0])],
            &MockResolver {
                map: BTreeMap::new(),
            },
        )
        .unwrap();
        assert_eq!(got, EvalValue::Number(20.0));
    }

    #[test]
    fn lookup_array_form_uses_first_column_and_last_column_when_tall() {
        let array = CallArgValue::Eval(EvalValue::Array(
            EvalArray::from_rows(vec![
                vec![ArrayCellValue::Number(1.0), ArrayCellValue::Number(10.0)],
                vec![ArrayCellValue::Number(2.0), ArrayCellValue::Number(20.0)],
                vec![ArrayCellValue::Number(3.0), ArrayCellValue::Number(30.0)],
            ])
            .unwrap(),
        ));
        let got = eval_lookup_surface(
            &[num(2.9), array],
            &MockResolver {
                map: BTreeMap::new(),
            },
        )
        .unwrap();
        assert_eq!(got, EvalValue::Number(20.0));
    }

    #[test]
    fn lookup_can_return_text_from_result_vector() {
        let result = CallArgValue::Eval(EvalValue::Array(
            EvalArray::from_rows(vec![vec![
                ArrayCellValue::Text(ExcelText::from_utf16_code_units(
                    "a".encode_utf16().collect(),
                )),
                ArrayCellValue::Text(ExcelText::from_utf16_code_units(
                    "b".encode_utf16().collect(),
                )),
                ArrayCellValue::Text(ExcelText::from_utf16_code_units(
                    "c".encode_utf16().collect(),
                )),
            ]])
            .unwrap(),
        ));
        let got = eval_lookup_surface(
            &[num(2.2), row(&[1.0, 2.0, 3.0]), result],
            &MockResolver {
                map: BTreeMap::new(),
            },
        )
        .unwrap();
        match got {
            EvalValue::Text(t) => assert_eq!(t.to_string_lossy(), "b"),
            other => panic!("expected text, got {other:?}"),
        }
    }

    #[test]
    fn frequency_returns_vertical_counts() {
        let got = eval_frequency_surface(
            &[col(&[1.0, 2.0, 2.0, 3.0, 4.0]), col(&[2.0, 3.0])],
            &MockResolver {
                map: BTreeMap::new(),
            },
        )
        .unwrap();
        assert_eq!(expect_vertical_numbers(got), vec![3.0, 1.0, 1.0]);
    }

    #[test]
    fn prob_supports_point_and_interval_queries() {
        let point = eval_prob_surface(
            &[col(&[0.0, 1.0, 2.0]), col(&[0.2, 0.3, 0.5]), num(1.0)],
            &MockResolver {
                map: BTreeMap::new(),
            },
        )
        .unwrap();
        assert_eq!(point, EvalValue::Number(0.3));

        let interval = eval_prob_surface(
            &[
                col(&[0.0, 1.0, 2.0]),
                col(&[0.2, 0.3, 0.5]),
                num(1.0),
                num(2.0),
            ],
            &MockResolver {
                map: BTreeMap::new(),
            },
        )
        .unwrap();
        assert_eq!(interval, EvalValue::Number(0.8));
    }

    #[test]
    fn prob_rejects_probability_vectors_that_do_not_sum_to_one() {
        let err = eval_prob_surface(
            &[col(&[0.0, 1.0, 2.0]), col(&[0.2, 0.3, 0.4]), num(1.0)],
            &MockResolver {
                map: BTreeMap::new(),
            },
        )
        .unwrap_err();
        assert_eq!(
            map_lookup_prob_frequency_error_to_ws(&err),
            WorksheetErrorCode::Num
        );
    }

    #[test]
    fn mode_mult_returns_all_modes_sorted_ascending() {
        let got = eval_mode_mult_surface(
            &[col(&[1.0, 2.0, 2.0, 3.0, 3.0])],
            &MockResolver {
                map: BTreeMap::new(),
            },
        )
        .unwrap();
        assert_eq!(expect_vertical_numbers(got), vec![2.0, 3.0]);
    }

    #[test]
    fn mode_mult_returns_na_when_no_mode_survives() {
        let error = eval_mode_mult_surface(
            &[col(&[1.0, 2.0, 3.0])],
            &MockResolver {
                map: BTreeMap::new(),
            },
        )
        .unwrap_err();
        assert_eq!(
            map_lookup_prob_frequency_error_to_ws(&error),
            WorksheetErrorCode::NA
        );
    }

    #[test]
    fn reference_resolution_is_admitted_for_prob() {
        let mut map = BTreeMap::new();
        map.insert(
            "A1:A3".to_string(),
            EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Number(0.0)],
                    vec![ArrayCellValue::Number(1.0)],
                    vec![ArrayCellValue::Number(2.0)],
                ])
                .unwrap(),
            ),
        );
        map.insert(
            "B1:B3".to_string(),
            EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Number(0.2)],
                    vec![ArrayCellValue::Number(0.3)],
                    vec![ArrayCellValue::Number(0.5)],
                ])
                .unwrap(),
            ),
        );
        let got = eval_prob_surface(
            &[ref_arg("A1:A3"), ref_arg("B1:B3"), num(1.0), num(2.0)],
            &MockResolver { map },
        )
        .unwrap();
        assert_eq!(got, EvalValue::Number(0.8));
    }

    #[test]
    fn unsorted_lookup_and_bins_are_out_of_slice() {
        let lookup_err = eval_lookup_surface(
            &[num(2.0), row(&[2.0, 1.0, 3.0]), row(&[20.0, 10.0, 30.0])],
            &MockResolver {
                map: BTreeMap::new(),
            },
        )
        .unwrap_err();
        assert_eq!(
            map_lookup_prob_frequency_error_to_ws(&lookup_err),
            WorksheetErrorCode::NA
        );

        let freq_err = eval_frequency_surface(
            &[col(&[1.0, 2.0, 3.0]), col(&[3.0, 2.0])],
            &MockResolver {
                map: BTreeMap::new(),
            },
        )
        .unwrap_err();
        assert_eq!(
            map_lookup_prob_frequency_error_to_ws(&freq_err),
            WorksheetErrorCode::Num
        );
    }

    #[test]
    fn matrix_lookup_vector_with_matching_result_vector_uses_bounded_heuristic() {
        let matrix = CallArgValue::Eval(EvalValue::Array(
            EvalArray::from_rows(vec![
                vec![ArrayCellValue::Number(1.0), ArrayCellValue::Number(2.0)],
                vec![ArrayCellValue::Number(3.0), ArrayCellValue::Number(4.0)],
                vec![ArrayCellValue::Number(5.0), ArrayCellValue::Number(6.0)],
            ])
            .unwrap(),
        ));
        let got = eval_lookup_surface(
            &[num(2.0), matrix, row(&[10.0, 20.0, 30.0])],
            &MockResolver {
                map: BTreeMap::new(),
            },
        )
        .unwrap();
        assert_eq!(got, EvalValue::Number(10.0));
    }
}

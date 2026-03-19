use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::resolver::{ReferenceResolver, resolve_eval_value};
use crate::value::{ArrayCellValue, CallArgValue, EvalArray, EvalValue, WorksheetErrorCode};

const REGRESSION_FORECAST_BASE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.REGRESSION_FORECAST_BASE",
    arity: Arity { min: 1, max: 4 },
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

pub const GROWTH_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.GROWTH",
    ..REGRESSION_FORECAST_BASE_META
};

pub const FORECAST_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.FORECAST",
    arity: Arity::exact(3),
    ..REGRESSION_FORECAST_BASE_META
};

pub const FORECAST_LINEAR_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.FORECAST.LINEAR",
    arity: Arity::exact(3),
    ..REGRESSION_FORECAST_BASE_META
};

pub const TREND_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.TREND",
    ..REGRESSION_FORECAST_BASE_META
};

pub const LINEST_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.LINEST",
    ..REGRESSION_FORECAST_BASE_META
};

pub const LOGEST_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.LOGEST",
    ..REGRESSION_FORECAST_BASE_META
};

#[derive(Debug, Clone, PartialEq)]
pub enum RegressionForecastEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
    Domain(WorksheetErrorCode),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VectorShape {
    Scalar,
    Row(usize),
    Column(usize),
}

#[derive(Debug, Clone, PartialEq)]
struct NumericVector {
    values: Vec<f64>,
    shape: VectorShape,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OutputShape {
    Scalar,
    Row(usize),
    Column(usize),
    Matrix { rows: usize, cols: usize },
}

#[derive(Debug, Clone, PartialEq)]
struct NumericMatrix {
    rows: usize,
    cols: usize,
    values: Vec<f64>,
}

#[derive(Debug, Clone, PartialEq)]
struct PreparedPredictorInput {
    predictors: NumericMatrix,
    output_shape: OutputShape,
}

#[derive(Debug, Clone, PartialEq)]
struct LinearRegressionModel {
    coefficients: Vec<f64>,
    intercept: f64,
}

fn arity_error(meta: &FunctionMeta, actual: usize) -> RegressionForecastEvalError {
    RegressionForecastEvalError::ArityMismatch {
        expected_min: meta.arity.min,
        expected_max: meta.arity.max,
        actual,
    }
}

fn resolve_arg_eval(
    arg: &CallArgValue,
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, RegressionForecastEvalError> {
    match arg {
        CallArgValue::Reference(reference)
        | CallArgValue::Eval(EvalValue::Reference(reference)) => {
            resolve_eval_value(resolver, reference)
                .map_err(CoercionError::RefResolution)
                .map_err(RegressionForecastEvalError::Coercion)
        }
        CallArgValue::Eval(value) => Ok(value.clone()),
        CallArgValue::MissingArg => Err(RegressionForecastEvalError::Coercion(
            CoercionError::MissingArg,
        )),
        CallArgValue::EmptyCell => Err(RegressionForecastEvalError::Domain(
            WorksheetErrorCode::Value,
        )),
    }
}

fn optional_arg_value(
    arg: Option<&CallArgValue>,
    resolver: &impl ReferenceResolver,
) -> Result<Option<EvalValue>, RegressionForecastEvalError> {
    match arg {
        None | Some(CallArgValue::MissingArg) => Ok(None),
        Some(other) => Ok(Some(resolve_arg_eval(other, resolver)?)),
    }
}

fn numeric_from_cell(cell: &ArrayCellValue) -> Result<f64, RegressionForecastEvalError> {
    match cell {
        ArrayCellValue::Number(n) => Ok(*n),
        ArrayCellValue::Error(code) => Err(RegressionForecastEvalError::Domain(*code)),
        ArrayCellValue::Text(_) | ArrayCellValue::Logical(_) | ArrayCellValue::EmptyCell => Err(
            RegressionForecastEvalError::Domain(WorksheetErrorCode::Value),
        ),
    }
}

fn numeric_scalar_from_eval(value: &EvalValue) -> Result<f64, RegressionForecastEvalError> {
    match value {
        EvalValue::Number(n) => Ok(*n),
        EvalValue::Logical(flag) => Ok(if *flag { 1.0 } else { 0.0 }),
        EvalValue::Error(code) => Err(RegressionForecastEvalError::Domain(*code)),
        EvalValue::Array(array) => {
            let shape = array.shape();
            if shape.rows == 1 && shape.cols == 1 {
                numeric_from_cell(array.get(0, 0).expect("single-cell array"))
            } else {
                Err(RegressionForecastEvalError::Domain(
                    WorksheetErrorCode::Value,
                ))
            }
        }
        EvalValue::Text(_) | EvalValue::Reference(_) | EvalValue::Lambda(_) => Err(
            RegressionForecastEvalError::Domain(WorksheetErrorCode::Value),
        ),
    }
}

fn bool_flag_from_eval_or_default(
    value: Option<EvalValue>,
    default: bool,
) -> Result<bool, RegressionForecastEvalError> {
    match value {
        None => Ok(default),
        Some(EvalValue::Logical(flag)) => Ok(flag),
        Some(other) => Ok(numeric_scalar_from_eval(&other)? != 0.0),
    }
}

fn collect_numeric_vector_from_eval(
    value: &EvalValue,
) -> Result<NumericVector, RegressionForecastEvalError> {
    match value {
        EvalValue::Number(n) => Ok(NumericVector {
            values: vec![*n],
            shape: VectorShape::Scalar,
        }),
        EvalValue::Error(code) => Err(RegressionForecastEvalError::Domain(*code)),
        EvalValue::Array(array) => {
            let shape = array.shape();
            if shape.rows > 1 && shape.cols > 1 {
                return Err(RegressionForecastEvalError::Domain(WorksheetErrorCode::Ref));
            }
            let mut values = Vec::with_capacity(shape.rows * shape.cols);
            for cell in array.iter_row_major() {
                values.push(numeric_from_cell(cell)?);
            }
            let vector_shape = if shape.rows == 1 && shape.cols == 1 {
                VectorShape::Scalar
            } else if shape.rows == 1 {
                VectorShape::Row(shape.cols)
            } else {
                VectorShape::Column(shape.rows)
            };
            Ok(NumericVector {
                values,
                shape: vector_shape,
            })
        }
        EvalValue::Text(_)
        | EvalValue::Logical(_)
        | EvalValue::Reference(_)
        | EvalValue::Lambda(_) => Err(RegressionForecastEvalError::Domain(
            WorksheetErrorCode::Value,
        )),
    }
}

fn collect_numeric_vector_arg(
    arg: &CallArgValue,
    resolver: &impl ReferenceResolver,
) -> Result<NumericVector, RegressionForecastEvalError> {
    let eval = resolve_arg_eval(arg, resolver)?;
    collect_numeric_vector_from_eval(&eval)
}

impl NumericMatrix {
    fn cell(&self, row: usize, col: usize) -> f64 {
        self.values[row * self.cols + col]
    }

    fn transpose(&self) -> Self {
        let mut values = vec![0.0; self.rows * self.cols];
        for row in 0..self.rows {
            for col in 0..self.cols {
                values[col * self.rows + row] = self.cell(row, col);
            }
        }
        Self {
            rows: self.cols,
            cols: self.rows,
            values,
        }
    }

    fn row_values(&self, row: usize) -> &[f64] {
        let start = row * self.cols;
        &self.values[start..(start + self.cols)]
    }
}

fn collect_numeric_matrix_from_eval(
    value: &EvalValue,
) -> Result<NumericMatrix, RegressionForecastEvalError> {
    match value {
        EvalValue::Number(n) => Ok(NumericMatrix {
            rows: 1,
            cols: 1,
            values: vec![*n],
        }),
        EvalValue::Error(code) => Err(RegressionForecastEvalError::Domain(*code)),
        EvalValue::Array(array) => {
            let shape = array.shape();
            let mut values = Vec::with_capacity(shape.rows * shape.cols);
            for cell in array.iter_row_major() {
                values.push(numeric_from_cell(cell)?);
            }
            Ok(NumericMatrix {
                rows: shape.rows,
                cols: shape.cols,
                values,
            })
        }
        EvalValue::Text(_)
        | EvalValue::Logical(_)
        | EvalValue::Reference(_)
        | EvalValue::Lambda(_) => Err(RegressionForecastEvalError::Domain(
            WorksheetErrorCode::Value,
        )),
    }
}

fn collect_numeric_matrix_arg(
    arg: &CallArgValue,
    resolver: &impl ReferenceResolver,
) -> Result<NumericMatrix, RegressionForecastEvalError> {
    let eval = resolve_arg_eval(arg, resolver)?;
    collect_numeric_matrix_from_eval(&eval)
}

fn sequence_predictors(len: usize) -> NumericMatrix {
    NumericMatrix {
        rows: len,
        cols: 1,
        values: (1..=len).map(|n| n as f64).collect(),
    }
}

fn default_output_shape_for_known_y(shape: VectorShape) -> OutputShape {
    match shape {
        VectorShape::Scalar => OutputShape::Scalar,
        VectorShape::Row(cols) => OutputShape::Row(cols),
        VectorShape::Column(rows) => OutputShape::Column(rows),
    }
}

fn normalize_known_x_matrix(
    raw: NumericMatrix,
    observation_count: usize,
) -> Result<NumericMatrix, RegressionForecastEvalError> {
    if raw.rows == observation_count {
        Ok(raw)
    } else if raw.rows == 1 && raw.cols == observation_count {
        Ok(raw.transpose())
    } else {
        Err(RegressionForecastEvalError::Domain(WorksheetErrorCode::Ref))
    }
}

fn prepare_predictor_input(
    raw: Option<NumericMatrix>,
    known_y: &NumericVector,
    treat_single_predictor_cells_as_observations: bool,
) -> Result<PreparedPredictorInput, RegressionForecastEvalError> {
    let observation_count = known_y.values.len();
    match raw {
        None => Ok(PreparedPredictorInput {
            predictors: sequence_predictors(observation_count),
            output_shape: default_output_shape_for_known_y(known_y.shape),
        }),
        Some(matrix) => {
            let normalized = normalize_known_x_matrix(matrix.clone(), observation_count)?;
            if treat_single_predictor_cells_as_observations && normalized.cols == 1 {
                let output_shape = if matrix.rows == 1 && matrix.cols == 1 {
                    OutputShape::Scalar
                } else if matrix.rows == 1 {
                    OutputShape::Row(matrix.cols)
                } else if matrix.cols == 1 {
                    OutputShape::Column(matrix.rows)
                } else {
                    OutputShape::Matrix {
                        rows: matrix.rows,
                        cols: matrix.cols,
                    }
                };
                return Ok(PreparedPredictorInput {
                    predictors: NumericMatrix {
                        rows: matrix.rows * matrix.cols,
                        cols: 1,
                        values: matrix.values,
                    },
                    output_shape,
                });
            }

            if normalized.cols == 1 {
                let output_shape = if matrix.rows == 1 && matrix.cols == 1 {
                    OutputShape::Scalar
                } else if matrix.rows == 1 {
                    OutputShape::Row(matrix.cols)
                } else {
                    OutputShape::Column(matrix.rows)
                };
                return Ok(PreparedPredictorInput {
                    predictors: normalized,
                    output_shape,
                });
            }

            if matrix.cols != normalized.cols {
                return Err(RegressionForecastEvalError::Domain(WorksheetErrorCode::Ref));
            }
            let output_shape = if matrix.rows == 1 {
                OutputShape::Scalar
            } else {
                OutputShape::Column(matrix.rows)
            };
            Ok(PreparedPredictorInput {
                predictors: normalized,
                output_shape,
            })
        }
    }
}

fn predictor_input_from_optional(
    raw: Option<NumericMatrix>,
    known_y: &NumericVector,
) -> Result<PreparedPredictorInput, RegressionForecastEvalError> {
    prepare_predictor_input(raw, known_y, false)
}

fn prediction_input_from_optional(
    raw: Option<NumericMatrix>,
    known_y: &NumericVector,
    known_x: &PreparedPredictorInput,
) -> Result<PreparedPredictorInput, RegressionForecastEvalError> {
    match raw {
        None => Ok(PreparedPredictorInput {
            predictors: known_x.predictors.clone(),
            output_shape: default_output_shape_for_known_y(known_y.shape),
        }),
        Some(matrix) => {
            let predictor_count = known_x.predictors.cols;
            if predictor_count == 1 {
                let output_shape = if matrix.rows == 1 && matrix.cols == 1 {
                    OutputShape::Scalar
                } else if matrix.rows == 1 {
                    OutputShape::Row(matrix.cols)
                } else if matrix.cols == 1 {
                    OutputShape::Column(matrix.rows)
                } else {
                    OutputShape::Matrix {
                        rows: matrix.rows,
                        cols: matrix.cols,
                    }
                };
                return Ok(PreparedPredictorInput {
                    predictors: NumericMatrix {
                        rows: matrix.rows * matrix.cols,
                        cols: 1,
                        values: matrix.values,
                    },
                    output_shape,
                });
            }

            if matrix.cols != predictor_count {
                return Err(RegressionForecastEvalError::Domain(WorksheetErrorCode::Ref));
            }
            let output_shape = if matrix.rows == 1 {
                OutputShape::Scalar
            } else {
                OutputShape::Column(matrix.rows)
            };
            Ok(PreparedPredictorInput {
                predictors: matrix,
                output_shape,
            })
        }
    }
}

fn transpose(matrix: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let rows = matrix.len();
    let cols = matrix.first().map_or(0, Vec::len);
    let mut result = vec![vec![0.0; rows]; cols];
    for (row_idx, row) in matrix.iter().enumerate() {
        for (col_idx, value) in row.iter().enumerate() {
            result[col_idx][row_idx] = *value;
        }
    }
    result
}

fn mat_mul(left: &[Vec<f64>], right: &[Vec<f64>]) -> Result<Vec<Vec<f64>>, WorksheetErrorCode> {
    if left.is_empty() || right.is_empty() {
        return Err(WorksheetErrorCode::Value);
    }
    let left_cols = left[0].len();
    let right_cols = right[0].len();
    if left.iter().any(|row| row.len() != left_cols)
        || right.iter().any(|row| row.len() != right_cols)
        || left_cols != right.len()
    {
        return Err(WorksheetErrorCode::Value);
    }
    let mut result = vec![vec![0.0; right_cols]; left.len()];
    for row in 0..left.len() {
        for col in 0..right_cols {
            let mut sum = 0.0;
            for idx in 0..left_cols {
                sum += left[row][idx] * right[idx][col];
            }
            result[row][col] = sum;
        }
    }
    Ok(result)
}

fn inverse_square(matrix: &[Vec<f64>]) -> Result<Vec<Vec<f64>>, WorksheetErrorCode> {
    let n = matrix.len();
    if n == 0 || matrix.iter().any(|row| row.len() != n) {
        return Err(WorksheetErrorCode::Value);
    }

    let mut augmented = Vec::with_capacity(n);
    for (row_idx, row) in matrix.iter().enumerate() {
        let mut augmented_row = Vec::with_capacity(n * 2);
        augmented_row.extend_from_slice(row);
        for col in 0..n {
            augmented_row.push(if row_idx == col { 1.0 } else { 0.0 });
        }
        augmented.push(augmented_row);
    }

    const EPS: f64 = 1e-12;
    for pivot_idx in 0..n {
        let mut pivot_row = pivot_idx;
        let mut pivot_abs = augmented[pivot_idx][pivot_idx].abs();
        for row in (pivot_idx + 1)..n {
            let candidate = augmented[row][pivot_idx].abs();
            if candidate > pivot_abs {
                pivot_abs = candidate;
                pivot_row = row;
            }
        }

        if pivot_abs < EPS {
            return Err(WorksheetErrorCode::Num);
        }

        if pivot_row != pivot_idx {
            augmented.swap(pivot_row, pivot_idx);
        }

        let pivot = augmented[pivot_idx][pivot_idx];
        for col in 0..(n * 2) {
            augmented[pivot_idx][col] /= pivot;
        }

        for row in 0..n {
            if row == pivot_idx {
                continue;
            }
            let factor = augmented[row][pivot_idx];
            if factor == 0.0 {
                continue;
            }
            for col in 0..(n * 2) {
                augmented[row][col] -= factor * augmented[pivot_idx][col];
            }
        }
    }

    Ok(augmented
        .into_iter()
        .map(|row| row[n..].to_vec())
        .collect::<Vec<_>>())
}

fn solve_least_squares(x: &[Vec<f64>], y: &[f64]) -> Result<Vec<f64>, WorksheetErrorCode> {
    let xt = transpose(x);
    let y_column = y
        .iter()
        .copied()
        .map(|value| vec![value])
        .collect::<Vec<_>>();
    let xtx = mat_mul(&xt, x)?;
    let xty = mat_mul(&xt, &y_column)?;
    let inv = inverse_square(&xtx)?;
    let beta = mat_mul(&inv, &xty)?;
    Ok(beta.into_iter().map(|row| row[0]).collect())
}

fn linear_model_from_data(
    known_y: &NumericVector,
    known_x: &PreparedPredictorInput,
    use_const: bool,
) -> Result<LinearRegressionModel, WorksheetErrorCode> {
    let observation_count = known_y.values.len();
    if known_x.predictors.rows != observation_count {
        return Err(WorksheetErrorCode::Ref);
    }

    let mut design = Vec::with_capacity(observation_count);
    for row_idx in 0..observation_count {
        let mut row = known_x.predictors.row_values(row_idx).to_vec();
        if use_const {
            row.push(1.0);
        }
        design.push(row);
    }

    let beta = solve_least_squares(&design, &known_y.values)?;
    if use_const {
        let intercept = *beta.last().expect("const=true keeps intercept coefficient");
        Ok(LinearRegressionModel {
            coefficients: beta[..beta.len() - 1].to_vec(),
            intercept,
        })
    } else {
        Ok(LinearRegressionModel {
            coefficients: beta,
            intercept: 0.0,
        })
    }
}

fn exponential_model_from_data(
    known_y: &NumericVector,
    known_x: &PreparedPredictorInput,
    use_const: bool,
) -> Result<(Vec<f64>, f64), WorksheetErrorCode> {
    let mut log_y = Vec::with_capacity(known_y.values.len());
    for value in &known_y.values {
        if *value <= 0.0 || !value.is_finite() {
            return Err(WorksheetErrorCode::Num);
        }
        log_y.push(value.ln());
    }
    let logged = NumericVector {
        values: log_y,
        shape: known_y.shape,
    };
    let model = linear_model_from_data(&logged, known_x, use_const)?;
    Ok((
        model
            .coefficients
            .iter()
            .copied()
            .map(f64::exp)
            .collect::<Vec<_>>(),
        model.intercept.exp(),
    ))
}

fn eval_value_from_output_shape(
    values: &[f64],
    shape: OutputShape,
) -> Result<EvalValue, WorksheetErrorCode> {
    match shape {
        OutputShape::Scalar => Ok(EvalValue::Number(values[0])),
        OutputShape::Row(cols) => EvalArray::from_rows(vec![
            values
                .iter()
                .copied()
                .take(cols)
                .map(ArrayCellValue::Number)
                .collect::<Vec<_>>(),
        ])
        .map(EvalValue::Array)
        .ok_or(WorksheetErrorCode::Value),
        OutputShape::Column(rows) => EvalArray::from_rows(
            values
                .iter()
                .copied()
                .take(rows)
                .map(|value| vec![ArrayCellValue::Number(value)])
                .collect::<Vec<_>>(),
        )
        .map(EvalValue::Array)
        .ok_or(WorksheetErrorCode::Value),
        OutputShape::Matrix { rows, cols } => {
            let row_major = values
                .chunks(cols)
                .take(rows)
                .map(|chunk| chunk.iter().copied().map(ArrayCellValue::Number).collect())
                .collect::<Vec<Vec<ArrayCellValue>>>();
            EvalArray::from_rows(row_major)
                .map(EvalValue::Array)
                .ok_or(WorksheetErrorCode::Value)
        }
    }
}

fn row_array(values: &[f64]) -> EvalValue {
    EvalValue::Array(
        EvalArray::from_rows(vec![
            values.iter().copied().map(ArrayCellValue::Number).collect(),
        ])
        .expect("non-empty coefficient row"),
    )
}

fn trend_kernel(
    known_y: &NumericVector,
    known_x: &PreparedPredictorInput,
    new_x: &PreparedPredictorInput,
    use_const: bool,
) -> Result<EvalValue, WorksheetErrorCode> {
    let model = linear_model_from_data(known_y, known_x, use_const)?;
    if new_x.predictors.cols != model.coefficients.len() {
        return Err(WorksheetErrorCode::Ref);
    }
    let predicted = (0..new_x.predictors.rows)
        .map(|row_idx| {
            model
                .coefficients
                .iter()
                .copied()
                .zip(new_x.predictors.row_values(row_idx).iter().copied())
                .map(|(coef, x)| coef * x)
                .sum::<f64>()
                + model.intercept
        })
        .collect::<Vec<_>>();
    eval_value_from_output_shape(&predicted, new_x.output_shape)
}

fn growth_kernel(
    known_y: &NumericVector,
    known_x: &PreparedPredictorInput,
    new_x: &PreparedPredictorInput,
    use_const: bool,
) -> Result<EvalValue, WorksheetErrorCode> {
    let (factors, base) = exponential_model_from_data(known_y, known_x, use_const)?;
    if new_x.predictors.cols != factors.len() {
        return Err(WorksheetErrorCode::Ref);
    }
    let predicted = (0..new_x.predictors.rows)
        .map(|row_idx| {
            base * factors
                .iter()
                .copied()
                .zip(new_x.predictors.row_values(row_idx).iter().copied())
                .fold(1.0, |acc, (factor, x)| acc * factor.powf(x))
        })
        .collect::<Vec<_>>();
    eval_value_from_output_shape(&predicted, new_x.output_shape)
}

fn linest_kernel(
    known_y: &NumericVector,
    known_x: &PreparedPredictorInput,
    use_const: bool,
    stats: bool,
) -> Result<EvalValue, WorksheetErrorCode> {
    if stats {
        return Err(WorksheetErrorCode::Ref);
    }
    let model = linear_model_from_data(known_y, known_x, use_const)?;
    let mut row = model.coefficients.iter().rev().copied().collect::<Vec<_>>();
    row.push(if use_const { model.intercept } else { 0.0 });
    Ok(row_array(&row))
}

fn logest_kernel(
    known_y: &NumericVector,
    known_x: &PreparedPredictorInput,
    use_const: bool,
    stats: bool,
) -> Result<EvalValue, WorksheetErrorCode> {
    if stats {
        return Err(WorksheetErrorCode::Ref);
    }
    let (factors, base) = exponential_model_from_data(known_y, known_x, use_const)?;
    let mut row = factors.iter().rev().copied().collect::<Vec<_>>();
    row.push(if use_const { base } else { 1.0 });
    Ok(row_array(&row))
}

fn forecast_pair_kernel(
    x: f64,
    known_y: &NumericVector,
    known_x: &NumericVector,
) -> Result<EvalValue, WorksheetErrorCode> {
    if known_x.values.len() != known_y.values.len() {
        return Err(WorksheetErrorCode::NA);
    }
    let known_x_prepared = predictor_input_from_optional(
        Some(NumericMatrix {
            rows: known_x.values.len(),
            cols: 1,
            values: known_x.values.clone(),
        }),
        known_y,
    )
    .map_err(|error| map_regression_forecast_error_to_ws(&error))?;
    let new_x = PreparedPredictorInput {
        predictors: NumericMatrix {
            rows: 1,
            cols: 1,
            values: vec![x],
        },
        output_shape: OutputShape::Scalar,
    };
    trend_kernel(known_y, &known_x_prepared, &new_x, true)
}

pub fn eval_trend_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, RegressionForecastEvalError> {
    if !TREND_META.arity.accepts(args.len()) {
        return Err(arity_error(&TREND_META, args.len()));
    }
    let known_y = collect_numeric_vector_arg(&args[0], resolver)?;
    let known_x_raw = args
        .get(1)
        .filter(|arg| !matches!(arg, CallArgValue::MissingArg))
        .map(|arg| collect_numeric_matrix_arg(arg, resolver))
        .transpose()?;
    let known_x = predictor_input_from_optional(known_x_raw, &known_y)?;
    let new_x_raw = args
        .get(2)
        .filter(|arg| !matches!(arg, CallArgValue::MissingArg))
        .map(|arg| collect_numeric_matrix_arg(arg, resolver))
        .transpose()?;
    let new_x = prediction_input_from_optional(new_x_raw, &known_y, &known_x)?;
    let use_const =
        bool_flag_from_eval_or_default(optional_arg_value(args.get(3), resolver)?, true)?;
    trend_kernel(&known_y, &known_x, &new_x, use_const).map_err(RegressionForecastEvalError::Domain)
}

pub fn eval_growth_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, RegressionForecastEvalError> {
    if !GROWTH_META.arity.accepts(args.len()) {
        return Err(arity_error(&GROWTH_META, args.len()));
    }
    let known_y = collect_numeric_vector_arg(&args[0], resolver)?;
    let known_x_raw = args
        .get(1)
        .filter(|arg| !matches!(arg, CallArgValue::MissingArg))
        .map(|arg| collect_numeric_matrix_arg(arg, resolver))
        .transpose()?;
    let known_x = predictor_input_from_optional(known_x_raw, &known_y)?;
    let new_x_raw = args
        .get(2)
        .filter(|arg| !matches!(arg, CallArgValue::MissingArg))
        .map(|arg| collect_numeric_matrix_arg(arg, resolver))
        .transpose()?;
    let new_x = prediction_input_from_optional(new_x_raw, &known_y, &known_x)?;
    let use_const =
        bool_flag_from_eval_or_default(optional_arg_value(args.get(3), resolver)?, true)?;
    growth_kernel(&known_y, &known_x, &new_x, use_const)
        .map_err(RegressionForecastEvalError::Domain)
}

pub fn eval_forecast_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, RegressionForecastEvalError> {
    if !FORECAST_META.arity.accepts(args.len()) {
        return Err(arity_error(&FORECAST_META, args.len()));
    }
    let x = numeric_scalar_from_eval(&resolve_arg_eval(&args[0], resolver)?)?;
    let known_y = collect_numeric_vector_arg(&args[1], resolver)?;
    let known_x = collect_numeric_vector_arg(&args[2], resolver)?;
    forecast_pair_kernel(x, &known_y, &known_x).map_err(RegressionForecastEvalError::Domain)
}

pub fn eval_forecast_linear_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, RegressionForecastEvalError> {
    if !FORECAST_LINEAR_META.arity.accepts(args.len()) {
        return Err(arity_error(&FORECAST_LINEAR_META, args.len()));
    }
    let x = numeric_scalar_from_eval(&resolve_arg_eval(&args[0], resolver)?)?;
    let known_y = collect_numeric_vector_arg(&args[1], resolver)?;
    let known_x = collect_numeric_vector_arg(&args[2], resolver)?;
    forecast_pair_kernel(x, &known_y, &known_x).map_err(RegressionForecastEvalError::Domain)
}

pub fn eval_linest_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, RegressionForecastEvalError> {
    if !LINEST_META.arity.accepts(args.len()) {
        return Err(arity_error(&LINEST_META, args.len()));
    }
    let known_y = collect_numeric_vector_arg(&args[0], resolver)?;
    let known_x_raw = args
        .get(1)
        .filter(|arg| !matches!(arg, CallArgValue::MissingArg))
        .map(|arg| collect_numeric_matrix_arg(arg, resolver))
        .transpose()?;
    let known_x = predictor_input_from_optional(known_x_raw, &known_y)?;
    let use_const =
        bool_flag_from_eval_or_default(optional_arg_value(args.get(2), resolver)?, true)?;
    let stats = bool_flag_from_eval_or_default(optional_arg_value(args.get(3), resolver)?, false)?;
    linest_kernel(&known_y, &known_x, use_const, stats).map_err(RegressionForecastEvalError::Domain)
}

pub fn eval_logest_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, RegressionForecastEvalError> {
    if !LOGEST_META.arity.accepts(args.len()) {
        return Err(arity_error(&LOGEST_META, args.len()));
    }
    let known_y = collect_numeric_vector_arg(&args[0], resolver)?;
    let known_x_raw = args
        .get(1)
        .filter(|arg| !matches!(arg, CallArgValue::MissingArg))
        .map(|arg| collect_numeric_matrix_arg(arg, resolver))
        .transpose()?;
    let known_x = predictor_input_from_optional(known_x_raw, &known_y)?;
    let use_const =
        bool_flag_from_eval_or_default(optional_arg_value(args.get(2), resolver)?, true)?;
    let stats = bool_flag_from_eval_or_default(optional_arg_value(args.get(3), resolver)?, false)?;
    logest_kernel(&known_y, &known_x, use_const, stats).map_err(RegressionForecastEvalError::Domain)
}

pub fn map_regression_forecast_error_to_ws(
    error: &RegressionForecastEvalError,
) -> WorksheetErrorCode {
    match error {
        RegressionForecastEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        RegressionForecastEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        RegressionForecastEvalError::Coercion(_) => WorksheetErrorCode::Value,
        RegressionForecastEvalError::Domain(code) => *code,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::{CallerContext, RefResolutionError, ResolverCapabilities};
    use crate::value::{ArrayShape, ReferenceKind, ReferenceLike};
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

        fn caller_context(&self) -> Option<CallerContext> {
            None
        }
    }

    fn num(n: f64) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Number(n))
    }

    fn bool_arg(flag: bool) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Logical(flag))
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

    fn matrix(rows: &[&[f64]]) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Array(
            EvalArray::from_rows(
                rows.iter()
                    .map(|row| row.iter().copied().map(ArrayCellValue::Number).collect())
                    .collect(),
            )
            .unwrap(),
        ))
    }

    fn ref_arg(target: &str) -> CallArgValue {
        CallArgValue::Reference(ReferenceLike {
            kind: ReferenceKind::Area,
            target: target.to_string(),
        })
    }

    fn assert_close(left: f64, right: f64) {
        assert!((left - right).abs() < 1e-9, "left={left}, right={right}");
    }

    fn expect_number(value: EvalValue) -> f64 {
        match value {
            EvalValue::Number(n) => n,
            other => panic!("expected scalar, got {other:?}"),
        }
    }

    fn expect_row_array(value: EvalValue) -> Vec<f64> {
        match value {
            EvalValue::Array(array) => {
                assert_eq!(
                    array.shape(),
                    ArrayShape {
                        rows: 1,
                        cols: array.shape().cols
                    }
                );
                array
                    .iter_row_major()
                    .map(|cell| match cell {
                        ArrayCellValue::Number(n) => *n,
                        other => panic!("unexpected cell: {other:?}"),
                    })
                    .collect()
            }
            other => panic!("expected array, got {other:?}"),
        }
    }

    #[test]
    fn metadata_matches_admitted_family_shape() {
        assert_eq!(TREND_META.function_id, "FUNC.TREND");
        assert_eq!(GROWTH_META.arity, Arity { min: 1, max: 4 });
        assert_eq!(
            LINEST_META.arg_preparation_profile,
            ArgPreparationProfile::RefsVisibleInAdapter
        );
        assert_eq!(
            LOGEST_META.surface_fec_dependency_profile,
            FecDependencyProfile::RefOnly
        );
    }

    #[test]
    fn trend_defaults_known_x_to_sequence_and_preserves_row_shape() {
        let got = eval_trend_surface(
            &[row(&[2.0, 4.0, 6.0])],
            &MockResolver {
                map: BTreeMap::new(),
            },
        )
        .unwrap();
        let values = expect_row_array(got);
        assert_eq!(values.len(), 3);
        assert_close(values[0], 2.0);
        assert_close(values[1], 4.0);
        assert_close(values[2], 6.0);
    }

    #[test]
    fn trend_projects_column_vector() {
        let got = eval_trend_surface(
            &[
                col(&[2.0, 4.0, 6.0]),
                col(&[1.0, 2.0, 3.0]),
                col(&[4.0, 5.0]),
            ],
            &MockResolver {
                map: BTreeMap::new(),
            },
        )
        .unwrap();
        match got {
            EvalValue::Array(array) => {
                assert_eq!(array.shape(), ArrayShape { rows: 2, cols: 1 });
                assert_close(numeric_from_cell(array.get(0, 0).unwrap()).unwrap(), 8.0);
                assert_close(numeric_from_cell(array.get(1, 0).unwrap()).unwrap(), 10.0);
            }
            other => panic!("expected array, got {other:?}"),
        }
    }

    #[test]
    fn growth_projects_exponential_sequence() {
        let got = eval_growth_surface(
            &[
                col(&[2.0, 4.0, 8.0]),
                CallArgValue::MissingArg,
                col(&[4.0, 5.0]),
            ],
            &MockResolver {
                map: BTreeMap::new(),
            },
        )
        .unwrap();
        match got {
            EvalValue::Array(array) => {
                assert_close(numeric_from_cell(array.get(0, 0).unwrap()).unwrap(), 16.0);
                assert_close(numeric_from_cell(array.get(1, 0).unwrap()).unwrap(), 32.0);
            }
            other => panic!("expected array, got {other:?}"),
        }
    }

    #[test]
    fn growth_rejects_nonpositive_known_y() {
        let error = eval_growth_surface(
            &[col(&[2.0, 0.0, 8.0])],
            &MockResolver {
                map: BTreeMap::new(),
            },
        )
        .unwrap_err();
        assert_eq!(
            map_regression_forecast_error_to_ws(&error),
            WorksheetErrorCode::Num
        );
    }

    #[test]
    fn growth_projects_multivariate_rows_to_column_result() {
        let got = eval_growth_surface(
            &[
                col(&[9.0, 18.0, 27.0, 54.0, 72.0]),
                matrix(&[
                    &[1.0, 1.0],
                    &[2.0, 1.0],
                    &[1.0, 2.0],
                    &[2.0, 2.0],
                    &[3.0, 2.0],
                ]),
                matrix(&[&[4.0, 1.0], &[4.0, 2.0]]),
            ],
            &MockResolver {
                map: BTreeMap::new(),
            },
        )
        .unwrap();
        match got {
            EvalValue::Array(array) => {
                assert_eq!(array.shape(), ArrayShape { rows: 2, cols: 1 });
                assert_close(numeric_from_cell(array.get(0, 0).unwrap()).unwrap(), 48.0);
                assert_close(
                    numeric_from_cell(array.get(1, 0).unwrap()).unwrap(),
                    136.421762958185,
                );
            }
            other => panic!("expected array, got {other:?}"),
        }
    }

    #[test]
    fn forecast_and_forecast_linear_match_seed_lanes() {
        let forecast = eval_forecast_surface(
            &[num(3.0), row(&[1.0, 2.0]), row(&[2.0, 4.0])],
            &MockResolver {
                map: BTreeMap::new(),
            },
        )
        .unwrap();
        let forecast_linear = eval_forecast_linear_surface(
            &[num(3.0), row(&[1.0, 2.0]), row(&[2.0, 4.0])],
            &MockResolver {
                map: BTreeMap::new(),
            },
        )
        .unwrap();
        assert_close(expect_number(forecast), 1.5);
        assert_close(expect_number(forecast_linear), 1.5);
    }

    #[test]
    fn forecast_length_mismatch_returns_na() {
        let error = eval_forecast_surface(
            &[num(3.0), row(&[1.0, 2.0, 3.0]), row(&[2.0, 4.0])],
            &MockResolver {
                map: BTreeMap::new(),
            },
        )
        .unwrap_err();
        assert_eq!(
            map_regression_forecast_error_to_ws(&error),
            WorksheetErrorCode::NA
        );
    }

    #[test]
    fn linest_returns_single_predictor_coefficients() {
        let got = eval_linest_surface(
            &[col(&[3.0, 5.0, 7.0]), col(&[1.0, 2.0, 3.0])],
            &MockResolver {
                map: BTreeMap::new(),
            },
        )
        .unwrap();
        let coeffs = expect_row_array(got);
        assert_eq!(coeffs.len(), 2);
        assert_close(coeffs[0], 2.0);
        assert_close(coeffs[1], 1.0);
    }

    #[test]
    fn linest_without_const_keeps_trailing_zero_intercept_cell() {
        let got = eval_linest_surface(
            &[
                col(&[2.0, 4.0, 6.0]),
                col(&[1.0, 2.0, 3.0]),
                bool_arg(false),
            ],
            &MockResolver {
                map: BTreeMap::new(),
            },
        )
        .unwrap();
        let coeffs = expect_row_array(got);
        assert_eq!(coeffs, vec![2.0, 0.0]);
    }

    #[test]
    fn linest_returns_reversed_multivariate_coefficients() {
        let got = eval_linest_surface(
            &[
                col(&[6.0, 8.0, 9.0, 11.0, 10.0]),
                matrix(&[
                    &[1.0, 1.0],
                    &[2.0, 1.0],
                    &[1.0, 2.0],
                    &[2.0, 2.0],
                    &[3.0, 1.0],
                ]),
            ],
            &MockResolver {
                map: BTreeMap::new(),
            },
        )
        .unwrap();
        let coeffs = expect_row_array(got);
        assert_close(coeffs[0], 3.0);
        assert_close(coeffs[1], 2.0);
        assert_close(coeffs[2], 1.0);
    }

    #[test]
    fn logest_returns_growth_factor_and_base() {
        let got = eval_logest_surface(
            &[col(&[3.0, 6.0, 12.0]), col(&[1.0, 2.0, 3.0])],
            &MockResolver {
                map: BTreeMap::new(),
            },
        )
        .unwrap();
        let coeffs = expect_row_array(got);
        assert_eq!(coeffs.len(), 2);
        assert_close(coeffs[0], 2.0);
        assert_close(coeffs[1], 1.5);
    }

    #[test]
    fn logest_without_const_keeps_trailing_one_base_cell() {
        let got = eval_logest_surface(
            &[
                col(&[3.0, 6.0, 12.0]),
                col(&[1.0, 2.0, 3.0]),
                bool_arg(false),
            ],
            &MockResolver {
                map: BTreeMap::new(),
            },
        )
        .unwrap();
        let coeffs = expect_row_array(got);
        assert_close(coeffs[0], 2.37956557896877);
        assert_close(coeffs[1], 1.0);
    }

    #[test]
    fn logest_returns_reversed_multivariate_factors() {
        let got = eval_logest_surface(
            &[
                col(&[9.0, 18.0, 27.0, 54.0, 72.0]),
                matrix(&[
                    &[1.0, 1.0],
                    &[2.0, 1.0],
                    &[1.0, 2.0],
                    &[2.0, 2.0],
                    &[3.0, 2.0],
                ]),
            ],
            &MockResolver {
                map: BTreeMap::new(),
            },
        )
        .unwrap();
        let coeffs = expect_row_array(got);
        assert_close(coeffs[0], 2.84212006162885);
        assert_close(coeffs[1], 1.70056600083439);
        assert_close(coeffs[2], 2.01941161117829);
    }

    #[test]
    fn stats_true_is_explicitly_out_of_slice() {
        let error = eval_logest_surface(
            &[
                col(&[3.0, 6.0, 12.0]),
                col(&[1.0, 2.0, 3.0]),
                bool_arg(true),
                bool_arg(true),
            ],
            &MockResolver {
                map: BTreeMap::new(),
            },
        )
        .unwrap_err();
        assert_eq!(
            map_regression_forecast_error_to_ws(&error),
            WorksheetErrorCode::Ref
        );
    }

    #[test]
    fn multivariate_new_x_requires_matching_column_count() {
        let error = eval_trend_surface(
            &[
                col(&[3.0, 5.0, 7.0]),
                matrix(&[&[1.0, 2.0], &[2.0, 4.0], &[3.0, 6.0]]),
                col(&[4.0, 5.0]),
            ],
            &MockResolver {
                map: BTreeMap::new(),
            },
        )
        .unwrap_err();
        assert_eq!(
            map_regression_forecast_error_to_ws(&error),
            WorksheetErrorCode::Ref
        );
    }

    #[test]
    fn surface_eval_resolves_reference_vectors() {
        let mut map = BTreeMap::new();
        map.insert(
            "A1:A3".to_string(),
            EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Number(1.0)],
                    vec![ArrayCellValue::Number(2.0)],
                    vec![ArrayCellValue::Number(3.0)],
                ])
                .unwrap(),
            ),
        );
        map.insert(
            "B1:B3".to_string(),
            EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Number(2.0)],
                    vec![ArrayCellValue::Number(4.0)],
                    vec![ArrayCellValue::Number(6.0)],
                ])
                .unwrap(),
            ),
        );
        let got = eval_linest_surface(&[ref_arg("B1:B3"), ref_arg("A1:A3")], &MockResolver { map })
            .unwrap();
        let coeffs = expect_row_array(got);
        assert_close(coeffs[0], 2.0);
        assert_close(coeffs[1], 0.0);
    }

    #[test]
    fn arity_errors_map_to_value() {
        let error = eval_growth_surface(
            &[],
            &MockResolver {
                map: BTreeMap::new(),
            },
        )
        .unwrap_err();
        assert_eq!(
            map_regression_forecast_error_to_ws(&error),
            WorksheetErrorCode::Value
        );
    }

    #[test]
    fn scalar_new_x_returns_scalar_trend_result() {
        let got = eval_trend_surface(
            &[col(&[2.0, 4.0, 6.0]), col(&[1.0, 2.0, 3.0]), num(4.0)],
            &MockResolver {
                map: BTreeMap::new(),
            },
        )
        .unwrap();
        assert_close(expect_number(got), 8.0);
    }
    #[test]
    fn trend_projects_matrix_shape_for_single_predictor_new_x() {
        let got = eval_trend_surface(
            &[
                col(&[2.0, 4.0, 6.0]),
                col(&[1.0, 2.0, 3.0]),
                matrix(&[&[1.0, 2.0], &[3.0, 4.0]]),
            ],
            &MockResolver {
                map: BTreeMap::new(),
            },
        )
        .unwrap();
        match got {
            EvalValue::Array(array) => {
                assert_eq!(array.shape(), ArrayShape { rows: 2, cols: 2 });
                assert_close(numeric_from_cell(array.get(0, 0).unwrap()).unwrap(), 2.0);
                assert_close(numeric_from_cell(array.get(0, 1).unwrap()).unwrap(), 4.0);
                assert_close(numeric_from_cell(array.get(1, 0).unwrap()).unwrap(), 6.0);
                assert_close(numeric_from_cell(array.get(1, 1).unwrap()).unwrap(), 8.0);
            }
            other => panic!("expected array, got {other:?}"),
        }
    }

    #[test]
    fn trend_projects_multivariate_rows_to_column_result() {
        let got = eval_trend_surface(
            &[
                col(&[6.0, 8.0, 9.0, 11.0, 10.0]),
                matrix(&[
                    &[1.0, 1.0],
                    &[2.0, 1.0],
                    &[1.0, 2.0],
                    &[2.0, 2.0],
                    &[3.0, 1.0],
                ]),
                matrix(&[&[4.0, 1.0], &[4.0, 2.0]]),
            ],
            &MockResolver {
                map: BTreeMap::new(),
            },
        )
        .unwrap();
        match got {
            EvalValue::Array(array) => {
                assert_eq!(array.shape(), ArrayShape { rows: 2, cols: 1 });
                assert_close(numeric_from_cell(array.get(0, 0).unwrap()).unwrap(), 12.0);
                assert_close(numeric_from_cell(array.get(1, 0).unwrap()).unwrap(), 15.0);
            }
            other => panic!("expected array, got {other:?}"),
        }
    }
}

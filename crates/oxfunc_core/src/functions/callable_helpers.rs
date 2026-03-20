use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{PreparedArgValue, prepare_args_values_only};
use crate::resolver::ReferenceResolver;
use crate::value::{
    ArrayCellValue, ArrayShape, CallArgValue, EvalArray, EvalValue, LambdaValue, WorksheetErrorCode,
};

const FUNCTIONAL_LAMBDA_BASE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.FUNCTIONAL_LAMBDA_BASE",
    arity: Arity { min: 2, max: 255 },
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

pub const MAP_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.MAP",
    arity: Arity { min: 2, max: 255 },
    ..FUNCTIONAL_LAMBDA_BASE_META
};

pub const REDUCE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.REDUCE",
    arity: Arity { min: 3, max: 3 },
    ..FUNCTIONAL_LAMBDA_BASE_META
};

pub const SCAN_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.SCAN",
    arity: Arity { min: 3, max: 3 },
    ..FUNCTIONAL_LAMBDA_BASE_META
};

pub const BYROW_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.BYROW",
    arity: Arity { min: 2, max: 2 },
    ..FUNCTIONAL_LAMBDA_BASE_META
};

pub const BYCOL_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.BYCOL",
    arity: Arity { min: 2, max: 2 },
    ..FUNCTIONAL_LAMBDA_BASE_META
};

pub const MAKEARRAY_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.MAKEARRAY",
    arity: Arity { min: 3, max: 3 },
    ..FUNCTIONAL_LAMBDA_BASE_META
};

pub const ISOMITTED_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.ISOMITTED",
    arity: Arity { min: 1, max: 1 },
    ..FUNCTIONAL_LAMBDA_BASE_META
};

#[derive(Debug, Clone, PartialEq)]
pub enum CallableInvocationError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    UnsupportedCallableToken(String),
    UnsupportedResultKind(&'static str),
    Worksheet(WorksheetErrorCode),
}

#[derive(Debug, Clone, PartialEq)]
pub enum LambdaHelperEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Invocation(CallableInvocationError),
    Preparation(crate::coercion::CoercionError),
    MissingCallable,
    NonScalarHelperResult,
    InvalidGeneratedDimensions,
}

pub trait CallableInvoker {
    fn invoke(
        &self,
        callable: &LambdaValue,
        args: &[PreparedArgValue],
    ) -> Result<PreparedArgValue, CallableInvocationError>;
}

pub fn invoke_callable_prepared(
    callable: &LambdaValue,
    args: &[PreparedArgValue],
    invoker: &(impl CallableInvoker + ?Sized),
) -> Result<PreparedArgValue, CallableInvocationError> {
    let argc = args.len();
    if !callable.arity_shape.accepts(argc) {
        return Err(CallableInvocationError::ArityMismatch {
            expected_min: callable.arity_shape.min,
            expected_max: callable.arity_shape.max,
            actual: argc,
        });
    }
    invoker.invoke(callable, args)
}

fn prepared_from_array_cell(cell: &ArrayCellValue) -> PreparedArgValue {
    match cell {
        ArrayCellValue::Number(n) => PreparedArgValue::Eval(EvalValue::Number(*n)),
        ArrayCellValue::Text(t) => PreparedArgValue::Eval(EvalValue::Text(t.clone())),
        ArrayCellValue::Logical(b) => PreparedArgValue::Eval(EvalValue::Logical(*b)),
        ArrayCellValue::Error(code) => PreparedArgValue::Eval(EvalValue::Error(*code)),
        ArrayCellValue::EmptyCell => PreparedArgValue::EmptyCell,
    }
}

fn materialize_iterable(prepared: &PreparedArgValue) -> Vec<PreparedArgValue> {
    match prepared {
        PreparedArgValue::Eval(EvalValue::Array(array)) => array
            .iter_row_major()
            .map(prepared_from_array_cell)
            .collect(),
        other => vec![other.clone()],
    }
}

fn scalar_cell_from_prepared(
    prepared: &PreparedArgValue,
) -> Result<ArrayCellValue, CallableInvocationError> {
    match prepared {
        PreparedArgValue::Eval(EvalValue::Number(n)) => Ok(ArrayCellValue::Number(*n)),
        PreparedArgValue::Eval(EvalValue::Text(t)) => Ok(ArrayCellValue::Text(t.clone())),
        PreparedArgValue::Eval(EvalValue::Logical(b)) => Ok(ArrayCellValue::Logical(*b)),
        PreparedArgValue::Eval(EvalValue::Error(code)) => Ok(ArrayCellValue::Error(*code)),
        PreparedArgValue::MissingArg | PreparedArgValue::EmptyCell => Ok(ArrayCellValue::EmptyCell),
        PreparedArgValue::Eval(EvalValue::Array(_)) => {
            Err(CallableInvocationError::UnsupportedResultKind("array"))
        }
        PreparedArgValue::Eval(EvalValue::Reference(_)) => Err(
            CallableInvocationError::UnsupportedResultKind("reference_like"),
        ),
        PreparedArgValue::Eval(EvalValue::Lambda(_)) => Err(
            CallableInvocationError::UnsupportedResultKind("lambda_value"),
        ),
    }
}

fn row_vector_from_slice(row: &[ArrayCellValue]) -> PreparedArgValue {
    PreparedArgValue::Eval(EvalValue::Array(
        EvalArray::from_rows(vec![row.to_vec()]).expect("row slice is non-empty"),
    ))
}

fn column_vector_from_array(array: &EvalArray, col: usize) -> PreparedArgValue {
    let cells = (0..array.shape().rows)
        .map(|row| {
            vec![
                array
                    .get(row, col)
                    .cloned()
                    .expect("validated column access"),
            ]
        })
        .collect::<Vec<_>>();
    PreparedArgValue::Eval(EvalValue::Array(
        EvalArray::from_rows(cells).expect("column slice dimensions are valid"),
    ))
}

fn inferred_map_output_shape(
    inputs: &[PreparedArgValue],
    cell_count: usize,
) -> Result<ArrayShape, CallableInvocationError> {
    if let Some(shape) = inputs.iter().find_map(|arg| match arg {
        PreparedArgValue::Eval(EvalValue::Array(array)) => Some(array.shape()),
        _ => None,
    }) {
        if shape.cell_count() == cell_count {
            return Ok(shape);
        }
    }

    ArrayShape {
        rows: 1,
        cols: cell_count.max(1),
    }
    .cell_count()
    .checked_sub(0)
    .map(|_| ArrayShape {
        rows: 1,
        cols: cell_count.max(1),
    })
    .ok_or(CallableInvocationError::UnsupportedResultKind(
        "map_output_shape",
    ))
}

pub fn eval_map_prepared(
    inputs: &[PreparedArgValue],
    callable: &LambdaValue,
    invoker: &(impl CallableInvoker + ?Sized),
) -> Result<EvalValue, LambdaHelperEvalError> {
    if inputs.is_empty() {
        return Err(LambdaHelperEvalError::MissingCallable);
    }

    let materialized: Vec<Vec<PreparedArgValue>> =
        inputs.iter().map(materialize_iterable).collect();
    let cell_count = materialized.iter().map(Vec::len).max().unwrap_or(1);
    let output_shape =
        inferred_map_output_shape(inputs, cell_count).map_err(LambdaHelperEvalError::Invocation)?;

    let mut cells = Vec::with_capacity(cell_count);
    for index in 0..cell_count {
        let lambda_args = materialized
            .iter()
            .map(|values| {
                values.get(index).cloned().unwrap_or_else(|| {
                    PreparedArgValue::Eval(EvalValue::Error(WorksheetErrorCode::NA))
                })
            })
            .collect::<Vec<_>>();
        let result = invoke_callable_prepared(callable, &lambda_args, invoker)
            .map_err(LambdaHelperEvalError::Invocation)?;
        cells.push(scalar_cell_from_prepared(&result).map_err(LambdaHelperEvalError::Invocation)?);
    }

    Ok(EvalValue::Array(
        EvalArray::new(output_shape, cells).expect("map output shape is validated"),
    ))
}

pub fn eval_reduce_prepared(
    initial: &PreparedArgValue,
    iterable: &PreparedArgValue,
    callable: &LambdaValue,
    invoker: &(impl CallableInvoker + ?Sized),
) -> Result<PreparedArgValue, LambdaHelperEvalError> {
    let mut accumulator = initial.clone();
    for item in materialize_iterable(iterable) {
        accumulator = invoke_callable_prepared(callable, &[accumulator, item], invoker)
            .map_err(LambdaHelperEvalError::Invocation)?;
    }
    Ok(accumulator)
}

pub fn eval_scan_prepared(
    initial: &PreparedArgValue,
    iterable: &PreparedArgValue,
    callable: &LambdaValue,
    invoker: &(impl CallableInvoker + ?Sized),
) -> Result<EvalValue, LambdaHelperEvalError> {
    let values = materialize_iterable(iterable);
    let shape = match iterable {
        PreparedArgValue::Eval(EvalValue::Array(array)) => array.shape(),
        _ => ArrayShape {
            rows: 1,
            cols: values.len().max(1),
        },
    };

    let mut accumulator = initial.clone();
    let mut cells = Vec::with_capacity(values.len());
    for item in values {
        accumulator = invoke_callable_prepared(callable, &[accumulator, item], invoker)
            .map_err(LambdaHelperEvalError::Invocation)?;
        cells.push(
            scalar_cell_from_prepared(&accumulator).map_err(LambdaHelperEvalError::Invocation)?,
        );
    }

    Ok(EvalValue::Array(
        EvalArray::new(shape, cells).expect("scan output shape is validated"),
    ))
}

pub fn eval_byrow_prepared(
    source: &PreparedArgValue,
    callable: &LambdaValue,
    invoker: &(impl CallableInvoker + ?Sized),
) -> Result<EvalValue, LambdaHelperEvalError> {
    let source_array = match source {
        PreparedArgValue::Eval(EvalValue::Array(array)) => array.clone(),
        other => EvalArray::from_scalar(
            scalar_cell_from_prepared(other).map_err(LambdaHelperEvalError::Invocation)?,
        ),
    };

    let mut cells = Vec::with_capacity(source_array.shape().rows);
    for row in 0..source_array.shape().rows {
        let row_arg = row_vector_from_slice(
            source_array
                .row_slice(row)
                .expect("validated row access for byrow"),
        );
        let result = invoke_callable_prepared(callable, &[row_arg], invoker)
            .map_err(LambdaHelperEvalError::Invocation)?;
        match scalar_cell_from_prepared(&result) {
            Ok(cell) => cells.push(cell),
            Err(CallableInvocationError::UnsupportedResultKind("array")) => {
                return Err(LambdaHelperEvalError::NonScalarHelperResult);
            }
            Err(other) => return Err(LambdaHelperEvalError::Invocation(other)),
        }
    }

    Ok(EvalValue::Array(
        EvalArray::new(
            ArrayShape {
                rows: source_array.shape().rows,
                cols: 1,
            },
            cells,
        )
        .expect("byrow output shape is valid"),
    ))
}

pub fn eval_bycol_prepared(
    source: &PreparedArgValue,
    callable: &LambdaValue,
    invoker: &(impl CallableInvoker + ?Sized),
) -> Result<EvalValue, LambdaHelperEvalError> {
    let source_array = match source {
        PreparedArgValue::Eval(EvalValue::Array(array)) => array.clone(),
        other => EvalArray::from_scalar(
            scalar_cell_from_prepared(other).map_err(LambdaHelperEvalError::Invocation)?,
        ),
    };

    let mut cells = Vec::with_capacity(source_array.shape().cols);
    for col in 0..source_array.shape().cols {
        let col_arg = column_vector_from_array(&source_array, col);
        let result = invoke_callable_prepared(callable, &[col_arg], invoker)
            .map_err(LambdaHelperEvalError::Invocation)?;
        match scalar_cell_from_prepared(&result) {
            Ok(cell) => cells.push(cell),
            Err(CallableInvocationError::UnsupportedResultKind("array")) => {
                return Err(LambdaHelperEvalError::NonScalarHelperResult);
            }
            Err(other) => return Err(LambdaHelperEvalError::Invocation(other)),
        }
    }

    Ok(EvalValue::Array(
        EvalArray::new(
            ArrayShape {
                rows: 1,
                cols: source_array.shape().cols,
            },
            cells,
        )
        .expect("bycol output shape is valid"),
    ))
}

pub fn eval_makearray_prepared(
    rows: usize,
    cols: usize,
    callable: &LambdaValue,
    invoker: &(impl CallableInvoker + ?Sized),
) -> Result<EvalValue, LambdaHelperEvalError> {
    if rows == 0 || cols == 0 {
        return Err(LambdaHelperEvalError::InvalidGeneratedDimensions);
    }

    let mut cells = Vec::with_capacity(rows * cols);
    for row in 0..rows {
        for col in 0..cols {
            let result = invoke_callable_prepared(
                callable,
                &[
                    PreparedArgValue::Eval(EvalValue::Number((row + 1) as f64)),
                    PreparedArgValue::Eval(EvalValue::Number((col + 1) as f64)),
                ],
                invoker,
            )
            .map_err(LambdaHelperEvalError::Invocation)?;
            cells.push(
                scalar_cell_from_prepared(&result).map_err(LambdaHelperEvalError::Invocation)?,
            );
        }
    }

    Ok(EvalValue::Array(
        EvalArray::new(ArrayShape { rows, cols }, cells).expect("makearray output shape is valid"),
    ))
}

pub fn prepare_and_invoke_callable(
    args: &[CallArgValue],
    callable: &LambdaValue,
    resolver: &impl ReferenceResolver,
    invoker: &(impl CallableInvoker + ?Sized),
) -> Result<PreparedArgValue, LambdaHelperEvalError> {
    let prepared =
        prepare_args_values_only(args, resolver).map_err(LambdaHelperEvalError::Preparation)?;
    invoke_callable_prepared(callable, &prepared, invoker)
        .map_err(LambdaHelperEvalError::Invocation)
}

fn require_callable(prepared: &PreparedArgValue) -> Result<&LambdaValue, LambdaHelperEvalError> {
    match prepared {
        PreparedArgValue::Eval(EvalValue::Lambda(callable)) => Ok(callable),
        PreparedArgValue::Eval(EvalValue::Error(code)) => Err(LambdaHelperEvalError::Invocation(
            CallableInvocationError::Worksheet(*code),
        )),
        _ => Err(LambdaHelperEvalError::Invocation(
            CallableInvocationError::Worksheet(WorksheetErrorCode::Value),
        )),
    }
}

fn surface_arity_error(meta: &FunctionMeta, actual: usize) -> LambdaHelperEvalError {
    LambdaHelperEvalError::ArityMismatch {
        expected_min: meta.arity.min,
        expected_max: meta.arity.max,
        actual,
    }
}

fn parse_positive_dimension(prepared: &PreparedArgValue) -> Result<usize, LambdaHelperEvalError> {
    let raw = crate::functions::adapters::coerce_prepared_to_number(prepared)
        .map_err(LambdaHelperEvalError::Preparation)?;
    if !raw.is_finite() || raw < 1.0 {
        return Err(LambdaHelperEvalError::InvalidGeneratedDimensions);
    }
    let truncated = raw.trunc();
    if (truncated - raw).abs() > f64::EPSILON {
        return Err(LambdaHelperEvalError::InvalidGeneratedDimensions);
    }
    Ok(truncated as usize)
}

pub fn eval_map_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
    invoker: &(impl CallableInvoker + ?Sized),
) -> Result<EvalValue, LambdaHelperEvalError> {
    if !MAP_META.arity.accepts(args.len()) {
        return Err(surface_arity_error(&MAP_META, args.len()));
    }
    let prepared =
        prepare_args_values_only(args, resolver).map_err(LambdaHelperEvalError::Preparation)?;
    let (input_args, callable_arg) = prepared.split_at(prepared.len() - 1);
    let callable = require_callable(&callable_arg[0])?;
    eval_map_prepared(input_args, callable, invoker)
}

pub fn eval_reduce_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
    invoker: &(impl CallableInvoker + ?Sized),
) -> Result<PreparedArgValue, LambdaHelperEvalError> {
    if !REDUCE_META.arity.accepts(args.len()) {
        return Err(surface_arity_error(&REDUCE_META, args.len()));
    }
    let prepared =
        prepare_args_values_only(args, resolver).map_err(LambdaHelperEvalError::Preparation)?;
    let callable = require_callable(&prepared[2])?;
    eval_reduce_prepared(&prepared[0], &prepared[1], callable, invoker)
}

pub fn eval_scan_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
    invoker: &(impl CallableInvoker + ?Sized),
) -> Result<EvalValue, LambdaHelperEvalError> {
    if !SCAN_META.arity.accepts(args.len()) {
        return Err(surface_arity_error(&SCAN_META, args.len()));
    }
    let prepared =
        prepare_args_values_only(args, resolver).map_err(LambdaHelperEvalError::Preparation)?;
    let callable = require_callable(&prepared[2])?;
    eval_scan_prepared(&prepared[0], &prepared[1], callable, invoker)
}

pub fn eval_byrow_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
    invoker: &(impl CallableInvoker + ?Sized),
) -> Result<EvalValue, LambdaHelperEvalError> {
    if !BYROW_META.arity.accepts(args.len()) {
        return Err(surface_arity_error(&BYROW_META, args.len()));
    }
    let prepared =
        prepare_args_values_only(args, resolver).map_err(LambdaHelperEvalError::Preparation)?;
    let callable = require_callable(&prepared[1])?;
    eval_byrow_prepared(&prepared[0], callable, invoker)
}

pub fn eval_bycol_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
    invoker: &(impl CallableInvoker + ?Sized),
) -> Result<EvalValue, LambdaHelperEvalError> {
    if !BYCOL_META.arity.accepts(args.len()) {
        return Err(surface_arity_error(&BYCOL_META, args.len()));
    }
    let prepared =
        prepare_args_values_only(args, resolver).map_err(LambdaHelperEvalError::Preparation)?;
    let callable = require_callable(&prepared[1])?;
    eval_bycol_prepared(&prepared[0], callable, invoker)
}

pub fn eval_makearray_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
    invoker: &(impl CallableInvoker + ?Sized),
) -> Result<EvalValue, LambdaHelperEvalError> {
    if !MAKEARRAY_META.arity.accepts(args.len()) {
        return Err(surface_arity_error(&MAKEARRAY_META, args.len()));
    }
    let prepared =
        prepare_args_values_only(args, resolver).map_err(LambdaHelperEvalError::Preparation)?;
    let rows = parse_positive_dimension(&prepared[0])?;
    let cols = parse_positive_dimension(&prepared[1])?;
    let callable = require_callable(&prepared[2])?;
    eval_makearray_prepared(rows, cols, callable, invoker)
}

pub fn map_lambda_helper_error_to_ws(error: &LambdaHelperEvalError) -> WorksheetErrorCode {
    match error {
        LambdaHelperEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        LambdaHelperEvalError::Invocation(CallableInvocationError::ArityMismatch { .. }) => {
            WorksheetErrorCode::Value
        }
        LambdaHelperEvalError::Invocation(CallableInvocationError::Worksheet(code)) => *code,
        LambdaHelperEvalError::Invocation(CallableInvocationError::UnsupportedCallableToken(_))
        | LambdaHelperEvalError::Invocation(CallableInvocationError::UnsupportedResultKind(_))
        | LambdaHelperEvalError::Preparation(_)
        | LambdaHelperEvalError::MissingCallable => WorksheetErrorCode::Value,
        LambdaHelperEvalError::NonScalarHelperResult => WorksheetErrorCode::Calc,
        LambdaHelperEvalError::InvalidGeneratedDimensions => WorksheetErrorCode::Value,
    }
}

pub fn eval_isomitted_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, LambdaHelperEvalError> {
    if !ISOMITTED_META.arity.accepts(args.len()) {
        return Err(surface_arity_error(&ISOMITTED_META, args.len()));
    }
    let prepared =
        prepare_args_values_only(args, resolver).map_err(LambdaHelperEvalError::Preparation)?;
    Ok(EvalValue::Logical(matches!(
        prepared.first(),
        Some(PreparedArgValue::MissingArg)
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::functions::adapters::{PreparedArgValue, coerce_prepared_to_number};
    use crate::resolver::{RefResolutionError, ReferenceResolver, ResolverCapabilities};
    use crate::value::{
        CallableArityShape, CallableCaptureMode, EvalArray, ExcelText, ReferenceLike,
    };

    struct MockCallableInvoker;

    impl CallableInvoker for MockCallableInvoker {
        fn invoke(
            &self,
            callable: &LambdaValue,
            args: &[PreparedArgValue],
        ) -> Result<PreparedArgValue, CallableInvocationError> {
            if let Some(code) = args.iter().find_map(|arg| match arg {
                PreparedArgValue::Eval(EvalValue::Error(code)) => Some(*code),
                _ => None,
            }) {
                return Ok(PreparedArgValue::Eval(EvalValue::Error(code)));
            }

            match callable.callable_token.as_str() {
                "helper.add1" => {
                    let n = coerce_prepared_to_number(&args[0]).map_err(|_| {
                        CallableInvocationError::Worksheet(WorksheetErrorCode::Value)
                    })?;
                    Ok(PreparedArgValue::Eval(EvalValue::Number(n + 1.0)))
                }
                "helper.sum2" => {
                    let a = coerce_prepared_to_number(&args[0]).map_err(|_| {
                        CallableInvocationError::Worksheet(WorksheetErrorCode::Value)
                    })?;
                    let b = coerce_prepared_to_number(&args[1]).map_err(|_| {
                        CallableInvocationError::Worksheet(WorksheetErrorCode::Value)
                    })?;
                    Ok(PreparedArgValue::Eval(EvalValue::Number(a + b)))
                }
                "helper.mul2" => {
                    let a = coerce_prepared_to_number(&args[0]).map_err(|_| {
                        CallableInvocationError::Worksheet(WorksheetErrorCode::Value)
                    })?;
                    let b = coerce_prepared_to_number(&args[1]).map_err(|_| {
                        CallableInvocationError::Worksheet(WorksheetErrorCode::Value)
                    })?;
                    Ok(PreparedArgValue::Eval(EvalValue::Number(a * b)))
                }
                "name.capadd" => {
                    let n = coerce_prepared_to_number(&args[0]).map_err(|_| {
                        CallableInvocationError::Worksheet(WorksheetErrorCode::Value)
                    })?;
                    Ok(PreparedArgValue::Eval(EvalValue::Number(n + 2.0)))
                }
                "helper.sum_array" => match &args[0] {
                    PreparedArgValue::Eval(EvalValue::Array(array)) => {
                        let total = array
                            .iter_row_major()
                            .map(|cell| match cell {
                                ArrayCellValue::Number(n) => Ok(*n),
                                ArrayCellValue::Error(code) => {
                                    Err(CallableInvocationError::Worksheet(*code))
                                }
                                _ => Err(CallableInvocationError::Worksheet(
                                    WorksheetErrorCode::Value,
                                )),
                            })
                            .sum::<Result<f64, _>>()?;
                        Ok(PreparedArgValue::Eval(EvalValue::Number(total)))
                    }
                    _ => Err(CallableInvocationError::Worksheet(
                        WorksheetErrorCode::Value,
                    )),
                },
                "helper.nonscalar_plus1" => match &args[0] {
                    PreparedArgValue::Eval(EvalValue::Array(array)) => {
                        let cells = array
                            .iter_row_major()
                            .map(|cell| match cell {
                                ArrayCellValue::Number(n) => ArrayCellValue::Number(n + 1.0),
                                ArrayCellValue::Error(code) => ArrayCellValue::Error(*code),
                                _ => ArrayCellValue::Error(WorksheetErrorCode::Value),
                            })
                            .collect::<Vec<_>>();
                        Ok(PreparedArgValue::Eval(EvalValue::Array(
                            EvalArray::new(array.shape(), cells).expect("shape preserved"),
                        )))
                    }
                    _ => Err(CallableInvocationError::Worksheet(
                        WorksheetErrorCode::Value,
                    )),
                },
                "helper.makearray_coords" => {
                    let r = coerce_prepared_to_number(&args[0]).map_err(|_| {
                        CallableInvocationError::Worksheet(WorksheetErrorCode::Value)
                    })?;
                    let c = coerce_prepared_to_number(&args[1]).map_err(|_| {
                        CallableInvocationError::Worksheet(WorksheetErrorCode::Value)
                    })?;
                    Ok(PreparedArgValue::Eval(EvalValue::Number(r * 10.0 + c)))
                }
                other => Err(CallableInvocationError::UnsupportedCallableToken(
                    other.to_string(),
                )),
            }
        }
    }

    struct NoResolver;

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

    fn helper(callable_token: &str, arity: usize) -> LambdaValue {
        LambdaValue::helper_lambda(
            callable_token.to_string(),
            CallableArityShape::exact(arity),
            CallableCaptureMode::LexicalCapture,
            "helper.invoke.v1",
        )
    }

    fn defined_name(callable_token: &str, arity: usize) -> LambdaValue {
        LambdaValue::defined_name_callable(
            callable_token.to_string(),
            CallableArityShape::exact(arity),
            CallableCaptureMode::LexicalCapture,
            "name.invoke.v1",
        )
    }

    fn num(n: f64) -> PreparedArgValue {
        PreparedArgValue::Eval(EvalValue::Number(n))
    }

    fn callable_arg(callable: LambdaValue) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Lambda(callable))
    }

    #[test]
    fn invoke_callable_prepared_checks_arity_before_invoker() {
        let callable = helper("helper.add1", 1);
        let got = invoke_callable_prepared(&callable, &[num(1.0), num(2.0)], &MockCallableInvoker);
        assert_eq!(
            got,
            Err(CallableInvocationError::ArityMismatch {
                expected_min: 1,
                expected_max: 1,
                actual: 2,
            })
        );
    }

    #[test]
    fn eval_map_prepared_supports_helper_callable_token() {
        let input = PreparedArgValue::Eval(EvalValue::Array(
            EvalArray::from_rows(vec![vec![
                ArrayCellValue::Number(1.0),
                ArrayCellValue::Number(2.0),
            ]])
            .unwrap(),
        ));
        let got = eval_map_prepared(&[input], &helper("helper.add1", 1), &MockCallableInvoker);
        assert_eq!(
            got,
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Number(2.0),
                    ArrayCellValue::Number(3.0),
                ]])
                .unwrap()
            ))
        );
    }

    #[test]
    fn eval_map_prepared_supports_defined_name_callable_token() {
        let input = PreparedArgValue::Eval(EvalValue::Array(
            EvalArray::from_rows(vec![vec![
                ArrayCellValue::Number(1.0),
                ArrayCellValue::Number(2.0),
            ]])
            .unwrap(),
        ));
        let got = eval_map_prepared(
            &[input],
            &defined_name("name.capadd", 1),
            &MockCallableInvoker,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Number(3.0),
                    ArrayCellValue::Number(4.0),
                ]])
                .unwrap()
            ))
        );
    }

    #[test]
    fn eval_map_prepared_pads_missing_partner_with_na() {
        let a = PreparedArgValue::Eval(EvalValue::Array(
            EvalArray::from_rows(vec![vec![
                ArrayCellValue::Number(1.0),
                ArrayCellValue::Number(2.0),
            ]])
            .unwrap(),
        ));
        let b = PreparedArgValue::Eval(EvalValue::Array(
            EvalArray::from_rows(vec![vec![ArrayCellValue::Number(10.0)]]).unwrap(),
        ));
        let got = eval_map_prepared(&[a, b], &helper("helper.sum2", 2), &MockCallableInvoker);
        assert_eq!(
            got,
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Number(11.0),
                    ArrayCellValue::Error(WorksheetErrorCode::NA),
                ]])
                .unwrap()
            ))
        );
    }

    #[test]
    fn eval_reduce_prepared_folds_over_iterable() {
        let iterable = PreparedArgValue::Eval(EvalValue::Array(
            EvalArray::from_rows(vec![vec![
                ArrayCellValue::Number(1.0),
                ArrayCellValue::Number(2.0),
                ArrayCellValue::Number(3.0),
            ]])
            .unwrap(),
        ));
        let got = eval_reduce_prepared(
            &num(0.0),
            &iterable,
            &helper("helper.sum2", 2),
            &MockCallableInvoker,
        );
        assert_eq!(got, Ok(num(6.0)));
    }

    #[test]
    fn eval_scan_prepared_spills_intermediate_accumulations() {
        let iterable = PreparedArgValue::Eval(EvalValue::Array(
            EvalArray::from_rows(vec![vec![
                ArrayCellValue::Number(1.0),
                ArrayCellValue::Number(2.0),
                ArrayCellValue::Number(3.0),
            ]])
            .unwrap(),
        ));
        let got = eval_scan_prepared(
            &num(0.0),
            &iterable,
            &helper("helper.sum2", 2),
            &MockCallableInvoker,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Number(1.0),
                    ArrayCellValue::Number(3.0),
                    ArrayCellValue::Number(6.0),
                ]])
                .unwrap()
            ))
        );
    }

    #[test]
    fn eval_byrow_prepared_returns_one_scalar_result_per_row() {
        let source = PreparedArgValue::Eval(EvalValue::Array(
            EvalArray::from_rows(vec![
                vec![ArrayCellValue::Number(1.0), ArrayCellValue::Number(2.0)],
                vec![ArrayCellValue::Number(3.0), ArrayCellValue::Number(4.0)],
            ])
            .unwrap(),
        ));
        let got = eval_byrow_prepared(
            &source,
            &helper("helper.sum_array", 1),
            &MockCallableInvoker,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Number(3.0)],
                    vec![ArrayCellValue::Number(7.0)],
                ])
                .unwrap()
            ))
        );
    }

    #[test]
    fn eval_bycol_prepared_returns_one_scalar_result_per_column() {
        let source = PreparedArgValue::Eval(EvalValue::Array(
            EvalArray::from_rows(vec![
                vec![ArrayCellValue::Number(1.0), ArrayCellValue::Number(2.0)],
                vec![ArrayCellValue::Number(3.0), ArrayCellValue::Number(4.0)],
            ])
            .unwrap(),
        ));
        let got = eval_bycol_prepared(
            &source,
            &helper("helper.sum_array", 1),
            &MockCallableInvoker,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Number(4.0),
                    ArrayCellValue::Number(6.0),
                ]])
                .unwrap()
            ))
        );
    }

    #[test]
    fn eval_byrow_prepared_rejects_non_scalar_lambda_result() {
        let source = PreparedArgValue::Eval(EvalValue::Array(
            EvalArray::from_rows(vec![
                vec![ArrayCellValue::Number(1.0), ArrayCellValue::Number(2.0)],
                vec![ArrayCellValue::Number(3.0), ArrayCellValue::Number(4.0)],
            ])
            .unwrap(),
        ));
        let got = eval_byrow_prepared(
            &source,
            &helper("helper.nonscalar_plus1", 1),
            &MockCallableInvoker,
        );
        assert_eq!(got, Err(LambdaHelperEvalError::NonScalarHelperResult));
    }

    #[test]
    fn eval_bycol_prepared_rejects_non_scalar_lambda_result() {
        let source = PreparedArgValue::Eval(EvalValue::Array(
            EvalArray::from_rows(vec![
                vec![ArrayCellValue::Number(1.0), ArrayCellValue::Number(2.0)],
                vec![ArrayCellValue::Number(3.0), ArrayCellValue::Number(4.0)],
            ])
            .unwrap(),
        ));
        let got = eval_bycol_prepared(
            &source,
            &helper("helper.nonscalar_plus1", 1),
            &MockCallableInvoker,
        );
        assert_eq!(got, Err(LambdaHelperEvalError::NonScalarHelperResult));
    }

    #[test]
    fn eval_makearray_prepared_uses_one_based_generated_coordinates() {
        let got = eval_makearray_prepared(
            2,
            3,
            &helper("helper.makearray_coords", 2),
            &MockCallableInvoker,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![
                        ArrayCellValue::Number(11.0),
                        ArrayCellValue::Number(12.0),
                        ArrayCellValue::Number(13.0),
                    ],
                    vec![
                        ArrayCellValue::Number(21.0),
                        ArrayCellValue::Number(22.0),
                        ArrayCellValue::Number(23.0),
                    ],
                ])
                .unwrap()
            ))
        );
    }

    #[test]
    fn eval_makearray_prepared_rejects_zero_dimensions() {
        let got = eval_makearray_prepared(
            0,
            3,
            &helper("helper.makearray_coords", 2),
            &MockCallableInvoker,
        );
        assert_eq!(got, Err(LambdaHelperEvalError::InvalidGeneratedDimensions));
    }

    #[test]
    fn eval_map_surface_matches_seeded_bare_spill_lane() {
        let got = eval_map_surface(
            &[
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![vec![
                        ArrayCellValue::Number(1.0),
                        ArrayCellValue::Number(2.0),
                    ]])
                    .unwrap(),
                )),
                callable_arg(helper("helper.add1", 1)),
            ],
            &NoResolver,
            &MockCallableInvoker,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Number(2.0),
                    ArrayCellValue::Number(3.0),
                ]])
                .unwrap()
            ))
        );
    }

    #[test]
    fn eval_map_surface_matches_seeded_mismatch_lane() {
        let got = eval_map_surface(
            &[
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![vec![
                        ArrayCellValue::Number(1.0),
                        ArrayCellValue::Number(2.0),
                    ]])
                    .unwrap(),
                )),
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![vec![ArrayCellValue::Number(10.0)]]).unwrap(),
                )),
                callable_arg(helper("helper.sum2", 2)),
            ],
            &NoResolver,
            &MockCallableInvoker,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Number(11.0),
                    ArrayCellValue::Error(WorksheetErrorCode::NA),
                ]])
                .unwrap()
            ))
        );
    }

    #[test]
    fn eval_reduce_surface_matches_seeded_sum_lane() {
        let got = eval_reduce_surface(
            &[
                CallArgValue::Eval(EvalValue::Number(0.0)),
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![vec![
                        ArrayCellValue::Number(1.0),
                        ArrayCellValue::Number(2.0),
                        ArrayCellValue::Number(3.0),
                    ]])
                    .unwrap(),
                )),
                callable_arg(helper("helper.sum2", 2)),
            ],
            &NoResolver,
            &MockCallableInvoker,
        );
        assert_eq!(got, Ok(num(6.0)));
    }

    #[test]
    fn eval_scan_surface_matches_seeded_spill_lane() {
        let got = eval_scan_surface(
            &[
                CallArgValue::Eval(EvalValue::Number(0.0)),
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![vec![
                        ArrayCellValue::Number(1.0),
                        ArrayCellValue::Number(2.0),
                        ArrayCellValue::Number(3.0),
                    ]])
                    .unwrap(),
                )),
                callable_arg(helper("helper.sum2", 2)),
            ],
            &NoResolver,
            &MockCallableInvoker,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Number(1.0),
                    ArrayCellValue::Number(3.0),
                    ArrayCellValue::Number(6.0),
                ]])
                .unwrap()
            ))
        );
    }

    #[test]
    fn eval_byrow_surface_matches_seeded_scalar_lane() {
        let got = eval_byrow_surface(
            &[
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![
                        vec![ArrayCellValue::Number(1.0), ArrayCellValue::Number(2.0)],
                        vec![ArrayCellValue::Number(3.0), ArrayCellValue::Number(4.0)],
                    ])
                    .unwrap(),
                )),
                callable_arg(helper("helper.sum_array", 1)),
            ],
            &NoResolver,
            &MockCallableInvoker,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Number(3.0)],
                    vec![ArrayCellValue::Number(7.0)],
                ])
                .unwrap()
            ))
        );
    }

    #[test]
    fn eval_byrow_surface_maps_non_scalar_result_to_calc() {
        let err = eval_byrow_surface(
            &[
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![
                        vec![ArrayCellValue::Number(1.0), ArrayCellValue::Number(2.0)],
                        vec![ArrayCellValue::Number(3.0), ArrayCellValue::Number(4.0)],
                    ])
                    .unwrap(),
                )),
                callable_arg(helper("helper.nonscalar_plus1", 1)),
            ],
            &NoResolver,
            &MockCallableInvoker,
        )
        .unwrap_err();
        assert_eq!(
            map_lambda_helper_error_to_ws(&err),
            WorksheetErrorCode::Calc
        );
    }

    #[test]
    fn eval_bycol_surface_matches_seeded_scalar_lane() {
        let got = eval_bycol_surface(
            &[
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![
                        vec![ArrayCellValue::Number(1.0), ArrayCellValue::Number(2.0)],
                        vec![ArrayCellValue::Number(3.0), ArrayCellValue::Number(4.0)],
                    ])
                    .unwrap(),
                )),
                callable_arg(helper("helper.sum_array", 1)),
            ],
            &NoResolver,
            &MockCallableInvoker,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Number(4.0),
                    ArrayCellValue::Number(6.0),
                ]])
                .unwrap()
            ))
        );
    }

    #[test]
    fn eval_makearray_surface_matches_seeded_basic_lane() {
        let got = eval_makearray_surface(
            &[
                CallArgValue::Eval(EvalValue::Number(2.0)),
                CallArgValue::Eval(EvalValue::Number(3.0)),
                callable_arg(helper("helper.makearray_coords", 2)),
            ],
            &NoResolver,
            &MockCallableInvoker,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![
                        ArrayCellValue::Number(11.0),
                        ArrayCellValue::Number(12.0),
                        ArrayCellValue::Number(13.0),
                    ],
                    vec![
                        ArrayCellValue::Number(21.0),
                        ArrayCellValue::Number(22.0),
                        ArrayCellValue::Number(23.0),
                    ],
                ])
                .unwrap()
            ))
        );
    }

    #[test]
    fn eval_isomitted_surface_returns_false_for_present_arg() {
        let got =
            eval_isomitted_surface(&[CallArgValue::Eval(EvalValue::Number(1.0))], &NoResolver);
        assert_eq!(got, Ok(EvalValue::Logical(false)));
    }

    #[test]
    fn eval_isomitted_surface_returns_true_for_missing_arg() {
        let got = eval_isomitted_surface(&[CallArgValue::MissingArg], &NoResolver);
        assert_eq!(got, Ok(EvalValue::Logical(true)));
    }

    #[test]
    fn prepare_and_invoke_callable_handles_direct_invocation_lane() {
        let callable = defined_name("name.capadd", 1);
        let got = prepare_and_invoke_callable(
            &[CallArgValue::Eval(EvalValue::Number(3.0))],
            &callable,
            &NoResolver,
            &MockCallableInvoker,
        );
        assert_eq!(got, Ok(num(5.0)));
    }

    #[test]
    fn invoke_callable_can_return_textual_values() {
        struct TextInvoker;

        impl CallableInvoker for TextInvoker {
            fn invoke(
                &self,
                callable: &LambdaValue,
                _args: &[PreparedArgValue],
            ) -> Result<PreparedArgValue, CallableInvocationError> {
                if callable.callable_token == "helper.text" {
                    return Ok(PreparedArgValue::Eval(EvalValue::Text(
                        ExcelText::from_utf16_code_units("ok".encode_utf16().collect()),
                    )));
                }
                Err(CallableInvocationError::UnsupportedCallableToken(
                    callable.callable_token.clone(),
                ))
            }
        }

        let got = invoke_callable_prepared(&helper("helper.text", 0), &[], &TextInvoker);
        assert_eq!(
            got,
            Ok(PreparedArgValue::Eval(EvalValue::Text(
                ExcelText::from_utf16_code_units("ok".encode_utf16().collect())
            )))
        );
    }
}

use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    PreparedArgValue, coerce_prepared_to_number, prepare_arg_values_only,
};
use crate::resolver::{ReferenceResolver, resolve_eval_value};
use crate::value::{
    ArrayCellValue, ArrayShape, CallArgValue, EvalArray, EvalValue, WorksheetErrorCode,
};

const MATRIX_BASE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.MATRIX_BASE",
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

pub const MDETERM_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.MDETERM",
    ..MATRIX_BASE_META
};

pub const MINVERSE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.MINVERSE",
    ..MATRIX_BASE_META
};

pub const MUNIT_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.MUNIT",
    arity: Arity::exact(1),
    ..MATRIX_BASE_META
};

pub const MMULT_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.MMULT",
    arity: Arity::exact(2),
    ..MATRIX_BASE_META
};

#[derive(Debug, Clone, PartialEq)]
pub enum MatrixEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
    Domain(WorksheetErrorCode),
}

fn resolve_arg_eval(
    arg: &CallArgValue,
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, MatrixEvalError> {
    match arg {
        CallArgValue::Eval(EvalValue::Reference(reference))
        | CallArgValue::Reference(reference) => {
            let resolved = resolve_eval_value(resolver, reference)
                .map_err(CoercionError::RefResolution)
                .map_err(MatrixEvalError::Coercion)?;
            resolve_arg_eval(&CallArgValue::Eval(resolved), resolver)
        }
        CallArgValue::Eval(value) => Ok(value.clone()),
        CallArgValue::MissingArg => Err(MatrixEvalError::Coercion(CoercionError::MissingArg)),
        CallArgValue::EmptyCell => Err(MatrixEvalError::Domain(WorksheetErrorCode::Value)),
    }
}

fn matrix_from_value(value: &EvalValue) -> Result<Vec<Vec<f64>>, WorksheetErrorCode> {
    match value {
        EvalValue::Number(n) => Ok(vec![vec![*n]]),
        EvalValue::Error(code) => Err(*code),
        EvalValue::Array(array) => {
            let shape = array.shape();
            let mut rows = Vec::with_capacity(shape.rows);
            for row_idx in 0..shape.rows {
                let mut row = Vec::with_capacity(shape.cols);
                for cell in array
                    .row_slice(row_idx)
                    .expect("row index bounded by shape rows")
                {
                    match cell {
                        ArrayCellValue::Number(n) => row.push(*n),
                        ArrayCellValue::Error(code) => return Err(*code),
                        ArrayCellValue::Text(_)
                        | ArrayCellValue::Logical(_)
                        | ArrayCellValue::EmptyCell => return Err(WorksheetErrorCode::Value),
                    }
                }
                rows.push(row);
            }
            Ok(rows)
        }
        EvalValue::Text(_)
        | EvalValue::Logical(_)
        | EvalValue::Reference(_)
        | EvalValue::Lambda(_) => Err(WorksheetErrorCode::Value),
    }
}

fn value_from_matrix(matrix: &[Vec<f64>]) -> Result<EvalValue, WorksheetErrorCode> {
    let rows = matrix.len();
    let cols = matrix.first().map_or(0, Vec::len);

    let cells = matrix
        .iter()
        .flat_map(|row| row.iter().copied().map(ArrayCellValue::Number))
        .collect::<Vec<_>>();
    EvalArray::new(ArrayShape { rows, cols }, cells)
        .map(EvalValue::Array)
        .ok_or(WorksheetErrorCode::Value)
}

fn determinant_kernel(matrix: &[Vec<f64>]) -> Result<f64, WorksheetErrorCode> {
    let n = matrix.len();
    if n == 0 || matrix.iter().any(|row| row.len() != n) {
        return Err(WorksheetErrorCode::Value);
    }
    let mut work = matrix.to_vec();
    let mut swaps = 0usize;
    let mut det = 1.0;
    const EPS: f64 = 1e-12;

    for pivot_idx in 0..n {
        let mut pivot_row = pivot_idx;
        let mut pivot_abs = work[pivot_idx][pivot_idx].abs();
        for row in (pivot_idx + 1)..n {
            let candidate = work[row][pivot_idx].abs();
            if candidate > pivot_abs {
                pivot_abs = candidate;
                pivot_row = row;
            }
        }

        if pivot_abs < EPS {
            return Ok(0.0);
        }

        if pivot_row != pivot_idx {
            work.swap(pivot_row, pivot_idx);
            swaps += 1;
        }

        let pivot = work[pivot_idx][pivot_idx];
        det *= pivot;
        for row in (pivot_idx + 1)..n {
            let factor = work[row][pivot_idx] / pivot;
            for col in pivot_idx..n {
                work[row][col] -= factor * work[pivot_idx][col];
            }
        }
    }

    if swaps % 2 == 1 {
        det = -det;
    }
    Ok(det)
}

fn inverse_kernel(matrix: &[Vec<f64>]) -> Result<Vec<Vec<f64>>, WorksheetErrorCode> {
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

    let mut inverse = Vec::with_capacity(n);
    for row in augmented {
        inverse.push(row[n..].to_vec());
    }
    Ok(inverse)
}

fn mmult_kernel(
    left: &[Vec<f64>],
    right: &[Vec<f64>],
) -> Result<Vec<Vec<f64>>, WorksheetErrorCode> {
    if left.is_empty() || right.is_empty() {
        return Err(WorksheetErrorCode::Value);
    }
    let left_cols = left.first().map_or(0, Vec::len);
    let right_cols = right.first().map_or(0, Vec::len);
    if left.iter().any(|row| row.len() != left_cols)
        || right.iter().any(|row| row.len() != right_cols)
    {
        return Err(WorksheetErrorCode::Value);
    }
    if left_cols != right.len() {
        return Err(WorksheetErrorCode::Value);
    }

    let mut result = Vec::with_capacity(left.len());
    for left_row in left {
        let mut row = Vec::with_capacity(right_cols);
        for col_idx in 0..right_cols {
            let mut acc = 0.0;
            for inner_idx in 0..left_cols {
                acc += left_row[inner_idx] * right[inner_idx][col_idx];
            }
            row.push(acc);
        }
        result.push(row);
    }
    Ok(result)
}

fn munit_size_from_prepared(arg: &PreparedArgValue) -> Result<usize, MatrixEvalError> {
    let n = coerce_prepared_to_number(arg).map_err(MatrixEvalError::Coercion)?;
    if !n.is_finite() {
        return Err(MatrixEvalError::Domain(WorksheetErrorCode::Value));
    }
    let truncated = n.trunc();
    if truncated <= 0.0 {
        return Err(MatrixEvalError::Domain(WorksheetErrorCode::Value));
    }
    if truncated > (usize::MAX as f64) {
        return Err(MatrixEvalError::Domain(WorksheetErrorCode::Value));
    }
    Ok(truncated as usize)
}

fn identity_matrix(size: usize) -> Vec<Vec<f64>> {
    let mut rows = Vec::with_capacity(size);
    for row_idx in 0..size {
        let mut row = Vec::with_capacity(size);
        for col_idx in 0..size {
            row.push(if row_idx == col_idx { 1.0 } else { 0.0 });
        }
        rows.push(row);
    }
    rows
}

pub fn eval_mdeterm_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, MatrixEvalError> {
    if !MDETERM_META.arity.accepts(args.len()) {
        return Err(MatrixEvalError::ArityMismatch {
            expected_min: MDETERM_META.arity.min,
            expected_max: MDETERM_META.arity.max,
            actual: args.len(),
        });
    }
    let matrix = matrix_from_value(&resolve_arg_eval(&args[0], resolver)?)
        .map_err(MatrixEvalError::Domain)?;
    determinant_kernel(&matrix)
        .map(EvalValue::Number)
        .map_err(MatrixEvalError::Domain)
}

pub fn eval_minverse_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, MatrixEvalError> {
    if !MINVERSE_META.arity.accepts(args.len()) {
        return Err(MatrixEvalError::ArityMismatch {
            expected_min: MINVERSE_META.arity.min,
            expected_max: MINVERSE_META.arity.max,
            actual: args.len(),
        });
    }
    let matrix = matrix_from_value(&resolve_arg_eval(&args[0], resolver)?)
        .map_err(MatrixEvalError::Domain)?;
    inverse_kernel(&matrix)
        .and_then(|inverse| value_from_matrix(&inverse))
        .map_err(MatrixEvalError::Domain)
}

pub fn eval_mmult_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, MatrixEvalError> {
    if !MMULT_META.arity.accepts(args.len()) {
        return Err(MatrixEvalError::ArityMismatch {
            expected_min: MMULT_META.arity.min,
            expected_max: MMULT_META.arity.max,
            actual: args.len(),
        });
    }
    let left = matrix_from_value(&resolve_arg_eval(&args[0], resolver)?)
        .map_err(MatrixEvalError::Domain)?;
    let right = matrix_from_value(&resolve_arg_eval(&args[1], resolver)?)
        .map_err(MatrixEvalError::Domain)?;
    mmult_kernel(&left, &right)
        .and_then(|product| value_from_matrix(&product))
        .map_err(MatrixEvalError::Domain)
}

pub fn eval_munit_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, MatrixEvalError> {
    if !MUNIT_META.arity.accepts(args.len()) {
        return Err(MatrixEvalError::ArityMismatch {
            expected_min: MUNIT_META.arity.min,
            expected_max: MUNIT_META.arity.max,
            actual: args.len(),
        });
    }
    let prepared =
        prepare_arg_values_only(&args[0], resolver).map_err(MatrixEvalError::Coercion)?;
    let size = munit_size_from_prepared(&prepared)?;
    value_from_matrix(&identity_matrix(size)).map_err(MatrixEvalError::Domain)
}

pub fn map_matrix_error_to_ws(error: &MatrixEvalError) -> WorksheetErrorCode {
    match error {
        MatrixEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        MatrixEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        MatrixEvalError::Coercion(_) => WorksheetErrorCode::Value,
        MatrixEvalError::Domain(code) => *code,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::{RefResolutionError, ResolverCapabilities};
    use crate::value::{ExcelText, ReferenceLike};

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

    #[test]
    fn mdeterm_matches_excel_seed_rows() {
        let got = eval_mdeterm_surface(
            &[CallArgValue::Eval(EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Number(1.0), ArrayCellValue::Number(2.0)],
                    vec![ArrayCellValue::Number(3.0), ArrayCellValue::Number(4.0)],
                ])
                .unwrap(),
            ))],
            &NoResolver,
        );
        assert_eq!(got, Ok(EvalValue::Number(-2.0)));
    }

    #[test]
    fn minverse_spills_excel_seed_matrix() {
        let got = eval_minverse_surface(
            &[CallArgValue::Eval(EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Number(1.0), ArrayCellValue::Number(2.0)],
                    vec![ArrayCellValue::Number(3.0), ArrayCellValue::Number(4.0)],
                ])
                .unwrap(),
            ))],
            &NoResolver,
        )
        .unwrap();

        let EvalValue::Array(array) = got else {
            panic!("MINVERSE should return an array");
        };
        let expected = [[-2.0_f64, 1.0_f64], [1.5_f64, -0.5_f64]];
        for row in 0..2 {
            for col in 0..2 {
                match array.get(row, col) {
                    Some(ArrayCellValue::Number(n)) => {
                        assert!((*n - expected[row][col]).abs() < 1e-12);
                    }
                    other => panic!("unexpected inverse cell: {:?}", other),
                }
            }
        }
    }

    #[test]
    fn mmult_matches_excel_seed_rows() {
        let got = eval_mmult_surface(
            &[
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![
                        vec![ArrayCellValue::Number(1.0), ArrayCellValue::Number(2.0)],
                        vec![ArrayCellValue::Number(3.0), ArrayCellValue::Number(4.0)],
                    ])
                    .unwrap(),
                )),
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![
                        vec![ArrayCellValue::Number(5.0)],
                        vec![ArrayCellValue::Number(6.0)],
                    ])
                    .unwrap(),
                )),
            ],
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Number(17.0)],
                    vec![ArrayCellValue::Number(39.0)],
                ])
                .unwrap(),
            ))
        );
    }

    #[test]
    fn munit_matches_excel_seed_rows() {
        let got = eval_munit_surface(&[CallArgValue::Eval(EvalValue::Number(3.0))], &NoResolver);
        assert_eq!(
            got,
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![
                        ArrayCellValue::Number(1.0),
                        ArrayCellValue::Number(0.0),
                        ArrayCellValue::Number(0.0),
                    ],
                    vec![
                        ArrayCellValue::Number(0.0),
                        ArrayCellValue::Number(1.0),
                        ArrayCellValue::Number(0.0),
                    ],
                    vec![
                        ArrayCellValue::Number(0.0),
                        ArrayCellValue::Number(0.0),
                        ArrayCellValue::Number(1.0),
                    ],
                ])
                .unwrap(),
            ))
        );
    }

    #[test]
    fn munit_one_preserves_array_value() {
        let got = eval_munit_surface(&[CallArgValue::Eval(EvalValue::Number(1.0))], &NoResolver);
        assert_eq!(
            got,
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![vec![ArrayCellValue::Number(1.0)]],).unwrap()
            ))
        );
    }

    #[test]
    fn munit_truncates_and_coerces_text_numeric() {
        let got = eval_munit_surface(
            &[CallArgValue::Eval(EvalValue::Text(
                ExcelText::from_utf16_code_units("2.9".encode_utf16().collect()),
            ))],
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Number(1.0), ArrayCellValue::Number(0.0)],
                    vec![ArrayCellValue::Number(0.0), ArrayCellValue::Number(1.0)],
                ])
                .unwrap(),
            ))
        );
    }

    #[test]
    fn singular_inverse_maps_to_num() {
        let got = eval_minverse_surface(
            &[CallArgValue::Eval(EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Number(1.0), ArrayCellValue::Number(2.0)],
                    vec![ArrayCellValue::Number(2.0), ArrayCellValue::Number(4.0)],
                ])
                .unwrap(),
            ))],
            &NoResolver,
        );
        assert_eq!(got, Err(MatrixEvalError::Domain(WorksheetErrorCode::Num)));
    }

    #[test]
    fn nonsquare_inputs_map_to_value() {
        let nonsquare = CallArgValue::Eval(EvalValue::Array(
            EvalArray::from_rows(vec![
                vec![
                    ArrayCellValue::Number(1.0),
                    ArrayCellValue::Number(2.0),
                    ArrayCellValue::Number(3.0),
                ],
                vec![
                    ArrayCellValue::Number(4.0),
                    ArrayCellValue::Number(5.0),
                    ArrayCellValue::Number(6.0),
                ],
            ])
            .unwrap(),
        ));
        assert_eq!(
            eval_mdeterm_surface(&[nonsquare.clone()], &NoResolver),
            Err(MatrixEvalError::Domain(WorksheetErrorCode::Value))
        );
        assert_eq!(
            eval_minverse_surface(&[nonsquare], &NoResolver),
            Err(MatrixEvalError::Domain(WorksheetErrorCode::Value))
        );
    }

    #[test]
    fn matrix_inputs_reject_non_numeric_cells() {
        let got = eval_mmult_surface(
            &[
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![
                        vec![ArrayCellValue::Logical(true), ArrayCellValue::Number(2.0)],
                        vec![ArrayCellValue::Number(3.0), ArrayCellValue::Number(4.0)],
                    ])
                    .unwrap(),
                )),
                CallArgValue::Eval(EvalValue::Number(2.0)),
            ],
            &NoResolver,
        );
        assert_eq!(got, Err(MatrixEvalError::Domain(WorksheetErrorCode::Value)));
    }

    #[test]
    fn scalar_numeric_inputs_preserve_one_by_one_matrix_results_as_arrays() {
        assert_eq!(
            eval_mdeterm_surface(&[CallArgValue::Eval(EvalValue::Number(5.0))], &NoResolver),
            Ok(EvalValue::Number(5.0))
        );
        assert_eq!(
            eval_minverse_surface(&[CallArgValue::Eval(EvalValue::Number(5.0))], &NoResolver),
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![vec![ArrayCellValue::Number(0.2)]],).unwrap()
            ))
        );
        assert_eq!(
            eval_mmult_surface(
                &[
                    CallArgValue::Eval(EvalValue::Number(5.0)),
                    CallArgValue::Eval(EvalValue::Number(2.0)),
                ],
                &NoResolver,
            ),
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![vec![ArrayCellValue::Number(10.0)]],).unwrap()
            ))
        );
    }

    #[test]
    fn matrix_meta_matches_batch_shape() {
        assert_eq!(MDETERM_META.function_id, "FUNC.MDETERM");
        assert_eq!(MINVERSE_META.function_id, "FUNC.MINVERSE");
        assert_eq!(MUNIT_META.function_id, "FUNC.MUNIT");
        assert_eq!(MMULT_META.function_id, "FUNC.MMULT");
        assert_eq!(MMULT_META.arity, Arity::exact(2));
        assert_eq!(
            MDETERM_META.arg_preparation_profile,
            ArgPreparationProfile::RefsVisibleInAdapter
        );
    }
}

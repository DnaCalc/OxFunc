use crate::coercion::CoercionError;
use crate::excel_numeric::{excel_x87_div, excel_x87_mul, excel_x87_sub};
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, ErrorCollapseProfile,
    FecDependencyProfile, FunctionMeta, HostInteractionClass, KernelSignatureClass,
    ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{coerce_prepared_to_number, prepare_arg_values_only};
use crate::resolver::{ReferenceSystemProvider, resolve_eval_value};
use crate::value::{ArrayShape, CalcArray, WorksheetErrorCode};
use crate::value::{CalcValue, CoreValue};

const MATRIX_BASE_META: FunctionMeta = function_spec! {
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
    error_collapse_profile: ErrorCollapseProfile::ReductionFold,
    precision_rounding_profile: FunctionMeta::DEFAULT_PRECISION_ROUNDING_PROFILE,
    ..MATRIX_BASE_META
};

pub const MINVERSE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.MINVERSE",
    error_collapse_profile: ErrorCollapseProfile::ReductionFold,
    precision_rounding_profile: FunctionMeta::DEFAULT_PRECISION_ROUNDING_PROFILE,
    ..MATRIX_BASE_META
};

// MUNIT's single dimension argument is scalar-shaped by-index and broadcasts over an array
// (`[0]`). The other matrix surfaces take array arguments and lift natively (default). Verified
// live Excel 16.0 build 20026.
pub const MUNIT_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.MUNIT",
    arity: Arity::exact(1),
    lift_broadcast_profile: FunctionMeta::lift_at(&[0]),
    ..MATRIX_BASE_META
};

pub const MMULT_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.MMULT",
    arity: Arity::exact(2),
    error_collapse_profile: ErrorCollapseProfile::ReductionFold,
    precision_rounding_profile: FunctionMeta::DEFAULT_PRECISION_ROUNDING_PROFILE,
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
    arg: &CalcValue,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, MatrixEvalError> {
    match arg.core() {
        CoreValue::Reference(reference) => {
            let resolved = resolve_eval_value(resolver, reference)
                .map_err(CoercionError::RefResolution)
                .map_err(MatrixEvalError::Coercion)?;
            resolve_arg_eval(&(resolved), resolver)
        }
        CoreValue::Missing => Err(MatrixEvalError::Coercion(CoercionError::MissingArg)),
        CoreValue::Empty => Err(MatrixEvalError::Domain(WorksheetErrorCode::Value)),
        _ => Ok(arg.clone()),
    }
}

fn matrix_from_value(value: &CalcValue) -> Result<Vec<Vec<f64>>, WorksheetErrorCode> {
    match value.core() {
        CoreValue::Number(n) => Ok(vec![vec![*n]]),
        CoreValue::Error(code) => Err(*code),
        CoreValue::Array(array) => {
            let shape = array.shape();
            let mut rows = Vec::with_capacity(shape.rows);
            for row_idx in 0..shape.rows {
                let mut row = Vec::with_capacity(shape.cols);
                for cell in array
                    .row_slice(row_idx)
                    .expect("row index bounded by shape rows")
                {
                    match cell.core() {
                        CoreValue::Number(n) => row.push(*n),
                        CoreValue::Error(code) => return Err(*code),
                        CoreValue::Text(_)
                        | CoreValue::Logical(_)
                        | CoreValue::Empty
                        | CoreValue::Missing
                        | CoreValue::Array(_)
                        | CoreValue::Reference(_) => return Err(WorksheetErrorCode::Value),
                    }
                }
                rows.push(row);
            }
            Ok(rows)
        }
        CoreValue::Text(_) | CoreValue::Logical(_) | CoreValue::Reference(_) => {
            Err(WorksheetErrorCode::Value)
        }
        _ => Err(WorksheetErrorCode::Value),
    }
}

fn value_from_matrix(matrix: &[Vec<f64>]) -> Result<CalcValue, WorksheetErrorCode> {
    let rows = matrix.len();
    let cols = matrix.first().map_or(0, Vec::len);

    let cells = matrix
        .iter()
        .flat_map(|row| row.iter().copied().map(CalcValue::number))
        .collect::<Vec<_>>();
    CalcArray::new(ArrayShape { rows, cols }, cells)
        .map(CalcValue::array)
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

// Excel MINVERSE runs a Doolittle LU with partial pivoting, then solves the inverse one
// column at a time against a permuted unit vector: forward substitution through the unit
// lower factor (no division), then division-form back substitution through the upper
// factor. Every arithmetic site publishes through the Excel x87 PC64-to-PC53 double-round
// boundary: multiplier division, elimination multiply/subtract, forward and backward solve
// multiply/subtract, and final division. The output array canonicalizes numeric zero to +0.
// The graph is exact on 607 banked cells, 576 refinement cells, and a disjoint 416-cell
// current-build publication gate; that final gate independently discriminates all eight
// x87 sites and the signed-zero publication rule.
fn inverse_kernel(matrix: &[Vec<f64>]) -> Result<Vec<Vec<f64>>, WorksheetErrorCode> {
    let n = matrix.len();
    if n == 0 || matrix.iter().any(|row| row.len() != n) {
        return Err(WorksheetErrorCode::Value);
    }

    // In-place LU: below the diagonal holds the unit-lower multipliers, on and above holds U.
    let mut a = matrix.to_vec();
    let mut piv: Vec<usize> = (0..n).collect();
    const EPS: f64 = 1e-12;

    for k in 0..n {
        let mut pivot_row = k;
        let mut pivot_abs = a[k][k].abs();
        for row in (k + 1)..n {
            let candidate = a[row][k].abs();
            if candidate > pivot_abs {
                pivot_abs = candidate;
                pivot_row = row;
            }
        }

        if pivot_abs < EPS {
            return Err(WorksheetErrorCode::Num);
        }

        if pivot_row != k {
            a.swap(pivot_row, k);
            piv.swap(pivot_row, k);
        }

        let pivot = a[k][k];
        for row in (k + 1)..n {
            let factor = excel_x87_div(a[row][k], pivot);
            a[row][k] = factor;
            for col in (k + 1)..n {
                let product = excel_x87_mul(factor, a[k][col]);
                a[row][col] = excel_x87_sub(a[row][col], product);
            }
        }
    }

    // Solve A·x = e_j for each column j of the inverse, using the stored LU and the row
    // permutation. Forward: L·y = P·e_j (unit lower). Back: U·x = y (plain division).
    let mut inverse = vec![vec![0.0_f64; n]; n];
    for j in 0..n {
        let mut y = vec![0.0_f64; n];
        for i in 0..n {
            let mut s = if piv[i] == j { 1.0 } else { 0.0 };
            for kk in 0..i {
                let product = excel_x87_mul(a[i][kk], y[kk]);
                s = excel_x87_sub(s, product);
            }
            y[i] = s;
        }

        let mut x = vec![0.0_f64; n];
        for i in (0..n).rev() {
            let mut s = y[i];
            for kk in (i + 1)..n {
                let product = excel_x87_mul(a[i][kk], x[kk]);
                s = excel_x87_sub(s, product);
            }
            x[i] = excel_x87_div(s, a[i][i]);
        }

        for (i, value) in x.into_iter().enumerate() {
            inverse[i][j] = if value == 0.0 { 0.0 } else { value };
        }
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

fn munit_size_from_prepared(arg: &CalcValue) -> Result<usize, MatrixEvalError> {
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
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, MatrixEvalError> {
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
        .map(CalcValue::number)
        .map_err(MatrixEvalError::Domain)
}

pub fn eval_minverse_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, MatrixEvalError> {
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
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, MatrixEvalError> {
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
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, MatrixEvalError> {
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
    use crate::resolver::ReferenceSystemCapabilities;
    use crate::value::ExcelText;

    struct NoResolver;

    impl ReferenceSystemProvider for NoResolver {
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

    fn assert_minverse_cell(matrix_bits: &[&[u64]], row: usize, col: usize, expected_bits: u64) {
        let matrix = CalcArray::from_rows(
            matrix_bits
                .iter()
                .map(|values| {
                    values
                        .iter()
                        .map(|bits| CalcValue::number(f64::from_bits(*bits)))
                        .collect()
                })
                .collect(),
        )
        .unwrap();
        let got = eval_minverse_surface(&[CalcValue::array(matrix)], &NoResolver).unwrap();
        let CoreValue::Array(array) = got.core else {
            panic!("MINVERSE should return an array");
        };
        let Some(CoreValue::Number(value)) = array.get(row, col).map(CalcValue::core) else {
            panic!("MINVERSE cell ({row}, {col}) should be numeric");
        };
        assert_eq!(value.to_bits(), expected_bits);
    }

    #[test]
    fn mdeterm_matches_excel_seed_rows() {
        let got = eval_mdeterm_surface(
            &[(CalcValue::array(
                CalcArray::from_rows(vec![
                    vec![CalcValue::number(1.0), CalcValue::number(2.0)],
                    vec![CalcValue::number(3.0), CalcValue::number(4.0)],
                ])
                .unwrap(),
            ))],
            &NoResolver,
        );
        assert_eq!(got, Ok(CalcValue::number(-2.0)));
    }

    #[test]
    fn minverse_spills_excel_seed_matrix() {
        let got = eval_minverse_surface(
            &[(CalcValue::array(
                CalcArray::from_rows(vec![
                    vec![CalcValue::number(1.0), CalcValue::number(2.0)],
                    vec![CalcValue::number(3.0), CalcValue::number(4.0)],
                ])
                .unwrap(),
            ))],
            &NoResolver,
        )
        .unwrap();

        let CoreValue::Array(array) = got.core else {
            panic!("MINVERSE should return an array");
        };
        let expected = [[-2.0_f64, 1.0_f64], [1.5_f64, -0.5_f64]];
        for row in 0..2 {
            for col in 0..2 {
                match array.get(row, col).map(CalcValue::core) {
                    Some(CoreValue::Number(n)) => {
                        assert!((*n - expected[row][col]).abs() < 1e-12);
                    }
                    other => panic!("unexpected inverse cell: {:?}", other),
                }
            }
        }
    }

    #[test]
    fn minverse_full_x87_publication_graph_exact_pins() {
        // One fresh build-20228/CV2 NoCache discriminator per arithmetic site,
        // followed by the independently selected +0 publication witness.
        assert_minverse_cell(
            &[
                &[0x4003_4846_3058_f9c4, 0xc005_6605_08a6_2bed],
                &[0xc019_c907_fe00_7922, 0x4005_5a34_5bda_f0a9],
            ],
            1,
            1,
            0xbfcc_8a8c_66e7_c011,
        );
        assert_minverse_cell(
            &[
                &[
                    0xc002_2269_2454_97cd,
                    0x0000_0000_0000_0000,
                    0x0000_0000_0000_0000,
                ],
                &[
                    0x3fd7_cfa3_af17_fc9a,
                    0x4006_4335_da4c_47c0,
                    0x0000_0000_0000_0000,
                ],
                &[
                    0xc00a_fbb8_6050_2392,
                    0xc00f_e04b_d340_f1f2,
                    0x4013_9a68_ea9d_61ef,
                ],
            ],
            1,
            2,
            0x0000_0000_0000_0000,
        );
        assert_minverse_cell(
            &[
                &[0x3d80_0000_000c_2fdb, 0xc015_76d2_a0cf_03df],
                &[0x4006_e628_4234_0ce7, 0xc010_fcac_3dad_b044],
            ],
            1,
            0,
            0xbfc7_da8d_d3a1_426d,
        );
        assert_minverse_cell(
            &[
                &[
                    0xc020_0000_0000_0000,
                    0x3ff0_0000_0000_0000,
                    0x4022_0000_0000_0000,
                    0xc010_0000_0000_0000,
                ],
                &[
                    0xc010_0000_0000_0000,
                    0x4022_0000_0000_0000,
                    0x4020_0000_0000_0000,
                    0xc020_0000_0000_0000,
                ],
                &[
                    0x4020_0000_0000_0000,
                    0xc000_0000_0000_0000,
                    0xc01c_0000_0000_0000,
                    0x0000_0000_0000_0000,
                ],
                &[
                    0x401c_0000_0000_0000,
                    0x4022_0000_0000_0000,
                    0x3ff0_0000_0000_0000,
                    0x4014_0000_0000_0000,
                ],
            ],
            2,
            3,
            0x3fbb_fa67_84e5_6bb8,
        );
        assert_minverse_cell(
            &[
                &[
                    0x4020_0000_0000_0000,
                    0x4008_0000_0000_0000,
                    0x401c_0000_0000_0000,
                    0x4010_0000_0000_0000,
                ],
                &[
                    0xc022_0000_0000_0000,
                    0x4008_0000_0000_0000,
                    0xc018_0000_0000_0000,
                    0xc000_0000_0000_0000,
                ],
                &[
                    0xc01c_0000_0000_0000,
                    0xc022_0000_0000_0000,
                    0xc020_0000_0000_0000,
                    0xc000_0000_0000_0000,
                ],
                &[
                    0x401c_0000_0000_0000,
                    0xc020_0000_0000_0000,
                    0x3ff0_0000_0000_0000,
                    0x4010_0000_0000_0000,
                ],
            ],
            1,
            1,
            0x3fc0_f0f0_f0f0_f0ec,
        );
        assert_minverse_cell(
            &[
                &[
                    0xc01f_aa3a_69fc_19b5,
                    0x3ff3_10d3_9dc5_798c,
                    0xc002_ce56_c460_6f51,
                ],
                &[
                    0xbfee_b3cf_ae57_09d4,
                    0x3fdc_d439_29f4_0a74,
                    0xc00e_dcfe_0f54_118b,
                ],
                &[
                    0x400f_bd84_c716_d36b,
                    0x4006_f3d0_8c4d_2e61,
                    0xc001_5fe3_a453_2caf,
                ],
            ],
            0,
            1,
            0x3fa7_aa39_bab6_6cda,
        );
        assert_minverse_cell(
            &[
                &[
                    0x3ff0_0000_0000_05bf,
                    0xbd59_c48b_eccd_f026,
                    0xbd4e_5336_a5a8_baef,
                ],
                &[
                    0xbd56_1e4e_9f2c_d5ef,
                    0x3ff0_0000_0000_08d1,
                    0x3d7e_bc65_241c_b50f,
                ],
                &[
                    0x3d7b_fd45_ca17_36d3,
                    0x3d56_5979_7981_2e9b,
                    0x3fef_ffff_ffff_e9bb,
                ],
            ],
            0,
            1,
            0x3d59_c48b_eccd_d367,
        );
        assert_minverse_cell(
            &[
                &[0x3ff0_0000_0000_c070, 0xbd7b_f5e9_c250_7848],
                &[0x3d71_29bd_ebc3_c9b3, 0x3fef_ffff_ffff_1417],
            ],
            0,
            1,
            0x3d7b_f5e9_c24f_f61e,
        );
        assert_minverse_cell(
            &[
                &[0x3fd8_80d9_cde9_9b2b, 0xc018_7fb6_be08_1b2e],
                &[0x0000_0000_0000_0000, 0xc01b_587d_2cea_ea89],
            ],
            1,
            0,
            0x0000_0000_0000_0000,
        );
    }

    #[test]
    fn mmult_matches_excel_seed_rows() {
        let got = eval_mmult_surface(
            &[
                (CalcValue::array(
                    CalcArray::from_rows(vec![
                        vec![CalcValue::number(1.0), CalcValue::number(2.0)],
                        vec![CalcValue::number(3.0), CalcValue::number(4.0)],
                    ])
                    .unwrap(),
                )),
                (CalcValue::array(
                    CalcArray::from_rows(vec![
                        vec![CalcValue::number(5.0)],
                        vec![CalcValue::number(6.0)],
                    ])
                    .unwrap(),
                )),
            ],
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(CalcValue::array(
                CalcArray::from_rows(vec![
                    vec![CalcValue::number(17.0)],
                    vec![CalcValue::number(39.0)],
                ])
                .unwrap(),
            ))
        );
    }

    #[test]
    fn munit_matches_excel_seed_rows() {
        let got = eval_munit_surface(&[(CalcValue::number(3.0))], &NoResolver);
        assert_eq!(
            got,
            Ok(CalcValue::array(
                CalcArray::from_rows(vec![
                    vec![
                        CalcValue::number(1.0),
                        CalcValue::number(0.0),
                        CalcValue::number(0.0),
                    ],
                    vec![
                        CalcValue::number(0.0),
                        CalcValue::number(1.0),
                        CalcValue::number(0.0),
                    ],
                    vec![
                        CalcValue::number(0.0),
                        CalcValue::number(0.0),
                        CalcValue::number(1.0),
                    ],
                ])
                .unwrap(),
            ))
        );
    }

    #[test]
    fn munit_one_preserves_array_value() {
        let got = eval_munit_surface(&[(CalcValue::number(1.0))], &NoResolver);
        assert_eq!(
            got,
            Ok(CalcValue::array(
                CalcArray::from_rows(vec![vec![CalcValue::number(1.0)]],).unwrap()
            ))
        );
    }

    #[test]
    fn munit_truncates_and_coerces_text_numeric() {
        let got = eval_munit_surface(
            &[(CalcValue::text(ExcelText::from_utf16_code_units(
                "2.9".encode_utf16().collect(),
            )))],
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(CalcValue::array(
                CalcArray::from_rows(vec![
                    vec![CalcValue::number(1.0), CalcValue::number(0.0)],
                    vec![CalcValue::number(0.0), CalcValue::number(1.0)],
                ])
                .unwrap(),
            ))
        );
    }

    #[test]
    fn singular_inverse_maps_to_num() {
        let got = eval_minverse_surface(
            &[(CalcValue::array(
                CalcArray::from_rows(vec![
                    vec![CalcValue::number(1.0), CalcValue::number(2.0)],
                    vec![CalcValue::number(2.0), CalcValue::number(4.0)],
                ])
                .unwrap(),
            ))],
            &NoResolver,
        );
        assert_eq!(got, Err(MatrixEvalError::Domain(WorksheetErrorCode::Num)));
    }

    #[test]
    fn nonsquare_inputs_map_to_value() {
        let nonsquare = CalcValue::array(
            CalcArray::from_rows(vec![
                vec![
                    CalcValue::number(1.0),
                    CalcValue::number(2.0),
                    CalcValue::number(3.0),
                ],
                vec![
                    CalcValue::number(4.0),
                    CalcValue::number(5.0),
                    CalcValue::number(6.0),
                ],
            ])
            .unwrap(),
        );
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
                (CalcValue::array(
                    CalcArray::from_rows(vec![
                        vec![CalcValue::logical(true), CalcValue::number(2.0)],
                        vec![CalcValue::number(3.0), CalcValue::number(4.0)],
                    ])
                    .unwrap(),
                )),
                (CalcValue::number(2.0)),
            ],
            &NoResolver,
        );
        assert_eq!(got, Err(MatrixEvalError::Domain(WorksheetErrorCode::Value)));
    }

    #[test]
    fn scalar_numeric_inputs_preserve_one_by_one_matrix_results_as_arrays() {
        assert_eq!(
            eval_mdeterm_surface(&[(CalcValue::number(5.0))], &NoResolver),
            Ok(CalcValue::number(5.0))
        );
        assert_eq!(
            eval_minverse_surface(&[(CalcValue::number(5.0))], &NoResolver),
            Ok(CalcValue::array(
                CalcArray::from_rows(vec![vec![CalcValue::number(0.2)]],).unwrap()
            ))
        );
        assert_eq!(
            eval_mmult_surface(
                &[(CalcValue::number(5.0)), (CalcValue::number(2.0)),],
                &NoResolver,
            ),
            Ok(CalcValue::array(
                CalcArray::from_rows(vec![vec![CalcValue::number(10.0)]],).unwrap()
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

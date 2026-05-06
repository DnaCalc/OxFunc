use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::resolver::{ReferenceResolver, resolve_eval_value};
use crate::value::{ArrayCellValue, CallArgValue, EvalValue, WorksheetErrorCode};

const SUMPRODUCT_BASE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.SUMPRODUCT_BASE",
    arity: Arity {
        min: 1,
        max: usize::MAX,
    },
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

pub const SUMPRODUCT_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.SUMPRODUCT",
    ..SUMPRODUCT_BASE_META
};

pub const SUMX2MY2_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.SUMX2MY2",
    arity: Arity::exact(2),
    ..SUMPRODUCT_BASE_META
};

pub const SUMX2PY2_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.SUMX2PY2",
    arity: Arity::exact(2),
    ..SUMPRODUCT_BASE_META
};

pub const SUMXMY2_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.SUMXMY2",
    arity: Arity::exact(2),
    ..SUMPRODUCT_BASE_META
};

pub const SERIESSUM_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.SERIESSUM",
    arity: Arity::exact(4),
    ..SUMPRODUCT_BASE_META
};

#[derive(Debug, Clone, PartialEq)]
pub enum SumproductEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
    Domain(WorksheetErrorCode),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RectShape {
    rows: usize,
    cols: usize,
}

#[derive(Debug, Clone, PartialEq)]
struct NumericRect {
    shape: RectShape,
    values: Vec<f64>,
}

#[derive(Debug, Clone, PartialEq)]
struct OptionalNumericRect {
    shape: RectShape,
    values: Vec<Option<f64>>,
}

fn scalar_text_number(value: &EvalValue) -> Result<f64, SumproductEvalError> {
    match value {
        EvalValue::Number(n) => Ok(*n),
        EvalValue::Text(t) => t
            .to_string_lossy()
            .trim()
            .parse::<f64>()
            .ok()
            .ok_or(SumproductEvalError::Domain(WorksheetErrorCode::Value)),
        EvalValue::Error(code) => Err(SumproductEvalError::Domain(*code)),
        EvalValue::Logical(_)
        | EvalValue::Array(_)
        | EvalValue::Reference(_)
        | EvalValue::Lambda(_) => Err(SumproductEvalError::Domain(WorksheetErrorCode::Value)),
    }
}

fn resolve_eval(
    arg: &CallArgValue,
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, SumproductEvalError> {
    match arg {
        CallArgValue::Reference(r) | CallArgValue::Eval(EvalValue::Reference(r)) => {
            let resolved = resolve_eval_value(resolver, r)
                .map_err(|e| SumproductEvalError::Coercion(CoercionError::RefResolution(e)))?;
            resolve_eval(&CallArgValue::Eval(resolved), resolver)
        }
        CallArgValue::Eval(v) => Ok(v.clone()),
        CallArgValue::MissingArg | CallArgValue::EmptyCell => {
            Err(SumproductEvalError::Domain(WorksheetErrorCode::Value))
        }
    }
}

fn to_sumproduct_rect(
    arg: &CallArgValue,
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<NumericRect, SumproductEvalError> {
    let eval = resolve_eval(arg, resolver)?;
    match eval {
        EvalValue::Array(array) => {
            let shape = array.shape();
            let mut values = Vec::with_capacity(shape.rows * shape.cols);
            for cell in array.iter_row_major() {
                values.push(match cell {
                    ArrayCellValue::Number(n) => *n,
                    ArrayCellValue::Error(code) => return Err(SumproductEvalError::Domain(*code)),
                    ArrayCellValue::Text(_)
                    | ArrayCellValue::Logical(_)
                    | ArrayCellValue::EmptyCell => 0.0,
                });
            }
            Ok(NumericRect {
                shape: RectShape {
                    rows: shape.rows,
                    cols: shape.cols,
                },
                values,
            })
        }
        EvalValue::Number(n) => Ok(NumericRect {
            shape: RectShape { rows: 1, cols: 1 },
            values: vec![n],
        }),
        EvalValue::Text(_) | EvalValue::Logical(_) => Ok(NumericRect {
            shape: RectShape { rows: 1, cols: 1 },
            values: vec![0.0],
        }),
        EvalValue::Error(code) => Err(SumproductEvalError::Domain(code)),
        EvalValue::Reference(_) | EvalValue::Lambda(_) => {
            Err(SumproductEvalError::Domain(WorksheetErrorCode::Value))
        }
    }
}

fn to_optional_numeric_rect(
    arg: &CallArgValue,
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<OptionalNumericRect, SumproductEvalError> {
    let eval = resolve_eval(arg, resolver)?;
    match eval {
        EvalValue::Array(array) => {
            let shape = array.shape();
            let mut values = Vec::with_capacity(shape.rows * shape.cols);
            for cell in array.iter_row_major() {
                values.push(match cell {
                    ArrayCellValue::Number(n) => Some(*n),
                    ArrayCellValue::Error(code) => return Err(SumproductEvalError::Domain(*code)),
                    ArrayCellValue::Text(_)
                    | ArrayCellValue::Logical(_)
                    | ArrayCellValue::EmptyCell => None,
                });
            }
            Ok(OptionalNumericRect {
                shape: RectShape {
                    rows: shape.rows,
                    cols: shape.cols,
                },
                values,
            })
        }
        EvalValue::Number(n) => Ok(OptionalNumericRect {
            shape: RectShape { rows: 1, cols: 1 },
            values: vec![Some(n)],
        }),
        EvalValue::Text(_) | EvalValue::Logical(_) => Ok(OptionalNumericRect {
            shape: RectShape { rows: 1, cols: 1 },
            values: vec![None],
        }),
        EvalValue::Error(code) => Err(SumproductEvalError::Domain(code)),
        EvalValue::Reference(_) | EvalValue::Lambda(_) => {
            Err(SumproductEvalError::Domain(WorksheetErrorCode::Value))
        }
    }
}

fn shapes_match<T>(rects: &[T], shape_of: impl Fn(&T) -> RectShape) -> bool {
    let Some(first) = rects.first() else {
        return true;
    };
    let base = shape_of(first);
    rects.iter().all(|r| shape_of(r) == base)
}

fn eval_sumproduct(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, SumproductEvalError> {
    if !SUMPRODUCT_META.arity.accepts(args.len()) {
        return Err(SumproductEvalError::ArityMismatch {
            expected_min: SUMPRODUCT_META.arity.min,
            expected_max: SUMPRODUCT_META.arity.max,
            actual: args.len(),
        });
    }
    let rects = args
        .iter()
        .map(|arg| to_sumproduct_rect(arg, resolver))
        .collect::<Result<Vec<_>, _>>()?;
    if !shapes_match(&rects, |r| r.shape) {
        return Err(SumproductEvalError::Domain(WorksheetErrorCode::Value));
    }
    let len = rects[0].values.len();
    let mut sum = 0.0;
    for idx in 0..len {
        let mut product = 1.0;
        for rect in &rects {
            product *= rect.values[idx];
        }
        sum += product;
    }
    Ok(EvalValue::Number(sum))
}

fn eval_sumx_pairwise(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
    kernel: impl Fn(f64, f64) -> f64,
) -> Result<EvalValue, SumproductEvalError> {
    let left = to_optional_numeric_rect(&args[0], resolver)?;
    let right = to_optional_numeric_rect(&args[1], resolver)?;
    if left.shape != right.shape {
        return Err(SumproductEvalError::Domain(WorksheetErrorCode::NA));
    }
    let mut seen = false;
    let mut sum = 0.0;
    for idx in 0..left.values.len() {
        if let (Some(x), Some(y)) = (left.values[idx], right.values[idx]) {
            seen = true;
            sum += kernel(x, y);
        }
    }
    if !seen {
        return Err(SumproductEvalError::Domain(WorksheetErrorCode::Div0));
    }
    Ok(EvalValue::Number(sum))
}

fn strict_coefficients(
    arg: &CallArgValue,
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<Vec<f64>, SumproductEvalError> {
    let eval = resolve_eval(arg, resolver)?;
    match eval {
        EvalValue::Array(array) => {
            let mut values = Vec::with_capacity(array.shape().rows * array.shape().cols);
            for cell in array.iter_row_major() {
                values.push(match cell {
                    ArrayCellValue::Number(n) => *n,
                    ArrayCellValue::Error(code) => return Err(SumproductEvalError::Domain(*code)),
                    ArrayCellValue::Text(_)
                    | ArrayCellValue::Logical(_)
                    | ArrayCellValue::EmptyCell => {
                        return Err(SumproductEvalError::Domain(WorksheetErrorCode::Value));
                    }
                });
            }
            Ok(values)
        }
        EvalValue::Number(n) => Ok(vec![n]),
        EvalValue::Error(code) => Err(SumproductEvalError::Domain(code)),
        EvalValue::Text(_)
        | EvalValue::Logical(_)
        | EvalValue::Reference(_)
        | EvalValue::Lambda(_) => Err(SumproductEvalError::Domain(WorksheetErrorCode::Value)),
    }
}

pub fn eval_sumproduct_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, SumproductEvalError> {
    eval_sumproduct(args, resolver)
}

pub fn eval_sumx2my2_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, SumproductEvalError> {
    if !SUMX2MY2_META.arity.accepts(args.len()) {
        return Err(SumproductEvalError::ArityMismatch {
            expected_min: 2,
            expected_max: 2,
            actual: args.len(),
        });
    }
    eval_sumx_pairwise(args, resolver, |x, y| x * x - y * y)
}

pub fn eval_sumx2py2_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, SumproductEvalError> {
    if !SUMX2PY2_META.arity.accepts(args.len()) {
        return Err(SumproductEvalError::ArityMismatch {
            expected_min: 2,
            expected_max: 2,
            actual: args.len(),
        });
    }
    eval_sumx_pairwise(args, resolver, |x, y| x * x + y * y)
}

pub fn eval_sumxmy2_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, SumproductEvalError> {
    if !SUMXMY2_META.arity.accepts(args.len()) {
        return Err(SumproductEvalError::ArityMismatch {
            expected_min: 2,
            expected_max: 2,
            actual: args.len(),
        });
    }
    eval_sumx_pairwise(args, resolver, |x, y| (x - y) * (x - y))
}

pub fn eval_seriessum_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, SumproductEvalError> {
    if !SERIESSUM_META.arity.accepts(args.len()) {
        return Err(SumproductEvalError::ArityMismatch {
            expected_min: 4,
            expected_max: 4,
            actual: args.len(),
        });
    }
    let x = scalar_text_number(&resolve_eval(&args[0], resolver)?)?;
    let n = scalar_text_number(&resolve_eval(&args[1], resolver)?)?;
    let m = scalar_text_number(&resolve_eval(&args[2], resolver)?)?;
    let coeffs = strict_coefficients(&args[3], resolver)?;
    let mut sum = 0.0;
    for (idx, coefficient) in coeffs.into_iter().enumerate() {
        sum += coefficient * x.powf(n + (idx as f64) * m);
    }
    Ok(EvalValue::Number(sum))
}

pub fn map_sumproduct_error_to_ws(err: &SumproductEvalError) -> WorksheetErrorCode {
    match err {
        SumproductEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        SumproductEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        SumproductEvalError::Coercion(_) => WorksheetErrorCode::Value,
        SumproductEvalError::Domain(code) => *code,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::{RefResolutionError, ResolverCapabilities};
    use crate::value::{ExcelText, ReferenceKind, ReferenceLike};
    use std::collections::HashMap;

    struct MockResolver {
        resolved_values: HashMap<String, EvalValue>,
    }

    impl MockResolver {
        fn empty() -> Self {
            Self {
                resolved_values: HashMap::new(),
            }
        }

        fn with_binding(target: &str, value: EvalValue) -> Self {
            let mut resolved_values = HashMap::new();
            resolved_values.insert(target.to_string(), value);
            Self { resolved_values }
        }

        fn with_bindings(bindings: Vec<(&str, EvalValue)>) -> Self {
            Self {
                resolved_values: bindings
                    .into_iter()
                    .map(|(target, value)| (target.to_string(), value))
                    .collect(),
            }
        }
    }

    impl ReferenceResolver for MockResolver {
        fn capabilities(&self) -> ResolverCapabilities {
            ResolverCapabilities::permissive_local()
        }

        fn resolve_reference(
            &self,
            reference: &ReferenceLike,
        ) -> Result<EvalValue, RefResolutionError> {
            self.resolved_values.get(&reference.target).cloned().ok_or(
                RefResolutionError::UnresolvedReference {
                    target: reference.target.clone(),
                },
            )
        }
    }

    fn array(rows: Vec<Vec<ArrayCellValue>>) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Array(
            crate::value::EvalArray::from_rows(rows).unwrap(),
        ))
    }

    fn eval_array(rows: Vec<Vec<ArrayCellValue>>) -> EvalValue {
        EvalValue::Array(crate::value::EvalArray::from_rows(rows).unwrap())
    }

    fn reference(target: &str) -> CallArgValue {
        CallArgValue::Reference(ReferenceLike {
            kind: ReferenceKind::Area,
            target: target.to_string(),
        })
    }

    #[test]
    fn sumproduct_matches_excel_seed_rows() {
        let got = eval_sumproduct_surface(
            &[
                array(vec![vec![
                    ArrayCellValue::Number(1.0),
                    ArrayCellValue::Number(2.0),
                ]]),
                array(vec![vec![
                    ArrayCellValue::Number(3.0),
                    ArrayCellValue::Number(4.0),
                ]]),
            ],
            &MockResolver::empty(),
        );
        assert_eq!(got, Ok(EvalValue::Number(11.0)));
    }

    #[test]
    fn sumproduct_coerces_non_numeric_array_cells_to_zero() {
        let got = eval_sumproduct_surface(
            &[
                array(vec![vec![
                    ArrayCellValue::Number(1.0),
                    ArrayCellValue::Logical(true),
                ]]),
                array(vec![vec![
                    ArrayCellValue::Number(3.0),
                    ArrayCellValue::Number(4.0),
                ]]),
            ],
            &MockResolver::empty(),
        );
        assert_eq!(got, Ok(EvalValue::Number(3.0)));
    }

    #[test]
    fn sumproduct_resolves_reference_arrays_and_propagates_errors() {
        let resolver = MockResolver::with_bindings(vec![
            (
                "A1:A2",
                eval_array(vec![vec![
                    ArrayCellValue::Number(1.0),
                    ArrayCellValue::Text(ExcelText::from_interop_assignment("x")),
                ]]),
            ),
            (
                "B1:B2",
                eval_array(vec![vec![
                    ArrayCellValue::Number(3.0),
                    ArrayCellValue::Number(4.0),
                ]]),
            ),
        ]);
        let got = eval_sumproduct_surface(&[reference("A1:A2"), reference("B1:B2")], &resolver);
        assert_eq!(got, Ok(EvalValue::Number(3.0)));

        let resolver = MockResolver::with_bindings(vec![
            (
                "A1:A2",
                eval_array(vec![vec![
                    ArrayCellValue::Number(1.0),
                    ArrayCellValue::Error(WorksheetErrorCode::NA),
                ]]),
            ),
            (
                "B1:B2",
                eval_array(vec![vec![
                    ArrayCellValue::Number(3.0),
                    ArrayCellValue::Number(4.0),
                ]]),
            ),
        ]);
        let got = eval_sumproduct_surface(&[reference("A1:A2"), reference("B1:B2")], &resolver);
        assert_eq!(
            got,
            Err(SumproductEvalError::Domain(WorksheetErrorCode::NA))
        );
    }

    #[test]
    fn sumproduct_rejects_shape_mismatch() {
        let got = eval_sumproduct_surface(
            &[
                array(vec![vec![
                    ArrayCellValue::Number(1.0),
                    ArrayCellValue::Number(2.0),
                ]]),
                CallArgValue::Eval(EvalValue::Number(3.0)),
            ],
            &MockResolver::empty(),
        );
        assert_eq!(
            got,
            Err(SumproductEvalError::Domain(WorksheetErrorCode::Value))
        );
    }

    #[test]
    fn sumproduct_single_array_argument_sums_elements() {
        let got = eval_sumproduct_surface(
            &[array(vec![vec![
                ArrayCellValue::Number(1.0),
                ArrayCellValue::Number(4.0),
                ArrayCellValue::Number(9.0),
            ]])],
            &MockResolver::empty(),
        );
        assert_eq!(got, Ok(EvalValue::Number(14.0)));
    }

    #[test]
    fn sumx_family_matches_excel_seed_rows() {
        assert_eq!(
            eval_sumx2my2_surface(
                &[
                    CallArgValue::Eval(EvalValue::Number(5.0)),
                    CallArgValue::Eval(EvalValue::Number(6.0))
                ],
                &MockResolver::empty()
            ),
            Ok(EvalValue::Number(-11.0))
        );
        assert_eq!(
            eval_sumx2py2_surface(
                &[
                    array(vec![vec![
                        ArrayCellValue::Number(1.0),
                        ArrayCellValue::Number(2.0)
                    ]]),
                    array(vec![vec![
                        ArrayCellValue::Number(3.0),
                        ArrayCellValue::Number(4.0)
                    ]]),
                ],
                &MockResolver::empty()
            ),
            Ok(EvalValue::Number(30.0))
        );
        assert_eq!(
            eval_sumxmy2_surface(
                &[
                    array(vec![vec![
                        ArrayCellValue::Number(1.0),
                        ArrayCellValue::Number(2.0)
                    ]]),
                    array(vec![vec![
                        ArrayCellValue::Number(3.0),
                        ArrayCellValue::Number(4.0)
                    ]]),
                ],
                &MockResolver::empty()
            ),
            Ok(EvalValue::Number(8.0))
        );
    }

    #[test]
    fn sumx_family_resolves_reference_pairs_and_ignores_non_numeric_positions() {
        let resolver = MockResolver::with_bindings(vec![
            (
                "A1:A2",
                eval_array(vec![vec![
                    ArrayCellValue::Number(1.0),
                    ArrayCellValue::Logical(true),
                ]]),
            ),
            (
                "B1:B2",
                eval_array(vec![vec![
                    ArrayCellValue::Number(3.0),
                    ArrayCellValue::Number(4.0),
                ]]),
            ),
        ]);
        let got = eval_sumx2my2_surface(&[reference("A1:A2"), reference("B1:B2")], &resolver);
        assert_eq!(got, Ok(EvalValue::Number(-8.0)));
    }

    #[test]
    fn sumx_family_ignores_non_numeric_pairs_and_div0_when_none_survive() {
        let got = eval_sumx2my2_surface(
            &[
                array(vec![vec![
                    ArrayCellValue::Number(1.0),
                    ArrayCellValue::Logical(true),
                ]]),
                array(vec![vec![
                    ArrayCellValue::Number(3.0),
                    ArrayCellValue::Number(4.0),
                ]]),
            ],
            &MockResolver::empty(),
        );
        assert_eq!(got, Ok(EvalValue::Number(-8.0)));

        let got = eval_sumx2my2_surface(
            &[
                CallArgValue::Eval(EvalValue::Logical(true)),
                CallArgValue::Eval(EvalValue::Number(6.0)),
            ],
            &MockResolver::empty(),
        );
        assert_eq!(
            got,
            Err(SumproductEvalError::Domain(WorksheetErrorCode::Div0))
        );
    }

    #[test]
    fn sumx_family_reports_shape_mismatch_as_na() {
        let got = eval_sumx2my2_surface(
            &[
                array(vec![vec![
                    ArrayCellValue::Number(1.0),
                    ArrayCellValue::Number(2.0),
                ]]),
                CallArgValue::Eval(EvalValue::Number(3.0)),
            ],
            &MockResolver::empty(),
        );
        assert_eq!(
            got,
            Err(SumproductEvalError::Domain(WorksheetErrorCode::NA))
        );
    }

    #[test]
    fn seriessum_matches_excel_seed_rows() {
        let got = eval_seriessum_surface(
            &[
                CallArgValue::Eval(EvalValue::Number(2.0)),
                CallArgValue::Eval(EvalValue::Number(1.0)),
                CallArgValue::Eval(EvalValue::Number(2.0)),
                array(vec![vec![
                    ArrayCellValue::Number(1.0),
                    ArrayCellValue::Number(2.0),
                    ArrayCellValue::Number(3.0),
                ]]),
            ],
            &MockResolver::empty(),
        );
        assert_eq!(got, Ok(EvalValue::Number(114.0)));
    }

    #[test]
    fn seriessum_allows_numeric_text_scalars_but_not_logicals() {
        let got = eval_seriessum_surface(
            &[
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_interop_assignment("2"))),
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_interop_assignment("1"))),
                CallArgValue::Eval(EvalValue::Number(2.0)),
                CallArgValue::Eval(EvalValue::Number(3.0)),
            ],
            &MockResolver::empty(),
        );
        assert_eq!(got, Ok(EvalValue::Number(6.0)));

        let got = eval_seriessum_surface(
            &[
                CallArgValue::Eval(EvalValue::Logical(true)),
                CallArgValue::Eval(EvalValue::Number(1.0)),
                CallArgValue::Eval(EvalValue::Number(2.0)),
                CallArgValue::Eval(EvalValue::Number(3.0)),
            ],
            &MockResolver::empty(),
        );
        assert_eq!(
            got,
            Err(SumproductEvalError::Domain(WorksheetErrorCode::Value))
        );
    }

    #[test]
    fn seriessum_resolves_reference_coefficients_row_major() {
        let resolver = MockResolver::with_binding(
            "A1:B2",
            eval_array(vec![
                vec![ArrayCellValue::Number(1.0), ArrayCellValue::Number(2.0)],
                vec![ArrayCellValue::Number(3.0), ArrayCellValue::Number(4.0)],
            ]),
        );
        let got = eval_seriessum_surface(
            &[
                CallArgValue::Eval(EvalValue::Number(2.0)),
                CallArgValue::Eval(EvalValue::Number(0.0)),
                CallArgValue::Eval(EvalValue::Number(1.0)),
                reference("A1:B2"),
            ],
            &resolver,
        );
        assert_eq!(got, Ok(EvalValue::Number(49.0)));
    }

    #[test]
    fn seriessum_rejects_non_numeric_coefficients() {
        let got = eval_seriessum_surface(
            &[
                CallArgValue::Eval(EvalValue::Number(2.0)),
                CallArgValue::Eval(EvalValue::Number(1.0)),
                CallArgValue::Eval(EvalValue::Number(2.0)),
                array(vec![vec![
                    ArrayCellValue::Number(1.0),
                    ArrayCellValue::Text(ExcelText::from_interop_assignment("2")),
                ]]),
            ],
            &MockResolver::empty(),
        );
        assert_eq!(
            got,
            Err(SumproductEvalError::Domain(WorksheetErrorCode::Value))
        );
    }
}

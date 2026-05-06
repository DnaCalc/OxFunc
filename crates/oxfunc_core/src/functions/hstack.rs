use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{PreparedArgValue, run_values_only_prepared};
use crate::resolver::ReferenceResolver;
use crate::value::{
    ArrayCellValue, ArrayShape, CallArgValue, EvalArray, EvalValue, WorksheetErrorCode,
};

pub const HSTACK_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.HSTACK",
    arity: Arity { min: 1, max: 255 },
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

#[derive(Debug, Clone, PartialEq)]
pub enum HstackEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Preparation(CoercionError),
}

fn scalar_cell(arg: &PreparedArgValue) -> ArrayCellValue {
    match arg {
        PreparedArgValue::Eval(EvalValue::Number(n)) => ArrayCellValue::Number(*n),
        PreparedArgValue::Eval(EvalValue::Text(t)) => ArrayCellValue::Text(t.clone()),
        PreparedArgValue::Eval(EvalValue::Logical(b)) => ArrayCellValue::Logical(*b),
        PreparedArgValue::Eval(EvalValue::Error(code)) => ArrayCellValue::Error(*code),
        PreparedArgValue::Eval(EvalValue::Reference(_))
        | PreparedArgValue::Eval(EvalValue::Lambda(_)) => {
            ArrayCellValue::Error(WorksheetErrorCode::Value)
        }
        PreparedArgValue::Eval(EvalValue::Array(_)) => unreachable!(),
        PreparedArgValue::MissingArg | PreparedArgValue::EmptyCell => ArrayCellValue::EmptyCell,
    }
}

enum HstackArgSource<'a> {
    Array(&'a EvalArray),
    Scalar(ArrayCellValue),
}

impl<'a> HstackArgSource<'a> {
    fn new(arg: &'a PreparedArgValue) -> Self {
        match arg {
            PreparedArgValue::Eval(EvalValue::Array(array)) => Self::Array(array),
            other => Self::Scalar(scalar_cell(other)),
        }
    }

    fn shape(&self) -> ArrayShape {
        match self {
            Self::Array(array) => array.shape(),
            Self::Scalar(_) => ArrayShape { rows: 1, cols: 1 },
        }
    }

    fn get(&self, row: usize, col: usize) -> Option<&ArrayCellValue> {
        match self {
            Self::Array(array) => array.get(row, col),
            Self::Scalar(cell) if row == 0 && col == 0 => Some(cell),
            Self::Scalar(_) => None,
        }
    }
}

pub fn eval_hstack_adapter_prepared(
    args: &[PreparedArgValue],
) -> Result<EvalValue, HstackEvalError> {
    let argc = args.len();
    if !HSTACK_META.arity.accepts(argc) {
        return Err(HstackEvalError::ArityMismatch {
            expected_min: HSTACK_META.arity.min,
            expected_max: HSTACK_META.arity.max,
            actual: argc,
        });
    }

    let sources: Vec<HstackArgSource<'_>> = args.iter().map(HstackArgSource::new).collect();
    let rows = sources
        .iter()
        .map(|source| source.shape().rows)
        .max()
        .unwrap_or(1);
    let cols = sources.iter().map(|source| source.shape().cols).sum();

    Ok(EvalValue::Array(
        EvalArray::from_cells_iter(
            ArrayShape { rows, cols },
            (0..rows).flat_map(|row| {
                sources.iter().flat_map(move |source| {
                    (0..source.shape().cols).map(move |col| {
                        source
                            .get(row, col)
                            .cloned()
                            .unwrap_or(ArrayCellValue::Error(WorksheetErrorCode::NA))
                    })
                })
            }),
        )
        .expect("hstack dimensions are computed"),
    ))
}

pub fn eval_hstack_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, HstackEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_hstack_adapter_prepared,
        HstackEvalError::Preparation,
    )
}

pub fn map_hstack_error_to_ws(e: &HstackEvalError) -> WorksheetErrorCode {
    match e {
        HstackEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        HstackEvalError::Preparation(CoercionError::WorksheetError(code)) => *code,
        HstackEvalError::Preparation(_) => WorksheetErrorCode::Value,
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
    fn eval_hstack_combines_scalar_and_array_shapes() {
        let got = eval_hstack_surface(
            &[
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![
                        vec![ArrayCellValue::Number(1.0), ArrayCellValue::Number(2.0)],
                        vec![ArrayCellValue::Number(3.0), ArrayCellValue::Number(4.0)],
                    ])
                    .unwrap(),
                )),
                CallArgValue::Eval(EvalValue::Number(1.0)),
            ],
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![
                        ArrayCellValue::Number(1.0),
                        ArrayCellValue::Number(2.0),
                        ArrayCellValue::Number(1.0),
                    ],
                    vec![
                        ArrayCellValue::Number(3.0),
                        ArrayCellValue::Number(4.0),
                        ArrayCellValue::Error(WorksheetErrorCode::NA),
                    ],
                ])
                .unwrap()
            ))
        );
    }

    #[test]
    fn eval_hstack_preserves_empty_scalar_cells() {
        let got = eval_hstack_surface(
            &[
                CallArgValue::EmptyCell,
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
                    "x".encode_utf16().collect(),
                ))),
            ],
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::EmptyCell,
                    ArrayCellValue::Text(ExcelText::from_utf16_code_units(
                        "x".encode_utf16().collect(),
                    )),
                ]])
                .unwrap()
            ))
        );
    }

    #[test]
    fn eval_hstack_pads_shorter_scalar_argument_with_na() {
        let got = eval_hstack_surface(
            &[
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![
                        vec![ArrayCellValue::Number(1.0)],
                        vec![ArrayCellValue::Number(2.0)],
                    ])
                    .unwrap(),
                )),
                CallArgValue::Eval(EvalValue::Text(
                    ExcelText::from_utf16_code_units(Vec::new()),
                )),
            ],
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![
                        ArrayCellValue::Number(1.0),
                        ArrayCellValue::Text(ExcelText::from_utf16_code_units(Vec::new())),
                    ],
                    vec![
                        ArrayCellValue::Number(2.0),
                        ArrayCellValue::Error(WorksheetErrorCode::NA),
                    ],
                ])
                .unwrap()
            ))
        );
    }
}

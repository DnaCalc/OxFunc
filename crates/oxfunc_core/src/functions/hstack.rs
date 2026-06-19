use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::run_values_only_prepared;
use crate::resolver::ReferenceSystemProvider;
use crate::value::{ArrayShape, CalcArray, CalcValue, CoreValue, WorksheetErrorCode};

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
    real_result_policy: FunctionMeta::DEFAULT_REAL_RESULT_POLICY,
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

fn scalar_cell(arg: &CalcValue) -> CalcValue {
    match arg.core() {
        CoreValue::Number(_) | CoreValue::Text(_) | CoreValue::Logical(_) | CoreValue::Error(_) => {
            arg.clone()
        }
        CoreValue::Reference(_) => CalcValue::error(WorksheetErrorCode::Value),
        CoreValue::Array(_) => unreachable!(),
        CoreValue::Missing | CoreValue::Empty => CalcValue::empty(),
    }
}

enum HstackArgSource<'a> {
    Array(&'a CalcArray),
    Scalar(CalcValue),
}

impl<'a> HstackArgSource<'a> {
    fn new(arg: &'a CalcValue) -> Self {
        match arg.core() {
            CoreValue::Array(array) => Self::Array(array),
            _ => Self::Scalar(scalar_cell(arg)),
        }
    }

    fn shape(&self) -> ArrayShape {
        match self {
            Self::Array(array) => array.shape(),
            Self::Scalar(_) => ArrayShape { rows: 1, cols: 1 },
        }
    }

    fn get(&self, row: usize, col: usize) -> Option<&CalcValue> {
        match self {
            Self::Array(array) => array.get(row, col),
            Self::Scalar(cell) if row == 0 && col == 0 => Some(cell),
            Self::Scalar(_) => None,
        }
    }
}

pub fn eval_hstack_adapter_prepared(args: &[CalcValue]) -> Result<CalcValue, HstackEvalError> {
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

    Ok(CalcValue::array(
        CalcArray::from_cells_iter(
            ArrayShape { rows, cols },
            (0..rows).flat_map(|row| {
                sources.iter().flat_map(move |source| {
                    (0..source.shape().cols).map(move |col| {
                        source
                            .get(row, col)
                            .cloned()
                            .unwrap_or(CalcValue::error(WorksheetErrorCode::NA))
                    })
                })
            }),
        )
        .expect("hstack dimensions are computed"),
    ))
}

pub fn eval_hstack_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, HstackEvalError> {
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

    #[test]
    fn eval_hstack_combines_scalar_and_array_shapes() {
        let got = eval_hstack_surface(
            &[
                (CalcValue::array(
                    CalcArray::from_rows(vec![
                        vec![CalcValue::number(1.0), CalcValue::number(2.0)],
                        vec![CalcValue::number(3.0), CalcValue::number(4.0)],
                    ])
                    .unwrap(),
                )),
                (CalcValue::number(1.0)),
            ],
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(CalcValue::array(
                CalcArray::from_rows(vec![
                    vec![
                        CalcValue::number(1.0),
                        CalcValue::number(2.0),
                        CalcValue::number(1.0),
                    ],
                    vec![
                        CalcValue::number(3.0),
                        CalcValue::number(4.0),
                        CalcValue::error(WorksheetErrorCode::NA),
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
                CalcValue::empty(),
                (CalcValue::text(ExcelText::from_utf16_code_units(
                    "x".encode_utf16().collect(),
                ))),
            ],
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(CalcValue::array(
                CalcArray::from_rows(vec![vec![
                    CalcValue::empty(),
                    CalcValue::text(ExcelText::from_utf16_code_units(
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
                (CalcValue::array(
                    CalcArray::from_rows(vec![
                        vec![CalcValue::number(1.0)],
                        vec![CalcValue::number(2.0)],
                    ])
                    .unwrap(),
                )),
                (CalcValue::text(ExcelText::from_utf16_code_units(Vec::new()))),
            ],
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(CalcValue::array(
                CalcArray::from_rows(vec![
                    vec![
                        CalcValue::number(1.0),
                        CalcValue::text(ExcelText::from_utf16_code_units(Vec::new())),
                    ],
                    vec![
                        CalcValue::number(2.0),
                        CalcValue::error(WorksheetErrorCode::NA),
                    ],
                ])
                .unwrap()
            ))
        );
    }
}

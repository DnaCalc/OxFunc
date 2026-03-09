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
        PreparedArgValue::Eval(EvalValue::Reference(_)) | PreparedArgValue::Eval(EvalValue::Lambda(_)) => {
            ArrayCellValue::Error(WorksheetErrorCode::Value)
        }
        PreparedArgValue::Eval(EvalValue::Array(_)) => unreachable!(),
        PreparedArgValue::MissingArg | PreparedArgValue::EmptyCell => ArrayCellValue::EmptyCell,
    }
}

fn materialize_hstack_arg(arg: &PreparedArgValue) -> EvalArray {
    match arg {
        PreparedArgValue::Eval(EvalValue::Array(array)) => array.clone(),
        other => EvalArray::from_scalar(scalar_cell(other)),
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

    let arrays: Vec<EvalArray> = args.iter().map(materialize_hstack_arg).collect();
    let rows = arrays
        .iter()
        .map(|array| array.shape().rows)
        .max()
        .unwrap_or(1);
    let cols = arrays.iter().map(|array| array.shape().cols).sum();

    let mut cells = Vec::with_capacity(rows * cols);
    for row in 0..rows {
        for array in &arrays {
            for col in 0..array.shape().cols {
                let cell = array
                    .get(row, col)
                    .cloned()
                    .unwrap_or(ArrayCellValue::Error(WorksheetErrorCode::NA));
                cells.push(cell);
            }
        }
    }

    Ok(EvalValue::Array(
        EvalArray::new(ArrayShape { rows, cols }, cells).expect("hstack dimensions are computed"),
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
}

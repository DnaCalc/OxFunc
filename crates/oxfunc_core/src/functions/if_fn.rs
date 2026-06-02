use crate::coercion::{CoercionError, coerce_eval_to_number};
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{PreparedValue, prepare_arg_values_only};
use crate::resolver::{ReferenceSystemProvider, resolve_eval_value};
use crate::value::{
    ArrayShape, FunctionArg, FunctionArray, FunctionArrayCell, FunctionValue, WorksheetErrorCode,
};

pub const IF_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.IF",
    arity: Arity { min: 2, max: 3 },
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::RefsVisibleInAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
};

#[derive(Debug, Clone, PartialEq)]
pub enum IfEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    ConditionCoercion(CoercionError),
    BranchPreparation(CoercionError),
}

fn prepared_to_eval_value(prepared: PreparedValue) -> FunctionValue {
    match prepared {
        PreparedValue::Eval(v) => v,
        PreparedValue::MissingArg => FunctionValue::Logical(false),
        PreparedValue::EmptyCell => FunctionValue::Number(0.0),
    }
}

fn eval_condition_cell(cell: &FunctionArrayCell) -> Result<bool, CoercionError> {
    match cell {
        FunctionArrayCell::Logical(b) => Ok(*b),
        FunctionArrayCell::Number(n) => Ok(*n != 0.0),
        FunctionArrayCell::Error(code) => Err(CoercionError::WorksheetError(*code)),
        FunctionArrayCell::Text(text) => Err(CoercionError::NonNumericText(text.to_string_lossy())),
        FunctionArrayCell::EmptyCell => Ok(false),
    }
}

fn scalar_cell_from_eval_value(value: &FunctionValue) -> Result<FunctionArrayCell, CoercionError> {
    match value {
        FunctionValue::Number(n) => Ok(FunctionArrayCell::Number(*n)),
        FunctionValue::Text(t) => Ok(FunctionArrayCell::Text(t.clone())),
        FunctionValue::Logical(b) => Ok(FunctionArrayCell::Logical(*b)),
        FunctionValue::Error(code) => Ok(FunctionArrayCell::Error(*code)),
        FunctionValue::Array(_) | FunctionValue::Reference(_) => {
            Err(CoercionError::UnsupportedValueKind("if_branch_scalar"))
        }
        _ => Err(CoercionError::UnsupportedValueKind("if_branch_scalar")),
    }
}

fn materialize_branch_for_shape(
    value: &FunctionValue,
    shape: ArrayShape,
) -> Result<FunctionArray, CoercionError> {
    match value {
        FunctionValue::Array(array) if array.shape() == shape => Ok(array.clone()),
        FunctionValue::Array(_) => Err(CoercionError::UnsupportedValueKind("if_branch_shape")),
        other => {
            let cell = scalar_cell_from_eval_value(other)?;
            FunctionArray::new(shape, vec![cell; shape.rows * shape.cols])
                .ok_or(CoercionError::UnsupportedValueKind("if_branch_shape"))
        }
    }
}

fn eval_condition_bool(
    arg: &FunctionArg,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<bool, CoercionError> {
    match arg {
        FunctionArg::MissingArg | FunctionArg::EmptyCell => Ok(false),
        FunctionArg::Eval(v) => match v {
            FunctionValue::Logical(b) => Ok(*b),
            FunctionValue::Number(n) => Ok(*n != 0.0),
            _ => {
                let n = coerce_eval_to_number(v, resolver)?;
                Ok(n != 0.0)
            }
        },
        FunctionArg::Reference(r) => {
            let resolved = resolve_eval_value(resolver, r).map_err(CoercionError::RefResolution)?;
            eval_condition_bool(&FunctionArg::Eval(resolved), resolver)
        }
    }
}

fn eval_if_elementwise_surface(
    condition: &FunctionArray,
    true_arg: &FunctionArg,
    false_arg: Option<&FunctionArg>,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<FunctionValue, IfEvalError> {
    let true_value = prepared_to_eval_value(
        prepare_arg_values_only(true_arg, resolver).map_err(IfEvalError::BranchPreparation)?,
    );
    let false_value = match false_arg {
        Some(arg) => prepared_to_eval_value(
            prepare_arg_values_only(arg, resolver).map_err(IfEvalError::BranchPreparation)?,
        ),
        None => FunctionValue::Logical(false),
    };
    let true_array = materialize_branch_for_shape(&true_value, condition.shape())
        .map_err(IfEvalError::BranchPreparation)?;
    let false_array = materialize_branch_for_shape(&false_value, condition.shape())
        .map_err(IfEvalError::BranchPreparation)?;

    let mut cells = Vec::with_capacity(condition.shape().rows * condition.shape().cols);
    for row in 0..condition.shape().rows {
        for col in 0..condition.shape().cols {
            let keep = eval_condition_cell(
                condition
                    .get(row, col)
                    .expect("validated IF condition shape"),
            )
            .map_err(IfEvalError::ConditionCoercion)?;
            let chosen = if keep {
                true_array.get(row, col).expect("validated IF true shape")
            } else {
                false_array.get(row, col).expect("validated IF false shape")
            };
            cells.push(chosen.clone());
        }
    }

    Ok(FunctionValue::Array(
        FunctionArray::new(condition.shape(), cells).expect("validated IF result shape"),
    ))
}

pub fn eval_if_surface(
    args: &[FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<FunctionValue, IfEvalError> {
    let argc = args.len();
    if !IF_META.arity.accepts(argc) {
        return Err(IfEvalError::ArityMismatch {
            expected_min: IF_META.arity.min,
            expected_max: IF_META.arity.max,
            actual: argc,
        });
    }

    if let FunctionArg::Eval(FunctionValue::Array(condition)) = &args[0] {
        return eval_if_elementwise_surface(condition, &args[1], args.get(2), resolver);
    }

    let cond = eval_condition_bool(&args[0], resolver).map_err(IfEvalError::ConditionCoercion)?;

    let branch_arg = if cond {
        &args[1]
    } else if argc >= 3 {
        &args[2]
    } else {
        return Ok(FunctionValue::Logical(false));
    };

    let prepared =
        prepare_arg_values_only(branch_arg, resolver).map_err(IfEvalError::BranchPreparation)?;
    Ok(prepared_to_eval_value(prepared))
}

pub fn map_if_error_to_ws(e: &IfEvalError) -> WorksheetErrorCode {
    match e {
        IfEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        IfEvalError::ConditionCoercion(CoercionError::WorksheetError(code)) => *code,
        IfEvalError::BranchPreparation(CoercionError::WorksheetError(code)) => *code,
        _ => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::ReferenceSystemCapabilities;
    use crate::value::{FunctionArray, FunctionArrayCell};

    struct NoResolver;
    impl ReferenceSystemProvider for NoResolver {
        fn capabilities(&self) -> ReferenceSystemCapabilities {
            ReferenceSystemCapabilities::permissive_local()
        }
        fn dereference(
            &self,
            request: &crate::resolver::ReferenceDereferenceRequest,
        ) -> Result<FunctionValue, crate::resolver::ReferenceResolutionError> {
            let reference = &request.reference;
            Err(
                crate::resolver::ReferenceResolutionError::UnresolvedReference {
                    target: reference.target().to_string(),
                },
            )
        }
    }

    #[test]
    fn eval_if_true_branch_only() {
        let args = vec![
            FunctionArg::Eval(FunctionValue::Logical(true)),
            FunctionArg::Eval(FunctionValue::Number(1.0)),
            FunctionArg::Eval(FunctionValue::Error(WorksheetErrorCode::Div0)),
        ];
        let got = eval_if_surface(&args, &NoResolver);
        assert_eq!(got, Ok(FunctionValue::Number(1.0)));
    }

    #[test]
    fn eval_if_false_branch_only() {
        let args = vec![
            FunctionArg::Eval(FunctionValue::Logical(false)),
            FunctionArg::Eval(FunctionValue::Error(WorksheetErrorCode::Div0)),
            FunctionArg::Eval(FunctionValue::Number(2.0)),
        ];
        let got = eval_if_surface(&args, &NoResolver);
        assert_eq!(got, Ok(FunctionValue::Number(2.0)));
    }

    #[test]
    fn eval_if_missing_false_branch_defaults_false() {
        let args = vec![
            FunctionArg::Eval(FunctionValue::Logical(false)),
            FunctionArg::Eval(FunctionValue::Number(1.0)),
        ];
        let got = eval_if_surface(&args, &NoResolver);
        assert_eq!(got, Ok(FunctionValue::Logical(false)));
    }

    #[test]
    fn eval_if_empty_text_condition_returns_value_error() {
        let args = vec![
            FunctionArg::Eval(FunctionValue::Text(
                crate::value::ExcelText::from_interop_assignment(""),
            )),
            FunctionArg::Eval(FunctionValue::Number(1.0)),
            FunctionArg::Eval(FunctionValue::Number(2.0)),
        ];
        let got = eval_if_surface(&args, &NoResolver);
        assert_eq!(
            got,
            Err(IfEvalError::ConditionCoercion(
                CoercionError::NonNumericText("".to_string())
            ))
        );
    }

    #[test]
    fn eval_if_lifts_array_condition_elementwise() {
        let args = vec![
            FunctionArg::Eval(FunctionValue::Array(
                FunctionArray::from_rows(vec![vec![
                    FunctionArrayCell::Logical(true),
                    FunctionArrayCell::Logical(false),
                    FunctionArrayCell::Logical(true),
                ]])
                .unwrap(),
            )),
            FunctionArg::Eval(FunctionValue::Array(
                FunctionArray::from_rows(vec![vec![
                    FunctionArrayCell::Number(1.0),
                    FunctionArrayCell::Number(2.0),
                    FunctionArrayCell::Number(3.0),
                ]])
                .unwrap(),
            )),
            FunctionArg::Eval(FunctionValue::Number(0.0)),
        ];
        let got = eval_if_surface(&args, &NoResolver);
        assert_eq!(
            got,
            Ok(FunctionValue::Array(
                FunctionArray::from_rows(vec![vec![
                    FunctionArrayCell::Number(1.0),
                    FunctionArrayCell::Number(0.0),
                    FunctionArrayCell::Number(3.0),
                ]])
                .unwrap()
            ))
        );
    }
}

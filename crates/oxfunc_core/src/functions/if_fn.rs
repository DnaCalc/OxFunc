use crate::coercion::{CoercionError, coerce_eval_to_number};
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{PreparedArgValue, prepare_arg_values_only};
use crate::resolver::ReferenceResolver;
use crate::value::{
    ArrayCellValue, ArrayShape, CallArgValue, EvalArray, EvalValue, WorksheetErrorCode,
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

fn prepared_to_eval_value(prepared: PreparedArgValue) -> EvalValue {
    match prepared {
        PreparedArgValue::Eval(v) => v,
        PreparedArgValue::MissingArg => EvalValue::Logical(false),
        PreparedArgValue::EmptyCell => EvalValue::Number(0.0),
    }
}

fn eval_condition_cell(cell: &ArrayCellValue) -> Result<bool, CoercionError> {
    match cell {
        ArrayCellValue::Logical(b) => Ok(*b),
        ArrayCellValue::Number(n) => Ok(*n != 0.0),
        ArrayCellValue::Error(code) => Err(CoercionError::WorksheetError(*code)),
        ArrayCellValue::Text(text) => Err(CoercionError::NonNumericText(text.to_string_lossy())),
        ArrayCellValue::EmptyCell => Ok(false),
    }
}

fn scalar_cell_from_eval_value(value: &EvalValue) -> Result<ArrayCellValue, CoercionError> {
    match value {
        EvalValue::Number(n) => Ok(ArrayCellValue::Number(*n)),
        EvalValue::Text(t) => Ok(ArrayCellValue::Text(t.clone())),
        EvalValue::Logical(b) => Ok(ArrayCellValue::Logical(*b)),
        EvalValue::Error(code) => Ok(ArrayCellValue::Error(*code)),
        EvalValue::Array(_) | EvalValue::Reference(_) | EvalValue::Lambda(_) => {
            Err(CoercionError::UnsupportedValueKind("if_branch_scalar"))
        }
    }
}

fn materialize_branch_for_shape(
    value: &EvalValue,
    shape: ArrayShape,
) -> Result<EvalArray, CoercionError> {
    match value {
        EvalValue::Array(array) if array.shape() == shape => Ok(array.clone()),
        EvalValue::Array(_) => Err(CoercionError::UnsupportedValueKind("if_branch_shape")),
        other => {
            let cell = scalar_cell_from_eval_value(other)?;
            EvalArray::new(shape, vec![cell; shape.rows * shape.cols])
                .ok_or(CoercionError::UnsupportedValueKind("if_branch_shape"))
        }
    }
}

fn eval_condition_bool(
    arg: &CallArgValue,
    resolver: &impl ReferenceResolver,
) -> Result<bool, CoercionError> {
    match arg {
        CallArgValue::MissingArg | CallArgValue::EmptyCell => Ok(false),
        CallArgValue::Eval(v) => match v {
            EvalValue::Logical(b) => Ok(*b),
            EvalValue::Number(n) => Ok(*n != 0.0),
            _ => {
                let n = coerce_eval_to_number(v, resolver)?;
                Ok(n != 0.0)
            }
        },
        CallArgValue::Reference(r) => {
            let resolved = resolver
                .resolve_reference(r)
                .map_err(CoercionError::RefResolution)?;
            eval_condition_bool(&CallArgValue::Eval(resolved), resolver)
        }
    }
}

fn eval_if_elementwise_surface(
    condition: &EvalArray,
    true_arg: &CallArgValue,
    false_arg: Option<&CallArgValue>,
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, IfEvalError> {
    let true_value = prepared_to_eval_value(
        prepare_arg_values_only(true_arg, resolver).map_err(IfEvalError::BranchPreparation)?,
    );
    let false_value = match false_arg {
        Some(arg) => prepared_to_eval_value(
            prepare_arg_values_only(arg, resolver).map_err(IfEvalError::BranchPreparation)?,
        ),
        None => EvalValue::Logical(false),
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

    Ok(EvalValue::Array(
        EvalArray::new(condition.shape(), cells).expect("validated IF result shape"),
    ))
}

pub fn eval_if_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, IfEvalError> {
    let argc = args.len();
    if !IF_META.arity.accepts(argc) {
        return Err(IfEvalError::ArityMismatch {
            expected_min: IF_META.arity.min,
            expected_max: IF_META.arity.max,
            actual: argc,
        });
    }

    if let CallArgValue::Eval(EvalValue::Array(condition)) = &args[0] {
        return eval_if_elementwise_surface(condition, &args[1], args.get(2), resolver);
    }

    let cond = eval_condition_bool(&args[0], resolver).map_err(IfEvalError::ConditionCoercion)?;

    let branch_arg = if cond {
        &args[1]
    } else if argc >= 3 {
        &args[2]
    } else {
        return Ok(EvalValue::Logical(false));
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
    use crate::resolver::{RefResolutionError, ResolverCapabilities};
    use crate::value::{ArrayCellValue, EvalArray, ReferenceLike};

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
    fn eval_if_true_branch_only() {
        let args = vec![
            CallArgValue::Eval(EvalValue::Logical(true)),
            CallArgValue::Eval(EvalValue::Number(1.0)),
            CallArgValue::Eval(EvalValue::Error(WorksheetErrorCode::Div0)),
        ];
        let got = eval_if_surface(&args, &NoResolver);
        assert_eq!(got, Ok(EvalValue::Number(1.0)));
    }

    #[test]
    fn eval_if_false_branch_only() {
        let args = vec![
            CallArgValue::Eval(EvalValue::Logical(false)),
            CallArgValue::Eval(EvalValue::Error(WorksheetErrorCode::Div0)),
            CallArgValue::Eval(EvalValue::Number(2.0)),
        ];
        let got = eval_if_surface(&args, &NoResolver);
        assert_eq!(got, Ok(EvalValue::Number(2.0)));
    }

    #[test]
    fn eval_if_missing_false_branch_defaults_false() {
        let args = vec![
            CallArgValue::Eval(EvalValue::Logical(false)),
            CallArgValue::Eval(EvalValue::Number(1.0)),
        ];
        let got = eval_if_surface(&args, &NoResolver);
        assert_eq!(got, Ok(EvalValue::Logical(false)));
    }

    #[test]
    fn eval_if_empty_text_condition_returns_value_error() {
        let args = vec![
            CallArgValue::Eval(EvalValue::Text(
                crate::value::ExcelText::from_interop_assignment(""),
            )),
            CallArgValue::Eval(EvalValue::Number(1.0)),
            CallArgValue::Eval(EvalValue::Number(2.0)),
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
            CallArgValue::Eval(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Logical(true),
                    ArrayCellValue::Logical(false),
                    ArrayCellValue::Logical(true),
                ]])
                .unwrap(),
            )),
            CallArgValue::Eval(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Number(1.0),
                    ArrayCellValue::Number(2.0),
                    ArrayCellValue::Number(3.0),
                ]])
                .unwrap(),
            )),
            CallArgValue::Eval(EvalValue::Number(0.0)),
        ];
        let got = eval_if_surface(&args, &NoResolver);
        assert_eq!(
            got,
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Number(1.0),
                    ArrayCellValue::Number(0.0),
                    ArrayCellValue::Number(3.0),
                ]])
                .unwrap()
            ))
        );
    }
}

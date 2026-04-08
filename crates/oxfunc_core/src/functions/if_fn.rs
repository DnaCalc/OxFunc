use crate::coercion::{CoercionError, coerce_eval_to_number};
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{PreparedArgValue, prepare_arg_values_only};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

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
    match prepared {
        PreparedArgValue::Eval(v) => Ok(v),
        PreparedArgValue::MissingArg => Ok(EvalValue::Logical(false)),
        PreparedArgValue::EmptyCell => Ok(EvalValue::Number(0.0)),
    }
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
    use crate::value::ReferenceLike;

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
}

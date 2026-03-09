use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{PreparedArgValue, coerce_prepared_to_number, run_values_only_prepared};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

pub const OP_ADD_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.OP_ADD",
    arity: Arity::exact(2),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::NumsToNum,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
};

#[derive(Debug, Clone, PartialEq)]
pub enum OpAddEvalError {
    ArityMismatch { expected: usize, actual: usize },
    Coercion(CoercionError),
}

pub fn op_add_kernel(lhs: f64, rhs: f64) -> f64 {
    lhs + rhs
}

pub fn eval_op_add_adapter_prepared(
    args: &[PreparedArgValue],
) -> Result<EvalValue, OpAddEvalError> {
    if !OP_ADD_META.arity.accepts(args.len()) {
        return Err(OpAddEvalError::ArityMismatch {
            expected: OP_ADD_META.arity.min,
            actual: args.len(),
        });
    }
    let lhs = coerce_prepared_to_number(&args[0]).map_err(OpAddEvalError::Coercion)?;
    let rhs = coerce_prepared_to_number(&args[1]).map_err(OpAddEvalError::Coercion)?;
    Ok(EvalValue::Number(op_add_kernel(lhs, rhs)))
}

pub fn eval_op_add_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, OpAddEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_op_add_adapter_prepared,
        OpAddEvalError::Coercion,
    )
}

pub fn map_op_add_error_to_ws(e: &OpAddEvalError) -> WorksheetErrorCode {
    match e {
        OpAddEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        OpAddEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        OpAddEvalError::Coercion(_) => WorksheetErrorCode::Value,
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
    fn eval_op_add_two_numbers() {
        let args = [
            CallArgValue::Eval(EvalValue::Number(2.0)),
            CallArgValue::Eval(EvalValue::Number(3.0)),
        ];
        let got = eval_op_add_surface(&args, &NoResolver);
        assert_eq!(got, Ok(EvalValue::Number(5.0)));
    }

    #[test]
    fn eval_op_add_numeric_text_and_logical() {
        let args = [
            CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
                "2".encode_utf16().collect(),
            ))),
            CallArgValue::Eval(EvalValue::Logical(true)),
        ];
        let got = eval_op_add_surface(&args, &NoResolver);
        assert_eq!(got, Ok(EvalValue::Number(3.0)));
    }

    #[test]
    fn eval_op_add_non_numeric_text_fails() {
        let args = [
            CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
                "bad".encode_utf16().collect(),
            ))),
            CallArgValue::Eval(EvalValue::Number(1.0)),
        ];
        let got = eval_op_add_surface(&args, &NoResolver);
        assert!(matches!(got, Err(OpAddEvalError::Coercion(_))));
    }
}

use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    PreparedArgValue, coerce_prepared_to_number, run_values_only_prepared,
};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

pub const TRUNC_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.TRUNC",
    arity: Arity { min: 1, max: 2 },
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
pub enum TruncEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
}

pub fn trunc_kernel(number: f64, digits: i32) -> f64 {
    if digits >= 0 {
        let factor = 10f64.powi(digits);
        (number * factor).trunc() / factor
    } else {
        let factor = 10f64.powi(-digits);
        (number / factor).trunc() * factor
    }
}

pub fn eval_trunc_adapter_prepared(args: &[PreparedArgValue]) -> Result<EvalValue, TruncEvalError> {
    if !TRUNC_META.arity.accepts(args.len()) {
        return Err(TruncEvalError::ArityMismatch {
            expected_min: TRUNC_META.arity.min,
            expected_max: TRUNC_META.arity.max,
            actual: args.len(),
        });
    }
    let number = coerce_prepared_to_number(&args[0]).map_err(TruncEvalError::Coercion)?;
    let digits = if args.len() == 1 {
        0
    } else {
        coerce_prepared_to_number(&args[1])
            .map_err(TruncEvalError::Coercion)?
            .trunc() as i32
    };
    Ok(EvalValue::Number(trunc_kernel(number, digits)))
}

pub fn eval_trunc_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, TruncEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_trunc_adapter_prepared,
        TruncEvalError::Coercion,
    )
}

pub fn map_trunc_error_to_ws(e: &TruncEvalError) -> WorksheetErrorCode {
    match e {
        TruncEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        TruncEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        TruncEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}

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

pub const ROUNDDOWN_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.ROUNDDOWN",
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
pub enum RoundDownEvalError {
    ArityMismatch { expected: usize, actual: usize },
    Coercion(CoercionError),
}

fn parse_digits(arg: &PreparedArgValue) -> Result<i32, RoundDownEvalError> {
    Ok(coerce_prepared_to_number(arg)
        .map_err(RoundDownEvalError::Coercion)?
        .trunc() as i32)
}

pub fn rounddown_kernel(n: f64, digits: i32) -> f64 {
    if digits >= 308 {
        return n;
    }
    if digits <= -308 {
        return if n.is_sign_negative() { -0.0 } else { 0.0 };
    }

    if digits >= 0 {
        let factor = 10f64.powi(digits);
        (n * factor).trunc() / factor
    } else {
        let factor = 10f64.powi(-digits);
        (n / factor).trunc() * factor
    }
}

fn eval_rounddown_prepared(args: &[PreparedArgValue]) -> Result<EvalValue, RoundDownEvalError> {
    if args.len() != 2 {
        return Err(RoundDownEvalError::ArityMismatch {
            expected: 2,
            actual: args.len(),
        });
    }
    let value = coerce_prepared_to_number(&args[0]).map_err(RoundDownEvalError::Coercion)?;
    let digits = parse_digits(&args[1])?;
    Ok(EvalValue::Number(rounddown_kernel(value, digits)))
}

pub fn eval_rounddown_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, RoundDownEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_rounddown_prepared,
        RoundDownEvalError::Coercion,
    )
}

pub fn map_rounddown_error_to_ws(e: &RoundDownEvalError) -> WorksheetErrorCode {
    match e {
        RoundDownEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        RoundDownEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        RoundDownEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rounddown_kernel_matches_excel_probe() {
        assert_eq!(rounddown_kernel(3.14159, 3), 3.141);
        assert_eq!(rounddown_kernel(-3.14159, 3), -3.141);
        assert_eq!(rounddown_kernel(314.159, -2), 300.0);
    }
}

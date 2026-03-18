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

pub const ROUNDUP_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.ROUNDUP",
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
pub enum RoundUpEvalError {
    ArityMismatch { expected: usize, actual: usize },
    Coercion(CoercionError),
}

fn parse_digits(arg: &PreparedArgValue) -> Result<i32, RoundUpEvalError> {
    Ok(coerce_prepared_to_number(arg)
        .map_err(RoundUpEvalError::Coercion)?
        .trunc() as i32)
}

pub fn roundup_kernel(n: f64, digits: i32) -> f64 {
    if n == 0.0 {
        return 0.0;
    }
    if digits >= 308 {
        return n;
    }
    if digits <= -308 {
        return if n.is_sign_negative() { -0.0 } else { 0.0 };
    }

    let sign = if n.is_sign_negative() { -1.0 } else { 1.0 };
    let abs_n = n.abs();
    if digits >= 0 {
        let factor = 10f64.powi(digits);
        sign * ((abs_n * factor).ceil() / factor)
    } else {
        let factor = 10f64.powi(-digits);
        sign * ((abs_n / factor).ceil() * factor)
    }
}

fn eval_roundup_prepared(args: &[PreparedArgValue]) -> Result<EvalValue, RoundUpEvalError> {
    if args.len() != 2 {
        return Err(RoundUpEvalError::ArityMismatch {
            expected: 2,
            actual: args.len(),
        });
    }
    let value = coerce_prepared_to_number(&args[0]).map_err(RoundUpEvalError::Coercion)?;
    let digits = parse_digits(&args[1])?;
    Ok(EvalValue::Number(roundup_kernel(value, digits)))
}

pub fn eval_roundup_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, RoundUpEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_roundup_prepared,
        RoundUpEvalError::Coercion,
    )
}

pub fn map_roundup_error_to_ws(e: &RoundUpEvalError) -> WorksheetErrorCode {
    match e {
        RoundUpEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        RoundUpEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        RoundUpEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundup_kernel_matches_excel_probe() {
        assert_eq!(roundup_kernel(3.14159, 3), 3.142);
        assert_eq!(roundup_kernel(-3.14159, 3), -3.142);
        assert_eq!(roundup_kernel(314.159, -2), 400.0);
    }
}

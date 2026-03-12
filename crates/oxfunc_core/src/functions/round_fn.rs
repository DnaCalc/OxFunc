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

pub const ROUND_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.ROUND",
    arity: Arity::exact(2),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
    coercion_lift_profile: CoercionLiftProfile::UnaryNumericScalarOnly,
    kernel_signature_class: KernelSignatureClass::NumsToNum,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
};

#[derive(Debug, Clone, PartialEq)]
pub enum RoundEvalError {
    ArityMismatch { expected: usize, actual: usize },
    Coercion(CoercionError),
}

fn parse_digits(arg: &PreparedArgValue) -> Result<i32, RoundEvalError> {
    let digits = coerce_prepared_to_number(arg).map_err(RoundEvalError::Coercion)?;
    Ok(digits.trunc() as i32)
}

pub fn round_kernel(n: f64, digits: i32) -> f64 {
    if digits >= 308 {
        return n;
    }
    if digits <= -308 {
        return if n.is_sign_negative() { -0.0 } else { 0.0 };
    }

    if digits >= 0 {
        let factor = 10f64.powi(digits);
        (n * factor).round() / factor
    } else {
        let factor = 10f64.powi(-digits);
        (n / factor).round() * factor
    }
}

pub fn eval_round_adapter_prepared(args: &[PreparedArgValue]) -> Result<EvalValue, RoundEvalError> {
    if !ROUND_META.arity.accepts(args.len()) {
        return Err(RoundEvalError::ArityMismatch {
            expected: ROUND_META.arity.min,
            actual: args.len(),
        });
    }

    let value = coerce_prepared_to_number(&args[0]).map_err(RoundEvalError::Coercion)?;
    let digits = parse_digits(&args[1])?;
    Ok(EvalValue::Number(round_kernel(value, digits)))
}

pub fn eval_round_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, RoundEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_round_adapter_prepared,
        RoundEvalError::Coercion,
    )
}

pub fn map_round_error_to_ws(e: &RoundEvalError) -> WorksheetErrorCode {
    match e {
        RoundEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        RoundEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        RoundEvalError::Coercion(_) => WorksheetErrorCode::Value,
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
    fn round_kernel_rounds_half_away_from_zero() {
        assert_eq!(round_kernel(1.25, 1), 1.3);
        assert_eq!(round_kernel(-1.25, 1), -1.3);
    }

    #[test]
    fn eval_round_supports_negative_digits() {
        let got = eval_round_surface(
            &[
                CallArgValue::Eval(EvalValue::Number(123.0)),
                CallArgValue::Eval(EvalValue::Number(-1.0)),
            ],
            &NoResolver,
        );
        assert_eq!(got, Ok(EvalValue::Number(120.0)));
    }

    #[test]
    fn eval_round_truncates_digits_toward_zero() {
        let got = eval_round_surface(
            &[
                CallArgValue::Eval(EvalValue::Number(1.5)),
                CallArgValue::Eval(EvalValue::Number(0.9)),
            ],
            &NoResolver,
        );
        assert_eq!(got, Ok(EvalValue::Number(2.0)));
    }
}

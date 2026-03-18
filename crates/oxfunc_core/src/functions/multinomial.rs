use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{PreparedArgValue, prepare_args_values_only};
use crate::functions::factorial_common::{factorial_of_int, trunc_nonnegative};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

pub const MULTINOMIAL_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.MULTINOMIAL",
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
pub enum MultinomialEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
    Domain(WorksheetErrorCode),
}

fn coerce_prepared_to_nonnegative_int(arg: &PreparedArgValue) -> Result<i64, MultinomialEvalError> {
    let n = crate::functions::adapters::coerce_prepared_to_number(arg)
        .map_err(MultinomialEvalError::Coercion)?;
    trunc_nonnegative(n).map_err(MultinomialEvalError::Domain)
}

pub fn multinomial_kernel(items: &[i64]) -> Result<f64, WorksheetErrorCode> {
    let total = items.iter().sum::<i64>();
    let denominator = items
        .iter()
        .fold(1.0, |acc, item| acc * factorial_of_int(*item));
    Ok(factorial_of_int(total) / denominator)
}

pub fn eval_multinomial_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, MultinomialEvalError> {
    let argc = args.len();
    if !MULTINOMIAL_META.arity.accepts(argc) {
        return Err(MultinomialEvalError::ArityMismatch {
            expected_min: MULTINOMIAL_META.arity.min,
            expected_max: MULTINOMIAL_META.arity.max,
            actual: argc,
        });
    }

    let prepared =
        prepare_args_values_only(args, resolver).map_err(MultinomialEvalError::Coercion)?;
    let items = prepared
        .iter()
        .map(coerce_prepared_to_nonnegative_int)
        .collect::<Result<Vec<_>, _>>()?;
    multinomial_kernel(&items)
        .map(EvalValue::Number)
        .map_err(MultinomialEvalError::Domain)
}

pub fn map_multinomial_error_to_ws(e: &MultinomialEvalError) -> WorksheetErrorCode {
    match e {
        MultinomialEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        MultinomialEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        MultinomialEvalError::Coercion(_) => WorksheetErrorCode::Value,
        MultinomialEvalError::Domain(code) => *code,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multinomial_meta_function_id_is_stable() {
        assert_eq!(MULTINOMIAL_META.function_id, "FUNC.MULTINOMIAL");
    }

    #[test]
    fn multinomial_kernel_matches_excel_seed_rows() {
        assert_eq!(multinomial_kernel(&[1, 2, 3]), Ok(60.0));
        assert_eq!(multinomial_kernel(&[5]), Ok(1.0));
        assert_eq!(multinomial_kernel(&[0, 0]), Ok(1.0));
    }
}

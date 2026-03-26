// PIVOTBY scaffold — W038_BLOCKED: callable invocation requires LAMBDA/LET infrastructure.
//
// This scaffold implements META, error enum, arity validation, and the non-callable
// argument parsing layer. The callable 4th-argument invocation path depends on W038
// (LAMBDA/LET family callable infrastructure), which is not yet complete.

use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::prepare_args_values_only;
use crate::functions::callable_helpers::{CallableInvoker, LambdaHelperEvalError};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

pub const PIVOTBY_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.PIVOTBY",
    arity: Arity { min: 4, max: 255 },
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

fn surface_arity_error(actual: usize) -> LambdaHelperEvalError {
    LambdaHelperEvalError::ArityMismatch {
        expected_min: PIVOTBY_META.arity.min,
        expected_max: PIVOTBY_META.arity.max,
        actual,
    }
}

pub fn eval_pivotby_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
    _invoker: &(impl CallableInvoker + ?Sized),
) -> Result<EvalValue, LambdaHelperEvalError> {
    if !PIVOTBY_META.arity.accepts(args.len()) {
        return Err(surface_arity_error(args.len()));
    }

    // Prepare arguments (dereferences references).
    let _prepared =
        prepare_args_values_only(args, resolver).map_err(LambdaHelperEvalError::Preparation)?;

    // W038_BLOCKED: callable invocation requires LAMBDA/LET infrastructure.
    // The 4th argument is the aggregation function (callable).
    // Until W038 provides the callable-invocation seam, this returns #VALUE!
    // via MissingCallable, as the callable resolution path is not yet wired.
    Err(LambdaHelperEvalError::MissingCallable)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::function::{DeterminismClass, VolatilityClass};

    // --- Meta tests ---

    #[test]
    fn pivotby_meta_arity() {
        assert_eq!(PIVOTBY_META.arity.min, 4);
        assert_eq!(PIVOTBY_META.arity.max, 255);
    }

    #[test]
    fn pivotby_meta_deterministic() {
        assert_eq!(PIVOTBY_META.determinism, DeterminismClass::Deterministic);
        assert_eq!(PIVOTBY_META.volatility, VolatilityClass::NonVolatile);
    }

    #[test]
    fn pivotby_meta_values_only() {
        assert_eq!(
            PIVOTBY_META.arg_preparation_profile,
            ArgPreparationProfile::ValuesOnlyPreAdapter
        );
    }
}

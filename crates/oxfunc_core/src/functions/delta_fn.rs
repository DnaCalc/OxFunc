use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::binary_numeric::{
    BinaryNumericSurfaceError, eval_binary_numeric_surface, map_binary_numeric_error_to_ws,
};
use crate::resolver::ReferenceResolver;
use crate::value::{EvalValue, WorksheetErrorCode};

pub const DELTA_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.DELTA",
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

pub fn delta_kernel(lhs: f64, rhs: f64) -> Result<f64, WorksheetErrorCode> {
    Ok(if lhs == rhs { 1.0 } else { 0.0 })
}

pub fn eval_delta_surface(
    args: &[crate::value::CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, BinaryNumericSurfaceError> {
    let actual = args.len();
    if actual == 1 {
        return eval_binary_numeric_surface(
            &[
                args[0].clone(),
                crate::value::CallArgValue::Eval(EvalValue::Number(0.0)),
            ],
            resolver,
            delta_kernel,
        );
    }
    eval_binary_numeric_surface(args, resolver, delta_kernel)
}

pub fn map_delta_error_to_ws(e: &BinaryNumericSurfaceError) -> WorksheetErrorCode {
    map_binary_numeric_error_to_ws(e)
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
    fn delta_exact_match_does_not_tolerate_near_equal_numbers() {
        assert_eq!(delta_kernel(0.1 + 0.2, 0.3), Ok(0.0));
        assert_eq!(
            eval_delta_surface(
                &[
                    crate::value::CallArgValue::Eval(EvalValue::Number(0.1 + 0.2)),
                    crate::value::CallArgValue::Eval(EvalValue::Number(0.3)),
                ],
                &NoResolver,
            ),
            Ok(EvalValue::Number(0.0))
        );

        let boundary_probe = ((123_456_789_012_345_f64 * 10.0) + 5.0) / 1.0e25;
        let boundary_stored = ((123_456_789_012_345_f64 * 10.0) + 4.0) / 1.0e25;
        assert_eq!(delta_kernel(boundary_probe, boundary_stored), Ok(0.0));
    }
}

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
    resolver: &impl ReferenceResolver,
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

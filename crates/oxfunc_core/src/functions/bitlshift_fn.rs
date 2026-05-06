use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::binary_numeric::{
    BinaryNumericSurfaceError, eval_binary_numeric_surface, map_binary_numeric_error_to_ws,
};
use crate::functions::bit_common::{BIT_MAX, coerce_bit_operand, coerce_shift_count};
use crate::resolver::ReferenceResolver;
use crate::value::{EvalValue, WorksheetErrorCode};

pub const BITLSHIFT_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.BITLSHIFT",
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

pub fn bitlshift_kernel(number: f64, shift: f64) -> Result<f64, WorksheetErrorCode> {
    let number = coerce_bit_operand(number)?;
    let shift = coerce_shift_count(shift)?;
    let result = if shift >= 0 {
        number
            .checked_shl(shift as u32)
            .ok_or(WorksheetErrorCode::Num)?
    } else {
        number >> (-shift as u32)
    };
    if result > BIT_MAX {
        return Err(WorksheetErrorCode::Num);
    }
    Ok(result as f64)
}

pub fn eval_bitlshift_surface(
    args: &[crate::value::CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, BinaryNumericSurfaceError> {
    eval_binary_numeric_surface(args, resolver, bitlshift_kernel)
}

pub fn map_bitlshift_error_to_ws(e: &BinaryNumericSurfaceError) -> WorksheetErrorCode {
    map_binary_numeric_error_to_ws(e)
}

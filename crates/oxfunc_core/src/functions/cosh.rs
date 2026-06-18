use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::excel_numeric::finite_or_num;
use crate::functions::unary_numeric::{
    UnaryNumericSurfaceError, eval_unary_numeric_surface, map_unary_numeric_error_to_ws,
};
use crate::resolver::ReferenceSystemProvider;
use crate::value::CalcValue;
use crate::value::WorksheetErrorCode;

pub const COSH_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.COSH",
    arity: Arity::exact(1),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
    coercion_lift_profile: CoercionLiftProfile::UnaryNumericScalarOrArrayElementwise,
    kernel_signature_class: KernelSignatureClass::NumToNum,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
};

pub fn cosh_kernel(n: f64) -> f64 {
    n.cosh()
}

pub fn eval_cosh_surface(
    args: &[crate::value::CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, UnaryNumericSurfaceError> {
    // BUG-FUNC-027 CLASS-A3: COSH overflows to #NUM! in Excel, not +Inf.
    eval_unary_numeric_surface(args, resolver, |n| finite_or_num(cosh_kernel(n)))
}

pub fn map_cosh_error_to_ws(e: &UnaryNumericSurfaceError) -> WorksheetErrorCode {
    map_unary_numeric_error_to_ws(e)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cosh_meta_function_id_is_stable() {
        assert_eq!(COSH_META.function_id, "FUNC.COSH");
    }

    #[test]
    fn cosh_kernel_matches_std() {
        assert_eq!(cosh_kernel(1.0), 1.0f64.cosh());
    }

    // BUG-FUNC-027 CLASS-A3: live Excel 16.0 b20026 COSH(-24230)=#NUM!.
    #[test]
    fn cosh_overflow_maps_to_num() {
        assert_eq!(
            finite_or_num(cosh_kernel(-24230.0)),
            Err(WorksheetErrorCode::Num)
        );
        assert_eq!(finite_or_num(cosh_kernel(1.0)), Ok(1.0f64.cosh()));
    }
}

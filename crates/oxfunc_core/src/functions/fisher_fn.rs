use crate::function::{
    Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile, FunctionMeta,
    HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::unary_numeric::{
    UnaryNumericExecSpec, UnaryNumericSurfaceError, eval_unary_numeric_via_executor,
    map_unary_numeric_error_to_ws,
};
use crate::resolver::ReferenceSystemProvider;
use crate::value::CalcValue;
use crate::value::WorksheetErrorCode;

pub const FISHER_META: FunctionMeta = function_spec! {
    function_id: "FUNC.FISHER",
    arity: Arity::exact(1),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    coercion_lift_profile: CoercionLiftProfile::UnaryNumericScalarOrArrayElementwise,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
};

pub fn fisher_kernel(x: f64) -> Result<f64, WorksheetErrorCode> {
    if x.abs() >= 1.0 {
        return Err(WorksheetErrorCode::Num);
    }
    // Inverse-problem identity, live Excel 16.0 b20228: FISHER(x) ==
    // 0.5*LN((1+x)/(1-x)) on 33/33 signed rows including tiny and near-1.
    // FISHER is not ATANH (21/33): ATANH keeps a cubic small-x body.
    // Split LN(1+x)-LN(1-x) is not the graph (10/33).
    Ok(0.5 * crate::excel_numeric::excel_log((1.0 + x) / (1.0 - x)))
}

pub fn eval_fisher_surface(
    args: &[crate::value::CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, UnaryNumericSurfaceError> {
    eval_unary_numeric_via_executor(
        args,
        resolver,
        UnaryNumericExecSpec::fallible(fisher_kernel, FISHER_META.real_result_policy),
    )
}

pub fn map_fisher_error_to_ws(e: &UnaryNumericSurfaceError) -> WorksheetErrorCode {
    map_unary_numeric_error_to_ws(e)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fisher_follows_worksheet_ln_ratio_identity() {
        for x in [0.0, 1e-8, 0.1, 0.5, -0.5, 0.9] {
            let got = fisher_kernel(x).unwrap();
            let expect = 0.5 * crate::excel_numeric::excel_log((1.0 + x) / (1.0 - x));
            assert_eq!(got.to_bits(), expect.to_bits(), "x={x}");
        }
        assert_eq!(fisher_kernel(1.0), Err(WorksheetErrorCode::Num));
        assert_eq!(fisher_kernel(-1.0), Err(WorksheetErrorCode::Num));
    }
}

use crate::coercion::{CoercionError, coerce_calc_scalar_to_number};
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::expand_aggregate_arg;
use crate::functions::factorial_common::trunc_nonnegative;
use crate::functions::gcd_lcm_common::lcm_int;
use crate::resolver::ReferenceSystemProvider;
use crate::value::CalcValue;
use crate::value::WorksheetErrorCode;

pub const LCM_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.LCM",
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
    real_result_policy: FunctionMeta::DEFAULT_REAL_RESULT_POLICY,
};

#[derive(Debug, Clone, PartialEq)]
pub enum LcmEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
    Domain(WorksheetErrorCode),
}

fn coerce_calc_to_nonnegative_int(arg: &CalcValue) -> Result<i64, LcmEvalError> {
    let n = coerce_calc_scalar_to_number(arg).map_err(LcmEvalError::Coercion)?;
    trunc_nonnegative(n).map_err(LcmEvalError::Domain)
}

pub fn lcm_kernel(items: &[i64]) -> f64 {
    items.iter().copied().fold(1, lcm_int) as f64
}

pub fn eval_lcm_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, LcmEvalError> {
    let argc = args.len();
    if !LCM_META.arity.accepts(argc) {
        return Err(LcmEvalError::ArityMismatch {
            expected_min: LCM_META.arity.min,
            expected_max: LCM_META.arity.max,
            actual: argc,
        });
    }
    // Accept array arguments by flattening each into its constituent values
    // (Excel reduces LCM/GCD over arrays to a scalar, like GCD's surface).
    let mut items = Vec::new();
    for arg in args {
        let expanded = expand_aggregate_arg(arg, resolver).map_err(LcmEvalError::Coercion)?;
        for item in expanded {
            items.push(coerce_calc_to_nonnegative_int(&item.0)?);
        }
    }
    Ok(CalcValue::number(lcm_kernel(&items)))
}

pub fn map_lcm_error_to_ws(e: &LcmEvalError) -> WorksheetErrorCode {
    match e {
        LcmEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        LcmEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        LcmEvalError::Coercion(_) => WorksheetErrorCode::Value,
        LcmEvalError::Domain(code) => *code,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lcm_meta_function_id_is_stable() {
        assert_eq!(LCM_META.function_id, "FUNC.LCM");
    }

    #[test]
    fn lcm_kernel_matches_excel_seed_rows() {
        assert_eq!(lcm_kernel(&[6, 8]), 24.0);
        assert_eq!(lcm_kernel(&[0, 5]), 0.0);
        assert_eq!(lcm_kernel(&[0, 0]), 0.0);
    }
}

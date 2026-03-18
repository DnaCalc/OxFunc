use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{PreparedArgValue, prepare_args_values_only};
use crate::functions::factorial_common::trunc_nonnegative;
use crate::functions::gcd_lcm_common::lcm_int;
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

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

fn coerce_prepared_to_nonnegative_int(arg: &PreparedArgValue) -> Result<i64, LcmEvalError> {
    let n = crate::functions::adapters::coerce_prepared_to_number(arg)
        .map_err(LcmEvalError::Coercion)?;
    trunc_nonnegative(n).map_err(LcmEvalError::Domain)
}

pub fn lcm_kernel(items: &[i64]) -> f64 {
    items.iter().copied().fold(1, lcm_int) as f64
}

pub fn eval_lcm_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, LcmEvalError> {
    let argc = args.len();
    if !LCM_META.arity.accepts(argc) {
        return Err(LcmEvalError::ArityMismatch {
            expected_min: LCM_META.arity.min,
            expected_max: LCM_META.arity.max,
            actual: argc,
        });
    }
    let prepared = prepare_args_values_only(args, resolver).map_err(LcmEvalError::Coercion)?;
    let items = prepared
        .iter()
        .map(coerce_prepared_to_nonnegative_int)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(EvalValue::Number(lcm_kernel(&items)))
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

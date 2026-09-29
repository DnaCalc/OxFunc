use crate::coercion::CoercionError;
use crate::function::{
    Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile, FunctionMeta,
    HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::gcd_lcm_common::{collect_integer_groups, lcm_int};
use crate::resolver::ReferenceSystemProvider;
use crate::value::CalcValue;
use crate::value::WorksheetErrorCode;

pub const LCM_META: FunctionMeta = function_spec! {
    function_id: "FUNC.LCM",
    arity: Arity { min: 1, max: 255 },
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
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

pub fn lcm_kernel(items: &[i64]) -> Result<f64, WorksheetErrorCode> {
    if items.contains(&0) {
        return Ok(0.0);
    }
    // Excel reduces the flattened values in reverse worksheet/argument order.
    // Rounded intermediates at 2^53 make this order observable.
    items
        .iter()
        .rev()
        .copied()
        .try_fold(1, lcm_int)
        .map(|n| n as f64)
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
    let groups = collect_integer_groups(args, resolver).map_err(LcmEvalError::Coercion)?;
    let items = groups.into_iter().flatten().collect::<Vec<_>>();
    lcm_kernel(&items)
        .map(CalcValue::number)
        .map_err(LcmEvalError::Domain)
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
        assert_eq!(lcm_kernel(&[6, 8]), Ok(24.0));
        assert_eq!(lcm_kernel(&[0, 5]), Ok(0.0));
        assert_eq!(lcm_kernel(&[0, 0]), Ok(0.0));
        assert_eq!(
            lcm_kernel(&[3, 3, 3_002_399_751_580_331]),
            Err(WorksheetErrorCode::Num)
        );
        assert_eq!(
            lcm_kernel(&[3_002_399_751_580_331, 3, 3]),
            Ok(9_007_199_254_740_992.0)
        );
        assert_eq!(
            lcm_kernel(&[0, 9_007_199_254_740_991, 9_007_199_254_740_990]),
            Ok(0.0)
        );
    }
}

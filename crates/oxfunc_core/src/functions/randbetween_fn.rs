use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{coerce_prepared_to_number, run_values_only_prepared};
use crate::functions::rand_fn::RandomProvider;
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

pub const RANDBETWEEN_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.RANDBETWEEN",
    arity: Arity::exact(2),
    determinism: DeterminismClass::PseudoRandom,
    volatility: VolatilityClass::VolatileFull,
    host_interaction: HostInteractionClass::ApplicationState,
    thread_safety: ThreadSafetyClass::HostSerialized,
    arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::RandomProvider,
    surface_fec_dependency_profile: FecDependencyProfile::RandomProvider,
};

#[derive(Debug, Clone, PartialEq)]
pub enum RandbetweenEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
    BottomGreaterThanTop,
    ProviderOutOfRange(f64),
}

pub fn randbetween_kernel(
    provider: &impl RandomProvider,
    bottom: f64,
    top: f64,
) -> Result<EvalValue, RandbetweenEvalError> {
    // Excel truncates bottom upward (ceil) and top downward (floor) to integers.
    let lo = bottom.ceil();
    let hi = top.floor();

    if lo > hi {
        return Err(RandbetweenEvalError::BottomGreaterThanTop);
    }

    if !lo.is_finite() || !hi.is_finite() {
        return Err(RandbetweenEvalError::BottomGreaterThanTop);
    }

    let raw = provider.random_unit();
    if !raw.is_finite() || !(0.0..1.0).contains(&raw) {
        return Err(RandbetweenEvalError::ProviderOutOfRange(raw));
    }

    // Scale [0,1) into [lo, hi] inclusive.
    let range = hi - lo + 1.0;
    let result = lo + (raw * range).floor();
    // Clamp to hi in case of floating-point edge case where raw * range rounds up.
    let clamped = result.min(hi);
    Ok(EvalValue::Number(clamped))
}

pub fn eval_randbetween_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
    provider: &impl RandomProvider,
) -> Result<EvalValue, RandbetweenEvalError> {
    if !RANDBETWEEN_META.arity.accepts(args.len()) {
        return Err(RandbetweenEvalError::ArityMismatch {
            expected_min: RANDBETWEEN_META.arity.min,
            expected_max: RANDBETWEEN_META.arity.max,
            actual: args.len(),
        });
    }

    run_values_only_prepared(
        args,
        resolver,
        |prepared| {
            let bottom =
                coerce_prepared_to_number(&prepared[0]).map_err(RandbetweenEvalError::Coercion)?;
            let top =
                coerce_prepared_to_number(&prepared[1]).map_err(RandbetweenEvalError::Coercion)?;
            randbetween_kernel(provider, bottom, top)
        },
        RandbetweenEvalError::Coercion,
    )
}

pub fn map_randbetween_error_to_ws(e: &RandbetweenEvalError) -> WorksheetErrorCode {
    match e {
        RandbetweenEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        RandbetweenEvalError::Coercion(_) => WorksheetErrorCode::Value,
        RandbetweenEvalError::BottomGreaterThanTop => WorksheetErrorCode::Num,
        RandbetweenEvalError::ProviderOutOfRange(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::function::{DeterminismClass, FecDependencyProfile, VolatilityClass};
    use crate::resolver::{CallerContext, RefResolutionError, ResolverCapabilities};
    use crate::value::ReferenceLike;

    struct FixedProvider {
        value: f64,
    }

    impl RandomProvider for FixedProvider {
        fn random_unit(&self) -> f64 {
            self.value
        }
    }

    struct MockResolver;
    impl ReferenceResolver for MockResolver {
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
        fn caller_context(&self) -> Option<CallerContext> {
            None
        }
    }

    fn num(v: f64) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Number(v))
    }

    // --- Meta tests ---

    #[test]
    fn randbetween_meta_arity() {
        assert_eq!(RANDBETWEEN_META.arity.min, 2);
        assert_eq!(RANDBETWEEN_META.arity.max, 2);
    }

    #[test]
    fn randbetween_meta_volatile() {
        assert_eq!(RANDBETWEEN_META.volatility, VolatilityClass::VolatileFull);
        assert_eq!(RANDBETWEEN_META.determinism, DeterminismClass::PseudoRandom);
    }

    #[test]
    fn randbetween_meta_random_provider() {
        assert_eq!(
            RANDBETWEEN_META.fec_dependency_profile,
            FecDependencyProfile::RandomProvider
        );
    }

    // --- Arity tests ---

    #[test]
    fn randbetween_rejects_one_arg() {
        let got =
            eval_randbetween_surface(&[num(1.0)], &MockResolver, &FixedProvider { value: 0.5 });
        assert!(matches!(
            got,
            Err(RandbetweenEvalError::ArityMismatch { .. })
        ));
    }

    #[test]
    fn randbetween_rejects_three_args() {
        let got = eval_randbetween_surface(
            &[num(1.0), num(10.0), num(5.0)],
            &MockResolver,
            &FixedProvider { value: 0.5 },
        );
        assert!(matches!(
            got,
            Err(RandbetweenEvalError::ArityMismatch { .. })
        ));
    }

    // --- Kernel tests ---

    #[test]
    fn randbetween_basic_range() {
        // provider=0.5, range [1,10]: lo=1, hi=10, range_size=10, 1 + floor(0.5*10) = 1+5 = 6
        let got = randbetween_kernel(&FixedProvider { value: 0.5 }, 1.0, 10.0);
        assert_eq!(got, Ok(EvalValue::Number(6.0)));
    }

    #[test]
    fn randbetween_provider_zero_gives_bottom() {
        let got = randbetween_kernel(&FixedProvider { value: 0.0 }, 1.0, 10.0);
        assert_eq!(got, Ok(EvalValue::Number(1.0)));
    }

    #[test]
    fn randbetween_provider_near_one_gives_top() {
        // 0.999... * 10 = 9.99, floor = 9, 1 + 9 = 10
        let got = randbetween_kernel(&FixedProvider { value: 0.999 }, 1.0, 10.0);
        assert_eq!(got, Ok(EvalValue::Number(10.0)));
    }

    #[test]
    fn randbetween_equal_bounds() {
        let got = randbetween_kernel(&FixedProvider { value: 0.5 }, 5.0, 5.0);
        assert_eq!(got, Ok(EvalValue::Number(5.0)));
    }

    #[test]
    fn randbetween_negative_range() {
        // range [-10, -1]: lo=-10, hi=-1, range_size=10, -10 + floor(0.3*10) = -10+3 = -7
        let got = randbetween_kernel(&FixedProvider { value: 0.3 }, -10.0, -1.0);
        assert_eq!(got, Ok(EvalValue::Number(-7.0)));
    }

    #[test]
    fn randbetween_non_integer_truncation() {
        // bottom=1.3 -> ceil=2, top=4.7 -> floor=4, range [2,4], size=3
        // 2 + floor(0.5*3) = 2+1 = 3
        let got = randbetween_kernel(&FixedProvider { value: 0.5 }, 1.3, 4.7);
        assert_eq!(got, Ok(EvalValue::Number(3.0)));
    }

    #[test]
    fn randbetween_negative_non_integer_truncation() {
        // bottom=-4.7 -> ceil=-4, top=-1.3 -> floor=-2, range [-4,-2], size=3
        // -4 + floor(0.0*3) = -4+0 = -4
        let got = randbetween_kernel(&FixedProvider { value: 0.0 }, -4.7, -1.3);
        assert_eq!(got, Ok(EvalValue::Number(-4.0)));
    }

    #[test]
    fn randbetween_bottom_greater_than_top_returns_num_error() {
        let got = randbetween_kernel(&FixedProvider { value: 0.5 }, 10.0, 1.0);
        assert_eq!(got, Err(RandbetweenEvalError::BottomGreaterThanTop));
    }

    #[test]
    fn randbetween_fractional_bottom_gt_top_after_truncation() {
        // bottom=4.8 -> ceil=5, top=4.2 -> floor=4, 5 > 4 -> #NUM!
        let got = randbetween_kernel(&FixedProvider { value: 0.5 }, 4.8, 4.2);
        assert_eq!(got, Err(RandbetweenEvalError::BottomGreaterThanTop));
    }

    #[test]
    fn randbetween_provider_out_of_range() {
        let got = randbetween_kernel(&FixedProvider { value: 1.5 }, 1.0, 10.0);
        assert_eq!(got, Err(RandbetweenEvalError::ProviderOutOfRange(1.5)));
    }

    // --- Surface integration tests ---

    #[test]
    fn randbetween_surface_basic() {
        let got = eval_randbetween_surface(
            &[num(1.0), num(10.0)],
            &MockResolver,
            &FixedProvider { value: 0.5 },
        );
        assert_eq!(got, Ok(EvalValue::Number(6.0)));
    }

    #[test]
    fn randbetween_surface_boolean_coercion() {
        // TRUE=1, FALSE=0 -> range [0,1]
        let got = eval_randbetween_surface(
            &[
                CallArgValue::Eval(EvalValue::Logical(false)),
                CallArgValue::Eval(EvalValue::Logical(true)),
            ],
            &MockResolver,
            &FixedProvider { value: 0.0 },
        );
        assert_eq!(got, Ok(EvalValue::Number(0.0)));
    }

    // --- Error mapping tests ---

    #[test]
    fn randbetween_error_mapping() {
        assert_eq!(
            map_randbetween_error_to_ws(&RandbetweenEvalError::BottomGreaterThanTop),
            WorksheetErrorCode::Num
        );
        assert_eq!(
            map_randbetween_error_to_ws(&RandbetweenEvalError::ArityMismatch {
                expected_min: 2,
                expected_max: 2,
                actual: 1,
            }),
            WorksheetErrorCode::Value
        );
    }
}

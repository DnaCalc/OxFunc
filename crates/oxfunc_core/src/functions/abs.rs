use crate::coercion::CoercionError;
use crate::function::{
    Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile, FunctionMeta,
    HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    UnaryNumericCoercionLiftProfile, apply_unary_numeric_scalar_prepared, map_values_only_prepared,
    run_values_only_prepared,
};
use crate::functions::unary_numeric::{
    UnaryNumericExecSpec, UnaryNumericSurfaceError, eval_unary_numeric_via_executor,
    map_unary_numeric_error_to_ws,
};
use crate::resolver::ReferenceSystemProvider;
use crate::value::CalcValue;
use crate::value::WorksheetErrorCode;

pub const ABS_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.ABS",
    arity: Arity::exact(1),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: FunctionMeta::DEFAULT_ARG_PREPARATION_PROFILE,
    coercion_lift_profile: CoercionLiftProfile::UnaryNumericScalarOrArrayElementwise,
    lift_broadcast_profile: FunctionMeta::DEFAULT_LIFT_BROADCAST_PROFILE,
    kernel_signature_class: KernelSignatureClass::NumToNum,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
    real_result_policy: FunctionMeta::DEFAULT_REAL_RESULT_POLICY,
    error_collapse_profile: FunctionMeta::DEFAULT_ERROR_COLLAPSE_PROFILE,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AbsEvalError {
    ArityMismatch { expected: usize, actual: usize },
    Coercion(CoercionError),
}

#[derive(Debug, Clone, PartialEq)]
pub enum AbsLiftOutcome {
    Number(f64),
    Error(CoercionError),
}

pub const ABS_COERCION_LIFT_PROFILE: UnaryNumericCoercionLiftProfile =
    UnaryNumericCoercionLiftProfile::ScalarOrArrayElementwise;

pub fn abs_kernel(n: f64) -> f64 {
    n.abs()
}

/// Surface evaluation for `FUNC.ABS`, routed through the W105 oxf-y2uw.3 unary-numeric
/// executor (ODR-FN-004 Layer 3) exactly like the rest of the unary-numeric family
/// (`eval_sin_surface` et al.). ABS's `real_result_policy` is the default `PASS`, so its
/// kernel is bound as a `Raw` spec and the executor publishes the raw result unchanged.
///
/// W105 oxf-y2uw.6: this REPLACES ABS's former bespoke by-index `eval_abs_scalar_value` arm
/// plus the `observed_scalar_array_lift_positions(FUNC.ABS) = [0]` table entry. For a unary
/// function that observed `[0]` lift is exactly the standard elementwise lift the executor's
/// surface helper performs natively, so ABS now carries the default `SurfaceNative`
/// `lift_broadcast_profile` and lifts here, off the id-keyed lift table. Kept bit-exact under
/// the .1 harness Law 1 (kernel publication) and Law 2 (coercion / lift / broadcast).
pub fn eval_abs_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, UnaryNumericSurfaceError> {
    eval_unary_numeric_via_executor(
        args,
        resolver,
        UnaryNumericExecSpec::raw(abs_kernel, ABS_META.real_result_policy),
    )
}

pub fn map_abs_surface_error_to_ws(e: &UnaryNumericSurfaceError) -> WorksheetErrorCode {
    map_unary_numeric_error_to_ws(e)
}

pub fn eval_abs_adapter_scalar_prepared(args: &[CalcValue]) -> Result<f64, AbsEvalError> {
    if !ABS_META.arity.accepts(args.len()) {
        return Err(AbsEvalError::ArityMismatch {
            expected: ABS_META.arity.min,
            actual: args.len(),
        });
    }

    apply_unary_numeric_scalar_prepared(&args[0], abs_kernel).map_err(AbsEvalError::Coercion)
}

pub fn eval_abs_adapter_scalar_prepared_value(
    args: &[CalcValue],
) -> Result<CalcValue, AbsEvalError> {
    eval_abs_adapter_scalar_prepared(args).map(CalcValue::number)
}

pub fn eval_abs_adapter_arg_prepared(arg: &CalcValue) -> AbsLiftOutcome {
    match apply_unary_numeric_scalar_prepared(arg, abs_kernel) {
        Ok(n) => AbsLiftOutcome::Number(n),
        Err(e) => AbsLiftOutcome::Error(e),
    }
}

pub fn eval_abs_adapter_array_lift_prepared(args: &[CalcValue]) -> Vec<AbsLiftOutcome> {
    args.iter().map(eval_abs_adapter_arg_prepared).collect()
}

pub fn eval_abs_scalar(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<f64, AbsEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_abs_adapter_scalar_prepared,
        AbsEvalError::Coercion,
    )
}

pub fn eval_abs_scalar_value(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, AbsEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_abs_adapter_scalar_prepared_value,
        AbsEvalError::Coercion,
    )
}

pub fn eval_abs_array_lift(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Vec<AbsLiftOutcome> {
    map_values_only_prepared(
        args,
        resolver,
        eval_abs_adapter_arg_prepared,
        AbsLiftOutcome::Error,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::ReferenceSystemCapabilities;
    use crate::value::{ExcelText, ReferenceKind, ReferenceLike, WorksheetErrorCode};

    fn text_prepared(s: &str) -> CalcValue {
        CalcValue::text(ExcelText::from_utf16_code_units(s.encode_utf16().collect()))
    }

    struct MockResolver {
        caps: ReferenceSystemCapabilities,
        resolved_value: Option<CalcValue>,
    }

    impl ReferenceSystemProvider for MockResolver {
        fn capabilities(&self) -> ReferenceSystemCapabilities {
            self.caps
        }

        fn dereference(
            &self,
            request: &crate::resolver::ReferenceDereferenceRequest,
        ) -> Result<CalcValue, crate::resolver::ReferenceResolutionError> {
            let reference = &request.reference;
            self.resolved_value.clone().ok_or(
                crate::resolver::ReferenceResolutionError::UnresolvedReference {
                    target: reference.target().to_string(),
                },
            )
        }
    }

    fn resolver() -> MockResolver {
        MockResolver {
            caps: ReferenceSystemCapabilities::permissive_local(),
            resolved_value: None,
        }
    }

    fn text_arg(s: &str) -> CalcValue {
        CalcValue::text(ExcelText::from_utf16_code_units(s.encode_utf16().collect()))
    }

    #[test]
    fn abs_meta_matches_w5_contract_shape() {
        assert_eq!(ABS_META.function_id, "FUNC.ABS");
        assert_eq!(ABS_META.arity, Arity::exact(1));
        assert_eq!(ABS_META.determinism, DeterminismClass::Deterministic);
        assert_eq!(ABS_META.volatility, VolatilityClass::NonVolatile);
        assert_eq!(ABS_META.host_interaction, HostInteractionClass::None);
        assert_eq!(ABS_META.thread_safety, ThreadSafetyClass::SafePure);
        assert_eq!(
            ABS_META.arg_preparation_profile,
            FunctionMeta::DEFAULT_ARG_PREPARATION_PROFILE
        );
        assert_eq!(
            ABS_META.coercion_lift_profile,
            CoercionLiftProfile::UnaryNumericScalarOrArrayElementwise
        );
        assert_eq!(
            ABS_META.kernel_signature_class,
            KernelSignatureClass::NumToNum
        );
        assert_eq!(ABS_META.fec_dependency_profile, FecDependencyProfile::None);
        assert_eq!(
            ABS_META.surface_fec_dependency_profile,
            FecDependencyProfile::RefOnly
        );
    }

    #[test]
    fn eval_abs_adapter_scalar_prepared_rejects_non_unary_calls() {
        assert_eq!(
            eval_abs_adapter_scalar_prepared(&[]),
            Err(AbsEvalError::ArityMismatch {
                expected: 1,
                actual: 0
            })
        );

        let args = [(CalcValue::number(1.0)), (CalcValue::number(2.0))];
        assert_eq!(
            eval_abs_adapter_scalar_prepared(&args),
            Err(AbsEvalError::ArityMismatch {
                expected: 1,
                actual: 2
            })
        );
    }

    #[test]
    fn eval_abs_adapter_scalar_prepared_on_number() {
        let args = [(CalcValue::number(-2.5))];
        let got = eval_abs_adapter_scalar_prepared(&args);
        assert_eq!(got, Ok(2.5));
    }

    #[test]
    fn eval_abs_adapter_scalar_prepared_text_numeric_is_coerced() {
        let args = [text_prepared(" -2 ")];
        let got = eval_abs_adapter_scalar_prepared(&args);
        assert_eq!(got, Ok(2.0));
    }

    #[test]
    fn eval_abs_adapter_scalar_prepared_non_numeric_text_returns_coercion_error() {
        let args = [text_prepared("asd")];
        let got = eval_abs_adapter_scalar_prepared(&args);
        assert_eq!(
            got,
            Err(AbsEvalError::Coercion(CoercionError::NonNumericText(
                "asd".to_string()
            )))
        );
    }

    #[test]
    fn eval_abs_adapter_scalar_prepared_propagates_worksheet_error_via_coercion() {
        let args = [(CalcValue::error(WorksheetErrorCode::Div0))];
        let got = eval_abs_adapter_scalar_prepared(&args);
        assert_eq!(
            got,
            Err(AbsEvalError::Coercion(CoercionError::WorksheetError(
                WorksheetErrorCode::Div0
            )))
        );
    }

    #[test]
    fn abs_kernel_negative_zero_is_normalized_to_positive_zero() {
        let got = abs_kernel(-0.0);
        assert_eq!(got.to_bits(), 0.0f64.to_bits());
        assert_eq!(got, 0.0);
    }

    #[test]
    fn abs_kernel_infinity_and_nan_behavior() {
        assert!(abs_kernel(f64::INFINITY).is_infinite());
        assert_eq!(abs_kernel(f64::INFINITY), f64::INFINITY);
        assert_eq!(abs_kernel(f64::NEG_INFINITY), f64::INFINITY);
        assert!(abs_kernel(f64::NAN).is_nan());
    }

    #[test]
    fn eval_abs_adapter_array_lift_prepared_preserves_per_element_outcome() {
        let args = vec![
            (CalcValue::number(-1.0)),
            text_prepared("asd"),
            (CalcValue::logical(true)),
            (CalcValue::number(2.0)),
        ];

        let got = eval_abs_adapter_array_lift_prepared(&args);
        assert_eq!(
            got,
            vec![
                AbsLiftOutcome::Number(1.0),
                AbsLiftOutcome::Error(CoercionError::NonNumericText("asd".to_string())),
                AbsLiftOutcome::Number(1.0),
                AbsLiftOutcome::Number(2.0),
            ]
        );
    }

    #[test]
    fn eval_abs_adapter_scalar_prepared_value_wraps_scalar_result_as_eval_number() {
        let args = [(CalcValue::number(-3.0))];
        let got = eval_abs_adapter_scalar_prepared_value(&args);
        assert_eq!(got, Ok(CalcValue::number(3.0)));
    }

    #[test]
    fn eval_abs_adapter_scalar_prepared_is_values_only_and_needs_no_resolver() {
        let args = [(CalcValue::number(-9.0))];
        let got = eval_abs_adapter_scalar_prepared(&args);
        assert_eq!(got, Ok(9.0));
    }

    #[test]
    fn eval_abs_adapter_array_lift_prepared_maps_without_reference_seam() {
        let args = vec![
            (CalcValue::number(-4.0)),
            (CalcValue::text(ExcelText::from_utf16_code_units(
                "bad".encode_utf16().collect(),
            ))),
        ];
        let got = eval_abs_adapter_array_lift_prepared(&args);
        assert_eq!(
            got,
            vec![
                AbsLiftOutcome::Number(4.0),
                AbsLiftOutcome::Error(CoercionError::NonNumericText("bad".to_string()))
            ]
        );
    }

    #[test]
    fn abs_coercion_lift_profile_is_declared() {
        assert_eq!(
            ABS_COERCION_LIFT_PROFILE,
            UnaryNumericCoercionLiftProfile::ScalarOrArrayElementwise
        );
        assert_eq!(
            ABS_META.coercion_lift_profile,
            CoercionLiftProfile::UnaryNumericScalarOrArrayElementwise
        );
    }

    #[test]
    fn eval_abs_scalar_rejects_non_unary_calls() {
        assert_eq!(
            eval_abs_scalar(&[], &resolver()),
            Err(AbsEvalError::ArityMismatch {
                expected: ABS_META.arity.min,
                actual: 0
            })
        );

        let args = [(CalcValue::number(1.0)), (CalcValue::number(2.0))];
        assert_eq!(
            eval_abs_scalar(&args, &resolver()),
            Err(AbsEvalError::ArityMismatch {
                expected: ABS_META.arity.min,
                actual: 2
            })
        );
    }

    #[test]
    fn eval_abs_scalar_on_number() {
        let args = [(CalcValue::number(-2.5))];
        let got = eval_abs_scalar(&args, &resolver());
        assert_eq!(got, Ok(2.5));
    }

    #[test]
    fn eval_abs_scalar_text_numeric_is_coerced() {
        let args = [text_arg(" -2 ")];
        let got = eval_abs_scalar(&args, &resolver());
        assert_eq!(got, Ok(2.0));
    }

    #[test]
    fn eval_abs_scalar_non_numeric_text_returns_coercion_error() {
        let args = [text_arg("asd")];
        let got = eval_abs_scalar(&args, &resolver());
        assert_eq!(
            got,
            Err(AbsEvalError::Coercion(CoercionError::NonNumericText(
                "asd".to_string()
            )))
        );
    }

    #[test]
    fn eval_abs_scalar_reference_uses_resolver() {
        let r = MockResolver {
            caps: ReferenceSystemCapabilities::permissive_local(),
            resolved_value: Some(CalcValue::number(-7.0)),
        };
        let args = [CalcValue::reference(ReferenceLike::new(
            ReferenceKind::A1,
            "A1".to_string(),
        ))];

        let got = eval_abs_scalar(&args, &r);
        assert_eq!(got, Ok(7.0));
    }

    #[test]
    fn eval_abs_scalar_propagates_worksheet_error_via_coercion() {
        let args = [(CalcValue::error(WorksheetErrorCode::Div0))];
        let got = eval_abs_scalar(&args, &resolver());
        assert_eq!(
            got,
            Err(AbsEvalError::Coercion(CoercionError::WorksheetError(
                WorksheetErrorCode::Div0
            )))
        );
    }

    #[test]
    fn eval_abs_array_lift_preserves_per_element_outcome() {
        let args = vec![
            (CalcValue::number(-1.0)),
            text_arg("asd"),
            (CalcValue::logical(true)),
            (CalcValue::number(2.0)),
        ];

        let got = eval_abs_array_lift(&args, &resolver());
        assert_eq!(
            got,
            vec![
                AbsLiftOutcome::Number(1.0),
                AbsLiftOutcome::Error(CoercionError::NonNumericText("asd".to_string())),
                AbsLiftOutcome::Number(1.0),
                AbsLiftOutcome::Number(2.0),
            ]
        );
    }

    #[test]
    fn eval_abs_scalar_value_wraps_scalar_result_as_eval_number() {
        let args = [(CalcValue::number(-3.0))];
        let got = eval_abs_scalar_value(&args, &resolver());
        assert_eq!(got, Ok(CalcValue::number(3.0)));
    }

    #[test]
    fn eval_abs_surface_scalar_matches_adapter_for_prepared_numeric_input() {
        let args = [(CalcValue::number(-5.0))];
        let surface = eval_abs_scalar(&args, &resolver());
        let adapter = eval_abs_adapter_scalar_prepared(&[(CalcValue::number(-5.0))]);
        assert_eq!(surface, adapter);
    }
}

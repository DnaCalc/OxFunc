use crate::coercion::{CoercionError, coerce_arg_to_number};
use crate::function::{
    Arity, DeterminismClass, FecDependencyProfile, FunctionMeta, HostInteractionClass,
    ThreadSafetyClass, VolatilityClass,
};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue};

pub const ABS_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.ABS",
    arity: Arity::exact(1),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    fec_dependency_profile: FecDependencyProfile::RefOnly,
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

pub fn abs_kernel(n: f64) -> f64 {
    n.abs()
}

pub fn eval_abs_scalar(args: &[CallArgValue], resolver: &impl ReferenceResolver) -> Result<f64, AbsEvalError> {
    if !ABS_META.arity.accepts(args.len()) {
        return Err(AbsEvalError::ArityMismatch {
            expected: ABS_META.arity.min,
            actual: args.len(),
        });
    }

    let n = coerce_arg_to_number(&args[0], resolver).map_err(AbsEvalError::Coercion)?;
    Ok(abs_kernel(n))
}

pub fn eval_abs_scalar_value(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, AbsEvalError> {
    eval_abs_scalar(args, resolver).map(EvalValue::Number)
}

pub fn eval_abs_array_lift(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Vec<AbsLiftOutcome> {
    args.iter()
        .map(|arg| match coerce_arg_to_number(arg, resolver) {
            Ok(n) => AbsLiftOutcome::Number(abs_kernel(n)),
            Err(e) => AbsLiftOutcome::Error(e),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::{RefResolutionError, ResolverCapabilities};
    use crate::value::{EvalValue, ExcelText, ReferenceKind, ReferenceLike, WorksheetErrorCode};

    struct MockResolver {
        caps: ResolverCapabilities,
        resolved_value: Option<EvalValue>,
    }

    impl ReferenceResolver for MockResolver {
        fn capabilities(&self) -> ResolverCapabilities {
            self.caps
        }

        fn resolve_reference(
            &self,
            reference: &ReferenceLike,
        ) -> Result<EvalValue, RefResolutionError> {
            self.resolved_value
                .clone()
                .ok_or(RefResolutionError::UnresolvedReference {
                    target: reference.target.clone(),
                })
        }
    }

    fn resolver() -> MockResolver {
        MockResolver {
            caps: ResolverCapabilities::permissive_local(),
            resolved_value: None,
        }
    }

    fn text_arg(s: &str) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
            s.encode_utf16().collect(),
        )))
    }

    #[test]
    fn abs_meta_matches_w5_contract_shape() {
        assert_eq!(ABS_META.function_id, "FUNC.ABS");
        assert_eq!(ABS_META.arity, Arity::exact(1));
        assert_eq!(ABS_META.determinism, DeterminismClass::Deterministic);
        assert_eq!(ABS_META.volatility, VolatilityClass::NonVolatile);
        assert_eq!(ABS_META.host_interaction, HostInteractionClass::None);
        assert_eq!(ABS_META.thread_safety, ThreadSafetyClass::SafePure);
        assert_eq!(ABS_META.fec_dependency_profile, FecDependencyProfile::RefOnly);
    }

    #[test]
    fn eval_abs_scalar_rejects_non_unary_calls() {
        assert_eq!(
            eval_abs_scalar(&[], &resolver()),
            Err(AbsEvalError::ArityMismatch {
                expected: 1,
                actual: 0
            })
        );

        let args = [
            CallArgValue::Eval(EvalValue::Number(1.0)),
            CallArgValue::Eval(EvalValue::Number(2.0)),
        ];
        assert_eq!(
            eval_abs_scalar(&args, &resolver()),
            Err(AbsEvalError::ArityMismatch {
                expected: 1,
                actual: 2
            })
        );
    }

    #[test]
    fn eval_abs_scalar_on_number() {
        let args = [CallArgValue::Eval(EvalValue::Number(-2.5))];
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
            caps: ResolverCapabilities::permissive_local(),
            resolved_value: Some(EvalValue::Number(-7.0)),
        };
        let args = [CallArgValue::Reference(ReferenceLike {
            kind: ReferenceKind::A1,
            target: "A1".to_string(),
        })];

        let got = eval_abs_scalar(&args, &r);
        assert_eq!(got, Ok(7.0));
    }

    #[test]
    fn eval_abs_scalar_propagates_worksheet_error_via_coercion() {
        let args = [CallArgValue::Eval(EvalValue::Error(WorksheetErrorCode::Div0))];
        let got = eval_abs_scalar(&args, &resolver());
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
    fn eval_abs_array_lift_preserves_per_element_outcome() {
        let args = vec![
            CallArgValue::Eval(EvalValue::Number(-1.0)),
            text_arg("asd"),
            CallArgValue::Eval(EvalValue::Logical(true)),
            CallArgValue::Eval(EvalValue::Number(2.0)),
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
        let args = [CallArgValue::Eval(EvalValue::Number(-3.0))];
        let got = eval_abs_scalar_value(&args, &resolver());
        assert_eq!(got, Ok(EvalValue::Number(3.0)));
    }
}

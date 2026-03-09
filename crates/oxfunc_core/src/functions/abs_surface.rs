use crate::functions::abs::{
    AbsEvalError, AbsLiftOutcome, eval_abs_adapter_arg_prepared, eval_abs_adapter_scalar_prepared,
    eval_abs_adapter_scalar_prepared_value,
};
use crate::functions::adapters::{prepare_arg_values_only, prepare_args_values_only};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue};

pub fn eval_abs_scalar(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<f64, AbsEvalError> {
    let prepared = prepare_args_values_only(args, resolver).map_err(AbsEvalError::Coercion)?;
    eval_abs_adapter_scalar_prepared(&prepared)
}

pub fn eval_abs_scalar_value(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, AbsEvalError> {
    let prepared = prepare_args_values_only(args, resolver).map_err(AbsEvalError::Coercion)?;
    eval_abs_adapter_scalar_prepared_value(&prepared)
}

pub fn eval_abs_array_lift(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Vec<AbsLiftOutcome> {
    args.iter()
        .map(|arg| match prepare_arg_values_only(arg, resolver) {
            Ok(prepared) => eval_abs_adapter_arg_prepared(&prepared),
            Err(e) => AbsLiftOutcome::Error(e),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coercion::CoercionError;
    use crate::functions::abs::ABS_META;
    use crate::resolver::{RefResolutionError, ResolverCapabilities};
    use crate::value::{ExcelText, ReferenceKind, ReferenceLike, WorksheetErrorCode};

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
    fn eval_abs_scalar_rejects_non_unary_calls() {
        assert_eq!(
            eval_abs_scalar(&[], &resolver()),
            Err(AbsEvalError::ArityMismatch {
                expected: ABS_META.arity.min,
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
                expected: ABS_META.arity.min,
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
        let args = [CallArgValue::Eval(EvalValue::Error(
            WorksheetErrorCode::Div0,
        ))];
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

    #[test]
    fn eval_abs_surface_scalar_matches_adapter_for_prepared_numeric_input() {
        let args = [CallArgValue::Eval(EvalValue::Number(-5.0))];
        let surface = eval_abs_scalar(&args, &resolver());
        let adapter = eval_abs_adapter_scalar_prepared(&[
            crate::functions::adapters::PreparedArgValue::Eval(EvalValue::Number(-5.0)),
        ]);
        assert_eq!(surface, adapter);
    }
}

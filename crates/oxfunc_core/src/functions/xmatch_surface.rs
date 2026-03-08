use crate::functions::adapters::prepare_arg_values_only;
use crate::functions::xmatch::{
    XmatchEvalError, eval_xmatch_adapter_prepared, eval_xmatch_adapter_prepared_value,
    validate_xmatch_surface_arity,
};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue};

pub fn eval_xmatch_surface(
    lookup_value: &CallArgValue,
    lookup_array: &[CallArgValue],
    match_mode: Option<&CallArgValue>,
    search_mode: Option<&CallArgValue>,
    resolver: &impl ReferenceResolver,
) -> Result<f64, XmatchEvalError> {
    let argc = 2 + usize::from(match_mode.is_some()) + usize::from(search_mode.is_some());
    validate_xmatch_surface_arity(argc)?;

    let prepared_lookup_value =
        prepare_arg_values_only(lookup_value, resolver).map_err(XmatchEvalError::Coercion)?;
    let prepared_lookup_array = lookup_array
        .iter()
        .map(|arg| prepare_arg_values_only(arg, resolver))
        .collect::<Result<Vec<_>, _>>()
        .map_err(XmatchEvalError::Coercion)?;
    let prepared_match_mode = match_mode
        .map(|arg| prepare_arg_values_only(arg, resolver))
        .transpose()
        .map_err(XmatchEvalError::Coercion)?;
    let prepared_search_mode = search_mode
        .map(|arg| prepare_arg_values_only(arg, resolver))
        .transpose()
        .map_err(XmatchEvalError::Coercion)?;

    eval_xmatch_adapter_prepared(
        &prepared_lookup_value,
        &prepared_lookup_array,
        prepared_match_mode.as_ref(),
        prepared_search_mode.as_ref(),
    )
}

pub fn eval_xmatch_surface_value(
    lookup_value: &CallArgValue,
    lookup_array: &[CallArgValue],
    match_mode: Option<&CallArgValue>,
    search_mode: Option<&CallArgValue>,
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, XmatchEvalError> {
    let argc = 2 + usize::from(match_mode.is_some()) + usize::from(search_mode.is_some());
    validate_xmatch_surface_arity(argc)?;

    let prepared_lookup_value =
        prepare_arg_values_only(lookup_value, resolver).map_err(XmatchEvalError::Coercion)?;
    let prepared_lookup_array = lookup_array
        .iter()
        .map(|arg| prepare_arg_values_only(arg, resolver))
        .collect::<Result<Vec<_>, _>>()
        .map_err(XmatchEvalError::Coercion)?;
    let prepared_match_mode = match_mode
        .map(|arg| prepare_arg_values_only(arg, resolver))
        .transpose()
        .map_err(XmatchEvalError::Coercion)?;
    let prepared_search_mode = search_mode
        .map(|arg| prepare_arg_values_only(arg, resolver))
        .transpose()
        .map_err(XmatchEvalError::Coercion)?;

    eval_xmatch_adapter_prepared_value(
        &prepared_lookup_value,
        &prepared_lookup_array,
        prepared_match_mode.as_ref(),
        prepared_search_mode.as_ref(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coercion::CoercionError;
    use crate::function::Arity;
    use crate::functions::adapters::PreparedArgValue;
    use crate::functions::xmatch::XMATCH_META;
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
    fn eval_xmatch_surface_uses_reference_preparation_for_lookup_value() {
        let r = MockResolver {
            caps: ResolverCapabilities::permissive_local(),
            resolved_value: Some(EvalValue::Number(2.0)),
        };

        let got = eval_xmatch_surface(
            &CallArgValue::Reference(ReferenceLike {
                kind: ReferenceKind::A1,
                target: "A1".to_string(),
            }),
            &[CallArgValue::Eval(EvalValue::Number(1.0)), CallArgValue::Eval(EvalValue::Number(2.0))],
            None,
            None,
            &r,
        );
        assert_eq!(got, Ok(2.0));
    }

    #[test]
    fn eval_xmatch_surface_uses_reference_preparation_for_lookup_array() {
        let r = MockResolver {
            caps: ResolverCapabilities::permissive_local(),
            resolved_value: Some(EvalValue::Number(7.0)),
        };

        let got = eval_xmatch_surface(
            &CallArgValue::Eval(EvalValue::Number(7.0)),
            &[
                CallArgValue::Reference(ReferenceLike {
                    kind: ReferenceKind::A1,
                    target: "A1".to_string(),
                }),
                CallArgValue::Eval(EvalValue::Number(9.0)),
            ],
            None,
            None,
            &r,
        );
        assert_eq!(got, Ok(1.0));
    }

    #[test]
    fn eval_xmatch_surface_propagates_reference_resolution_error() {
        let got = eval_xmatch_surface(
            &CallArgValue::Reference(ReferenceLike {
                kind: ReferenceKind::A1,
                target: "A1".to_string(),
            }),
            &[CallArgValue::Eval(EvalValue::Number(1.0))],
            None,
            None,
            &resolver(),
        );
        assert_eq!(
            got,
            Err(XmatchEvalError::Coercion(CoercionError::RefResolution(
                RefResolutionError::UnresolvedReference {
                    target: "A1".to_string()
                }
            )))
        );
    }

    #[test]
    fn eval_xmatch_surface_value_wraps_index_as_eval_number() {
        let got = eval_xmatch_surface_value(
            &CallArgValue::Eval(EvalValue::Number(3.0)),
            &[CallArgValue::Eval(EvalValue::Number(3.0))],
            None,
            None,
            &resolver(),
        );
        assert_eq!(got, Ok(EvalValue::Number(1.0)));
    }

    #[test]
    fn eval_xmatch_surface_scalar_matches_adapter_on_prepared_numeric_case() {
        let lookup_value = CallArgValue::Eval(EvalValue::Number(2.0));
        let lookup_array = [
            CallArgValue::Eval(EvalValue::Number(1.0)),
            CallArgValue::Eval(EvalValue::Number(2.0)),
        ];

        let surface = eval_xmatch_surface(
            &lookup_value,
            &lookup_array,
            None,
            None,
            &resolver(),
        );

        let prepared_lookup_value = PreparedArgValue::Eval(EvalValue::Number(2.0));
        let prepared_lookup_array = vec![
            PreparedArgValue::Eval(EvalValue::Number(1.0)),
            PreparedArgValue::Eval(EvalValue::Number(2.0)),
        ];
        let adapter = eval_xmatch_adapter_prepared(
            &prepared_lookup_value,
            &prepared_lookup_array,
            None,
            None,
        );
        assert_eq!(surface, adapter);
    }

    #[test]
    fn eval_xmatch_surface_search_mode_uses_prepared_coercion() {
        let got = eval_xmatch_surface(
            &CallArgValue::Eval(EvalValue::Number(2.0)),
            &[
                CallArgValue::Eval(EvalValue::Number(2.0)),
                CallArgValue::Eval(EvalValue::Number(2.0)),
            ],
            Some(&CallArgValue::Eval(EvalValue::Number(0.0))),
            Some(&CallArgValue::Eval(EvalValue::Number(-1.0))),
            &resolver(),
        );
        assert_eq!(got, Ok(2.0));
    }

    #[test]
    fn eval_xmatch_surface_coercion_error_from_mode_is_propagated() {
        let got = eval_xmatch_surface(
            &CallArgValue::Eval(EvalValue::Number(1.0)),
            &[CallArgValue::Eval(EvalValue::Number(1.0))],
            Some(&text_arg("asd")),
            None,
            &resolver(),
        );
        assert_eq!(
            got,
            Err(XmatchEvalError::Coercion(CoercionError::NonNumericText(
                "asd".to_string()
            )))
        );
    }

    #[test]
    fn eval_xmatch_surface_lookup_array_error_is_skipped_via_adapter_lane() {
        let got = eval_xmatch_surface(
            &CallArgValue::Eval(EvalValue::Number(1.0)),
            &[CallArgValue::Eval(EvalValue::Error(WorksheetErrorCode::Value))],
            None,
            None,
            &resolver(),
        );
        assert_eq!(got, Err(XmatchEvalError::NotAvailable));
    }

    #[test]
    fn eval_xmatch_surface_lookup_value_error_is_propagated() {
        let got = eval_xmatch_surface(
            &CallArgValue::Eval(EvalValue::Error(WorksheetErrorCode::Value)),
            &[CallArgValue::Eval(EvalValue::Number(1.0))],
            None,
            None,
            &resolver(),
        );
        assert_eq!(
            got,
            Err(XmatchEvalError::Coercion(CoercionError::WorksheetError(
                WorksheetErrorCode::Value
            )))
        );
    }

    #[test]
    fn xmatch_meta_arity_is_two_to_four_in_surface_context() {
        assert_eq!(XMATCH_META.arity, Arity { min: 2, max: 4 });
    }
}

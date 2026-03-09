use crate::coercion::{CoercionError, coerce_eval_to_number};
use crate::resolver::{
    RefResolutionError, ReferenceResolver, ResolverCapabilities, resolve_eval_value,
};
use crate::value::{CallArgValue, EvalValue, ReferenceKind, ReferenceLike};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryNumericCoercionLiftProfile {
    ScalarOnly,
    ScalarOrArrayElementwise,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PreparedArgValue {
    Eval(EvalValue),
    MissingArg,
    EmptyCell,
}

fn resolve_eval_references(
    value: &EvalValue,
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, CoercionError> {
    match value {
        EvalValue::Reference(r) => {
            let resolved = resolve_eval_value(resolver, r).map_err(CoercionError::RefResolution)?;
            resolve_eval_references(&resolved, resolver)
        }
        _ => Ok(value.clone()),
    }
}

pub fn prepare_arg_values_only(
    arg: &CallArgValue,
    resolver: &impl ReferenceResolver,
) -> Result<PreparedArgValue, CoercionError> {
    match arg {
        CallArgValue::Eval(v) => Ok(PreparedArgValue::Eval(resolve_eval_references(
            v, resolver,
        )?)),
        CallArgValue::MissingArg => Ok(PreparedArgValue::MissingArg),
        CallArgValue::EmptyCell => Ok(PreparedArgValue::EmptyCell),
        CallArgValue::Reference(r) => {
            let resolved = resolve_eval_value(resolver, r).map_err(CoercionError::RefResolution)?;
            Ok(PreparedArgValue::Eval(resolve_eval_references(
                &resolved, resolver,
            )?))
        }
    }
}

pub fn prepare_args_values_only(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<Vec<PreparedArgValue>, CoercionError> {
    args.iter()
        .map(|arg| prepare_arg_values_only(arg, resolver))
        .collect()
}

struct NoReferenceResolver;

impl ReferenceResolver for NoReferenceResolver {
    fn capabilities(&self) -> ResolverCapabilities {
        ResolverCapabilities {
            allow_eval_time_deref: false,
            allow_three_d_refs: false,
            allow_structured_refs: false,
            allow_spill_anchor_refs: false,
            allow_external_refs: false,
        }
    }

    fn resolve_reference(
        &self,
        reference: &ReferenceLike,
    ) -> Result<EvalValue, RefResolutionError> {
        Err(RefResolutionError::CapabilityDenied {
            kind: match reference.kind {
                ReferenceKind::A1 => ReferenceKind::A1,
                ReferenceKind::Area => ReferenceKind::Area,
                ReferenceKind::ThreeD => ReferenceKind::ThreeD,
                ReferenceKind::Structured => ReferenceKind::Structured,
                ReferenceKind::SpillAnchor => ReferenceKind::SpillAnchor,
            },
            capability: "values_only_pre_adapter_invariant",
        })
    }
}

pub fn coerce_prepared_to_number(arg: &PreparedArgValue) -> Result<f64, CoercionError> {
    match arg {
        PreparedArgValue::Eval(v) => coerce_eval_to_number(v, &NoReferenceResolver),
        PreparedArgValue::MissingArg => Err(CoercionError::MissingArg),
        PreparedArgValue::EmptyCell => Err(CoercionError::EmptyCell),
    }
}

pub fn apply_unary_numeric_scalar_prepared(
    arg: &PreparedArgValue,
    kernel: fn(f64) -> f64,
) -> Result<f64, CoercionError> {
    let n = coerce_prepared_to_number(arg)?;
    Ok(kernel(n))
}

pub fn apply_unary_numeric_array_map_prepared(
    args: &[PreparedArgValue],
    kernel: fn(f64) -> f64,
) -> Vec<Result<f64, CoercionError>> {
    args.iter()
        .map(|arg| apply_unary_numeric_scalar_prepared(arg, kernel))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::{RefResolutionError, ResolverCapabilities};
    use crate::value::{ExcelText, ReferenceKind, WorksheetErrorCode};

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

    fn resolver_with(value: EvalValue) -> MockResolver {
        MockResolver {
            caps: ResolverCapabilities::permissive_local(),
            resolved_value: Some(value),
        }
    }

    #[test]
    fn prepare_values_only_dereferences_reference_arg() {
        let arg = CallArgValue::Reference(ReferenceLike {
            kind: ReferenceKind::A1,
            target: "A1".to_string(),
        });
        let prepared = prepare_arg_values_only(&arg, &resolver_with(EvalValue::Number(3.0)));
        assert_eq!(prepared, Ok(PreparedArgValue::Eval(EvalValue::Number(3.0))));
    }

    #[test]
    fn prepare_values_only_preserves_missing_and_empty() {
        assert_eq!(
            prepare_arg_values_only(
                &CallArgValue::MissingArg,
                &resolver_with(EvalValue::Number(1.0))
            ),
            Ok(PreparedArgValue::MissingArg)
        );
        assert_eq!(
            prepare_arg_values_only(
                &CallArgValue::EmptyCell,
                &resolver_with(EvalValue::Number(1.0))
            ),
            Ok(PreparedArgValue::EmptyCell)
        );
    }

    #[test]
    fn prepared_coercion_numeric_text_and_error_paths() {
        let text = PreparedArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
            "2".encode_utf16().collect(),
        )));
        assert_eq!(coerce_prepared_to_number(&text), Ok(2.0));

        let err = PreparedArgValue::Eval(EvalValue::Error(WorksheetErrorCode::Value));
        assert_eq!(
            coerce_prepared_to_number(&err),
            Err(CoercionError::WorksheetError(WorksheetErrorCode::Value))
        );
    }

    #[test]
    fn prepared_coercion_rejects_reference_if_invariant_broken() {
        let prepared = PreparedArgValue::Eval(EvalValue::Reference(ReferenceLike {
            kind: ReferenceKind::A1,
            target: "A1".to_string(),
        }));
        let got = coerce_prepared_to_number(&prepared);
        assert_eq!(
            got,
            Err(CoercionError::RefResolution(
                RefResolutionError::EvalTimeDerefNotAllowed
            ))
        );
    }

    #[test]
    fn unary_numeric_array_map_prepared_preserves_per_element_results() {
        let args = vec![
            PreparedArgValue::Eval(EvalValue::Number(-2.0)),
            PreparedArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
                "asd".encode_utf16().collect(),
            ))),
            PreparedArgValue::Eval(EvalValue::Logical(true)),
        ];
        let got = apply_unary_numeric_array_map_prepared(&args, f64::abs);
        assert_eq!(got.len(), 3);
        assert_eq!(got[0], Ok(2.0));
        assert_eq!(got[2], Ok(1.0));
        assert_eq!(
            got[1],
            Err(CoercionError::NonNumericText("asd".to_string()))
        );
    }
}

use crate::resolver::{RefResolutionError, ReferenceResolver, resolve_eval_value};
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AggregateScanPolicy {
    StrictAllNumeric,
    IgnoreTextAndEmpty,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoercionError {
    MissingArg,
    EmptyCell,
    NonNumericText(String),
    UnsupportedValueKind(&'static str),
    WorksheetError(WorksheetErrorCode),
    RefResolution(RefResolutionError),
}

fn parse_excel_number(text: &str) -> Option<f64> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }

    let parsed = trimmed.parse::<f64>().ok()?;
    if !parsed.is_finite() {
        return None;
    }
    Some(parsed)
}

pub fn coerce_eval_to_number(
    value: &EvalValue,
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<f64, CoercionError> {
    match value {
        EvalValue::Number(n) => Ok(*n),
        EvalValue::Logical(b) => Ok(if *b { 1.0 } else { 0.0 }),
        EvalValue::Text(t) => {
            let raw = t.to_string_lossy();
            parse_excel_number(&raw).ok_or(CoercionError::NonNumericText(raw))
        }
        EvalValue::Error(code) => Err(CoercionError::WorksheetError(*code)),
        EvalValue::Array(_) => Err(CoercionError::UnsupportedValueKind("array")),
        EvalValue::Lambda(_) => Err(CoercionError::UnsupportedValueKind("lambda_value")),
        EvalValue::Reference(reference) => {
            let resolved =
                resolve_eval_value(resolver, reference).map_err(CoercionError::RefResolution)?;
            coerce_eval_to_number(&resolved, resolver)
        }
    }
}

pub fn coerce_arg_to_number(
    arg: &CallArgValue,
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<f64, CoercionError> {
    match arg {
        CallArgValue::Eval(eval) => coerce_eval_to_number(eval, resolver),
        CallArgValue::MissingArg => Err(CoercionError::MissingArg),
        CallArgValue::EmptyCell => Err(CoercionError::EmptyCell),
        CallArgValue::Reference(reference) => {
            let resolved =
                resolve_eval_value(resolver, reference).map_err(CoercionError::RefResolution)?;
            coerce_eval_to_number(&resolved, resolver)
        }
    }
}

pub fn coerce_direct_args_to_numbers(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<Vec<f64>, CoercionError> {
    args.iter()
        .map(|arg| coerce_arg_to_number(arg, resolver))
        .collect()
}

pub fn aggregate_scan_sum(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
    policy: AggregateScanPolicy,
) -> Result<f64, CoercionError> {
    let mut acc = 0.0;
    for arg in args {
        match coerce_arg_to_number(arg, resolver) {
            Ok(n) => acc += n,
            Err(CoercionError::MissingArg | CoercionError::EmptyCell)
                if policy == AggregateScanPolicy::IgnoreTextAndEmpty => {}
            Err(CoercionError::NonNumericText(_))
                if policy == AggregateScanPolicy::IgnoreTextAndEmpty => {}
            Err(err) => return Err(err),
        }
    }
    Ok(acc)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::{ReferenceResolver, ResolverCapabilities};
    use crate::value::{EvalValue, ExcelText, ReferenceKind, ReferenceLike};

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

    #[test]
    fn coerce_text_numeric_to_number() {
        let value = EvalValue::Text(ExcelText::from_utf16_code_units(
            "1".encode_utf16().collect(),
        ));
        let got = coerce_eval_to_number(&value, &resolver());
        assert_eq!(got, Ok(1.0));
    }

    #[test]
    fn coerce_text_non_numeric_fails() {
        let value = EvalValue::Text(ExcelText::from_utf16_code_units(
            "asd".encode_utf16().collect(),
        ));
        let got = coerce_eval_to_number(&value, &resolver());
        assert_eq!(got, Err(CoercionError::NonNumericText("asd".to_string())));
    }

    #[test]
    fn missing_arg_is_distinct_error() {
        let got = coerce_arg_to_number(&CallArgValue::MissingArg, &resolver());
        assert_eq!(got, Err(CoercionError::MissingArg));
    }

    #[test]
    fn empty_cell_is_distinct_error() {
        let got = coerce_arg_to_number(&CallArgValue::EmptyCell, &resolver());
        assert_eq!(got, Err(CoercionError::EmptyCell));
    }

    #[test]
    fn reference_is_dereferenced_via_resolver() {
        let r = MockResolver {
            caps: ResolverCapabilities::permissive_local(),
            resolved_value: Some(EvalValue::Number(2.5)),
        };
        let arg = CallArgValue::Reference(ReferenceLike {
            kind: ReferenceKind::A1,
            target: "A1".to_string(),
        });

        let got = coerce_arg_to_number(&arg, &r);
        assert_eq!(got, Ok(2.5));
    }

    #[test]
    fn unresolved_reference_propagates_resolution_error() {
        let arg = CallArgValue::Reference(ReferenceLike {
            kind: ReferenceKind::A1,
            target: "A1".to_string(),
        });

        let got = coerce_arg_to_number(&arg, &resolver());
        assert_eq!(
            got,
            Err(CoercionError::RefResolution(
                RefResolutionError::UnresolvedReference {
                    target: "A1".to_string()
                }
            ))
        );
    }

    #[test]
    fn aggregate_policy_contrast_direct_vs_scan() {
        let args = vec![
            CallArgValue::Eval(EvalValue::Number(1.0)),
            CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
                "asd".encode_utf16().collect(),
            ))),
            CallArgValue::Eval(EvalValue::Number(2.0)),
        ];

        let strict = aggregate_scan_sum(&args, &resolver(), AggregateScanPolicy::StrictAllNumeric);
        assert_eq!(
            strict,
            Err(CoercionError::NonNumericText("asd".to_string()))
        );

        let relaxed =
            aggregate_scan_sum(&args, &resolver(), AggregateScanPolicy::IgnoreTextAndEmpty);
        assert_eq!(relaxed, Ok(3.0));
    }
}

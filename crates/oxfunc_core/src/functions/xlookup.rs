use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    PreparedArgValue, coerce_prepared_to_number, prepare_arg_values_only,
};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

pub const XLOOKUP_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.XLOOKUP",
    arity: Arity { min: 3, max: 6 },
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::WorkbookState,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::RefsVisibleInAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::LookupMatch,
    fec_dependency_profile: FecDependencyProfile::RefOnly,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum XlookupSearchMode {
    FirstToLast,
    LastToFirst,
}

#[derive(Debug, Clone, PartialEq)]
pub enum XlookupEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    LengthMismatch {
        lookup_len: usize,
        return_len: usize,
    },
    Coercion(CoercionError),
    UnsupportedValueKind(&'static str),
    InvalidMatchMode(f64),
    InvalidSearchMode(f64),
    UnsupportedMatchModeForSeed(f64),
    NotAvailable,
}

#[derive(Debug, Clone, PartialEq)]
enum XlookupComparable {
    Number(f64),
    Text(String),
    Logical(bool),
}

fn parse_match_mode(arg: Option<&PreparedArgValue>) -> Result<(), XlookupEvalError> {
    let Some(arg) = arg else {
        return Ok(());
    };

    let n = coerce_prepared_to_number(arg).map_err(XlookupEvalError::Coercion)?;
    if n == 0.0 {
        Ok(())
    } else if n == 1.0 || n == -1.0 || n == 2.0 {
        Err(XlookupEvalError::UnsupportedMatchModeForSeed(n))
    } else {
        Err(XlookupEvalError::InvalidMatchMode(n))
    }
}

fn parse_search_mode(
    arg: Option<&PreparedArgValue>,
) -> Result<XlookupSearchMode, XlookupEvalError> {
    let Some(arg) = arg else {
        return Ok(XlookupSearchMode::FirstToLast);
    };

    let n = coerce_prepared_to_number(arg).map_err(XlookupEvalError::Coercion)?;
    if n == 1.0 {
        Ok(XlookupSearchMode::FirstToLast)
    } else if n == -1.0 {
        Ok(XlookupSearchMode::LastToFirst)
    } else if n == 2.0 || n == -2.0 {
        Err(XlookupEvalError::InvalidSearchMode(n))
    } else {
        Err(XlookupEvalError::InvalidSearchMode(n))
    }
}

fn to_lookup_value(prepared: &PreparedArgValue) -> Result<XlookupComparable, XlookupEvalError> {
    match prepared {
        PreparedArgValue::Eval(EvalValue::Number(n)) => Ok(XlookupComparable::Number(*n)),
        PreparedArgValue::Eval(EvalValue::Text(t)) => {
            Ok(XlookupComparable::Text(t.to_string_lossy()))
        }
        PreparedArgValue::Eval(EvalValue::Logical(b)) => Ok(XlookupComparable::Logical(*b)),
        PreparedArgValue::Eval(EvalValue::Error(code)) => Err(XlookupEvalError::Coercion(
            CoercionError::WorksheetError(*code),
        )),
        PreparedArgValue::MissingArg => Err(XlookupEvalError::Coercion(CoercionError::MissingArg)),
        PreparedArgValue::EmptyCell => Err(XlookupEvalError::Coercion(CoercionError::EmptyCell)),
        PreparedArgValue::Eval(EvalValue::Array(_)) => {
            Err(XlookupEvalError::UnsupportedValueKind("array"))
        }
        PreparedArgValue::Eval(EvalValue::Reference(_)) => {
            Err(XlookupEvalError::UnsupportedValueKind("reference_like"))
        }
        PreparedArgValue::Eval(EvalValue::Lambda(_)) => {
            Err(XlookupEvalError::UnsupportedValueKind("lambda_value"))
        }
    }
}

fn to_lookup_candidate(
    prepared: &PreparedArgValue,
) -> Result<Option<XlookupComparable>, XlookupEvalError> {
    match prepared {
        PreparedArgValue::Eval(EvalValue::Number(n)) => Ok(Some(XlookupComparable::Number(*n))),
        PreparedArgValue::Eval(EvalValue::Text(t)) => {
            Ok(Some(XlookupComparable::Text(t.to_string_lossy())))
        }
        PreparedArgValue::Eval(EvalValue::Logical(b)) => Ok(Some(XlookupComparable::Logical(*b))),
        PreparedArgValue::Eval(EvalValue::Error(_)) => Ok(None),
        PreparedArgValue::MissingArg => Ok(None),
        PreparedArgValue::EmptyCell => Ok(None),
        PreparedArgValue::Eval(EvalValue::Array(_)) => {
            Err(XlookupEvalError::UnsupportedValueKind("array"))
        }
        PreparedArgValue::Eval(EvalValue::Reference(_)) => {
            Err(XlookupEvalError::UnsupportedValueKind("reference_like"))
        }
        PreparedArgValue::Eval(EvalValue::Lambda(_)) => {
            Err(XlookupEvalError::UnsupportedValueKind("lambda_value"))
        }
    }
}

fn comparable_eq(lhs: &XlookupComparable, rhs: &XlookupComparable) -> bool {
    match (lhs, rhs) {
        (XlookupComparable::Number(a), XlookupComparable::Number(b)) => a == b,
        (XlookupComparable::Text(a), XlookupComparable::Text(b)) => a == b,
        (XlookupComparable::Logical(a), XlookupComparable::Logical(b)) => a == b,
        _ => false,
    }
}

fn materialize_return_arg(
    arg: &CallArgValue,
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, XlookupEvalError> {
    match arg {
        CallArgValue::Reference(r) => Ok(EvalValue::Reference(r.clone())),
        CallArgValue::Eval(EvalValue::Reference(r)) => Ok(EvalValue::Reference(r.clone())),
        _ => {
            let prepared =
                prepare_arg_values_only(arg, resolver).map_err(XlookupEvalError::Coercion)?;
            match prepared {
                PreparedArgValue::Eval(v) => Ok(v),
                PreparedArgValue::MissingArg => Ok(EvalValue::Error(WorksheetErrorCode::NA)),
                PreparedArgValue::EmptyCell => Ok(EvalValue::Number(0.0)),
            }
        }
    }
}

pub fn eval_xlookup_surface(
    lookup_value: &CallArgValue,
    lookup_array: &[CallArgValue],
    return_array: &[CallArgValue],
    if_not_found: Option<&CallArgValue>,
    match_mode: Option<&CallArgValue>,
    search_mode: Option<&CallArgValue>,
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, XlookupEvalError> {
    let argc = 3
        + usize::from(if_not_found.is_some())
        + usize::from(match_mode.is_some())
        + usize::from(search_mode.is_some());
    if !XLOOKUP_META.arity.accepts(argc) {
        return Err(XlookupEvalError::ArityMismatch {
            expected_min: XLOOKUP_META.arity.min,
            expected_max: XLOOKUP_META.arity.max,
            actual: argc,
        });
    }

    if lookup_array.len() != return_array.len() {
        return Err(XlookupEvalError::LengthMismatch {
            lookup_len: lookup_array.len(),
            return_len: return_array.len(),
        });
    }

    if lookup_array.is_empty() {
        return Err(XlookupEvalError::NotAvailable);
    }

    let prepared_lookup =
        prepare_arg_values_only(lookup_value, resolver).map_err(XlookupEvalError::Coercion)?;
    let lookup = to_lookup_value(&prepared_lookup)?;

    let prepared_match_mode = match match_mode {
        None => None,
        Some(arg) => {
            Some(prepare_arg_values_only(arg, resolver).map_err(XlookupEvalError::Coercion)?)
        }
    };
    parse_match_mode(prepared_match_mode.as_ref())?;

    let prepared_search_mode = match search_mode {
        None => None,
        Some(arg) => {
            Some(prepare_arg_values_only(arg, resolver).map_err(XlookupEvalError::Coercion)?)
        }
    };
    let search_mode = parse_search_mode(prepared_search_mode.as_ref())?;

    let iter: Box<dyn Iterator<Item = usize>> = match search_mode {
        XlookupSearchMode::FirstToLast => Box::new(0..lookup_array.len()),
        XlookupSearchMode::LastToFirst => Box::new((0..lookup_array.len()).rev()),
    };

    for idx in iter {
        let candidate = prepare_arg_values_only(&lookup_array[idx], resolver)
            .map_err(XlookupEvalError::Coercion)?;
        if let Some(candidate) = to_lookup_candidate(&candidate)? {
            if comparable_eq(&lookup, &candidate) {
                return materialize_return_arg(&return_array[idx], resolver);
            }
        }
    }

    if let Some(if_not_found) = if_not_found {
        return materialize_return_arg(if_not_found, resolver);
    }

    Err(XlookupEvalError::NotAvailable)
}

pub fn map_xlookup_error_to_ws(e: &XlookupEvalError) -> WorksheetErrorCode {
    match e {
        XlookupEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        XlookupEvalError::LengthMismatch { .. } => WorksheetErrorCode::Value,
        XlookupEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        XlookupEvalError::UnsupportedValueKind(_) => WorksheetErrorCode::Value,
        XlookupEvalError::InvalidMatchMode(_) => WorksheetErrorCode::Value,
        XlookupEvalError::InvalidSearchMode(_) => WorksheetErrorCode::Value,
        XlookupEvalError::UnsupportedMatchModeForSeed(_) => WorksheetErrorCode::NA,
        XlookupEvalError::NotAvailable => WorksheetErrorCode::NA,
        XlookupEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::{RefResolutionError, ResolverCapabilities};
    use crate::value::{ExcelText, ReferenceKind, ReferenceLike};

    struct NoResolver;
    impl ReferenceResolver for NoResolver {
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
    }

    fn text_arg(s: &str) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
            s.encode_utf16().collect(),
        )))
    }

    #[test]
    fn eval_xlookup_exact_forward_returns_value() {
        let got = eval_xlookup_surface(
            &CallArgValue::Eval(EvalValue::Number(2.0)),
            &[
                CallArgValue::Eval(EvalValue::Number(1.0)),
                CallArgValue::Eval(EvalValue::Number(2.0)),
            ],
            &[text_arg("one"), text_arg("two")],
            None,
            None,
            None,
            &NoResolver,
        );
        assert_eq!(got, Ok(text_arg("two").into_eval().unwrap()));
    }

    #[test]
    fn eval_xlookup_no_match_uses_if_not_found() {
        let got = eval_xlookup_surface(
            &CallArgValue::Eval(EvalValue::Number(9.0)),
            &[CallArgValue::Eval(EvalValue::Number(1.0))],
            &[text_arg("one")],
            Some(&text_arg("nf")),
            None,
            None,
            &NoResolver,
        );
        assert_eq!(got, Ok(text_arg("nf").into_eval().unwrap()));
    }

    #[test]
    fn eval_xlookup_reverse_search_mode_returns_last_match() {
        let got = eval_xlookup_surface(
            &CallArgValue::Eval(EvalValue::Number(1.0)),
            &[
                CallArgValue::Eval(EvalValue::Number(1.0)),
                CallArgValue::Eval(EvalValue::Number(1.0)),
            ],
            &[text_arg("first"), text_arg("last")],
            None,
            None,
            Some(&CallArgValue::Eval(EvalValue::Number(-1.0))),
            &NoResolver,
        );
        assert_eq!(got, Ok(text_arg("last").into_eval().unwrap()));
    }

    #[test]
    fn eval_xlookup_preserves_reference_return_lane() {
        let got = eval_xlookup_surface(
            &CallArgValue::Eval(EvalValue::Number(1.0)),
            &[CallArgValue::Eval(EvalValue::Number(1.0))],
            &[CallArgValue::Reference(ReferenceLike {
                kind: ReferenceKind::Area,
                target: "B1:B3".to_string(),
            })],
            None,
            None,
            None,
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Reference(ReferenceLike {
                kind: ReferenceKind::Area,
                target: "B1:B3".to_string(),
            }))
        );
    }

    #[test]
    fn eval_xlookup_preserves_eval_wrapped_reference_return_lane() {
        let got = eval_xlookup_surface(
            &CallArgValue::Eval(EvalValue::Number(1.0)),
            &[CallArgValue::Eval(EvalValue::Number(1.0))],
            &[CallArgValue::Eval(EvalValue::Reference(ReferenceLike {
                kind: ReferenceKind::A1,
                target: "C1".to_string(),
            }))],
            None,
            None,
            None,
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Reference(ReferenceLike {
                kind: ReferenceKind::A1,
                target: "C1".to_string(),
            }))
        );
    }

    #[test]
    fn eval_xlookup_if_not_found_preserves_reference_lane() {
        let got = eval_xlookup_surface(
            &CallArgValue::Eval(EvalValue::Number(9.0)),
            &[CallArgValue::Eval(EvalValue::Number(1.0))],
            &[text_arg("one")],
            Some(&CallArgValue::Reference(ReferenceLike {
                kind: ReferenceKind::Area,
                target: "D2:D4".to_string(),
            })),
            None,
            None,
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Reference(ReferenceLike {
                kind: ReferenceKind::Area,
                target: "D2:D4".to_string(),
            }))
        );
    }

    #[test]
    fn eval_xlookup_length_mismatch_is_rejected() {
        let got = eval_xlookup_surface(
            &CallArgValue::Eval(EvalValue::Number(1.0)),
            &[CallArgValue::Eval(EvalValue::Number(1.0))],
            &[],
            None,
            None,
            None,
            &NoResolver,
        );
        assert_eq!(
            got,
            Err(XlookupEvalError::LengthMismatch {
                lookup_len: 1,
                return_len: 0,
            })
        );
    }

    trait IntoEval {
        fn into_eval(self) -> Option<EvalValue>;
    }

    impl IntoEval for CallArgValue {
        fn into_eval(self) -> Option<EvalValue> {
            match self {
                CallArgValue::Eval(v) => Some(v),
                _ => None,
            }
        }
    }
}

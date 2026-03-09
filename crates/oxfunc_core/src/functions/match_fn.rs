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

pub const MATCH_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.MATCH",
    arity: Arity { min: 2, max: 3 },
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
    coercion_lift_profile: CoercionLiftProfile::LookupMatchProfile,
    kernel_signature_class: KernelSignatureClass::LookupMatch,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
};

#[derive(Debug, Clone, PartialEq)]
pub enum MatchEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    EmptyLookupArray,
    Coercion(CoercionError),
    UnsupportedValueKind(&'static str),
    UnsupportedMatchTypeForSeed(f64),
    NotAvailable,
}

#[derive(Debug, Clone, PartialEq)]
enum MatchComparable {
    Number(f64),
    Text(String),
    Logical(bool),
}

fn to_lookup_value(prepared: &PreparedArgValue) -> Result<MatchComparable, MatchEvalError> {
    match prepared {
        PreparedArgValue::Eval(EvalValue::Number(n)) => Ok(MatchComparable::Number(*n)),
        PreparedArgValue::Eval(EvalValue::Text(t)) => {
            Ok(MatchComparable::Text(t.to_string_lossy()))
        }
        PreparedArgValue::Eval(EvalValue::Logical(b)) => Ok(MatchComparable::Logical(*b)),
        PreparedArgValue::Eval(EvalValue::Error(code)) => Err(MatchEvalError::Coercion(
            CoercionError::WorksheetError(*code),
        )),
        PreparedArgValue::MissingArg => Err(MatchEvalError::Coercion(CoercionError::MissingArg)),
        PreparedArgValue::EmptyCell => Err(MatchEvalError::Coercion(CoercionError::EmptyCell)),
        PreparedArgValue::Eval(EvalValue::Array(_)) => {
            Err(MatchEvalError::UnsupportedValueKind("array"))
        }
        PreparedArgValue::Eval(EvalValue::Reference(_)) => {
            Err(MatchEvalError::UnsupportedValueKind("reference_like"))
        }
        PreparedArgValue::Eval(EvalValue::Lambda(_)) => {
            Err(MatchEvalError::UnsupportedValueKind("lambda_value"))
        }
    }
}

fn to_lookup_candidate(
    prepared: &PreparedArgValue,
) -> Result<Option<MatchComparable>, MatchEvalError> {
    match prepared {
        PreparedArgValue::Eval(EvalValue::Number(n)) => Ok(Some(MatchComparable::Number(*n))),
        PreparedArgValue::Eval(EvalValue::Text(t)) => {
            Ok(Some(MatchComparable::Text(t.to_string_lossy())))
        }
        PreparedArgValue::Eval(EvalValue::Logical(b)) => Ok(Some(MatchComparable::Logical(*b))),
        PreparedArgValue::Eval(EvalValue::Error(_)) => Ok(None),
        PreparedArgValue::MissingArg => Ok(None),
        PreparedArgValue::EmptyCell => Ok(None),
        PreparedArgValue::Eval(EvalValue::Array(_)) => {
            Err(MatchEvalError::UnsupportedValueKind("array"))
        }
        PreparedArgValue::Eval(EvalValue::Reference(_)) => {
            Err(MatchEvalError::UnsupportedValueKind("reference_like"))
        }
        PreparedArgValue::Eval(EvalValue::Lambda(_)) => {
            Err(MatchEvalError::UnsupportedValueKind("lambda_value"))
        }
    }
}

fn comparable_eq(lhs: &MatchComparable, rhs: &MatchComparable) -> bool {
    match (lhs, rhs) {
        (MatchComparable::Number(a), MatchComparable::Number(b)) => a == b,
        (MatchComparable::Text(a), MatchComparable::Text(b)) => a == b,
        (MatchComparable::Logical(a), MatchComparable::Logical(b)) => a == b,
        _ => false,
    }
}

pub fn eval_match_surface(
    lookup_value: &CallArgValue,
    lookup_array: &[CallArgValue],
    match_type: Option<&CallArgValue>,
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, MatchEvalError> {
    let argc = 2 + usize::from(match_type.is_some());
    if !MATCH_META.arity.accepts(argc) {
        return Err(MatchEvalError::ArityMismatch {
            expected_min: MATCH_META.arity.min,
            expected_max: MATCH_META.arity.max,
            actual: argc,
        });
    }

    if lookup_array.is_empty() {
        return Err(MatchEvalError::EmptyLookupArray);
    }

    let match_type_num = match match_type {
        None => 0.0,
        Some(arg) => {
            let prepared =
                prepare_arg_values_only(arg, resolver).map_err(MatchEvalError::Coercion)?;
            coerce_prepared_to_number(&prepared).map_err(MatchEvalError::Coercion)?
        }
    };
    if match_type_num != 0.0 {
        return Err(MatchEvalError::UnsupportedMatchTypeForSeed(match_type_num));
    }

    let prepared_lookup =
        prepare_arg_values_only(lookup_value, resolver).map_err(MatchEvalError::Coercion)?;
    let lookup = to_lookup_value(&prepared_lookup)?;

    for (idx, raw) in lookup_array.iter().enumerate() {
        let prepared = prepare_arg_values_only(raw, resolver).map_err(MatchEvalError::Coercion)?;
        if let Some(candidate) = to_lookup_candidate(&prepared)? {
            if comparable_eq(&lookup, &candidate) {
                return Ok(EvalValue::Number((idx + 1) as f64));
            }
        }
    }

    Err(MatchEvalError::NotAvailable)
}

pub fn map_match_error_to_ws(e: &MatchEvalError) -> WorksheetErrorCode {
    match e {
        MatchEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        MatchEvalError::EmptyLookupArray => WorksheetErrorCode::NA,
        MatchEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        MatchEvalError::UnsupportedValueKind(_) => WorksheetErrorCode::Value,
        MatchEvalError::UnsupportedMatchTypeForSeed(_) => WorksheetErrorCode::NA,
        MatchEvalError::NotAvailable => WorksheetErrorCode::NA,
        MatchEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::{RefResolutionError, ResolverCapabilities};
    use crate::value::{ExcelText, ReferenceLike};

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
    fn eval_match_exact_returns_first_index() {
        let got = eval_match_surface(
            &CallArgValue::Eval(EvalValue::Number(3.0)),
            &[
                CallArgValue::Eval(EvalValue::Number(1.0)),
                CallArgValue::Eval(EvalValue::Number(3.0)),
            ],
            None,
            &NoResolver,
        );
        assert_eq!(got, Ok(EvalValue::Number(2.0)));
    }

    #[test]
    fn eval_match_skips_candidate_errors_in_exact_seed() {
        let got = eval_match_surface(
            &CallArgValue::Eval(EvalValue::Number(3.0)),
            &[
                CallArgValue::Eval(EvalValue::Error(WorksheetErrorCode::Div0)),
                CallArgValue::Eval(EvalValue::Number(3.0)),
            ],
            None,
            &NoResolver,
        );
        assert_eq!(got, Ok(EvalValue::Number(2.0)));
    }

    #[test]
    fn eval_match_non_exact_mode_is_explicitly_unsupported_for_seed() {
        let got = eval_match_surface(
            &CallArgValue::Eval(EvalValue::Number(3.0)),
            &[CallArgValue::Eval(EvalValue::Number(3.0))],
            Some(&CallArgValue::Eval(EvalValue::Number(1.0))),
            &NoResolver,
        );
        assert_eq!(got, Err(MatchEvalError::UnsupportedMatchTypeForSeed(1.0)));
    }

    #[test]
    fn eval_match_text_is_case_sensitive() {
        let got = eval_match_surface(
            &text_arg("Abc"),
            &[text_arg("abc"), text_arg("Abc")],
            None,
            &NoResolver,
        );
        assert_eq!(got, Ok(EvalValue::Number(2.0)));
    }
}

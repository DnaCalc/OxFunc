use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{PreparedArgValue, coerce_prepared_to_number};
use crate::value::EvalValue;
use std::cmp::Ordering;

pub const XMATCH_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.XMATCH",
    arity: Arity { min: 2, max: 4 },
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XmatchMatchMode {
    Exact,
    ExactOrNextLarger,
    ExactOrNextSmaller,
    Wildcard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XmatchSearchMode {
    FirstToLast,
    LastToFirst,
    BinaryAscending,
    BinaryDescending,
}

#[derive(Debug, Clone, PartialEq)]
pub enum XmatchEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    EmptyLookupArray,
    MissingArg,
    EmptyCell,
    Coercion(CoercionError),
    UnsupportedValueKind(&'static str),
    InvalidMatchMode(f64),
    InvalidSearchMode(f64),
    UnsupportedMatchModeForSeed(XmatchMatchMode),
    UnsupportedSearchModeForSeed(XmatchSearchMode),
    NotAvailable,
}

#[derive(Debug, Clone, PartialEq)]
pub enum XmatchComparable {
    Number(f64),
    Text(String),
    Logical(bool),
}

fn parse_match_mode(n: f64) -> Result<XmatchMatchMode, XmatchEvalError> {
    if n == 0.0 {
        return Ok(XmatchMatchMode::Exact);
    }
    if n == 1.0 {
        return Ok(XmatchMatchMode::ExactOrNextLarger);
    }
    if n == -1.0 {
        return Ok(XmatchMatchMode::ExactOrNextSmaller);
    }
    if n == 2.0 {
        return Ok(XmatchMatchMode::Wildcard);
    }
    Err(XmatchEvalError::InvalidMatchMode(n))
}

fn parse_search_mode(n: f64) -> Result<XmatchSearchMode, XmatchEvalError> {
    if n == 1.0 {
        return Ok(XmatchSearchMode::FirstToLast);
    }
    if n == -1.0 {
        return Ok(XmatchSearchMode::LastToFirst);
    }
    if n == 2.0 {
        return Ok(XmatchSearchMode::BinaryAscending);
    }
    if n == -2.0 {
        return Ok(XmatchSearchMode::BinaryDescending);
    }
    Err(XmatchEvalError::InvalidSearchMode(n))
}

fn to_lookup_value_comparable(
    prepared: &PreparedArgValue,
) -> Result<XmatchComparable, XmatchEvalError> {
    match prepared {
        PreparedArgValue::Eval(EvalValue::Number(n)) => Ok(XmatchComparable::Number(*n)),
        PreparedArgValue::Eval(EvalValue::Text(t)) => {
            Ok(XmatchComparable::Text(t.to_string_lossy()))
        }
        PreparedArgValue::Eval(EvalValue::Logical(b)) => Ok(XmatchComparable::Logical(*b)),
        PreparedArgValue::Eval(EvalValue::Error(code)) => Err(XmatchEvalError::Coercion(
            CoercionError::WorksheetError(*code),
        )),
        PreparedArgValue::Eval(EvalValue::Array(_)) => {
            Err(XmatchEvalError::UnsupportedValueKind("array"))
        }
        PreparedArgValue::Eval(EvalValue::Reference(_)) => {
            Err(XmatchEvalError::UnsupportedValueKind("reference_like"))
        }
        PreparedArgValue::Eval(EvalValue::Lambda(_)) => {
            Err(XmatchEvalError::UnsupportedValueKind("lambda_value"))
        }
        PreparedArgValue::MissingArg => Err(XmatchEvalError::MissingArg),
        PreparedArgValue::EmptyCell => Err(XmatchEvalError::EmptyCell),
    }
}

fn to_lookup_candidate_comparable(
    prepared: &PreparedArgValue,
) -> Result<Option<XmatchComparable>, XmatchEvalError> {
    match prepared {
        PreparedArgValue::Eval(EvalValue::Number(n)) => Ok(Some(XmatchComparable::Number(*n))),
        PreparedArgValue::Eval(EvalValue::Text(t)) => {
            Ok(Some(XmatchComparable::Text(t.to_string_lossy())))
        }
        PreparedArgValue::Eval(EvalValue::Logical(b)) => Ok(Some(XmatchComparable::Logical(*b))),
        PreparedArgValue::Eval(EvalValue::Error(_)) => Ok(None),
        PreparedArgValue::MissingArg => Ok(None),
        PreparedArgValue::EmptyCell => Ok(None),
        PreparedArgValue::Eval(EvalValue::Array(_)) => {
            Err(XmatchEvalError::UnsupportedValueKind("array"))
        }
        PreparedArgValue::Eval(EvalValue::Reference(_)) => {
            Err(XmatchEvalError::UnsupportedValueKind("reference_like"))
        }
        PreparedArgValue::Eval(EvalValue::Lambda(_)) => {
            Err(XmatchEvalError::UnsupportedValueKind("lambda_value"))
        }
    }
}

fn parse_optional_match_mode(
    mode: Option<&PreparedArgValue>,
) -> Result<XmatchMatchMode, XmatchEvalError> {
    match mode {
        None => Ok(XmatchMatchMode::Exact),
        Some(p) => {
            parse_match_mode(coerce_prepared_to_number(p).map_err(XmatchEvalError::Coercion)?)
        }
    }
}

fn parse_optional_search_mode(
    mode: Option<&PreparedArgValue>,
) -> Result<XmatchSearchMode, XmatchEvalError> {
    match mode {
        None => Ok(XmatchSearchMode::FirstToLast),
        Some(p) => {
            parse_search_mode(coerce_prepared_to_number(p).map_err(XmatchEvalError::Coercion)?)
        }
    }
}

fn comparable_eq(lhs: &XmatchComparable, rhs: &XmatchComparable) -> bool {
    match (lhs, rhs) {
        (XmatchComparable::Number(a), XmatchComparable::Number(b)) => a == b,
        (XmatchComparable::Text(a), XmatchComparable::Text(b)) => a.to_lowercase() == b.to_lowercase(),
        (XmatchComparable::Logical(a), XmatchComparable::Logical(b)) => a == b,
        _ => false,
    }
}

fn comparable_order(lhs: &XmatchComparable, rhs: &XmatchComparable) -> Option<Ordering> {
    match (lhs, rhs) {
        (XmatchComparable::Number(a), XmatchComparable::Number(b)) => a.partial_cmp(b),
        (XmatchComparable::Text(a), XmatchComparable::Text(b)) => {
            Some(a.to_lowercase().cmp(&b.to_lowercase()))
        }
        (XmatchComparable::Logical(a), XmatchComparable::Logical(b)) => Some(a.cmp(b)),
        _ => None,
    }
}

fn wildcard_match(pattern: &str, text: &str) -> bool {
    let pattern: Vec<char> = pattern.to_lowercase().chars().collect();
    let text: Vec<char> = text.to_lowercase().chars().collect();
    let mut pattern_index = 0usize;
    let mut text_index = 0usize;
    let mut star_index = None;
    let mut resume_text_index = 0usize;

    while text_index < text.len() {
        if pattern_index < pattern.len() {
            match pattern[pattern_index] {
                '~'
                    if pattern_index + 1 < pattern.len()
                        && pattern[pattern_index + 1] == text[text_index] =>
                {
                    pattern_index += 2;
                    text_index += 1;
                    continue;
                }
                '?' => {
                    pattern_index += 1;
                    text_index += 1;
                    continue;
                }
                '*' => {
                    star_index = Some(pattern_index);
                    pattern_index += 1;
                    resume_text_index = text_index;
                    continue;
                }
                ch if ch == text[text_index] => {
                    pattern_index += 1;
                    text_index += 1;
                    continue;
                }
                _ => {}
            }
        }

        if let Some(star) = star_index {
            pattern_index = star + 1;
            resume_text_index += 1;
            text_index = resume_text_index;
        } else {
            return false;
        }
    }

    while pattern_index < pattern.len() && pattern[pattern_index] == '*' {
        pattern_index += 1;
    }

    pattern_index == pattern.len()
}

fn scan_indices(search_mode: XmatchSearchMode, len: usize) -> Box<dyn Iterator<Item = usize>> {
    match search_mode {
        XmatchSearchMode::FirstToLast | XmatchSearchMode::BinaryAscending => Box::new(0..len),
        XmatchSearchMode::LastToFirst | XmatchSearchMode::BinaryDescending => {
            Box::new((0..len).rev())
        }
    }
}

fn candidate_matches(
    lookup_value: &XmatchComparable,
    candidate: &XmatchComparable,
    match_mode: XmatchMatchMode,
) -> bool {
    match match_mode {
        XmatchMatchMode::Wildcard => match (lookup_value, candidate) {
            (XmatchComparable::Text(pattern), XmatchComparable::Text(text)) => {
                wildcard_match(pattern, text)
            }
            _ => false,
        },
        _ => comparable_eq(lookup_value, candidate),
    }
}

fn xmatch_scan(
    lookup_value: &XmatchComparable,
    lookup_array: &[PreparedArgValue],
    match_mode: XmatchMatchMode,
    search_mode: XmatchSearchMode,
) -> Result<f64, XmatchEvalError> {
    let mut best: Option<(usize, XmatchComparable)> = None;

    for idx in scan_indices(search_mode, lookup_array.len()) {
        let Some(candidate) = to_lookup_candidate_comparable(&lookup_array[idx])? else {
            continue;
        };

        if candidate_matches(lookup_value, &candidate, match_mode) {
            return Ok((idx + 1) as f64);
        }

        let Some(order) = comparable_order(&candidate, lookup_value) else {
            continue;
        };

        match match_mode {
            XmatchMatchMode::Exact | XmatchMatchMode::Wildcard => {}
            XmatchMatchMode::ExactOrNextLarger if order == Ordering::Greater => {
                let replace = match &best {
                    None => true,
                    Some((_, current)) => comparable_order(&candidate, current)
                        .is_some_and(|candidate_vs_best| candidate_vs_best == Ordering::Less),
                };
                if replace {
                    best = Some((idx, candidate));
                }
            }
            XmatchMatchMode::ExactOrNextSmaller if order == Ordering::Less => {
                let replace = match &best {
                    None => true,
                    Some((_, current)) => comparable_order(&candidate, current)
                        .is_some_and(|candidate_vs_best| candidate_vs_best == Ordering::Greater),
                };
                if replace {
                    best = Some((idx, candidate));
                }
            }
            _ => {}
        }
    }

    best.map(|(idx, _)| (idx + 1) as f64)
        .ok_or(XmatchEvalError::NotAvailable)
}

pub fn eval_xmatch_adapter_prepared(
    lookup_value: &PreparedArgValue,
    lookup_array: &[PreparedArgValue],
    match_mode: Option<&PreparedArgValue>,
    search_mode: Option<&PreparedArgValue>,
) -> Result<f64, XmatchEvalError> {
    if lookup_array.is_empty() {
        return Err(XmatchEvalError::EmptyLookupArray);
    }

    let parsed_match_mode = parse_optional_match_mode(match_mode)?;
    let parsed_search_mode = parse_optional_search_mode(search_mode)?;
    let lookup_value = to_lookup_value_comparable(lookup_value)?;

    xmatch_scan(
        &lookup_value,
        lookup_array,
        parsed_match_mode,
        parsed_search_mode,
    )
}

pub fn eval_xmatch_adapter_prepared_value(
    lookup_value: &PreparedArgValue,
    lookup_array: &[PreparedArgValue],
    match_mode: Option<&PreparedArgValue>,
    search_mode: Option<&PreparedArgValue>,
) -> Result<EvalValue, XmatchEvalError> {
    eval_xmatch_adapter_prepared(lookup_value, lookup_array, match_mode, search_mode)
        .map(EvalValue::Number)
}

pub fn validate_xmatch_surface_arity(argc: usize) -> Result<(), XmatchEvalError> {
    if XMATCH_META.arity.accepts(argc) {
        Ok(())
    } else {
        Err(XmatchEvalError::ArityMismatch {
            expected_min: XMATCH_META.arity.min,
            expected_max: XMATCH_META.arity.max,
            actual: argc,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::{ExcelText, WorksheetErrorCode};

    fn num(n: f64) -> PreparedArgValue {
        PreparedArgValue::Eval(EvalValue::Number(n))
    }

    fn text(s: &str) -> PreparedArgValue {
        PreparedArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
            s.encode_utf16().collect(),
        )))
    }

    #[test]
    fn xmatch_meta_matches_w6_seed_shape() {
        assert_eq!(XMATCH_META.function_id, "FUNC.XMATCH");
        assert_eq!(XMATCH_META.arity, Arity { min: 2, max: 4 });
        assert_eq!(XMATCH_META.determinism, DeterminismClass::Deterministic);
        assert_eq!(XMATCH_META.volatility, VolatilityClass::NonVolatile);
        assert_eq!(XMATCH_META.host_interaction, HostInteractionClass::None);
        assert_eq!(XMATCH_META.thread_safety, ThreadSafetyClass::SafePure);
        assert_eq!(
            XMATCH_META.arg_preparation_profile,
            ArgPreparationProfile::ValuesOnlyPreAdapter
        );
        assert_eq!(
            XMATCH_META.coercion_lift_profile,
            CoercionLiftProfile::LookupMatchProfile
        );
        assert_eq!(
            XMATCH_META.kernel_signature_class,
            KernelSignatureClass::LookupMatch
        );
        assert_eq!(
            XMATCH_META.fec_dependency_profile,
            FecDependencyProfile::None
        );
        assert_eq!(
            XMATCH_META.surface_fec_dependency_profile,
            FecDependencyProfile::RefOnly
        );
    }

    #[test]
    fn validate_xmatch_surface_arity_enforces_two_to_four() {
        assert_eq!(validate_xmatch_surface_arity(2), Ok(()));
        assert_eq!(validate_xmatch_surface_arity(4), Ok(()));
        assert_eq!(
            validate_xmatch_surface_arity(1),
            Err(XmatchEvalError::ArityMismatch {
                expected_min: 2,
                expected_max: 4,
                actual: 1,
            })
        );
        assert_eq!(
            validate_xmatch_surface_arity(5),
            Err(XmatchEvalError::ArityMismatch {
                expected_min: 2,
                expected_max: 4,
                actual: 5,
            })
        );
    }

    #[test]
    fn eval_xmatch_adapter_prepared_defaults_to_exact_forward() {
        let got =
            eval_xmatch_adapter_prepared(&num(2.0), &[num(1.0), num(2.0), num(3.0)], None, None);
        assert_eq!(got, Ok(2.0));
    }

    #[test]
    fn eval_xmatch_adapter_prepared_reverse_mode_returns_last_match() {
        let got = eval_xmatch_adapter_prepared(
            &num(2.0),
            &[num(2.0), num(1.0), num(2.0)],
            Some(&num(0.0)),
            Some(&num(-1.0)),
        );
        assert_eq!(got, Ok(3.0));
    }

    #[test]
    fn eval_xmatch_adapter_prepared_not_found_returns_na_lane() {
        let got = eval_xmatch_adapter_prepared(&num(9.0), &[num(1.0), num(2.0)], None, None);
        assert_eq!(got, Err(XmatchEvalError::NotAvailable));
    }

    #[test]
    fn eval_xmatch_adapter_prepared_text_match_is_case_insensitive() {
        let got =
            eval_xmatch_adapter_prepared(&text("Abc"), &[text("abc"), text("zzz")], None, None);
        assert_eq!(got, Ok(1.0));
    }

    #[test]
    fn eval_xmatch_adapter_prepared_handles_unicode_casefold_baseline() {
        let got =
            eval_xmatch_adapter_prepared(&text("Äbc"), &[text("äBC"), text("zzz")], None, None);
        assert_eq!(got, Ok(1.0));
    }

    #[test]
    fn eval_xmatch_adapter_prepared_mixed_type_comparison_has_no_match() {
        let got = eval_xmatch_adapter_prepared(&num(1.0), &[text("1")], None, None);
        assert_eq!(got, Err(XmatchEvalError::NotAvailable));
    }

    #[test]
    fn eval_xmatch_adapter_prepared_rejects_invalid_modes() {
        let bad_match = eval_xmatch_adapter_prepared(&num(1.0), &[num(1.0)], Some(&num(9.0)), None);
        assert_eq!(bad_match, Err(XmatchEvalError::InvalidMatchMode(9.0)));

        let bad_search =
            eval_xmatch_adapter_prepared(&num(1.0), &[num(1.0)], None, Some(&num(9.0)));
        assert_eq!(bad_search, Err(XmatchEvalError::InvalidSearchMode(9.0)));
    }

    #[test]
    fn eval_xmatch_adapter_prepared_supports_wildcard_mode() {
        let wildcard =
            eval_xmatch_adapter_prepared(&text("a*"), &[text("zzz"), text("abc")], Some(&num(2.0)), None);
        assert_eq!(wildcard, Ok(2.0));
    }

    #[test]
    fn eval_xmatch_adapter_prepared_honors_wildcard_escaping() {
        let literal_star = eval_xmatch_adapter_prepared(
            &text("a~*"),
            &[text("abc"), text("a*")],
            Some(&num(2.0)),
            None,
        );
        assert_eq!(literal_star, Ok(2.0));

        let literal_question = eval_xmatch_adapter_prepared(
            &text("a~?"),
            &[text("a1"), text("a?")],
            Some(&num(2.0)),
            None,
        );
        assert_eq!(literal_question, Ok(2.0));

        let literal_tilde = eval_xmatch_adapter_prepared(
            &text("a~~b"),
            &[text("a~b"), text("ab")],
            Some(&num(2.0)),
            None,
        );
        assert_eq!(literal_tilde, Ok(1.0));
    }

    #[test]
    fn eval_xmatch_adapter_prepared_reverse_wildcard_returns_last_match() {
        let got = eval_xmatch_adapter_prepared(
            &text("a*"),
            &[text("abc"), text("def"), text("ade")],
            Some(&num(2.0)),
            Some(&num(-1.0)),
        );
        assert_eq!(got, Ok(3.0));
    }

    #[test]
    fn eval_xmatch_adapter_prepared_supports_binary_modes() {
        let ascending = eval_xmatch_adapter_prepared(
            &num(3.0),
            &[num(1.0), num(2.0), num(3.0)],
            None,
            Some(&num(2.0)),
        );
        assert_eq!(ascending, Ok(3.0));

        let descending = eval_xmatch_adapter_prepared(
            &num(3.0),
            &[num(4.0), num(3.0), num(2.0)],
            None,
            Some(&num(-2.0)),
        );
        assert_eq!(descending, Ok(2.0));
    }

    #[test]
    fn eval_xmatch_adapter_prepared_supports_approximate_modes() {
        let next_larger = eval_xmatch_adapter_prepared(
            &num(2.5),
            &[num(1.0), num(2.0), num(3.0)],
            Some(&num(1.0)),
            None,
        );
        assert_eq!(next_larger, Ok(3.0));

        let next_smaller = eval_xmatch_adapter_prepared(
            &num(2.5),
            &[num(1.0), num(2.0), num(3.0)],
            Some(&num(-1.0)),
            None,
        );
        assert_eq!(next_smaller, Ok(2.0));
    }

    #[test]
    fn eval_xmatch_adapter_prepared_approximate_modes_prefer_exact_match() {
        let exact_larger = eval_xmatch_adapter_prepared(
            &num(2.0),
            &[num(1.0), num(2.0), num(3.0)],
            Some(&num(1.0)),
            None,
        );
        assert_eq!(exact_larger, Ok(2.0));

        let exact_smaller = eval_xmatch_adapter_prepared(
            &num(2.0),
            &[num(1.0), num(2.0), num(3.0)],
            Some(&num(-1.0)),
            None,
        );
        assert_eq!(exact_smaller, Ok(2.0));
    }

    #[test]
    fn eval_xmatch_adapter_prepared_binary_modes_accept_text_and_logical_ordering() {
        let text_match = eval_xmatch_adapter_prepared(
            &text("Beta"),
            &[text("alpha"), text("beta"), text("gamma")],
            None,
            Some(&num(2.0)),
        );
        assert_eq!(text_match, Ok(2.0));

        let logical_match = eval_xmatch_adapter_prepared(
            &PreparedArgValue::Eval(EvalValue::Logical(true)),
            &[
                PreparedArgValue::Eval(EvalValue::Logical(false)),
                PreparedArgValue::Eval(EvalValue::Logical(true)),
            ],
            None,
            Some(&num(2.0)),
        );
        assert_eq!(logical_match, Ok(2.0));
    }

    #[test]
    fn eval_xmatch_adapter_prepared_propagates_lookup_value_error_lane() {
        let got = eval_xmatch_adapter_prepared(
            &PreparedArgValue::Eval(EvalValue::Error(WorksheetErrorCode::Value)),
            &[num(1.0)],
            None,
            None,
        );
        assert_eq!(
            got,
            Err(XmatchEvalError::Coercion(CoercionError::WorksheetError(
                WorksheetErrorCode::Value
            )))
        );
    }

    #[test]
    fn eval_xmatch_adapter_prepared_skips_lookup_array_errors_when_match_exists() {
        let got = eval_xmatch_adapter_prepared(
            &num(2.0),
            &[PreparedArgValue::Eval(EvalValue::Error(
                WorksheetErrorCode::Value,
            ))],
            None,
            None,
        );
        assert_eq!(got, Err(XmatchEvalError::NotAvailable));

        let got_match = eval_xmatch_adapter_prepared(
            &num(2.0),
            &[
                PreparedArgValue::Eval(EvalValue::Error(WorksheetErrorCode::Div0)),
                num(2.0),
            ],
            None,
            None,
        );
        assert_eq!(got_match, Ok(2.0));
    }

    #[test]
    fn eval_xmatch_adapter_prepared_skips_lookup_array_errors_when_no_match() {
        let got = eval_xmatch_adapter_prepared(
            &num(9.0),
            &[
                num(1.0),
                PreparedArgValue::Eval(EvalValue::Error(WorksheetErrorCode::Div0)),
                num(2.0),
            ],
            None,
            None,
        );
        assert_eq!(got, Err(XmatchEvalError::NotAvailable));
    }

    #[test]
    fn eval_xmatch_adapter_prepared_value_wraps_index_as_eval_number() {
        let got = eval_xmatch_adapter_prepared_value(&num(3.0), &[num(3.0), num(4.0)], None, None);
        assert_eq!(got, Ok(EvalValue::Number(1.0)));
    }
}

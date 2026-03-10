use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{expand_lookup_vector_arg, prepare_arg_values_only};
use crate::functions::xmatch::{
    BlankLookupBehavior, XmatchComparable, XmatchEvalError, XmatchMatchMode, comparable_eq,
    comparable_order, eval_xmatch_adapter_prepared_with_blank_behavior,
    prepared_lookup_candidate_comparable, prepared_lookup_comparable,
};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};
use std::cmp::Ordering;

pub const MATCH_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.MATCH",
    arity: Arity { min: 2, max: 3 },
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::RefsVisibleInAdapter,
    coercion_lift_profile: CoercionLiftProfile::LookupMatchProfile,
    kernel_signature_class: KernelSignatureClass::LookupMatch,
    fec_dependency_profile: FecDependencyProfile::RefOnly,
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
    InvalidMatchType(f64),
    NotAvailable,
}

fn map_xmatch_error(err: XmatchEvalError) -> MatchEvalError {
    match err {
        XmatchEvalError::ArityMismatch {
            expected_min,
            expected_max,
            actual,
        } => MatchEvalError::ArityMismatch {
            expected_min,
            expected_max,
            actual,
        },
        XmatchEvalError::EmptyLookupArray => MatchEvalError::EmptyLookupArray,
        XmatchEvalError::Coercion(err) => MatchEvalError::Coercion(err),
        XmatchEvalError::InvalidMatchMode(n) => MatchEvalError::InvalidMatchType(n),
        XmatchEvalError::InvalidSearchMode(_) => MatchEvalError::Coercion(
            CoercionError::UnsupportedValueKind("match_search_mode"),
        ),
        XmatchEvalError::NotAvailable => MatchEvalError::NotAvailable,
        XmatchEvalError::MissingArg => MatchEvalError::Coercion(CoercionError::MissingArg),
        XmatchEvalError::EmptyCell => MatchEvalError::Coercion(CoercionError::EmptyCell),
        XmatchEvalError::UnsupportedValueKind(kind) => {
            MatchEvalError::Coercion(CoercionError::UnsupportedValueKind(kind))
        }
        XmatchEvalError::UnsupportedMatchModeForSeed(mode) => MatchEvalError::InvalidMatchType(
            match mode {
                XmatchMatchMode::Exact => 0.0,
                XmatchMatchMode::ExactOrNextLarger => 1.0,
                XmatchMatchMode::ExactOrNextSmaller => -1.0,
                XmatchMatchMode::Wildcard => 2.0,
            },
        ),
        XmatchEvalError::UnsupportedSearchModeForSeed(_) => MatchEvalError::Coercion(
            CoercionError::UnsupportedValueKind("match_search_mode"),
        ),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MatchApproximateMode {
    AscendingNextSmaller,
    DescendingNextLarger,
}

fn contains_unescaped_wildcard(text: &str) -> bool {
    let chars: Vec<char> = text.chars().collect();
    let mut idx = 0usize;
    while idx < chars.len() {
        match chars[idx] {
            '*' | '?' => return true,
            '~' => {
                if idx + 1 < chars.len() && matches!(chars[idx + 1], '*' | '?' | '~') {
                    return true;
                }
                idx += 2;
                continue;
            }
            _ => {}
        }
        idx += 1;
    }
    false
}

fn match_type_to_xmatch_mode(
    lookup_value: &crate::functions::adapters::PreparedArgValue,
    match_type: f64,
) -> Result<f64, MatchEvalError> {
    if match_type == 0.0 {
        return Ok(match lookup_value {
            crate::functions::adapters::PreparedArgValue::Eval(EvalValue::Text(t))
                if contains_unescaped_wildcard(&t.to_string_lossy()) => 2.0,
            _ => 0.0,
        });
    }
    if match_type == 1.0 {
        return Ok(-1.0);
    }
    if match_type == -1.0 {
        return Ok(1.0);
    }
    Err(MatchEvalError::InvalidMatchType(match_type))
}

fn first_greater_ascending(candidates: &[XmatchComparable], lookup_value: &XmatchComparable) -> usize {
    let mut low = 1usize;
    let mut high = candidates.len();
    let mut best = 0usize;
    while low <= high {
        let mid = low + ((high - low) / 2);
        match comparable_order(&candidates[mid - 1], lookup_value) {
            Some(Ordering::Greater) => {
                best = mid;
                if mid == 1 {
                    break;
                }
                high = mid - 1;
            }
            Some(Ordering::Equal | Ordering::Less) => low = mid + 1,
            None => return 0,
        }
    }
    best
}

fn first_less_or_equal_descending(
    candidates: &[XmatchComparable],
    lookup_value: &XmatchComparable,
) -> usize {
    let mut low = 1usize;
    let mut high = candidates.len();
    let mut best = 0usize;
    while low <= high {
        let mid = low + ((high - low) / 2);
        match comparable_order(&candidates[mid - 1], lookup_value) {
            Some(Ordering::Greater) => low = mid + 1,
            Some(Ordering::Equal | Ordering::Less) => {
                best = mid;
                if mid == 1 {
                    break;
                }
                high = mid - 1;
            }
            None => return 0,
        }
    }
    best
}

fn collect_match_candidates(
    lookup_array: &[crate::functions::adapters::PreparedArgValue],
) -> Result<Vec<XmatchComparable>, MatchEvalError> {
    let mut candidates = Vec::with_capacity(lookup_array.len());
    for value in lookup_array {
        let Some(candidate) = prepared_lookup_candidate_comparable(value).map_err(map_xmatch_error)? else {
            return Err(MatchEvalError::NotAvailable);
        };
        candidates.push(candidate);
    }
    Ok(candidates)
}

fn eval_match_approximate_prepared(
    lookup_value: &crate::functions::adapters::PreparedArgValue,
    lookup_array: &[crate::functions::adapters::PreparedArgValue],
    mode: MatchApproximateMode,
) -> Result<EvalValue, MatchEvalError> {
    let lookup_value = match prepared_lookup_comparable(lookup_value) {
        Ok(value) => value,
        Err(XmatchEvalError::MissingArg | XmatchEvalError::EmptyCell) => {
            return Err(MatchEvalError::NotAvailable)
        }
        Err(err) => return Err(map_xmatch_error(err)),
    };

    let candidates = collect_match_candidates(lookup_array)?;
    if candidates.is_empty() {
        return Err(MatchEvalError::NotAvailable);
    }

    let index = match mode {
        MatchApproximateMode::AscendingNextSmaller => {
            let first_greater = first_greater_ascending(&candidates, &lookup_value);
            if first_greater == 0 {
                candidates.len()
            } else if first_greater == 1 {
                return Err(MatchEvalError::NotAvailable);
            } else {
                first_greater - 1
            }
        }
        MatchApproximateMode::DescendingNextLarger => {
            let first_less_or_equal = first_less_or_equal_descending(&candidates, &lookup_value);
            if first_less_or_equal == 0 {
                candidates.len()
            } else if comparable_eq(&candidates[first_less_or_equal - 1], &lookup_value) {
                first_less_or_equal
            } else if first_less_or_equal == 1 {
                return Err(MatchEvalError::NotAvailable);
            } else {
                first_less_or_equal - 1
            }
        }
    };

    Ok(EvalValue::Number(index as f64))
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

    let prepared_lookup_value =
        prepare_arg_values_only(lookup_value, resolver).map_err(MatchEvalError::Coercion)?;
    let mut prepared_lookup_array = Vec::new();
    for arg in lookup_array {
        prepared_lookup_array
            .extend(expand_lookup_vector_arg(arg, resolver).map_err(MatchEvalError::Coercion)?);
    }

    let prepared_match_type = match match_type {
        None => None,
        Some(arg) => Some(prepare_arg_values_only(arg, resolver).map_err(MatchEvalError::Coercion)?),
    };

    let match_type_value = match prepared_match_type.as_ref() {
        None | Some(crate::functions::adapters::PreparedArgValue::MissingArg) => None,
        Some(arg) => Some(match arg {
            crate::functions::adapters::PreparedArgValue::Eval(EvalValue::Number(n)) => *n,
            other => crate::functions::adapters::coerce_prepared_to_number(other)
                .map_err(MatchEvalError::Coercion)?,
        }),
    };

    match match_type_value {
        None | Some(1.0) => eval_match_approximate_prepared(
            &prepared_lookup_value,
            &prepared_lookup_array,
            MatchApproximateMode::AscendingNextSmaller,
        ),
        Some(-1.0) => eval_match_approximate_prepared(
            &prepared_lookup_value,
            &prepared_lookup_array,
            MatchApproximateMode::DescendingNextLarger,
        ),
        Some(0.0) => {
            let xmatch_match_mode = crate::functions::adapters::PreparedArgValue::Eval(
                EvalValue::Number(match_type_to_xmatch_mode(&prepared_lookup_value, 0.0)?),
            );
            eval_xmatch_adapter_prepared_with_blank_behavior(
                &prepared_lookup_value,
                &prepared_lookup_array,
                Some(&xmatch_match_mode),
                None,
                BlankLookupBehavior::NotAvailable,
            )
            .map(EvalValue::Number)
            .map_err(map_xmatch_error)
        }
        Some(other) => Err(MatchEvalError::InvalidMatchType(other)),
    }
}

pub fn map_match_error_to_ws(e: &MatchEvalError) -> WorksheetErrorCode {
    match e {
        MatchEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        MatchEvalError::EmptyLookupArray => WorksheetErrorCode::NA,
        MatchEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        MatchEvalError::InvalidMatchType(_) => WorksheetErrorCode::NA,
        MatchEvalError::NotAvailable => WorksheetErrorCode::NA,
        MatchEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::{RefResolutionError, ResolverCapabilities};
    use crate::value::{ArrayCellValue, EvalArray, ExcelText, ReferenceLike};

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
            Some(&CallArgValue::Eval(EvalValue::Number(0.0))),
            &NoResolver,
        );
        assert_eq!(got, Ok(EvalValue::Number(2.0)));
    }

    #[test]
    fn eval_match_default_match_type_uses_approximate_next_smaller() {
        let got = eval_match_surface(
            &CallArgValue::Eval(EvalValue::Number(2.5)),
            &[
                CallArgValue::Eval(EvalValue::Number(1.0)),
                CallArgValue::Eval(EvalValue::Number(2.0)),
                CallArgValue::Eval(EvalValue::Number(3.0)),
            ],
            None,
            &NoResolver,
        );
        assert_eq!(got, Ok(EvalValue::Number(2.0)));
    }

    #[test]
    fn eval_match_default_and_type_one_follow_excel_duplicate_selection() {
        let exact_duplicates = eval_match_surface(
            &CallArgValue::Eval(EvalValue::Number(2.0)),
            &[CallArgValue::Eval(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Number(1.0),
                    ArrayCellValue::Number(2.0),
                    ArrayCellValue::Number(2.0),
                    ArrayCellValue::Number(2.0),
                    ArrayCellValue::Number(3.0),
                ]])
                .unwrap(),
            ))],
            Some(&CallArgValue::Eval(EvalValue::Number(1.0))),
            &NoResolver,
        );
        assert_eq!(exact_duplicates, Ok(EvalValue::Number(4.0)));

        let omitted_match_type = eval_match_surface(
            &CallArgValue::Eval(EvalValue::Number(2.0)),
            &[CallArgValue::Eval(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Number(1.0),
                    ArrayCellValue::Number(2.0),
                    ArrayCellValue::Number(2.0),
                    ArrayCellValue::Number(2.0),
                    ArrayCellValue::Number(3.0),
                ]])
                .unwrap(),
            ))],
            Some(&CallArgValue::MissingArg),
            &NoResolver,
        );
        assert_eq!(omitted_match_type, Ok(EvalValue::Number(4.0)));
    }

    #[test]
    fn eval_match_type_negative_one_prefers_first_descending_duplicate() {
        let got = eval_match_surface(
            &CallArgValue::Eval(EvalValue::Number(2.0)),
            &[CallArgValue::Eval(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Number(3.0),
                    ArrayCellValue::Number(2.0),
                    ArrayCellValue::Number(2.0),
                    ArrayCellValue::Number(2.0),
                    ArrayCellValue::Number(1.0),
                ]])
                .unwrap(),
            ))],
            Some(&CallArgValue::Eval(EvalValue::Number(-1.0))),
            &NoResolver,
        );
        assert_eq!(got, Ok(EvalValue::Number(2.0)));
    }

    #[test]
    fn eval_match_approximate_unsorted_empirical_lanes_match_excel() {
        let type_one = eval_match_surface(
            &CallArgValue::Eval(EvalValue::Number(2.5)),
            &[CallArgValue::Eval(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Number(1.0),
                    ArrayCellValue::Number(3.0),
                    ArrayCellValue::Number(2.0),
                    ArrayCellValue::Number(4.0),
                ]])
                .unwrap(),
            ))],
            Some(&CallArgValue::Eval(EvalValue::Number(1.0))),
            &NoResolver,
        );
        assert_eq!(type_one, Ok(EvalValue::Number(1.0)));

        let type_neg_one = eval_match_surface(
            &CallArgValue::Eval(EvalValue::Number(2.5)),
            &[CallArgValue::Eval(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Number(3.0),
                    ArrayCellValue::Number(1.0),
                    ArrayCellValue::Number(4.0),
                    ArrayCellValue::Number(2.0),
                ]])
                .unwrap(),
            ))],
            Some(&CallArgValue::Eval(EvalValue::Number(-1.0))),
            &NoResolver,
        );
        assert_eq!(type_neg_one, Ok(EvalValue::Number(1.0)));
    }

    #[test]
    fn eval_match_blank_lookup_value_returns_not_available() {
        let got = eval_match_surface(
            &CallArgValue::EmptyCell,
            &[CallArgValue::Eval(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::EmptyCell,
                    ArrayCellValue::Number(1.0),
                ]])
                .unwrap(),
            ))],
            Some(&CallArgValue::Eval(EvalValue::Number(0.0))),
            &NoResolver,
        );
        assert_eq!(got, Err(MatchEvalError::NotAvailable));
    }

    #[test]
    fn eval_match_exact_wildcard_mode_handles_escaped_patterns() {
        let got = eval_match_surface(
            &text_arg("a~*"),
            &[CallArgValue::Eval(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Text(ExcelText::from_utf16_code_units(
                        "abc".encode_utf16().collect(),
                    )),
                    ArrayCellValue::Text(ExcelText::from_utf16_code_units(
                        "a*".encode_utf16().collect(),
                    )),
                ]])
                .unwrap(),
            ))],
            Some(&CallArgValue::Eval(EvalValue::Number(0.0))),
            &NoResolver,
        );
        assert_eq!(got, Ok(EvalValue::Number(2.0)));
    }

    #[test]
    fn eval_match_case_insensitive_text_comparison() {
        let got = eval_match_surface(
            &text_arg("Abc"),
            &[text_arg("abc"), text_arg("def")],
            Some(&CallArgValue::Eval(EvalValue::Number(0.0))),
            &NoResolver,
        );
        assert_eq!(got, Ok(EvalValue::Number(1.0)));
    }

    #[test]
    fn eval_match_flattens_lookup_vectors_and_rejects_two_dimensional_arrays() {
        let flat = eval_match_surface(
            &CallArgValue::Eval(EvalValue::Number(2.0)),
            &[CallArgValue::Eval(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Number(1.0),
                    ArrayCellValue::Number(2.0),
                ]])
                .unwrap(),
            ))],
            Some(&CallArgValue::Eval(EvalValue::Number(0.0))),
            &NoResolver,
        );
        assert_eq!(flat, Ok(EvalValue::Number(2.0)));

        let two_d = eval_match_surface(
            &CallArgValue::Eval(EvalValue::Number(2.0)),
            &[CallArgValue::Eval(EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Number(1.0), ArrayCellValue::Number(2.0)],
                    vec![ArrayCellValue::Number(3.0), ArrayCellValue::Number(4.0)],
                ])
                .unwrap(),
            ))],
            Some(&CallArgValue::Eval(EvalValue::Number(0.0))),
            &NoResolver,
        );
        assert_eq!(
            two_d,
            Err(MatchEvalError::Coercion(CoercionError::UnsupportedValueKind(
                "two_dimensional_array"
            )))
        );
    }
}

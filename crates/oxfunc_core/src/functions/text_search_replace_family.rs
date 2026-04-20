use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    PreparedArgValue, coerce_prepared_to_number, coerce_prepared_to_text, run_values_only_prepared,
};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, ExcelText, WorksheetErrorCode};
use std::collections::HashMap;

const TEXT_SEARCH_REPLACE_BASE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.TEXT_SEARCH_REPLACE_BASE",
    arity: Arity::exact(1),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
    coercion_lift_profile: CoercionLiftProfile::None,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
};

pub const PROPER_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.PROPER",
    ..TEXT_SEARCH_REPLACE_BASE_META
};

pub const SUBSTITUTE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.SUBSTITUTE",
    arity: Arity { min: 3, max: 4 },
    ..TEXT_SEARCH_REPLACE_BASE_META
};

pub const REPLACE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.REPLACE",
    arity: Arity::exact(4),
    ..TEXT_SEARCH_REPLACE_BASE_META
};

pub const FIND_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.FIND",
    arity: Arity { min: 2, max: 3 },
    ..TEXT_SEARCH_REPLACE_BASE_META
};

pub const SEARCH_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.SEARCH",
    arity: Arity { min: 2, max: 3 },
    ..TEXT_SEARCH_REPLACE_BASE_META
};

#[derive(Debug, Clone, PartialEq)]
pub enum TextSearchReplaceEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
    Domain(WorksheetErrorCode),
}

fn value_error() -> TextSearchReplaceEvalError {
    TextSearchReplaceEvalError::Domain(WorksheetErrorCode::Value)
}

fn text_from_string(s: String) -> ExcelText {
    ExcelText::from_utf16_code_units(s.encode_utf16().collect())
}

fn text_from_units(units: Vec<u16>) -> ExcelText {
    ExcelText::from_utf16_code_units(units)
}

fn truncate_toward_zero(n: f64) -> Result<f64, TextSearchReplaceEvalError> {
    if !n.is_finite() {
        return Err(value_error());
    }
    Ok(n.trunc())
}

fn one_based_start_from_number(n: f64) -> Result<usize, TextSearchReplaceEvalError> {
    let truncated = truncate_toward_zero(n)?;
    if truncated < 1.0 {
        return Err(value_error());
    }
    if truncated > usize::MAX as f64 {
        return Ok(usize::MAX);
    }
    Ok(truncated as usize)
}

fn nonnegative_count_from_number(n: f64) -> Result<usize, TextSearchReplaceEvalError> {
    let truncated = truncate_toward_zero(n)?;
    if truncated < 0.0 {
        return Err(value_error());
    }
    if truncated > usize::MAX as f64 {
        return Ok(usize::MAX);
    }
    Ok(truncated as usize)
}

fn positive_instance_from_number(n: f64) -> Result<usize, TextSearchReplaceEvalError> {
    let truncated = truncate_toward_zero(n)?;
    if truncated < 1.0 {
        return Err(value_error());
    }
    if truncated > usize::MAX as f64 {
        return Ok(usize::MAX);
    }
    Ok(truncated as usize)
}

fn parse_optional_start_arg(
    prepared: Option<&PreparedArgValue>,
) -> Result<usize, TextSearchReplaceEvalError> {
    match prepared {
        None | Some(PreparedArgValue::MissingArg) => Ok(1),
        Some(arg) => {
            let start =
                coerce_prepared_to_number(arg).map_err(TextSearchReplaceEvalError::Coercion)?;
            one_based_start_from_number(start)
        }
    }
}

fn parse_optional_instance_arg(
    prepared: Option<&PreparedArgValue>,
) -> Result<Option<usize>, TextSearchReplaceEvalError> {
    match prepared {
        None | Some(PreparedArgValue::MissingArg) => Ok(None),
        Some(arg) => {
            let instance =
                coerce_prepared_to_number(arg).map_err(TextSearchReplaceEvalError::Coercion)?;
            Ok(Some(positive_instance_from_number(instance)?))
        }
    }
}

fn ascii_lower_unit(unit: u16) -> u16 {
    if (b'A' as u16..=b'Z' as u16).contains(&unit) {
        unit + 32
    } else {
        unit
    }
}

fn search_units_equal(lhs: u16, rhs: u16) -> bool {
    ascii_lower_unit(lhs) == ascii_lower_unit(rhs)
}

pub fn proper_kernel(text: &ExcelText) -> ExcelText {
    let mut out = String::new();
    let mut start_of_word = true;
    for ch in text.to_string_lossy().chars() {
        if ch.is_alphabetic() {
            if start_of_word {
                out.extend(ch.to_uppercase());
            } else {
                out.extend(ch.to_lowercase());
            }
            start_of_word = false;
        } else {
            out.push(ch);
            start_of_word = true;
        }
    }
    text_from_string(out)
}

pub fn substitute_kernel(
    text: &ExcelText,
    old_text: &ExcelText,
    new_text: &ExcelText,
    instance_num: Option<usize>,
) -> Result<ExcelText, TextSearchReplaceEvalError> {
    let hay = text.utf16_code_units();
    let needle = old_text.utf16_code_units();
    let replacement = new_text.utf16_code_units();

    if needle.is_empty() {
        return Ok(text.clone());
    }

    let mut out = Vec::with_capacity(hay.len());
    let mut index = 0usize;
    let mut occurrence = 0usize;

    while index < hay.len() {
        if index + needle.len() <= hay.len() && &hay[index..index + needle.len()] == needle {
            occurrence += 1;
            if instance_num.is_none() || instance_num == Some(occurrence) {
                out.extend_from_slice(replacement);
                index += needle.len();
                continue;
            }
        }
        out.push(hay[index]);
        index += 1;
    }

    Ok(text_from_units(out))
}

pub fn replace_kernel(
    old_text: &ExcelText,
    start_num: usize,
    num_chars: usize,
    new_text: &ExcelText,
) -> Result<ExcelText, TextSearchReplaceEvalError> {
    if start_num == 0 {
        return Err(value_error());
    }

    let units = old_text.utf16_code_units();
    let start_index = start_num.saturating_sub(1).min(units.len());
    let end_index = start_index.saturating_add(num_chars).min(units.len());
    let mut out = Vec::with_capacity(
        start_index + new_text.len_utf16_code_units() + units.len().saturating_sub(end_index),
    );
    out.extend_from_slice(&units[..start_index]);
    out.extend_from_slice(new_text.utf16_code_units());
    out.extend_from_slice(&units[end_index..]);
    Ok(text_from_units(out))
}

pub fn find_kernel(
    find_text: &ExcelText,
    within_text: &ExcelText,
    start_num: usize,
) -> Result<f64, TextSearchReplaceEvalError> {
    let needle = find_text.utf16_code_units();
    let hay = within_text.utf16_code_units();

    if needle.is_empty() {
        if start_num <= hay.len() + 1 {
            return Ok(start_num as f64);
        }
        return Err(value_error());
    }

    if start_num == 0 {
        return Err(value_error());
    }
    let start_index = start_num - 1;
    if start_index >= hay.len() || needle.len() > hay.len() {
        return Err(value_error());
    }

    for index in start_index..=hay.len() - needle.len() {
        if &hay[index..index + needle.len()] == needle {
            return Ok((index + 1) as f64);
        }
    }

    Err(value_error())
}

fn search_pattern_matches_prefix(pattern: &[u16], text: &[u16]) -> bool {
    fn rec(
        pattern: &[u16],
        text: &[u16],
        pattern_index: usize,
        text_index: usize,
        memo: &mut HashMap<(usize, usize), bool>,
    ) -> bool {
        if let Some(hit) = memo.get(&(pattern_index, text_index)) {
            return *hit;
        }

        let result = if pattern_index == pattern.len() {
            true
        } else {
            match pattern[pattern_index] {
                42 => {
                    let next_pattern = pattern_index + 1;
                    if next_pattern == pattern.len() {
                        true
                    } else {
                        (text_index..=text.len())
                            .any(|next_text| rec(pattern, text, next_pattern, next_text, memo))
                    }
                }
                63 => {
                    text_index < text.len()
                        && rec(pattern, text, pattern_index + 1, text_index + 1, memo)
                }
                126 if pattern_index + 1 < pattern.len() => {
                    text_index < text.len()
                        && search_units_equal(pattern[pattern_index + 1], text[text_index])
                        && rec(pattern, text, pattern_index + 2, text_index + 1, memo)
                }
                ch => {
                    text_index < text.len()
                        && search_units_equal(ch, text[text_index])
                        && rec(pattern, text, pattern_index + 1, text_index + 1, memo)
                }
            }
        };

        memo.insert((pattern_index, text_index), result);
        result
    }

    let mut memo = HashMap::new();
    rec(pattern, text, 0, 0, &mut memo)
}

pub fn search_kernel(
    find_text: &ExcelText,
    within_text: &ExcelText,
    start_num: usize,
) -> Result<f64, TextSearchReplaceEvalError> {
    let pattern = find_text.utf16_code_units();
    let hay = within_text.utf16_code_units();

    if start_num == 0 {
        return Err(value_error());
    }

    if pattern.is_empty() {
        if hay.is_empty() && start_num == 1 {
            return Ok(1.0);
        }
        if start_num <= hay.len() {
            return Ok(start_num as f64);
        }
        return Err(value_error());
    }

    let start_index = start_num - 1;
    if start_index >= hay.len() {
        return Err(value_error());
    }

    for index in start_index..hay.len() {
        if search_pattern_matches_prefix(pattern, &hay[index..]) {
            return Ok((index + 1) as f64);
        }
    }

    Err(value_error())
}

pub fn eval_proper_adapter_prepared(
    args: &[PreparedArgValue],
) -> Result<EvalValue, TextSearchReplaceEvalError> {
    if !PROPER_META.arity.accepts(args.len()) {
        return Err(TextSearchReplaceEvalError::ArityMismatch {
            expected_min: PROPER_META.arity.min,
            expected_max: PROPER_META.arity.max,
            actual: args.len(),
        });
    }

    let text = coerce_prepared_to_text(&args[0]).map_err(TextSearchReplaceEvalError::Coercion)?;
    Ok(EvalValue::Text(proper_kernel(&text)))
}

pub fn eval_substitute_adapter_prepared(
    args: &[PreparedArgValue],
) -> Result<EvalValue, TextSearchReplaceEvalError> {
    if !SUBSTITUTE_META.arity.accepts(args.len()) {
        return Err(TextSearchReplaceEvalError::ArityMismatch {
            expected_min: SUBSTITUTE_META.arity.min,
            expected_max: SUBSTITUTE_META.arity.max,
            actual: args.len(),
        });
    }

    let text = coerce_prepared_to_text(&args[0]).map_err(TextSearchReplaceEvalError::Coercion)?;
    let old_text =
        coerce_prepared_to_text(&args[1]).map_err(TextSearchReplaceEvalError::Coercion)?;
    let new_text =
        coerce_prepared_to_text(&args[2]).map_err(TextSearchReplaceEvalError::Coercion)?;
    let instance_num = parse_optional_instance_arg(args.get(3))?;
    Ok(EvalValue::Text(substitute_kernel(
        &text,
        &old_text,
        &new_text,
        instance_num,
    )?))
}

pub fn eval_replace_adapter_prepared(
    args: &[PreparedArgValue],
) -> Result<EvalValue, TextSearchReplaceEvalError> {
    if !REPLACE_META.arity.accepts(args.len()) {
        return Err(TextSearchReplaceEvalError::ArityMismatch {
            expected_min: REPLACE_META.arity.min,
            expected_max: REPLACE_META.arity.max,
            actual: args.len(),
        });
    }

    let old_text =
        coerce_prepared_to_text(&args[0]).map_err(TextSearchReplaceEvalError::Coercion)?;
    let start_num = coerce_prepared_to_number(&args[1])
        .map_err(TextSearchReplaceEvalError::Coercion)
        .and_then(one_based_start_from_number)?;
    let num_chars = coerce_prepared_to_number(&args[2])
        .map_err(TextSearchReplaceEvalError::Coercion)
        .and_then(nonnegative_count_from_number)?;
    let new_text =
        coerce_prepared_to_text(&args[3]).map_err(TextSearchReplaceEvalError::Coercion)?;
    Ok(EvalValue::Text(replace_kernel(
        &old_text, start_num, num_chars, &new_text,
    )?))
}

pub fn eval_find_adapter_prepared(
    args: &[PreparedArgValue],
) -> Result<EvalValue, TextSearchReplaceEvalError> {
    if !FIND_META.arity.accepts(args.len()) {
        return Err(TextSearchReplaceEvalError::ArityMismatch {
            expected_min: FIND_META.arity.min,
            expected_max: FIND_META.arity.max,
            actual: args.len(),
        });
    }

    let find_text =
        coerce_prepared_to_text(&args[0]).map_err(TextSearchReplaceEvalError::Coercion)?;
    let within_text =
        coerce_prepared_to_text(&args[1]).map_err(TextSearchReplaceEvalError::Coercion)?;
    let start_num = parse_optional_start_arg(args.get(2))?;
    Ok(EvalValue::Number(find_kernel(
        &find_text,
        &within_text,
        start_num,
    )?))
}

pub fn eval_search_adapter_prepared(
    args: &[PreparedArgValue],
) -> Result<EvalValue, TextSearchReplaceEvalError> {
    if !SEARCH_META.arity.accepts(args.len()) {
        return Err(TextSearchReplaceEvalError::ArityMismatch {
            expected_min: SEARCH_META.arity.min,
            expected_max: SEARCH_META.arity.max,
            actual: args.len(),
        });
    }

    let find_text =
        coerce_prepared_to_text(&args[0]).map_err(TextSearchReplaceEvalError::Coercion)?;
    let within_text =
        coerce_prepared_to_text(&args[1]).map_err(TextSearchReplaceEvalError::Coercion)?;
    let start_num = parse_optional_start_arg(args.get(2))?;
    Ok(EvalValue::Number(search_kernel(
        &find_text,
        &within_text,
        start_num,
    )?))
}

pub fn eval_proper_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, TextSearchReplaceEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_proper_adapter_prepared,
        TextSearchReplaceEvalError::Coercion,
    )
}

pub fn eval_substitute_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, TextSearchReplaceEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_substitute_adapter_prepared,
        TextSearchReplaceEvalError::Coercion,
    )
}

pub fn eval_replace_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, TextSearchReplaceEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_replace_adapter_prepared,
        TextSearchReplaceEvalError::Coercion,
    )
}

pub fn eval_find_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, TextSearchReplaceEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_find_adapter_prepared,
        TextSearchReplaceEvalError::Coercion,
    )
}

pub fn eval_search_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, TextSearchReplaceEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_search_adapter_prepared,
        TextSearchReplaceEvalError::Coercion,
    )
}

pub fn map_text_search_replace_error_to_ws(
    error: &TextSearchReplaceEvalError,
) -> WorksheetErrorCode {
    match error {
        TextSearchReplaceEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        TextSearchReplaceEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        TextSearchReplaceEvalError::Coercion(_) => WorksheetErrorCode::Value,
        TextSearchReplaceEvalError::Domain(code) => *code,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::{RefResolutionError, ResolverCapabilities};
    use crate::value::ReferenceLike;

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

    fn text_prepared(s: &str) -> PreparedArgValue {
        PreparedArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
            s.encode_utf16().collect(),
        )))
    }

    fn number_arg(n: f64) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Number(n))
    }

    fn number_prepared(n: f64) -> PreparedArgValue {
        PreparedArgValue::Eval(EvalValue::Number(n))
    }

    #[test]
    fn proper_matches_basic_native_rows() {
        assert_eq!(
            eval_proper_surface(&[text_arg("hello world")], &NoResolver),
            Ok(EvalValue::Text(text_from_string("Hello World".to_string())))
        );
        assert_eq!(
            eval_proper_surface(&[text_arg("o'brien")], &NoResolver),
            Ok(EvalValue::Text(text_from_string("O'Brien".to_string())))
        );
        assert_eq!(
            eval_proper_surface(&[text_arg("abc123def")], &NoResolver),
            Ok(EvalValue::Text(text_from_string("Abc123Def".to_string())))
        );
    }

    // Current repo-local theory note:
    // PROPER currently inherits Rust Unicode casing behavior. That is sufficient for several
    // Latin and Greek witnesses but is not yet a justified Excel-family model. Combined with the
    // UPPER sharp-s override, the current implementation should be treated as a stopgap rather
    // than a principled cross-function casing layer.
    #[test]
    fn proper_unicode_casing_matches_current_local_results() {
        let cases = [
            (
                "PROPER straße",
                eval_proper_surface(&[text_arg("straße")], &NoResolver),
                Ok(EvalValue::Text(text_from_string("Straße".to_string()))),
            ),
            (
                "PROPER weiß",
                eval_proper_surface(&[text_arg("weiß")], &NoResolver),
                Ok(EvalValue::Text(text_from_string("Weiß".to_string()))),
            ),
            (
                "PROPER İstanbul",
                eval_proper_surface(&[text_arg("İstanbul")], &NoResolver),
                Ok(EvalValue::Text(text_from_string("İstanbul".to_string()))),
            ),
            (
                "PROPER κόσμος",
                eval_proper_surface(&[text_arg("κόσμος")], &NoResolver),
                Ok(EvalValue::Text(text_from_string("Κόσμος".to_string()))),
            ),
            (
                "PROPER café",
                eval_proper_surface(&[text_arg("café")], &NoResolver),
                Ok(EvalValue::Text(text_from_string("Café".to_string()))),
            ),
            (
                "PROPER Ångström",
                eval_proper_surface(&[text_arg("Ångström")], &NoResolver),
                Ok(EvalValue::Text(text_from_string("Ångström".to_string()))),
            ),
        ];

        for (name, got, expected) in cases {
            assert_eq!(got, expected, "{name}");
        }
    }

    #[test]
    fn substitute_honors_all_and_instance_num() {
        assert_eq!(
            eval_substitute_surface(
                &[text_arg("abab"), text_arg("a"), text_arg("x")],
                &NoResolver
            ),
            Ok(EvalValue::Text(text_from_string("xbxb".to_string())))
        );
        assert_eq!(
            eval_substitute_surface(
                &[
                    text_arg("abab"),
                    text_arg("a"),
                    text_arg("x"),
                    number_arg(2.0)
                ],
                &NoResolver,
            ),
            Ok(EvalValue::Text(text_from_string("abxb".to_string())))
        );
        assert_eq!(
            eval_substitute_surface(
                &[
                    text_arg("abab"),
                    text_arg("a"),
                    text_arg("x"),
                    number_arg(0.0)
                ],
                &NoResolver,
            ),
            Err(TextSearchReplaceEvalError::Domain(
                WorksheetErrorCode::Value
            ))
        );
        assert_eq!(
            eval_substitute_surface(&[text_arg("abc"), text_arg(""), text_arg("x")], &NoResolver),
            Ok(EvalValue::Text(text_from_string("abc".to_string())))
        );
    }

    #[test]
    fn replace_uses_utf16_unit_positions_and_validates_bounds() {
        assert_eq!(
            eval_replace_surface(
                &[
                    text_arg("abcdef"),
                    number_arg(2.0),
                    number_arg(3.0),
                    text_arg("ZZ")
                ],
                &NoResolver,
            ),
            Ok(EvalValue::Text(text_from_string("aZZef".to_string())))
        );
        assert_eq!(
            eval_replace_surface(
                &[
                    text_arg("abcdef"),
                    number_arg(7.0),
                    number_arg(0.0),
                    text_arg("Z")
                ],
                &NoResolver,
            ),
            Ok(EvalValue::Text(text_from_string("abcdefZ".to_string())))
        );
        assert_eq!(
            eval_replace_surface(
                &[
                    CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(vec![
                        0xD83D,
                        0xDE00,
                        b'a' as u16
                    ],))),
                    number_arg(2.0),
                    number_arg(1.0),
                    text_arg("Z"),
                ],
                &NoResolver,
            ),
            Ok(EvalValue::Text(ExcelText::from_utf16_code_units(vec![
                0xD83D,
                b'Z' as u16,
                b'a' as u16,
            ])))
        );
        assert_eq!(
            eval_replace_surface(
                &[
                    text_arg("abc"),
                    number_arg(0.0),
                    number_arg(1.0),
                    text_arg("x")
                ],
                &NoResolver,
            ),
            Err(TextSearchReplaceEvalError::Domain(
                WorksheetErrorCode::Value
            ))
        );
        assert_eq!(
            eval_replace_surface(
                &[
                    text_arg("abc"),
                    number_arg(1.0),
                    number_arg(-1.0),
                    text_arg("x")
                ],
                &NoResolver,
            ),
            Err(TextSearchReplaceEvalError::Domain(
                WorksheetErrorCode::Value
            ))
        );
    }

    #[test]
    fn find_is_case_sensitive_and_defaults_start_to_one() {
        assert_eq!(
            eval_find_surface(&[text_arg("b"), text_arg("abc")], &NoResolver),
            Ok(EvalValue::Number(2.0))
        );
        assert_eq!(
            eval_find_surface(&[text_arg("B"), text_arg("abc")], &NoResolver),
            Err(TextSearchReplaceEvalError::Domain(
                WorksheetErrorCode::Value
            ))
        );
        assert_eq!(
            eval_find_adapter_prepared(&[
                text_prepared("b"),
                text_prepared("abc"),
                PreparedArgValue::MissingArg,
            ]),
            Ok(EvalValue::Number(2.0))
        );
        assert_eq!(
            eval_find_surface(
                &[
                    text_arg("a"),
                    CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(vec![
                        0xD83D,
                        0xDE00,
                        b'a' as u16,
                    ]))),
                ],
                &NoResolver,
            ),
            Ok(EvalValue::Number(3.0))
        );
        assert_eq!(
            eval_find_surface(
                &[text_arg(""), text_arg("abc"), number_arg(4.0)],
                &NoResolver
            ),
            Ok(EvalValue::Number(4.0))
        );
    }

    #[test]
    fn search_is_case_insensitive_and_honors_wildcards() {
        assert_eq!(
            eval_search_surface(&[text_arg("b"), text_arg("ABC")], &NoResolver),
            Ok(EvalValue::Number(2.0))
        );
        assert_eq!(
            eval_search_surface(
                &[text_arg("a?c"), text_arg("axc"), number_arg(1.0)],
                &NoResolver,
            ),
            Ok(EvalValue::Number(1.0))
        );
        assert_eq!(
            eval_search_surface(
                &[text_arg("a*c"), text_arg("abbbbbc"), number_arg(1.0)],
                &NoResolver,
            ),
            Ok(EvalValue::Number(1.0))
        );
        assert_eq!(
            eval_search_surface(&[text_arg("a~*c"), text_arg("a*c")], &NoResolver),
            Ok(EvalValue::Number(1.0))
        );
        assert_eq!(
            eval_search_surface(&[text_arg("a~?c"), text_arg("a?c")], &NoResolver),
            Ok(EvalValue::Number(1.0))
        );
    }

    #[test]
    fn search_defaults_start_and_uses_utf16_unit_positions() {
        assert_eq!(
            eval_search_adapter_prepared(&[
                text_prepared("b"),
                text_prepared("abc"),
                PreparedArgValue::MissingArg,
            ]),
            Ok(EvalValue::Number(2.0))
        );
        assert_eq!(
            eval_search_surface(
                &[
                    text_arg("a"),
                    CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(vec![
                        0xD83D,
                        0xDE00,
                        b'a' as u16,
                    ]))),
                ],
                &NoResolver,
            ),
            Ok(EvalValue::Number(3.0))
        );
        assert_eq!(
            eval_search_surface(
                &[text_arg(""), text_arg("abc"), number_arg(4.0)],
                &NoResolver
            ),
            Err(TextSearchReplaceEvalError::Domain(
                WorksheetErrorCode::Value
            ))
        );
        assert_eq!(
            eval_search_surface(
                &[text_arg("a"), text_arg("abc"), number_arg(0.0)],
                &NoResolver
            ),
            Err(TextSearchReplaceEvalError::Domain(
                WorksheetErrorCode::Value
            ))
        );
    }

    #[test]
    fn map_error_preserves_worksheet_errors() {
        assert_eq!(
            map_text_search_replace_error_to_ws(&TextSearchReplaceEvalError::Domain(
                WorksheetErrorCode::Value,
            )),
            WorksheetErrorCode::Value
        );
        assert_eq!(
            map_text_search_replace_error_to_ws(&TextSearchReplaceEvalError::Coercion(
                CoercionError::WorksheetError(WorksheetErrorCode::Ref)
            )),
            WorksheetErrorCode::Ref
        );
    }

    #[test]
    fn arity_mismatch_is_reported_explicitly() {
        assert_eq!(
            eval_proper_adapter_prepared(&[]),
            Err(TextSearchReplaceEvalError::ArityMismatch {
                expected_min: 1,
                expected_max: 1,
                actual: 0,
            })
        );
    }

    #[test]
    fn substitute_missing_optional_instance_defaults_to_all_and_truncates_instance() {
        assert_eq!(
            eval_substitute_adapter_prepared(&[
                text_prepared("abab"),
                text_prepared("a"),
                text_prepared("x"),
                PreparedArgValue::MissingArg,
            ]),
            Ok(EvalValue::Text(text_from_string("xbxb".to_string())))
        );
        assert_eq!(
            eval_substitute_adapter_prepared(&[
                text_prepared("abab"),
                text_prepared("a"),
                text_prepared("x"),
                number_prepared(2.9),
            ]),
            Ok(EvalValue::Text(text_from_string("abxb".to_string())))
        );
    }
}

use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    PreparedArgValue, coerce_prepared_to_number, coerce_prepared_to_text, prepare_args_values_only,
};
use crate::functions::excel_casing::proper_text;
use crate::resolver::ReferenceResolver;
use crate::value::{
    ArrayCellValue, CallArgValue, EvalArray, EvalValue, ExcelText, WorksheetErrorCode,
};
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

#[cfg(test)]
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

fn prepared_from_array_cell(cell: &ArrayCellValue) -> PreparedArgValue {
    match cell {
        ArrayCellValue::Number(n) => PreparedArgValue::Eval(EvalValue::Number(*n)),
        ArrayCellValue::Text(t) => PreparedArgValue::Eval(EvalValue::Text(t.clone())),
        ArrayCellValue::Logical(b) => PreparedArgValue::Eval(EvalValue::Logical(*b)),
        ArrayCellValue::Error(code) => PreparedArgValue::Eval(EvalValue::Error(*code)),
        ArrayCellValue::EmptyCell => PreparedArgValue::EmptyCell,
    }
}

fn text_search_replace_result_to_array_cell(
    result: Result<EvalValue, TextSearchReplaceEvalError>,
) -> ArrayCellValue {
    match result {
        Ok(EvalValue::Number(n)) => ArrayCellValue::Number(n),
        Ok(EvalValue::Text(text)) => ArrayCellValue::Text(text),
        Ok(EvalValue::Logical(value)) => ArrayCellValue::Logical(value),
        Ok(EvalValue::Error(code)) => ArrayCellValue::Error(code),
        Ok(_) => ArrayCellValue::Error(WorksheetErrorCode::Value),
        Err(err) => ArrayCellValue::Error(map_text_search_replace_error_to_ws(&err)),
    }
}

fn eval_text_search_replace_with_single_array_lift(
    prepared: &[PreparedArgValue],
    allowed_array_arg_indexes: &[usize],
    eval_scalar: impl Fn(&[PreparedArgValue]) -> Result<EvalValue, TextSearchReplaceEvalError>,
) -> Result<EvalValue, TextSearchReplaceEvalError> {
    let array_args = prepared
        .iter()
        .enumerate()
        .filter_map(|(idx, arg)| match arg {
            PreparedArgValue::Eval(EvalValue::Array(array))
                if allowed_array_arg_indexes.contains(&idx) =>
            {
                Some((idx, array))
            }
            _ => None,
        })
        .collect::<Vec<_>>();

    match array_args.as_slice() {
        [] => eval_scalar(prepared),
        [(arg_index, array)] => {
            let cells = array
                .iter_row_major()
                .map(|cell| {
                    let mut scalar_args = prepared.to_vec();
                    scalar_args[*arg_index] = prepared_from_array_cell(cell);
                    text_search_replace_result_to_array_cell(eval_scalar(&scalar_args))
                })
                .collect();
            Ok(EvalValue::Array(
                EvalArray::new(array.shape(), cells)
                    .expect("text search/replace lifted array shape remains valid"),
            ))
        }
        _ => eval_scalar(prepared),
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
    proper_text(text)
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
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, TextSearchReplaceEvalError> {
    let prepared =
        prepare_args_values_only(args, resolver).map_err(TextSearchReplaceEvalError::Coercion)?;
    eval_text_search_replace_with_single_array_lift(&prepared, &[0], eval_proper_adapter_prepared)
}

pub fn eval_substitute_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, TextSearchReplaceEvalError> {
    let prepared =
        prepare_args_values_only(args, resolver).map_err(TextSearchReplaceEvalError::Coercion)?;
    eval_text_search_replace_with_single_array_lift(
        &prepared,
        &[0, 1, 2, 3],
        eval_substitute_adapter_prepared,
    )
}

pub fn eval_replace_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, TextSearchReplaceEvalError> {
    let prepared =
        prepare_args_values_only(args, resolver).map_err(TextSearchReplaceEvalError::Coercion)?;
    eval_text_search_replace_with_single_array_lift(
        &prepared,
        &[0, 1, 2, 3],
        eval_replace_adapter_prepared,
    )
}

pub fn eval_find_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, TextSearchReplaceEvalError> {
    let prepared =
        prepare_args_values_only(args, resolver).map_err(TextSearchReplaceEvalError::Coercion)?;
    eval_text_search_replace_with_single_array_lift(
        &prepared,
        &[0, 1, 2],
        eval_find_adapter_prepared,
    )
}

pub fn eval_search_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, TextSearchReplaceEvalError> {
    let prepared =
        prepare_args_values_only(args, resolver).map_err(TextSearchReplaceEvalError::Coercion)?;
    eval_text_search_replace_with_single_array_lift(
        &prepared,
        &[0, 1, 2],
        eval_search_adapter_prepared,
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

    fn text_cell(s: &str) -> ArrayCellValue {
        ArrayCellValue::Text(ExcelText::from_interop_assignment(s))
    }

    fn text_array_arg(rows: Vec<Vec<&str>>) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Array(
            EvalArray::from_rows(
                rows.into_iter()
                    .map(|row| row.into_iter().map(text_cell).collect())
                    .collect(),
            )
            .unwrap(),
        ))
    }

    fn number_array_arg(rows: Vec<Vec<f64>>) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Array(
            EvalArray::from_rows(
                rows.into_iter()
                    .map(|row| row.into_iter().map(ArrayCellValue::Number).collect())
                    .collect(),
            )
            .unwrap(),
        ))
    }

    fn expected_array(rows: Vec<Vec<ArrayCellValue>>) -> EvalValue {
        EvalValue::Array(EvalArray::from_rows(rows).unwrap())
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

    // PROPER is now routed through the shared Excel-style worksheet casing helper so that the
    // family uses one policy surface rather than mixed raw Rust casing behavior.
    #[test]
    fn proper_unicode_casing_matches_excel_observed_rows() {
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
    fn find_and_search_spill_array_arguments() {
        assert_eq!(
            eval_find_surface(
                &[text_array_arg(vec![vec!["a", "b"]]), text_arg("abc")],
                &NoResolver,
            ),
            Ok(expected_array(vec![vec![
                ArrayCellValue::Number(1.0),
                ArrayCellValue::Number(2.0),
            ]]))
        );
        assert_eq!(
            eval_find_surface(
                &[text_arg("a"), text_array_arg(vec![vec!["abc", "bca"]])],
                &NoResolver,
            ),
            Ok(expected_array(vec![vec![
                ArrayCellValue::Number(1.0),
                ArrayCellValue::Number(3.0),
            ]]))
        );
        assert_eq!(
            eval_find_surface(
                &[
                    text_arg("a"),
                    text_arg("abc"),
                    number_array_arg(vec![vec![1.0], vec![2.0], vec![3.0]]),
                ],
                &NoResolver,
            ),
            Ok(expected_array(vec![
                vec![ArrayCellValue::Number(1.0)],
                vec![ArrayCellValue::Error(WorksheetErrorCode::Value)],
                vec![ArrayCellValue::Error(WorksheetErrorCode::Value)],
            ]))
        );
        assert_eq!(
            eval_search_surface(
                &[text_array_arg(vec![vec!["a", "b"]]), text_arg("abc")],
                &NoResolver,
            ),
            Ok(expected_array(vec![vec![
                ArrayCellValue::Number(1.0),
                ArrayCellValue::Number(2.0),
            ]]))
        );
        assert_eq!(
            eval_search_surface(
                &[text_arg("A"), text_array_arg(vec![vec!["abc", "bca"]])],
                &NoResolver,
            ),
            Ok(expected_array(vec![vec![
                ArrayCellValue::Number(1.0),
                ArrayCellValue::Number(3.0),
            ]]))
        );
    }

    #[test]
    fn replace_spills_single_array_arguments() {
        assert_eq!(
            eval_replace_surface(
                &[
                    text_array_arg(vec![vec!["abc", "def"]]),
                    number_arg(2.0),
                    number_arg(1.0),
                    text_arg("Z"),
                ],
                &NoResolver,
            ),
            Ok(expected_array(vec![vec![
                text_cell("aZc"),
                text_cell("dZf")
            ]]))
        );
        assert_eq!(
            eval_replace_surface(
                &[
                    text_arg("abc"),
                    number_array_arg(vec![vec![1.0], vec![2.0], vec![3.0]]),
                    number_arg(1.0),
                    text_arg("Z"),
                ],
                &NoResolver,
            ),
            Ok(expected_array(vec![
                vec![text_cell("Zbc")],
                vec![text_cell("aZc")],
                vec![text_cell("abZ")],
            ]))
        );
        assert_eq!(
            eval_replace_surface(
                &[
                    text_arg("abc"),
                    number_arg(2.0),
                    number_array_arg(vec![vec![1.0], vec![2.0], vec![3.0]]),
                    text_arg("Z"),
                ],
                &NoResolver,
            ),
            Ok(expected_array(vec![
                vec![text_cell("aZc")],
                vec![text_cell("aZ")],
                vec![text_cell("aZ")],
            ]))
        );
        assert_eq!(
            eval_replace_surface(
                &[
                    text_arg("abc"),
                    number_arg(2.0),
                    number_arg(1.0),
                    text_array_arg(vec![vec!["X", "Y"]]),
                ],
                &NoResolver,
            ),
            Ok(expected_array(vec![vec![
                text_cell("aXc"),
                text_cell("aYc")
            ]]))
        );
    }

    #[test]
    fn proper_and_substitute_spill_array_arguments() {
        assert_eq!(
            eval_proper_surface(
                &[text_array_arg(vec![vec!["hello world", "o'brien"]])],
                &NoResolver,
            ),
            Ok(expected_array(vec![vec![
                text_cell("Hello World"),
                text_cell("O'Brien"),
            ]]))
        );
        assert_eq!(
            eval_substitute_surface(
                &[
                    text_array_arg(vec![vec!["foo bar", "bar foo"]]),
                    text_arg("foo"),
                    text_arg("x"),
                ],
                &NoResolver,
            ),
            Ok(expected_array(vec![vec![
                text_cell("x bar"),
                text_cell("bar x")
            ]]))
        );
        assert_eq!(
            eval_substitute_surface(
                &[
                    text_arg("foo bar foo"),
                    text_array_arg(vec![vec!["foo", "bar"]]),
                    text_arg("x"),
                ],
                &NoResolver,
            ),
            Ok(expected_array(vec![vec![
                text_cell("x bar x"),
                text_cell("foo x foo"),
            ]]))
        );
        assert_eq!(
            eval_substitute_surface(
                &[
                    text_arg("foo bar foo"),
                    text_arg("foo"),
                    text_array_arg(vec![vec!["x", "y"]]),
                ],
                &NoResolver,
            ),
            Ok(expected_array(vec![vec![
                text_cell("x bar x"),
                text_cell("y bar y"),
            ]]))
        );
        assert_eq!(
            eval_substitute_surface(
                &[
                    text_arg("foo foo"),
                    text_arg("foo"),
                    text_arg("x"),
                    number_array_arg(vec![vec![1.0], vec![2.0], vec![3.0]]),
                ],
                &NoResolver,
            ),
            Ok(expected_array(vec![
                vec![text_cell("x foo")],
                vec![text_cell("foo x")],
                vec![text_cell("foo foo")],
            ]))
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

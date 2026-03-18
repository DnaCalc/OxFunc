use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    PreparedArgValue, coerce_prepared_to_number, coerce_prepared_to_text, prepare_args_values_only,
};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, ExcelText, WorksheetErrorCode};

const TEXT_DELIM_BASE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.TEXT_DELIM_BASE",
    arity: Arity { min: 2, max: 6 },
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::TextToText,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
};

pub const TEXTAFTER_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.TEXTAFTER",
    ..TEXT_DELIM_BASE_META
};

pub const TEXTBEFORE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.TEXTBEFORE",
    ..TEXT_DELIM_BASE_META
};

#[derive(Debug, Clone, PartialEq)]
pub enum TextDelimEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
    InvalidInstanceNum(f64),
    InvalidMatchMode(f64),
    InvalidMatchEnd(f64),
    NotAvailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TextDelimDirection {
    After,
    Before,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DelimiterBoundary {
    start: usize,
    delimiter_len: usize,
}

fn empty_text() -> ExcelText {
    ExcelText::from_utf16_code_units(Vec::new())
}

fn materialize_prepared_value(prepared: &PreparedArgValue) -> EvalValue {
    match prepared {
        PreparedArgValue::Eval(value) => value.clone(),
        PreparedArgValue::MissingArg => EvalValue::Error(WorksheetErrorCode::NA),
        PreparedArgValue::EmptyCell => EvalValue::Number(0.0),
    }
}

fn parse_truncated_integer(
    prepared: &PreparedArgValue,
    invalid: fn(f64) -> TextDelimEvalError,
) -> Result<f64, TextDelimEvalError> {
    let raw = coerce_prepared_to_number(prepared).map_err(TextDelimEvalError::Coercion)?;
    if !raw.is_finite() {
        return Err(invalid(raw));
    }
    Ok(raw.trunc())
}

fn parse_instance_num(prepared: &PreparedArgValue) -> Result<isize, TextDelimEvalError> {
    let truncated = parse_truncated_integer(prepared, TextDelimEvalError::InvalidInstanceNum)?;
    if truncated == 0.0 || truncated.abs() > isize::MAX as f64 {
        return Err(TextDelimEvalError::InvalidInstanceNum(truncated));
    }
    Ok(truncated as isize)
}

fn parse_binary_flag(
    prepared: &PreparedArgValue,
    invalid: fn(f64) -> TextDelimEvalError,
) -> Result<bool, TextDelimEvalError> {
    let truncated = parse_truncated_integer(prepared, invalid)?;
    match truncated {
        0.0 => Ok(false),
        1.0 => Ok(true),
        _ => Err(invalid(truncated)),
    }
}

fn fold_ascii_case(unit: u16) -> u16 {
    if (b'A' as u16..=b'Z' as u16).contains(&unit) {
        unit + 32
    } else {
        unit
    }
}

fn slices_match(haystack: &[u16], needle: &[u16], match_mode_case_insensitive: bool) -> bool {
    haystack.len() == needle.len()
        && haystack.iter().zip(needle.iter()).all(|(left, right)| {
            if match_mode_case_insensitive {
                fold_ascii_case(*left) == fold_ascii_case(*right)
            } else {
                left == right
            }
        })
}

fn find_delimiter_boundaries(
    text: &ExcelText,
    delimiter: &ExcelText,
    match_mode_case_insensitive: bool,
    match_end: bool,
) -> Vec<DelimiterBoundary> {
    let haystack = text.utf16_code_units();
    let needle = delimiter.utf16_code_units();
    let mut out = Vec::new();
    if needle.is_empty() {
        return out;
    }

    let mut idx = 0usize;
    while idx + needle.len() <= haystack.len() {
        if slices_match(
            &haystack[idx..idx + needle.len()],
            needle,
            match_mode_case_insensitive,
        ) {
            out.push(DelimiterBoundary {
                start: idx,
                delimiter_len: needle.len(),
            });
            idx += needle.len();
        } else {
            idx += 1;
        }
    }

    if match_end {
        out.push(DelimiterBoundary {
            start: haystack.len(),
            delimiter_len: 0,
        });
    }

    out
}

fn apply_empty_delimiter_rule(
    direction: TextDelimDirection,
    text: &ExcelText,
    instance_num: isize,
) -> ExcelText {
    match (direction, instance_num.is_negative()) {
        (TextDelimDirection::After, false) => text.clone(),
        (TextDelimDirection::After, true) => empty_text(),
        (TextDelimDirection::Before, false) => empty_text(),
        (TextDelimDirection::Before, true) => text.clone(),
    }
}

fn slice_around_boundary(
    direction: TextDelimDirection,
    text: &ExcelText,
    boundary: DelimiterBoundary,
) -> ExcelText {
    let units = text.utf16_code_units();
    match direction {
        TextDelimDirection::After => {
            let start = boundary.start.saturating_add(boundary.delimiter_len);
            ExcelText::from_utf16_code_units(units[start..].to_vec())
        }
        TextDelimDirection::Before => {
            ExcelText::from_utf16_code_units(units[..boundary.start].to_vec())
        }
    }
}

fn eval_text_delim_kernel(
    direction: TextDelimDirection,
    text: &ExcelText,
    delimiter: &ExcelText,
    instance_num: isize,
    match_mode_case_insensitive: bool,
    match_end: bool,
) -> Result<ExcelText, TextDelimEvalError> {
    let requested = instance_num.unsigned_abs();
    if requested > text.len_utf16_code_units() {
        return Err(TextDelimEvalError::InvalidInstanceNum(instance_num as f64));
    }

    if delimiter.utf16_code_units().is_empty() {
        return Ok(apply_empty_delimiter_rule(direction, text, instance_num));
    }

    let boundaries =
        find_delimiter_boundaries(text, delimiter, match_mode_case_insensitive, match_end);
    let selected = if instance_num.is_negative() {
        boundaries
            .len()
            .checked_sub(requested)
            .and_then(|index| boundaries.get(index))
            .copied()
    } else {
        boundaries.get(requested - 1).copied()
    };

    match selected {
        Some(boundary) => Ok(slice_around_boundary(direction, text, boundary)),
        None => Err(TextDelimEvalError::NotAvailable),
    }
}

fn eval_text_delim_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
    meta: &FunctionMeta,
    direction: TextDelimDirection,
) -> Result<EvalValue, TextDelimEvalError> {
    let prepared =
        prepare_args_values_only(args, resolver).map_err(TextDelimEvalError::Coercion)?;
    if !meta.arity.accepts(prepared.len()) {
        return Err(TextDelimEvalError::ArityMismatch {
            expected_min: meta.arity.min,
            expected_max: meta.arity.max,
            actual: prepared.len(),
        });
    }

    let text = coerce_prepared_to_text(&prepared[0]).map_err(TextDelimEvalError::Coercion)?;
    let delimiter = coerce_prepared_to_text(&prepared[1]).map_err(TextDelimEvalError::Coercion)?;
    let instance_num = if prepared.len() >= 3 {
        parse_instance_num(&prepared[2])?
    } else {
        1
    };
    let match_mode_case_insensitive = if prepared.len() >= 4 {
        parse_binary_flag(&prepared[3], TextDelimEvalError::InvalidMatchMode)?
    } else {
        false
    };
    let match_end = if prepared.len() >= 5 {
        parse_binary_flag(&prepared[4], TextDelimEvalError::InvalidMatchEnd)?
    } else {
        false
    };

    match eval_text_delim_kernel(
        direction,
        &text,
        &delimiter,
        instance_num,
        match_mode_case_insensitive,
        match_end,
    ) {
        Ok(result) => Ok(EvalValue::Text(result)),
        Err(TextDelimEvalError::NotAvailable) if prepared.len() >= 6 => {
            Ok(materialize_prepared_value(&prepared[5]))
        }
        Err(err) => Err(err),
    }
}

pub fn eval_textafter_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, TextDelimEvalError> {
    eval_text_delim_surface(args, resolver, &TEXTAFTER_META, TextDelimDirection::After)
}

pub fn eval_textbefore_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, TextDelimEvalError> {
    eval_text_delim_surface(args, resolver, &TEXTBEFORE_META, TextDelimDirection::Before)
}

pub fn map_text_delim_error_to_ws(error: &TextDelimEvalError) -> WorksheetErrorCode {
    match error {
        TextDelimEvalError::ArityMismatch { .. }
        | TextDelimEvalError::InvalidInstanceNum(_)
        | TextDelimEvalError::InvalidMatchMode(_)
        | TextDelimEvalError::InvalidMatchEnd(_) => WorksheetErrorCode::Value,
        TextDelimEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        TextDelimEvalError::Coercion(_) => WorksheetErrorCode::Value,
        TextDelimEvalError::NotAvailable => WorksheetErrorCode::NA,
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
        CallArgValue::Eval(EvalValue::Text(ExcelText::from_interop_assignment(s)))
    }

    fn number_arg(n: f64) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Number(n))
    }

    #[test]
    fn textafter_returns_text_after_positive_and_negative_instances() {
        assert_eq!(
            eval_textafter_surface(
                &[text_arg("One,Two,Three"), text_arg(","), number_arg(1.0)],
                &NoResolver,
            ),
            Ok(EvalValue::Text(ExcelText::from_interop_assignment(
                "Two,Three"
            )))
        );
        assert_eq!(
            eval_textafter_surface(
                &[text_arg("One,Two,Three"), text_arg(","), number_arg(-1.0)],
                &NoResolver,
            ),
            Ok(EvalValue::Text(ExcelText::from_interop_assignment("Three")))
        );
    }

    #[test]
    fn textbefore_returns_text_before_positive_and_negative_instances() {
        assert_eq!(
            eval_textbefore_surface(
                &[text_arg("One,Two,Three"), text_arg(","), number_arg(1.0)],
                &NoResolver,
            ),
            Ok(EvalValue::Text(ExcelText::from_interop_assignment("One")))
        );
        assert_eq!(
            eval_textbefore_surface(
                &[text_arg("One,Two,Three"), text_arg(","), number_arg(-1.0)],
                &NoResolver,
            ),
            Ok(EvalValue::Text(ExcelText::from_interop_assignment(
                "One,Two"
            )))
        );
    }

    #[test]
    fn textdelim_match_end_adds_synthetic_end_boundary() {
        assert_eq!(
            eval_textafter_surface(
                &[
                    text_arg("Socrates"),
                    text_arg(" "),
                    number_arg(1.0),
                    number_arg(0.0),
                    number_arg(1.0),
                    text_arg("fallback"),
                ],
                &NoResolver,
            ),
            Ok(EvalValue::Text(empty_text()))
        );
        assert_eq!(
            eval_textbefore_surface(
                &[
                    text_arg("Socrates"),
                    text_arg(" "),
                    number_arg(1.0),
                    number_arg(0.0),
                    number_arg(1.0),
                    text_arg("fallback"),
                ],
                &NoResolver,
            ),
            Ok(EvalValue::Text(ExcelText::from_interop_assignment(
                "Socrates"
            )))
        );
    }

    #[test]
    fn textdelim_empty_delimiter_follows_documented_polarity() {
        assert_eq!(
            eval_textafter_surface(
                &[text_arg("abc"), text_arg(""), number_arg(1.0)],
                &NoResolver,
            ),
            Ok(EvalValue::Text(ExcelText::from_interop_assignment("abc")))
        );
        assert_eq!(
            eval_textbefore_surface(
                &[text_arg("abc"), text_arg(""), number_arg(1.0)],
                &NoResolver,
            ),
            Ok(EvalValue::Text(empty_text()))
        );
        assert_eq!(
            eval_textafter_surface(
                &[text_arg("abc"), text_arg(""), number_arg(-1.0)],
                &NoResolver,
            ),
            Ok(EvalValue::Text(empty_text()))
        );
        assert_eq!(
            eval_textbefore_surface(
                &[text_arg("abc"), text_arg(""), number_arg(-1.0)],
                &NoResolver,
            ),
            Ok(EvalValue::Text(ExcelText::from_interop_assignment("abc")))
        );
    }

    #[test]
    fn textdelim_if_not_found_returns_explicit_fallback() {
        assert_eq!(
            eval_textafter_surface(
                &[
                    text_arg("abc"),
                    text_arg("/"),
                    number_arg(1.0),
                    number_arg(0.0),
                    number_arg(0.0),
                    number_arg(7.0),
                ],
                &NoResolver,
            ),
            Ok(EvalValue::Number(7.0))
        );
    }

    #[test]
    fn textdelim_case_insensitive_mode_matches_ascii() {
        assert_eq!(
            eval_textafter_surface(
                &[
                    text_arg("ABcdEF"),
                    text_arg("CD"),
                    number_arg(1.0),
                    number_arg(1.0),
                    number_arg(0.0),
                    text_arg("fallback"),
                ],
                &NoResolver,
            ),
            Ok(EvalValue::Text(ExcelText::from_interop_assignment("EF")))
        );
    }

    #[test]
    fn textdelim_rejects_zero_instance_number() {
        assert_eq!(
            eval_textbefore_surface(
                &[text_arg("abc"), text_arg("b"), number_arg(0.0)],
                &NoResolver,
            ),
            Err(TextDelimEvalError::InvalidInstanceNum(0.0))
        );
    }
}

use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    coerce_prepared_to_number, coerce_prepared_to_text, prepare_args_values_only,
    run_values_only_prepared,
};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, ExcelText, WorksheetErrorCode};

pub const CHAR_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.CHAR",
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

pub const CODE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.CODE",
    kernel_signature_class: KernelSignatureClass::Custom,
    ..CHAR_META
};

pub const LOWER_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.LOWER",
    kernel_signature_class: KernelSignatureClass::TextToText,
    ..CHAR_META
};

pub const UPPER_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.UPPER",
    kernel_signature_class: KernelSignatureClass::TextToText,
    ..CHAR_META
};

pub const TRIM_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.TRIM",
    kernel_signature_class: KernelSignatureClass::TextToText,
    ..CHAR_META
};

pub const REPT_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.REPT",
    arity: Arity::exact(2),
    kernel_signature_class: KernelSignatureClass::Custom,
    ..CHAR_META
};

#[derive(Debug, Clone, PartialEq)]
pub enum TextScalarEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
    Domain(WorksheetErrorCode),
}

fn text_from_string(s: String) -> ExcelText {
    ExcelText::from_utf16_code_units(s.encode_utf16().collect())
}

fn truncate_toward_zero(n: f64) -> f64 {
    n.trunc()
}

fn char_from_number(n: f64) -> Result<ExcelText, TextScalarEvalError> {
    let n = truncate_toward_zero(n);
    if !(1.0..=255.0).contains(&n) {
        return Err(TextScalarEvalError::Domain(WorksheetErrorCode::Value));
    }
    Ok(ExcelText::from_utf16_code_units(vec![n as u16]))
}

fn code_of_text(text: &ExcelText) -> Result<f64, TextScalarEvalError> {
    match text.utf16_code_units().first().copied() {
        Some(unit) => Ok(unit as f64),
        None => Err(TextScalarEvalError::Domain(WorksheetErrorCode::Value)),
    }
}

fn lower_text(text: &ExcelText) -> ExcelText {
    text_from_string(String::from_utf16_lossy(text.utf16_code_units()).to_lowercase())
}

fn upper_text(text: &ExcelText) -> ExcelText {
    text_from_string(String::from_utf16_lossy(text.utf16_code_units()).to_uppercase())
}

fn trim_ascii_spaces(text: &ExcelText) -> ExcelText {
    let mut out = Vec::new();
    let mut pending_space = false;
    let mut started = false;
    for unit in text.utf16_code_units() {
        if *unit == 32 {
            if started {
                pending_space = true;
            }
            continue;
        }
        if pending_space && !out.is_empty() {
            out.push(32);
        }
        out.push(*unit);
        started = true;
        pending_space = false;
    }
    ExcelText::from_utf16_code_units(out)
}

fn rept_text(text: &ExcelText, count: f64) -> Result<ExcelText, TextScalarEvalError> {
    let count = truncate_toward_zero(count);
    if count < 0.0 {
        return Err(TextScalarEvalError::Domain(WorksheetErrorCode::Value));
    }
    let count = count as usize;
    let units = text.utf16_code_units();
    if units.len().saturating_mul(count) > 32767 {
        return Err(TextScalarEvalError::Domain(WorksheetErrorCode::Value));
    }
    let mut out = Vec::with_capacity(units.len().saturating_mul(count));
    for _ in 0..count {
        out.extend_from_slice(units);
    }
    Ok(ExcelText::from_utf16_code_units(out))
}

pub fn eval_char_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, TextScalarEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| {
            if !CHAR_META.arity.accepts(prepared.len()) {
                return Err(TextScalarEvalError::ArityMismatch {
                    expected_min: CHAR_META.arity.min,
                    expected_max: CHAR_META.arity.max,
                    actual: prepared.len(),
                });
            }
            let n =
                coerce_prepared_to_number(&prepared[0]).map_err(TextScalarEvalError::Coercion)?;
            Ok(EvalValue::Text(char_from_number(n)?))
        },
        TextScalarEvalError::Coercion,
    )
}

pub fn eval_code_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, TextScalarEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| {
            if !CODE_META.arity.accepts(prepared.len()) {
                return Err(TextScalarEvalError::ArityMismatch {
                    expected_min: CODE_META.arity.min,
                    expected_max: CODE_META.arity.max,
                    actual: prepared.len(),
                });
            }
            let text =
                coerce_prepared_to_text(&prepared[0]).map_err(TextScalarEvalError::Coercion)?;
            Ok(EvalValue::Number(code_of_text(&text)?))
        },
        TextScalarEvalError::Coercion,
    )
}

fn eval_text_unary_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
    meta: &FunctionMeta,
    kernel: fn(&ExcelText) -> ExcelText,
) -> Result<EvalValue, TextScalarEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| {
            if !meta.arity.accepts(prepared.len()) {
                return Err(TextScalarEvalError::ArityMismatch {
                    expected_min: meta.arity.min,
                    expected_max: meta.arity.max,
                    actual: prepared.len(),
                });
            }
            let text =
                coerce_prepared_to_text(&prepared[0]).map_err(TextScalarEvalError::Coercion)?;
            Ok(EvalValue::Text(kernel(&text)))
        },
        TextScalarEvalError::Coercion,
    )
}

pub fn eval_lower_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, TextScalarEvalError> {
    eval_text_unary_surface(args, resolver, &LOWER_META, lower_text)
}

pub fn eval_upper_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, TextScalarEvalError> {
    eval_text_unary_surface(args, resolver, &UPPER_META, upper_text)
}

pub fn eval_trim_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, TextScalarEvalError> {
    eval_text_unary_surface(args, resolver, &TRIM_META, trim_ascii_spaces)
}

pub fn eval_rept_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, TextScalarEvalError> {
    let prepared =
        prepare_args_values_only(args, resolver).map_err(TextScalarEvalError::Coercion)?;
    if !REPT_META.arity.accepts(prepared.len()) {
        return Err(TextScalarEvalError::ArityMismatch {
            expected_min: REPT_META.arity.min,
            expected_max: REPT_META.arity.max,
            actual: prepared.len(),
        });
    }
    let text = coerce_prepared_to_text(&prepared[0]).map_err(TextScalarEvalError::Coercion)?;
    let count = coerce_prepared_to_number(&prepared[1]).map_err(TextScalarEvalError::Coercion)?;
    Ok(EvalValue::Text(rept_text(&text, count)?))
}

pub fn map_text_scalar_error_to_ws(e: &TextScalarEvalError) -> WorksheetErrorCode {
    match e {
        TextScalarEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        TextScalarEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        TextScalarEvalError::Coercion(_) => WorksheetErrorCode::Value,
        TextScalarEvalError::Domain(code) => *code,
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

    #[test]
    fn char_truncates_and_rejects_out_of_range() {
        assert_eq!(
            eval_char_surface(&[CallArgValue::Eval(EvalValue::Number(65.9))], &NoResolver),
            Ok(EvalValue::Text(ExcelText::from_utf16_code_units(
                "A".encode_utf16().collect(),
            )))
        );
        assert_eq!(
            eval_char_surface(&[CallArgValue::Eval(EvalValue::Number(0.0))], &NoResolver),
            Err(TextScalarEvalError::Domain(WorksheetErrorCode::Value))
        );
    }

    #[test]
    fn code_uses_first_character_and_rejects_empty() {
        assert_eq!(
            eval_code_surface(
                &[CallArgValue::Eval(EvalValue::Text(
                    ExcelText::from_utf16_code_units("AB".encode_utf16().collect(),)
                ))],
                &NoResolver,
            ),
            Ok(EvalValue::Number(65.0))
        );
        assert_eq!(
            eval_code_surface(
                &[CallArgValue::Eval(EvalValue::Text(
                    ExcelText::from_utf16_code_units(Vec::new(),)
                ))],
                &NoResolver,
            ),
            Err(TextScalarEvalError::Domain(WorksheetErrorCode::Value))
        );
    }

    #[test]
    fn lower_and_upper_coerce_logicals_to_text() {
        assert_eq!(
            eval_lower_surface(&[CallArgValue::Eval(EvalValue::Logical(true))], &NoResolver),
            Ok(EvalValue::Text(ExcelText::from_utf16_code_units(
                "true".encode_utf16().collect(),
            )))
        );
        assert_eq!(
            eval_upper_surface(&[CallArgValue::Eval(EvalValue::Logical(true))], &NoResolver),
            Ok(EvalValue::Text(ExcelText::from_utf16_code_units(
                "TRUE".encode_utf16().collect(),
            )))
        );
    }

    #[test]
    fn trim_collapses_ascii_spaces_but_not_nbsp() {
        assert_eq!(
            trim_ascii_spaces(&ExcelText::from_utf16_code_units(
                " A   B ".encode_utf16().collect()
            )),
            ExcelText::from_utf16_code_units("A B".encode_utf16().collect())
        );
        assert_eq!(
            trim_ascii_spaces(&ExcelText::from_utf16_code_units(vec![160, 65, 160])),
            ExcelText::from_utf16_code_units(vec![160, 65, 160])
        );
    }

    #[test]
    fn rept_truncates_count_and_enforces_limit() {
        assert_eq!(
            eval_rept_surface(
                &[
                    CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
                        "ab".encode_utf16().collect(),
                    ))),
                    CallArgValue::Eval(EvalValue::Number(2.9)),
                ],
                &NoResolver,
            ),
            Ok(EvalValue::Text(ExcelText::from_utf16_code_units(
                "abab".encode_utf16().collect(),
            )))
        );
        assert_eq!(
            eval_rept_surface(
                &[
                    CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
                        "a".encode_utf16().collect(),
                    ))),
                    CallArgValue::Eval(EvalValue::Number(32768.0)),
                ],
                &NoResolver,
            ),
            Err(TextScalarEvalError::Domain(WorksheetErrorCode::Value))
        );
    }
}

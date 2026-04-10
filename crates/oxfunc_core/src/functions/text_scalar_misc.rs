use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    coerce_prepared_to_number, coerce_prepared_to_text, prepare_args_values_only,
};
use crate::resolver::ReferenceResolver;
use crate::value::{
    ArrayCellValue, CallArgValue, EvalArray, EvalValue, ExcelText, WorksheetErrorCode,
};

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

fn prepared_from_array_cell(cell: &ArrayCellValue) -> crate::functions::adapters::PreparedArgValue {
    match cell {
        ArrayCellValue::Number(n) => {
            crate::functions::adapters::PreparedArgValue::Eval(EvalValue::Number(*n))
        }
        ArrayCellValue::Text(t) => {
            crate::functions::adapters::PreparedArgValue::Eval(EvalValue::Text(t.clone()))
        }
        ArrayCellValue::Logical(b) => {
            crate::functions::adapters::PreparedArgValue::Eval(EvalValue::Logical(*b))
        }
        ArrayCellValue::Error(code) => {
            crate::functions::adapters::PreparedArgValue::Eval(EvalValue::Error(*code))
        }
        ArrayCellValue::EmptyCell => crate::functions::adapters::PreparedArgValue::EmptyCell,
    }
}

fn text_scalar_result_to_array_cell(
    result: Result<EvalValue, TextScalarEvalError>,
) -> ArrayCellValue {
    match result {
        Ok(EvalValue::Number(n)) => ArrayCellValue::Number(n),
        Ok(EvalValue::Text(text)) => ArrayCellValue::Text(text),
        Ok(EvalValue::Logical(value)) => ArrayCellValue::Logical(value),
        Ok(EvalValue::Error(code)) => ArrayCellValue::Error(code),
        Ok(_) => ArrayCellValue::Error(WorksheetErrorCode::Value),
        Err(err) => ArrayCellValue::Error(map_text_scalar_error_to_ws(&err)),
    }
}

fn eval_text_scalar_with_single_array_lift(
    prepared: &[crate::functions::adapters::PreparedArgValue],
    allowed_array_arg_indexes: &[usize],
    eval_scalar: impl Fn(
        &[crate::functions::adapters::PreparedArgValue],
    ) -> Result<EvalValue, TextScalarEvalError>,
) -> Result<EvalValue, TextScalarEvalError> {
    let array_args = prepared
        .iter()
        .enumerate()
        .filter_map(|(idx, arg)| match arg {
            crate::functions::adapters::PreparedArgValue::Eval(EvalValue::Array(array))
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
                    text_scalar_result_to_array_cell(eval_scalar(&scalar_args))
                })
                .collect();
            Ok(EvalValue::Array(
                EvalArray::new(array.shape(), cells)
                    .expect("text-scalar lifted array shape remains valid"),
            ))
        }
        _ => eval_scalar(prepared),
    }
}

fn eval_char_prepared_value(
    prepared: &[crate::functions::adapters::PreparedArgValue],
) -> Result<EvalValue, TextScalarEvalError> {
    if !CHAR_META.arity.accepts(prepared.len()) {
        return Err(TextScalarEvalError::ArityMismatch {
            expected_min: CHAR_META.arity.min,
            expected_max: CHAR_META.arity.max,
            actual: prepared.len(),
        });
    }
    let n = coerce_prepared_to_number(&prepared[0]).map_err(TextScalarEvalError::Coercion)?;
    Ok(EvalValue::Text(char_from_number(n)?))
}

fn eval_code_prepared_value(
    prepared: &[crate::functions::adapters::PreparedArgValue],
) -> Result<EvalValue, TextScalarEvalError> {
    if !CODE_META.arity.accepts(prepared.len()) {
        return Err(TextScalarEvalError::ArityMismatch {
            expected_min: CODE_META.arity.min,
            expected_max: CODE_META.arity.max,
            actual: prepared.len(),
        });
    }
    let text = coerce_prepared_to_text(&prepared[0]).map_err(TextScalarEvalError::Coercion)?;
    Ok(EvalValue::Number(code_of_text(&text)?))
}

fn eval_rept_prepared_value(
    prepared: &[crate::functions::adapters::PreparedArgValue],
) -> Result<EvalValue, TextScalarEvalError> {
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

pub fn eval_char_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, TextScalarEvalError> {
    let prepared =
        prepare_args_values_only(args, resolver).map_err(TextScalarEvalError::Coercion)?;
    eval_text_scalar_with_single_array_lift(&prepared, &[0], eval_char_prepared_value)
}

pub fn eval_code_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, TextScalarEvalError> {
    let prepared =
        prepare_args_values_only(args, resolver).map_err(TextScalarEvalError::Coercion)?;
    eval_text_scalar_with_single_array_lift(&prepared, &[0], eval_code_prepared_value)
}

fn eval_text_unary_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
    meta: &FunctionMeta,
    kernel: fn(&ExcelText) -> ExcelText,
) -> Result<EvalValue, TextScalarEvalError> {
    let prepared =
        prepare_args_values_only(args, resolver).map_err(TextScalarEvalError::Coercion)?;
    eval_text_scalar_with_single_array_lift(&prepared, &[0], |prepared| {
        if !meta.arity.accepts(prepared.len()) {
            return Err(TextScalarEvalError::ArityMismatch {
                expected_min: meta.arity.min,
                expected_max: meta.arity.max,
                actual: prepared.len(),
            });
        }
        let text = coerce_prepared_to_text(&prepared[0]).map_err(TextScalarEvalError::Coercion)?;
        Ok(EvalValue::Text(kernel(&text)))
    })
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
    eval_text_scalar_with_single_array_lift(&prepared, &[0, 1], eval_rept_prepared_value)
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

    #[test]
    fn char_spills_array_numbers() {
        let got = eval_char_surface(
            &[CallArgValue::Eval(EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Number(65.0)],
                    vec![ArrayCellValue::Number(66.0)],
                    vec![ArrayCellValue::Number(67.0)],
                ])
                .unwrap(),
            ))],
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Text(ExcelText::from_interop_assignment(
                        "A"
                    ))],
                    vec![ArrayCellValue::Text(ExcelText::from_interop_assignment(
                        "B"
                    ))],
                    vec![ArrayCellValue::Text(ExcelText::from_interop_assignment(
                        "C"
                    ))],
                ])
                .unwrap()
            ))
        );
    }

    #[test]
    fn code_spills_array_texts() {
        let got = eval_code_surface(
            &[CallArgValue::Eval(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Text(ExcelText::from_interop_assignment("A")),
                    ArrayCellValue::Text(ExcelText::from_interop_assignment("B")),
                ]])
                .unwrap(),
            ))],
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Number(65.0),
                    ArrayCellValue::Number(66.0),
                ]])
                .unwrap()
            ))
        );
    }

    #[test]
    fn lower_upper_and_trim_spill_array_texts() {
        assert_eq!(
            eval_lower_surface(
                &[CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![vec![
                        ArrayCellValue::Text(ExcelText::from_interop_assignment("A")),
                        ArrayCellValue::Text(ExcelText::from_interop_assignment("B")),
                    ]])
                    .unwrap(),
                ))],
                &NoResolver,
            ),
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Text(ExcelText::from_interop_assignment("a")),
                    ArrayCellValue::Text(ExcelText::from_interop_assignment("b")),
                ]])
                .unwrap()
            ))
        );
        assert_eq!(
            eval_upper_surface(
                &[CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![vec![
                        ArrayCellValue::Text(ExcelText::from_interop_assignment("a")),
                        ArrayCellValue::Text(ExcelText::from_interop_assignment("b")),
                    ]])
                    .unwrap(),
                ))],
                &NoResolver,
            ),
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Text(ExcelText::from_interop_assignment("A")),
                    ArrayCellValue::Text(ExcelText::from_interop_assignment("B")),
                ]])
                .unwrap()
            ))
        );
        assert_eq!(
            eval_trim_surface(
                &[CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![vec![
                        ArrayCellValue::Text(ExcelText::from_interop_assignment("  a  ")),
                        ArrayCellValue::Text(ExcelText::from_interop_assignment(" b ")),
                    ]])
                    .unwrap(),
                ))],
                &NoResolver,
            ),
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Text(ExcelText::from_interop_assignment("a")),
                    ArrayCellValue::Text(ExcelText::from_interop_assignment("b")),
                ]])
                .unwrap()
            ))
        );
    }

    #[test]
    fn rept_spills_array_counts_and_texts() {
        assert_eq!(
            eval_rept_surface(
                &[
                    CallArgValue::Eval(EvalValue::Text(ExcelText::from_interop_assignment("x"))),
                    CallArgValue::Eval(EvalValue::Array(
                        EvalArray::from_rows(vec![
                            vec![ArrayCellValue::Number(1.0)],
                            vec![ArrayCellValue::Number(2.0)],
                            vec![ArrayCellValue::Number(3.0)],
                        ])
                        .unwrap(),
                    )),
                ],
                &NoResolver,
            ),
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Text(ExcelText::from_interop_assignment(
                        "x"
                    ))],
                    vec![ArrayCellValue::Text(ExcelText::from_interop_assignment(
                        "xx"
                    ))],
                    vec![ArrayCellValue::Text(ExcelText::from_interop_assignment(
                        "xxx"
                    ))],
                ])
                .unwrap()
            ))
        );
        assert_eq!(
            eval_rept_surface(
                &[
                    CallArgValue::Eval(EvalValue::Array(
                        EvalArray::from_rows(vec![vec![
                            ArrayCellValue::Text(ExcelText::from_interop_assignment("a")),
                            ArrayCellValue::Text(ExcelText::from_interop_assignment("b")),
                        ]])
                        .unwrap(),
                    )),
                    CallArgValue::Eval(EvalValue::Number(2.0)),
                ],
                &NoResolver,
            ),
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Text(ExcelText::from_interop_assignment("aa")),
                    ArrayCellValue::Text(ExcelText::from_interop_assignment("bb")),
                ]])
                .unwrap()
            ))
        );
    }
}

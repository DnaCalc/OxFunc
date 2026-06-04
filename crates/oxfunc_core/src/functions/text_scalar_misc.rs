use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    coerce_prepared_to_number, coerce_prepared_to_text, prepare_args_values_only,
};
use crate::functions::excel_casing::{lower_text, upper_text};
use crate::resolver::ReferenceSystemProvider;
use crate::value::{CalcArray, CalcValue, CoreValue, ExcelText, WorksheetErrorCode};

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

fn prepared_from_array_cell(cell: &CalcValue) -> crate::functions::adapters::CalcValue {
    cell.clone()
}

fn text_scalar_result_to_array_cell(result: Result<CalcValue, TextScalarEvalError>) -> CalcValue {
    match result {
        Ok(value) => match value.core() {
            CoreValue::Number(_)
            | CoreValue::Text(_)
            | CoreValue::Logical(_)
            | CoreValue::Error(_) => value,
            _ => CalcValue::error(WorksheetErrorCode::Value),
        },
        Err(err) => CalcValue::error(map_text_scalar_error_to_ws(&err)),
    }
}

fn eval_text_scalar_with_single_array_lift(
    prepared: &[crate::functions::adapters::CalcValue],
    allowed_array_arg_indexes: &[usize],
    eval_scalar: impl Fn(
        &[crate::functions::adapters::CalcValue],
    ) -> Result<CalcValue, TextScalarEvalError>,
) -> Result<CalcValue, TextScalarEvalError> {
    let array_args = prepared
        .iter()
        .enumerate()
        .filter_map(|(idx, arg)| match arg.core() {
            CoreValue::Array(array) if allowed_array_arg_indexes.contains(&idx) => {
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
            Ok(CalcValue::array(
                CalcArray::new(array.shape(), cells)
                    .expect("text-scalar lifted array shape remains valid"),
            ))
        }
        _ => eval_scalar(prepared),
    }
}

fn eval_char_prepared_value(
    prepared: &[crate::functions::adapters::CalcValue],
) -> Result<CalcValue, TextScalarEvalError> {
    if !CHAR_META.arity.accepts(prepared.len()) {
        return Err(TextScalarEvalError::ArityMismatch {
            expected_min: CHAR_META.arity.min,
            expected_max: CHAR_META.arity.max,
            actual: prepared.len(),
        });
    }
    let n = coerce_prepared_to_number(&prepared[0]).map_err(TextScalarEvalError::Coercion)?;
    Ok(CalcValue::text(char_from_number(n)?))
}

fn eval_code_prepared_value(
    prepared: &[crate::functions::adapters::CalcValue],
) -> Result<CalcValue, TextScalarEvalError> {
    if !CODE_META.arity.accepts(prepared.len()) {
        return Err(TextScalarEvalError::ArityMismatch {
            expected_min: CODE_META.arity.min,
            expected_max: CODE_META.arity.max,
            actual: prepared.len(),
        });
    }
    let text = coerce_prepared_to_text(&prepared[0]).map_err(TextScalarEvalError::Coercion)?;
    Ok(CalcValue::number(code_of_text(&text)?))
}

fn eval_rept_prepared_value(
    prepared: &[crate::functions::adapters::CalcValue],
) -> Result<CalcValue, TextScalarEvalError> {
    if !REPT_META.arity.accepts(prepared.len()) {
        return Err(TextScalarEvalError::ArityMismatch {
            expected_min: REPT_META.arity.min,
            expected_max: REPT_META.arity.max,
            actual: prepared.len(),
        });
    }
    let text = coerce_prepared_to_text(&prepared[0]).map_err(TextScalarEvalError::Coercion)?;
    let count = coerce_prepared_to_number(&prepared[1]).map_err(TextScalarEvalError::Coercion)?;
    Ok(CalcValue::text(rept_text(&text, count)?))
}

pub fn eval_char_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, TextScalarEvalError> {
    let prepared =
        prepare_args_values_only(args, resolver).map_err(TextScalarEvalError::Coercion)?;
    eval_text_scalar_with_single_array_lift(&prepared, &[0], eval_char_prepared_value)
}

pub fn eval_code_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, TextScalarEvalError> {
    let prepared =
        prepare_args_values_only(args, resolver).map_err(TextScalarEvalError::Coercion)?;
    eval_text_scalar_with_single_array_lift(&prepared, &[0], eval_code_prepared_value)
}

fn eval_text_unary_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
    meta: &FunctionMeta,
    kernel: fn(&ExcelText) -> ExcelText,
) -> Result<CalcValue, TextScalarEvalError> {
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
        Ok(CalcValue::text(kernel(&text)))
    })
}

pub fn eval_lower_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, TextScalarEvalError> {
    eval_text_unary_surface(args, resolver, &LOWER_META, lower_text)
}

pub fn eval_upper_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, TextScalarEvalError> {
    eval_text_unary_surface(args, resolver, &UPPER_META, upper_text)
}

pub fn eval_trim_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, TextScalarEvalError> {
    eval_text_unary_surface(args, resolver, &TRIM_META, trim_ascii_spaces)
}

pub fn eval_rept_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, TextScalarEvalError> {
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
    use crate::resolver::ReferenceSystemCapabilities;

    struct NoResolver;

    impl ReferenceSystemProvider for NoResolver {
        fn capabilities(&self) -> ReferenceSystemCapabilities {
            ReferenceSystemCapabilities::permissive_local()
        }

        fn dereference(
            &self,
            request: &crate::resolver::ReferenceDereferenceRequest,
        ) -> Result<CalcValue, crate::resolver::ReferenceResolutionError> {
            let reference = &request.reference;
            Err(
                crate::resolver::ReferenceResolutionError::UnresolvedReference {
                    target: reference.target().to_string(),
                },
            )
        }
    }

    #[test]
    fn char_truncates_and_rejects_out_of_range() {
        assert_eq!(
            eval_char_surface(&[(CalcValue::number(65.9))], &NoResolver),
            Ok(CalcValue::text(ExcelText::from_utf16_code_units(
                "A".encode_utf16().collect(),
            )))
        );
        assert_eq!(
            eval_char_surface(&[(CalcValue::number(0.0))], &NoResolver),
            Err(TextScalarEvalError::Domain(WorksheetErrorCode::Value))
        );
    }

    #[test]
    fn code_uses_first_character_and_rejects_empty() {
        assert_eq!(
            eval_code_surface(
                &[(CalcValue::text(ExcelText::from_utf16_code_units(
                    "AB".encode_utf16().collect(),
                )))],
                &NoResolver,
            ),
            Ok(CalcValue::number(65.0))
        );
        assert_eq!(
            eval_code_surface(
                &[(CalcValue::text(ExcelText::from_utf16_code_units(Vec::new(),)))],
                &NoResolver,
            ),
            Err(TextScalarEvalError::Domain(WorksheetErrorCode::Value))
        );
    }

    #[test]
    fn lower_and_upper_coerce_logicals_to_text() {
        assert_eq!(
            eval_lower_surface(&[(CalcValue::logical(true))], &NoResolver),
            Ok(CalcValue::text(ExcelText::from_utf16_code_units(
                "true".encode_utf16().collect(),
            )))
        );
        assert_eq!(
            eval_upper_surface(&[(CalcValue::logical(true))], &NoResolver),
            Ok(CalcValue::text(ExcelText::from_utf16_code_units(
                "TRUE".encode_utf16().collect(),
            )))
        );
    }

    #[test]
    fn upper_preserves_german_sharp_s() {
        assert_eq!(
            eval_upper_surface(
                &[(CalcValue::text(ExcelText::from_interop_assignment("straße")))],
                &NoResolver,
            ),
            Ok(CalcValue::text(ExcelText::from_interop_assignment(
                "STRAßE"
            )))
        );
    }

    // Current repo-local theory note:
    // OxFunc now routes worksheet text casing through a shared Excel-style casing helper.
    // The observed worksheet behavior is not full Unicode special casing; it is closer to
    // simple single-codepoint mappings with selected script-aware behavior such as Greek final
    // sigma on lowering, no Turkish locale-sensitive dotted-I expansion, and no ß expansion in
    // UPPER. The matrix below pins the currently observed Excel-aligned lanes.
    #[test]
    fn unicode_casing_matrix_matches_excel_observed_rows() {
        let cases = [
            (
                "UPPER straße",
                eval_upper_surface(
                    &[(CalcValue::text(ExcelText::from_interop_assignment("straße")))],
                    &NoResolver,
                ),
                Ok(CalcValue::text(ExcelText::from_interop_assignment(
                    "STRAßE",
                ))),
            ),
            (
                "LOWER STRAẞE",
                eval_lower_surface(
                    &[(CalcValue::text(ExcelText::from_interop_assignment("STRAẞE")))],
                    &NoResolver,
                ),
                Ok(CalcValue::text(ExcelText::from_interop_assignment(
                    "straẞe",
                ))),
            ),
            (
                "UPPER weiß",
                eval_upper_surface(
                    &[(CalcValue::text(ExcelText::from_interop_assignment("weiß")))],
                    &NoResolver,
                ),
                Ok(CalcValue::text(ExcelText::from_interop_assignment("WEIß"))),
            ),
            (
                "UPPER İstanbul",
                eval_upper_surface(
                    &[(CalcValue::text(ExcelText::from_interop_assignment("İstanbul")))],
                    &NoResolver,
                ),
                Ok(CalcValue::text(ExcelText::from_interop_assignment(
                    "İSTANBUL",
                ))),
            ),
            (
                "LOWER İSTANBUL",
                eval_lower_surface(
                    &[(CalcValue::text(ExcelText::from_interop_assignment("İSTANBUL")))],
                    &NoResolver,
                ),
                Ok(CalcValue::text(ExcelText::from_interop_assignment(
                    "istanbul",
                ))),
            ),
            (
                "UPPER istanbul",
                eval_upper_surface(
                    &[(CalcValue::text(ExcelText::from_interop_assignment("istanbul")))],
                    &NoResolver,
                ),
                Ok(CalcValue::text(ExcelText::from_interop_assignment(
                    "ISTANBUL",
                ))),
            ),
            (
                "LOWER I",
                eval_lower_surface(
                    &[(CalcValue::text(ExcelText::from_interop_assignment("I")))],
                    &NoResolver,
                ),
                Ok(CalcValue::text(ExcelText::from_interop_assignment("i"))),
            ),
            (
                "LOWER İ",
                eval_lower_surface(
                    &[(CalcValue::text(ExcelText::from_interop_assignment("İ")))],
                    &NoResolver,
                ),
                Ok(CalcValue::text(ExcelText::from_interop_assignment("i"))),
            ),
            (
                "UPPER κόσμος",
                eval_upper_surface(
                    &[(CalcValue::text(ExcelText::from_interop_assignment("κόσμος")))],
                    &NoResolver,
                ),
                Ok(CalcValue::text(ExcelText::from_interop_assignment(
                    "ΚΟΣΜΟΣ",
                ))),
            ),
            (
                "LOWER ΟΣ",
                eval_lower_surface(
                    &[(CalcValue::text(ExcelText::from_interop_assignment("ΟΣ")))],
                    &NoResolver,
                ),
                Ok(CalcValue::text(ExcelText::from_interop_assignment("ος"))),
            ),
            (
                "UPPER café",
                eval_upper_surface(
                    &[(CalcValue::text(ExcelText::from_interop_assignment("café")))],
                    &NoResolver,
                ),
                Ok(CalcValue::text(ExcelText::from_interop_assignment("CAFÉ"))),
            ),
            (
                "UPPER Ångström",
                eval_upper_surface(
                    &[(CalcValue::text(ExcelText::from_interop_assignment("Ångström")))],
                    &NoResolver,
                ),
                Ok(CalcValue::text(ExcelText::from_interop_assignment(
                    "ÅNGSTRÖM",
                ))),
            ),
        ];

        for (name, got, expected) in cases {
            assert_eq!(got, expected, "{name}");
        }
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
                    (CalcValue::text(ExcelText::from_utf16_code_units(
                        "ab".encode_utf16().collect(),
                    ))),
                    (CalcValue::number(2.9)),
                ],
                &NoResolver,
            ),
            Ok(CalcValue::text(ExcelText::from_utf16_code_units(
                "abab".encode_utf16().collect(),
            )))
        );
        assert_eq!(
            eval_rept_surface(
                &[
                    (CalcValue::text(ExcelText::from_utf16_code_units(
                        "a".encode_utf16().collect(),
                    ))),
                    (CalcValue::number(32768.0)),
                ],
                &NoResolver,
            ),
            Err(TextScalarEvalError::Domain(WorksheetErrorCode::Value))
        );
    }

    #[test]
    fn char_spills_array_numbers() {
        let got = eval_char_surface(
            &[(CalcValue::array(
                CalcArray::from_rows(vec![
                    vec![CalcValue::number(65.0)],
                    vec![CalcValue::number(66.0)],
                    vec![CalcValue::number(67.0)],
                ])
                .unwrap(),
            ))],
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(CalcValue::array(
                CalcArray::from_rows(vec![
                    vec![CalcValue::text(ExcelText::from_interop_assignment("A"))],
                    vec![CalcValue::text(ExcelText::from_interop_assignment("B"))],
                    vec![CalcValue::text(ExcelText::from_interop_assignment("C"))],
                ])
                .unwrap()
            ))
        );
    }

    #[test]
    fn code_spills_array_texts() {
        let got = eval_code_surface(
            &[(CalcValue::array(
                CalcArray::from_rows(vec![vec![
                    CalcValue::text(ExcelText::from_interop_assignment("A")),
                    CalcValue::text(ExcelText::from_interop_assignment("B")),
                ]])
                .unwrap(),
            ))],
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(CalcValue::array(
                CalcArray::from_rows(vec![
                    vec![CalcValue::number(65.0), CalcValue::number(66.0),]
                ])
                .unwrap()
            ))
        );
    }

    #[test]
    fn lower_upper_and_trim_spill_array_texts() {
        assert_eq!(
            eval_lower_surface(
                &[(CalcValue::array(
                    CalcArray::from_rows(vec![vec![
                        CalcValue::text(ExcelText::from_interop_assignment("A")),
                        CalcValue::text(ExcelText::from_interop_assignment("B")),
                    ]])
                    .unwrap(),
                ))],
                &NoResolver,
            ),
            Ok(CalcValue::array(
                CalcArray::from_rows(vec![vec![
                    CalcValue::text(ExcelText::from_interop_assignment("a")),
                    CalcValue::text(ExcelText::from_interop_assignment("b")),
                ]])
                .unwrap()
            ))
        );
        assert_eq!(
            eval_upper_surface(
                &[(CalcValue::array(
                    CalcArray::from_rows(vec![vec![
                        CalcValue::text(ExcelText::from_interop_assignment("a")),
                        CalcValue::text(ExcelText::from_interop_assignment("b")),
                    ]])
                    .unwrap(),
                ))],
                &NoResolver,
            ),
            Ok(CalcValue::array(
                CalcArray::from_rows(vec![vec![
                    CalcValue::text(ExcelText::from_interop_assignment("A")),
                    CalcValue::text(ExcelText::from_interop_assignment("B")),
                ]])
                .unwrap()
            ))
        );
        assert_eq!(
            eval_trim_surface(
                &[(CalcValue::array(
                    CalcArray::from_rows(vec![vec![
                        CalcValue::text(ExcelText::from_interop_assignment("  a  ")),
                        CalcValue::text(ExcelText::from_interop_assignment(" b ")),
                    ]])
                    .unwrap(),
                ))],
                &NoResolver,
            ),
            Ok(CalcValue::array(
                CalcArray::from_rows(vec![vec![
                    CalcValue::text(ExcelText::from_interop_assignment("a")),
                    CalcValue::text(ExcelText::from_interop_assignment("b")),
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
                    (CalcValue::text(ExcelText::from_interop_assignment("x"))),
                    (CalcValue::array(
                        CalcArray::from_rows(vec![
                            vec![CalcValue::number(1.0)],
                            vec![CalcValue::number(2.0)],
                            vec![CalcValue::number(3.0)],
                        ])
                        .unwrap(),
                    )),
                ],
                &NoResolver,
            ),
            Ok(CalcValue::array(
                CalcArray::from_rows(vec![
                    vec![CalcValue::text(ExcelText::from_interop_assignment("x"))],
                    vec![CalcValue::text(ExcelText::from_interop_assignment("xx"))],
                    vec![CalcValue::text(ExcelText::from_interop_assignment("xxx"))],
                ])
                .unwrap()
            ))
        );
        assert_eq!(
            eval_rept_surface(
                &[
                    (CalcValue::array(
                        CalcArray::from_rows(vec![vec![
                            CalcValue::text(ExcelText::from_interop_assignment("a")),
                            CalcValue::text(ExcelText::from_interop_assignment("b")),
                        ]])
                        .unwrap(),
                    )),
                    (CalcValue::number(2.0)),
                ],
                &NoResolver,
            ),
            Ok(CalcValue::array(
                CalcArray::from_rows(vec![vec![
                    CalcValue::text(ExcelText::from_interop_assignment("aa")),
                    CalcValue::text(ExcelText::from_interop_assignment("bb")),
                ]])
                .unwrap()
            ))
        );
    }
}

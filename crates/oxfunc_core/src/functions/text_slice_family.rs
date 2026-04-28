use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    PreparedArgValue, coerce_prepared_to_number, coerce_prepared_to_text, prepare_args_values_only,
    run_values_only_prepared,
};
use crate::resolver::ReferenceResolver;
use crate::value::{
    ArrayCellValue, CallArgValue, EvalArray, EvalValue, ExcelText, WorksheetErrorCode,
};

const TEXT_SLICE_BASE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.TEXT_SLICE_BASE",
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

pub const LEN_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.LEN",
    ..TEXT_SLICE_BASE_META
};

pub const LEFT_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.LEFT",
    arity: Arity { min: 1, max: 2 },
    ..TEXT_SLICE_BASE_META
};

pub const RIGHT_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.RIGHT",
    arity: Arity { min: 1, max: 2 },
    ..TEXT_SLICE_BASE_META
};

pub const MID_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.MID",
    arity: Arity::exact(3),
    ..TEXT_SLICE_BASE_META
};

#[derive(Debug, Clone, PartialEq)]
pub enum TextSliceEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
    Domain(WorksheetErrorCode),
}

fn domain_value_error() -> TextSliceEvalError {
    TextSliceEvalError::Domain(WorksheetErrorCode::Value)
}

fn nonnegative_count_from_number(n: f64) -> Result<usize, TextSliceEvalError> {
    if !n.is_finite() {
        return Err(domain_value_error());
    }

    let truncated = n.trunc();
    if truncated < 0.0 {
        return Err(domain_value_error());
    }
    if truncated > usize::MAX as f64 {
        return Ok(usize::MAX);
    }

    Ok(truncated as usize)
}

fn one_based_start_from_number(n: f64) -> Result<usize, TextSliceEvalError> {
    if !n.is_finite() {
        return Err(domain_value_error());
    }

    let truncated = n.trunc();
    if truncated < 1.0 {
        return Err(domain_value_error());
    }
    if truncated > usize::MAX as f64 {
        return Ok(usize::MAX);
    }

    Ok(truncated as usize)
}

fn take_left_units(text: &ExcelText, count: usize) -> ExcelText {
    let end = count.min(text.len_utf16_code_units());
    ExcelText::from_utf16_code_units(text.utf16_code_units()[..end].to_vec())
}

fn take_right_units(text: &ExcelText, count: usize) -> ExcelText {
    let take_count = count.min(text.len_utf16_code_units());
    let start = text.len_utf16_code_units() - take_count;
    ExcelText::from_utf16_code_units(text.utf16_code_units()[start..].to_vec())
}

fn take_mid_units(text: &ExcelText, start_one_based: usize, count: usize) -> ExcelText {
    if count == 0 {
        return ExcelText::from_utf16_code_units(Vec::new());
    }

    let start_index = start_one_based.saturating_sub(1);
    if start_index >= text.len_utf16_code_units() {
        return ExcelText::from_utf16_code_units(Vec::new());
    }

    let end_index = start_index
        .saturating_add(count)
        .min(text.len_utf16_code_units());
    ExcelText::from_utf16_code_units(text.utf16_code_units()[start_index..end_index].to_vec())
}

fn len_character_count(text: &ExcelText) -> usize {
    std::char::decode_utf16(text.utf16_code_units().iter().copied()).count()
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

fn text_slice_result_to_array_cell(
    result: Result<EvalValue, TextSliceEvalError>,
) -> ArrayCellValue {
    match result {
        Ok(EvalValue::Text(text)) => ArrayCellValue::Text(text),
        Ok(EvalValue::Error(code)) => ArrayCellValue::Error(code),
        Ok(_) => ArrayCellValue::Error(WorksheetErrorCode::Value),
        Err(err) => ArrayCellValue::Error(map_text_slice_error_to_ws(&err)),
    }
}

fn eval_text_slice_with_single_array_lift(
    prepared: &[PreparedArgValue],
    eval_scalar: impl Fn(&[PreparedArgValue]) -> Result<EvalValue, TextSliceEvalError>,
) -> Result<EvalValue, TextSliceEvalError> {
    let array_args = prepared
        .iter()
        .enumerate()
        .filter_map(|(idx, arg)| match arg {
            PreparedArgValue::Eval(EvalValue::Array(array)) => Some((idx, array)),
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
                    text_slice_result_to_array_cell(eval_scalar(&scalar_args))
                })
                .collect();
            Ok(EvalValue::Array(
                EvalArray::new(array.shape(), cells)
                    .expect("text-slice lifted array shape remains valid"),
            ))
        }
        _ => eval_scalar(prepared),
    }
}

fn eval_left_prepared_value(
    prepared: &[PreparedArgValue],
) -> Result<EvalValue, TextSliceEvalError> {
    if !LEFT_META.arity.accepts(prepared.len()) {
        return Err(TextSliceEvalError::ArityMismatch {
            expected_min: LEFT_META.arity.min,
            expected_max: LEFT_META.arity.max,
            actual: prepared.len(),
        });
    }

    let text = coerce_prepared_to_text(&prepared[0]).map_err(TextSliceEvalError::Coercion)?;
    let count = resolve_optional_count(prepared)?;
    Ok(EvalValue::Text(take_left_units(&text, count)))
}

fn eval_right_prepared_value(
    prepared: &[PreparedArgValue],
) -> Result<EvalValue, TextSliceEvalError> {
    if !RIGHT_META.arity.accepts(prepared.len()) {
        return Err(TextSliceEvalError::ArityMismatch {
            expected_min: RIGHT_META.arity.min,
            expected_max: RIGHT_META.arity.max,
            actual: prepared.len(),
        });
    }

    let text = coerce_prepared_to_text(&prepared[0]).map_err(TextSliceEvalError::Coercion)?;
    let count = resolve_optional_count(prepared)?;
    Ok(EvalValue::Text(take_right_units(&text, count)))
}

fn eval_mid_prepared_value(prepared: &[PreparedArgValue]) -> Result<EvalValue, TextSliceEvalError> {
    if !MID_META.arity.accepts(prepared.len()) {
        return Err(TextSliceEvalError::ArityMismatch {
            expected_min: MID_META.arity.min,
            expected_max: MID_META.arity.max,
            actual: prepared.len(),
        });
    }

    let text = coerce_prepared_to_text(&prepared[0]).map_err(TextSliceEvalError::Coercion)?;
    let start = coerce_prepared_to_number(&prepared[1]).map_err(TextSliceEvalError::Coercion)?;
    let count = coerce_prepared_to_number(&prepared[2]).map_err(TextSliceEvalError::Coercion)?;
    let start = one_based_start_from_number(start)?;
    let count = nonnegative_count_from_number(count)?;
    Ok(EvalValue::Text(take_mid_units(&text, start, count)))
}

pub fn eval_len_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, TextSliceEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| {
            if !LEN_META.arity.accepts(prepared.len()) {
                return Err(TextSliceEvalError::ArityMismatch {
                    expected_min: LEN_META.arity.min,
                    expected_max: LEN_META.arity.max,
                    actual: prepared.len(),
                });
            }

            let text =
                coerce_prepared_to_text(&prepared[0]).map_err(TextSliceEvalError::Coercion)?;
            Ok(EvalValue::Number(len_character_count(&text) as f64))
        },
        TextSliceEvalError::Coercion,
    )
}

fn resolve_optional_count(prepared: &[PreparedArgValue]) -> Result<usize, TextSliceEvalError> {
    if prepared.len() == 1 {
        return Ok(1);
    }

    let count = coerce_prepared_to_number(&prepared[1]).map_err(TextSliceEvalError::Coercion)?;
    nonnegative_count_from_number(count)
}

pub fn eval_left_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, TextSliceEvalError> {
    let prepared =
        prepare_args_values_only(args, resolver).map_err(TextSliceEvalError::Coercion)?;
    eval_text_slice_with_single_array_lift(&prepared, eval_left_prepared_value)
}

pub fn eval_right_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, TextSliceEvalError> {
    let prepared =
        prepare_args_values_only(args, resolver).map_err(TextSliceEvalError::Coercion)?;
    eval_text_slice_with_single_array_lift(&prepared, eval_right_prepared_value)
}

pub fn eval_mid_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, TextSliceEvalError> {
    let prepared =
        prepare_args_values_only(args, resolver).map_err(TextSliceEvalError::Coercion)?;
    eval_text_slice_with_single_array_lift(&prepared, eval_mid_prepared_value)
}

pub fn map_text_slice_error_to_ws(e: &TextSliceEvalError) -> WorksheetErrorCode {
    match e {
        TextSliceEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        TextSliceEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        TextSliceEvalError::Coercion(_) => WorksheetErrorCode::Value,
        TextSliceEvalError::Domain(code) => *code,
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

    fn text_value(units: Vec<u16>) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(units)))
    }

    fn number_value(n: f64) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Number(n))
    }

    #[test]
    fn ftc_0640_len_counts_unicode_scalars_for_surrogate_pairs() {
        let emoji = text_value(vec![0xD83D, 0xDE00]);
        let combining = text_value(vec![0x0065, 0x0301]);
        let dangling_tail = ExcelText::from_interop_assignment(&"\u{1F600}".repeat(40_000));

        assert_eq!(
            eval_len_surface(&[emoji], &NoResolver),
            Ok(EvalValue::Number(1.0))
        );
        assert_eq!(
            eval_len_surface(&[combining], &NoResolver),
            Ok(EvalValue::Number(2.0))
        );
        assert!(dangling_tail.has_dangling_high_surrogate_tail());
        assert_eq!(
            eval_len_surface(
                &[CallArgValue::Eval(EvalValue::Text(dangling_tail.clone()))],
                &NoResolver,
            ),
            Ok(EvalValue::Number(16_384.0))
        );
    }

    #[test]
    fn left_defaults_to_one_and_slices_utf16_code_units() {
        assert_eq!(
            eval_left_surface(&[text_value("ABC".encode_utf16().collect())], &NoResolver),
            Ok(EvalValue::Text(ExcelText::from_utf16_code_units(
                "A".encode_utf16().collect(),
            )))
        );
        assert_eq!(
            eval_left_surface(
                &[text_value(vec![0xD83D, 0xDE00]), number_value(1.0)],
                &NoResolver
            ),
            Ok(EvalValue::Text(ExcelText::from_utf16_code_units(vec![
                0xD83D
            ])))
        );
        assert_eq!(
            eval_left_surface(
                &[
                    CallArgValue::Eval(EvalValue::Logical(true)),
                    number_value(2.0),
                ],
                &NoResolver,
            ),
            Ok(EvalValue::Text(ExcelText::from_utf16_code_units(
                "TR".encode_utf16().collect(),
            )))
        );
    }

    #[test]
    fn right_defaults_to_one_and_slices_utf16_code_units() {
        assert_eq!(
            eval_right_surface(&[text_value("ABC".encode_utf16().collect())], &NoResolver),
            Ok(EvalValue::Text(ExcelText::from_utf16_code_units(
                "C".encode_utf16().collect(),
            )))
        );
        assert_eq!(
            eval_right_surface(
                &[text_value(vec![0xD83D, 0xDE00]), number_value(1.0)],
                &NoResolver
            ),
            Ok(EvalValue::Text(ExcelText::from_utf16_code_units(vec![
                0xDE00
            ])))
        );
        assert_eq!(
            eval_right_surface(
                &[text_value("AB".encode_utf16().collect()), number_value(9.0)],
                &NoResolver,
            ),
            Ok(EvalValue::Text(ExcelText::from_utf16_code_units(
                "AB".encode_utf16().collect(),
            )))
        );
    }

    #[test]
    fn mid_uses_one_based_truncated_offsets_and_raw_code_units() {
        assert_eq!(
            eval_mid_surface(
                &[
                    text_value("ABCD".encode_utf16().collect()),
                    number_value(2.9),
                    number_value(1.9),
                ],
                &NoResolver,
            ),
            Ok(EvalValue::Text(ExcelText::from_utf16_code_units(
                "B".encode_utf16().collect(),
            )))
        );
        assert_eq!(
            eval_mid_surface(
                &[
                    text_value("ABCD".encode_utf16().collect()),
                    number_value(10.0),
                    number_value(2.0),
                ],
                &NoResolver,
            ),
            Ok(EvalValue::Text(
                ExcelText::from_utf16_code_units(Vec::new())
            ))
        );
        assert_eq!(
            eval_mid_surface(
                &[
                    text_value(vec![0xD83D, 0xDE00]),
                    number_value(2.0),
                    number_value(1.0),
                ],
                &NoResolver,
            ),
            Ok(EvalValue::Text(ExcelText::from_utf16_code_units(vec![
                0xDE00
            ])))
        );
    }

    #[test]
    fn text_slice_domain_lanes_match_excel_value_errors() {
        assert_eq!(
            eval_left_surface(
                &[
                    text_value("ABC".encode_utf16().collect()),
                    number_value(-1.0)
                ],
                &NoResolver,
            ),
            Err(TextSliceEvalError::Domain(WorksheetErrorCode::Value))
        );
        assert_eq!(
            eval_right_surface(
                &[
                    text_value("ABC".encode_utf16().collect()),
                    number_value(-1.0)
                ],
                &NoResolver,
            ),
            Err(TextSliceEvalError::Domain(WorksheetErrorCode::Value))
        );
        assert_eq!(
            eval_mid_surface(
                &[
                    text_value("ABC".encode_utf16().collect()),
                    number_value(0.0),
                    number_value(1.0),
                ],
                &NoResolver,
            ),
            Err(TextSliceEvalError::Domain(WorksheetErrorCode::Value))
        );
        assert_eq!(
            eval_mid_surface(
                &[
                    text_value("ABC".encode_utf16().collect()),
                    number_value(1.0),
                    number_value(-1.0),
                ],
                &NoResolver,
            ),
            Err(TextSliceEvalError::Domain(WorksheetErrorCode::Value))
        );
    }

    #[test]
    fn left_and_right_truncate_fractional_counts_and_allow_zero_length_results() {
        assert_eq!(
            eval_left_surface(
                &[
                    text_value("ABCD".encode_utf16().collect()),
                    number_value(1.9)
                ],
                &NoResolver,
            ),
            Ok(EvalValue::Text(ExcelText::from_utf16_code_units(
                "A".encode_utf16().collect(),
            )))
        );
        assert_eq!(
            eval_right_surface(
                &[
                    text_value("ABCD".encode_utf16().collect()),
                    number_value(1.9)
                ],
                &NoResolver,
            ),
            Ok(EvalValue::Text(ExcelText::from_utf16_code_units(
                "D".encode_utf16().collect(),
            )))
        );
        assert_eq!(
            eval_left_surface(
                &[
                    text_value("ABCD".encode_utf16().collect()),
                    number_value(0.9)
                ],
                &NoResolver,
            ),
            Ok(EvalValue::Text(
                ExcelText::from_utf16_code_units(Vec::new())
            ))
        );
        assert_eq!(
            eval_right_surface(
                &[
                    text_value("ABCD".encode_utf16().collect()),
                    number_value(0.9)
                ],
                &NoResolver,
            ),
            Ok(EvalValue::Text(
                ExcelText::from_utf16_code_units(Vec::new())
            ))
        );
    }

    #[test]
    fn left_spills_array_counts() {
        let got = eval_left_surface(
            &[
                text_value("MISSISSIPPI".encode_utf16().collect()),
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
        );
        assert_eq!(
            got,
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Text(ExcelText::from_utf16_code_units(
                        "M".encode_utf16().collect(),
                    ))],
                    vec![ArrayCellValue::Text(ExcelText::from_utf16_code_units(
                        "MI".encode_utf16().collect(),
                    ))],
                    vec![ArrayCellValue::Text(ExcelText::from_utf16_code_units(
                        "MIS".encode_utf16().collect(),
                    ))],
                ])
                .unwrap()
            ))
        );
    }

    #[test]
    fn right_spills_array_counts() {
        let got = eval_right_surface(
            &[
                text_value("MISSISSIPPI".encode_utf16().collect()),
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
        );
        assert_eq!(
            got,
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Text(ExcelText::from_utf16_code_units(
                        "I".encode_utf16().collect(),
                    ))],
                    vec![ArrayCellValue::Text(ExcelText::from_utf16_code_units(
                        "PI".encode_utf16().collect(),
                    ))],
                    vec![ArrayCellValue::Text(ExcelText::from_utf16_code_units(
                        "PPI".encode_utf16().collect(),
                    ))],
                ])
                .unwrap()
            ))
        );
    }

    #[test]
    fn mid_spills_array_start_positions() {
        let got = eval_mid_surface(
            &[
                text_value("MISSISSIPPI".encode_utf16().collect()),
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![
                        vec![ArrayCellValue::Number(1.0)],
                        vec![ArrayCellValue::Number(2.0)],
                        vec![ArrayCellValue::Number(3.0)],
                        vec![ArrayCellValue::Number(4.0)],
                        vec![ArrayCellValue::Number(5.0)],
                        vec![ArrayCellValue::Number(6.0)],
                        vec![ArrayCellValue::Number(7.0)],
                        vec![ArrayCellValue::Number(8.0)],
                        vec![ArrayCellValue::Number(9.0)],
                        vec![ArrayCellValue::Number(10.0)],
                        vec![ArrayCellValue::Number(11.0)],
                    ])
                    .unwrap(),
                )),
                number_value(1.0),
            ],
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Text(ExcelText::from_utf16_code_units(
                        "M".encode_utf16().collect(),
                    ))],
                    vec![ArrayCellValue::Text(ExcelText::from_utf16_code_units(
                        "I".encode_utf16().collect(),
                    ))],
                    vec![ArrayCellValue::Text(ExcelText::from_utf16_code_units(
                        "S".encode_utf16().collect(),
                    ))],
                    vec![ArrayCellValue::Text(ExcelText::from_utf16_code_units(
                        "S".encode_utf16().collect(),
                    ))],
                    vec![ArrayCellValue::Text(ExcelText::from_utf16_code_units(
                        "I".encode_utf16().collect(),
                    ))],
                    vec![ArrayCellValue::Text(ExcelText::from_utf16_code_units(
                        "S".encode_utf16().collect(),
                    ))],
                    vec![ArrayCellValue::Text(ExcelText::from_utf16_code_units(
                        "S".encode_utf16().collect(),
                    ))],
                    vec![ArrayCellValue::Text(ExcelText::from_utf16_code_units(
                        "I".encode_utf16().collect(),
                    ))],
                    vec![ArrayCellValue::Text(ExcelText::from_utf16_code_units(
                        "P".encode_utf16().collect(),
                    ))],
                    vec![ArrayCellValue::Text(ExcelText::from_utf16_code_units(
                        "P".encode_utf16().collect(),
                    ))],
                    vec![ArrayCellValue::Text(ExcelText::from_utf16_code_units(
                        "I".encode_utf16().collect(),
                    ))],
                ])
                .unwrap()
            ))
        );
    }

    #[test]
    fn len_treats_empty_cell_as_empty_text() {
        assert_eq!(
            eval_len_surface(&[CallArgValue::EmptyCell], &NoResolver),
            Ok(EvalValue::Number(0.0))
        );
    }
}

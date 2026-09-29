use crate::coercion::CoercionError;
use crate::function::{
    Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile, FunctionMeta,
    HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    coerce_prepared_to_number, coerce_prepared_to_text, prepare_args_values_only,
};
use crate::resolver::ReferenceSystemProvider;
use crate::value::{ArrayShape, CalcArray, CalcValue, CoreValue, ExcelText, WorksheetErrorCode};

/// Text scalar functions evaluate error precedence after shape expansion.
/// A missing coordinate is an argument #N/A, not an unconditional result #N/A:
/// an earlier explicit argument error can still be the cell's result.
pub(crate) fn run_text_lifted<E>(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
    on_cell: impl Fn(&[CalcValue]) -> Result<CalcValue, E>,
    map_error: impl Fn(&E) -> WorksheetErrorCode,
    map_preparation_error: impl FnOnce(CoercionError) -> E,
) -> Result<CalcValue, E> {
    let prepared = prepare_args_values_only(args, resolver).map_err(map_preparation_error)?;
    let shape = prepared
        .iter()
        .fold(ArrayShape { rows: 1, cols: 1 }, |out, arg| {
            let shape = match arg.core() {
                CoreValue::Array(a) => a.shape(),
                _ => ArrayShape { rows: 1, cols: 1 },
            };
            ArrayShape {
                rows: out.rows.max(shape.rows),
                cols: out.cols.max(shape.cols),
            }
        });
    if shape.rows == 1 && shape.cols == 1 {
        return on_cell(&prepared);
    }
    let mut cells = Vec::with_capacity(shape.cell_count());
    for row in 0..shape.rows {
        for col in 0..shape.cols {
            let values = prepared
                .iter()
                .map(|arg| match arg.core() {
                    CoreValue::Array(a) => a
                        .get(
                            if a.shape().rows == 1 { 0 } else { row },
                            if a.shape().cols == 1 { 0 } else { col },
                        )
                        .cloned()
                        .unwrap_or_else(|| CalcValue::error(WorksheetErrorCode::NA)),
                    _ => arg.clone(),
                })
                .collect::<Vec<_>>();
            cells
                .push(on_cell(&values).unwrap_or_else(|error| CalcValue::error(map_error(&error))));
        }
    }
    Ok(CalcValue::array(
        CalcArray::new(shape, cells).expect("text broadcast shape preserved"),
    ))
}

const TEXT_SLICE_BASE_META: FunctionMeta = function_spec! {
    function_id: "FUNC.TEXT_SLICE_BASE",
    arity: Arity::exact(1),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
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

fn nonnegative_count_from_number(n: f64, tolerant: bool) -> Result<usize, TextSliceEvalError> {
    if !n.is_finite() || n < 0.0 || (!tolerant && n >= 2147483648.0) {
        return Err(domain_value_error());
    }

    // LEFT/RIGHT share the characterized ADDRESS/DATE integer boundary.
    // MID/MIDB use plain truncation and signed-32-bit admission instead.
    let truncated = if tolerant {
        crate::functions::date_fn::date_integer_floor(n)
    } else {
        n.trunc()
    };
    if truncated < 0.0 {
        return Err(domain_value_error());
    }
    if truncated > usize::MAX as f64 {
        return Ok(usize::MAX);
    }

    Ok(truncated as usize)
}

fn one_based_start_from_number(n: f64) -> Result<usize, TextSliceEvalError> {
    if !n.is_finite() || n >= 2147483648.0 {
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

fn character_boundary(text: &ExcelText, count: usize) -> usize {
    let units = text.utf16_code_units();
    let (mut offset, mut seen) = (0, 0);
    while offset < units.len() && seen < count {
        let paired = (0xD800..=0xDBFF).contains(&units[offset])
            && units
                .get(offset + 1)
                .is_some_and(|next| (0xDC00..=0xDFFF).contains(next));
        offset += if paired { 2 } else { 1 };
        seen += 1;
    }
    offset
}

fn take_left_units(text: &ExcelText, count: usize) -> ExcelText {
    // If the forward character scan stops at an unmatched final high surrogate
    // before reaching count, Excel falls back to the original raw-unit count.
    // It can therefore split an earlier pair; the maximum-length heldout is a
    // strong discriminator from clipping to the successfully scanned prefix.
    let end = if character_scan_end(text) < text.len_utf16_code_units()
        && count > len_character_count(text)
    {
        count.min(text.len_utf16_code_units())
    } else {
        character_boundary(text, count)
    };
    ExcelText::from_utf16_code_units(text.utf16_code_units()[..end].to_vec())
}

fn take_right_units(text: &ExcelText, count: usize) -> ExcelText {
    let start = character_boundary(text, slice_character_count(text).saturating_sub(count));
    ExcelText::from_utf16_code_units(text.utf16_code_units()[start..].to_vec())
}

fn take_mid_units(
    text: &ExcelText,
    start_one_based: usize,
    count: usize,
    raw_units: bool,
) -> ExcelText {
    if count == 0 {
        return ExcelText::from_utf16_code_units(Vec::new());
    }

    let start_index = if raw_units {
        start_one_based.saturating_sub(1)
    } else {
        character_boundary(text, start_one_based.saturating_sub(1))
    };
    let scan_end = if raw_units {
        text.len_utf16_code_units()
    } else {
        character_scan_end(text)
    };
    if start_index >= scan_end {
        return ExcelText::from_utf16_code_units(Vec::new());
    }

    let end_index = if raw_units {
        start_index
            .saturating_add(count)
            .min(text.len_utf16_code_units())
    } else {
        character_boundary(
            text,
            start_one_based.saturating_sub(1).saturating_add(count),
        )
        .min(scan_end)
    };
    ExcelText::from_utf16_code_units(text.utf16_code_units()[start_index..end_index].to_vec())
}

fn slice_character_count(text: &ExcelText) -> usize {
    std::char::decode_utf16(text.utf16_code_units().iter().copied()).count()
}

fn character_scan_end(text: &ExcelText) -> usize {
    text.len_utf16_code_units()
        - usize::from(
            text.utf16_code_units()
                .last()
                .is_some_and(|unit| (0xD800..=0xDBFF).contains(unit)),
        )
}

fn len_character_count(text: &ExcelText) -> usize {
    // The CV2 forward count does not count a trailing unmatched high surrogate.
    // RIGHT traverses from the end and preserves it; LEFT has a raw-count
    // fallback when the character scan does not reach count. Raw probes bind all
    // three paths separately instead of routing through lossy Rust strings.
    slice_character_count(text)
        - usize::from(character_scan_end(text) < text.len_utf16_code_units())
}

fn eval_left_prepared_value(prepared: &[CalcValue]) -> Result<CalcValue, TextSliceEvalError> {
    if !LEFT_META.arity.accepts(prepared.len()) {
        return Err(TextSliceEvalError::ArityMismatch {
            expected_min: LEFT_META.arity.min,
            expected_max: LEFT_META.arity.max,
            actual: prepared.len(),
        });
    }

    let text = coerce_prepared_to_text(&prepared[0]).map_err(TextSliceEvalError::Coercion)?;
    let count = resolve_optional_count(prepared)?;
    Ok(CalcValue::text(take_left_units(&text, count)))
}

fn eval_right_prepared_value(prepared: &[CalcValue]) -> Result<CalcValue, TextSliceEvalError> {
    if !RIGHT_META.arity.accepts(prepared.len()) {
        return Err(TextSliceEvalError::ArityMismatch {
            expected_min: RIGHT_META.arity.min,
            expected_max: RIGHT_META.arity.max,
            actual: prepared.len(),
        });
    }

    let text = coerce_prepared_to_text(&prepared[0]).map_err(TextSliceEvalError::Coercion)?;
    let count = resolve_optional_count(prepared)?;
    Ok(CalcValue::text(take_right_units(&text, count)))
}

fn numeric_slot(arg: &CalcValue) -> Result<f64, TextSliceEvalError> {
    if matches!(arg.core(), CoreValue::Missing) {
        Ok(0.0)
    } else {
        coerce_prepared_to_number(arg).map_err(TextSliceEvalError::Coercion)
    }
}

fn eval_mid_prepared_value(
    prepared: &[CalcValue],
    raw_units: bool,
) -> Result<CalcValue, TextSliceEvalError> {
    if !MID_META.arity.accepts(prepared.len()) {
        return Err(TextSliceEvalError::ArityMismatch {
            expected_min: MID_META.arity.min,
            expected_max: MID_META.arity.max,
            actual: prepared.len(),
        });
    }

    let text = coerce_prepared_to_text(&prepared[0]).map_err(TextSliceEvalError::Coercion)?;
    let start = numeric_slot(&prepared[1])?;
    let count = numeric_slot(&prepared[2])?;
    let start = one_based_start_from_number(start)?;
    let count = nonnegative_count_from_number(count, false)?;
    Ok(CalcValue::text(take_mid_units(
        &text, start, count, raw_units,
    )))
}

pub fn eval_len_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, TextSliceEvalError> {
    run_text_lifted(
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
            Ok(CalcValue::number(len_character_count(&text) as f64))
        },
        map_text_slice_error_to_ws,
        TextSliceEvalError::Coercion,
    )
}

fn resolve_optional_count(prepared: &[CalcValue]) -> Result<usize, TextSliceEvalError> {
    if prepared.len() == 1 {
        return Ok(1);
    }

    let count = numeric_slot(&prepared[1])?;
    nonnegative_count_from_number(count, true)
}

pub fn eval_left_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, TextSliceEvalError> {
    run_text_lifted(
        args,
        resolver,
        eval_left_prepared_value,
        map_text_slice_error_to_ws,
        TextSliceEvalError::Coercion,
    )
}

pub fn eval_right_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, TextSliceEvalError> {
    run_text_lifted(
        args,
        resolver,
        eval_right_prepared_value,
        map_text_slice_error_to_ws,
        TextSliceEvalError::Coercion,
    )
}

pub fn eval_mid_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, TextSliceEvalError> {
    run_text_lifted(
        args,
        resolver,
        |prepared| eval_mid_prepared_value(prepared, false),
        map_text_slice_error_to_ws,
        TextSliceEvalError::Coercion,
    )
}

pub(crate) fn eval_mid_utf16_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, TextSliceEvalError> {
    run_text_lifted(
        args,
        resolver,
        |prepared| eval_mid_prepared_value(prepared, true),
        map_text_slice_error_to_ws,
        TextSliceEvalError::Coercion,
    )
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
    use crate::resolver::ReferenceSystemCapabilities;
    use crate::value::CalcArray;

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

    fn text_value(units: Vec<u16>) -> CalcValue {
        CalcValue::text(ExcelText::from_utf16_code_units(units))
    }

    fn number_value(n: f64) -> CalcValue {
        CalcValue::number(n)
    }

    #[test]
    fn ftc_0640_len_counts_unicode_scalars_for_surrogate_pairs() {
        let emoji = text_value(vec![0xD83D, 0xDE00]);
        let combining = text_value(vec![0x0065, 0x0301]);
        let dangling_tail = ExcelText::from_interop_assignment(&"\u{1F600}".repeat(40_000));

        assert_eq!(
            eval_len_surface(&[emoji], &NoResolver),
            Ok(CalcValue::number(1.0))
        );
        assert_eq!(
            eval_len_surface(&[combining], &NoResolver),
            Ok(CalcValue::number(2.0))
        );
        assert!(dangling_tail.has_dangling_high_surrogate_tail());
        assert_eq!(
            eval_len_surface(&[(CalcValue::text(dangling_tail.clone()))], &NoResolver,),
            Ok(CalcValue::number(16_383.0))
        );
    }

    #[test]
    fn left_defaults_to_one_and_preserves_surrogate_pairs() {
        assert_eq!(
            eval_left_surface(&[text_value("ABC".encode_utf16().collect())], &NoResolver),
            Ok(CalcValue::text(ExcelText::from_utf16_code_units(
                "A".encode_utf16().collect(),
            )))
        );
        assert_eq!(
            eval_left_surface(
                &[text_value(vec![0xD83D, 0xDE00]), number_value(1.0)],
                &NoResolver
            ),
            Ok(CalcValue::text(ExcelText::from_utf16_code_units(vec![
                0xD83D, 0xDE00
            ])))
        );
        assert_eq!(
            eval_left_surface(
                &[(CalcValue::logical(true)), number_value(2.0),],
                &NoResolver,
            ),
            Ok(CalcValue::text(ExcelText::from_utf16_code_units(
                "TR".encode_utf16().collect(),
            )))
        );
    }

    #[test]
    fn right_defaults_to_one_and_preserves_surrogate_pairs() {
        assert_eq!(
            eval_right_surface(&[text_value("ABC".encode_utf16().collect())], &NoResolver),
            Ok(CalcValue::text(ExcelText::from_utf16_code_units(
                "C".encode_utf16().collect(),
            )))
        );
        assert_eq!(
            eval_right_surface(
                &[text_value(vec![0xD83D, 0xDE00]), number_value(1.0)],
                &NoResolver
            ),
            Ok(CalcValue::text(ExcelText::from_utf16_code_units(vec![
                0xD83D, 0xDE00
            ])))
        );
        assert_eq!(
            eval_right_surface(
                &[text_value("AB".encode_utf16().collect()), number_value(9.0)],
                &NoResolver,
            ),
            Ok(CalcValue::text(ExcelText::from_utf16_code_units(
                "AB".encode_utf16().collect(),
            )))
        );
    }

    #[test]
    fn mid_uses_one_based_character_offsets() {
        assert_eq!(
            eval_mid_surface(
                &[
                    text_value("ABCD".encode_utf16().collect()),
                    number_value(2.9),
                    number_value(1.9),
                ],
                &NoResolver,
            ),
            Ok(CalcValue::text(ExcelText::from_utf16_code_units(
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
            Ok(CalcValue::text(
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
            Ok(CalcValue::text(ExcelText::from_utf16_code_units(vec![])))
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
            Ok(CalcValue::text(ExcelText::from_utf16_code_units(
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
            Ok(CalcValue::text(ExcelText::from_utf16_code_units(
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
            Ok(CalcValue::text(
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
            Ok(CalcValue::text(
                ExcelText::from_utf16_code_units(Vec::new())
            ))
        );
    }

    #[test]
    fn left_spills_array_counts() {
        let got = eval_left_surface(
            &[
                text_value("MISSISSIPPI".encode_utf16().collect()),
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
        );
        assert_eq!(
            got,
            Ok(CalcValue::array(
                CalcArray::from_rows(vec![
                    vec![CalcValue::text(ExcelText::from_utf16_code_units(
                        "M".encode_utf16().collect(),
                    ))],
                    vec![CalcValue::text(ExcelText::from_utf16_code_units(
                        "MI".encode_utf16().collect(),
                    ))],
                    vec![CalcValue::text(ExcelText::from_utf16_code_units(
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
        );
        assert_eq!(
            got,
            Ok(CalcValue::array(
                CalcArray::from_rows(vec![
                    vec![CalcValue::text(ExcelText::from_utf16_code_units(
                        "I".encode_utf16().collect(),
                    ))],
                    vec![CalcValue::text(ExcelText::from_utf16_code_units(
                        "PI".encode_utf16().collect(),
                    ))],
                    vec![CalcValue::text(ExcelText::from_utf16_code_units(
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
                (CalcValue::array(
                    CalcArray::from_rows(vec![
                        vec![CalcValue::number(1.0)],
                        vec![CalcValue::number(2.0)],
                        vec![CalcValue::number(3.0)],
                        vec![CalcValue::number(4.0)],
                        vec![CalcValue::number(5.0)],
                        vec![CalcValue::number(6.0)],
                        vec![CalcValue::number(7.0)],
                        vec![CalcValue::number(8.0)],
                        vec![CalcValue::number(9.0)],
                        vec![CalcValue::number(10.0)],
                        vec![CalcValue::number(11.0)],
                    ])
                    .unwrap(),
                )),
                number_value(1.0),
            ],
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(CalcValue::array(
                CalcArray::from_rows(vec![
                    vec![CalcValue::text(ExcelText::from_utf16_code_units(
                        "M".encode_utf16().collect(),
                    ))],
                    vec![CalcValue::text(ExcelText::from_utf16_code_units(
                        "I".encode_utf16().collect(),
                    ))],
                    vec![CalcValue::text(ExcelText::from_utf16_code_units(
                        "S".encode_utf16().collect(),
                    ))],
                    vec![CalcValue::text(ExcelText::from_utf16_code_units(
                        "S".encode_utf16().collect(),
                    ))],
                    vec![CalcValue::text(ExcelText::from_utf16_code_units(
                        "I".encode_utf16().collect(),
                    ))],
                    vec![CalcValue::text(ExcelText::from_utf16_code_units(
                        "S".encode_utf16().collect(),
                    ))],
                    vec![CalcValue::text(ExcelText::from_utf16_code_units(
                        "S".encode_utf16().collect(),
                    ))],
                    vec![CalcValue::text(ExcelText::from_utf16_code_units(
                        "I".encode_utf16().collect(),
                    ))],
                    vec![CalcValue::text(ExcelText::from_utf16_code_units(
                        "P".encode_utf16().collect(),
                    ))],
                    vec![CalcValue::text(ExcelText::from_utf16_code_units(
                        "P".encode_utf16().collect(),
                    ))],
                    vec![CalcValue::text(ExcelText::from_utf16_code_units(
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
            eval_len_surface(&[CalcValue::empty()], &NoResolver),
            Ok(CalcValue::number(0.0))
        );
    }
}

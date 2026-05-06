use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    PreparedArgValue, coerce_prepared_to_number, coerce_prepared_to_text, prepare_args_values_only,
};
use crate::resolver::ReferenceResolver;
use crate::value::{
    ArrayCellValue, ArrayShape, CallArgValue, EvalArray, EvalValue, ExcelText, WorksheetErrorCode,
};

const ARRAY_TEXT_SPLIT_BASE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.ARRAY_TEXT_SPLIT_BASE",
    arity: Arity { min: 1, max: 6 },
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
};

pub const ARRAYTOTEXT_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.ARRAYTOTEXT",
    arity: Arity { min: 1, max: 2 },
    kernel_signature_class: KernelSignatureClass::TextToText,
    ..ARRAY_TEXT_SPLIT_BASE_META
};

pub const TEXTSPLIT_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.TEXTSPLIT",
    arity: Arity { min: 2, max: 6 },
    ..ARRAY_TEXT_SPLIT_BASE_META
};

#[derive(Debug, Clone, PartialEq)]
pub enum ArrayTextSplitEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
    InvalidArrayToTextFormat(f64),
    InvalidIgnoreEmpty(f64),
    InvalidMatchMode(f64),
    MissingDelimiter,
    UnsupportedPadWith(&'static str),
}

fn empty_text() -> ExcelText {
    ExcelText::from_utf16_code_units(Vec::new())
}

fn worksheet_error_literal(code: WorksheetErrorCode) -> &'static str {
    match code {
        WorksheetErrorCode::Null => "#NULL!",
        WorksheetErrorCode::Div0 => "#DIV/0!",
        WorksheetErrorCode::Value => "#VALUE!",
        WorksheetErrorCode::Ref => "#REF!",
        WorksheetErrorCode::Name => "#NAME?",
        WorksheetErrorCode::Num => "#NUM!",
        WorksheetErrorCode::NA => "#N/A",
        WorksheetErrorCode::Busy => "#BUSY!",
        WorksheetErrorCode::GettingData => "#GETTING_DATA",
        WorksheetErrorCode::Spill => "#SPILL!",
        WorksheetErrorCode::Calc => "#CALC!",
        WorksheetErrorCode::Field => "#FIELD!",
        WorksheetErrorCode::Blocked => "#BLOCKED!",
        WorksheetErrorCode::Connect => "#CONNECT!",
    }
}

fn scalar_cell_from_prepared(arg: &PreparedArgValue) -> ArrayCellValue {
    match arg {
        PreparedArgValue::Eval(EvalValue::Number(n)) => ArrayCellValue::Number(*n),
        PreparedArgValue::Eval(EvalValue::Text(t)) => ArrayCellValue::Text(t.clone()),
        PreparedArgValue::Eval(EvalValue::Logical(b)) => ArrayCellValue::Logical(*b),
        PreparedArgValue::Eval(EvalValue::Error(code)) => ArrayCellValue::Error(*code),
        PreparedArgValue::Eval(EvalValue::Reference(_))
        | PreparedArgValue::Eval(EvalValue::Lambda(_)) => {
            ArrayCellValue::Error(WorksheetErrorCode::Value)
        }
        PreparedArgValue::Eval(EvalValue::Array(_)) => unreachable!(),
        PreparedArgValue::MissingArg | PreparedArgValue::EmptyCell => ArrayCellValue::EmptyCell,
    }
}

fn eval_value_to_array_cell(value: &EvalValue) -> Result<ArrayCellValue, ArrayTextSplitEvalError> {
    match value {
        EvalValue::Number(n) => Ok(ArrayCellValue::Number(*n)),
        EvalValue::Text(t) => Ok(ArrayCellValue::Text(t.clone())),
        EvalValue::Logical(b) => Ok(ArrayCellValue::Logical(*b)),
        EvalValue::Error(code) => Ok(ArrayCellValue::Error(*code)),
        EvalValue::Array(_) => Err(ArrayTextSplitEvalError::UnsupportedPadWith("array")),
        EvalValue::Reference(_) => Err(ArrayTextSplitEvalError::UnsupportedPadWith(
            "reference_like",
        )),
        EvalValue::Lambda(_) => Err(ArrayTextSplitEvalError::UnsupportedPadWith("lambda_value")),
    }
}

fn materialize_arraytotext_input(prepared: &PreparedArgValue) -> EvalArray {
    match prepared {
        PreparedArgValue::Eval(EvalValue::Array(array)) => array.clone(),
        other => EvalArray::from_scalar(scalar_cell_from_prepared(other)),
    }
}

fn parse_truncated_flag(
    prepared: &PreparedArgValue,
    invalid: fn(f64) -> ArrayTextSplitEvalError,
) -> Result<bool, ArrayTextSplitEvalError> {
    let raw = coerce_prepared_to_number(prepared).map_err(ArrayTextSplitEvalError::Coercion)?;
    if !raw.is_finite() {
        return Err(invalid(raw));
    }
    match raw.trunc() {
        0.0 => Ok(false),
        1.0 => Ok(true),
        other => Err(invalid(other)),
    }
}

fn parse_arraytotext_format(
    prepared: Option<&PreparedArgValue>,
) -> Result<bool, ArrayTextSplitEvalError> {
    match prepared {
        None | Some(PreparedArgValue::MissingArg) | Some(PreparedArgValue::EmptyCell) => Ok(false),
        Some(arg) => {
            let raw = coerce_prepared_to_number(arg).map_err(ArrayTextSplitEvalError::Coercion)?;
            if !raw.is_finite() {
                return Err(ArrayTextSplitEvalError::InvalidArrayToTextFormat(raw));
            }
            match raw.trunc() {
                0.0 => Ok(false),
                1.0 => Ok(true),
                other => Err(ArrayTextSplitEvalError::InvalidArrayToTextFormat(other)),
            }
        }
    }
}

fn array_cell_to_concise_fragment(cell: &ArrayCellValue) -> String {
    match cell {
        ArrayCellValue::Number(n) => format!("{n}"),
        ArrayCellValue::Text(t) => t.to_string_lossy(),
        ArrayCellValue::Logical(b) => {
            if *b {
                "TRUE".to_string()
            } else {
                "FALSE".to_string()
            }
        }
        ArrayCellValue::Error(code) => worksheet_error_literal(*code).to_string(),
        ArrayCellValue::EmptyCell => String::new(),
    }
}

fn escape_strict_text(text: &ExcelText) -> String {
    text.to_string_lossy().replace('"', "\"\"")
}

fn array_cell_to_strict_fragment(cell: &ArrayCellValue) -> String {
    match cell {
        ArrayCellValue::Number(n) => format!("{n}"),
        ArrayCellValue::Text(t) => format!("\"{}\"", escape_strict_text(t)),
        ArrayCellValue::Logical(b) => {
            if *b {
                "TRUE".to_string()
            } else {
                "FALSE".to_string()
            }
        }
        ArrayCellValue::Error(code) => worksheet_error_literal(*code).to_string(),
        ArrayCellValue::EmptyCell => String::new(),
    }
}

fn arraytotext_concise(array: &EvalArray) -> ExcelText {
    let joined = array
        .iter_row_major()
        .map(array_cell_to_concise_fragment)
        .collect::<Vec<_>>()
        .join(", ");
    ExcelText::from_utf16_code_units(joined.encode_utf16().collect())
}

fn arraytotext_strict(array: &EvalArray) -> ExcelText {
    let mut rows = Vec::with_capacity(array.shape().rows);
    for row in 0..array.shape().rows {
        let row_text = array
            .row_slice(row)
            .expect("row is in bounds")
            .iter()
            .map(array_cell_to_strict_fragment)
            .collect::<Vec<_>>()
            .join(",");
        rows.push(row_text);
    }
    let rendered = format!("{{{}}}", rows.join(";"));
    ExcelText::from_utf16_code_units(rendered.encode_utf16().collect())
}

pub fn eval_arraytotext_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, ArrayTextSplitEvalError> {
    let prepared =
        prepare_args_values_only(args, resolver).map_err(ArrayTextSplitEvalError::Coercion)?;
    if !ARRAYTOTEXT_META.arity.accepts(prepared.len()) {
        return Err(ArrayTextSplitEvalError::ArityMismatch {
            expected_min: ARRAYTOTEXT_META.arity.min,
            expected_max: ARRAYTOTEXT_META.arity.max,
            actual: prepared.len(),
        });
    }

    let strict = parse_arraytotext_format(prepared.get(1))?;
    let array = materialize_arraytotext_input(&prepared[0]);
    Ok(EvalValue::Text(if strict {
        arraytotext_strict(&array)
    } else {
        arraytotext_concise(&array)
    }))
}

fn fold_ascii_case(unit: u16) -> u16 {
    if (b'A' as u16..=b'Z' as u16).contains(&unit) {
        unit + 32
    } else {
        unit
    }
}

fn utf16_match(left: &[u16], right: &[u16], case_insensitive: bool) -> bool {
    left.len() == right.len()
        && left.iter().zip(right.iter()).all(|(l, r)| {
            if case_insensitive {
                fold_ascii_case(*l) == fold_ascii_case(*r)
            } else {
                l == r
            }
        })
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

fn delimiter_list_from_prepared(
    prepared: &PreparedArgValue,
) -> Result<Option<Vec<ExcelText>>, ArrayTextSplitEvalError> {
    match prepared {
        PreparedArgValue::MissingArg => Ok(None),
        PreparedArgValue::EmptyCell => Ok(Some(vec![empty_text()])),
        PreparedArgValue::Eval(EvalValue::Array(array)) => {
            let mut out = Vec::with_capacity(array.shape().rows * array.shape().cols);
            for cell in array.iter_row_major() {
                let prepared_cell = prepared_from_array_cell(cell);
                out.push(
                    coerce_prepared_to_text(&prepared_cell)
                        .map_err(ArrayTextSplitEvalError::Coercion)?,
                );
            }
            Ok(Some(out))
        }
        _ => Ok(Some(vec![
            coerce_prepared_to_text(prepared).map_err(ArrayTextSplitEvalError::Coercion)?,
        ])),
    }
}

fn split_per_utf16_unit(units: &[u16], ignore_empty: bool) -> Vec<ExcelText> {
    if units.is_empty() {
        return if ignore_empty {
            Vec::new()
        } else {
            vec![empty_text()]
        };
    }
    units
        .iter()
        .map(|unit| ExcelText::from_utf16_code_units(vec![*unit]))
        .collect()
}

fn find_next_delimiter(
    units: &[u16],
    start: usize,
    delimiters: &[ExcelText],
    case_insensitive: bool,
) -> Option<(usize, usize)> {
    for idx in start..=units.len() {
        for delimiter in delimiters {
            let needle = delimiter.utf16_code_units();
            if needle.is_empty() || idx + needle.len() > units.len() {
                continue;
            }
            if utf16_match(&units[idx..idx + needle.len()], needle, case_insensitive) {
                return Some((idx, needle.len()));
            }
        }
    }
    None
}

fn split_text_by_delimiters(
    text: &ExcelText,
    delimiters: &[ExcelText],
    ignore_empty: bool,
    case_insensitive: bool,
) -> Vec<ExcelText> {
    if delimiters.is_empty() {
        return vec![text.clone()];
    }
    if delimiters
        .iter()
        .any(|delimiter| delimiter.utf16_code_units().is_empty())
    {
        return split_per_utf16_unit(text.utf16_code_units(), ignore_empty);
    }

    let units = text.utf16_code_units();
    let mut out = Vec::new();
    let mut start = 0usize;
    while let Some((idx, len)) = find_next_delimiter(units, start, delimiters, case_insensitive) {
        if idx > start || !ignore_empty {
            out.push(ExcelText::from_utf16_code_units(units[start..idx].to_vec()));
        }
        start = idx + len;
    }
    if start < units.len() || !ignore_empty {
        out.push(ExcelText::from_utf16_code_units(units[start..].to_vec()));
    }
    out
}

fn parse_pad_with(
    prepared: Option<&PreparedArgValue>,
) -> Result<ArrayCellValue, ArrayTextSplitEvalError> {
    match prepared {
        None | Some(PreparedArgValue::MissingArg) => {
            Ok(ArrayCellValue::Error(WorksheetErrorCode::NA))
        }
        Some(PreparedArgValue::EmptyCell) => Ok(ArrayCellValue::Number(0.0)),
        Some(PreparedArgValue::Eval(value)) => eval_value_to_array_cell(value),
    }
}

pub fn eval_textsplit_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, ArrayTextSplitEvalError> {
    let prepared =
        prepare_args_values_only(args, resolver).map_err(ArrayTextSplitEvalError::Coercion)?;
    if !TEXTSPLIT_META.arity.accepts(prepared.len()) {
        return Err(ArrayTextSplitEvalError::ArityMismatch {
            expected_min: TEXTSPLIT_META.arity.min,
            expected_max: TEXTSPLIT_META.arity.max,
            actual: prepared.len(),
        });
    }

    let text = coerce_prepared_to_text(&prepared[0]).map_err(ArrayTextSplitEvalError::Coercion)?;
    let col_delimiters = delimiter_list_from_prepared(&prepared[1])?;
    let row_delimiters = if prepared.len() >= 3 {
        delimiter_list_from_prepared(&prepared[2])?
    } else {
        None
    };
    if col_delimiters.is_none() && row_delimiters.is_none() {
        return Err(ArrayTextSplitEvalError::MissingDelimiter);
    }

    let ignore_empty = if prepared.len() >= 4 {
        parse_truncated_flag(&prepared[3], ArrayTextSplitEvalError::InvalidIgnoreEmpty)?
    } else {
        false
    };
    let case_insensitive = if prepared.len() >= 5 {
        parse_truncated_flag(&prepared[4], ArrayTextSplitEvalError::InvalidMatchMode)?
    } else {
        false
    };
    let pad_with = parse_pad_with(prepared.get(5))?;

    let mut row_parts = if let Some(delimiters) = row_delimiters.as_ref() {
        split_text_by_delimiters(&text, delimiters, ignore_empty, case_insensitive)
    } else {
        vec![text.clone()]
    };
    if row_parts.is_empty() {
        row_parts.push(empty_text());
    }

    let mut rows: Vec<Vec<ArrayCellValue>> = Vec::with_capacity(row_parts.len());
    for row_text in row_parts {
        let mut cols = if let Some(delimiters) = col_delimiters.as_ref() {
            split_text_by_delimiters(&row_text, delimiters, ignore_empty, case_insensitive)
        } else {
            vec![row_text]
        };
        if cols.is_empty() {
            cols.push(empty_text());
        }
        rows.push(cols.into_iter().map(ArrayCellValue::Text).collect());
    }

    let row_count = rows.len().max(1);
    let col_count = rows.iter().map(Vec::len).max().unwrap_or(1).max(1);
    let mut cells = Vec::with_capacity(row_count * col_count);
    for row in rows {
        for cell in &row {
            cells.push(cell.clone());
        }
        for _ in row.len()..col_count {
            cells.push(pad_with.clone());
        }
    }

    Ok(EvalValue::Array(
        EvalArray::new(
            ArrayShape {
                rows: row_count,
                cols: col_count,
            },
            cells,
        )
        .expect("textsplit dimensions are computed"),
    ))
}

pub fn map_array_text_split_error_to_ws(error: &ArrayTextSplitEvalError) -> WorksheetErrorCode {
    match error {
        ArrayTextSplitEvalError::ArityMismatch { .. }
        | ArrayTextSplitEvalError::InvalidArrayToTextFormat(_)
        | ArrayTextSplitEvalError::InvalidIgnoreEmpty(_)
        | ArrayTextSplitEvalError::InvalidMatchMode(_)
        | ArrayTextSplitEvalError::MissingDelimiter
        | ArrayTextSplitEvalError::UnsupportedPadWith(_) => WorksheetErrorCode::Value,
        ArrayTextSplitEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        ArrayTextSplitEvalError::Coercion(_) => WorksheetErrorCode::Value,
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
    fn arraytotext_concise_formats_row_major_values() {
        let got = eval_arraytotext_surface(
            &[
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![
                        vec![
                            ArrayCellValue::Logical(true),
                            ArrayCellValue::Error(WorksheetErrorCode::Value),
                        ],
                        vec![
                            ArrayCellValue::Text(ExcelText::from_interop_assignment("Hello")),
                            ArrayCellValue::Number(2.0),
                        ],
                    ])
                    .unwrap(),
                )),
                number_arg(0.0),
            ],
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Text(ExcelText::from_interop_assignment(
                "TRUE, #VALUE!, Hello, 2"
            )))
        );
    }

    #[test]
    fn arraytotext_strict_quotes_text_and_preserves_shape_markers() {
        let got = eval_arraytotext_surface(
            &[
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![
                        vec![
                            ArrayCellValue::Logical(true),
                            ArrayCellValue::Error(WorksheetErrorCode::Value),
                        ],
                        vec![
                            ArrayCellValue::Text(ExcelText::from_interop_assignment("Hello")),
                            ArrayCellValue::Number(2.0),
                        ],
                    ])
                    .unwrap(),
                )),
                number_arg(1.0),
            ],
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Text(ExcelText::from_interop_assignment(
                "{TRUE,#VALUE!;\"Hello\",2}"
            )))
        );
    }

    #[test]
    fn arraytotext_rejects_invalid_format_mode() {
        assert_eq!(
            eval_arraytotext_surface(&[text_arg("x"), number_arg(2.0)], &NoResolver),
            Err(ArrayTextSplitEvalError::InvalidArrayToTextFormat(2.0))
        );
    }

    #[test]
    fn textsplit_splits_across_columns() {
        let got = eval_textsplit_surface(
            &[text_arg("Dakota Lennon Sanchez"), text_arg(" ")],
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Text(ExcelText::from_interop_assignment("Dakota")),
                    ArrayCellValue::Text(ExcelText::from_interop_assignment("Lennon")),
                    ArrayCellValue::Text(ExcelText::from_interop_assignment("Sanchez")),
                ]])
                .unwrap(),
            ))
        );
    }

    #[test]
    fn textsplit_splits_rows_and_columns_and_pads_default_na() {
        let got = eval_textsplit_surface(
            &[text_arg("1,2,3;4,5"), text_arg(","), text_arg(";")],
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![
                        ArrayCellValue::Text(ExcelText::from_interop_assignment("1")),
                        ArrayCellValue::Text(ExcelText::from_interop_assignment("2")),
                        ArrayCellValue::Text(ExcelText::from_interop_assignment("3")),
                    ],
                    vec![
                        ArrayCellValue::Text(ExcelText::from_interop_assignment("4")),
                        ArrayCellValue::Text(ExcelText::from_interop_assignment("5")),
                        ArrayCellValue::Error(WorksheetErrorCode::NA),
                    ],
                ])
                .unwrap(),
            ))
        );
    }

    #[test]
    fn textsplit_supports_multiple_delimiters_and_ignore_empty() {
        let got = eval_textsplit_surface(
            &[
                text_arg("Do. Or do not. There is no try. -Anonymous"),
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![vec![
                        ArrayCellValue::Text(ExcelText::from_interop_assignment(".")),
                        ArrayCellValue::Text(ExcelText::from_interop_assignment("-")),
                    ]])
                    .unwrap(),
                )),
                CallArgValue::MissingArg,
                number_arg(1.0),
            ],
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Text(ExcelText::from_interop_assignment("Do")),
                    ArrayCellValue::Text(ExcelText::from_interop_assignment(" Or do not")),
                    ArrayCellValue::Text(ExcelText::from_interop_assignment(" There is no try")),
                    ArrayCellValue::Text(ExcelText::from_interop_assignment(" ")),
                    ArrayCellValue::Text(ExcelText::from_interop_assignment("Anonymous")),
                ]])
                .unwrap(),
            ))
        );
    }

    #[test]
    fn textsplit_honors_case_insensitive_match_mode_and_custom_padding() {
        let got = eval_textsplit_surface(
            &[
                text_arg("aXbxc"),
                text_arg("x"),
                CallArgValue::MissingArg,
                number_arg(0.0),
                number_arg(1.0),
                text_arg("pad"),
            ],
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Text(ExcelText::from_interop_assignment("a")),
                    ArrayCellValue::Text(ExcelText::from_interop_assignment("b")),
                    ArrayCellValue::Text(ExcelText::from_interop_assignment("c")),
                ]])
                .unwrap(),
            ))
        );
    }
}

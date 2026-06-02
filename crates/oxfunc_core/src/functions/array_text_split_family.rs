use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    PreparedValue, coerce_prepared_to_number, coerce_prepared_to_text, prepare_args_values_only,
};
use crate::resolver::ReferenceSystemProvider;
use crate::value::{
    ArrayShape, ExcelText, FunctionArg, FunctionArray, FunctionArrayCell, FunctionValue,
    WorksheetErrorCode,
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

fn scalar_cell_from_prepared(arg: &PreparedValue) -> FunctionArrayCell {
    match arg {
        PreparedValue::Eval(FunctionValue::Number(n)) => FunctionArrayCell::Number(*n),
        PreparedValue::Eval(FunctionValue::Text(t)) => FunctionArrayCell::Text(t.clone()),
        PreparedValue::Eval(FunctionValue::Logical(b)) => FunctionArrayCell::Logical(*b),
        PreparedValue::Eval(FunctionValue::Error(code)) => FunctionArrayCell::Error(*code),
        PreparedValue::Eval(FunctionValue::Reference(_)) => {
            FunctionArrayCell::Error(WorksheetErrorCode::Value)
        }
        PreparedValue::Eval(FunctionValue::Array(_)) => unreachable!(),
        PreparedValue::MissingArg | PreparedValue::EmptyCell => FunctionArrayCell::EmptyCell,
        _ => FunctionArrayCell::Error(WorksheetErrorCode::Value),
    }
}

fn eval_value_to_array_cell(
    value: &FunctionValue,
) -> Result<FunctionArrayCell, ArrayTextSplitEvalError> {
    match value {
        FunctionValue::Number(n) => Ok(FunctionArrayCell::Number(*n)),
        FunctionValue::Text(t) => Ok(FunctionArrayCell::Text(t.clone())),
        FunctionValue::Logical(b) => Ok(FunctionArrayCell::Logical(*b)),
        FunctionValue::Error(code) => Ok(FunctionArrayCell::Error(*code)),
        FunctionValue::Array(_) => Err(ArrayTextSplitEvalError::UnsupportedPadWith("array")),
        FunctionValue::Reference(_) => Err(ArrayTextSplitEvalError::UnsupportedPadWith(
            "reference_like",
        )),
        _ => Err(ArrayTextSplitEvalError::UnsupportedPadWith(
            "unsupported_value",
        )),
    }
}

fn materialize_arraytotext_input(prepared: &PreparedValue) -> FunctionArray {
    match prepared {
        PreparedValue::Eval(FunctionValue::Array(array)) => array.clone(),
        other => FunctionArray::from_scalar(scalar_cell_from_prepared(other)),
    }
}

fn parse_truncated_flag(
    prepared: &PreparedValue,
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
    prepared: Option<&PreparedValue>,
) -> Result<bool, ArrayTextSplitEvalError> {
    match prepared {
        None | Some(PreparedValue::MissingArg) | Some(PreparedValue::EmptyCell) => Ok(false),
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

fn array_cell_to_concise_fragment(cell: &FunctionArrayCell) -> String {
    match cell {
        FunctionArrayCell::Number(n) => format!("{n}"),
        FunctionArrayCell::Text(t) => t.to_string_lossy(),
        FunctionArrayCell::Logical(b) => {
            if *b {
                "TRUE".to_string()
            } else {
                "FALSE".to_string()
            }
        }
        FunctionArrayCell::Error(code) => worksheet_error_literal(*code).to_string(),
        FunctionArrayCell::EmptyCell => String::new(),
    }
}

fn escape_strict_text(text: &ExcelText) -> String {
    text.to_string_lossy().replace('"', "\"\"")
}

fn array_cell_to_strict_fragment(cell: &FunctionArrayCell) -> String {
    match cell {
        FunctionArrayCell::Number(n) => format!("{n}"),
        FunctionArrayCell::Text(t) => format!("\"{}\"", escape_strict_text(t)),
        FunctionArrayCell::Logical(b) => {
            if *b {
                "TRUE".to_string()
            } else {
                "FALSE".to_string()
            }
        }
        FunctionArrayCell::Error(code) => worksheet_error_literal(*code).to_string(),
        FunctionArrayCell::EmptyCell => String::new(),
    }
}

fn arraytotext_concise(array: &FunctionArray) -> ExcelText {
    let joined = array
        .iter_row_major()
        .map(array_cell_to_concise_fragment)
        .collect::<Vec<_>>()
        .join(", ");
    ExcelText::from_utf16_code_units(joined.encode_utf16().collect())
}

fn arraytotext_strict(array: &FunctionArray) -> ExcelText {
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
    args: &[FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<FunctionValue, ArrayTextSplitEvalError> {
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
    Ok(FunctionValue::Text(if strict {
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

fn prepared_from_array_cell(cell: &FunctionArrayCell) -> PreparedValue {
    match cell {
        FunctionArrayCell::Number(n) => PreparedValue::Eval(FunctionValue::Number(*n)),
        FunctionArrayCell::Text(t) => PreparedValue::Eval(FunctionValue::Text(t.clone())),
        FunctionArrayCell::Logical(b) => PreparedValue::Eval(FunctionValue::Logical(*b)),
        FunctionArrayCell::Error(code) => PreparedValue::Eval(FunctionValue::Error(*code)),
        FunctionArrayCell::EmptyCell => PreparedValue::EmptyCell,
    }
}

fn delimiter_list_from_prepared(
    prepared: &PreparedValue,
) -> Result<Option<Vec<ExcelText>>, ArrayTextSplitEvalError> {
    match prepared {
        PreparedValue::MissingArg => Ok(None),
        PreparedValue::EmptyCell => Ok(Some(vec![empty_text()])),
        PreparedValue::Eval(FunctionValue::Array(array)) => {
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
    prepared: Option<&PreparedValue>,
) -> Result<FunctionArrayCell, ArrayTextSplitEvalError> {
    match prepared {
        None | Some(PreparedValue::MissingArg) => {
            Ok(FunctionArrayCell::Error(WorksheetErrorCode::NA))
        }
        Some(PreparedValue::EmptyCell) => Ok(FunctionArrayCell::Number(0.0)),
        Some(PreparedValue::Eval(value)) => eval_value_to_array_cell(value),
    }
}

pub fn eval_textsplit_surface(
    args: &[FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<FunctionValue, ArrayTextSplitEvalError> {
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

    let mut rows: Vec<Vec<FunctionArrayCell>> = Vec::with_capacity(row_parts.len());
    for row_text in row_parts {
        let mut cols = if let Some(delimiters) = col_delimiters.as_ref() {
            split_text_by_delimiters(&row_text, delimiters, ignore_empty, case_insensitive)
        } else {
            vec![row_text]
        };
        if cols.is_empty() {
            cols.push(empty_text());
        }
        rows.push(cols.into_iter().map(FunctionArrayCell::Text).collect());
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

    Ok(FunctionValue::Array(
        FunctionArray::new(
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
    use crate::resolver::ReferenceSystemCapabilities;

    struct NoResolver;

    impl ReferenceSystemProvider for NoResolver {
        fn capabilities(&self) -> ReferenceSystemCapabilities {
            ReferenceSystemCapabilities::permissive_local()
        }

        fn dereference(
            &self,
            request: &crate::resolver::ReferenceDereferenceRequest,
        ) -> Result<FunctionValue, crate::resolver::ReferenceResolutionError> {
            let reference = &request.reference;
            Err(
                crate::resolver::ReferenceResolutionError::UnresolvedReference {
                    target: reference.target().to_string(),
                },
            )
        }
    }

    fn text_arg(s: &str) -> FunctionArg {
        FunctionArg::Eval(FunctionValue::Text(ExcelText::from_interop_assignment(s)))
    }

    fn number_arg(n: f64) -> FunctionArg {
        FunctionArg::Eval(FunctionValue::Number(n))
    }

    #[test]
    fn arraytotext_concise_formats_row_major_values() {
        let got = eval_arraytotext_surface(
            &[
                FunctionArg::Eval(FunctionValue::Array(
                    FunctionArray::from_rows(vec![
                        vec![
                            FunctionArrayCell::Logical(true),
                            FunctionArrayCell::Error(WorksheetErrorCode::Value),
                        ],
                        vec![
                            FunctionArrayCell::Text(ExcelText::from_interop_assignment("Hello")),
                            FunctionArrayCell::Number(2.0),
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
            Ok(FunctionValue::Text(ExcelText::from_interop_assignment(
                "TRUE, #VALUE!, Hello, 2"
            )))
        );
    }

    #[test]
    fn arraytotext_strict_quotes_text_and_preserves_shape_markers() {
        let got = eval_arraytotext_surface(
            &[
                FunctionArg::Eval(FunctionValue::Array(
                    FunctionArray::from_rows(vec![
                        vec![
                            FunctionArrayCell::Logical(true),
                            FunctionArrayCell::Error(WorksheetErrorCode::Value),
                        ],
                        vec![
                            FunctionArrayCell::Text(ExcelText::from_interop_assignment("Hello")),
                            FunctionArrayCell::Number(2.0),
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
            Ok(FunctionValue::Text(ExcelText::from_interop_assignment(
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
            Ok(FunctionValue::Array(
                FunctionArray::from_rows(vec![vec![
                    FunctionArrayCell::Text(ExcelText::from_interop_assignment("Dakota")),
                    FunctionArrayCell::Text(ExcelText::from_interop_assignment("Lennon")),
                    FunctionArrayCell::Text(ExcelText::from_interop_assignment("Sanchez")),
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
            Ok(FunctionValue::Array(
                FunctionArray::from_rows(vec![
                    vec![
                        FunctionArrayCell::Text(ExcelText::from_interop_assignment("1")),
                        FunctionArrayCell::Text(ExcelText::from_interop_assignment("2")),
                        FunctionArrayCell::Text(ExcelText::from_interop_assignment("3")),
                    ],
                    vec![
                        FunctionArrayCell::Text(ExcelText::from_interop_assignment("4")),
                        FunctionArrayCell::Text(ExcelText::from_interop_assignment("5")),
                        FunctionArrayCell::Error(WorksheetErrorCode::NA),
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
                FunctionArg::Eval(FunctionValue::Array(
                    FunctionArray::from_rows(vec![vec![
                        FunctionArrayCell::Text(ExcelText::from_interop_assignment(".")),
                        FunctionArrayCell::Text(ExcelText::from_interop_assignment("-")),
                    ]])
                    .unwrap(),
                )),
                FunctionArg::MissingArg,
                number_arg(1.0),
            ],
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(FunctionValue::Array(
                FunctionArray::from_rows(vec![vec![
                    FunctionArrayCell::Text(ExcelText::from_interop_assignment("Do")),
                    FunctionArrayCell::Text(ExcelText::from_interop_assignment(" Or do not")),
                    FunctionArrayCell::Text(ExcelText::from_interop_assignment(" There is no try")),
                    FunctionArrayCell::Text(ExcelText::from_interop_assignment(" ")),
                    FunctionArrayCell::Text(ExcelText::from_interop_assignment("Anonymous")),
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
                FunctionArg::MissingArg,
                number_arg(0.0),
                number_arg(1.0),
                text_arg("pad"),
            ],
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(FunctionValue::Array(
                FunctionArray::from_rows(vec![vec![
                    FunctionArrayCell::Text(ExcelText::from_interop_assignment("a")),
                    FunctionArrayCell::Text(ExcelText::from_interop_assignment("b")),
                    FunctionArrayCell::Text(ExcelText::from_interop_assignment("c")),
                ]])
                .unwrap(),
            ))
        );
    }
}

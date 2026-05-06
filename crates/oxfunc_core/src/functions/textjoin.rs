use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    PreparedArgValue, coerce_prepared_to_number, coerce_prepared_to_text, expand_arg_values_only,
    prepare_arg_values_only,
};
use crate::resolver::ReferenceResolver;
use crate::value::{
    CallArgValue, EXCEL_TEXT_MAX_UTF16_CODE_UNITS, EvalValue, ExcelText, WorksheetErrorCode,
};

pub const TEXTJOIN_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.TEXTJOIN",
    arity: Arity { min: 3, max: 255 },
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

#[derive(Debug, Clone, PartialEq)]
pub enum TextJoinEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
    ResultTooLong {
        actual_utf16_len: usize,
    },
}

fn parse_ignore_empty(arg: &PreparedArgValue) -> Result<bool, TextJoinEvalError> {
    let n = coerce_prepared_to_number(arg).map_err(TextJoinEvalError::Coercion)?;
    Ok(n != 0.0)
}

pub fn eval_textjoin_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, TextJoinEvalError> {
    let argc = args.len();
    if !TEXTJOIN_META.arity.accepts(argc) {
        return Err(TextJoinEvalError::ArityMismatch {
            expected_min: TEXTJOIN_META.arity.min,
            expected_max: TEXTJOIN_META.arity.max,
            actual: argc,
        });
    }

    let delimiter =
        prepare_arg_values_only(&args[0], resolver).map_err(TextJoinEvalError::Coercion)?;
    let ignore_empty_arg =
        prepare_arg_values_only(&args[1], resolver).map_err(TextJoinEvalError::Coercion)?;
    let delimiter = coerce_prepared_to_text(&delimiter).map_err(TextJoinEvalError::Coercion)?;
    let ignore_empty = parse_ignore_empty(&ignore_empty_arg)?;

    let delimiter_utf16_len = delimiter.utf16_code_units().len();
    let mut parts: Vec<ExcelText> = Vec::new();
    let mut total_utf16_len = 0usize;
    for arg in &args[2..] {
        for prepared in
            expand_arg_values_only(arg, resolver).map_err(TextJoinEvalError::Coercion)?
        {
            match prepared {
                PreparedArgValue::MissingArg | PreparedArgValue::EmptyCell if ignore_empty => {}
                ref other => {
                    let text =
                        coerce_prepared_to_text(other).map_err(TextJoinEvalError::Coercion)?;
                    if ignore_empty && text.utf16_code_units().is_empty() {
                        continue;
                    }
                    let next_total = total_utf16_len
                        .saturating_add(text.utf16_code_units().len())
                        .saturating_add(if parts.is_empty() {
                            0
                        } else {
                            delimiter_utf16_len
                        });
                    if next_total > EXCEL_TEXT_MAX_UTF16_CODE_UNITS {
                        return Err(TextJoinEvalError::ResultTooLong {
                            actual_utf16_len: next_total,
                        });
                    }
                    total_utf16_len = next_total;
                    parts.push(text);
                }
            }
        }
    }

    let mut out = Vec::with_capacity(total_utf16_len);
    for (idx, part) in parts.iter().enumerate() {
        if idx > 0 {
            out.extend_from_slice(delimiter.utf16_code_units());
        }
        out.extend_from_slice(part.utf16_code_units());
    }

    Ok(EvalValue::Text(ExcelText::from_utf16_code_units(out)))
}

pub fn map_textjoin_error_to_ws(e: &TextJoinEvalError) -> WorksheetErrorCode {
    match e {
        TextJoinEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        TextJoinEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        TextJoinEvalError::ResultTooLong { .. } => WorksheetErrorCode::Calc,
        TextJoinEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::{RefResolutionError, ResolverCapabilities};
    use crate::value::{ArrayCellValue, EvalArray, ReferenceLike};

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
    fn eval_textjoin_joins_text_and_numbers() {
        let got = eval_textjoin_surface(
            &[
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
                    ",".encode_utf16().collect(),
                ))),
                CallArgValue::Eval(EvalValue::Logical(true)),
                CallArgValue::Eval(EvalValue::Number(1.0)),
                CallArgValue::Eval(EvalValue::Number(2.0)),
            ],
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Text(ExcelText::from_utf16_code_units(
                "1,2".encode_utf16().collect(),
            )))
        );
    }

    #[test]
    fn eval_textjoin_flattens_arrays_row_major() {
        let got = eval_textjoin_surface(
            &[
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
                    "|".encode_utf16().collect(),
                ))),
                CallArgValue::Eval(EvalValue::Number(0.0)),
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![
                        vec![ArrayCellValue::Number(1.0), ArrayCellValue::Number(2.0)],
                        vec![
                            ArrayCellValue::Text(ExcelText::from_utf16_code_units(
                                "x".encode_utf16().collect(),
                            )),
                            ArrayCellValue::EmptyCell,
                        ],
                    ])
                    .unwrap(),
                )),
            ],
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Text(ExcelText::from_utf16_code_units(
                "1|2|x|".encode_utf16().collect(),
            )))
        );
    }

    #[test]
    fn eval_textjoin_skips_empty_values_when_requested() {
        let got = eval_textjoin_surface(
            &[
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
                    "|".encode_utf16().collect(),
                ))),
                CallArgValue::Eval(EvalValue::Number(1.0)),
                CallArgValue::EmptyCell,
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
                    "x".encode_utf16().collect(),
                ))),
            ],
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Text(ExcelText::from_utf16_code_units(
                "x".encode_utf16().collect(),
            )))
        );
    }

    #[test]
    fn eval_textjoin_numeric_and_logical_delimiters_are_textified() {
        let numeric = eval_textjoin_surface(
            &[
                CallArgValue::Eval(EvalValue::Number(1.0)),
                CallArgValue::Eval(EvalValue::Logical(false)),
                CallArgValue::Eval(EvalValue::Number(2.0)),
                CallArgValue::Eval(EvalValue::Number(3.0)),
            ],
            &NoResolver,
        );
        assert_eq!(
            numeric,
            Ok(EvalValue::Text(ExcelText::from_utf16_code_units(
                "213".encode_utf16().collect(),
            )))
        );

        let logical = eval_textjoin_surface(
            &[
                CallArgValue::Eval(EvalValue::Logical(true)),
                CallArgValue::Eval(EvalValue::Logical(false)),
                CallArgValue::Eval(EvalValue::Number(1.0)),
                CallArgValue::Eval(EvalValue::Number(2.0)),
            ],
            &NoResolver,
        );
        assert_eq!(
            logical,
            Ok(EvalValue::Text(ExcelText::from_utf16_code_units(
                "1TRUE2".encode_utf16().collect(),
            )))
        );
    }

    #[test]
    fn eval_textjoin_enforces_excel_text_length_limit() {
        let base = "x".repeat(20_000);
        let exact = "x".repeat(12_767);
        let overflow = "x".repeat(12_768);

        let exact_limit = eval_textjoin_surface(
            &[
                CallArgValue::Eval(EvalValue::Text(
                    ExcelText::from_utf16_code_units(Vec::new()),
                )),
                CallArgValue::Eval(EvalValue::Logical(false)),
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
                    base.encode_utf16().collect(),
                ))),
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
                    exact.encode_utf16().collect(),
                ))),
            ],
            &NoResolver,
        );
        assert_eq!(
            exact_limit,
            Ok(EvalValue::Text(ExcelText::from_utf16_code_units(
                "x".repeat(EXCEL_TEXT_MAX_UTF16_CODE_UNITS)
                    .encode_utf16()
                    .collect(),
            )))
        );

        let too_long = eval_textjoin_surface(
            &[
                CallArgValue::Eval(EvalValue::Text(
                    ExcelText::from_utf16_code_units(Vec::new()),
                )),
                CallArgValue::Eval(EvalValue::Logical(false)),
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
                    base.encode_utf16().collect(),
                ))),
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
                    overflow.encode_utf16().collect(),
                ))),
            ],
            &NoResolver,
        );
        assert_eq!(
            too_long,
            Err(TextJoinEvalError::ResultTooLong {
                actual_utf16_len: EXCEL_TEXT_MAX_UTF16_CODE_UNITS + 1,
            })
        );
        assert_eq!(
            map_textjoin_error_to_ws(&too_long.unwrap_err()),
            WorksheetErrorCode::Calc
        );
    }
}

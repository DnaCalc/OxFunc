use crate::coercion::CoercionError;
use crate::function::{
    Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile, FunctionMeta,
    HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    coerce_prepared_to_number, coerce_prepared_to_text, expand_arg_values_only,
    prepare_arg_values_only,
};
use crate::resolver::ReferenceSystemProvider;
use crate::value::{
    CalcValue, CoreValue, EXCEL_TEXT_MAX_UTF16_CODE_UNITS, ExcelText, WorksheetErrorCode,
};

pub const TEXTJOIN_META: FunctionMeta = function_spec! {
    function_id: "FUNC.TEXTJOIN",
    arity: Arity { min: 3, max: 255 },
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
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

fn parse_ignore_empty(arg: &CalcValue) -> Result<bool, TextJoinEvalError> {
    let n = coerce_prepared_to_number(arg).map_err(TextJoinEvalError::Coercion)?;
    Ok(n != 0.0)
}

pub fn eval_textjoin_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, TextJoinEvalError> {
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
            match prepared.core() {
                CoreValue::Missing | CoreValue::Empty if ignore_empty => {}
                _ => {
                    let text =
                        coerce_prepared_to_text(&prepared).map_err(TextJoinEvalError::Coercion)?;
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

    Ok(CalcValue::text(ExcelText::from_utf16_code_units(out)))
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

    #[test]
    fn eval_textjoin_joins_text_and_numbers() {
        let got = eval_textjoin_surface(
            &[
                (CalcValue::text(ExcelText::from_utf16_code_units(
                    ",".encode_utf16().collect(),
                ))),
                (CalcValue::logical(true)),
                (CalcValue::number(1.0)),
                (CalcValue::number(2.0)),
            ],
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(CalcValue::text(ExcelText::from_utf16_code_units(
                "1,2".encode_utf16().collect(),
            )))
        );
    }

    #[test]
    fn eval_textjoin_flattens_arrays_row_major() {
        let got = eval_textjoin_surface(
            &[
                (CalcValue::text(ExcelText::from_utf16_code_units(
                    "|".encode_utf16().collect(),
                ))),
                (CalcValue::number(0.0)),
                (CalcValue::array(
                    CalcArray::from_rows(vec![
                        vec![CalcValue::number(1.0), CalcValue::number(2.0)],
                        vec![
                            CalcValue::text(ExcelText::from_utf16_code_units(
                                "x".encode_utf16().collect(),
                            )),
                            CalcValue::empty(),
                        ],
                    ])
                    .unwrap(),
                )),
            ],
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(CalcValue::text(ExcelText::from_utf16_code_units(
                "1|2|x|".encode_utf16().collect(),
            )))
        );
    }

    #[test]
    fn eval_textjoin_skips_empty_values_when_requested() {
        let got = eval_textjoin_surface(
            &[
                (CalcValue::text(ExcelText::from_utf16_code_units(
                    "|".encode_utf16().collect(),
                ))),
                (CalcValue::number(1.0)),
                CalcValue::empty(),
                (CalcValue::text(ExcelText::from_utf16_code_units(
                    "x".encode_utf16().collect(),
                ))),
            ],
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(CalcValue::text(ExcelText::from_utf16_code_units(
                "x".encode_utf16().collect(),
            )))
        );
    }

    #[test]
    fn eval_textjoin_numeric_and_logical_delimiters_are_textified() {
        let numeric = eval_textjoin_surface(
            &[
                (CalcValue::number(1.0)),
                (CalcValue::logical(false)),
                (CalcValue::number(2.0)),
                (CalcValue::number(3.0)),
            ],
            &NoResolver,
        );
        assert_eq!(
            numeric,
            Ok(CalcValue::text(ExcelText::from_utf16_code_units(
                "213".encode_utf16().collect(),
            )))
        );

        let logical = eval_textjoin_surface(
            &[
                (CalcValue::logical(true)),
                (CalcValue::logical(false)),
                (CalcValue::number(1.0)),
                (CalcValue::number(2.0)),
            ],
            &NoResolver,
        );
        assert_eq!(
            logical,
            Ok(CalcValue::text(ExcelText::from_utf16_code_units(
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
                (CalcValue::text(ExcelText::from_utf16_code_units(Vec::new()))),
                (CalcValue::logical(false)),
                (CalcValue::text(ExcelText::from_utf16_code_units(
                    base.encode_utf16().collect(),
                ))),
                (CalcValue::text(ExcelText::from_utf16_code_units(
                    exact.encode_utf16().collect(),
                ))),
            ],
            &NoResolver,
        );
        assert_eq!(
            exact_limit,
            Ok(CalcValue::text(ExcelText::from_utf16_code_units(
                "x".repeat(EXCEL_TEXT_MAX_UTF16_CODE_UNITS)
                    .encode_utf16()
                    .collect(),
            )))
        );

        let too_long = eval_textjoin_surface(
            &[
                (CalcValue::text(ExcelText::from_utf16_code_units(Vec::new()))),
                (CalcValue::logical(false)),
                (CalcValue::text(ExcelText::from_utf16_code_units(
                    base.encode_utf16().collect(),
                ))),
                (CalcValue::text(ExcelText::from_utf16_code_units(
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

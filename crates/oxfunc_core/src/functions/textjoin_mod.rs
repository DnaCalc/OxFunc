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
use crate::value::{CallArgValue, EvalValue, ExcelText, WorksheetErrorCode};

pub const TEXTJOIN_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.TEXTJOIN",
    arity: Arity { min: 3, max: 255 },
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::RefsVisibleInAdapter,
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
}

fn parse_ignore_empty(arg: &PreparedArgValue) -> Result<bool, TextJoinEvalError> {
    let n = coerce_prepared_to_number(arg).map_err(TextJoinEvalError::Coercion)?;
    Ok(n != 0.0)
}

pub fn eval_textjoin_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, TextJoinEvalError> {
    let argc = args.len();
    if !TEXTJOIN_META.arity.accepts(argc) {
        return Err(TextJoinEvalError::ArityMismatch {
            expected_min: TEXTJOIN_META.arity.min,
            expected_max: TEXTJOIN_META.arity.max,
            actual: argc,
        });
    }

    let delimiter = prepare_arg_values_only(&args[0], resolver).map_err(TextJoinEvalError::Coercion)?;
    let ignore_empty_arg =
        prepare_arg_values_only(&args[1], resolver).map_err(TextJoinEvalError::Coercion)?;
    let delimiter_str = coerce_prepared_to_text(&delimiter)
        .map_err(TextJoinEvalError::Coercion)?
        .to_string_lossy();
    let ignore_empty = parse_ignore_empty(&ignore_empty_arg)?;

    let mut parts = Vec::new();
    for arg in &args[2..] {
        for prepared in expand_arg_values_only(arg, resolver).map_err(TextJoinEvalError::Coercion)? {
            match prepared {
                PreparedArgValue::MissingArg | PreparedArgValue::EmptyCell if ignore_empty => {}
                ref other => {
                    let text = coerce_prepared_to_text(other)
                        .map_err(TextJoinEvalError::Coercion)?
                        .to_string_lossy();
                    if ignore_empty && text.is_empty() {
                        continue;
                    }
                    parts.push(text);
                }
            }
        }
    }

    Ok(EvalValue::Text(ExcelText::from_utf16_code_units(
        parts.join(&delimiter_str).encode_utf16().collect(),
    )))
}

pub fn map_textjoin_error_to_ws(e: &TextJoinEvalError) -> WorksheetErrorCode {
    match e {
        TextJoinEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        TextJoinEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
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
                        vec![ArrayCellValue::Text(ExcelText::from_utf16_code_units(
                            "x".encode_utf16().collect(),
                        )), ArrayCellValue::EmptyCell],
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
}

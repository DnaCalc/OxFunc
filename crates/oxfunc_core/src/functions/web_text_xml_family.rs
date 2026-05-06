use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{coerce_prepared_to_text, prepare_args_values_only};
use crate::resolver::ReferenceResolver;
use crate::value::{ArrayCellValue, EvalArray, EvalValue, ExcelText, WorksheetErrorCode};
use sxd_document::parser;
use sxd_xpath::{Context, Factory, Value};

pub const ENCODEURL_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.ENCODEURL",
    arity: Arity::exact(1),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::TextToText,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::None,
};

pub const FILTERXML_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.FILTERXML",
    arity: Arity::exact(2),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::None,
};

#[derive(Debug, Clone, PartialEq)]
pub enum WebTextXmlEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
    XmlParse,
    XPathParse,
    XPathEval,
    UnsupportedXPathResult,
    EmptyNodeSet,
}

fn arity_error(meta: &FunctionMeta, actual: usize) -> WebTextXmlEvalError {
    WebTextXmlEvalError::ArityMismatch {
        expected_min: meta.arity.min,
        expected_max: meta.arity.max,
        actual,
    }
}

fn parse_excel_number(text: &str) -> Option<f64> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }
    let parsed = trimmed.parse::<f64>().ok()?;
    parsed.is_finite().then_some(parsed)
}

fn parse_logical_text(text: &str) -> Option<bool> {
    if text.eq_ignore_ascii_case("TRUE") {
        Some(true)
    } else if text.eq_ignore_ascii_case("FALSE") {
        Some(false)
    } else {
        None
    }
}

fn scalar_from_xpath_string(text: &str) -> ArrayCellValue {
    if let Some(number) = parse_excel_number(text) {
        ArrayCellValue::Number(number)
    } else if let Some(logical) = parse_logical_text(text) {
        ArrayCellValue::Logical(logical)
    } else {
        ArrayCellValue::Text(ExcelText::from_interop_assignment(text))
    }
}

fn is_unreserved_url_byte(byte: u8) -> bool {
    matches!(byte,
        b'A'..=b'Z' |
        b'a'..=b'z' |
        b'0'..=b'9' |
        b'-' | b'_' | b'.' | b'~'
    )
}

pub fn encodeurl_kernel(text: &str) -> String {
    let mut encoded = String::with_capacity(text.len());
    for byte in text.as_bytes() {
        if is_unreserved_url_byte(*byte) {
            encoded.push(char::from(*byte));
        } else {
            encoded.push('%');
            encoded.push_str(&format!("{byte:02X}"));
        }
    }
    encoded
}

pub fn eval_encodeurl_surface(
    args: &[crate::value::CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, WebTextXmlEvalError> {
    if !ENCODEURL_META.arity.accepts(args.len()) {
        return Err(arity_error(&ENCODEURL_META, args.len()));
    }
    let prepared =
        prepare_args_values_only(args, resolver).map_err(WebTextXmlEvalError::Coercion)?;
    let text = coerce_prepared_to_text(&prepared[0]).map_err(WebTextXmlEvalError::Coercion)?;
    Ok(EvalValue::Text(ExcelText::from_interop_assignment(
        &encodeurl_kernel(&text.to_string_lossy()),
    )))
}

pub fn eval_filterxml_surface(
    args: &[crate::value::CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, WebTextXmlEvalError> {
    if !FILTERXML_META.arity.accepts(args.len()) {
        return Err(arity_error(&FILTERXML_META, args.len()));
    }
    let prepared =
        prepare_args_values_only(args, resolver).map_err(WebTextXmlEvalError::Coercion)?;
    let xml = coerce_prepared_to_text(&prepared[0]).map_err(WebTextXmlEvalError::Coercion)?;
    let xpath = coerce_prepared_to_text(&prepared[1]).map_err(WebTextXmlEvalError::Coercion)?;
    let package =
        parser::parse(&xml.to_string_lossy()).map_err(|_| WebTextXmlEvalError::XmlParse)?;
    let document = package.as_document();
    let factory = Factory::new();
    let xpath = factory
        .build(&xpath.to_string_lossy())
        .map_err(|_| WebTextXmlEvalError::XPathParse)?
        .ok_or(WebTextXmlEvalError::XPathParse)?;
    let value = xpath
        .evaluate(&Context::new(), document.root())
        .map_err(|_| WebTextXmlEvalError::XPathEval)?;

    match value {
        Value::Nodeset(nodeset) => {
            let nodes = nodeset.document_order();
            if nodes.is_empty() {
                return Err(WebTextXmlEvalError::EmptyNodeSet);
            }
            let cells = nodes
                .into_iter()
                .map(|node| scalar_from_xpath_string(&node.string_value()))
                .collect::<Vec<_>>();
            if cells.len() == 1 {
                Ok(cells[0]
                    .to_eval_value()
                    .unwrap_or_else(|| EvalValue::Text(ExcelText::from_interop_assignment(""))))
            } else {
                Ok(EvalValue::Array(
                    EvalArray::from_rows(cells.into_iter().map(|cell| vec![cell]).collect())
                        .expect("vertical FILTERXML result array"),
                ))
            }
        }
        _ => Err(WebTextXmlEvalError::UnsupportedXPathResult),
    }
}

pub fn map_web_text_xml_error_to_ws(error: &WebTextXmlEvalError) -> WorksheetErrorCode {
    match error {
        WebTextXmlEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        WebTextXmlEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        WebTextXmlEvalError::Coercion(_) => WorksheetErrorCode::Value,
        WebTextXmlEvalError::XmlParse
        | WebTextXmlEvalError::XPathParse
        | WebTextXmlEvalError::XPathEval
        | WebTextXmlEvalError::UnsupportedXPathResult
        | WebTextXmlEvalError::EmptyNodeSet => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::{RefResolutionError, ResolverCapabilities};
    use crate::value::{CallArgValue, ReferenceLike};

    struct MockResolver;

    impl ReferenceResolver for MockResolver {
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

    fn text_arg(text: &str) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Text(ExcelText::from_interop_assignment(text)))
    }

    #[test]
    fn encodeurl_kernel_matches_seeded_cases() {
        assert_eq!(encodeurl_kernel("a b+c"), "a%20b%2Bc");
        assert_eq!(encodeurl_kernel("123"), "123");
        assert_eq!(encodeurl_kernel("TRUE"), "TRUE");
        assert_eq!(encodeurl_kernel(""), "");
        assert_eq!(encodeurl_kernel("café"), "caf%C3%A9");
    }

    #[test]
    fn eval_encodeurl_surface_coerces_scalar_inputs_to_text() {
        assert_eq!(
            eval_encodeurl_surface(
                &[CallArgValue::Eval(EvalValue::Number(123.0))],
                &MockResolver
            ),
            Ok(EvalValue::Text(ExcelText::from_interop_assignment("123")))
        );
        assert_eq!(
            eval_encodeurl_surface(
                &[CallArgValue::Eval(EvalValue::Logical(true))],
                &MockResolver
            ),
            Ok(EvalValue::Text(ExcelText::from_interop_assignment("TRUE")))
        );
    }

    #[test]
    fn filterxml_surface_matches_seeded_nodeset_slice() {
        assert_eq!(
            eval_filterxml_surface(
                &[text_arg("<root><a>1</a><a>2</a></root>"), text_arg("//a")],
                &MockResolver
            ),
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Number(1.0)],
                    vec![ArrayCellValue::Number(2.0)],
                ])
                .unwrap()
            ))
        );
        assert_eq!(
            eval_filterxml_surface(
                &[text_arg("<root><a>hello</a></root>"), text_arg("//a")],
                &MockResolver
            ),
            Ok(EvalValue::Text(ExcelText::from_interop_assignment("hello")))
        );
    }

    #[test]
    fn filterxml_surface_rejects_non_nodeset_xpath_and_missing_results() {
        assert_eq!(
            eval_filterxml_surface(
                &[
                    text_arg("<root><a attr=\"x\"/></root>"),
                    text_arg("string(//a/@attr)")
                ],
                &MockResolver
            ),
            Err(WebTextXmlEvalError::UnsupportedXPathResult)
        );
        assert_eq!(
            eval_filterxml_surface(
                &[text_arg("<root><a>1</a></root>"), text_arg("//b")],
                &MockResolver
            ),
            Err(WebTextXmlEvalError::EmptyNodeSet)
        );
        assert_eq!(
            map_web_text_xml_error_to_ws(&WebTextXmlEvalError::UnsupportedXPathResult),
            WorksheetErrorCode::Value
        );
    }
}

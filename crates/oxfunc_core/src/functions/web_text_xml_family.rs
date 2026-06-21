use crate::coercion::CoercionError;
use crate::function::{
    Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile, FunctionMeta,
    HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{coerce_prepared_to_text, prepare_args_values_only};
use crate::resolver::ReferenceSystemProvider;
use crate::value::CalcValue;
use crate::value::{CalcArray, ExcelText, WorksheetErrorCode};
use sxd_document::parser;
use sxd_xpath::{Context, Factory, Value};

pub const ENCODEURL_META: FunctionMeta = function_spec! {
    function_id: "FUNC.ENCODEURL",
    arity: Arity::exact(1),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::TextToText,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::None,
};

pub const FILTERXML_META: FunctionMeta = function_spec! {
    function_id: "FUNC.FILTERXML",
    arity: Arity::exact(2),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
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

fn scalar_from_xpath_string(text: &str) -> CalcValue {
    if let Some(number) = parse_excel_number(text) {
        CalcValue::number(number)
    } else if let Some(logical) = parse_logical_text(text) {
        CalcValue::logical(logical)
    } else {
        CalcValue::text(ExcelText::from_interop_assignment(text))
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
    args: &[crate::value::CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, WebTextXmlEvalError> {
    if !ENCODEURL_META.arity.accepts(args.len()) {
        return Err(arity_error(&ENCODEURL_META, args.len()));
    }
    let prepared =
        prepare_args_values_only(args, resolver).map_err(WebTextXmlEvalError::Coercion)?;
    let text = coerce_prepared_to_text(&prepared[0]).map_err(WebTextXmlEvalError::Coercion)?;
    Ok(CalcValue::text(ExcelText::from_interop_assignment(
        &encodeurl_kernel(&text.to_string_lossy()),
    )))
}

pub fn eval_filterxml_surface(
    args: &[crate::value::CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, WebTextXmlEvalError> {
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
                Ok(cells[0].clone())
            } else {
                Ok(CalcValue::array(
                    CalcArray::from_rows(cells.into_iter().map(|cell| vec![cell]).collect())
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
    use crate::resolver::ReferenceSystemCapabilities;
    use crate::value::CalcValue;

    struct MockResolver;

    impl ReferenceSystemProvider for MockResolver {
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

    fn text_arg(text: &str) -> CalcValue {
        CalcValue::text(ExcelText::from_interop_assignment(text))
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
            eval_encodeurl_surface(&[(CalcValue::number(123.0))], &MockResolver),
            Ok(CalcValue::text(ExcelText::from_interop_assignment("123")))
        );
        assert_eq!(
            eval_encodeurl_surface(&[(CalcValue::logical(true))], &MockResolver),
            Ok(CalcValue::text(ExcelText::from_interop_assignment("TRUE")))
        );
    }

    #[test]
    fn filterxml_surface_matches_seeded_nodeset_slice() {
        assert_eq!(
            eval_filterxml_surface(
                &[text_arg("<root><a>1</a><a>2</a></root>"), text_arg("//a")],
                &MockResolver
            ),
            Ok(CalcValue::array(
                CalcArray::from_rows(vec![
                    vec![CalcValue::number(1.0)],
                    vec![CalcValue::number(2.0)],
                ])
                .unwrap()
            ))
        );
        assert_eq!(
            eval_filterxml_surface(
                &[text_arg("<root><a>hello</a></root>"), text_arg("//a")],
                &MockResolver
            ),
            Ok(CalcValue::text(ExcelText::from_interop_assignment("hello")))
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

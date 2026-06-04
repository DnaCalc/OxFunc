use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{coerce_prepared_to_text, prepare_args_values_only};
use crate::resolver::ReferenceSystemProvider;
use crate::value::CalcValue;
use crate::value::{ExcelText, WorksheetErrorCode};

pub const RTD_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.RTD",
    arity: Arity { min: 3, max: 255 },
    determinism: DeterminismClass::ExternalEventDependent,
    volatility: VolatilityClass::VolatileContextual,
    host_interaction: HostInteractionClass::ExternalProvider,
    thread_safety: ThreadSafetyClass::HostSerialized,
    arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::ExternalProvider,
    surface_fec_dependency_profile: FecDependencyProfile::ExternalProvider,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RtdRequest {
    pub prog_id: ExcelText,
    pub server_name: ExcelText,
    pub topic_strings: Vec<ExcelText>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RtdProviderResult {
    Value(CalcValue),
    NoValueYet,
    CapabilityDenied,
    ConnectionFailed,
    ProviderError(WorksheetErrorCode),
}

pub trait RtdProvider {
    fn resolve_rtd(&self, request: &RtdRequest) -> RtdProviderResult;
}

#[derive(Debug, Clone, PartialEq)]
pub enum RtdEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    TextCoercion(CoercionError),
    ProviderMissing,
}

pub fn parse_rtd_request(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<RtdRequest, RtdEvalError> {
    if !RTD_META.arity.accepts(args.len()) {
        return Err(RtdEvalError::ArityMismatch {
            expected_min: RTD_META.arity.min,
            expected_max: RTD_META.arity.max,
            actual: args.len(),
        });
    }

    let prepared = prepare_args_values_only(args, resolver).map_err(RtdEvalError::TextCoercion)?;
    let prog_id = coerce_prepared_to_text(&prepared[0]).map_err(RtdEvalError::TextCoercion)?;
    let server_name = coerce_prepared_to_text(&prepared[1]).map_err(RtdEvalError::TextCoercion)?;
    let topic_strings = prepared[2..]
        .iter()
        .map(coerce_prepared_to_text)
        .collect::<Result<Vec<_>, _>>()
        .map_err(RtdEvalError::TextCoercion)?;

    Ok(RtdRequest {
        prog_id,
        server_name,
        topic_strings,
    })
}

pub fn eval_rtd_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
    provider: Option<&dyn RtdProvider>,
) -> Result<CalcValue, RtdEvalError> {
    let request = parse_rtd_request(args, resolver)?;
    let provider = provider.ok_or(RtdEvalError::ProviderMissing)?;
    match provider.resolve_rtd(&request) {
        RtdProviderResult::Value(value) => Ok(value),
        RtdProviderResult::NoValueYet => Ok(CalcValue::error(WorksheetErrorCode::NA)),
        RtdProviderResult::CapabilityDenied => Ok(CalcValue::error(WorksheetErrorCode::Blocked)),
        RtdProviderResult::ConnectionFailed => Ok(CalcValue::error(WorksheetErrorCode::Connect)),
        RtdProviderResult::ProviderError(code) => Ok(CalcValue::error(code)),
    }
}

pub fn map_rtd_error_to_ws(error: &RtdEvalError) -> WorksheetErrorCode {
    match error {
        RtdEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        RtdEvalError::TextCoercion(CoercionError::WorksheetError(code)) => *code,
        RtdEvalError::TextCoercion(_) => WorksheetErrorCode::Value,
        RtdEvalError::ProviderMissing => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::ReferenceSystemCapabilities;
    use crate::value::{ArrayShape, CalcArray};

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

    struct RecordingProvider {
        expected: RtdProviderResult,
    }

    impl RtdProvider for RecordingProvider {
        fn resolve_rtd(&self, _request: &RtdRequest) -> RtdProviderResult {
            self.expected.clone()
        }
    }

    fn text_arg(text: &str) -> CalcValue {
        CalcValue::text(ExcelText::from_utf16_code_units(
            text.encode_utf16().collect(),
        ))
    }

    #[test]
    fn parse_rtd_request_preserves_progid_server_and_topics() {
        let request = parse_rtd_request(
            &[
                text_arg("My.Server"),
                text_arg(""),
                text_arg("StockQuote"),
                text_arg("MSFT"),
                text_arg("NASDAQ"),
            ],
            &MockResolver,
        )
        .expect("request");

        assert_eq!(request.prog_id.to_string_lossy(), "My.Server");
        assert_eq!(request.server_name.to_string_lossy(), "");
        assert_eq!(
            request
                .topic_strings
                .iter()
                .map(ExcelText::to_string_lossy)
                .collect::<Vec<_>>(),
            vec!["StockQuote", "MSFT", "NASDAQ"]
        );
    }

    #[test]
    fn parse_rtd_request_coerces_numbers_and_blanks_to_text_topics() {
        let request = parse_rtd_request(
            &[
                text_arg("TimerRTD.RtdServer"),
                CalcValue::empty(),
                text_arg("WAVE"),
                (CalcValue::number(2.5)),
                CalcValue::missing(),
            ],
            &MockResolver,
        )
        .expect("request");

        assert_eq!(request.server_name.to_string_lossy(), "");
        assert_eq!(
            request
                .topic_strings
                .iter()
                .map(ExcelText::to_string_lossy)
                .collect::<Vec<_>>(),
            vec!["WAVE", "2.5", ""]
        );
    }

    #[test]
    fn eval_rtd_surface_passes_through_value_payload() {
        let got = eval_rtd_surface(
            &[text_arg("My.Server"), text_arg(""), text_arg("TOPIC")],
            &MockResolver,
            Some(&RecordingProvider {
                expected: RtdProviderResult::Value(CalcValue::number(42.0)),
            }),
        );
        assert_eq!(got, Ok(CalcValue::number(42.0)));
    }

    #[test]
    fn eval_rtd_surface_maps_no_value_yet_to_na() {
        let got = eval_rtd_surface(
            &[text_arg("My.Server"), text_arg(""), text_arg("TOPIC")],
            &MockResolver,
            Some(&RecordingProvider {
                expected: RtdProviderResult::NoValueYet,
            }),
        );
        assert_eq!(got, Ok(CalcValue::error(WorksheetErrorCode::NA)));
    }

    #[test]
    fn eval_rtd_surface_maps_capability_and_connection_outcomes() {
        let blocked = eval_rtd_surface(
            &[text_arg("My.Server"), text_arg(""), text_arg("TOPIC")],
            &MockResolver,
            Some(&RecordingProvider {
                expected: RtdProviderResult::CapabilityDenied,
            }),
        );
        assert_eq!(blocked, Ok(CalcValue::error(WorksheetErrorCode::Blocked)));

        let connect = eval_rtd_surface(
            &[text_arg("My.Server"), text_arg(""), text_arg("TOPIC")],
            &MockResolver,
            Some(&RecordingProvider {
                expected: RtdProviderResult::ConnectionFailed,
            }),
        );
        assert_eq!(connect, Ok(CalcValue::error(WorksheetErrorCode::Connect)));
    }

    #[test]
    fn eval_rtd_surface_supports_array_payload_projection() {
        let array = CalcArray::new(
            ArrayShape { rows: 1, cols: 2 },
            vec![CalcValue::number(1.0), CalcValue::number(2.0)],
        )
        .expect("array");
        let got = eval_rtd_surface(
            &[text_arg("My.Server"), text_arg(""), text_arg("TOPIC")],
            &MockResolver,
            Some(&RecordingProvider {
                expected: RtdProviderResult::Value(CalcValue::array(array.clone())),
            }),
        );
        assert_eq!(got, Ok(CalcValue::array(array)));
    }

    #[test]
    fn eval_rtd_surface_requires_provider() {
        let got = eval_rtd_surface(
            &[text_arg("My.Server"), text_arg(""), text_arg("TOPIC")],
            &MockResolver,
            None,
        );
        assert_eq!(got, Err(RtdEvalError::ProviderMissing));
    }
}

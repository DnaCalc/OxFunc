use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{PreparedArgValue, prepare_args_values_only};
use crate::host_info::{
    HostInfoError, HostInfoProvider, ImageProviderResult, ImageRequest, ImageSizingMode,
    ResolvedWebImage,
};
use crate::resolver::ReferenceResolver;
use crate::value::{
    CallArgValue, EvalValue, ExcelText, ExtendedValue, RichValue, RichValueData, RichValueKeyFlag,
    RichValueKeyValue, RichValueType, WorksheetErrorCode,
};

pub const IMAGE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.IMAGE",
    arity: Arity { min: 1, max: 5 },
    determinism: DeterminismClass::ExternalEventDependent,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::ExternalProvider,
    thread_safety: ThreadSafetyClass::HostSerialized,
    arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::ExternalProvider,
    surface_fec_dependency_profile: FecDependencyProfile::ExternalProvider,
};

const WEB_IMAGE_TYPE_NAME: &str = "_webimage";
const WEB_IMAGE_IDENTIFIER_KEY: &str = "WebImageIdentifier";
const DISPLAY_STRING_KEY: &str = "_DisplayString";
const SOURCE_KEY: &str = "Source";
const ALT_TEXT_KEY: &str = "AltText";
const SIZING_KEY: &str = "Sizing";
const HEIGHT_KEY: &str = "Height";
const WIDTH_KEY: &str = "Width";

#[derive(Debug, Clone, PartialEq)]
pub enum ImageEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Preparation(CoercionError),
    Domain(WorksheetErrorCode),
    HostInfoProviderMissing(&'static str),
    HostInfo(HostInfoError),
}

fn required_text_arg(
    prepared: &[PreparedArgValue],
    index: usize,
) -> Result<ExcelText, ImageEvalError> {
    match prepared.get(index) {
        Some(PreparedArgValue::Eval(EvalValue::Text(text))) => Ok(text.clone()),
        _ => Err(ImageEvalError::Domain(WorksheetErrorCode::Value)),
    }
}

fn optional_text_arg(
    prepared: &[PreparedArgValue],
    index: usize,
) -> Result<Option<ExcelText>, ImageEvalError> {
    match prepared.get(index) {
        None | Some(PreparedArgValue::MissingArg) => Ok(None),
        Some(PreparedArgValue::Eval(EvalValue::Text(text))) => Ok(Some(text.clone())),
        _ => Err(ImageEvalError::Domain(WorksheetErrorCode::Value)),
    }
}

fn optional_number_arg(
    prepared: &[PreparedArgValue],
    index: usize,
) -> Result<Option<f64>, ImageEvalError> {
    match prepared.get(index) {
        None | Some(PreparedArgValue::MissingArg) => Ok(None),
        Some(PreparedArgValue::Eval(EvalValue::Number(number))) if number.is_finite() => {
            Ok(Some(*number))
        }
        _ => Err(ImageEvalError::Domain(WorksheetErrorCode::Value)),
    }
}

fn parse_sizing_mode(
    prepared: &[PreparedArgValue],
    index: usize,
) -> Result<ImageSizingMode, ImageEvalError> {
    match optional_number_arg(prepared, index)? {
        None => Ok(ImageSizingMode::FitCell),
        Some(number) if number.fract() == 0.0 => match number as i32 {
            0 => Ok(ImageSizingMode::FitCell),
            1 => Ok(ImageSizingMode::FillCell),
            2 => Ok(ImageSizingMode::OriginalSize),
            3 => Ok(ImageSizingMode::Custom),
            _ => Err(ImageEvalError::Domain(WorksheetErrorCode::Value)),
        },
        Some(_) => Err(ImageEvalError::Domain(WorksheetErrorCode::Value)),
    }
}

fn validate_image_request(request: &ImageRequest) -> Result<(), ImageEvalError> {
    let source = request.source.to_string_lossy();
    if !source.to_ascii_lowercase().starts_with("https://") {
        return Err(ImageEvalError::Domain(WorksheetErrorCode::Value));
    }

    let custom_dimension_valid = |value: f64| value.is_finite() && value >= 1.0;

    match request.sizing {
        ImageSizingMode::Custom => {
            if request.height.is_none() && request.width.is_none() {
                return Err(ImageEvalError::Domain(WorksheetErrorCode::Value));
            }
            if request
                .height
                .is_some_and(|value| !custom_dimension_valid(value))
                || request
                    .width
                    .is_some_and(|value| !custom_dimension_valid(value))
            {
                return Err(ImageEvalError::Domain(WorksheetErrorCode::Value));
            }
        }
        ImageSizingMode::FitCell | ImageSizingMode::FillCell | ImageSizingMode::OriginalSize => {
            if request.height.is_some() || request.width.is_some() {
                return Err(ImageEvalError::Domain(WorksheetErrorCode::Value));
            }
        }
    }

    Ok(())
}

pub fn parse_image_request(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<ImageRequest, ImageEvalError> {
    if !IMAGE_META.arity.accepts(args.len()) {
        return Err(ImageEvalError::ArityMismatch {
            expected_min: IMAGE_META.arity.min,
            expected_max: IMAGE_META.arity.max,
            actual: args.len(),
        });
    }

    let prepared = prepare_args_values_only(args, resolver).map_err(ImageEvalError::Preparation)?;
    let request = ImageRequest {
        source: required_text_arg(&prepared, 0)?,
        alt_text: optional_text_arg(&prepared, 1)?,
        sizing: parse_sizing_mode(&prepared, 2)?,
        height: optional_number_arg(&prepared, 3)?,
        width: optional_number_arg(&prepared, 4)?,
    };
    validate_image_request(&request)?;
    Ok(request)
}

fn build_web_image_rich_value(request: &ImageRequest, resolved: &ResolvedWebImage) -> RichValue {
    let mut kvps = vec![
        RichValueKeyValue {
            key: WEB_IMAGE_IDENTIFIER_KEY.to_string(),
            value: RichValueData::Text(ExcelText::from_interop_assignment(
                &resolved.web_image_identifier,
            )),
        },
        RichValueKeyValue {
            key: DISPLAY_STRING_KEY.to_string(),
            value: RichValueData::Text(resolved.published_fallback.clone()),
        },
        RichValueKeyValue {
            key: SOURCE_KEY.to_string(),
            value: RichValueData::Text(request.source.clone()),
        },
        RichValueKeyValue {
            key: SIZING_KEY.to_string(),
            value: RichValueData::Number(match request.sizing {
                ImageSizingMode::FitCell => 0.0,
                ImageSizingMode::FillCell => 1.0,
                ImageSizingMode::OriginalSize => 2.0,
                ImageSizingMode::Custom => 3.0,
            }),
        },
    ];

    if let Some(alt_text) = &request.alt_text {
        kvps.push(RichValueKeyValue {
            key: ALT_TEXT_KEY.to_string(),
            value: RichValueData::Text(alt_text.clone()),
        });
    }
    if let Some(height) = request.height {
        kvps.push(RichValueKeyValue {
            key: HEIGHT_KEY.to_string(),
            value: RichValueData::Number(height),
        });
    }
    if let Some(width) = request.width {
        kvps.push(RichValueKeyValue {
            key: WIDTH_KEY.to_string(),
            value: RichValueData::Number(width),
        });
    }

    RichValue {
        value_type: RichValueType {
            type_name: WEB_IMAGE_TYPE_NAME.to_string(),
            required_keys: vec![WEB_IMAGE_IDENTIFIER_KEY.to_string()],
            key_flags: vec![RichValueKeyFlag {
                key: DISPLAY_STRING_KEY.to_string(),
                flag: "ExcludeFromCalcComparison".to_string(),
                value: true,
            }],
        },
        fallback: RichValueData::Text(resolved.published_fallback.clone()),
        kvps,
    }
}

fn image_provider_error_value(result: &ImageProviderResult) -> EvalValue {
    match result {
        ImageProviderResult::ConnectionFailed => EvalValue::Error(WorksheetErrorCode::Connect),
        ImageProviderResult::CapabilityDenied => EvalValue::Error(WorksheetErrorCode::Blocked),
        ImageProviderResult::ProviderError(code) => EvalValue::Error(*code),
        ImageProviderResult::Image(_) => unreachable!("success handled separately"),
    }
}

pub fn eval_image_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
    host_info: Option<&dyn HostInfoProvider>,
) -> Result<EvalValue, ImageEvalError> {
    let request = parse_image_request(args, resolver)?;
    let provider = host_info.ok_or(ImageEvalError::HostInfoProviderMissing("image_provider"))?;
    let result = provider
        .query_image(&request)
        .map_err(ImageEvalError::HostInfo)?;
    Ok(match result {
        ImageProviderResult::Image(image) => EvalValue::Text(image.published_fallback),
        other => image_provider_error_value(&other),
    })
}

pub fn eval_image_surface_extended(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
    host_info: Option<&dyn HostInfoProvider>,
) -> Result<ExtendedValue, ImageEvalError> {
    let request = parse_image_request(args, resolver)?;
    let provider = host_info.ok_or(ImageEvalError::HostInfoProviderMissing("image_provider"))?;
    let result = provider
        .query_image(&request)
        .map_err(ImageEvalError::HostInfo)?;
    Ok(match result {
        ImageProviderResult::Image(image) => {
            ExtendedValue::RichValue(Box::new(build_web_image_rich_value(&request, &image)))
        }
        other => ExtendedValue::Core(image_provider_error_value(&other)),
    })
}

pub fn map_image_error_to_ws(error: &ImageEvalError) -> WorksheetErrorCode {
    match error {
        ImageEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        ImageEvalError::Preparation(CoercionError::WorksheetError(code)) => *code,
        ImageEvalError::Preparation(_) => WorksheetErrorCode::Value,
        ImageEvalError::Domain(code) => *code,
        ImageEvalError::HostInfoProviderMissing(_) => WorksheetErrorCode::Value,
        ImageEvalError::HostInfo(HostInfoError::ProviderFailure { .. }) => {
            WorksheetErrorCode::Value
        }
        ImageEvalError::HostInfo(
            HostInfoError::UnsupportedImageQuery
            | HostInfoError::UnsupportedTranslateQuery
            | HostInfoError::UnsupportedWidthConversionProfileQuery(_)
            | HostInfoError::UnsupportedCellInfoQuery(_)
            | HostInfoError::UnsupportedInfoQuery(_)
            | HostInfoError::UnsupportedFormulaTextQuery
            | HostInfoError::UnsupportedSheetIndexQuery
            | HostInfoError::UnsupportedSheetCountQuery
            | HostInfoError::UnsupportedAggregateReferenceContextQuery,
        ) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::{RefResolutionError, ResolverCapabilities};
    use crate::value::ReferenceLike;

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

    #[derive(Clone)]
    struct MockImageProvider {
        result: ImageProviderResult,
    }

    impl HostInfoProvider for MockImageProvider {
        fn query_image(
            &self,
            _request: &ImageRequest,
        ) -> Result<ImageProviderResult, HostInfoError> {
            Ok(self.result.clone())
        }
    }

    fn text_arg(text: &str) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Text(ExcelText::from_interop_assignment(text)))
    }

    fn number_arg(value: f64) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Number(value))
    }

    #[test]
    fn parse_image_request_defaults_to_fit_cell() {
        let request = parse_image_request(
            &[
                text_arg("https://example.com/image.png"),
                text_arg("Sphere"),
            ],
            &MockResolver,
        )
        .expect("request");

        assert_eq!(
            request.source.to_string_lossy(),
            "https://example.com/image.png"
        );
        assert_eq!(
            request.alt_text.as_ref().map(ExcelText::to_string_lossy),
            Some("Sphere".to_string())
        );
        assert_eq!(request.sizing, ImageSizingMode::FitCell);
        assert_eq!(request.height, None);
        assert_eq!(request.width, None);
    }

    #[test]
    fn parse_image_request_accepts_custom_size_with_one_dimension() {
        let request = parse_image_request(
            &[
                text_arg("https://example.com/image.png"),
                CallArgValue::MissingArg,
                number_arg(3.0),
                number_arg(100.0),
            ],
            &MockResolver,
        )
        .expect("request");

        assert_eq!(request.sizing, ImageSizingMode::Custom);
        assert_eq!(request.height, Some(100.0));
        assert_eq!(request.width, None);
    }

    #[test]
    fn parse_image_request_rejects_non_text_and_non_https_inputs() {
        let non_text = parse_image_request(&[number_arg(1.0)], &MockResolver);
        assert_eq!(
            non_text,
            Err(ImageEvalError::Domain(WorksheetErrorCode::Value))
        );

        let non_https =
            parse_image_request(&[text_arg("http://example.com/image.png")], &MockResolver);
        assert_eq!(
            non_https,
            Err(ImageEvalError::Domain(WorksheetErrorCode::Value))
        );
    }

    #[test]
    fn parse_image_request_rejects_invalid_sizing_matrix() {
        let non_custom_with_height = parse_image_request(
            &[
                text_arg("https://example.com/image.png"),
                CallArgValue::MissingArg,
                number_arg(0.0),
                number_arg(100.0),
            ],
            &MockResolver,
        );
        assert_eq!(
            non_custom_with_height,
            Err(ImageEvalError::Domain(WorksheetErrorCode::Value))
        );

        let custom_without_dimensions = parse_image_request(
            &[
                text_arg("https://example.com/image.png"),
                CallArgValue::MissingArg,
                number_arg(3.0),
            ],
            &MockResolver,
        );
        assert_eq!(
            custom_without_dimensions,
            Err(ImageEvalError::Domain(WorksheetErrorCode::Value))
        );
    }

    #[test]
    fn image_surface_uses_provider_fallback_and_error_mapping() {
        let provider = MockImageProvider {
            result: ImageProviderResult::Image(ResolvedWebImage {
                web_image_identifier: "img-1".to_string(),
                published_fallback: ExcelText::from_interop_assignment("-2146826273"),
            }),
        };
        let got = eval_image_surface(
            &[
                text_arg("https://example.com/image.png"),
                text_arg("Sphere"),
            ],
            &MockResolver,
            Some(&provider),
        );
        assert_eq!(
            got,
            Ok(EvalValue::Text(ExcelText::from_interop_assignment(
                "-2146826273"
            )))
        );

        let blocked = MockImageProvider {
            result: ImageProviderResult::CapabilityDenied,
        };
        let got = eval_image_surface(
            &[text_arg("https://example.com/image.png")],
            &MockResolver,
            Some(&blocked),
        );
        assert_eq!(got, Ok(EvalValue::Error(WorksheetErrorCode::Blocked)));
    }

    #[test]
    fn image_extended_surface_returns_webimage_rich_value() {
        let provider = MockImageProvider {
            result: ImageProviderResult::Image(ResolvedWebImage {
                web_image_identifier: "img-1".to_string(),
                published_fallback: ExcelText::from_interop_assignment("-2146826273"),
            }),
        };
        let got = eval_image_surface_extended(
            &[
                text_arg("https://example.com/image.png"),
                text_arg("Sphere"),
                number_arg(3.0),
                number_arg(100.0),
                number_arg(200.0),
            ],
            &MockResolver,
            Some(&provider),
        )
        .expect("extended image");

        match got {
            ExtendedValue::RichValue(rich) => {
                assert_eq!(rich.value_type.type_name, WEB_IMAGE_TYPE_NAME);
                assert_eq!(
                    rich.kvps[0],
                    RichValueKeyValue {
                        key: WEB_IMAGE_IDENTIFIER_KEY.to_string(),
                        value: RichValueData::Text(ExcelText::from_interop_assignment("img-1")),
                    }
                );
                assert!(rich.kvps.iter().any(|kvp| kvp.key == SOURCE_KEY));
                assert!(rich.kvps.iter().any(|kvp| kvp.key == ALT_TEXT_KEY));
                assert!(rich.kvps.iter().any(|kvp| kvp.key == HEIGHT_KEY));
                assert!(rich.kvps.iter().any(|kvp| kvp.key == WIDTH_KEY));
            }
            other => panic!("expected rich value, got {other:?}"),
        }
    }

    #[test]
    fn image_extended_surface_maps_connect_failures() {
        let provider = MockImageProvider {
            result: ImageProviderResult::ConnectionFailed,
        };
        let got = eval_image_surface_extended(
            &[text_arg("https://example.com/image.png")],
            &MockResolver,
            Some(&provider),
        );
        assert_eq!(
            got,
            Ok(ExtendedValue::Core(EvalValue::Error(
                WorksheetErrorCode::Connect
            )))
        );
    }

    #[test]
    fn map_image_error_treats_missing_provider_as_value() {
        assert_eq!(
            map_image_error_to_ws(&ImageEvalError::HostInfoProviderMissing("image_provider")),
            WorksheetErrorCode::Value
        );
    }
}

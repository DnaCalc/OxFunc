use oxfml_core::seam::AcceptDecision;
use oxfml_core::test_support::host::SingleFormulaHost;
use oxfml_core::{ReturnedValueSurfaceKind, ValuePayload};
use oxfunc_core::host_info::{
    HostInfoError, HostInfoProvider, ImageProviderResult, ImageRequest, ImageSizingMode,
    ResolvedWebImage,
};
use oxfunc_core::locale_format::{
    FormatCodeEngine, FormatFailure, LocaleFormatContext, LocaleProfileId, LocaleValueParser,
    ParseFailure, WorkbookDateSystem, format_profile,
};
use oxfunc_core::value::{EvalValue, ExcelText};

struct TestImageProvider;
struct TestLocaleValueParser;
struct TestFormatCodeEngine;

static TEST_LOCALE_VALUE_PARSER: TestLocaleValueParser = TestLocaleValueParser;
static TEST_FORMAT_CODE_ENGINE: TestFormatCodeEngine = TestFormatCodeEngine;

fn en_us_test_locale() -> LocaleFormatContext<'static> {
    LocaleFormatContext {
        profile: format_profile(LocaleProfileId::EnUs),
        date_system: WorkbookDateSystem::System1900,
        parser: &TEST_LOCALE_VALUE_PARSER,
        formatter: &TEST_FORMAT_CODE_ENGINE,
    }
}

impl LocaleValueParser for TestLocaleValueParser {
    fn parse_value_text(
        &self,
        _profile: &oxfunc_core::locale_format::FormatProfile,
        _date_system: WorkbookDateSystem,
        text: &str,
    ) -> Result<f64, ParseFailure> {
        Err(ParseFailure::UnsupportedText(text.to_string()))
    }
}

impl FormatCodeEngine for TestFormatCodeEngine {
    fn render_with_code(
        &self,
        _profile: &oxfunc_core::locale_format::FormatProfile,
        _date_system: WorkbookDateSystem,
        _value: f64,
        code: &str,
    ) -> Result<ExcelText, FormatFailure> {
        Err(FormatFailure::UnsupportedCode(code.to_string()))
    }

    fn render_currency(
        &self,
        _profile: &oxfunc_core::locale_format::FormatProfile,
        _value: f64,
        _decimals: i32,
    ) -> Result<ExcelText, FormatFailure> {
        Err(FormatFailure::UnsupportedCode("currency".to_string()))
    }

    fn render_fixed(
        &self,
        _profile: &oxfunc_core::locale_format::FormatProfile,
        _value: f64,
        _decimals: i32,
        _no_commas: bool,
    ) -> Result<ExcelText, FormatFailure> {
        Err(FormatFailure::UnsupportedCode("fixed".to_string()))
    }
}

impl HostInfoProvider for TestImageProvider {
    fn query_image(&self, request: &ImageRequest) -> Result<ImageProviderResult, HostInfoError> {
        assert_eq!(
            request.source.to_string_lossy(),
            "https://example.com/image.png"
        );
        assert_eq!(
            request.alt_text.as_ref().map(ExcelText::to_string_lossy),
            Some("Sphere".to_string())
        );
        assert_eq!(request.sizing, ImageSizingMode::Custom);
        assert_eq!(request.height, Some(100.0));
        assert_eq!(request.width, Some(200.0));

        Ok(ImageProviderResult::Image(ResolvedWebImage {
            web_image_identifier: "img-1".to_string(),
            published_fallback: ExcelText::from_interop_assignment("-2146826273"),
        }))
    }
}

#[test]
fn image_formula_preserves_webimage_rich_value_carrier_from_oxfunc_side() {
    let locale = en_us_test_locale();
    let mut host = SingleFormulaHost::new(
        "formula:image-rich-carrier",
        "=IMAGE(\"https://example.com/image.png\",\"Sphere\",3,100,200)",
    );

    let run = host
        .recalc(Some(&TestImageProvider), Some(&locale))
        .expect("image host recalc");

    assert_eq!(
        run.published_worksheet_value,
        EvalValue::Text(ExcelText::from_interop_assignment("-2146826273"))
    );
    assert_eq!(
        run.returned_value_surface.kind,
        ReturnedValueSurfaceKind::RichValue
    );
    assert_eq!(
        run.returned_value_surface.payload_summary,
        "RichValue(_webimage)"
    );
    assert_eq!(
        run.returned_value_surface.rich_value_type_name.as_deref(),
        Some("_webimage")
    );
    assert_eq!(
        run.candidate_result.returned_value_surface.kind,
        ReturnedValueSurfaceKind::RichValue
    );

    match &run.commit_decision {
        AcceptDecision::Accepted(bundle) => {
            assert_eq!(
                bundle.returned_value_surface.kind,
                ReturnedValueSurfaceKind::RichValue
            );
            assert_eq!(
                bundle
                    .returned_value_surface
                    .rich_value_type_name
                    .as_deref(),
                Some("_webimage")
            );
            assert_eq!(
                bundle.value_delta.published_payload,
                ValuePayload::Text("-2146826273".to_string())
            );
        }
        AcceptDecision::Rejected(_) => panic!("expected accepted IMAGE recalc"),
    }
}

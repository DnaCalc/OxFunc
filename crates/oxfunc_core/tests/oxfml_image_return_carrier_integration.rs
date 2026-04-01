use oxfml_core::seam::AcceptDecision;
use oxfml_core::substrate::host::SingleFormulaHost;
use oxfml_core::{ReturnedValueSurfaceKind, ValuePayload};
use oxfunc_core::host_info::{
    HostInfoError, HostInfoProvider, ImageProviderResult, ImageRequest, ImageSizingMode,
    ResolvedWebImage,
};
use oxfunc_core::locale_format::en_us_context;
use oxfunc_core::value::{EvalValue, ExcelText};

struct TestImageProvider;

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
    let locale = en_us_context();
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

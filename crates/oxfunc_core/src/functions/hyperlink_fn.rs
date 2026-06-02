use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    PreparedValue, coerce_prepared_to_text, prepare_args_values_only, prepare_calc_values_only,
    prepared_from_calc_value,
};
use crate::resolver::ReferenceSystemProvider;
use crate::value::{
    CalcValue, CellStyleHint, CoreValue, ExcelText, FunctionArg, FunctionValue, PresentationHint,
    WorksheetErrorCode,
};

pub const HYPERLINK_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.HYPERLINK",
    arity: Arity { min: 1, max: 2 },
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::EnvironmentState,
    thread_safety: ThreadSafetyClass::HostSerialized,
    arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::TextToText,
    fec_dependency_profile: FecDependencyProfile::Composite,
    surface_fec_dependency_profile: FecDependencyProfile::Composite,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HyperlinkRequest {
    pub link_location: ExcelText,
    pub display_text: ExcelText,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HyperlinkEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
}

pub fn parse_hyperlink_request(
    args: &[FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<HyperlinkRequest, HyperlinkEvalError> {
    if !HYPERLINK_META.arity.accepts(args.len()) {
        return Err(HyperlinkEvalError::ArityMismatch {
            expected_min: HYPERLINK_META.arity.min,
            expected_max: HYPERLINK_META.arity.max,
            actual: args.len(),
        });
    }
    let prepared =
        prepare_args_values_only(args, resolver).map_err(HyperlinkEvalError::Coercion)?;
    parse_hyperlink_request_prepared(&prepared)
}

fn parse_hyperlink_request_prepared(
    prepared: &[PreparedValue],
) -> Result<HyperlinkRequest, HyperlinkEvalError> {
    if !HYPERLINK_META.arity.accepts(prepared.len()) {
        return Err(HyperlinkEvalError::ArityMismatch {
            expected_min: HYPERLINK_META.arity.min,
            expected_max: HYPERLINK_META.arity.max,
            actual: prepared.len(),
        });
    }
    let link_location =
        coerce_prepared_to_text(&prepared[0]).map_err(HyperlinkEvalError::Coercion)?;
    let display_text = if prepared.len() >= 2 {
        coerce_prepared_to_text(&prepared[1]).map_err(HyperlinkEvalError::Coercion)?
    } else {
        link_location.clone()
    };
    Ok(HyperlinkRequest {
        link_location,
        display_text,
    })
}

pub fn parse_hyperlink_request_calc(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<HyperlinkRequest, HyperlinkEvalError> {
    let prepared_calc =
        prepare_calc_values_only(args, resolver).map_err(HyperlinkEvalError::Coercion)?;
    let prepared = prepared_calc
        .iter()
        .map(prepared_from_calc_value)
        .collect::<Vec<_>>();
    parse_hyperlink_request_prepared(&prepared)
}

pub fn eval_hyperlink_surface(
    args: &[FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<FunctionValue, HyperlinkEvalError> {
    let request = parse_hyperlink_request(args, resolver)?;
    Ok(FunctionValue::Text(request.display_text))
}

pub fn eval_hyperlink_calc_surface_rich(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, HyperlinkEvalError> {
    let request = parse_hyperlink_request_calc(args, resolver)?;
    Ok(CalcValue::with_presentation(
        CoreValue::Text(request.display_text),
        PresentationHint::style(CellStyleHint::Hyperlink),
    ))
}

pub fn eval_hyperlink_surface_rich(
    args: &[FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, HyperlinkEvalError> {
    let FunctionValue::Text(value) = eval_hyperlink_surface(args, resolver)? else {
        unreachable!("hyperlink surface returns text");
    };
    Ok(CalcValue::with_presentation(
        CoreValue::Text(value),
        PresentationHint::style(CellStyleHint::Hyperlink),
    ))
}

pub fn map_hyperlink_error_to_ws(error: &HyperlinkEvalError) -> WorksheetErrorCode {
    match error {
        HyperlinkEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        HyperlinkEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        HyperlinkEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::ReferenceSystemCapabilities;

    struct MockResolver;

    impl ReferenceSystemProvider for MockResolver {
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

    fn text_arg(text: &str) -> FunctionArg {
        FunctionArg::Eval(FunctionValue::Text(ExcelText::from_interop_assignment(
            text,
        )))
    }

    #[test]
    fn hyperlink_surface_returns_link_location_when_friendly_name_is_omitted() {
        assert_eq!(
            eval_hyperlink_surface(&[text_arg("https://example.com")], &MockResolver),
            Ok(FunctionValue::Text(ExcelText::from_interop_assignment(
                "https://example.com"
            )))
        );
    }

    #[test]
    fn hyperlink_surface_returns_friendly_name_when_present() {
        let request = parse_hyperlink_request(
            &[text_arg("https://example.com"), text_arg("Go")],
            &MockResolver,
        )
        .expect("request");
        assert_eq!(
            request.link_location.to_string_lossy(),
            "https://example.com"
        );
        assert_eq!(request.display_text.to_string_lossy(), "Go");
        assert_eq!(
            eval_hyperlink_surface(
                &[text_arg("https://example.com"), text_arg("Go")],
                &MockResolver
            ),
            Ok(FunctionValue::Text(ExcelText::from_interop_assignment(
                "Go"
            )))
        );
    }

    #[test]
    fn hyperlink_rich_surface_wraps_text_with_hyperlink_style_hint() {
        assert_eq!(
            eval_hyperlink_surface_rich(
                &[text_arg("https://example.com"), text_arg("Go")],
                &MockResolver
            ),
            Ok(CalcValue::with_presentation(
                CoreValue::Text(ExcelText::from_interop_assignment("Go")),
                PresentationHint::style(CellStyleHint::Hyperlink),
            ))
        );
    }
}

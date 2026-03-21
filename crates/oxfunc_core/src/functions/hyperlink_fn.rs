use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{coerce_prepared_to_text, prepare_args_values_only};
use crate::resolver::ReferenceResolver;
use crate::value::{
    CallArgValue, CellStyleHint, EvalValue, ExcelText, ExtendedValue, PresentationHint,
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
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
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

pub fn eval_hyperlink_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, HyperlinkEvalError> {
    let request = parse_hyperlink_request(args, resolver)?;
    Ok(EvalValue::Text(request.display_text))
}

pub fn eval_hyperlink_surface_extended(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<ExtendedValue, HyperlinkEvalError> {
    let value = eval_hyperlink_surface(args, resolver)?;
    Ok(ExtendedValue::ValueWithPresentation {
        value,
        hint: PresentationHint::style(CellStyleHint::Hyperlink),
    })
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

    fn text_arg(text: &str) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Text(ExcelText::from_interop_assignment(text)))
    }

    #[test]
    fn hyperlink_surface_returns_link_location_when_friendly_name_is_omitted() {
        assert_eq!(
            eval_hyperlink_surface(&[text_arg("https://example.com")], &MockResolver),
            Ok(EvalValue::Text(ExcelText::from_interop_assignment(
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
            Ok(EvalValue::Text(ExcelText::from_interop_assignment("Go")))
        );
    }

    #[test]
    fn hyperlink_extended_surface_wraps_text_with_hyperlink_style_hint() {
        assert_eq!(
            eval_hyperlink_surface_extended(
                &[text_arg("https://example.com"), text_arg("Go")],
                &MockResolver
            ),
            Ok(ExtendedValue::ValueWithPresentation {
                value: EvalValue::Text(ExcelText::from_interop_assignment("Go")),
                hint: PresentationHint::style(CellStyleHint::Hyperlink),
            })
        );
    }
}

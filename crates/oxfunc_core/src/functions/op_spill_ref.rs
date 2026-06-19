use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::a1_refs::{format_relative_target, parse_a1_reference};
use crate::resolver::ReferenceSystemProvider;
use crate::value::CalcValue;
use crate::value::{ReferenceKind, ReferenceLike, WorksheetErrorCode};

pub const OP_SPILL_REF_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.OP_SPILL_REF",
    arity: Arity::exact(1),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::WorkbookState,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::RefsVisibleInAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::RefOnly,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
    real_result_policy: FunctionMeta::DEFAULT_REAL_RESULT_POLICY,
};

#[derive(Debug, Clone, PartialEq)]
pub enum SpillRefEvalError {
    ArityMismatch { expected: usize, actual: usize },
    AnchorReferenceRequired,
    InvalidAnchorShape(String),
}

fn reference_arg(arg: &CalcValue) -> Result<ReferenceLike, SpillRefEvalError> {
    arg.as_reference()
        .cloned()
        .ok_or(SpillRefEvalError::AnchorReferenceRequired)
}

fn normalize_anchor_target(reference: &ReferenceLike) -> Result<String, SpillRefEvalError> {
    let target = reference.target();
    let trimmed = target.trim();
    if trimmed.ends_with('#') {
        return Ok(trimmed.to_string());
    }

    if let Some(parsed) = parse_a1_reference(trimmed) {
        if parsed.width() != 1 || parsed.height() != 1 {
            return Err(SpillRefEvalError::InvalidAnchorShape(trimmed.to_string()));
        }
        let normalized = format_relative_target(&parsed)
            .ok_or_else(|| SpillRefEvalError::InvalidAnchorShape(trimmed.to_string()))?;
        return Ok(format!("{normalized}#"));
    }

    Ok(format!("{trimmed}#"))
}

pub fn eval_op_spill_ref_surface(
    args: &[CalcValue],
    _resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, SpillRefEvalError> {
    if args.len() != 1 {
        return Err(SpillRefEvalError::ArityMismatch {
            expected: 1,
            actual: args.len(),
        });
    }

    let reference = reference_arg(&args[0])?;
    let target = normalize_anchor_target(&reference)?;
    Ok(CalcValue::reference(ReferenceLike::new(
        ReferenceKind::SpillAnchor,
        target,
    )))
}

pub fn map_op_spill_ref_error_to_ws(e: &SpillRefEvalError) -> WorksheetErrorCode {
    match e {
        SpillRefEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        SpillRefEvalError::AnchorReferenceRequired => WorksheetErrorCode::Ref,
        SpillRefEvalError::InvalidAnchorShape(_) => WorksheetErrorCode::Ref,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::ReferenceSystemCapabilities;

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
    fn eval_op_spill_ref_a1_anchor_returns_spill_anchor_reference() {
        let got = eval_op_spill_ref_surface(
            &[CalcValue::reference(ReferenceLike::new(
                ReferenceKind::A1,
                "Sheet1!B2".to_string(),
            ))],
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(CalcValue::reference(ReferenceLike::new(
                ReferenceKind::SpillAnchor,
                "Sheet1!B2#".to_string()
            )))
        );
    }

    #[test]
    fn eval_op_spill_ref_passes_through_existing_spill_anchor() {
        let got = eval_op_spill_ref_surface(
            &[CalcValue::reference(ReferenceLike::new(
                ReferenceKind::SpillAnchor,
                "B1#".to_string(),
            ))],
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(CalcValue::reference(ReferenceLike::new(
                ReferenceKind::SpillAnchor,
                "B1#".to_string()
            )))
        );
    }

    #[test]
    fn eval_op_spill_ref_rejects_multi_cell_a1_area() {
        let got = eval_op_spill_ref_surface(
            &[CalcValue::reference(ReferenceLike::new(
                ReferenceKind::Area,
                "A1:A3".to_string(),
            ))],
            &NoResolver,
        );
        assert_eq!(
            got,
            Err(SpillRefEvalError::InvalidAnchorShape("A1:A3".to_string()))
        );
    }

    #[test]
    fn eval_op_spill_ref_accepts_named_anchor_text_verbatim() {
        let got = eval_op_spill_ref_surface(
            &[CalcValue::reference(ReferenceLike::new(
                ReferenceKind::A1,
                "SpillName".to_string(),
            ))],
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(CalcValue::reference(ReferenceLike::new(
                ReferenceKind::SpillAnchor,
                "SpillName#".to_string()
            )))
        );
    }

    #[test]
    fn eval_op_spill_ref_requires_reference_operand() {
        let got = eval_op_spill_ref_surface(&[(CalcValue::number(1.0))], &NoResolver);
        assert_eq!(got, Err(SpillRefEvalError::AnchorReferenceRequired));
    }
}

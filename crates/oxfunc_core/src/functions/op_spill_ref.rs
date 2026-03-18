use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::a1_refs::{format_relative_target, parse_a1_reference};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, ReferenceKind, ReferenceLike, WorksheetErrorCode};

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
};

#[derive(Debug, Clone, PartialEq)]
pub enum SpillRefEvalError {
    ArityMismatch { expected: usize, actual: usize },
    AnchorReferenceRequired,
    InvalidAnchorShape(String),
}

fn reference_arg(arg: &CallArgValue) -> Result<ReferenceLike, SpillRefEvalError> {
    match arg {
        CallArgValue::Reference(reference)
        | CallArgValue::Eval(EvalValue::Reference(reference)) => Ok(reference.clone()),
        _ => Err(SpillRefEvalError::AnchorReferenceRequired),
    }
}

fn normalize_anchor_target(reference: &ReferenceLike) -> Result<String, SpillRefEvalError> {
    let trimmed = reference.target.trim();
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
    args: &[CallArgValue],
    _resolver: &impl ReferenceResolver,
) -> Result<EvalValue, SpillRefEvalError> {
    if args.len() != 1 {
        return Err(SpillRefEvalError::ArityMismatch {
            expected: 1,
            actual: args.len(),
        });
    }

    let reference = reference_arg(&args[0])?;
    let target = normalize_anchor_target(&reference)?;
    Ok(EvalValue::Reference(ReferenceLike {
        kind: ReferenceKind::SpillAnchor,
        target,
    }))
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
    use crate::resolver::{RefResolutionError, ResolverCapabilities};

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
    fn eval_op_spill_ref_a1_anchor_returns_spill_anchor_reference() {
        let got = eval_op_spill_ref_surface(
            &[CallArgValue::Reference(ReferenceLike {
                kind: ReferenceKind::A1,
                target: "Sheet1!B2".to_string(),
            })],
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Reference(ReferenceLike {
                kind: ReferenceKind::SpillAnchor,
                target: "Sheet1!B2#".to_string(),
            }))
        );
    }

    #[test]
    fn eval_op_spill_ref_passes_through_existing_spill_anchor() {
        let got = eval_op_spill_ref_surface(
            &[CallArgValue::Reference(ReferenceLike {
                kind: ReferenceKind::SpillAnchor,
                target: "B1#".to_string(),
            })],
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Reference(ReferenceLike {
                kind: ReferenceKind::SpillAnchor,
                target: "B1#".to_string(),
            }))
        );
    }

    #[test]
    fn eval_op_spill_ref_rejects_multi_cell_a1_area() {
        let got = eval_op_spill_ref_surface(
            &[CallArgValue::Reference(ReferenceLike {
                kind: ReferenceKind::Area,
                target: "A1:A3".to_string(),
            })],
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
            &[CallArgValue::Reference(ReferenceLike {
                kind: ReferenceKind::A1,
                target: "SpillName".to_string(),
            })],
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Reference(ReferenceLike {
                kind: ReferenceKind::SpillAnchor,
                target: "SpillName#".to_string(),
            }))
        );
    }

    #[test]
    fn eval_op_spill_ref_requires_reference_operand() {
        let got =
            eval_op_spill_ref_surface(&[CallArgValue::Eval(EvalValue::Number(1.0))], &NoResolver);
        assert_eq!(got, Err(SpillRefEvalError::AnchorReferenceRequired));
    }
}

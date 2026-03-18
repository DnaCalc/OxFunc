use crate::coercion::{CoercionError, coerce_eval_to_number};
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::a1_refs::{format_relative_target, offset_reference, parse_a1_reference};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, ReferenceKind, ReferenceLike, WorksheetErrorCode};

pub const OFFSET_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.OFFSET",
    arity: Arity { min: 3, max: 5 },
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::VolatileContextual,
    host_interaction: HostInteractionClass::WorkbookState,
    thread_safety: ThreadSafetyClass::HostSerialized,
    arg_preparation_profile: ArgPreparationProfile::RefsVisibleInAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::CallerContext,
    surface_fec_dependency_profile: FecDependencyProfile::CallerContext,
};

#[derive(Debug, Clone, PartialEq)]
pub enum OffsetEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    RefArgRequired,
    InvalidReferenceText(String),
    Coercion(CoercionError),
    InvalidDimension,
}

fn parse_offset_number(
    arg: &CallArgValue,
    resolver: &impl ReferenceResolver,
) -> Result<i64, OffsetEvalError> {
    let number = match arg {
        CallArgValue::Eval(v) => coerce_eval_to_number(v, resolver),
        CallArgValue::Reference(r) => resolver
            .resolve_reference(r)
            .map_err(CoercionError::RefResolution)
            .and_then(|v| coerce_eval_to_number(&v, resolver)),
        CallArgValue::MissingArg => Err(CoercionError::MissingArg),
        CallArgValue::EmptyCell => Err(CoercionError::EmptyCell),
    }
    .map_err(OffsetEvalError::Coercion)?;
    Ok(number.trunc() as i64)
}

fn parse_optional_positive_dimension(
    arg: Option<&CallArgValue>,
    resolver: &impl ReferenceResolver,
) -> Result<Option<usize>, OffsetEvalError> {
    let Some(arg) = arg else {
        return Ok(None);
    };

    let value = parse_offset_number(arg, resolver)?;
    if value <= 0 {
        return Err(OffsetEvalError::InvalidDimension);
    }
    usize::try_from(value)
        .map(Some)
        .map_err(|_| OffsetEvalError::InvalidDimension)
}

fn parse_reference_arg(arg: &CallArgValue) -> Result<ReferenceLike, OffsetEvalError> {
    match arg {
        CallArgValue::Reference(r) => Ok(r.clone()),
        CallArgValue::Eval(EvalValue::Reference(r)) => Ok(r.clone()),
        _ => Err(OffsetEvalError::RefArgRequired),
    }
}

pub fn eval_offset_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, OffsetEvalError> {
    let argc = args.len();
    if !OFFSET_META.arity.accepts(argc) {
        return Err(OffsetEvalError::ArityMismatch {
            expected_min: OFFSET_META.arity.min,
            expected_max: OFFSET_META.arity.max,
            actual: argc,
        });
    }

    let base = parse_reference_arg(&args[0])?;
    let parsed = parse_a1_reference(&base.target)
        .ok_or_else(|| OffsetEvalError::InvalidReferenceText(base.target.clone()))?;
    let row_offset = parse_offset_number(&args[1], resolver)?;
    let col_offset = parse_offset_number(&args[2], resolver)?;
    let height = parse_optional_positive_dimension(args.get(3), resolver)?;
    let width = parse_optional_positive_dimension(args.get(4), resolver)?;
    let shifted = offset_reference(&parsed, row_offset, col_offset, height, width)
        .ok_or(OffsetEvalError::InvalidDimension)?;
    let target = format_relative_target(&shifted).ok_or(OffsetEvalError::InvalidDimension)?;

    Ok(EvalValue::Reference(ReferenceLike {
        kind: if shifted.width() == 1 && shifted.height() == 1 {
            ReferenceKind::A1
        } else {
            ReferenceKind::Area
        },
        target,
    }))
}

pub fn map_offset_error_to_ws(e: &OffsetEvalError) -> WorksheetErrorCode {
    match e {
        OffsetEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        OffsetEvalError::RefArgRequired => WorksheetErrorCode::Value,
        OffsetEvalError::InvalidReferenceText(_) => WorksheetErrorCode::Ref,
        OffsetEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        OffsetEvalError::Coercion(_) => WorksheetErrorCode::Value,
        OffsetEvalError::InvalidDimension => WorksheetErrorCode::Ref,
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
    fn eval_offset_shifts_single_cell_reference() {
        let got = eval_offset_surface(
            &[
                CallArgValue::Reference(ReferenceLike {
                    kind: ReferenceKind::A1,
                    target: "A1".to_string(),
                }),
                CallArgValue::Eval(EvalValue::Number(1.0)),
                CallArgValue::Eval(EvalValue::Number(2.0)),
            ],
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Reference(ReferenceLike {
                kind: ReferenceKind::A1,
                target: "C2".to_string(),
            }))
        );
    }

    #[test]
    fn eval_offset_resizes_reference_area() {
        let got = eval_offset_surface(
            &[
                CallArgValue::Reference(ReferenceLike {
                    kind: ReferenceKind::Area,
                    target: "A1:B2".to_string(),
                }),
                CallArgValue::Eval(EvalValue::Number(0.0)),
                CallArgValue::Eval(EvalValue::Number(1.0)),
                CallArgValue::Eval(EvalValue::Number(1.0)),
                CallArgValue::Eval(EvalValue::Number(3.0)),
            ],
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Reference(ReferenceLike {
                kind: ReferenceKind::Area,
                target: "B1:D1".to_string(),
            }))
        );
    }

    #[test]
    fn eval_offset_defaults_height_and_width_to_base_shape() {
        let got = eval_offset_surface(
            &[
                CallArgValue::Reference(ReferenceLike {
                    kind: ReferenceKind::Area,
                    target: "B2:C3".to_string(),
                }),
                CallArgValue::Eval(EvalValue::Number(1.0)),
                CallArgValue::Eval(EvalValue::Number(1.0)),
            ],
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Reference(ReferenceLike {
                kind: ReferenceKind::Area,
                target: "C3:D4".to_string(),
            }))
        );
    }

    #[test]
    fn eval_offset_preserves_sheet_prefix() {
        let got = eval_offset_surface(
            &[
                CallArgValue::Reference(ReferenceLike {
                    kind: ReferenceKind::A1,
                    target: "Sheet1!B2".to_string(),
                }),
                CallArgValue::Eval(EvalValue::Number(1.0)),
                CallArgValue::Eval(EvalValue::Number(2.0)),
            ],
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Reference(ReferenceLike {
                kind: ReferenceKind::A1,
                target: "Sheet1!D3".to_string(),
            }))
        );
    }
}

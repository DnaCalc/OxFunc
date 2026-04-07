use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::a1_refs::{
    A1Reference, A1ReferenceNotation, EXCEL_MAX_COLS, EXCEL_MAX_ROWS, format_relative_target,
    parse_a1_reference,
};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, ReferenceKind, ReferenceLike, WorksheetErrorCode};

const OP_REFERENCE_BASE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.OP_REFERENCE_BASE",
    arity: Arity::exact(2),
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

pub const OP_RANGE_REF_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.OP_RANGE_REF",
    arity: Arity { min: 2, max: 2 },
    ..OP_REFERENCE_BASE_META
};

pub const OP_INTERSECTION_REF_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.OP_INTERSECTION_REF",
    arity: Arity { min: 2, max: 2 },
    ..OP_REFERENCE_BASE_META
};

pub const OP_UNION_REF_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.OP_UNION_REF",
    arity: Arity { min: 2, max: 2 },
    ..OP_REFERENCE_BASE_META
};

pub const OP_TRIM_REF_LEADING_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.OP_TRIM_REF_LEADING",
    arity: Arity { min: 1, max: 1 },
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

pub const OP_TRIM_REF_TRAILING_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.OP_TRIM_REF_TRAILING",
    arity: Arity { min: 1, max: 1 },
    ..OP_TRIM_REF_LEADING_META
};

pub const OP_TRIM_REF_BOTH_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.OP_TRIM_REF_BOTH",
    arity: Arity { min: 1, max: 1 },
    ..OP_TRIM_REF_LEADING_META
};

#[derive(Debug, Clone, PartialEq)]
pub enum OperatorReferenceError {
    ArityMismatch { expected: usize, actual: usize },
    ReferenceRequired,
    UnsupportedReferenceSource(&'static str),
    NullIntersection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TrimMode {
    Leading,
    Trailing,
    Both,
}

fn infer_notation(reference: &A1Reference) -> A1ReferenceNotation {
    if reference.start_row == 1 && reference.end_row == EXCEL_MAX_ROWS {
        A1ReferenceNotation::WholeColumn
    } else if reference.start_col == 1 && reference.end_col == EXCEL_MAX_COLS {
        A1ReferenceNotation::WholeRow
    } else {
        A1ReferenceNotation::Rect
    }
}

fn reference_arg(arg: &CallArgValue) -> Result<ReferenceLike, OperatorReferenceError> {
    match arg {
        CallArgValue::Reference(reference)
        | CallArgValue::Eval(EvalValue::Reference(reference)) => Ok(reference.clone()),
        _ => Err(OperatorReferenceError::ReferenceRequired),
    }
}

fn parse_a1_operand(arg: &CallArgValue) -> Result<A1Reference, OperatorReferenceError> {
    let reference = reference_arg(arg)?;
    parse_a1_reference(&reference.target).ok_or(OperatorReferenceError::UnsupportedReferenceSource(
        "non_a1_reference",
    ))
}

fn ensure_same_prefix(
    lhs: &A1Reference,
    rhs: &A1Reference,
) -> Result<Option<String>, OperatorReferenceError> {
    if lhs.prefix == rhs.prefix {
        Ok(lhs.prefix.clone())
    } else {
        Err(OperatorReferenceError::UnsupportedReferenceSource(
            "mixed_prefix_reference",
        ))
    }
}

fn eval_from_a1(reference: A1Reference) -> Result<EvalValue, OperatorReferenceError> {
    let target = format_relative_target(&reference).ok_or(
        OperatorReferenceError::UnsupportedReferenceSource("unformattable_reference"),
    )?;
    Ok(EvalValue::Reference(ReferenceLike {
        kind: if reference.width() == 1 && reference.height() == 1 {
            ReferenceKind::A1
        } else {
            ReferenceKind::Area
        },
        target,
    }))
}

fn trim_reference(reference: ReferenceLike, mode: TrimMode) -> EvalValue {
    let target = match mode {
        TrimMode::Leading => reference.target.trim_start().to_string(),
        TrimMode::Trailing => reference.target.trim_end().to_string(),
        TrimMode::Both => reference.target.trim().to_string(),
    };
    EvalValue::Reference(ReferenceLike {
        kind: reference.kind,
        target,
    })
}

fn union_targets(reference: &ReferenceLike) -> Result<Vec<String>, OperatorReferenceError> {
    if matches!(reference.kind, ReferenceKind::MultiArea) {
        return reference.multi_area_targets().ok_or(
            OperatorReferenceError::UnsupportedReferenceSource("invalid_multi_area_reference"),
        );
    }

    let target = reference.target.trim();
    if target.is_empty() {
        return Err(OperatorReferenceError::UnsupportedReferenceSource(
            "invalid_multi_area_reference",
        ));
    }
    Ok(vec![target.to_string()])
}

pub fn eval_op_range_ref_surface(
    args: &[CallArgValue],
    _resolver: &impl ReferenceResolver,
) -> Result<EvalValue, OperatorReferenceError> {
    if args.len() != 2 {
        return Err(OperatorReferenceError::ArityMismatch {
            expected: 2,
            actual: args.len(),
        });
    }
    let lhs = parse_a1_operand(&args[0])?;
    let rhs = parse_a1_operand(&args[1])?;
    let prefix = ensure_same_prefix(&lhs, &rhs)?;
    let merged = A1Reference {
        prefix,
        start_row: lhs.start_row.min(rhs.start_row),
        start_col: lhs.start_col.min(rhs.start_col),
        end_row: lhs.end_row.max(rhs.end_row),
        end_col: lhs.end_col.max(rhs.end_col),
        notation: A1ReferenceNotation::Rect,
    };
    eval_from_a1(A1Reference {
        notation: infer_notation(&merged),
        ..merged
    })
}

pub fn eval_op_intersection_ref_surface(
    args: &[CallArgValue],
    _resolver: &impl ReferenceResolver,
) -> Result<EvalValue, OperatorReferenceError> {
    if args.len() != 2 {
        return Err(OperatorReferenceError::ArityMismatch {
            expected: 2,
            actual: args.len(),
        });
    }
    let lhs = parse_a1_operand(&args[0])?;
    let rhs = parse_a1_operand(&args[1])?;
    let prefix = ensure_same_prefix(&lhs, &rhs)?;
    let overlap = A1Reference {
        prefix,
        start_row: lhs.start_row.max(rhs.start_row),
        start_col: lhs.start_col.max(rhs.start_col),
        end_row: lhs.end_row.min(rhs.end_row),
        end_col: lhs.end_col.min(rhs.end_col),
        notation: A1ReferenceNotation::Rect,
    };
    if overlap.start_row > overlap.end_row || overlap.start_col > overlap.end_col {
        return Err(OperatorReferenceError::NullIntersection);
    }
    eval_from_a1(A1Reference {
        notation: infer_notation(&overlap),
        ..overlap
    })
}

pub fn eval_op_union_ref_surface(
    args: &[CallArgValue],
    _resolver: &impl ReferenceResolver,
) -> Result<EvalValue, OperatorReferenceError> {
    if args.len() != 2 {
        return Err(OperatorReferenceError::ArityMismatch {
            expected: 2,
            actual: args.len(),
        });
    }
    let lhs = reference_arg(&args[0])?;
    let rhs = reference_arg(&args[1])?;
    let mut targets = union_targets(&lhs)?;
    targets.extend(union_targets(&rhs)?);
    let multi = ReferenceLike::multi_area(targets).ok_or(
        OperatorReferenceError::UnsupportedReferenceSource("invalid_multi_area_reference"),
    )?;
    Ok(EvalValue::Reference(multi))
}

pub fn eval_op_trim_ref_leading_surface(
    args: &[CallArgValue],
    _resolver: &impl ReferenceResolver,
) -> Result<EvalValue, OperatorReferenceError> {
    if args.len() != 1 {
        return Err(OperatorReferenceError::ArityMismatch {
            expected: 1,
            actual: args.len(),
        });
    }
    Ok(trim_reference(reference_arg(&args[0])?, TrimMode::Leading))
}

pub fn eval_op_trim_ref_trailing_surface(
    args: &[CallArgValue],
    _resolver: &impl ReferenceResolver,
) -> Result<EvalValue, OperatorReferenceError> {
    if args.len() != 1 {
        return Err(OperatorReferenceError::ArityMismatch {
            expected: 1,
            actual: args.len(),
        });
    }
    Ok(trim_reference(reference_arg(&args[0])?, TrimMode::Trailing))
}

pub fn eval_op_trim_ref_both_surface(
    args: &[CallArgValue],
    _resolver: &impl ReferenceResolver,
) -> Result<EvalValue, OperatorReferenceError> {
    if args.len() != 1 {
        return Err(OperatorReferenceError::ArityMismatch {
            expected: 1,
            actual: args.len(),
        });
    }
    Ok(trim_reference(reference_arg(&args[0])?, TrimMode::Both))
}

pub fn map_operator_reference_error_to_ws(e: &OperatorReferenceError) -> WorksheetErrorCode {
    match e {
        OperatorReferenceError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        OperatorReferenceError::ReferenceRequired => WorksheetErrorCode::Ref,
        OperatorReferenceError::UnsupportedReferenceSource(_) => WorksheetErrorCode::Ref,
        OperatorReferenceError::NullIntersection => WorksheetErrorCode::Null,
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

    fn area(target: &str) -> CallArgValue {
        CallArgValue::Reference(ReferenceLike {
            kind: ReferenceKind::Area,
            target: target.to_string(),
        })
    }

    fn a1(target: &str) -> CallArgValue {
        CallArgValue::Reference(ReferenceLike {
            kind: ReferenceKind::A1,
            target: target.to_string(),
        })
    }

    #[test]
    fn range_operator_normalizes_bounds() {
        let got = eval_op_range_ref_surface(&[a1("B2"), a1("A1")], &NoResolver);
        assert_eq!(
            got,
            Ok(EvalValue::Reference(ReferenceLike {
                kind: ReferenceKind::Area,
                target: "A1:B2".to_string(),
            }))
        );
    }

    #[test]
    fn intersection_operator_projects_overlap_or_null() {
        let got = eval_op_intersection_ref_surface(&[area("A1:C3"), area("B2:D4")], &NoResolver);
        assert_eq!(
            got,
            Ok(EvalValue::Reference(ReferenceLike {
                kind: ReferenceKind::Area,
                target: "B2:C3".to_string(),
            }))
        );

        let none = eval_op_intersection_ref_surface(&[area("A1:A2"), area("C1:C2")], &NoResolver);
        assert_eq!(none, Err(OperatorReferenceError::NullIntersection));
    }

    #[test]
    fn union_operator_returns_first_class_multi_area_reference() {
        let got = eval_op_union_ref_surface(&[area("A1:A2"), area("G1:G2")], &NoResolver);
        assert_eq!(
            got,
            Ok(EvalValue::Reference(ReferenceLike {
                kind: ReferenceKind::MultiArea,
                target: "(A1:A2,G1:G2)".to_string(),
            }))
        );
    }

    #[test]
    fn union_operator_flattens_existing_multi_area_operands() {
        let lhs = CallArgValue::Reference(
            ReferenceLike::multi_area(vec!["A1:A2".to_string(), "G1:G2".to_string()]).unwrap(),
        );
        let rhs = area("J1:J2");
        let got = eval_op_union_ref_surface(&[lhs, rhs], &NoResolver);
        assert_eq!(
            got,
            Ok(EvalValue::Reference(ReferenceLike {
                kind: ReferenceKind::MultiArea,
                target: "(A1:A2,G1:G2,J1:J2)".to_string(),
            }))
        );
    }

    #[test]
    fn trim_ref_variants_trim_only_requested_edges() {
        let input = CallArgValue::Reference(ReferenceLike {
            kind: ReferenceKind::Area,
            target: "  Sheet1!A1:A2  ".to_string(),
        });
        assert_eq!(
            eval_op_trim_ref_leading_surface(std::slice::from_ref(&input), &NoResolver),
            Ok(EvalValue::Reference(ReferenceLike {
                kind: ReferenceKind::Area,
                target: "Sheet1!A1:A2  ".to_string(),
            }))
        );
        assert_eq!(
            eval_op_trim_ref_trailing_surface(std::slice::from_ref(&input), &NoResolver),
            Ok(EvalValue::Reference(ReferenceLike {
                kind: ReferenceKind::Area,
                target: "  Sheet1!A1:A2".to_string(),
            }))
        );
        assert_eq!(
            eval_op_trim_ref_both_surface(std::slice::from_ref(&input), &NoResolver),
            Ok(EvalValue::Reference(ReferenceLike {
                kind: ReferenceKind::Area,
                target: "Sheet1!A1:A2".to_string(),
            }))
        );
    }
}

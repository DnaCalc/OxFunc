use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::resolver::{
    ReferenceComposeOperation, ReferenceComposeRequest, ReferenceSystemError,
    ReferenceSystemProvider, ReferenceTransformKind, ReferenceTransformRequest, ReferenceTrimMode,
};
use crate::value::{CalcValue, ReferenceLike, WorksheetErrorCode};

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
    ReferenceSystem(ReferenceSystemError),
}

fn reference_arg(arg: &CalcValue) -> Result<ReferenceLike, OperatorReferenceError> {
    arg.as_reference()
        .cloned()
        .ok_or(OperatorReferenceError::ReferenceRequired)
}

pub fn eval_op_range_ref_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, OperatorReferenceError> {
    if args.len() != 2 {
        return Err(OperatorReferenceError::ArityMismatch {
            expected: 2,
            actual: args.len(),
        });
    }
    let lhs = reference_arg(&args[0])?;
    let rhs = reference_arg(&args[1])?;
    resolver
        .compose_references(&ReferenceComposeRequest {
            lhs,
            rhs,
            operation: ReferenceComposeOperation::Range,
        })
        .map(CalcValue::reference)
        .map_err(OperatorReferenceError::ReferenceSystem)
}

pub fn eval_op_intersection_ref_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, OperatorReferenceError> {
    if args.len() != 2 {
        return Err(OperatorReferenceError::ArityMismatch {
            expected: 2,
            actual: args.len(),
        });
    }
    let lhs = reference_arg(&args[0])?;
    let rhs = reference_arg(&args[1])?;
    resolver
        .compose_references(&ReferenceComposeRequest {
            lhs,
            rhs,
            operation: ReferenceComposeOperation::Intersection,
        })
        .map(CalcValue::reference)
        .map_err(|error| match error {
            ReferenceSystemError::ProviderFailure { detail } if detail == "null_intersection" => {
                OperatorReferenceError::NullIntersection
            }
            other => OperatorReferenceError::ReferenceSystem(other),
        })
}

pub fn eval_op_union_ref_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, OperatorReferenceError> {
    if args.len() != 2 {
        return Err(OperatorReferenceError::ArityMismatch {
            expected: 2,
            actual: args.len(),
        });
    }
    let lhs = reference_arg(&args[0])?;
    let rhs = reference_arg(&args[1])?;
    resolver
        .compose_references(&ReferenceComposeRequest {
            lhs,
            rhs,
            operation: ReferenceComposeOperation::Union,
        })
        .map(CalcValue::reference)
        .map_err(OperatorReferenceError::ReferenceSystem)
}

pub fn eval_op_trim_ref_leading_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, OperatorReferenceError> {
    if args.len() != 1 {
        return Err(OperatorReferenceError::ArityMismatch {
            expected: 1,
            actual: args.len(),
        });
    }
    resolver
        .transform_reference(&ReferenceTransformRequest {
            reference: reference_arg(&args[0])?,
            transform: ReferenceTransformKind::Trim {
                mode: ReferenceTrimMode::Leading,
            },
        })
        .map(CalcValue::reference)
        .map_err(OperatorReferenceError::ReferenceSystem)
}

pub fn eval_op_trim_ref_trailing_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, OperatorReferenceError> {
    if args.len() != 1 {
        return Err(OperatorReferenceError::ArityMismatch {
            expected: 1,
            actual: args.len(),
        });
    }
    resolver
        .transform_reference(&ReferenceTransformRequest {
            reference: reference_arg(&args[0])?,
            transform: ReferenceTransformKind::Trim {
                mode: ReferenceTrimMode::Trailing,
            },
        })
        .map(CalcValue::reference)
        .map_err(OperatorReferenceError::ReferenceSystem)
}

pub fn eval_op_trim_ref_both_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, OperatorReferenceError> {
    if args.len() != 1 {
        return Err(OperatorReferenceError::ArityMismatch {
            expected: 1,
            actual: args.len(),
        });
    }
    resolver
        .transform_reference(&ReferenceTransformRequest {
            reference: reference_arg(&args[0])?,
            transform: ReferenceTransformKind::Trim {
                mode: ReferenceTrimMode::Both,
            },
        })
        .map(CalcValue::reference)
        .map_err(OperatorReferenceError::ReferenceSystem)
}

pub fn map_operator_reference_error_to_ws(e: &OperatorReferenceError) -> WorksheetErrorCode {
    match e {
        OperatorReferenceError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        OperatorReferenceError::ReferenceRequired => WorksheetErrorCode::Ref,
        OperatorReferenceError::UnsupportedReferenceSource(_) => WorksheetErrorCode::Ref,
        OperatorReferenceError::NullIntersection => WorksheetErrorCode::Null,
        OperatorReferenceError::ReferenceSystem(_) => WorksheetErrorCode::Ref,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::ReferenceSystemCapabilities;
    use crate::value::ReferenceKind;

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

        fn transform_reference(
            &self,
            request: &ReferenceTransformRequest,
        ) -> Result<ReferenceLike, ReferenceSystemError> {
            match (request.reference.target(), request.transform.clone()) {
                ("  Sheet1!A1:A2  ", ReferenceTransformKind::Trim { mode }) => {
                    let target = match mode {
                        ReferenceTrimMode::Leading => "Sheet1!A1:A2  ",
                        ReferenceTrimMode::Trailing => "  Sheet1!A1:A2",
                        ReferenceTrimMode::Both => "Sheet1!A1:A2",
                    };
                    Ok(ReferenceLike::new(ReferenceKind::Area, target))
                }
                _ => Err(ReferenceSystemError::Unsupported {
                    operation: crate::resolver::ReferenceSystemOperation::Transform,
                }),
            }
        }

        fn compose_references(
            &self,
            request: &ReferenceComposeRequest,
        ) -> Result<ReferenceLike, ReferenceSystemError> {
            match (
                request.lhs.target(),
                request.rhs.target(),
                request.operation,
            ) {
                ("B2", "A1", ReferenceComposeOperation::Range) => {
                    Ok(ReferenceLike::new(ReferenceKind::Area, "A1:B2"))
                }
                ("A1:C3", "B2:D4", ReferenceComposeOperation::Intersection) => {
                    Ok(ReferenceLike::new(ReferenceKind::Area, "B2:C3"))
                }
                ("A1:A2", "C1:C2", ReferenceComposeOperation::Intersection) => {
                    Err(ReferenceSystemError::ProviderFailure {
                        detail: "null_intersection".to_string(),
                    })
                }
                ("A1:A2", "G1:G2", ReferenceComposeOperation::Union) => Ok(ReferenceLike::new(
                    ReferenceKind::MultiArea,
                    "(A1:A2,G1:G2)",
                )),
                ("(A1:A2,G1:G2)", "J1:J2", ReferenceComposeOperation::Union) => Ok(
                    ReferenceLike::new(ReferenceKind::MultiArea, "(A1:A2,G1:G2,J1:J2)"),
                ),
                _ => Err(ReferenceSystemError::Unsupported {
                    operation: crate::resolver::ReferenceSystemOperation::Compose,
                }),
            }
        }
    }

    fn area(target: &str) -> CalcValue {
        CalcValue::reference(ReferenceLike::new(ReferenceKind::Area, target.to_string()))
    }

    fn a1(target: &str) -> CalcValue {
        CalcValue::reference(ReferenceLike::new(ReferenceKind::A1, target.to_string()))
    }

    #[test]
    fn range_operator_normalizes_bounds() {
        let got = eval_op_range_ref_surface(&[a1("B2"), a1("A1")], &NoResolver);
        assert_eq!(
            got,
            Ok(CalcValue::reference(ReferenceLike::new(
                ReferenceKind::Area,
                "A1:B2".to_string()
            )))
        );
    }

    #[test]
    fn intersection_operator_projects_overlap_or_null() {
        let got = eval_op_intersection_ref_surface(&[area("A1:C3"), area("B2:D4")], &NoResolver);
        assert_eq!(
            got,
            Ok(CalcValue::reference(ReferenceLike::new(
                ReferenceKind::Area,
                "B2:C3".to_string()
            )))
        );

        let none = eval_op_intersection_ref_surface(&[area("A1:A2"), area("C1:C2")], &NoResolver);
        assert_eq!(none, Err(OperatorReferenceError::NullIntersection));
    }

    #[test]
    fn union_operator_returns_first_class_multi_area_reference() {
        let got = eval_op_union_ref_surface(&[area("A1:A2"), area("G1:G2")], &NoResolver);
        assert_eq!(
            got,
            Ok(CalcValue::reference(ReferenceLike::new(
                ReferenceKind::MultiArea,
                "(A1:A2,G1:G2)".to_string()
            )))
        );
    }

    #[test]
    fn union_operator_flattens_existing_multi_area_operands() {
        let lhs = CalcValue::reference(
            ReferenceLike::multi_area(vec!["A1:A2".to_string(), "G1:G2".to_string()]).unwrap(),
        );
        let rhs = area("J1:J2");
        let got = eval_op_union_ref_surface(&[lhs, rhs], &NoResolver);
        assert_eq!(
            got,
            Ok(CalcValue::reference(ReferenceLike::new(
                ReferenceKind::MultiArea,
                "(A1:A2,G1:G2,J1:J2)".to_string()
            )))
        );
    }

    #[test]
    fn trim_ref_variants_trim_only_requested_edges() {
        let input = CalcValue::reference(ReferenceLike::new(
            ReferenceKind::Area,
            "  Sheet1!A1:A2  ".to_string(),
        ));
        assert_eq!(
            eval_op_trim_ref_leading_surface(std::slice::from_ref(&input), &NoResolver),
            Ok(CalcValue::reference(ReferenceLike::new(
                ReferenceKind::Area,
                "Sheet1!A1:A2  ".to_string()
            )))
        );
        assert_eq!(
            eval_op_trim_ref_trailing_surface(std::slice::from_ref(&input), &NoResolver),
            Ok(CalcValue::reference(ReferenceLike::new(
                ReferenceKind::Area,
                "  Sheet1!A1:A2".to_string()
            )))
        );
        assert_eq!(
            eval_op_trim_ref_both_surface(std::slice::from_ref(&input), &NoResolver),
            Ok(CalcValue::reference(ReferenceLike::new(
                ReferenceKind::Area,
                "Sheet1!A1:A2".to_string()
            )))
        );
    }
}

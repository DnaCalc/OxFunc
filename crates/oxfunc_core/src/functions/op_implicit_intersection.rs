use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::a1_refs::{
    A1Reference, A1ReferenceNotation, format_relative_target, parse_a1_reference,
};
use crate::resolver::{
    CallerContext, ReferenceResolutionError, ReferenceSystemProvider, resolve_eval_value,
};
use crate::value::{CalcArray, ReferenceKind, ReferenceLike, WorksheetErrorCode};
use crate::value::{CalcValue, CoreValue};

pub const OP_IMPLICIT_INTERSECTION_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.OP_IMPLICIT_INTERSECTION",
    arity: Arity::exact(1),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::WorkbookState,
    thread_safety: ThreadSafetyClass::HostSerialized,
    arg_preparation_profile: ArgPreparationProfile::RefsVisibleInAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::Composite,
    surface_fec_dependency_profile: FecDependencyProfile::Composite,
    real_result_policy: FunctionMeta::DEFAULT_REAL_RESULT_POLICY,
};

#[derive(Debug, Clone, PartialEq)]
pub enum ImplicitIntersectionError {
    ArityMismatch { expected: usize, actual: usize },
    MissingCallerContext,
    UnsupportedReferenceSource(&'static str),
    TwoDimensionalReference,
    NoCallerAlignedIntersection,
    EmptyArray,
    EmptyCellTopLeft,
    RefResolution(ReferenceResolutionError),
}

fn scalar_from_array_value(value: &CalcValue) -> Result<CalcValue, ImplicitIntersectionError> {
    match value.core() {
        CoreValue::Number(_) | CoreValue::Text(_) | CoreValue::Logical(_) | CoreValue::Error(_) => {
            Ok(value.clone())
        }
        CoreValue::Empty | CoreValue::Missing => Err(ImplicitIntersectionError::EmptyCellTopLeft),
        CoreValue::Array(_) | CoreValue::Reference(_) => Err(
            ImplicitIntersectionError::UnsupportedReferenceSource("unsupported_array_cell"),
        ),
    }
}

fn top_left_array_value(array: &CalcArray) -> Result<CalcValue, ImplicitIntersectionError> {
    let cell = array
        .get(0, 0)
        .ok_or(ImplicitIntersectionError::EmptyArray)?;
    scalar_from_array_value(cell)
}

fn make_single_cell_reference(prefix: Option<String>, row: usize, col: usize) -> ReferenceLike {
    ReferenceLike::new(
        ReferenceKind::A1,
        format_relative_target(&A1Reference {
            prefix,
            start_row: row,
            start_col: col,
            end_row: row,
            end_col: col,
            notation: A1ReferenceNotation::Rect,
        })
        .expect("single-cell A1 reference should format"),
    )
}

fn select_reference_cell(
    reference: &A1Reference,
    caller: Option<&CallerContext>,
) -> Result<ReferenceLike, ImplicitIntersectionError> {
    if reference.width() == 1 && reference.height() == 1 {
        return Ok(make_single_cell_reference(
            reference.prefix.clone(),
            reference.start_row,
            reference.start_col,
        ));
    }

    if reference.width() == 1 {
        let caller = caller.ok_or(ImplicitIntersectionError::MissingCallerContext)?;
        if caller.row < reference.start_row || caller.row > reference.end_row {
            return Err(ImplicitIntersectionError::NoCallerAlignedIntersection);
        }
        return Ok(make_single_cell_reference(
            reference.prefix.clone(),
            caller.row,
            reference.start_col,
        ));
    }

    if reference.height() == 1 {
        let caller = caller.ok_or(ImplicitIntersectionError::MissingCallerContext)?;
        if caller.col < reference.start_col || caller.col > reference.end_col {
            return Err(ImplicitIntersectionError::NoCallerAlignedIntersection);
        }
        return Ok(make_single_cell_reference(
            reference.prefix.clone(),
            reference.start_row,
            caller.col,
        ));
    }

    Err(ImplicitIntersectionError::TwoDimensionalReference)
}

fn scalarize_eval_value(
    value: CalcValue,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
    caller: Option<&CallerContext>,
) -> Result<CalcValue, ImplicitIntersectionError> {
    match value.core() {
        CoreValue::Number(_) | CoreValue::Text(_) | CoreValue::Logical(_) | CoreValue::Error(_) => {
            Ok(value)
        }
        CoreValue::Array(array) => top_left_array_value(array),
        CoreValue::Reference(reference) => scalarize_reference(reference.clone(), resolver, caller),
        _ => Err(ImplicitIntersectionError::UnsupportedReferenceSource(
            "unsupported_value",
        )),
    }
}

fn scalarize_reference(
    reference: ReferenceLike,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
    caller: Option<&CallerContext>,
) -> Result<CalcValue, ImplicitIntersectionError> {
    match reference.kind() {
        ReferenceKind::A1 | ReferenceKind::Area => {
            let parsed = parse_a1_reference(reference.target()).ok_or(
                ImplicitIntersectionError::UnsupportedReferenceSource("non_a1_reference"),
            )?;
            let selected = select_reference_cell(&parsed, caller)?;
            let resolved = resolve_eval_value(resolver, &selected)
                .map_err(ImplicitIntersectionError::RefResolution)?;
            scalarize_eval_value(resolved, resolver, caller)
        }
        ReferenceKind::SpillAnchor => {
            let resolved = resolve_eval_value(resolver, &reference)
                .map_err(ImplicitIntersectionError::RefResolution)?;
            scalarize_eval_value(resolved, resolver, caller)
        }
        ReferenceKind::MultiArea => Err(ImplicitIntersectionError::UnsupportedReferenceSource(
            "multi_area_reference",
        )),
        ReferenceKind::ThreeD => Err(ImplicitIntersectionError::UnsupportedReferenceSource(
            "three_d_reference",
        )),
        ReferenceKind::Structured => Err(ImplicitIntersectionError::UnsupportedReferenceSource(
            "structured_reference",
        )),
    }
}

pub fn eval_op_implicit_intersection_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, ImplicitIntersectionError> {
    if args.len() != 1 {
        return Err(ImplicitIntersectionError::ArityMismatch {
            expected: 1,
            actual: args.len(),
        });
    }

    let caller = resolver.caller_context();
    match args[0].core() {
        CoreValue::Reference(reference) => {
            scalarize_reference(reference.clone(), resolver, caller.as_ref())
        }
        CoreValue::Missing | CoreValue::Empty => Err(
            ImplicitIntersectionError::UnsupportedReferenceSource("non_scalarized_call_arg"),
        ),
        _ => scalarize_eval_value(args[0].clone(), resolver, caller.as_ref()),
    }
}

pub fn map_op_implicit_intersection_error_to_ws(
    e: &ImplicitIntersectionError,
) -> WorksheetErrorCode {
    match e {
        ImplicitIntersectionError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        ImplicitIntersectionError::MissingCallerContext => WorksheetErrorCode::Ref,
        ImplicitIntersectionError::UnsupportedReferenceSource(_) => WorksheetErrorCode::Value,
        ImplicitIntersectionError::TwoDimensionalReference => WorksheetErrorCode::Value,
        ImplicitIntersectionError::NoCallerAlignedIntersection => WorksheetErrorCode::Value,
        ImplicitIntersectionError::EmptyArray => WorksheetErrorCode::Value,
        ImplicitIntersectionError::EmptyCellTopLeft => WorksheetErrorCode::Value,
        ImplicitIntersectionError::RefResolution(_) => WorksheetErrorCode::Ref,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::{ReferenceSystemCapabilities, ReferenceSystemProvider};
    use crate::value::ExcelText;
    use std::collections::BTreeMap;

    struct TestResolver {
        caller: Option<CallerContext>,
        resolved: BTreeMap<String, CalcValue>,
    }

    impl ReferenceSystemProvider for TestResolver {
        fn capabilities(&self) -> ReferenceSystemCapabilities {
            ReferenceSystemCapabilities::permissive_local()
        }

        fn dereference(
            &self,
            request: &crate::resolver::ReferenceDereferenceRequest,
        ) -> Result<CalcValue, crate::resolver::ReferenceResolutionError> {
            let reference = &request.reference;
            self.resolved.get(reference.target()).cloned().ok_or(
                crate::resolver::ReferenceResolutionError::UnresolvedReference {
                    target: reference.target().to_string(),
                },
            )
        }

        fn caller_context(&self) -> Option<CallerContext> {
            self.caller.clone()
        }
    }

    fn reference(kind: ReferenceKind, target: &str) -> CalcValue {
        CalcValue::reference(ReferenceLike::new(kind, target.to_string()))
    }

    #[test]
    fn scalar_passthrough_returns_operand() {
        let resolver = TestResolver {
            caller: None,
            resolved: BTreeMap::new(),
        };
        let got = eval_op_implicit_intersection_surface(&[(CalcValue::number(12.0))], &resolver);
        assert_eq!(got, Ok(CalcValue::number(12.0)));
    }

    #[test]
    fn array_payload_selects_top_left() {
        let resolver = TestResolver {
            caller: None,
            resolved: BTreeMap::new(),
        };
        let array = CalcArray::from_rows(vec![
            vec![CalcValue::number(10.0), CalcValue::number(20.0)],
            vec![CalcValue::number(30.0), CalcValue::number(40.0)],
        ])
        .unwrap();
        let got = eval_op_implicit_intersection_surface(&[(CalcValue::array(array))], &resolver);
        assert_eq!(got, Ok(CalcValue::number(10.0)));
    }

    #[test]
    fn single_column_reference_selects_same_row_value() {
        let resolver = TestResolver {
            caller: Some(CallerContext {
                prefix: None,
                row: 2,
                col: 2,
            }),
            resolved: BTreeMap::from([("A2".to_string(), CalcValue::number(20.0))]),
        };
        let got = eval_op_implicit_intersection_surface(
            &[reference(ReferenceKind::Area, "A1:A3")],
            &resolver,
        );
        assert_eq!(got, Ok(CalcValue::number(20.0)));
    }

    #[test]
    fn single_row_reference_selects_same_column_value() {
        let resolver = TestResolver {
            caller: Some(CallerContext {
                prefix: None,
                row: 2,
                col: 2,
            }),
            resolved: BTreeMap::from([("B1".to_string(), CalcValue::number(20.0))]),
        };
        let got = eval_op_implicit_intersection_surface(
            &[reference(ReferenceKind::Area, "A1:C1")],
            &resolver,
        );
        assert_eq!(got, Ok(CalcValue::number(20.0)));
    }

    #[test]
    fn spill_anchor_resolves_then_scalarizes_top_left() {
        let resolver = TestResolver {
            caller: Some(CallerContext {
                prefix: None,
                row: 1,
                col: 1,
            }),
            resolved: BTreeMap::from([(
                "B1#".to_string(),
                CalcValue::array(
                    CalcArray::from_rows(vec![vec![
                        CalcValue::number(1.0),
                        CalcValue::number(2.0),
                    ]])
                    .unwrap(),
                ),
            )]),
        };
        let got = eval_op_implicit_intersection_surface(
            &[reference(ReferenceKind::SpillAnchor, "B1#")],
            &resolver,
        );
        assert_eq!(got, Ok(CalcValue::number(1.0)));
    }

    #[test]
    fn reference_eval_value_uses_caller_relative_selection() {
        let resolver = TestResolver {
            caller: Some(CallerContext {
                prefix: None,
                row: 2,
                col: 3,
            }),
            resolved: BTreeMap::from([("A2".to_string(), CalcValue::number(20.0))]),
        };
        let got =
            eval_op_implicit_intersection_surface(
                &[(CalcValue::reference(ReferenceLike::new(
                    ReferenceKind::Area,
                    "A1:A3".to_string(),
                )))],
                &resolver,
            );
        assert_eq!(got, Ok(CalcValue::number(20.0)));
    }

    #[test]
    fn two_dimensional_reference_is_value_error_on_current_baseline() {
        let resolver = TestResolver {
            caller: Some(CallerContext {
                prefix: None,
                row: 3,
                col: 3,
            }),
            resolved: BTreeMap::new(),
        };
        let got = eval_op_implicit_intersection_surface(
            &[reference(ReferenceKind::Area, "A1:B2")],
            &resolver,
        );
        assert_eq!(got, Err(ImplicitIntersectionError::TwoDimensionalReference));
        assert_eq!(
            map_op_implicit_intersection_error_to_ws(
                &ImplicitIntersectionError::TwoDimensionalReference
            ),
            WorksheetErrorCode::Value
        );
    }

    #[test]
    fn text_passthrough_is_unchanged() {
        let resolver = TestResolver {
            caller: None,
            resolved: BTreeMap::new(),
        };
        let got = eval_op_implicit_intersection_surface(
            &[(CalcValue::text(ExcelText::from_interop_assignment("hello")))],
            &resolver,
        );
        assert_eq!(
            got,
            Ok(CalcValue::text(ExcelText::from_interop_assignment("hello")))
        );
    }
}

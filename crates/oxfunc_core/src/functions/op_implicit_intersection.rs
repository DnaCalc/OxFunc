use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::a1_refs::{
    A1Reference, A1ReferenceNotation, format_relative_target, parse_a1_reference,
};
use crate::resolver::{CallerContext, RefResolutionError, ReferenceResolver, resolve_eval_value};
use crate::value::{
    ArrayCellValue, CallArgValue, EvalArray, EvalValue, ReferenceKind, ReferenceLike,
    WorksheetErrorCode,
};

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
    RefResolution(RefResolutionError),
}

fn scalar_from_array_value(value: &ArrayCellValue) -> Result<EvalValue, ImplicitIntersectionError> {
    match value {
        ArrayCellValue::Number(n) => Ok(EvalValue::Number(*n)),
        ArrayCellValue::Text(t) => Ok(EvalValue::Text(t.clone())),
        ArrayCellValue::Logical(b) => Ok(EvalValue::Logical(*b)),
        ArrayCellValue::Error(code) => Ok(EvalValue::Error(*code)),
        ArrayCellValue::EmptyCell => Err(ImplicitIntersectionError::EmptyCellTopLeft),
    }
}

fn top_left_array_value(array: &EvalArray) -> Result<EvalValue, ImplicitIntersectionError> {
    let cell = array
        .get(0, 0)
        .ok_or(ImplicitIntersectionError::EmptyArray)?;
    scalar_from_array_value(cell)
}

fn make_single_cell_reference(prefix: Option<String>, row: usize, col: usize) -> ReferenceLike {
    ReferenceLike {
        kind: ReferenceKind::A1,
        target: format_relative_target(&A1Reference {
            prefix,
            start_row: row,
            start_col: col,
            end_row: row,
            end_col: col,
            notation: A1ReferenceNotation::Rect,
        })
        .expect("single-cell A1 reference should format"),
    }
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
    value: EvalValue,
    resolver: &impl ReferenceResolver,
    caller: Option<&CallerContext>,
) -> Result<EvalValue, ImplicitIntersectionError> {
    match value {
        EvalValue::Number(_)
        | EvalValue::Text(_)
        | EvalValue::Logical(_)
        | EvalValue::Error(_)
        | EvalValue::Lambda(_) => Ok(value),
        EvalValue::Array(array) => top_left_array_value(&array),
        EvalValue::Reference(reference) => scalarize_reference(reference, resolver, caller),
    }
}

fn scalarize_reference(
    reference: ReferenceLike,
    resolver: &impl ReferenceResolver,
    caller: Option<&CallerContext>,
) -> Result<EvalValue, ImplicitIntersectionError> {
    match reference.kind {
        ReferenceKind::A1 | ReferenceKind::Area => {
            let parsed = parse_a1_reference(&reference.target).ok_or(
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
        ReferenceKind::ThreeD => Err(ImplicitIntersectionError::UnsupportedReferenceSource(
            "three_d_reference",
        )),
        ReferenceKind::Structured => Err(ImplicitIntersectionError::UnsupportedReferenceSource(
            "structured_reference",
        )),
    }
}

pub fn eval_op_implicit_intersection_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, ImplicitIntersectionError> {
    if args.len() != 1 {
        return Err(ImplicitIntersectionError::ArityMismatch {
            expected: 1,
            actual: args.len(),
        });
    }

    let caller = resolver.caller_context();
    match &args[0] {
        CallArgValue::Eval(value) => scalarize_eval_value(value.clone(), resolver, caller.as_ref()),
        CallArgValue::Reference(reference) => {
            scalarize_reference(reference.clone(), resolver, caller.as_ref())
        }
        CallArgValue::MissingArg | CallArgValue::EmptyCell => Err(
            ImplicitIntersectionError::UnsupportedReferenceSource("non_scalarized_call_arg"),
        ),
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
    use crate::resolver::{ReferenceResolver, ResolverCapabilities};
    use crate::value::ExcelText;
    use std::collections::BTreeMap;

    struct TestResolver {
        caller: Option<CallerContext>,
        resolved: BTreeMap<String, EvalValue>,
    }

    impl ReferenceResolver for TestResolver {
        fn capabilities(&self) -> ResolverCapabilities {
            ResolverCapabilities::permissive_local()
        }

        fn resolve_reference(
            &self,
            reference: &ReferenceLike,
        ) -> Result<EvalValue, RefResolutionError> {
            self.resolved.get(&reference.target).cloned().ok_or(
                RefResolutionError::UnresolvedReference {
                    target: reference.target.clone(),
                },
            )
        }

        fn caller_context(&self) -> Option<CallerContext> {
            self.caller.clone()
        }
    }

    fn reference(kind: ReferenceKind, target: &str) -> CallArgValue {
        CallArgValue::Reference(ReferenceLike {
            kind,
            target: target.to_string(),
        })
    }

    #[test]
    fn scalar_passthrough_returns_operand() {
        let resolver = TestResolver {
            caller: None,
            resolved: BTreeMap::new(),
        };
        let got = eval_op_implicit_intersection_surface(
            &[CallArgValue::Eval(EvalValue::Number(12.0))],
            &resolver,
        );
        assert_eq!(got, Ok(EvalValue::Number(12.0)));
    }

    #[test]
    fn array_payload_selects_top_left() {
        let resolver = TestResolver {
            caller: None,
            resolved: BTreeMap::new(),
        };
        let array = EvalArray::from_rows(vec![
            vec![ArrayCellValue::Number(10.0), ArrayCellValue::Number(20.0)],
            vec![ArrayCellValue::Number(30.0), ArrayCellValue::Number(40.0)],
        ])
        .unwrap();
        let got = eval_op_implicit_intersection_surface(
            &[CallArgValue::Eval(EvalValue::Array(array))],
            &resolver,
        );
        assert_eq!(got, Ok(EvalValue::Number(10.0)));
    }

    #[test]
    fn single_column_reference_selects_same_row_value() {
        let resolver = TestResolver {
            caller: Some(CallerContext {
                prefix: None,
                row: 2,
                col: 2,
            }),
            resolved: BTreeMap::from([("A2".to_string(), EvalValue::Number(20.0))]),
        };
        let got = eval_op_implicit_intersection_surface(
            &[reference(ReferenceKind::Area, "A1:A3")],
            &resolver,
        );
        assert_eq!(got, Ok(EvalValue::Number(20.0)));
    }

    #[test]
    fn single_row_reference_selects_same_column_value() {
        let resolver = TestResolver {
            caller: Some(CallerContext {
                prefix: None,
                row: 2,
                col: 2,
            }),
            resolved: BTreeMap::from([("B1".to_string(), EvalValue::Number(20.0))]),
        };
        let got = eval_op_implicit_intersection_surface(
            &[reference(ReferenceKind::Area, "A1:C1")],
            &resolver,
        );
        assert_eq!(got, Ok(EvalValue::Number(20.0)));
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
                EvalValue::Array(
                    EvalArray::from_rows(vec![vec![
                        ArrayCellValue::Number(1.0),
                        ArrayCellValue::Number(2.0),
                    ]])
                    .unwrap(),
                ),
            )]),
        };
        let got = eval_op_implicit_intersection_surface(
            &[reference(ReferenceKind::SpillAnchor, "B1#")],
            &resolver,
        );
        assert_eq!(got, Ok(EvalValue::Number(1.0)));
    }

    #[test]
    fn reference_eval_value_uses_caller_relative_selection() {
        let resolver = TestResolver {
            caller: Some(CallerContext {
                prefix: None,
                row: 2,
                col: 3,
            }),
            resolved: BTreeMap::from([("A2".to_string(), EvalValue::Number(20.0))]),
        };
        let got = eval_op_implicit_intersection_surface(
            &[CallArgValue::Eval(EvalValue::Reference(ReferenceLike {
                kind: ReferenceKind::Area,
                target: "A1:A3".to_string(),
            }))],
            &resolver,
        );
        assert_eq!(got, Ok(EvalValue::Number(20.0)));
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
            &[CallArgValue::Eval(EvalValue::Text(
                ExcelText::from_interop_assignment("hello"),
            ))],
            &resolver,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Text(ExcelText::from_interop_assignment("hello")))
        );
    }
}

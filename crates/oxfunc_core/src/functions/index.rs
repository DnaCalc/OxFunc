use crate::coercion::{CoercionError, coerce_arg_to_number};
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::a1_refs::{
    A1Reference, A1ReferenceNotation, format_relative_target, parse_a1_reference,
};
use crate::resolver::ReferenceResolver;
use crate::value::{
    ArrayCellValue, ArrayShape, CallArgValue, EvalArray, EvalValue, ExcelText, ReferenceKind,
    ReferenceLike, WorksheetErrorCode,
};

pub const INDEX_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.INDEX",
    arity: Arity { min: 2, max: 4 },
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
pub enum IndexEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
    InvalidIndexNumber(f64),
    InvalidAreaNumber(f64),
    OutOfBounds {
        rows: usize,
        cols: usize,
        row: usize,
        col: usize,
    },
    UnsupportedSource(&'static str),
    ArrayPayloadUnavailable,
}

fn coerce_index_number(
    arg: &CallArgValue,
    resolver: &impl ReferenceResolver,
) -> Result<usize, IndexEvalError> {
    let n = coerce_arg_to_number(arg, resolver).map_err(IndexEvalError::Coercion)?;
    if !n.is_finite() || n < 0.0 || n.fract() != 0.0 {
        return Err(IndexEvalError::InvalidIndexNumber(n));
    }
    Ok(n as usize)
}

fn coerce_optional_index_number(
    arg: Option<&CallArgValue>,
    resolver: &impl ReferenceResolver,
    omitted_default: usize,
    blank_default: usize,
) -> Result<usize, IndexEvalError> {
    match arg {
        None => Ok(omitted_default),
        Some(CallArgValue::MissingArg | CallArgValue::EmptyCell) => Ok(blank_default),
        Some(other) => coerce_index_number(other, resolver),
    }
}

fn coerce_area_number(
    arg: Option<&CallArgValue>,
    resolver: &impl ReferenceResolver,
) -> Result<usize, IndexEvalError> {
    match arg {
        None => Ok(1),
        Some(CallArgValue::MissingArg | CallArgValue::EmptyCell) => Ok(1),
        Some(other) => {
            let n = coerce_arg_to_number(other, resolver).map_err(IndexEvalError::Coercion)?;
            if !n.is_finite() || n < 1.0 || n.fract() != 0.0 {
                return Err(IndexEvalError::InvalidAreaNumber(n));
            }
            Ok(n as usize)
        }
    }
}

fn project_reference(base: &ReferenceLike, row: usize, col: usize) -> EvalValue {
    EvalValue::Reference(ReferenceLike {
        kind: base.kind,
        target: format!("{}#INDEX({row},{col})", base.target),
    })
}

fn has_legacy_multi_area_carrier(reference: &ReferenceLike) -> bool {
    !matches!(reference.kind, ReferenceKind::MultiArea)
        && reference.target.trim().starts_with('(')
        && reference.target.trim().ends_with(')')
}

fn parse_reference_areas(
    reference: &ReferenceLike,
) -> Result<Option<Vec<A1Reference>>, IndexEvalError> {
    let parts: Vec<String> = if matches!(reference.kind, ReferenceKind::MultiArea) {
        reference
            .multi_area_targets()
            .ok_or(IndexEvalError::UnsupportedSource(
                "invalid_multi_area_reference",
            ))?
    } else {
        return Ok(None);
    };
    if parts.is_empty() {
        return Ok(None);
    }

    let mut areas = Vec::with_capacity(parts.len());
    for part in parts {
        let Some(area) = parse_a1_reference(&part) else {
            return Ok(None);
        };
        areas.push(area);
    }

    if areas.len() > 1 {
        let first_prefix = areas[0].prefix.clone();
        if areas.iter().any(|area| area.prefix != first_prefix) {
            return Err(IndexEvalError::UnsupportedSource("mixed_sheet_multi_area"));
        }
    }

    Ok(Some(areas))
}

fn select_a1_reference(
    base: &A1Reference,
    row: usize,
    col: usize,
) -> Result<A1Reference, IndexEvalError> {
    let height = base.height();
    let width = base.width();

    if row > height || col > width {
        return Err(IndexEvalError::OutOfBounds {
            rows: height,
            cols: width,
            row,
            col,
        });
    }

    let mut selected = match (row, col) {
        (0, 0) => base.clone(),
        (0, c) => A1Reference {
            prefix: base.prefix.clone(),
            start_row: base.start_row,
            end_row: base.end_row,
            start_col: base.start_col + c - 1,
            end_col: base.start_col + c - 1,
            notation: A1ReferenceNotation::Rect,
        },
        (r, 0) => A1Reference {
            prefix: base.prefix.clone(),
            start_row: base.start_row + r - 1,
            end_row: base.start_row + r - 1,
            start_col: base.start_col,
            end_col: base.end_col,
            notation: A1ReferenceNotation::Rect,
        },
        (r, c) => A1Reference {
            prefix: base.prefix.clone(),
            start_row: base.start_row + r - 1,
            end_row: base.start_row + r - 1,
            start_col: base.start_col + c - 1,
            end_col: base.start_col + c - 1,
            notation: A1ReferenceNotation::Rect,
        },
    };

    selected.notation = if selected.start_row == 1
        && selected.end_row == crate::functions::a1_refs::EXCEL_MAX_ROWS
    {
        A1ReferenceNotation::WholeColumn
    } else if selected.start_col == 1
        && selected.end_col == crate::functions::a1_refs::EXCEL_MAX_COLS
    {
        A1ReferenceNotation::WholeRow
    } else {
        A1ReferenceNotation::Rect
    };

    Ok(selected)
}

fn reference_from_a1(reference: A1Reference) -> Result<EvalValue, IndexEvalError> {
    let target = format_relative_target(&reference)
        .ok_or(IndexEvalError::UnsupportedSource("unformattable_reference"))?;
    Ok(EvalValue::Reference(ReferenceLike {
        kind: if reference.width() == 1 && reference.height() == 1 {
            ReferenceKind::A1
        } else {
            ReferenceKind::Area
        },
        target,
    }))
}

fn cell_to_eval_value(cell: &ArrayCellValue) -> EvalValue {
    match cell {
        ArrayCellValue::Number(n) => EvalValue::Number(*n),
        ArrayCellValue::Text(t) => EvalValue::Text(t.clone()),
        ArrayCellValue::Logical(b) => EvalValue::Logical(*b),
        ArrayCellValue::Error(code) => EvalValue::Error(*code),
        ArrayCellValue::EmptyCell => EvalValue::Text(ExcelText::from_utf16_code_units(Vec::new())),
    }
}

fn slice_array(array: &EvalArray, row: usize, col: usize) -> Result<EvalValue, IndexEvalError> {
    let shape = array.shape();
    if row > shape.rows || col > shape.cols {
        return Err(IndexEvalError::OutOfBounds {
            rows: shape.rows,
            cols: shape.cols,
            row,
            col,
        });
    }

    match (row, col) {
        (0, 0) => Ok(EvalValue::Array(array.clone())),
        (0, c) => {
            let mut cells = Vec::with_capacity(shape.rows);
            for r in 0..shape.rows {
                cells.push(
                    array
                        .get(r, c - 1)
                        .cloned()
                        .expect("column bounds validated"),
                );
            }
            Ok(EvalValue::Array(
                EvalArray::new(
                    ArrayShape {
                        rows: shape.rows,
                        cols: 1,
                    },
                    cells,
                )
                .expect("slice dimensions validated"),
            ))
        }
        (r, 0) => Ok(EvalValue::Array(
            EvalArray::new(
                ArrayShape {
                    rows: 1,
                    cols: shape.cols,
                },
                array
                    .row_slice(r - 1)
                    .expect("row bounds validated")
                    .to_vec(),
            )
            .expect("slice dimensions validated"),
        )),
        (r, c) => Ok(cell_to_eval_value(
            array.get(r - 1, c - 1).expect("cell bounds validated"),
        )),
    }
}

pub fn eval_index_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, IndexEvalError> {
    let argc = args.len();
    if !INDEX_META.arity.accepts(argc) {
        return Err(IndexEvalError::ArityMismatch {
            expected_min: INDEX_META.arity.min,
            expected_max: INDEX_META.arity.max,
            actual: argc,
        });
    }

    let row = coerce_optional_index_number(args.get(1), resolver, 0, 0)?;
    let col = coerce_optional_index_number(args.get(2), resolver, 1, 0)?;
    let area = coerce_area_number(args.get(3), resolver)?;

    match &args[0] {
        CallArgValue::Reference(r) | CallArgValue::Eval(EvalValue::Reference(r)) => {
            if let Some(areas) = parse_reference_areas(r)? {
                let Some(selected_area) = areas.get(area - 1) else {
                    return Err(IndexEvalError::InvalidAreaNumber(area as f64));
                };
                reference_from_a1(select_a1_reference(selected_area, row, col)?)
            } else if has_legacy_multi_area_carrier(r) {
                Err(IndexEvalError::UnsupportedSource(
                    "legacy_multi_area_carrier_removed",
                ))
            } else if area != 1 {
                Err(IndexEvalError::InvalidAreaNumber(area as f64))
            } else if let Some(parsed) = parse_a1_reference(&r.target) {
                reference_from_a1(select_a1_reference(&parsed, row, col)?)
            } else if row == 0 && col == 0 {
                Ok(EvalValue::Reference(r.clone()))
            } else {
                Ok(project_reference(r, row, col))
            }
        }
        CallArgValue::Eval(EvalValue::Array(array)) => slice_array(array, row, col),
        CallArgValue::Eval(_) => Err(IndexEvalError::UnsupportedSource("non_array_non_reference")),
        CallArgValue::MissingArg => Err(IndexEvalError::UnsupportedSource("missing_arg_source")),
        CallArgValue::EmptyCell => Err(IndexEvalError::UnsupportedSource("empty_cell_source")),
    }
}

pub fn map_index_error_to_ws(e: &IndexEvalError) -> WorksheetErrorCode {
    match e {
        IndexEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        IndexEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        IndexEvalError::InvalidIndexNumber(_) => WorksheetErrorCode::Value,
        IndexEvalError::InvalidAreaNumber(_) => WorksheetErrorCode::Ref,
        IndexEvalError::OutOfBounds { .. } => WorksheetErrorCode::Ref,
        IndexEvalError::UnsupportedSource(_) => WorksheetErrorCode::Value,
        IndexEvalError::ArrayPayloadUnavailable => WorksheetErrorCode::Calc,
        IndexEvalError::Coercion(_) => WorksheetErrorCode::Value,
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
    fn eval_index_reference_projection_projects_actual_a1_target() {
        let args = [
            CallArgValue::Reference(ReferenceLike {
                kind: ReferenceKind::Area,
                target: "A1:C3".to_string(),
            }),
            CallArgValue::Eval(EvalValue::Number(2.0)),
            CallArgValue::Eval(EvalValue::Number(1.0)),
        ];
        let got = eval_index_surface(&args, &NoResolver);
        assert_eq!(
            got,
            Ok(EvalValue::Reference(ReferenceLike {
                kind: ReferenceKind::A1,
                target: "A2".to_string(),
            }))
        );
    }

    #[test]
    fn eval_index_array_position_returns_payload_value() {
        let args = [
            CallArgValue::Eval(EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Number(1.0), ArrayCellValue::Number(2.0)],
                    vec![ArrayCellValue::Number(3.0), ArrayCellValue::Number(4.0)],
                    vec![ArrayCellValue::Number(5.0), ArrayCellValue::Number(6.0)],
                ])
                .unwrap(),
            )),
            CallArgValue::Eval(EvalValue::Number(1.0)),
            CallArgValue::Eval(EvalValue::Number(1.0)),
        ];
        let got = eval_index_surface(&args, &NoResolver);
        assert_eq!(got, Ok(EvalValue::Number(1.0)));
    }

    #[test]
    fn eval_index_array_bounds_checked() {
        let args = [
            CallArgValue::Eval(EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Number(1.0), ArrayCellValue::Number(2.0)],
                    vec![ArrayCellValue::Number(3.0), ArrayCellValue::Number(4.0)],
                ])
                .unwrap(),
            )),
            CallArgValue::Eval(EvalValue::Number(3.0)),
            CallArgValue::Eval(EvalValue::Number(1.0)),
        ];
        let got = eval_index_surface(&args, &NoResolver);
        assert_eq!(
            got,
            Err(IndexEvalError::OutOfBounds {
                rows: 2,
                cols: 2,
                row: 3,
                col: 1,
            })
        );
    }

    #[test]
    fn eval_index_array_zero_row_returns_column_array() {
        let args = [
            CallArgValue::Eval(EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Number(1.0), ArrayCellValue::Number(2.0)],
                    vec![ArrayCellValue::Number(3.0), ArrayCellValue::Number(4.0)],
                ])
                .unwrap(),
            )),
            CallArgValue::Eval(EvalValue::Number(0.0)),
            CallArgValue::Eval(EvalValue::Number(2.0)),
        ];
        let got = eval_index_surface(&args, &NoResolver);
        assert_eq!(
            got,
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Number(2.0)],
                    vec![ArrayCellValue::Number(4.0)],
                ])
                .unwrap()
            ))
        );
    }

    #[test]
    fn eval_index_invalid_area_num_rejected() {
        let args = [
            CallArgValue::Reference(ReferenceLike {
                kind: ReferenceKind::Area,
                target: "A1:C3".to_string(),
            }),
            CallArgValue::Eval(EvalValue::Number(1.0)),
            CallArgValue::Eval(EvalValue::Number(1.0)),
            CallArgValue::Eval(EvalValue::Number(2.0)),
        ];
        let got = eval_index_surface(&args, &NoResolver);
        assert_eq!(got, Err(IndexEvalError::InvalidAreaNumber(2.0)));
    }

    #[test]
    fn eval_index_missing_row_and_col_follow_excel_defaults() {
        let args = [
            CallArgValue::Reference(ReferenceLike {
                kind: ReferenceKind::Area,
                target: "B1:C2".to_string(),
            }),
            CallArgValue::MissingArg,
            CallArgValue::Eval(EvalValue::Number(2.0)),
        ];
        let got = eval_index_surface(&args, &NoResolver);
        assert_eq!(
            got,
            Ok(EvalValue::Reference(ReferenceLike {
                kind: ReferenceKind::Area,
                target: "C1:C2".to_string(),
            }))
        );

        let args = [
            CallArgValue::Reference(ReferenceLike {
                kind: ReferenceKind::Area,
                target: "B1:C2".to_string(),
            }),
            CallArgValue::Eval(EvalValue::Number(2.0)),
            CallArgValue::MissingArg,
        ];
        let got = eval_index_surface(&args, &NoResolver);
        assert_eq!(
            got,
            Ok(EvalValue::Reference(ReferenceLike {
                kind: ReferenceKind::Area,
                target: "B2:C2".to_string(),
            }))
        );
    }

    #[test]
    fn eval_index_multi_area_reference_selects_area_num() {
        let args = [
            CallArgValue::Reference(ReferenceLike {
                kind: ReferenceKind::MultiArea,
                target: "(A1:A2,G1:G2)".to_string(),
            }),
            CallArgValue::Eval(EvalValue::Number(2.0)),
            CallArgValue::Eval(EvalValue::Number(1.0)),
            CallArgValue::Eval(EvalValue::Number(2.0)),
        ];
        let got = eval_index_surface(&args, &NoResolver);
        assert_eq!(
            got,
            Ok(EvalValue::Reference(ReferenceLike {
                kind: ReferenceKind::A1,
                target: "G2".to_string(),
            }))
        );
    }

    #[test]
    fn eval_index_mixed_sheet_multi_area_is_rejected() {
        let args = [
            CallArgValue::Reference(ReferenceLike {
                kind: ReferenceKind::MultiArea,
                target: "(Sheet1!A1:A2,Sheet2!G1:G2)".to_string(),
            }),
            CallArgValue::Eval(EvalValue::Number(1.0)),
            CallArgValue::Eval(EvalValue::Number(1.0)),
        ];
        let got = eval_index_surface(&args, &NoResolver);
        assert_eq!(
            got,
            Err(IndexEvalError::UnsupportedSource("mixed_sheet_multi_area"))
        );
    }

    #[test]
    fn eval_index_rejects_legacy_parenthesized_area_carrier() {
        let args = [
            CallArgValue::Reference(ReferenceLike {
                kind: ReferenceKind::Area,
                target: "(A1:A2,G1:G2)".to_string(),
            }),
            CallArgValue::Eval(EvalValue::Number(2.0)),
            CallArgValue::Eval(EvalValue::Number(1.0)),
            CallArgValue::Eval(EvalValue::Number(2.0)),
        ];
        let got = eval_index_surface(&args, &NoResolver);
        assert_eq!(
            got,
            Err(IndexEvalError::UnsupportedSource(
                "legacy_multi_area_carrier_removed"
            ))
        );
    }
}

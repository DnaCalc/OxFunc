use crate::coercion::{CoercionError, coerce_eval_to_number};
use crate::resolver::{
    RefResolutionError, ReferenceResolver, ResolverCapabilities, resolve_eval_value,
};
use crate::value::{
    ArrayCellValue, ArrayShape, CallArgValue, EvalArray, EvalValue, ReferenceKind, ReferenceLike,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryNumericCoercionLiftProfile {
    ScalarOnly,
    ScalarOrArrayElementwise,
}

fn normalize_prepared_eval(value: EvalValue) -> PreparedArgValue {
    match value {
        EvalValue::Array(array) if array.shape().rows == 1 && array.shape().cols == 1 => {
            match array.get(0, 0) {
                Some(ArrayCellValue::EmptyCell) => PreparedArgValue::EmptyCell,
                Some(cell) => prepared_from_array_cell(cell),
                None => PreparedArgValue::Eval(EvalValue::Array(array)),
            }
        }
        other => PreparedArgValue::Eval(other),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PreparedArgValue {
    Eval(EvalValue),
    MissingArg,
    EmptyCell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AggregateArrayProvenance {
    DirectArrayLiteral,
    OpaqueArrayValue,
    ReferenceDerived,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AggregateArgOrigin {
    DirectScalar,
    ArrayLike(AggregateArrayProvenance),
}

#[derive(Debug, Clone, PartialEq)]
pub struct AggregatePreparedValue {
    pub origin: AggregateArgOrigin,
    pub value: PreparedArgValue,
}

fn prepared_from_array_cell(cell: &ArrayCellValue) -> PreparedArgValue {
    match cell {
        ArrayCellValue::Number(n) => PreparedArgValue::Eval(EvalValue::Number(*n)),
        ArrayCellValue::Text(t) => PreparedArgValue::Eval(EvalValue::Text(t.clone())),
        ArrayCellValue::Logical(b) => PreparedArgValue::Eval(EvalValue::Logical(*b)),
        ArrayCellValue::Error(code) => PreparedArgValue::Eval(EvalValue::Error(*code)),
        ArrayCellValue::EmptyCell => PreparedArgValue::EmptyCell,
    }
}

pub fn expand_aggregate_array_with_provenance(
    array: &EvalArray,
    provenance: AggregateArrayProvenance,
) -> Vec<AggregatePreparedValue> {
    array
        .iter_row_major()
        .map(prepared_from_array_cell)
        .map(|value| AggregatePreparedValue {
            origin: AggregateArgOrigin::ArrayLike(provenance),
            value,
        })
        .collect()
}

fn expand_resolved_eval_value(value: &EvalValue) -> Vec<PreparedArgValue> {
    match value {
        EvalValue::Array(array) => array
            .iter_row_major()
            .map(prepared_from_array_cell)
            .collect(),
        _ => vec![PreparedArgValue::Eval(value.clone())],
    }
}

fn expand_lookup_eval_value(value: &EvalValue) -> Result<Vec<PreparedArgValue>, CoercionError> {
    match value {
        EvalValue::Array(array) => {
            let shape = array.shape();
            if shape.rows > 1 && shape.cols > 1 {
                return Err(CoercionError::UnsupportedValueKind("two_dimensional_array"));
            }
            Ok(array
                .iter_row_major()
                .map(prepared_from_array_cell)
                .collect())
        }
        _ => Ok(vec![PreparedArgValue::Eval(value.clone())]),
    }
}

fn resolve_eval_references(
    value: &EvalValue,
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, CoercionError> {
    match value {
        EvalValue::Reference(r) => {
            let resolved = resolve_eval_value(resolver, r).map_err(CoercionError::RefResolution)?;
            resolve_eval_references(&resolved, resolver)
        }
        _ => Ok(value.clone()),
    }
}

pub fn prepare_arg_values_only(
    arg: &CallArgValue,
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<PreparedArgValue, CoercionError> {
    match arg {
        CallArgValue::Eval(v) => Ok(normalize_prepared_eval(resolve_eval_references(
            v, resolver,
        )?)),
        CallArgValue::MissingArg => Ok(PreparedArgValue::MissingArg),
        CallArgValue::EmptyCell => Ok(PreparedArgValue::EmptyCell),
        CallArgValue::Reference(r) => {
            let resolved = resolve_eval_value(resolver, r).map_err(CoercionError::RefResolution)?;
            Ok(normalize_prepared_eval(resolve_eval_references(
                &resolved, resolver,
            )?))
        }
    }
}

pub fn prepare_args_values_only(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<Vec<PreparedArgValue>, CoercionError> {
    args.iter()
        .map(|arg| prepare_arg_values_only(arg, resolver))
        .collect()
}

pub fn expand_arg_values_only(
    arg: &CallArgValue,
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<Vec<PreparedArgValue>, CoercionError> {
    match arg {
        CallArgValue::Eval(v) => Ok(expand_resolved_eval_value(&resolve_eval_references(
            v, resolver,
        )?)),
        CallArgValue::MissingArg => Ok(vec![PreparedArgValue::MissingArg]),
        CallArgValue::EmptyCell => Ok(vec![PreparedArgValue::EmptyCell]),
        CallArgValue::Reference(r) => {
            let resolved = resolve_eval_value(resolver, r).map_err(CoercionError::RefResolution)?;
            Ok(expand_resolved_eval_value(&resolve_eval_references(
                &resolved, resolver,
            )?))
        }
    }
}

pub fn expand_lookup_vector_arg(
    arg: &CallArgValue,
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<Vec<PreparedArgValue>, CoercionError> {
    match arg {
        CallArgValue::Eval(v) => expand_lookup_eval_value(&resolve_eval_references(v, resolver)?),
        CallArgValue::MissingArg => Ok(vec![PreparedArgValue::MissingArg]),
        CallArgValue::EmptyCell => Ok(vec![PreparedArgValue::EmptyCell]),
        CallArgValue::Reference(r) => {
            let resolved = resolve_eval_value(resolver, r).map_err(CoercionError::RefResolution)?;
            expand_lookup_eval_value(&resolve_eval_references(&resolved, resolver)?)
        }
    }
}

pub fn expand_aggregate_arg(
    arg: &CallArgValue,
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<Vec<AggregatePreparedValue>, CoercionError> {
    match arg {
        CallArgValue::Reference(r) => {
            let resolved = resolve_eval_value(resolver, r).map_err(CoercionError::RefResolution)?;
            match resolve_eval_references(&resolved, resolver)? {
                EvalValue::Array(array) => Ok(expand_aggregate_array_with_provenance(
                    &array,
                    AggregateArrayProvenance::ReferenceDerived,
                )),
                value => Ok(vec![AggregatePreparedValue {
                    origin: AggregateArgOrigin::ArrayLike(
                        AggregateArrayProvenance::ReferenceDerived,
                    ),
                    value: PreparedArgValue::Eval(value),
                }]),
            }
        }
        CallArgValue::Eval(EvalValue::Reference(r)) => {
            let resolved = resolve_eval_value(resolver, r).map_err(CoercionError::RefResolution)?;
            match resolve_eval_references(&resolved, resolver)? {
                EvalValue::Array(array) => Ok(expand_aggregate_array_with_provenance(
                    &array,
                    AggregateArrayProvenance::ReferenceDerived,
                )),
                value => Ok(vec![AggregatePreparedValue {
                    origin: AggregateArgOrigin::ArrayLike(
                        AggregateArrayProvenance::ReferenceDerived,
                    ),
                    value: PreparedArgValue::Eval(value),
                }]),
            }
        }
        CallArgValue::Eval(EvalValue::Array(array)) => Ok(expand_aggregate_array_with_provenance(
            array,
            AggregateArrayProvenance::OpaqueArrayValue,
        )),
        other => Ok(expand_arg_values_only(other, resolver)?
            .into_iter()
            .map(|value| AggregatePreparedValue {
                origin: AggregateArgOrigin::DirectScalar,
                value,
            })
            .collect()),
    }
}

pub fn run_values_only_prepared<Out, E>(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
    on_prepared: impl FnOnce(&[PreparedArgValue]) -> Result<Out, E>,
    map_preparation_error: impl FnOnce(CoercionError) -> E,
) -> Result<Out, E> {
    let prepared = prepare_args_values_only(args, resolver).map_err(map_preparation_error)?;
    on_prepared(&prepared)
}

pub fn map_values_only_prepared<Out>(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
    on_prepared_arg: impl Fn(&PreparedArgValue) -> Out,
    on_preparation_error: impl Fn(CoercionError) -> Out,
) -> Vec<Out> {
    args.iter()
        .map(|arg| match prepare_arg_values_only(arg, resolver) {
            Ok(prepared) => on_prepared_arg(&prepared),
            Err(e) => on_preparation_error(e),
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq)]
pub enum BroadcastPreparedPair {
    Pair(PreparedArgValue, PreparedArgValue),
    MissingCoordinate,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BroadcastPreparedGroup {
    Values(Vec<PreparedArgValue>),
    MissingCoordinate,
}

fn prepared_shape(value: &PreparedArgValue) -> ArrayShape {
    match value {
        PreparedArgValue::Eval(EvalValue::Array(array)) => array.shape(),
        _ => ArrayShape { rows: 1, cols: 1 },
    }
}

fn prepared_broadcast_value_at(
    value: &PreparedArgValue,
    row: usize,
    col: usize,
) -> Option<PreparedArgValue> {
    match value {
        PreparedArgValue::Eval(EvalValue::Array(array)) => {
            let shape = array.shape();
            let source_row = if shape.rows == 1 {
                0
            } else if row < shape.rows {
                row
            } else {
                return None;
            };
            let source_col = if shape.cols == 1 {
                0
            } else if col < shape.cols {
                col
            } else {
                return None;
            };
            array
                .get(source_row, source_col)
                .map(prepared_from_array_cell)
        }
        scalar => Some(scalar.clone()),
    }
}

pub fn expand_binary_broadcast_grid(
    lhs: &PreparedArgValue,
    rhs: &PreparedArgValue,
) -> Option<(ArrayShape, Vec<BroadcastPreparedPair>)> {
    let lhs_shape = prepared_shape(lhs);
    let rhs_shape = prepared_shape(rhs);
    if lhs_shape == (ArrayShape { rows: 1, cols: 1 })
        && rhs_shape == (ArrayShape { rows: 1, cols: 1 })
    {
        return None;
    }

    let shape = ArrayShape {
        rows: lhs_shape.rows.max(rhs_shape.rows),
        cols: lhs_shape.cols.max(rhs_shape.cols),
    };
    let mut cells = Vec::with_capacity(shape.cell_count());
    for row in 0..shape.rows {
        for col in 0..shape.cols {
            match (
                prepared_broadcast_value_at(lhs, row, col),
                prepared_broadcast_value_at(rhs, row, col),
            ) {
                (Some(lhs_value), Some(rhs_value)) => {
                    cells.push(BroadcastPreparedPair::Pair(lhs_value, rhs_value))
                }
                _ => cells.push(BroadcastPreparedPair::MissingCoordinate),
            }
        }
    }

    Some((shape, cells))
}

pub fn expand_prepared_broadcast_grid(
    args: &[PreparedArgValue],
) -> Option<(ArrayShape, Vec<BroadcastPreparedGroup>)> {
    let mut shape = ArrayShape { rows: 1, cols: 1 };
    let mut has_array = false;
    for arg in args {
        let arg_shape = prepared_shape(arg);
        if arg_shape != (ArrayShape { rows: 1, cols: 1 }) {
            has_array = true;
        }
        shape.rows = shape.rows.max(arg_shape.rows);
        shape.cols = shape.cols.max(arg_shape.cols);
    }
    if !has_array {
        return None;
    }

    let mut cells = Vec::with_capacity(shape.cell_count());
    for row in 0..shape.rows {
        for col in 0..shape.cols {
            let mut values = Vec::with_capacity(args.len());
            let mut missing = false;
            for arg in args {
                match prepared_broadcast_value_at(arg, row, col) {
                    Some(value) => values.push(value),
                    None => {
                        missing = true;
                        break;
                    }
                }
            }
            if missing {
                cells.push(BroadcastPreparedGroup::MissingCoordinate);
            } else {
                cells.push(BroadcastPreparedGroup::Values(values));
            }
        }
    }

    Some((shape, cells))
}

struct NoReferenceResolver;

impl ReferenceResolver for NoReferenceResolver {
    fn capabilities(&self) -> ResolverCapabilities {
        ResolverCapabilities {
            allow_eval_time_deref: false,
            allow_three_d_refs: false,
            allow_structured_refs: false,
            allow_spill_anchor_refs: false,
            allow_external_refs: false,
        }
    }

    fn resolve_reference(
        &self,
        reference: &ReferenceLike,
    ) -> Result<EvalValue, RefResolutionError> {
        Err(RefResolutionError::CapabilityDenied {
            kind: match reference.kind {
                ReferenceKind::A1 => ReferenceKind::A1,
                ReferenceKind::Area => ReferenceKind::Area,
                ReferenceKind::MultiArea => ReferenceKind::MultiArea,
                ReferenceKind::ThreeD => ReferenceKind::ThreeD,
                ReferenceKind::Structured => ReferenceKind::Structured,
                ReferenceKind::SpillAnchor => ReferenceKind::SpillAnchor,
            },
            capability: "values_only_pre_adapter_invariant",
        })
    }
}

pub fn coerce_prepared_to_number(arg: &PreparedArgValue) -> Result<f64, CoercionError> {
    match arg {
        PreparedArgValue::Eval(v) => coerce_eval_to_number(v, &NoReferenceResolver),
        PreparedArgValue::MissingArg => Err(CoercionError::MissingArg),
        PreparedArgValue::EmptyCell => Err(CoercionError::EmptyCell),
    }
}

pub fn coerce_prepared_to_text(
    arg: &PreparedArgValue,
) -> Result<crate::value::ExcelText, CoercionError> {
    use crate::value::ExcelText;

    match arg {
        PreparedArgValue::Eval(EvalValue::Text(t)) => Ok(t.clone()),
        PreparedArgValue::Eval(EvalValue::Number(n)) => Ok(ExcelText::from_utf16_code_units(
            format!("{n}").encode_utf16().collect(),
        )),
        PreparedArgValue::Eval(EvalValue::Logical(b)) => Ok(ExcelText::from_utf16_code_units(
            if *b { "TRUE" } else { "FALSE" }.encode_utf16().collect(),
        )),
        PreparedArgValue::Eval(EvalValue::Error(code)) => Err(CoercionError::WorksheetError(*code)),
        PreparedArgValue::Eval(EvalValue::Array(_)) => {
            Err(CoercionError::UnsupportedValueKind("array"))
        }
        PreparedArgValue::Eval(EvalValue::Reference(_)) => Err(CoercionError::RefResolution(
            RefResolutionError::EvalTimeDerefNotAllowed,
        )),
        PreparedArgValue::Eval(EvalValue::Lambda(_)) => {
            Err(CoercionError::UnsupportedValueKind("lambda_value"))
        }
        PreparedArgValue::MissingArg => Ok(ExcelText::from_utf16_code_units(Vec::new())),
        PreparedArgValue::EmptyCell => Ok(ExcelText::from_utf16_code_units(Vec::new())),
    }
}

pub fn apply_unary_numeric_scalar_prepared(
    arg: &PreparedArgValue,
    kernel: fn(f64) -> f64,
) -> Result<f64, CoercionError> {
    let n = coerce_prepared_to_number(arg)?;
    Ok(kernel(n))
}

pub fn apply_unary_numeric_array_map_prepared(
    args: &[PreparedArgValue],
    kernel: fn(f64) -> f64,
) -> Vec<Result<f64, CoercionError>> {
    args.iter()
        .map(|arg| apply_unary_numeric_scalar_prepared(arg, kernel))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::{RefResolutionError, ResolverCapabilities};
    use crate::value::{EvalArray, ExcelText, ReferenceKind, ReferenceLike, WorksheetErrorCode};
    use std::collections::BTreeMap;

    struct MockResolver {
        caps: ResolverCapabilities,
        resolved_value: Option<EvalValue>,
        by_target: BTreeMap<String, EvalValue>,
    }

    impl ReferenceResolver for MockResolver {
        fn capabilities(&self) -> ResolverCapabilities {
            self.caps
        }

        fn resolve_reference(
            &self,
            reference: &ReferenceLike,
        ) -> Result<EvalValue, RefResolutionError> {
            if let Some(value) = self.by_target.get(&reference.target) {
                return Ok(value.clone());
            }
            self.resolved_value
                .clone()
                .ok_or(RefResolutionError::UnresolvedReference {
                    target: reference.target.clone(),
                })
        }
    }

    fn resolver_with(value: EvalValue) -> MockResolver {
        MockResolver {
            caps: ResolverCapabilities::permissive_local(),
            resolved_value: Some(value),
            by_target: BTreeMap::new(),
        }
    }

    #[test]
    fn prepare_values_only_dereferences_reference_arg() {
        let arg = CallArgValue::Reference(ReferenceLike {
            kind: ReferenceKind::A1,
            target: "A1".to_string(),
        });
        let prepared = prepare_arg_values_only(&arg, &resolver_with(EvalValue::Number(3.0)));
        assert_eq!(prepared, Ok(PreparedArgValue::Eval(EvalValue::Number(3.0))));
    }

    #[test]
    fn prepare_values_only_normalizes_single_blank_area_to_empty_cell() {
        let arg = CallArgValue::Reference(ReferenceLike {
            kind: ReferenceKind::A1,
            target: "A1".to_string(),
        });
        let prepared = prepare_arg_values_only(
            &arg,
            &resolver_with(EvalValue::Array(
                EvalArray::from_rows(vec![vec![ArrayCellValue::EmptyCell]]).unwrap(),
            )),
        );
        assert_eq!(prepared, Ok(PreparedArgValue::EmptyCell));
    }

    #[test]
    fn prepare_values_only_preserves_missing_and_empty() {
        assert_eq!(
            prepare_arg_values_only(
                &CallArgValue::MissingArg,
                &resolver_with(EvalValue::Number(1.0))
            ),
            Ok(PreparedArgValue::MissingArg)
        );
        assert_eq!(
            prepare_arg_values_only(
                &CallArgValue::EmptyCell,
                &resolver_with(EvalValue::Number(1.0))
            ),
            Ok(PreparedArgValue::EmptyCell)
        );
    }

    #[test]
    fn prepare_values_only_materializes_multi_area_reference_into_row_vector() {
        let mut by_target = BTreeMap::new();
        by_target.insert(
            "Alpha!A1:A2".to_string(),
            EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Number(7.0)],
                    vec![ArrayCellValue::Number(11.0)],
                ])
                .unwrap(),
            ),
        );
        by_target.insert("Alpha!B2".to_string(), EvalValue::Number(13.0));
        let resolver = MockResolver {
            caps: ResolverCapabilities::permissive_local(),
            resolved_value: None,
            by_target,
        };
        let arg = CallArgValue::Reference(ReferenceLike::new(
            ReferenceKind::MultiArea,
            "(Alpha!A1:A2,Alpha!B2)",
        ));

        let prepared = prepare_arg_values_only(&arg, &resolver);
        assert_eq!(
            prepared,
            Ok(PreparedArgValue::Eval(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Number(7.0),
                    ArrayCellValue::Number(11.0),
                    ArrayCellValue::Number(13.0),
                ]])
                .unwrap()
            )))
        );
    }

    #[test]
    fn expand_lookup_vector_materializes_multi_area_reference_in_member_order() {
        let mut by_target = BTreeMap::new();
        by_target.insert(
            "A1:A2".to_string(),
            EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Number(1.0)],
                    vec![ArrayCellValue::Number(2.0)],
                ])
                .unwrap(),
            ),
        );
        by_target.insert("C1".to_string(), EvalValue::Number(3.0));
        let resolver = MockResolver {
            caps: ResolverCapabilities::permissive_local(),
            resolved_value: None,
            by_target,
        };
        let arg =
            CallArgValue::Reference(ReferenceLike::new(ReferenceKind::MultiArea, "(A1:A2,C1)"));

        let prepared = expand_lookup_vector_arg(&arg, &resolver);
        assert_eq!(
            prepared,
            Ok(vec![
                PreparedArgValue::Eval(EvalValue::Number(1.0)),
                PreparedArgValue::Eval(EvalValue::Number(2.0)),
                PreparedArgValue::Eval(EvalValue::Number(3.0)),
            ])
        );
    }

    #[test]
    fn prepared_coercion_numeric_text_and_error_paths() {
        let text = PreparedArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
            "2".encode_utf16().collect(),
        )));
        assert_eq!(coerce_prepared_to_number(&text), Ok(2.0));

        let err = PreparedArgValue::Eval(EvalValue::Error(WorksheetErrorCode::Value));
        assert_eq!(
            coerce_prepared_to_number(&err),
            Err(CoercionError::WorksheetError(WorksheetErrorCode::Value))
        );
    }

    #[test]
    fn prepared_text_coercion_formats_scalars_and_blanks() {
        let number = PreparedArgValue::Eval(EvalValue::Number(2.5));
        assert_eq!(
            coerce_prepared_to_text(&number),
            Ok(ExcelText::from_utf16_code_units(
                "2.5".encode_utf16().collect()
            ))
        );

        let blank = PreparedArgValue::EmptyCell;
        assert_eq!(
            coerce_prepared_to_text(&blank),
            Ok(ExcelText::from_utf16_code_units(Vec::new()))
        );
    }

    #[test]
    fn prepared_coercion_rejects_reference_if_invariant_broken() {
        let prepared = PreparedArgValue::Eval(EvalValue::Reference(ReferenceLike {
            kind: ReferenceKind::A1,
            target: "A1".to_string(),
        }));
        let got = coerce_prepared_to_number(&prepared);
        assert_eq!(
            got,
            Err(CoercionError::RefResolution(
                RefResolutionError::EvalTimeDerefNotAllowed
            ))
        );
    }

    #[test]
    fn unary_numeric_array_map_prepared_preserves_per_element_results() {
        let args = vec![
            PreparedArgValue::Eval(EvalValue::Number(-2.0)),
            PreparedArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
                "asd".encode_utf16().collect(),
            ))),
            PreparedArgValue::Eval(EvalValue::Logical(true)),
        ];
        let got = apply_unary_numeric_array_map_prepared(&args, f64::abs);
        assert_eq!(got.len(), 3);
        assert_eq!(got[0], Ok(2.0));
        assert_eq!(got[2], Ok(1.0));
        assert_eq!(
            got[1],
            Err(CoercionError::NonNumericText("asd".to_string()))
        );
    }

    #[test]
    fn run_values_only_prepared_passes_prepared_args_to_adapter() {
        let args = [CallArgValue::Eval(EvalValue::Number(2.0))];
        let got = run_values_only_prepared(
            &args,
            &resolver_with(EvalValue::Number(0.0)),
            |prepared| Ok::<f64, CoercionError>(coerce_prepared_to_number(&prepared[0])?),
            |e| e,
        );
        assert_eq!(got, Ok(2.0));
    }

    #[test]
    fn map_values_only_prepared_maps_preparation_errors_per_arg() {
        let args = vec![
            CallArgValue::Reference(ReferenceLike {
                kind: ReferenceKind::A1,
                target: "A1".to_string(),
            }),
            CallArgValue::Eval(EvalValue::Number(2.0)),
        ];
        let resolver = MockResolver {
            caps: ResolverCapabilities {
                allow_eval_time_deref: false,
                allow_three_d_refs: false,
                allow_structured_refs: false,
                allow_spill_anchor_refs: false,
                allow_external_refs: false,
            },
            resolved_value: None,
            by_target: BTreeMap::new(),
        };

        let got = map_values_only_prepared(
            &args,
            &resolver,
            |_| "ok".to_string(),
            |e| format!("err:{e:?}"),
        );
        assert_eq!(got.len(), 2);
        assert!(got[0].starts_with("err:"));
        assert_eq!(got[1], "ok");
    }

    #[test]
    fn expand_arg_values_only_flattens_array_payloads() {
        let arg = CallArgValue::Eval(EvalValue::Array(
            EvalArray::from_rows(vec![
                vec![ArrayCellValue::Number(1.0), ArrayCellValue::EmptyCell],
                vec![
                    ArrayCellValue::Text(ExcelText::from_utf16_code_units(
                        "x".encode_utf16().collect(),
                    )),
                    ArrayCellValue::Logical(true),
                ],
            ])
            .unwrap(),
        ));
        let got = expand_arg_values_only(&arg, &resolver_with(EvalValue::Number(0.0))).unwrap();
        assert_eq!(
            got,
            vec![
                PreparedArgValue::Eval(EvalValue::Number(1.0)),
                PreparedArgValue::EmptyCell,
                PreparedArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
                    "x".encode_utf16().collect(),
                ))),
                PreparedArgValue::Eval(EvalValue::Logical(true)),
            ]
        );
    }

    #[test]
    fn expand_aggregate_arg_marks_reference_derived_values() {
        let arg = CallArgValue::Reference(ReferenceLike {
            kind: ReferenceKind::Area,
            target: "A1:A2".to_string(),
        });
        let got = expand_aggregate_arg(
            &arg,
            &resolver_with(EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Number(1.0)],
                    vec![ArrayCellValue::Text(ExcelText::from_utf16_code_units(
                        "2".encode_utf16().collect(),
                    ))],
                ])
                .unwrap(),
            )),
        )
        .unwrap();
        assert_eq!(got.len(), 2);
        assert!(got.iter().all(|item| item.origin
            == AggregateArgOrigin::ArrayLike(AggregateArrayProvenance::ReferenceDerived)));
    }

    #[test]
    fn expand_aggregate_arg_marks_eval_arrays_as_opaque_array_values() {
        let got = expand_aggregate_arg(
            &CallArgValue::Eval(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Text(ExcelText::from_utf16_code_units(
                        "2".encode_utf16().collect(),
                    )),
                    ArrayCellValue::Logical(true),
                ]])
                .unwrap(),
            )),
            &resolver_with(EvalValue::Number(0.0)),
        )
        .unwrap();
        assert_eq!(got.len(), 2);
        assert!(got.iter().all(|item| item.origin
            == AggregateArgOrigin::ArrayLike(AggregateArrayProvenance::OpaqueArrayValue)));
    }

    #[test]
    fn expand_aggregate_array_with_provenance_marks_direct_array_literal() {
        let array = EvalArray::from_rows(vec![vec![
            ArrayCellValue::Text(ExcelText::from_utf16_code_units(
                "2".encode_utf16().collect(),
            )),
            ArrayCellValue::Logical(true),
        ]])
        .unwrap();

        let got = expand_aggregate_array_with_provenance(
            &array,
            AggregateArrayProvenance::DirectArrayLiteral,
        );

        assert_eq!(got.len(), 2);
        assert!(got.iter().all(|item| item.origin
            == AggregateArgOrigin::ArrayLike(AggregateArrayProvenance::DirectArrayLiteral)));
    }

    #[test]
    fn expand_lookup_vector_arg_rejects_two_dimensional_array() {
        let arg = CallArgValue::Eval(EvalValue::Array(
            EvalArray::from_rows(vec![
                vec![ArrayCellValue::Number(1.0), ArrayCellValue::Number(2.0)],
                vec![ArrayCellValue::Number(3.0), ArrayCellValue::Number(4.0)],
            ])
            .unwrap(),
        ));
        let got = expand_lookup_vector_arg(&arg, &resolver_with(EvalValue::Number(0.0)));
        assert_eq!(
            got,
            Err(CoercionError::UnsupportedValueKind("two_dimensional_array"))
        );
    }
}

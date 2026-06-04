use crate::coercion::{CoercionError, coerce_eval_to_number};
use crate::resolver::{
    ReferenceSystemCapabilities, ReferenceSystemProvider, ResolvedReferenceValues,
    enumerate_reference_values, materialize_resolved_reference_values, resolve_eval_value,
};
pub use crate::value::CalcValue;
use crate::value::{ArrayShape, CalcArray, CoreValue, WorksheetErrorCode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryNumericCoercionLiftProfile {
    ScalarOnly,
    ScalarOrArrayElementwise,
}

fn normalize_prepared_eval(value: CalcValue) -> CalcValue {
    match value.core() {
        CoreValue::Array(array) if array.shape().rows == 1 && array.shape().cols == 1 => {
            array.get(0, 0).cloned().unwrap_or_else(CalcValue::empty)
        }
        _ => value,
    }
}

fn normalize_prepared_calc_value(value: CalcValue) -> CalcValue {
    match value.core() {
        CoreValue::Array(array) if array.shape().rows == 1 && array.shape().cols == 1 => {
            array.get(0, 0).cloned().unwrap_or_else(CalcValue::empty)
        }
        _ => value,
    }
}

pub(crate) fn prepared_arg_to_calc_value_lossy(prepared: &CalcValue) -> CalcValue {
    prepared.clone()
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
pub(crate) struct AggregatePreparedValue {
    origin: AggregateArgOrigin,
    value: CalcValue,
}

impl AggregatePreparedValue {
    pub(crate) fn direct_scalar(value: CalcValue) -> Self {
        Self {
            origin: AggregateArgOrigin::DirectScalar,
            value,
        }
    }

    pub(crate) fn array_like(value: CalcValue, provenance: AggregateArrayProvenance) -> Self {
        Self {
            origin: AggregateArgOrigin::ArrayLike(provenance),
            value,
        }
    }

    pub(crate) fn origin(&self) -> AggregateArgOrigin {
        self.origin
    }

    pub(crate) fn value(&self) -> &CalcValue {
        &self.value
    }
}

fn prepared_from_array_cell(cell: &CalcValue) -> CalcValue {
    cell.clone()
}

fn calc_value_from_prepared(value: CalcValue) -> CalcValue {
    value
}

pub(crate) fn prepared_from_calc_value(value: &CalcValue) -> CalcValue {
    value.clone()
}

fn prepared_vec_from_calc_values(values: &[CalcValue]) -> Vec<CalcValue> {
    values.iter().map(prepared_from_calc_value).collect()
}

pub(crate) fn expand_aggregate_array_with_provenance(
    array: &CalcArray,
    provenance: AggregateArrayProvenance,
) -> Vec<AggregatePreparedValue> {
    array
        .iter_row_major()
        .cloned()
        .map(|value| AggregatePreparedValue::array_like(value, provenance))
        .collect()
}

fn expand_resolved_eval_value(value: &CalcValue) -> Vec<CalcValue> {
    match value.core() {
        CoreValue::Array(array) => array
            .iter_row_major()
            .map(prepared_from_array_cell)
            .collect(),
        _ => vec![value.clone()],
    }
}

fn expand_resolved_reference_values(
    values: ResolvedReferenceValues,
) -> Result<Vec<CalcValue>, CoercionError> {
    let array =
        materialize_resolved_reference_values(&values).map_err(CoercionError::RefResolution)?;
    Ok(array
        .iter_row_major()
        .map(prepared_from_array_cell)
        .collect())
}

pub(crate) fn expand_sparse_reference_values_with_provenance(
    values: ResolvedReferenceValues,
    provenance: AggregateArrayProvenance,
) -> Vec<AggregatePreparedValue> {
    values
        .defined_cells
        .into_iter()
        .map(|cell| cell.value)
        .map(|value| AggregatePreparedValue::array_like(value, provenance))
        .collect()
}

pub fn sparse_reference_values_for_aggregate_arg(
    arg: &CalcValue,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<Option<ResolvedReferenceValues>, CoercionError> {
    match arg.core() {
        CoreValue::Reference(r) => {
            enumerate_reference_values(resolver, r).map_err(CoercionError::RefResolution)
        }
        _ => Ok(None),
    }
}

fn expand_lookup_eval_value(value: &CalcValue) -> Result<Vec<CalcValue>, CoercionError> {
    match value.core() {
        CoreValue::Array(array) => {
            let shape = array.shape();
            if shape.rows > 1 && shape.cols > 1 {
                return Err(CoercionError::UnsupportedValueKind("two_dimensional_array"));
            }
            Ok(array
                .iter_row_major()
                .map(prepared_from_array_cell)
                .collect())
        }
        _ => Ok(vec![value.clone()]),
    }
}

fn resolve_eval_references(
    value: &CalcValue,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, CoercionError> {
    match value.core() {
        CoreValue::Reference(r) => {
            let resolved = resolve_eval_value(resolver, r).map_err(CoercionError::RefResolution)?;
            resolve_eval_references(&resolved, resolver)
        }
        _ => Ok(value.clone()),
    }
}

fn resolve_calc_references(
    value: &CalcValue,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, CoercionError> {
    match value.core() {
        CoreValue::Reference(reference) => {
            let resolved =
                resolve_eval_value(resolver, reference).map_err(CoercionError::RefResolution)?;
            resolve_calc_references(&resolved, resolver)
        }
        _ => Ok(value.clone()),
    }
}

fn calc_value_from_call_arg(arg: &CalcValue) -> CalcValue {
    arg.clone()
}

pub fn prepare_calc_value_values_only(
    arg: &CalcValue,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, CoercionError> {
    Ok(normalize_prepared_calc_value(resolve_calc_references(
        arg, resolver,
    )?))
}

pub fn prepare_calc_values_only(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<Vec<CalcValue>, CoercionError> {
    args.iter()
        .map(|arg| prepare_calc_value_values_only(arg, resolver))
        .collect()
}

pub fn prepare_call_arg_as_calc_value_values_only(
    arg: &CalcValue,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, CoercionError> {
    prepare_calc_value_values_only(&calc_value_from_call_arg(arg), resolver)
}

pub fn prepare_call_args_as_calc_values_only(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<Vec<CalcValue>, CoercionError> {
    args.iter()
        .map(|arg| prepare_call_arg_as_calc_value_values_only(arg, resolver))
        .collect()
}

pub fn prepare_arg_values_only(
    arg: &CalcValue,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, CoercionError> {
    prepare_call_arg_as_calc_value_values_only(arg, resolver).map(|value| {
        let prepared = prepared_from_calc_value(&value);
        normalize_prepared_eval(prepared)
    })
}

pub fn prepare_args_values_only(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<Vec<CalcValue>, CoercionError> {
    prepare_call_args_as_calc_values_only(args, resolver)
        .map(|values| prepared_vec_from_calc_values(&values))
}

pub fn expand_arg_values_only(
    arg: &CalcValue,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<Vec<CalcValue>, CoercionError> {
    match arg.core() {
        CoreValue::Reference(r) => {
            if let Some(values) =
                enumerate_reference_values(resolver, r).map_err(CoercionError::RefResolution)?
            {
                return expand_resolved_reference_values(values);
            }
            let resolved = resolve_eval_value(resolver, r).map_err(CoercionError::RefResolution)?;
            Ok(expand_resolved_eval_value(&resolve_eval_references(
                &resolved, resolver,
            )?))
        }
        _ => Ok(expand_resolved_eval_value(&resolve_eval_references(
            arg, resolver,
        )?)),
    }
}

pub fn expand_lookup_vector_arg(
    arg: &CalcValue,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<Vec<CalcValue>, CoercionError> {
    match arg.core() {
        CoreValue::Reference(r) => {
            if let Some(values) =
                enumerate_reference_values(resolver, r).map_err(CoercionError::RefResolution)?
            {
                let array = materialize_resolved_reference_values(&values)
                    .map_err(CoercionError::RefResolution)?;
                return expand_lookup_eval_value(&CalcValue::array(array));
            }
            let resolved = resolve_eval_value(resolver, r).map_err(CoercionError::RefResolution)?;
            expand_lookup_eval_value(&resolve_eval_references(&resolved, resolver)?)
        }
        _ => expand_lookup_eval_value(&resolve_eval_references(arg, resolver)?),
    }
}

pub(crate) fn expand_aggregate_arg(
    arg: &CalcValue,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<Vec<AggregatePreparedValue>, CoercionError> {
    match arg.core() {
        CoreValue::Reference(r) => {
            if let Some(values) = sparse_reference_values_for_aggregate_arg(arg, resolver)? {
                return Ok(expand_sparse_reference_values_with_provenance(
                    values,
                    AggregateArrayProvenance::ReferenceDerived,
                ));
            }
            let resolved = resolve_eval_value(resolver, r).map_err(CoercionError::RefResolution)?;
            let resolved = resolve_eval_references(&resolved, resolver)?;
            match resolved.core() {
                CoreValue::Array(array) => Ok(expand_aggregate_array_with_provenance(
                    &array,
                    AggregateArrayProvenance::ReferenceDerived,
                )),
                _ => Ok(vec![AggregatePreparedValue::array_like(
                    resolved,
                    AggregateArrayProvenance::ReferenceDerived,
                )]),
            }
        }
        CoreValue::Array(array) => Ok(expand_aggregate_array_with_provenance(
            array,
            AggregateArrayProvenance::OpaqueArrayValue,
        )),
        _ => Ok(expand_arg_values_only(arg, resolver)?
            .into_iter()
            .map(|value| AggregatePreparedValue::direct_scalar(calc_value_from_prepared(value)))
            .collect()),
    }
}

pub fn run_values_only_prepared<Out, E>(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
    on_prepared: impl FnOnce(&[CalcValue]) -> Result<Out, E>,
    map_preparation_error: impl FnOnce(CoercionError) -> E,
) -> Result<Out, E> {
    let prepared_calc =
        prepare_call_args_as_calc_values_only(args, resolver).map_err(map_preparation_error)?;
    let prepared = prepared_vec_from_calc_values(&prepared_calc);
    on_prepared(&prepared)
}

pub fn run_calc_values_only_prepared<Out, E>(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
    on_prepared: impl FnOnce(&[CalcValue]) -> Result<Out, E>,
    map_preparation_error: impl FnOnce(CoercionError) -> E,
) -> Result<Out, E> {
    let prepared = prepare_calc_values_only(args, resolver).map_err(map_preparation_error)?;
    on_prepared(&prepared)
}

/// Convert a scalar `CalcValue` into an array cell (for elementwise lift).
/// A scalar per-cell kernel should never yield array/reference/lambda.
fn scalar_eval_to_cell(value: CalcValue) -> CalcValue {
    match value.core() {
        CoreValue::Number(_) | CoreValue::Text(_) | CoreValue::Logical(_) | CoreValue::Error(_) => {
            value
        }
        _ => CalcValue::error(WorksheetErrorCode::Value),
    }
}

/// Like [`run_values_only_prepared`], but lifts the scalar per-cell evaluator
/// elementwise over array arguments (Excel spill), broadcasting scalar args to
/// the common shape. When no argument is an array it behaves exactly like the
/// scalar path. The per-cell evaluator returns a *scalar* `CalcValue`; on a
/// per-cell `Err`, `map_err_to_ws` decides the cell's worksheet error so the
/// array can carry per-element errors the way Excel does.
pub fn run_values_only_prepared_lifted<E>(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
    on_cell: impl Fn(&[CalcValue]) -> Result<CalcValue, E>,
    map_err_to_ws: impl Fn(&E) -> WorksheetErrorCode,
    map_preparation_error: impl FnOnce(CoercionError) -> E,
) -> Result<CalcValue, E> {
    let prepared = prepare_args_values_only(args, resolver).map_err(map_preparation_error)?;
    if let Some((shape, groups)) = expand_prepared_broadcast_grid(&prepared) {
        let cells = groups
            .into_iter()
            .map(|group| match group {
                BroadcastPreparedGroup::Values(values) => match on_cell(&values) {
                    Ok(value) => scalar_eval_to_cell(value),
                    Err(error) => CalcValue::error(map_err_to_ws(&error)),
                },
                BroadcastPreparedGroup::MissingCoordinate => {
                    CalcValue::error(WorksheetErrorCode::NA)
                }
            })
            .collect();
        return Ok(CalcValue::array(
            CalcArray::new(shape, cells).expect("broadcast shape preserved"),
        ));
    }
    on_cell(&prepared)
}

pub fn map_values_only_prepared<Out>(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
    on_prepared_arg: impl Fn(&CalcValue) -> Out,
    on_preparation_error: impl Fn(CoercionError) -> Out,
) -> Vec<Out> {
    args.iter()
        .map(|arg| match prepare_arg_values_only(arg, resolver) {
            Ok(prepared) => on_prepared_arg(&prepared),
            Err(e) => on_preparation_error(e),
        })
        .collect()
}

pub fn map_calc_values_only_prepared<Out>(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
    on_prepared_arg: impl Fn(&CalcValue) -> Out,
    on_preparation_error: impl Fn(CoercionError) -> Out,
) -> Vec<Out> {
    args.iter()
        .map(|arg| match prepare_calc_value_values_only(arg, resolver) {
            Ok(prepared) => on_prepared_arg(&prepared),
            Err(e) => on_preparation_error(e),
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq)]
pub enum BroadcastPreparedPair {
    Pair(CalcValue, CalcValue),
    MissingCoordinate,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BroadcastPreparedGroup {
    Values(Vec<CalcValue>),
    MissingCoordinate,
}

fn prepared_shape(value: &CalcValue) -> ArrayShape {
    match value.core() {
        CoreValue::Array(array) => array.shape(),
        _ => ArrayShape { rows: 1, cols: 1 },
    }
}

fn prepared_broadcast_value_at(value: &CalcValue, row: usize, col: usize) -> Option<CalcValue> {
    match value.core() {
        CoreValue::Array(array) => {
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
        _ => Some(value.clone()),
    }
}

pub fn expand_binary_broadcast_grid(
    lhs: &CalcValue,
    rhs: &CalcValue,
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
    args: &[CalcValue],
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

struct NoReferenceSystemProvider;

impl ReferenceSystemProvider for NoReferenceSystemProvider {
    fn capabilities(&self) -> ReferenceSystemCapabilities {
        ReferenceSystemCapabilities {
            allow_eval_time_deref: false,
            allow_three_d_refs: false,
            allow_structured_refs: false,
            allow_spill_anchor_refs: false,
            allow_external_refs: false,
        }
    }

    fn dereference(
        &self,
        _request: &crate::resolver::ReferenceDereferenceRequest,
    ) -> Result<CalcValue, crate::resolver::ReferenceResolutionError> {
        Err(
            crate::resolver::ReferenceResolutionError::CapabilityDenied {
                kind: crate::value::ReferenceKind::A1,
                capability: "values_only_pre_adapter_invariant",
            },
        )
    }
}

pub fn coerce_prepared_to_number(arg: &CalcValue) -> Result<f64, CoercionError> {
    match arg.core() {
        CoreValue::Missing => Err(CoercionError::MissingArg),
        CoreValue::Empty => Err(CoercionError::EmptyCell),
        _ => coerce_eval_to_number(arg, &NoReferenceSystemProvider),
    }
}

pub fn coerce_prepared_to_text(arg: &CalcValue) -> Result<crate::value::ExcelText, CoercionError> {
    use crate::value::ExcelText;

    match arg.core() {
        CoreValue::Text(t) => Ok(t.clone()),
        CoreValue::Number(n) => Ok(ExcelText::from_utf16_code_units(
            format!("{n}").encode_utf16().collect(),
        )),
        CoreValue::Logical(b) => Ok(ExcelText::from_utf16_code_units(
            if *b { "TRUE" } else { "FALSE" }.encode_utf16().collect(),
        )),
        CoreValue::Error(code) => Err(CoercionError::WorksheetError(*code)),
        CoreValue::Array(_) => Err(CoercionError::UnsupportedValueKind("array")),
        CoreValue::Reference(_) => Err(CoercionError::RefResolution(
            crate::resolver::ReferenceResolutionError::EvalTimeDerefNotAllowed,
        )),
        CoreValue::Missing | CoreValue::Empty => Ok(ExcelText::from_utf16_code_units(Vec::new())),
    }
}

pub fn apply_unary_numeric_scalar_prepared(
    arg: &CalcValue,
    kernel: fn(f64) -> f64,
) -> Result<f64, CoercionError> {
    let n = coerce_prepared_to_number(arg)?;
    Ok(kernel(n))
}

pub fn apply_unary_numeric_array_map_prepared(
    args: &[CalcValue],
    kernel: fn(f64) -> f64,
) -> Vec<Result<f64, CoercionError>> {
    args.iter()
        .map(|arg| apply_unary_numeric_scalar_prepared(arg, kernel))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::ReferenceSystemCapabilities;
    use crate::value::{
        CalcArray, CallableArityShape, CallableValue, ExcelText, OpaqueCallable, ReferenceKind,
        ReferenceLike, WorksheetErrorCode,
    };
    use std::collections::BTreeMap;
    use std::rc::Rc;

    struct MockResolver {
        caps: ReferenceSystemCapabilities,
        resolved_value: Option<CalcValue>,
        by_target: BTreeMap<String, CalcValue>,
    }
    #[derive(Debug)]
    struct TestCallableHandle;

    impl OpaqueCallable for TestCallableHandle {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    impl ReferenceSystemProvider for MockResolver {
        fn capabilities(&self) -> ReferenceSystemCapabilities {
            self.caps
        }

        fn dereference(
            &self,
            request: &crate::resolver::ReferenceDereferenceRequest,
        ) -> Result<CalcValue, crate::resolver::ReferenceResolutionError> {
            let reference = &request.reference;
            if let Some(value) = self.by_target.get(reference.target()) {
                return Ok(value.clone());
            }
            self.resolved_value.clone().ok_or(
                crate::resolver::ReferenceResolutionError::UnresolvedReference {
                    target: reference.target().to_string(),
                },
            )
        }
    }

    fn resolver_with(value: CalcValue) -> MockResolver {
        MockResolver {
            caps: ReferenceSystemCapabilities::permissive_local(),
            resolved_value: Some(value),
            by_target: BTreeMap::new(),
        }
    }

    #[test]
    fn prepare_values_only_dereferences_reference_arg() {
        let arg = CalcValue::reference(ReferenceLike::new(ReferenceKind::A1, "A1".to_string()));
        let prepared = prepare_arg_values_only(&arg, &resolver_with(CalcValue::number(3.0)));
        assert_eq!(prepared, Ok((CalcValue::number(3.0))));
    }

    #[test]
    fn prepare_values_only_normalizes_single_blank_area_to_empty_cell() {
        let arg = CalcValue::reference(ReferenceLike::new(ReferenceKind::A1, "A1".to_string()));
        let prepared = prepare_arg_values_only(
            &arg,
            &resolver_with(CalcValue::array(
                CalcArray::from_rows(vec![vec![CalcValue::empty()]]).unwrap(),
            )),
        );
        assert_eq!(prepared, Ok(CalcValue::empty()));
    }

    #[test]
    fn prepare_values_only_preserves_missing_and_empty() {
        assert_eq!(
            prepare_arg_values_only(
                &CalcValue::missing(),
                &resolver_with(CalcValue::number(1.0))
            ),
            Ok(CalcValue::missing())
        );
        assert_eq!(
            prepare_arg_values_only(&CalcValue::empty(), &resolver_with(CalcValue::number(1.0))),
            Ok(CalcValue::empty())
        );
    }

    #[test]
    fn prepare_values_only_accepts_provider_materialized_multi_area_reference() {
        let mut by_target = BTreeMap::new();
        by_target.insert(
            "(Alpha!A1:A2,Alpha!B2)".to_string(),
            CalcValue::array(
                CalcArray::from_rows(vec![vec![
                    CalcValue::number(7.0),
                    CalcValue::number(11.0),
                    CalcValue::number(13.0),
                ]])
                .unwrap(),
            ),
        );
        let resolver = MockResolver {
            caps: ReferenceSystemCapabilities::permissive_local(),
            resolved_value: None,
            by_target,
        };
        let arg = CalcValue::reference(ReferenceLike::new(
            ReferenceKind::MultiArea,
            "(Alpha!A1:A2,Alpha!B2)",
        ));

        let prepared = prepare_arg_values_only(&arg, &resolver);
        assert_eq!(
            prepared,
            Ok((CalcValue::array(
                CalcArray::from_rows(vec![vec![
                    CalcValue::number(7.0),
                    CalcValue::number(11.0),
                    CalcValue::number(13.0),
                ]])
                .unwrap()
            )))
        );
    }

    #[test]
    fn prepare_calc_values_only_preserves_calcvalue_carriers_without_public_wrapper() {
        let args = vec![
            CalcValue::number(3.0),
            CalcValue::empty(),
            CalcValue::missing(),
            CalcValue::array(
                crate::value::CalcArray::from_rows(vec![vec![
                    CalcValue::number(1.0),
                    CalcValue::empty(),
                ]])
                .unwrap(),
            ),
        ];

        let prepared = prepare_calc_values_only(&args, &resolver_with(CalcValue::number(0.0)));

        assert_eq!(prepared, Ok(args));
    }

    #[test]
    fn prepare_call_arg_as_calc_value_values_only_normalizes_single_blank_area() {
        let arg = CalcValue::reference(ReferenceLike::new(ReferenceKind::A1, "A1"));
        let prepared = prepare_call_arg_as_calc_value_values_only(
            &arg,
            &resolver_with(CalcValue::array(
                CalcArray::from_rows(vec![vec![CalcValue::empty()]]).unwrap(),
            )),
        );

        assert_eq!(prepared, Ok(CalcValue::empty()));
    }

    #[test]
    fn run_calc_values_only_prepared_keeps_adapter_values_as_calcvalue() {
        let args = vec![
            CalcValue::number(2.0),
            CalcValue::text(ExcelText::from_utf16_code_units(
                "x".encode_utf16().collect(),
            )),
        ];
        let got = run_calc_values_only_prepared(
            &args,
            &resolver_with(CalcValue::number(0.0)),
            |prepared| Ok::<_, CoercionError>(prepared.to_vec()),
            |e| e,
        );

        assert_eq!(got, Ok(args));
    }

    #[test]
    fn calc_preparation_preserves_native_callable_payload() {
        let arg = CalcValue::callable(CallableValue {
            arity: CallableArityShape::exact(1),
            summary: "helper.lambda".to_string(),
            handle: Rc::new(TestCallableHandle),
        });

        let prepared =
            prepare_calc_values_only(&[arg.clone()], &resolver_with(CalcValue::number(0.0)));

        assert_eq!(prepared, Ok(vec![arg]));
    }

    #[test]
    fn expand_lookup_vector_accepts_provider_materialized_multi_area_reference() {
        let mut by_target = BTreeMap::new();
        by_target.insert(
            "(A1:A2,C1)".to_string(),
            CalcValue::array(
                CalcArray::from_rows(vec![vec![
                    CalcValue::number(1.0),
                    CalcValue::number(2.0),
                    CalcValue::number(3.0),
                ]])
                .unwrap(),
            ),
        );
        let resolver = MockResolver {
            caps: ReferenceSystemCapabilities::permissive_local(),
            resolved_value: None,
            by_target,
        };
        let arg = CalcValue::reference(ReferenceLike::new(ReferenceKind::MultiArea, "(A1:A2,C1)"));

        let prepared = expand_lookup_vector_arg(&arg, &resolver);
        assert_eq!(
            prepared,
            Ok(vec![
                (CalcValue::number(1.0)),
                (CalcValue::number(2.0)),
                (CalcValue::number(3.0)),
            ])
        );
    }

    #[test]
    fn prepared_coercion_numeric_text_and_error_paths() {
        let text = (CalcValue::text(ExcelText::from_utf16_code_units(
            "2".encode_utf16().collect(),
        )));
        assert_eq!(coerce_prepared_to_number(&text), Ok(2.0));

        let err = (CalcValue::error(WorksheetErrorCode::Value));
        assert_eq!(
            coerce_prepared_to_number(&err),
            Err(CoercionError::WorksheetError(WorksheetErrorCode::Value))
        );
    }

    #[test]
    fn prepared_text_coercion_formats_scalars_and_blanks() {
        let number = (CalcValue::number(2.5));
        assert_eq!(
            coerce_prepared_to_text(&number),
            Ok(ExcelText::from_utf16_code_units(
                "2.5".encode_utf16().collect()
            ))
        );

        let blank = CalcValue::empty();
        assert_eq!(
            coerce_prepared_to_text(&blank),
            Ok(ExcelText::from_utf16_code_units(Vec::new()))
        );
    }

    #[test]
    fn prepared_coercion_rejects_reference_if_invariant_broken() {
        let prepared =
            (CalcValue::reference(ReferenceLike::new(ReferenceKind::A1, "A1".to_string())));
        let got = coerce_prepared_to_number(&prepared);
        assert_eq!(
            got,
            Err(CoercionError::RefResolution(
                crate::resolver::ReferenceResolutionError::EvalTimeDerefNotAllowed
            ))
        );
    }

    #[test]
    fn unary_numeric_array_map_prepared_preserves_per_element_results() {
        let args = vec![
            (CalcValue::number(-2.0)),
            (CalcValue::text(ExcelText::from_utf16_code_units(
                "asd".encode_utf16().collect(),
            ))),
            (CalcValue::logical(true)),
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
        let args = [(CalcValue::number(2.0))];
        let got = run_values_only_prepared(
            &args,
            &resolver_with(CalcValue::number(0.0)),
            |prepared| Ok::<f64, CoercionError>(coerce_prepared_to_number(&prepared[0])?),
            |e| e,
        );
        assert_eq!(got, Ok(2.0));
    }

    #[test]
    fn map_values_only_prepared_maps_preparation_errors_per_arg() {
        let args = vec![
            CalcValue::reference(ReferenceLike::new(ReferenceKind::A1, "A1".to_string())),
            (CalcValue::number(2.0)),
        ];
        let resolver = MockResolver {
            caps: ReferenceSystemCapabilities {
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
        let arg = (CalcValue::array(
            CalcArray::from_rows(vec![
                vec![CalcValue::number(1.0), CalcValue::empty()],
                vec![
                    CalcValue::text(ExcelText::from_utf16_code_units(
                        "x".encode_utf16().collect(),
                    )),
                    CalcValue::logical(true),
                ],
            ])
            .unwrap(),
        ));
        let got = expand_arg_values_only(&arg, &resolver_with(CalcValue::number(0.0))).unwrap();
        assert_eq!(
            got,
            vec![
                (CalcValue::number(1.0)),
                CalcValue::empty(),
                (CalcValue::text(ExcelText::from_utf16_code_units(
                    "x".encode_utf16().collect(),
                ))),
                (CalcValue::logical(true)),
            ]
        );
    }

    #[test]
    fn expand_aggregate_arg_marks_reference_derived_values() {
        let arg =
            CalcValue::reference(ReferenceLike::new(ReferenceKind::Area, "A1:A2".to_string()));
        let got = expand_aggregate_arg(
            &arg,
            &resolver_with(CalcValue::array(
                CalcArray::from_rows(vec![
                    vec![CalcValue::number(1.0)],
                    vec![CalcValue::text(ExcelText::from_utf16_code_units(
                        "2".encode_utf16().collect(),
                    ))],
                ])
                .unwrap(),
            )),
        )
        .unwrap();
        assert_eq!(got.len(), 2);
        assert!(got.iter().all(|item| item.origin()
            == AggregateArgOrigin::ArrayLike(AggregateArrayProvenance::ReferenceDerived)));
    }

    #[test]
    fn expand_aggregate_arg_admits_opaque_reference_values_as_reference_derived() {
        let arg = (CalcValue::reference(ReferenceLike::new(
            ReferenceKind::Area,
            "NameBackedRange".to_string(),
        )));
        let got = expand_aggregate_arg(
            &arg,
            &resolver_with(CalcValue::array(
                CalcArray::from_rows(vec![
                    vec![CalcValue::number(1.0)],
                    vec![CalcValue::number(2.0)],
                ])
                .unwrap(),
            )),
        )
        .unwrap();

        assert_eq!(got.len(), 2);
        assert!(got.iter().all(|item| item.origin()
            == AggregateArgOrigin::ArrayLike(AggregateArrayProvenance::ReferenceDerived)));
    }

    #[test]
    fn expand_aggregate_arg_marks_eval_arrays_as_opaque_array_values() {
        let got = expand_aggregate_arg(
            &(CalcValue::array(
                CalcArray::from_rows(vec![vec![
                    CalcValue::text(ExcelText::from_utf16_code_units(
                        "2".encode_utf16().collect(),
                    )),
                    CalcValue::logical(true),
                ]])
                .unwrap(),
            )),
            &resolver_with(CalcValue::number(0.0)),
        )
        .unwrap();
        assert_eq!(got.len(), 2);
        assert!(got.iter().all(|item| item.origin()
            == AggregateArgOrigin::ArrayLike(AggregateArrayProvenance::OpaqueArrayValue)));
    }

    #[test]
    fn expand_aggregate_array_with_provenance_marks_direct_array_literal() {
        let array = CalcArray::from_rows(vec![vec![
            CalcValue::text(ExcelText::from_utf16_code_units(
                "2".encode_utf16().collect(),
            )),
            CalcValue::logical(true),
        ]])
        .unwrap();

        let got = expand_aggregate_array_with_provenance(
            &array,
            AggregateArrayProvenance::DirectArrayLiteral,
        );

        assert_eq!(got.len(), 2);
        assert!(got.iter().all(|item| item.origin()
            == AggregateArgOrigin::ArrayLike(AggregateArrayProvenance::DirectArrayLiteral)));
    }

    #[test]
    fn expand_lookup_vector_arg_rejects_two_dimensional_array() {
        let arg = (CalcValue::array(
            CalcArray::from_rows(vec![
                vec![CalcValue::number(1.0), CalcValue::number(2.0)],
                vec![CalcValue::number(3.0), CalcValue::number(4.0)],
            ])
            .unwrap(),
        ));
        let got = expand_lookup_vector_arg(&arg, &resolver_with(CalcValue::number(0.0)));
        assert_eq!(
            got,
            Err(CoercionError::UnsupportedValueKind("two_dimensional_array"))
        );
    }
}

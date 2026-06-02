use crate::coercion::{CoercionError, coerce_eval_to_number};
use crate::resolver::{
    ReferenceSystemCapabilities, ReferenceSystemProvider, ResolvedReferenceValues,
    enumerate_reference_values, materialize_resolved_reference_values, resolve_eval_value,
};
use crate::value::{
    ArrayShape, CalcValue, CoreValue, FunctionArg, FunctionArray, FunctionArrayCell, FunctionValue,
    WorksheetErrorCode,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryNumericCoercionLiftProfile {
    ScalarOnly,
    ScalarOrArrayElementwise,
}

fn normalize_prepared_eval(value: FunctionValue) -> PreparedValue {
    match value {
        FunctionValue::Array(array) if array.shape().rows == 1 && array.shape().cols == 1 => {
            match array.get(0, 0) {
                Some(FunctionArrayCell::EmptyCell) => PreparedValue::EmptyCell,
                Some(cell) => prepared_from_array_cell(cell),
                None => PreparedValue::Eval(FunctionValue::Array(array)),
            }
        }
        other => PreparedValue::Eval(other),
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

/// Prepared values used by kernels that still operate on the function-facing
/// scalar/array/reference surface rather than native `CalcValue` arguments.
#[derive(Debug, Clone, PartialEq)]
pub enum PreparedValue {
    Eval(FunctionValue),
    MissingArg,
    EmptyCell,
}

pub(crate) fn prepared_arg_to_calc_value_lossy(prepared: &PreparedValue) -> CalcValue {
    match prepared {
        PreparedValue::Eval(value) => CalcValue::from(value.clone()),
        PreparedValue::MissingArg => CalcValue::missing(),
        PreparedValue::EmptyCell => CalcValue::empty(),
    }
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

fn prepared_from_array_cell(cell: &FunctionArrayCell) -> PreparedValue {
    match cell {
        FunctionArrayCell::Number(n) => PreparedValue::Eval(FunctionValue::Number(*n)),
        FunctionArrayCell::Text(t) => PreparedValue::Eval(FunctionValue::Text(t.clone())),
        FunctionArrayCell::Logical(b) => PreparedValue::Eval(FunctionValue::Logical(*b)),
        FunctionArrayCell::Error(code) => PreparedValue::Eval(FunctionValue::Error(*code)),
        FunctionArrayCell::EmptyCell => PreparedValue::EmptyCell,
    }
}

fn calc_value_from_prepared(value: PreparedValue) -> CalcValue {
    match value {
        PreparedValue::Eval(value) => CalcValue::from(value),
        PreparedValue::MissingArg => CalcValue::missing(),
        PreparedValue::EmptyCell => CalcValue::empty(),
    }
}

pub(crate) fn prepared_from_calc_value(value: &CalcValue) -> PreparedValue {
    match FunctionArg::value(value.clone()) {
        FunctionArg::Eval(value) => PreparedValue::Eval(value),
        FunctionArg::MissingArg => PreparedValue::MissingArg,
        FunctionArg::EmptyCell => PreparedValue::EmptyCell,
        FunctionArg::Reference(reference) => {
            PreparedValue::Eval(FunctionValue::Reference(reference))
        }
    }
}

fn prepared_vec_from_calc_values(values: &[CalcValue]) -> Vec<PreparedValue> {
    values.iter().map(prepared_from_calc_value).collect()
}

pub(crate) fn expand_aggregate_array_with_provenance(
    array: &FunctionArray,
    provenance: AggregateArrayProvenance,
) -> Vec<AggregatePreparedValue> {
    array
        .iter_row_major()
        .map(FunctionArrayCell::to_calc_value_lossy)
        .map(|value| AggregatePreparedValue::array_like(value, provenance))
        .collect()
}

fn expand_resolved_eval_value(value: &FunctionValue) -> Vec<PreparedValue> {
    match value {
        FunctionValue::Array(array) => array
            .iter_row_major()
            .map(prepared_from_array_cell)
            .collect(),
        _ => vec![PreparedValue::Eval(value.clone())],
    }
}

fn expand_resolved_reference_values(
    values: ResolvedReferenceValues,
) -> Result<Vec<PreparedValue>, CoercionError> {
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
        .map(|cell| cell.value.to_calc_value_lossy())
        .map(|value| AggregatePreparedValue::array_like(value, provenance))
        .collect()
}

pub fn sparse_reference_values_for_aggregate_arg(
    arg: &FunctionArg,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<Option<ResolvedReferenceValues>, CoercionError> {
    match arg {
        FunctionArg::Reference(r) | FunctionArg::Eval(FunctionValue::Reference(r)) => {
            enumerate_reference_values(resolver, r).map_err(CoercionError::RefResolution)
        }
        _ => Ok(None),
    }
}

fn expand_lookup_eval_value(value: &FunctionValue) -> Result<Vec<PreparedValue>, CoercionError> {
    match value {
        FunctionValue::Array(array) => {
            let shape = array.shape();
            if shape.rows > 1 && shape.cols > 1 {
                return Err(CoercionError::UnsupportedValueKind("two_dimensional_array"));
            }
            Ok(array
                .iter_row_major()
                .map(prepared_from_array_cell)
                .collect())
        }
        _ => Ok(vec![PreparedValue::Eval(value.clone())]),
    }
}

fn resolve_eval_references(
    value: &FunctionValue,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<FunctionValue, CoercionError> {
    match value {
        FunctionValue::Reference(r) => {
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
            resolve_calc_references(&CalcValue::from(resolved), resolver)
        }
        _ => Ok(value.clone()),
    }
}

fn calc_value_from_call_arg(arg: &FunctionArg) -> CalcValue {
    match arg {
        FunctionArg::Eval(value) => CalcValue::from(value.clone()),
        FunctionArg::MissingArg => CalcValue::missing(),
        FunctionArg::EmptyCell => CalcValue::empty(),
        FunctionArg::Reference(reference) => CalcValue::reference(reference.clone()),
    }
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
    arg: &FunctionArg,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, CoercionError> {
    prepare_calc_value_values_only(&calc_value_from_call_arg(arg), resolver)
}

pub fn prepare_call_args_as_calc_values_only(
    args: &[FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<Vec<CalcValue>, CoercionError> {
    args.iter()
        .map(|arg| prepare_call_arg_as_calc_value_values_only(arg, resolver))
        .collect()
}

pub fn prepare_arg_values_only(
    arg: &FunctionArg,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<PreparedValue, CoercionError> {
    prepare_call_arg_as_calc_value_values_only(arg, resolver).map(|value| {
        let prepared = prepared_from_calc_value(&value);
        match prepared {
            PreparedValue::Eval(value) => normalize_prepared_eval(value),
            other => other,
        }
    })
}

pub fn prepare_args_values_only(
    args: &[FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<Vec<PreparedValue>, CoercionError> {
    prepare_call_args_as_calc_values_only(args, resolver)
        .map(|values| prepared_vec_from_calc_values(&values))
}

pub fn expand_arg_values_only(
    arg: &FunctionArg,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<Vec<PreparedValue>, CoercionError> {
    match arg {
        FunctionArg::Eval(FunctionValue::Reference(r)) => {
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
        FunctionArg::Eval(v) => Ok(expand_resolved_eval_value(&resolve_eval_references(
            v, resolver,
        )?)),
        FunctionArg::MissingArg => Ok(vec![PreparedValue::MissingArg]),
        FunctionArg::EmptyCell => Ok(vec![PreparedValue::EmptyCell]),
        FunctionArg::Reference(r) => {
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
    }
}

pub fn expand_lookup_vector_arg(
    arg: &FunctionArg,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<Vec<PreparedValue>, CoercionError> {
    match arg {
        FunctionArg::Eval(FunctionValue::Reference(r)) => {
            if let Some(values) =
                enumerate_reference_values(resolver, r).map_err(CoercionError::RefResolution)?
            {
                let array = materialize_resolved_reference_values(&values)
                    .map_err(CoercionError::RefResolution)?;
                return expand_lookup_eval_value(&FunctionValue::Array(array));
            }
            let resolved = resolve_eval_value(resolver, r).map_err(CoercionError::RefResolution)?;
            expand_lookup_eval_value(&resolve_eval_references(&resolved, resolver)?)
        }
        FunctionArg::Eval(v) => expand_lookup_eval_value(&resolve_eval_references(v, resolver)?),
        FunctionArg::MissingArg => Ok(vec![PreparedValue::MissingArg]),
        FunctionArg::EmptyCell => Ok(vec![PreparedValue::EmptyCell]),
        FunctionArg::Reference(r) => {
            if let Some(values) =
                enumerate_reference_values(resolver, r).map_err(CoercionError::RefResolution)?
            {
                let array = materialize_resolved_reference_values(&values)
                    .map_err(CoercionError::RefResolution)?;
                return expand_lookup_eval_value(&FunctionValue::Array(array));
            }
            let resolved = resolve_eval_value(resolver, r).map_err(CoercionError::RefResolution)?;
            expand_lookup_eval_value(&resolve_eval_references(&resolved, resolver)?)
        }
    }
}

pub(crate) fn expand_aggregate_arg(
    arg: &FunctionArg,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<Vec<AggregatePreparedValue>, CoercionError> {
    match arg {
        FunctionArg::Reference(r) | FunctionArg::Eval(FunctionValue::Reference(r)) => {
            if let Some(values) = sparse_reference_values_for_aggregate_arg(arg, resolver)? {
                return Ok(expand_sparse_reference_values_with_provenance(
                    values,
                    AggregateArrayProvenance::ReferenceDerived,
                ));
            }
            let resolved = resolve_eval_value(resolver, r).map_err(CoercionError::RefResolution)?;
            match resolve_eval_references(&resolved, resolver)? {
                FunctionValue::Array(array) => Ok(expand_aggregate_array_with_provenance(
                    &array,
                    AggregateArrayProvenance::ReferenceDerived,
                )),
                value => Ok(vec![AggregatePreparedValue::array_like(
                    CalcValue::from(value),
                    AggregateArrayProvenance::ReferenceDerived,
                )]),
            }
        }
        FunctionArg::Eval(FunctionValue::Array(array)) => {
            Ok(expand_aggregate_array_with_provenance(
                array,
                AggregateArrayProvenance::OpaqueArrayValue,
            ))
        }
        other => Ok(expand_arg_values_only(other, resolver)?
            .into_iter()
            .map(|value| AggregatePreparedValue::direct_scalar(calc_value_from_prepared(value)))
            .collect()),
    }
}

pub fn run_values_only_prepared<Out, E>(
    args: &[FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
    on_prepared: impl FnOnce(&[PreparedValue]) -> Result<Out, E>,
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

/// Convert a scalar `FunctionValue` into an array cell (for elementwise lift).
/// A scalar per-cell kernel should never yield array/reference/lambda.
fn scalar_eval_to_cell(value: FunctionValue) -> FunctionArrayCell {
    match value {
        FunctionValue::Number(n) => FunctionArrayCell::Number(n),
        FunctionValue::Text(t) => FunctionArrayCell::Text(t),
        FunctionValue::Logical(b) => FunctionArrayCell::Logical(b),
        FunctionValue::Error(code) => FunctionArrayCell::Error(code),
        _ => FunctionArrayCell::Error(WorksheetErrorCode::Value),
    }
}

/// Like [`run_values_only_prepared`], but lifts the scalar per-cell evaluator
/// elementwise over array arguments (Excel spill), broadcasting scalar args to
/// the common shape. When no argument is an array it behaves exactly like the
/// scalar path. The per-cell evaluator returns a *scalar* `FunctionValue`; on a
/// per-cell `Err`, `map_err_to_ws` decides the cell's worksheet error so the
/// array can carry per-element errors the way Excel does.
pub fn run_values_only_prepared_lifted<E>(
    args: &[FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
    on_cell: impl Fn(&[PreparedValue]) -> Result<FunctionValue, E>,
    map_err_to_ws: impl Fn(&E) -> WorksheetErrorCode,
    map_preparation_error: impl FnOnce(CoercionError) -> E,
) -> Result<FunctionValue, E> {
    let prepared = prepare_args_values_only(args, resolver).map_err(map_preparation_error)?;
    if let Some((shape, groups)) = expand_prepared_broadcast_grid(&prepared) {
        let cells = groups
            .into_iter()
            .map(|group| match group {
                BroadcastPreparedGroup::Values(values) => match on_cell(&values) {
                    Ok(value) => scalar_eval_to_cell(value),
                    Err(error) => FunctionArrayCell::Error(map_err_to_ws(&error)),
                },
                BroadcastPreparedGroup::MissingCoordinate => {
                    FunctionArrayCell::Error(WorksheetErrorCode::NA)
                }
            })
            .collect();
        return Ok(FunctionValue::Array(
            FunctionArray::new(shape, cells).expect("broadcast shape preserved"),
        ));
    }
    on_cell(&prepared)
}

pub fn map_values_only_prepared<Out>(
    args: &[FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
    on_prepared_arg: impl Fn(&PreparedValue) -> Out,
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
    Pair(PreparedValue, PreparedValue),
    MissingCoordinate,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BroadcastPreparedGroup {
    Values(Vec<PreparedValue>),
    MissingCoordinate,
}

fn prepared_shape(value: &PreparedValue) -> ArrayShape {
    match value {
        PreparedValue::Eval(FunctionValue::Array(array)) => array.shape(),
        _ => ArrayShape { rows: 1, cols: 1 },
    }
}

fn prepared_broadcast_value_at(
    value: &PreparedValue,
    row: usize,
    col: usize,
) -> Option<PreparedValue> {
    match value {
        PreparedValue::Eval(FunctionValue::Array(array)) => {
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
    lhs: &PreparedValue,
    rhs: &PreparedValue,
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
    args: &[PreparedValue],
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
    ) -> Result<FunctionValue, crate::resolver::ReferenceResolutionError> {
        Err(
            crate::resolver::ReferenceResolutionError::CapabilityDenied {
                kind: crate::value::ReferenceKind::A1,
                capability: "values_only_pre_adapter_invariant",
            },
        )
    }
}

pub fn coerce_prepared_to_number(arg: &PreparedValue) -> Result<f64, CoercionError> {
    match arg {
        PreparedValue::Eval(v) => coerce_eval_to_number(v, &NoReferenceSystemProvider),
        PreparedValue::MissingArg => Err(CoercionError::MissingArg),
        PreparedValue::EmptyCell => Err(CoercionError::EmptyCell),
    }
}

pub fn coerce_prepared_to_text(
    arg: &PreparedValue,
) -> Result<crate::value::ExcelText, CoercionError> {
    use crate::value::ExcelText;

    match arg {
        PreparedValue::Eval(FunctionValue::Text(t)) => Ok(t.clone()),
        PreparedValue::Eval(FunctionValue::Number(n)) => Ok(ExcelText::from_utf16_code_units(
            format!("{n}").encode_utf16().collect(),
        )),
        PreparedValue::Eval(FunctionValue::Logical(b)) => Ok(ExcelText::from_utf16_code_units(
            if *b { "TRUE" } else { "FALSE" }.encode_utf16().collect(),
        )),
        PreparedValue::Eval(FunctionValue::Error(code)) => {
            Err(CoercionError::WorksheetError(*code))
        }
        PreparedValue::Eval(FunctionValue::Array(_)) => {
            Err(CoercionError::UnsupportedValueKind("array"))
        }
        PreparedValue::Eval(FunctionValue::Reference(_)) => Err(CoercionError::RefResolution(
            crate::resolver::ReferenceResolutionError::EvalTimeDerefNotAllowed,
        )),
        PreparedValue::MissingArg => Ok(ExcelText::from_utf16_code_units(Vec::new())),
        PreparedValue::EmptyCell => Ok(ExcelText::from_utf16_code_units(Vec::new())),
        _ => Err(CoercionError::UnsupportedValueKind("unsupported_value")),
    }
}

pub fn apply_unary_numeric_scalar_prepared(
    arg: &PreparedValue,
    kernel: fn(f64) -> f64,
) -> Result<f64, CoercionError> {
    let n = coerce_prepared_to_number(arg)?;
    Ok(kernel(n))
}

pub fn apply_unary_numeric_array_map_prepared(
    args: &[PreparedValue],
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
        CallableArityShape, CallableValue, ExcelText, FunctionArray, OpaqueCallable, ReferenceKind,
        ReferenceLike, WorksheetErrorCode,
    };
    use std::collections::BTreeMap;
    use std::rc::Rc;

    struct MockResolver {
        caps: ReferenceSystemCapabilities,
        resolved_value: Option<FunctionValue>,
        by_target: BTreeMap<String, FunctionValue>,
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
        ) -> Result<FunctionValue, crate::resolver::ReferenceResolutionError> {
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

    fn resolver_with(value: FunctionValue) -> MockResolver {
        MockResolver {
            caps: ReferenceSystemCapabilities::permissive_local(),
            resolved_value: Some(value),
            by_target: BTreeMap::new(),
        }
    }

    #[test]
    fn prepare_values_only_dereferences_reference_arg() {
        let arg = FunctionArg::Reference(ReferenceLike::new(ReferenceKind::A1, "A1".to_string()));
        let prepared = prepare_arg_values_only(&arg, &resolver_with(FunctionValue::Number(3.0)));
        assert_eq!(
            prepared,
            Ok(PreparedValue::Eval(FunctionValue::Number(3.0)))
        );
    }

    #[test]
    fn prepare_values_only_normalizes_single_blank_area_to_empty_cell() {
        let arg = FunctionArg::Reference(ReferenceLike::new(ReferenceKind::A1, "A1".to_string()));
        let prepared = prepare_arg_values_only(
            &arg,
            &resolver_with(FunctionValue::Array(
                FunctionArray::from_rows(vec![vec![FunctionArrayCell::EmptyCell]]).unwrap(),
            )),
        );
        assert_eq!(prepared, Ok(PreparedValue::EmptyCell));
    }

    #[test]
    fn prepare_values_only_preserves_missing_and_empty() {
        assert_eq!(
            prepare_arg_values_only(
                &FunctionArg::MissingArg,
                &resolver_with(FunctionValue::Number(1.0))
            ),
            Ok(PreparedValue::MissingArg)
        );
        assert_eq!(
            prepare_arg_values_only(
                &FunctionArg::EmptyCell,
                &resolver_with(FunctionValue::Number(1.0))
            ),
            Ok(PreparedValue::EmptyCell)
        );
    }

    #[test]
    fn prepare_values_only_accepts_provider_materialized_multi_area_reference() {
        let mut by_target = BTreeMap::new();
        by_target.insert(
            "(Alpha!A1:A2,Alpha!B2)".to_string(),
            FunctionValue::Array(
                FunctionArray::from_rows(vec![vec![
                    FunctionArrayCell::Number(7.0),
                    FunctionArrayCell::Number(11.0),
                    FunctionArrayCell::Number(13.0),
                ]])
                .unwrap(),
            ),
        );
        let resolver = MockResolver {
            caps: ReferenceSystemCapabilities::permissive_local(),
            resolved_value: None,
            by_target,
        };
        let arg = FunctionArg::Reference(ReferenceLike::new(
            ReferenceKind::MultiArea,
            "(Alpha!A1:A2,Alpha!B2)",
        ));

        let prepared = prepare_arg_values_only(&arg, &resolver);
        assert_eq!(
            prepared,
            Ok(PreparedValue::Eval(FunctionValue::Array(
                FunctionArray::from_rows(vec![vec![
                    FunctionArrayCell::Number(7.0),
                    FunctionArrayCell::Number(11.0),
                    FunctionArrayCell::Number(13.0),
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

        let prepared = prepare_calc_values_only(&args, &resolver_with(FunctionValue::Number(0.0)));

        assert_eq!(prepared, Ok(args));
    }

    #[test]
    fn prepare_call_arg_as_calc_value_values_only_normalizes_single_blank_area() {
        let arg = FunctionArg::Reference(ReferenceLike::new(ReferenceKind::A1, "A1"));
        let prepared = prepare_call_arg_as_calc_value_values_only(
            &arg,
            &resolver_with(FunctionValue::Array(
                FunctionArray::from_rows(vec![vec![FunctionArrayCell::EmptyCell]]).unwrap(),
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
            &resolver_with(FunctionValue::Number(0.0)),
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
            prepare_calc_values_only(&[arg.clone()], &resolver_with(FunctionValue::Number(0.0)));

        assert_eq!(prepared, Ok(vec![arg]));
    }

    #[test]
    fn expand_lookup_vector_accepts_provider_materialized_multi_area_reference() {
        let mut by_target = BTreeMap::new();
        by_target.insert(
            "(A1:A2,C1)".to_string(),
            FunctionValue::Array(
                FunctionArray::from_rows(vec![vec![
                    FunctionArrayCell::Number(1.0),
                    FunctionArrayCell::Number(2.0),
                    FunctionArrayCell::Number(3.0),
                ]])
                .unwrap(),
            ),
        );
        let resolver = MockResolver {
            caps: ReferenceSystemCapabilities::permissive_local(),
            resolved_value: None,
            by_target,
        };
        let arg =
            FunctionArg::Reference(ReferenceLike::new(ReferenceKind::MultiArea, "(A1:A2,C1)"));

        let prepared = expand_lookup_vector_arg(&arg, &resolver);
        assert_eq!(
            prepared,
            Ok(vec![
                PreparedValue::Eval(FunctionValue::Number(1.0)),
                PreparedValue::Eval(FunctionValue::Number(2.0)),
                PreparedValue::Eval(FunctionValue::Number(3.0)),
            ])
        );
    }

    #[test]
    fn prepared_coercion_numeric_text_and_error_paths() {
        let text = PreparedValue::Eval(FunctionValue::Text(ExcelText::from_utf16_code_units(
            "2".encode_utf16().collect(),
        )));
        assert_eq!(coerce_prepared_to_number(&text), Ok(2.0));

        let err = PreparedValue::Eval(FunctionValue::Error(WorksheetErrorCode::Value));
        assert_eq!(
            coerce_prepared_to_number(&err),
            Err(CoercionError::WorksheetError(WorksheetErrorCode::Value))
        );
    }

    #[test]
    fn prepared_text_coercion_formats_scalars_and_blanks() {
        let number = PreparedValue::Eval(FunctionValue::Number(2.5));
        assert_eq!(
            coerce_prepared_to_text(&number),
            Ok(ExcelText::from_utf16_code_units(
                "2.5".encode_utf16().collect()
            ))
        );

        let blank = PreparedValue::EmptyCell;
        assert_eq!(
            coerce_prepared_to_text(&blank),
            Ok(ExcelText::from_utf16_code_units(Vec::new()))
        );
    }

    #[test]
    fn prepared_coercion_rejects_reference_if_invariant_broken() {
        let prepared = PreparedValue::Eval(FunctionValue::Reference(ReferenceLike::new(
            ReferenceKind::A1,
            "A1".to_string(),
        )));
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
            PreparedValue::Eval(FunctionValue::Number(-2.0)),
            PreparedValue::Eval(FunctionValue::Text(ExcelText::from_utf16_code_units(
                "asd".encode_utf16().collect(),
            ))),
            PreparedValue::Eval(FunctionValue::Logical(true)),
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
        let args = [FunctionArg::Eval(FunctionValue::Number(2.0))];
        let got = run_values_only_prepared(
            &args,
            &resolver_with(FunctionValue::Number(0.0)),
            |prepared| Ok::<f64, CoercionError>(coerce_prepared_to_number(&prepared[0])?),
            |e| e,
        );
        assert_eq!(got, Ok(2.0));
    }

    #[test]
    fn map_values_only_prepared_maps_preparation_errors_per_arg() {
        let args = vec![
            FunctionArg::Reference(ReferenceLike::new(ReferenceKind::A1, "A1".to_string())),
            FunctionArg::Eval(FunctionValue::Number(2.0)),
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
        let arg = FunctionArg::Eval(FunctionValue::Array(
            FunctionArray::from_rows(vec![
                vec![FunctionArrayCell::Number(1.0), FunctionArrayCell::EmptyCell],
                vec![
                    FunctionArrayCell::Text(ExcelText::from_utf16_code_units(
                        "x".encode_utf16().collect(),
                    )),
                    FunctionArrayCell::Logical(true),
                ],
            ])
            .unwrap(),
        ));
        let got = expand_arg_values_only(&arg, &resolver_with(FunctionValue::Number(0.0))).unwrap();
        assert_eq!(
            got,
            vec![
                PreparedValue::Eval(FunctionValue::Number(1.0)),
                PreparedValue::EmptyCell,
                PreparedValue::Eval(FunctionValue::Text(ExcelText::from_utf16_code_units(
                    "x".encode_utf16().collect(),
                ))),
                PreparedValue::Eval(FunctionValue::Logical(true)),
            ]
        );
    }

    #[test]
    fn expand_aggregate_arg_marks_reference_derived_values() {
        let arg =
            FunctionArg::Reference(ReferenceLike::new(ReferenceKind::Area, "A1:A2".to_string()));
        let got = expand_aggregate_arg(
            &arg,
            &resolver_with(FunctionValue::Array(
                FunctionArray::from_rows(vec![
                    vec![FunctionArrayCell::Number(1.0)],
                    vec![FunctionArrayCell::Text(ExcelText::from_utf16_code_units(
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
        let arg = FunctionArg::Eval(FunctionValue::Reference(ReferenceLike::new(
            ReferenceKind::Area,
            "NameBackedRange".to_string(),
        )));
        let got = expand_aggregate_arg(
            &arg,
            &resolver_with(FunctionValue::Array(
                FunctionArray::from_rows(vec![
                    vec![FunctionArrayCell::Number(1.0)],
                    vec![FunctionArrayCell::Number(2.0)],
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
            &FunctionArg::Eval(FunctionValue::Array(
                FunctionArray::from_rows(vec![vec![
                    FunctionArrayCell::Text(ExcelText::from_utf16_code_units(
                        "2".encode_utf16().collect(),
                    )),
                    FunctionArrayCell::Logical(true),
                ]])
                .unwrap(),
            )),
            &resolver_with(FunctionValue::Number(0.0)),
        )
        .unwrap();
        assert_eq!(got.len(), 2);
        assert!(got.iter().all(|item| item.origin()
            == AggregateArgOrigin::ArrayLike(AggregateArrayProvenance::OpaqueArrayValue)));
    }

    #[test]
    fn expand_aggregate_array_with_provenance_marks_direct_array_literal() {
        let array = FunctionArray::from_rows(vec![vec![
            FunctionArrayCell::Text(ExcelText::from_utf16_code_units(
                "2".encode_utf16().collect(),
            )),
            FunctionArrayCell::Logical(true),
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
        let arg = FunctionArg::Eval(FunctionValue::Array(
            FunctionArray::from_rows(vec![
                vec![
                    FunctionArrayCell::Number(1.0),
                    FunctionArrayCell::Number(2.0),
                ],
                vec![
                    FunctionArrayCell::Number(3.0),
                    FunctionArrayCell::Number(4.0),
                ],
            ])
            .unwrap(),
        ));
        let got = expand_lookup_vector_arg(&arg, &resolver_with(FunctionValue::Number(0.0)));
        assert_eq!(
            got,
            Err(CoercionError::UnsupportedValueKind("two_dimensional_array"))
        );
    }
}

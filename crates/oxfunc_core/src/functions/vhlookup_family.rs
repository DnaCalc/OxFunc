use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{coerce_prepared_to_number, prepare_arg_values_only};
use crate::functions::match_fn::{eval_match_surface, map_match_error_to_ws};
use crate::resolver::{ReferenceSystemProvider, resolve_eval_value};
use crate::value::CalcValue;
use crate::value::{ArrayShape, CalcArray, CoreValue, WorksheetErrorCode};

const VHLOOKUP_BASE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.VHLOOKUP_BASE",
    arity: Arity { min: 3, max: 4 },
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::WorkbookState,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::RefsVisibleInAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::LookupMatch,
    fec_dependency_profile: FecDependencyProfile::RefOnly,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
};

pub const VLOOKUP_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.VLOOKUP",
    ..VHLOOKUP_BASE_META
};

pub const HLOOKUP_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.HLOOKUP",
    ..VHLOOKUP_BASE_META
};

#[derive(Debug, Clone, PartialEq)]
pub enum VhlookupEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
    Domain(WorksheetErrorCode),
}

fn resolve_table_value(
    arg: &CalcValue,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, VhlookupEvalError> {
    match arg.core() {
        CoreValue::Reference(reference) => {
            let resolved = resolve_eval_value(resolver, reference)
                .map_err(CoercionError::RefResolution)
                .map_err(VhlookupEvalError::Coercion)?;
            resolve_table_value(&(resolved), resolver)
        }
        CoreValue::Empty => Ok(CalcValue::array(
            CalcArray::from_scalar(CalcValue::empty())
                .expect("scalar table materialization always has one cell"),
        )),
        CoreValue::Missing => Err(VhlookupEvalError::Coercion(CoercionError::MissingArg)),
        _ => Ok(arg.clone()),
    }
}

fn scalar_to_cell(value: &CalcValue) -> Result<CalcValue, WorksheetErrorCode> {
    match value.core() {
        CoreValue::Number(n) => Ok(CalcValue::number(*n)),
        CoreValue::Text(t) => Ok(CalcValue::text(t.clone())),
        CoreValue::Logical(b) => Ok(CalcValue::logical(*b)),
        CoreValue::Error(code) => Ok(CalcValue::error(*code)),
        CoreValue::Array(_) | CoreValue::Reference(_) => Err(WorksheetErrorCode::Value),
        CoreValue::Missing | CoreValue::Empty => Err(WorksheetErrorCode::Value),
    }
}

fn table_from_arg(
    arg: &CalcValue,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<Vec<Vec<CalcValue>>, VhlookupEvalError> {
    let value = resolve_table_value(arg, resolver)?;
    match value.core() {
        CoreValue::Array(array) => {
            let shape = array.shape();
            let mut rows = Vec::with_capacity(shape.rows);
            for row_idx in 0..shape.rows {
                rows.push(
                    array
                        .row_slice(row_idx)
                        .expect("row index bounded by shape rows")
                        .to_vec(),
                );
            }
            Ok(rows)
        }
        _ => Ok(vec![vec![
            scalar_to_cell(&value).map_err(VhlookupEvalError::Domain)?,
        ]]),
    }
}

fn vector_arg_from_cells(cells: &[CalcValue], vertical: bool) -> CalcValue {
    let rows = if vertical { cells.len() } else { 1 };
    let cols = if vertical { 1 } else { cells.len() };
    let array =
        CalcArray::new(ArrayShape { rows, cols }, cells.to_vec()).expect("shape matches cells");
    CalcValue::array(array)
}

fn cell_to_eval_value(cell: &CalcValue) -> CalcValue {
    match cell.core() {
        CoreValue::Number(n) => CalcValue::number(*n),
        CoreValue::Text(t) => CalcValue::text(t.clone()),
        CoreValue::Logical(b) => CalcValue::logical(*b),
        CoreValue::Error(code) => CalcValue::error(*code),
        CoreValue::Empty | CoreValue::Missing => CalcValue::number(0.0),
        CoreValue::Array(_) | CoreValue::Reference(_) => {
            CalcValue::error(WorksheetErrorCode::Value)
        }
    }
}

fn parse_lookup_index(
    arg: &CalcValue,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<usize, VhlookupEvalError> {
    let prepared = prepare_arg_values_only(arg, resolver).map_err(VhlookupEvalError::Coercion)?;
    let n = coerce_prepared_to_number(&prepared).map_err(VhlookupEvalError::Coercion)?;
    if !n.is_finite() {
        return Err(VhlookupEvalError::Domain(WorksheetErrorCode::Value));
    }
    let truncated = n.trunc();
    if truncated < 1.0 {
        return Err(VhlookupEvalError::Domain(WorksheetErrorCode::Value));
    }
    if truncated > (usize::MAX as f64) {
        return Err(VhlookupEvalError::Domain(WorksheetErrorCode::Ref));
    }
    Ok(truncated as usize)
}

fn parse_range_lookup_match_type(
    arg: Option<&CalcValue>,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, VhlookupEvalError> {
    let approximate = match arg {
        None => true,
        Some(value) => {
            let prepared =
                prepare_arg_values_only(value, resolver).map_err(VhlookupEvalError::Coercion)?;
            let n = coerce_prepared_to_number(&prepared).map_err(VhlookupEvalError::Coercion)?;
            n != 0.0
        }
    };
    Ok(CalcValue::number(if approximate { 1.0 } else { 0.0 }))
}

fn match_index(
    lookup_value: &CalcValue,
    lookup_vector: CalcValue,
    match_type: &CalcValue,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, VhlookupEvalError> {
    eval_match_surface(lookup_value, &[lookup_vector], Some(match_type), resolver)
        .map_err(|e| VhlookupEvalError::Domain(map_match_error_to_ws(&e)))
}

fn match_result_to_index(cell: &CalcValue) -> Result<usize, WorksheetErrorCode> {
    match cell.core() {
        CoreValue::Number(n) if *n >= 1.0 && *n <= (usize::MAX as f64) => Ok(*n as usize),
        CoreValue::Error(code) => Err(*code),
        _ => Err(WorksheetErrorCode::Value),
    }
}

fn match_value_to_index(value: CalcValue) -> Result<usize, WorksheetErrorCode> {
    match value.core() {
        CoreValue::Number(n) if *n >= 1.0 && *n <= (usize::MAX as f64) => Ok(*n as usize),
        CoreValue::Error(code) => Err(*code),
        _ => Err(WorksheetErrorCode::Value),
    }
}

fn select_from_match_result(
    match_result: CalcValue,
    select_index: impl Fn(usize) -> CalcValue,
) -> Result<CalcValue, VhlookupEvalError> {
    match match_result.core() {
        CoreValue::Array(array) => {
            let cells = array
                .iter_row_major()
                .map(|cell| match match_result_to_index(cell) {
                    Ok(index) => select_index(index),
                    Err(code) => CalcValue::error(code),
                })
                .collect();
            Ok(CalcValue::array(
                CalcArray::new(array.shape(), cells)
                    .expect("match-result array shape remains valid"),
            ))
        }
        _ => match match_value_to_index(match_result) {
            Ok(index) => Ok(cell_to_eval_value(&select_index(index))),
            Err(code) => Err(VhlookupEvalError::Domain(code)),
        },
    }
}

pub fn eval_vlookup_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, VhlookupEvalError> {
    if !VLOOKUP_META.arity.accepts(args.len()) {
        return Err(VhlookupEvalError::ArityMismatch {
            expected_min: VLOOKUP_META.arity.min,
            expected_max: VLOOKUP_META.arity.max,
            actual: args.len(),
        });
    }
    let table = table_from_arg(&args[1], resolver)?;
    let height = table.len();
    let width = table.first().map_or(0, Vec::len);
    let column_index = parse_lookup_index(&args[2], resolver)?;
    if column_index > width {
        return Err(VhlookupEvalError::Domain(WorksheetErrorCode::Ref));
    }
    let match_type = parse_range_lookup_match_type(args.get(3), resolver)?;
    let first_column = table.iter().map(|row| row[0].clone()).collect::<Vec<_>>();
    let row_index = match_index(
        &args[0],
        vector_arg_from_cells(&first_column, true),
        &match_type,
        resolver,
    )?;
    select_from_match_result(row_index, |index| {
        if index == 0 || index > height {
            return CalcValue::error(WorksheetErrorCode::NA);
        }
        table[index - 1][column_index - 1].clone()
    })
}

pub fn eval_hlookup_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, VhlookupEvalError> {
    if !HLOOKUP_META.arity.accepts(args.len()) {
        return Err(VhlookupEvalError::ArityMismatch {
            expected_min: HLOOKUP_META.arity.min,
            expected_max: HLOOKUP_META.arity.max,
            actual: args.len(),
        });
    }
    let table = table_from_arg(&args[1], resolver)?;
    let height = table.len();
    let width = table.first().map_or(0, Vec::len);
    let row_index = parse_lookup_index(&args[2], resolver)?;
    if row_index > height {
        return Err(VhlookupEvalError::Domain(WorksheetErrorCode::Ref));
    }
    let match_type = parse_range_lookup_match_type(args.get(3), resolver)?;
    let first_row = table
        .first()
        .cloned()
        .ok_or(VhlookupEvalError::Domain(WorksheetErrorCode::Value))?;
    let column_index = match_index(
        &args[0],
        vector_arg_from_cells(&first_row, false),
        &match_type,
        resolver,
    )?;
    select_from_match_result(column_index, |index| {
        if index == 0 || index > width {
            return CalcValue::error(WorksheetErrorCode::NA);
        }
        table[row_index - 1][index - 1].clone()
    })
}

pub fn map_vhlookup_error_to_ws(error: &VhlookupEvalError) -> WorksheetErrorCode {
    match error {
        VhlookupEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        VhlookupEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        VhlookupEvalError::Coercion(_) => WorksheetErrorCode::Value,
        VhlookupEvalError::Domain(code) => *code,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::ExcelText;

    struct NoResolver;

    impl ReferenceSystemProvider for NoResolver {
        fn capabilities(&self) -> crate::resolver::ReferenceSystemCapabilities {
            crate::resolver::ReferenceSystemCapabilities::permissive_local()
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
    }

    fn txt(s: &str) -> CalcValue {
        CalcValue::text(ExcelText::from_utf16_code_units(s.encode_utf16().collect()))
    }

    fn scalar_num(n: f64) -> CalcValue {
        CalcValue::number(n)
    }

    fn scalar_bool(b: bool) -> CalcValue {
        CalcValue::logical(b)
    }

    fn scalar_text(s: &str) -> CalcValue {
        CalcValue::text(ExcelText::from_utf16_code_units(s.encode_utf16().collect()))
    }

    fn vtable() -> CalcValue {
        CalcValue::array(
            CalcArray::from_rows(vec![
                vec![CalcValue::number(1.0), CalcValue::number(10.0)],
                vec![CalcValue::number(2.0), CalcValue::number(20.0)],
                vec![CalcValue::number(3.0), CalcValue::number(30.0)],
            ])
            .unwrap(),
        )
    }

    fn htable() -> CalcValue {
        CalcValue::array(
            CalcArray::from_rows(vec![
                vec![
                    CalcValue::number(1.0),
                    CalcValue::number(2.0),
                    CalcValue::number(3.0),
                ],
                vec![
                    CalcValue::number(10.0),
                    CalcValue::number(20.0),
                    CalcValue::number(30.0),
                ],
            ])
            .unwrap(),
        )
    }

    #[test]
    fn vlookup_exact_and_approximate_match_excel_seed_rows() {
        assert_eq!(
            eval_vlookup_surface(
                &[
                    scalar_num(2.0),
                    vtable(),
                    scalar_num(2.0),
                    scalar_bool(false)
                ],
                &NoResolver,
            ),
            Ok(CalcValue::number(20.0))
        );
        assert_eq!(
            eval_vlookup_surface(&[scalar_num(2.9), vtable(), scalar_num(2.0)], &NoResolver,),
            Ok(CalcValue::number(20.0))
        );
    }

    #[test]
    fn hlookup_exact_and_approximate_match_excel_seed_rows() {
        assert_eq!(
            eval_hlookup_surface(
                &[
                    scalar_num(2.0),
                    htable(),
                    scalar_num(2.0),
                    scalar_bool(false)
                ],
                &NoResolver,
            ),
            Ok(CalcValue::number(20.0))
        );
        assert_eq!(
            eval_hlookup_surface(&[scalar_num(2.9), htable(), scalar_num(2.0)], &NoResolver,),
            Ok(CalcValue::number(20.0))
        );
    }

    #[test]
    fn approximate_lookup_below_first_key_returns_not_available() {
        assert_eq!(
            eval_vlookup_surface(&[scalar_num(0.5), vtable(), scalar_num(2.0)], &NoResolver),
            Err(VhlookupEvalError::Domain(WorksheetErrorCode::NA))
        );
        assert_eq!(
            eval_hlookup_surface(&[scalar_num(0.5), htable(), scalar_num(2.0)], &NoResolver),
            Err(VhlookupEvalError::Domain(WorksheetErrorCode::NA))
        );
    }

    #[test]
    fn exact_lookup_no_match_returns_not_available() {
        assert_eq!(
            eval_vlookup_surface(
                &[
                    scalar_num(4.0),
                    vtable(),
                    scalar_num(2.0),
                    scalar_bool(false)
                ],
                &NoResolver,
            ),
            Err(VhlookupEvalError::Domain(WorksheetErrorCode::NA))
        );
        assert_eq!(
            eval_hlookup_surface(
                &[
                    scalar_num(4.0),
                    htable(),
                    scalar_num(2.0),
                    scalar_bool(false)
                ],
                &NoResolver,
            ),
            Err(VhlookupEvalError::Domain(WorksheetErrorCode::NA))
        );
    }

    #[test]
    fn lookup_indices_truncate_and_validate_bounds() {
        assert_eq!(
            eval_vlookup_surface(
                &[
                    scalar_num(2.0),
                    vtable(),
                    scalar_num(1.9),
                    scalar_bool(false)
                ],
                &NoResolver,
            ),
            Ok(CalcValue::number(2.0))
        );
        assert_eq!(
            eval_hlookup_surface(
                &[
                    scalar_num(2.0),
                    htable(),
                    scalar_num(1.9),
                    scalar_bool(false)
                ],
                &NoResolver,
            ),
            Ok(CalcValue::number(2.0))
        );
        assert_eq!(
            eval_vlookup_surface(
                &[
                    scalar_num(2.0),
                    vtable(),
                    scalar_num(3.0),
                    scalar_bool(false)
                ],
                &NoResolver,
            ),
            Err(VhlookupEvalError::Domain(WorksheetErrorCode::Ref))
        );
        assert_eq!(
            eval_hlookup_surface(
                &[
                    scalar_num(2.0),
                    htable(),
                    scalar_num(3.0),
                    scalar_bool(false)
                ],
                &NoResolver,
            ),
            Err(VhlookupEvalError::Domain(WorksheetErrorCode::Ref))
        );
    }

    #[test]
    fn lookup_indices_below_one_return_value_error() {
        assert_eq!(
            eval_vlookup_surface(
                &[
                    scalar_num(2.0),
                    vtable(),
                    scalar_num(0.0),
                    scalar_bool(false)
                ],
                &NoResolver,
            ),
            Err(VhlookupEvalError::Domain(WorksheetErrorCode::Value))
        );
        assert_eq!(
            eval_vlookup_surface(
                &[
                    scalar_num(2.0),
                    vtable(),
                    scalar_num(-1.0),
                    scalar_bool(false)
                ],
                &NoResolver,
            ),
            Err(VhlookupEvalError::Domain(WorksheetErrorCode::Value))
        );
        assert_eq!(
            eval_hlookup_surface(
                &[
                    scalar_num(2.0),
                    htable(),
                    scalar_num(0.0),
                    scalar_bool(false)
                ],
                &NoResolver,
            ),
            Err(VhlookupEvalError::Domain(WorksheetErrorCode::Value))
        );
        assert_eq!(
            eval_hlookup_surface(
                &[
                    scalar_num(2.0),
                    htable(),
                    scalar_num(-1.0),
                    scalar_bool(false)
                ],
                &NoResolver,
            ),
            Err(VhlookupEvalError::Domain(WorksheetErrorCode::Value))
        );
    }

    #[test]
    fn lookup_exact_supports_wildcards() {
        let vtable = CalcValue::array(
            CalcArray::from_rows(vec![
                vec![txt("abc"), CalcValue::number(10.0)],
                vec![txt("bcd"), CalcValue::number(20.0)],
            ])
            .unwrap(),
        );
        let htable = CalcValue::array(
            CalcArray::from_rows(vec![
                vec![txt("abc"), txt("bcd")],
                vec![CalcValue::number(10.0), CalcValue::number(20.0)],
            ])
            .unwrap(),
        );
        assert_eq!(
            eval_vlookup_surface(
                &[
                    scalar_text("b*"),
                    vtable,
                    scalar_num(2.0),
                    scalar_bool(false)
                ],
                &NoResolver,
            ),
            Ok(CalcValue::number(20.0))
        );
        assert_eq!(
            eval_hlookup_surface(
                &[
                    scalar_text("b*"),
                    htable,
                    scalar_num(2.0),
                    scalar_bool(false)
                ],
                &NoResolver,
            ),
            Ok(CalcValue::number(20.0))
        );
    }

    #[test]
    fn vlookup_exact_matches_logicals() {
        let table = CalcValue::array(
            CalcArray::from_rows(vec![
                vec![CalcValue::logical(true), CalcValue::number(1.0)],
                vec![CalcValue::logical(false), CalcValue::number(2.0)],
            ])
            .unwrap(),
        );
        assert_eq!(
            eval_vlookup_surface(
                &[
                    scalar_bool(true),
                    table,
                    scalar_num(2.0),
                    scalar_bool(false)
                ],
                &NoResolver,
            ),
            Ok(CalcValue::number(1.0))
        );
    }

    #[test]
    fn vlookup_spills_array_lookup_value_results() {
        let got = eval_vlookup_surface(
            &[
                (CalcValue::array(
                    CalcArray::from_rows(vec![vec![
                        CalcValue::number(1.0),
                        CalcValue::number(2.0),
                        CalcValue::number(3.0),
                    ]])
                    .unwrap(),
                )),
                (CalcValue::array(
                    CalcArray::from_rows(vec![
                        vec![CalcValue::number(2.0), CalcValue::number(20.0)],
                        vec![CalcValue::number(4.0), CalcValue::number(40.0)],
                        vec![CalcValue::number(6.0), CalcValue::number(60.0)],
                        vec![CalcValue::number(8.0), CalcValue::number(80.0)],
                    ])
                    .unwrap(),
                )),
                scalar_num(2.0),
                scalar_bool(false),
            ],
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(CalcValue::array(
                CalcArray::from_rows(vec![vec![
                    CalcValue::error(WorksheetErrorCode::NA),
                    CalcValue::number(20.0),
                    CalcValue::error(WorksheetErrorCode::NA),
                ]])
                .unwrap()
            ))
        );
    }

    #[test]
    fn hlookup_spills_array_lookup_value_results() {
        let got = eval_hlookup_surface(
            &[
                (CalcValue::array(
                    CalcArray::from_rows(vec![vec![
                        CalcValue::number(1.0),
                        CalcValue::number(2.0),
                        CalcValue::number(3.0),
                    ]])
                    .unwrap(),
                )),
                (CalcValue::array(
                    CalcArray::from_rows(vec![
                        vec![
                            CalcValue::number(2.0),
                            CalcValue::number(4.0),
                            CalcValue::number(6.0),
                            CalcValue::number(8.0),
                        ],
                        vec![
                            CalcValue::number(20.0),
                            CalcValue::number(40.0),
                            CalcValue::number(60.0),
                            CalcValue::number(80.0),
                        ],
                    ])
                    .unwrap(),
                )),
                scalar_num(2.0),
                scalar_bool(false),
            ],
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(CalcValue::array(
                CalcArray::from_rows(vec![vec![
                    CalcValue::error(WorksheetErrorCode::NA),
                    CalcValue::number(20.0),
                    CalcValue::error(WorksheetErrorCode::NA),
                ]])
                .unwrap()
            ))
        );
    }

    #[test]
    fn vhlookup_meta_matches_batch_shape() {
        assert_eq!(VLOOKUP_META.function_id, "FUNC.VLOOKUP");
        assert_eq!(HLOOKUP_META.function_id, "FUNC.HLOOKUP");
        assert_eq!(VLOOKUP_META.arity, Arity { min: 3, max: 4 });
        assert_eq!(
            HLOOKUP_META.arg_preparation_profile,
            ArgPreparationProfile::RefsVisibleInAdapter
        );
    }
}

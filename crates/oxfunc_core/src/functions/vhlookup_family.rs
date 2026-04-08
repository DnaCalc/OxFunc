use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{coerce_prepared_to_number, prepare_arg_values_only};
use crate::functions::match_fn::{eval_match_surface, map_match_error_to_ws};
use crate::resolver::{ReferenceResolver, resolve_eval_value};
use crate::value::{
    ArrayCellValue, ArrayShape, CallArgValue, EvalArray, EvalValue, WorksheetErrorCode,
};

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
    arg: &CallArgValue,
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, VhlookupEvalError> {
    match arg {
        CallArgValue::Reference(reference)
        | CallArgValue::Eval(EvalValue::Reference(reference)) => {
            let resolved = resolve_eval_value(resolver, reference)
                .map_err(CoercionError::RefResolution)
                .map_err(VhlookupEvalError::Coercion)?;
            resolve_table_value(&CallArgValue::Eval(resolved), resolver)
        }
        CallArgValue::Eval(value) => Ok(value.clone()),
        CallArgValue::EmptyCell => Ok(EvalValue::Array(EvalArray::from_scalar(
            ArrayCellValue::EmptyCell,
        ))),
        CallArgValue::MissingArg => Err(VhlookupEvalError::Coercion(CoercionError::MissingArg)),
    }
}

fn scalar_to_cell(value: &EvalValue) -> Result<ArrayCellValue, WorksheetErrorCode> {
    match value {
        EvalValue::Number(n) => Ok(ArrayCellValue::Number(*n)),
        EvalValue::Text(t) => Ok(ArrayCellValue::Text(t.clone())),
        EvalValue::Logical(b) => Ok(ArrayCellValue::Logical(*b)),
        EvalValue::Error(code) => Ok(ArrayCellValue::Error(*code)),
        EvalValue::Array(_) | EvalValue::Reference(_) | EvalValue::Lambda(_) => {
            Err(WorksheetErrorCode::Value)
        }
    }
}

fn table_from_arg(
    arg: &CallArgValue,
    resolver: &impl ReferenceResolver,
) -> Result<Vec<Vec<ArrayCellValue>>, VhlookupEvalError> {
    let value = resolve_table_value(arg, resolver)?;
    match value {
        EvalValue::Array(array) => {
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
        scalar => Ok(vec![vec![
            scalar_to_cell(&scalar).map_err(VhlookupEvalError::Domain)?,
        ]]),
    }
}

fn vector_arg_from_cells(cells: &[ArrayCellValue], vertical: bool) -> CallArgValue {
    let rows = if vertical { cells.len() } else { 1 };
    let cols = if vertical { 1 } else { cells.len() };
    let array =
        EvalArray::new(ArrayShape { rows, cols }, cells.to_vec()).expect("shape matches cells");
    CallArgValue::Eval(EvalValue::Array(array))
}

fn cell_to_eval_value(cell: &ArrayCellValue) -> EvalValue {
    match cell {
        ArrayCellValue::Number(n) => EvalValue::Number(*n),
        ArrayCellValue::Text(t) => EvalValue::Text(t.clone()),
        ArrayCellValue::Logical(b) => EvalValue::Logical(*b),
        ArrayCellValue::Error(code) => EvalValue::Error(*code),
        ArrayCellValue::EmptyCell => EvalValue::Number(0.0),
    }
}

fn parse_lookup_index(
    arg: &CallArgValue,
    resolver: &impl ReferenceResolver,
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
    arg: Option<&CallArgValue>,
    resolver: &impl ReferenceResolver,
) -> Result<CallArgValue, VhlookupEvalError> {
    let approximate = match arg {
        None => true,
        Some(value) => {
            let prepared =
                prepare_arg_values_only(value, resolver).map_err(VhlookupEvalError::Coercion)?;
            let n = coerce_prepared_to_number(&prepared).map_err(VhlookupEvalError::Coercion)?;
            n != 0.0
        }
    };
    Ok(CallArgValue::Eval(EvalValue::Number(if approximate {
        1.0
    } else {
        0.0
    })))
}

fn match_index(
    lookup_value: &CallArgValue,
    lookup_vector: CallArgValue,
    match_type: &CallArgValue,
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, VhlookupEvalError> {
    eval_match_surface(lookup_value, &[lookup_vector], Some(match_type), resolver)
        .map_err(|e| VhlookupEvalError::Domain(map_match_error_to_ws(&e)))
}

fn match_result_to_index(cell: &ArrayCellValue) -> Result<usize, WorksheetErrorCode> {
    match cell {
        ArrayCellValue::Number(n) if *n >= 1.0 && *n <= (usize::MAX as f64) => Ok(*n as usize),
        ArrayCellValue::Error(code) => Err(*code),
        _ => Err(WorksheetErrorCode::Value),
    }
}

fn match_value_to_index(value: EvalValue) -> Result<usize, WorksheetErrorCode> {
    match value {
        EvalValue::Number(n) if n >= 1.0 && n <= (usize::MAX as f64) => Ok(n as usize),
        EvalValue::Error(code) => Err(code),
        _ => Err(WorksheetErrorCode::Value),
    }
}

fn select_from_match_result(
    match_result: EvalValue,
    select_index: impl Fn(usize) -> ArrayCellValue,
) -> Result<EvalValue, VhlookupEvalError> {
    match match_result {
        EvalValue::Array(array) => {
            let cells = array
                .iter_row_major()
                .map(|cell| match match_result_to_index(cell) {
                    Ok(index) => select_index(index),
                    Err(code) => ArrayCellValue::Error(code),
                })
                .collect();
            Ok(EvalValue::Array(
                EvalArray::new(array.shape(), cells)
                    .expect("match-result array shape remains valid"),
            ))
        }
        scalar => match match_value_to_index(scalar) {
            Ok(index) => Ok(cell_to_eval_value(&select_index(index))),
            Err(code) => Err(VhlookupEvalError::Domain(code)),
        },
    }
}

pub fn eval_vlookup_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, VhlookupEvalError> {
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
            return ArrayCellValue::Error(WorksheetErrorCode::NA);
        }
        table[index - 1][column_index - 1].clone()
    })
}

pub fn eval_hlookup_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, VhlookupEvalError> {
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
            return ArrayCellValue::Error(WorksheetErrorCode::NA);
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

    impl ReferenceResolver for NoResolver {
        fn capabilities(&self) -> crate::resolver::ResolverCapabilities {
            crate::resolver::ResolverCapabilities::permissive_local()
        }

        fn resolve_reference(
            &self,
            reference: &crate::value::ReferenceLike,
        ) -> Result<EvalValue, crate::resolver::RefResolutionError> {
            Err(crate::resolver::RefResolutionError::UnresolvedReference {
                target: reference.target.clone(),
            })
        }
    }

    fn txt(s: &str) -> ArrayCellValue {
        ArrayCellValue::Text(ExcelText::from_utf16_code_units(s.encode_utf16().collect()))
    }

    fn scalar_num(n: f64) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Number(n))
    }

    fn scalar_bool(b: bool) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Logical(b))
    }

    fn scalar_text(s: &str) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
            s.encode_utf16().collect(),
        )))
    }

    fn vtable() -> CallArgValue {
        CallArgValue::Eval(EvalValue::Array(
            EvalArray::from_rows(vec![
                vec![ArrayCellValue::Number(1.0), ArrayCellValue::Number(10.0)],
                vec![ArrayCellValue::Number(2.0), ArrayCellValue::Number(20.0)],
                vec![ArrayCellValue::Number(3.0), ArrayCellValue::Number(30.0)],
            ])
            .unwrap(),
        ))
    }

    fn htable() -> CallArgValue {
        CallArgValue::Eval(EvalValue::Array(
            EvalArray::from_rows(vec![
                vec![
                    ArrayCellValue::Number(1.0),
                    ArrayCellValue::Number(2.0),
                    ArrayCellValue::Number(3.0),
                ],
                vec![
                    ArrayCellValue::Number(10.0),
                    ArrayCellValue::Number(20.0),
                    ArrayCellValue::Number(30.0),
                ],
            ])
            .unwrap(),
        ))
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
            Ok(EvalValue::Number(20.0))
        );
        assert_eq!(
            eval_vlookup_surface(&[scalar_num(2.9), vtable(), scalar_num(2.0)], &NoResolver,),
            Ok(EvalValue::Number(20.0))
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
            Ok(EvalValue::Number(20.0))
        );
        assert_eq!(
            eval_hlookup_surface(&[scalar_num(2.9), htable(), scalar_num(2.0)], &NoResolver,),
            Ok(EvalValue::Number(20.0))
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
            Ok(EvalValue::Number(2.0))
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
            Ok(EvalValue::Number(2.0))
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
        let vtable = CallArgValue::Eval(EvalValue::Array(
            EvalArray::from_rows(vec![
                vec![txt("abc"), ArrayCellValue::Number(10.0)],
                vec![txt("bcd"), ArrayCellValue::Number(20.0)],
            ])
            .unwrap(),
        ));
        let htable = CallArgValue::Eval(EvalValue::Array(
            EvalArray::from_rows(vec![
                vec![txt("abc"), txt("bcd")],
                vec![ArrayCellValue::Number(10.0), ArrayCellValue::Number(20.0)],
            ])
            .unwrap(),
        ));
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
            Ok(EvalValue::Number(20.0))
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
            Ok(EvalValue::Number(20.0))
        );
    }

    #[test]
    fn vlookup_exact_matches_logicals() {
        let table = CallArgValue::Eval(EvalValue::Array(
            EvalArray::from_rows(vec![
                vec![ArrayCellValue::Logical(true), ArrayCellValue::Number(1.0)],
                vec![ArrayCellValue::Logical(false), ArrayCellValue::Number(2.0)],
            ])
            .unwrap(),
        ));
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
            Ok(EvalValue::Number(1.0))
        );
    }

    #[test]
    fn vlookup_spills_array_lookup_value_results() {
        let got = eval_vlookup_surface(
            &[
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![vec![
                        ArrayCellValue::Number(1.0),
                        ArrayCellValue::Number(2.0),
                        ArrayCellValue::Number(3.0),
                    ]])
                    .unwrap(),
                )),
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![
                        vec![ArrayCellValue::Number(2.0), ArrayCellValue::Number(20.0)],
                        vec![ArrayCellValue::Number(4.0), ArrayCellValue::Number(40.0)],
                        vec![ArrayCellValue::Number(6.0), ArrayCellValue::Number(60.0)],
                        vec![ArrayCellValue::Number(8.0), ArrayCellValue::Number(80.0)],
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
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Error(WorksheetErrorCode::NA),
                    ArrayCellValue::Number(20.0),
                    ArrayCellValue::Error(WorksheetErrorCode::NA),
                ]])
                .unwrap()
            ))
        );
    }

    #[test]
    fn hlookup_spills_array_lookup_value_results() {
        let got = eval_hlookup_surface(
            &[
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![vec![
                        ArrayCellValue::Number(1.0),
                        ArrayCellValue::Number(2.0),
                        ArrayCellValue::Number(3.0),
                    ]])
                    .unwrap(),
                )),
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![
                        vec![
                            ArrayCellValue::Number(2.0),
                            ArrayCellValue::Number(4.0),
                            ArrayCellValue::Number(6.0),
                            ArrayCellValue::Number(8.0),
                        ],
                        vec![
                            ArrayCellValue::Number(20.0),
                            ArrayCellValue::Number(40.0),
                            ArrayCellValue::Number(60.0),
                            ArrayCellValue::Number(80.0),
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
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Error(WorksheetErrorCode::NA),
                    ArrayCellValue::Number(20.0),
                    ArrayCellValue::Error(WorksheetErrorCode::NA),
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

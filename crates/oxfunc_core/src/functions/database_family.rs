use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{PreparedArgValue, prepare_arg_values_only};
use crate::functions::variance_common::{VarianceDivisor, stdev_from_values, variance_from_values};
use crate::functions::xmatch::wildcard_match;
use crate::resolver::{ReferenceResolver, resolve_eval_value};
use crate::value::{
    ArrayCellValue, CallArgValue, EvalArray, EvalValue, ExcelText, WorksheetErrorCode,
};

const DATABASE_META_BASE: FunctionMeta = FunctionMeta {
    function_id: "FUNC.DATABASE_BASE",
    arity: Arity::exact(3),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::RefsVisibleInAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::RefOnly,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
};

pub const DAVERAGE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.DAVERAGE",
    ..DATABASE_META_BASE
};
pub const DCOUNT_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.DCOUNT",
    ..DATABASE_META_BASE
};
pub const DCOUNTA_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.DCOUNTA",
    ..DATABASE_META_BASE
};
pub const DGET_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.DGET",
    ..DATABASE_META_BASE
};
pub const DMAX_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.DMAX",
    ..DATABASE_META_BASE
};
pub const DMIN_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.DMIN",
    ..DATABASE_META_BASE
};
pub const DPRODUCT_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.DPRODUCT",
    ..DATABASE_META_BASE
};
pub const DSTDEV_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.DSTDEV",
    ..DATABASE_META_BASE
};
pub const DSTDEVP_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.DSTDEVP",
    ..DATABASE_META_BASE
};
pub const DSUM_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.DSUM",
    ..DATABASE_META_BASE
};
pub const DVAR_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.DVAR",
    ..DATABASE_META_BASE
};
pub const DVARP_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.DVARP",
    ..DATABASE_META_BASE
};

#[derive(Debug, Clone, PartialEq)]
pub enum DatabaseEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
    Domain(WorksheetErrorCode),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CompareOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

#[derive(Debug, Clone, PartialEq)]
enum CriteriaOperand {
    Blank,
    Number(f64),
    Logical(bool),
    Text(String),
}

#[derive(Debug, Clone, PartialEq)]
struct CriteriaSpec {
    op: CompareOp,
    operand: CriteriaOperand,
    wildcard_text: bool,
    prefix_text: bool,
}

#[derive(Debug, Clone, PartialEq)]
struct DatabaseTable {
    headers: Vec<String>,
    rows: Vec<Vec<ArrayCellValue>>,
}

#[derive(Debug, Clone, PartialEq)]
struct CriteriaTable {
    headers: Vec<Option<String>>,
    rows: Vec<Vec<ArrayCellValue>>,
}

#[derive(Debug, Clone, PartialEq)]
struct CriteriaColumn {
    db_col: usize,
    spec: CriteriaSpec,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DatabaseAggregateKind {
    Average,
    Count,
    CountA,
    Get,
    Max,
    Min,
    Product,
    Stdev,
    StdevP,
    Sum,
    Var,
    VarP,
}

fn arity_error(meta: &FunctionMeta, actual: usize) -> DatabaseEvalError {
    DatabaseEvalError::ArityMismatch {
        expected_min: meta.arity.min,
        expected_max: meta.arity.max,
        actual,
    }
}

fn normalize_text(text: &ExcelText) -> String {
    text.to_string_lossy()
}

fn header_label(cell: &ArrayCellValue) -> Result<Option<String>, DatabaseEvalError> {
    match cell {
        ArrayCellValue::Text(text) => Ok(Some(normalize_text(text))),
        ArrayCellValue::Number(n) => Ok(Some(format!("{n}"))),
        ArrayCellValue::Logical(b) => Ok(Some(if *b { "TRUE" } else { "FALSE" }.to_string())),
        ArrayCellValue::EmptyCell => Ok(None),
        ArrayCellValue::Error(code) => Err(DatabaseEvalError::Domain(*code)),
    }
}

fn resolve_eval(
    arg: &CallArgValue,
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DatabaseEvalError> {
    match arg {
        CallArgValue::Reference(reference)
        | CallArgValue::Eval(EvalValue::Reference(reference)) => {
            let resolved = resolve_eval_value(resolver, reference)
                .map_err(CoercionError::RefResolution)
                .map_err(DatabaseEvalError::Coercion)?;
            resolve_eval(&CallArgValue::Eval(resolved), resolver)
        }
        CallArgValue::Eval(value) => Ok(value.clone()),
        CallArgValue::EmptyCell => Ok(EvalValue::Array(EvalArray::from_scalar(
            ArrayCellValue::EmptyCell,
        ))),
        CallArgValue::MissingArg => Err(DatabaseEvalError::Coercion(CoercionError::MissingArg)),
    }
}

fn eval_to_grid(value: EvalValue) -> Result<Vec<Vec<ArrayCellValue>>, DatabaseEvalError> {
    match value {
        EvalValue::Array(array) => {
            let shape = array.shape();
            let mut rows = Vec::with_capacity(shape.rows);
            for row_idx in 0..shape.rows {
                let mut row = Vec::with_capacity(shape.cols);
                for col_idx in 0..shape.cols {
                    row.push(
                        array
                            .get(row_idx, col_idx)
                            .cloned()
                            .ok_or(DatabaseEvalError::Domain(WorksheetErrorCode::Value))?,
                    );
                }
                rows.push(row);
            }
            Ok(rows)
        }
        _ => Err(DatabaseEvalError::Domain(WorksheetErrorCode::Value)),
    }
}

fn parse_database_table(
    arg: &CallArgValue,
    resolver: &impl ReferenceResolver,
) -> Result<DatabaseTable, DatabaseEvalError> {
    let rows = eval_to_grid(resolve_eval(arg, resolver)?)?;
    if rows.is_empty() || rows[0].is_empty() {
        return Err(DatabaseEvalError::Domain(WorksheetErrorCode::Value));
    }
    let headers = rows[0]
        .iter()
        .map(header_label)
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(|opt| opt.ok_or(DatabaseEvalError::Domain(WorksheetErrorCode::Value)))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(DatabaseTable {
        headers,
        rows: rows.into_iter().skip(1).collect(),
    })
}

fn parse_criteria_table(
    arg: &CallArgValue,
    resolver: &impl ReferenceResolver,
) -> Result<CriteriaTable, DatabaseEvalError> {
    let rows = eval_to_grid(resolve_eval(arg, resolver)?)?;
    if rows.is_empty() || rows[0].is_empty() {
        return Err(DatabaseEvalError::Domain(WorksheetErrorCode::Value));
    }
    let headers = rows[0]
        .iter()
        .map(header_label)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(CriteriaTable {
        headers,
        rows: rows.into_iter().skip(1).collect(),
    })
}

fn parse_excel_number(text: &str) -> Option<f64> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }
    let parsed = trimmed.parse::<f64>().ok()?;
    parsed.is_finite().then_some(parsed)
}

fn parse_logical_text(text: &str) -> Option<bool> {
    if text.eq_ignore_ascii_case("TRUE") {
        Some(true)
    } else if text.eq_ignore_ascii_case("FALSE") {
        Some(false)
    } else {
        None
    }
}

fn contains_unescaped_wildcard(text: &str) -> bool {
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '~' => {
                chars.next();
            }
            '*' | '?' => return true,
            _ => {}
        }
    }
    false
}

fn parse_operator_prefix(text: &str) -> (bool, CompareOp, &str) {
    if let Some(rest) = text.strip_prefix("<=") {
        (true, CompareOp::Le, rest)
    } else if let Some(rest) = text.strip_prefix(">=") {
        (true, CompareOp::Ge, rest)
    } else if let Some(rest) = text.strip_prefix("<>") {
        (true, CompareOp::Ne, rest)
    } else if let Some(rest) = text.strip_prefix('<') {
        (true, CompareOp::Lt, rest)
    } else if let Some(rest) = text.strip_prefix('>') {
        (true, CompareOp::Gt, rest)
    } else if let Some(rest) = text.strip_prefix('=') {
        (true, CompareOp::Eq, rest)
    } else {
        (false, CompareOp::Eq, text)
    }
}

fn criteria_from_cell(cell: &ArrayCellValue) -> Result<Option<CriteriaSpec>, DatabaseEvalError> {
    match cell {
        ArrayCellValue::EmptyCell => Ok(None),
        ArrayCellValue::Number(n) => Ok(Some(CriteriaSpec {
            op: CompareOp::Eq,
            operand: CriteriaOperand::Number(*n),
            wildcard_text: false,
            prefix_text: false,
        })),
        ArrayCellValue::Logical(b) => Ok(Some(CriteriaSpec {
            op: CompareOp::Eq,
            operand: CriteriaOperand::Logical(*b),
            wildcard_text: false,
            prefix_text: false,
        })),
        ArrayCellValue::Error(code) => Err(DatabaseEvalError::Domain(*code)),
        ArrayCellValue::Text(text) => {
            let raw = normalize_text(text);
            if raw.is_empty() {
                return Ok(None);
            }
            let (explicit, op, rhs) = parse_operator_prefix(&raw);
            let operand = if rhs.is_empty() {
                CriteriaOperand::Blank
            } else if let Some(number) = parse_excel_number(rhs) {
                CriteriaOperand::Number(number)
            } else if let Some(logical) = parse_logical_text(rhs) {
                CriteriaOperand::Logical(logical)
            } else {
                CriteriaOperand::Text(rhs.to_string())
            };
            Ok(Some(CriteriaSpec {
                op,
                wildcard_text: matches!(op, CompareOp::Eq | CompareOp::Ne)
                    && contains_unescaped_wildcard(rhs),
                prefix_text: !explicit && matches!(operand, CriteriaOperand::Text(_)),
                operand,
            }))
        }
    }
}

fn compare_numbers(op: CompareOp, lhs: f64, rhs: f64) -> bool {
    match op {
        CompareOp::Eq => lhs == rhs,
        CompareOp::Ne => lhs != rhs,
        CompareOp::Lt => lhs < rhs,
        CompareOp::Le => lhs <= rhs,
        CompareOp::Gt => lhs > rhs,
        CompareOp::Ge => lhs >= rhs,
    }
}

fn compare_bools(op: CompareOp, lhs: bool, rhs: bool) -> bool {
    match op {
        CompareOp::Eq => lhs == rhs,
        CompareOp::Ne => lhs != rhs,
        CompareOp::Lt => (lhs as u8) < (rhs as u8),
        CompareOp::Le => (lhs as u8) <= (rhs as u8),
        CompareOp::Gt => (lhs as u8) > (rhs as u8),
        CompareOp::Ge => (lhs as u8) >= (rhs as u8),
    }
}
fn compare_text(op: CompareOp, lhs: &str, rhs: &str, wildcard: bool, prefix: bool) -> bool {
    if wildcard {
        return match op {
            CompareOp::Eq => wildcard_match(rhs, lhs),
            CompareOp::Ne => !wildcard_match(rhs, lhs),
            _ => false,
        };
    }
    let left = lhs.to_lowercase();
    let right = rhs.to_lowercase();
    if prefix {
        return match op {
            CompareOp::Eq => left.starts_with(&right),
            CompareOp::Ne => !left.starts_with(&right),
            _ => false,
        };
    }
    match op {
        CompareOp::Eq => left == right,
        CompareOp::Ne => left != right,
        CompareOp::Lt => left < right,
        CompareOp::Le => left <= right,
        CompareOp::Gt => left > right,
        CompareOp::Ge => left >= right,
    }
}

fn cell_is_blank(cell: &ArrayCellValue) -> bool {
    match cell {
        ArrayCellValue::EmptyCell => true,
        ArrayCellValue::Text(text) => normalize_text(text).is_empty(),
        _ => false,
    }
}

fn criteria_matches_cell(criteria: &CriteriaSpec, cell: &ArrayCellValue) -> bool {
    match &criteria.operand {
        CriteriaOperand::Blank => match criteria.op {
            CompareOp::Eq => cell_is_blank(cell),
            CompareOp::Ne => !cell_is_blank(cell),
            _ => false,
        },
        CriteriaOperand::Number(target) => match cell {
            ArrayCellValue::Number(value) => compare_numbers(criteria.op, *value, *target),
            ArrayCellValue::Text(text) => parse_excel_number(&normalize_text(text))
                .is_some_and(|value| compare_numbers(criteria.op, value, *target)),
            _ => false,
        },
        CriteriaOperand::Logical(target) => match cell {
            ArrayCellValue::Logical(value) => compare_bools(criteria.op, *value, *target),
            ArrayCellValue::Text(text) => parse_logical_text(&normalize_text(text))
                .is_some_and(|value| compare_bools(criteria.op, value, *target)),
            _ => false,
        },
        CriteriaOperand::Text(target) => match cell {
            ArrayCellValue::Text(text) => compare_text(
                criteria.op,
                &normalize_text(text),
                target,
                criteria.wildcard_text,
                criteria.prefix_text,
            ),
            _ => false,
        },
    }
}

fn field_index_from_text(headers: &[String], label: &str) -> Option<usize> {
    headers
        .iter()
        .position(|header| header.eq_ignore_ascii_case(label))
}

fn resolve_field_index(
    database: &DatabaseTable,
    field_arg: &CallArgValue,
    resolver: &impl ReferenceResolver,
    allow_omitted: bool,
) -> Result<Option<usize>, DatabaseEvalError> {
    let prepared =
        prepare_arg_values_only(field_arg, resolver).map_err(DatabaseEvalError::Coercion)?;
    match prepared {
        PreparedArgValue::MissingArg | PreparedArgValue::EmptyCell if allow_omitted => Ok(None),
        PreparedArgValue::Eval(EvalValue::Text(text)) => {
            let label = normalize_text(&text);
            if allow_omitted && label.is_empty() {
                return Ok(None);
            }
            field_index_from_text(&database.headers, &label)
                .map(Some)
                .ok_or(DatabaseEvalError::Domain(WorksheetErrorCode::Value))
        }
        PreparedArgValue::Eval(EvalValue::Number(number)) => {
            if !number.is_finite() || number.fract() != 0.0 || number < 1.0 {
                return Err(DatabaseEvalError::Domain(WorksheetErrorCode::Value));
            }
            let idx = number as usize - 1;
            if idx < database.headers.len() {
                Ok(Some(idx))
            } else {
                Err(DatabaseEvalError::Domain(WorksheetErrorCode::Value))
            }
        }
        PreparedArgValue::Eval(EvalValue::Error(code)) => Err(DatabaseEvalError::Domain(code)),
        _ => Err(DatabaseEvalError::Domain(WorksheetErrorCode::Value)),
    }
}

fn compile_criteria_rows(
    database: &DatabaseTable,
    criteria: &CriteriaTable,
) -> Result<Vec<Vec<CriteriaColumn>>, DatabaseEvalError> {
    let mut compiled = Vec::with_capacity(criteria.rows.len());
    for row in &criteria.rows {
        let mut predicates = Vec::new();
        for (col_idx, cell) in row.iter().enumerate() {
            let Some(spec) = criteria_from_cell(cell)? else {
                continue;
            };
            let Some(header) = criteria.headers.get(col_idx).cloned().flatten() else {
                return Err(DatabaseEvalError::Domain(WorksheetErrorCode::Value));
            };
            let Some(db_col) = field_index_from_text(&database.headers, &header) else {
                return Err(DatabaseEvalError::Domain(WorksheetErrorCode::Value));
            };
            predicates.push(CriteriaColumn { db_col, spec });
        }
        compiled.push(predicates);
    }
    Ok(compiled)
}

fn matching_row_indices(
    database: &DatabaseTable,
    criteria: &CriteriaTable,
) -> Result<Vec<usize>, DatabaseEvalError> {
    if criteria.rows.is_empty() {
        return Ok((0..database.rows.len()).collect());
    }
    let compiled = compile_criteria_rows(database, criteria)?;
    let mut matched = Vec::new();
    'row_loop: for (row_idx, row) in database.rows.iter().enumerate() {
        for predicates in &compiled {
            if predicates
                .iter()
                .all(|predicate| criteria_matches_cell(&predicate.spec, &row[predicate.db_col]))
            {
                matched.push(row_idx);
                continue 'row_loop;
            }
        }
    }
    Ok(matched)
}

fn numeric_cell(cell: &ArrayCellValue) -> Result<Option<f64>, DatabaseEvalError> {
    match cell {
        ArrayCellValue::Number(n) => Ok(Some(*n)),
        ArrayCellValue::Error(code) => Err(DatabaseEvalError::Domain(*code)),
        ArrayCellValue::Text(_) | ArrayCellValue::Logical(_) | ArrayCellValue::EmptyCell => {
            Ok(None)
        }
    }
}

fn counta_included(cell: &ArrayCellValue) -> Result<bool, DatabaseEvalError> {
    match cell {
        ArrayCellValue::EmptyCell => Ok(false),
        ArrayCellValue::Error(code) => Err(DatabaseEvalError::Domain(*code)),
        ArrayCellValue::Number(_) | ArrayCellValue::Text(_) | ArrayCellValue::Logical(_) => {
            Ok(true)
        }
    }
}

fn scalar_from_cell(cell: &ArrayCellValue) -> EvalValue {
    match cell {
        ArrayCellValue::Number(n) => EvalValue::Number(*n),
        ArrayCellValue::Text(text) => EvalValue::Text(text.clone()),
        ArrayCellValue::Logical(b) => EvalValue::Logical(*b),
        ArrayCellValue::Error(code) => EvalValue::Error(*code),
        ArrayCellValue::EmptyCell => EvalValue::Number(0.0),
    }
}

fn selected_field_cells<'a>(
    database: &'a DatabaseTable,
    row_indices: &[usize],
    field_index: usize,
) -> Vec<&'a ArrayCellValue> {
    row_indices
        .iter()
        .map(|idx| &database.rows[*idx][field_index])
        .collect()
}

fn aggregate_numeric_values(cells: &[&ArrayCellValue]) -> Result<Vec<f64>, DatabaseEvalError> {
    let mut values = Vec::new();
    for cell in cells {
        if let Some(value) = numeric_cell(cell)? {
            values.push(value);
        }
    }
    Ok(values)
}

fn evaluate_database_aggregate(
    kind: DatabaseAggregateKind,
    database: &DatabaseTable,
    row_indices: &[usize],
    field_index: Option<usize>,
) -> Result<EvalValue, DatabaseEvalError> {
    match kind {
        DatabaseAggregateKind::Count if field_index.is_none() => {
            Ok(EvalValue::Number(row_indices.len() as f64))
        }
        DatabaseAggregateKind::CountA if field_index.is_none() => {
            Ok(EvalValue::Number(row_indices.len() as f64))
        }
        DatabaseAggregateKind::Get => {
            let field_index =
                field_index.ok_or(DatabaseEvalError::Domain(WorksheetErrorCode::Value))?;
            match row_indices.len() {
                0 => Ok(EvalValue::Error(WorksheetErrorCode::Value)),
                1 => Ok(scalar_from_cell(
                    &database.rows[row_indices[0]][field_index],
                )),
                _ => Ok(EvalValue::Error(WorksheetErrorCode::Num)),
            }
        }
        _ => {
            let field_index =
                field_index.ok_or(DatabaseEvalError::Domain(WorksheetErrorCode::Value))?;
            let cells = selected_field_cells(database, row_indices, field_index);
            match kind {
                DatabaseAggregateKind::Average => {
                    let values = aggregate_numeric_values(&cells)?;
                    if values.is_empty() {
                        Ok(EvalValue::Error(WorksheetErrorCode::Div0))
                    } else {
                        Ok(EvalValue::Number(
                            values.iter().sum::<f64>() / values.len() as f64,
                        ))
                    }
                }
                DatabaseAggregateKind::Count => Ok(EvalValue::Number(
                    aggregate_numeric_values(&cells)?.len() as f64,
                )),
                DatabaseAggregateKind::CountA => {
                    let mut count = 0.0;
                    for cell in cells {
                        if counta_included(cell)? {
                            count += 1.0;
                        }
                    }
                    Ok(EvalValue::Number(count))
                }
                DatabaseAggregateKind::Max => {
                    let values = aggregate_numeric_values(&cells)?;
                    Ok(EvalValue::Number(
                        values.into_iter().reduce(f64::max).unwrap_or(0.0),
                    ))
                }
                DatabaseAggregateKind::Min => {
                    let values = aggregate_numeric_values(&cells)?;
                    Ok(EvalValue::Number(
                        values.into_iter().reduce(f64::min).unwrap_or(0.0),
                    ))
                }
                DatabaseAggregateKind::Product => {
                    let values = aggregate_numeric_values(&cells)?;
                    if values.is_empty() {
                        Ok(EvalValue::Number(0.0))
                    } else {
                        Ok(EvalValue::Number(values.into_iter().product()))
                    }
                }
                DatabaseAggregateKind::Stdev => {
                    let values = aggregate_numeric_values(&cells)?;
                    Ok(match stdev_from_values(&values, VarianceDivisor::Sample) {
                        Ok(value) => EvalValue::Number(value),
                        Err(code) => EvalValue::Error(code),
                    })
                }
                DatabaseAggregateKind::StdevP => {
                    let values = aggregate_numeric_values(&cells)?;
                    Ok(
                        match stdev_from_values(&values, VarianceDivisor::Population) {
                            Ok(value) => EvalValue::Number(value),
                            Err(code) => EvalValue::Error(code),
                        },
                    )
                }
                DatabaseAggregateKind::Sum => Ok(EvalValue::Number(
                    aggregate_numeric_values(&cells)?.into_iter().sum(),
                )),
                DatabaseAggregateKind::Var => {
                    let values = aggregate_numeric_values(&cells)?;
                    Ok(
                        match variance_from_values(&values, VarianceDivisor::Sample) {
                            Ok(value) => EvalValue::Number(value),
                            Err(code) => EvalValue::Error(code),
                        },
                    )
                }
                DatabaseAggregateKind::VarP => {
                    let values = aggregate_numeric_values(&cells)?;
                    Ok(
                        match variance_from_values(&values, VarianceDivisor::Population) {
                            Ok(value) => EvalValue::Number(value),
                            Err(code) => EvalValue::Error(code),
                        },
                    )
                }
                DatabaseAggregateKind::Get => unreachable!(),
            }
        }
    }
}

fn eval_database_surface(
    meta: &FunctionMeta,
    kind: DatabaseAggregateKind,
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
    allow_omitted_field: bool,
) -> Result<EvalValue, DatabaseEvalError> {
    if !meta.arity.accepts(args.len()) {
        return Err(arity_error(meta, args.len()));
    }
    let database = parse_database_table(&args[0], resolver)?;
    let criteria = parse_criteria_table(&args[2], resolver)?;
    let field_index = resolve_field_index(&database, &args[1], resolver, allow_omitted_field)?;
    let matches = matching_row_indices(&database, &criteria)?;
    evaluate_database_aggregate(kind, &database, &matches, field_index)
}

pub fn eval_daverage_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DatabaseEvalError> {
    eval_database_surface(
        &DAVERAGE_META,
        DatabaseAggregateKind::Average,
        args,
        resolver,
        false,
    )
}
pub fn eval_dcount_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DatabaseEvalError> {
    eval_database_surface(
        &DCOUNT_META,
        DatabaseAggregateKind::Count,
        args,
        resolver,
        true,
    )
}
pub fn eval_dcounta_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DatabaseEvalError> {
    eval_database_surface(
        &DCOUNTA_META,
        DatabaseAggregateKind::CountA,
        args,
        resolver,
        true,
    )
}
pub fn eval_dget_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DatabaseEvalError> {
    eval_database_surface(
        &DGET_META,
        DatabaseAggregateKind::Get,
        args,
        resolver,
        false,
    )
}
pub fn eval_dmax_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DatabaseEvalError> {
    eval_database_surface(
        &DMAX_META,
        DatabaseAggregateKind::Max,
        args,
        resolver,
        false,
    )
}
pub fn eval_dmin_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DatabaseEvalError> {
    eval_database_surface(
        &DMIN_META,
        DatabaseAggregateKind::Min,
        args,
        resolver,
        false,
    )
}
pub fn eval_dproduct_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DatabaseEvalError> {
    eval_database_surface(
        &DPRODUCT_META,
        DatabaseAggregateKind::Product,
        args,
        resolver,
        false,
    )
}
pub fn eval_dstdev_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DatabaseEvalError> {
    eval_database_surface(
        &DSTDEV_META,
        DatabaseAggregateKind::Stdev,
        args,
        resolver,
        false,
    )
}
pub fn eval_dstdevp_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DatabaseEvalError> {
    eval_database_surface(
        &DSTDEVP_META,
        DatabaseAggregateKind::StdevP,
        args,
        resolver,
        false,
    )
}
pub fn eval_dsum_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DatabaseEvalError> {
    eval_database_surface(
        &DSUM_META,
        DatabaseAggregateKind::Sum,
        args,
        resolver,
        false,
    )
}
pub fn eval_dvar_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DatabaseEvalError> {
    eval_database_surface(
        &DVAR_META,
        DatabaseAggregateKind::Var,
        args,
        resolver,
        false,
    )
}
pub fn eval_dvarp_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DatabaseEvalError> {
    eval_database_surface(
        &DVARP_META,
        DatabaseAggregateKind::VarP,
        args,
        resolver,
        false,
    )
}

pub fn map_database_error_to_ws(error: &DatabaseEvalError) -> WorksheetErrorCode {
    match error {
        DatabaseEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        DatabaseEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        DatabaseEvalError::Coercion(_) => WorksheetErrorCode::Value,
        DatabaseEvalError::Domain(code) => *code,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::{RefResolutionError, ResolverCapabilities};
    use crate::value::{ReferenceKind, ReferenceLike};
    use std::collections::HashMap;

    struct MockResolver {
        cells: HashMap<String, EvalValue>,
    }

    impl ReferenceResolver for MockResolver {
        fn capabilities(&self) -> ResolverCapabilities {
            ResolverCapabilities::permissive_local()
        }
        fn resolve_reference(
            &self,
            reference: &ReferenceLike,
        ) -> Result<EvalValue, RefResolutionError> {
            self.cells.get(&reference.target).cloned().ok_or_else(|| {
                RefResolutionError::UnresolvedReference {
                    target: reference.target.clone(),
                }
            })
        }
    }

    fn text(value: &str) -> ArrayCellValue {
        ArrayCellValue::Text(ExcelText::from_utf16_code_units(
            value.encode_utf16().collect(),
        ))
    }

    fn ref_arg(target: &str) -> CallArgValue {
        CallArgValue::Reference(ReferenceLike {
            kind: ReferenceKind::Area,
            target: target.to_string(),
        })
    }

    fn field_text(name: &str) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
            name.encode_utf16().collect(),
        )))
    }

    fn array(rows: Vec<Vec<ArrayCellValue>>) -> EvalValue {
        EvalValue::Array(EvalArray::from_rows(rows).unwrap())
    }

    fn build_resolver() -> MockResolver {
        MockResolver {
            cells: HashMap::from([
                (
                    "DB".to_string(),
                    array(vec![
                        vec![
                            text("Type"),
                            text("Salesperson"),
                            text("Sales"),
                            text("Units"),
                        ],
                        vec![
                            text("Meat"),
                            text("Davolio"),
                            ArrayCellValue::Number(450.0),
                            ArrayCellValue::Number(8.0),
                        ],
                        vec![
                            text("Produce"),
                            text("Buchanan"),
                            ArrayCellValue::Number(6328.0),
                            ArrayCellValue::Number(10.0),
                        ],
                        vec![
                            text("Produce"),
                            text("Davolio"),
                            ArrayCellValue::Number(6544.0),
                            ArrayCellValue::EmptyCell,
                        ],
                        vec![
                            text("Dairy"),
                            text("David"),
                            ArrayCellValue::Number(834.0),
                            ArrayCellValue::Number(7.0),
                        ],
                        vec![
                            text("Produce"),
                            text("Davis"),
                            ArrayCellValue::Number(3000.0),
                            ArrayCellValue::Number(5.0),
                        ],
                    ]),
                ),
                (
                    "CRIT_DAV".to_string(),
                    array(vec![vec![text("Salesperson")], vec![text("Dav")]]),
                ),
                (
                    "CRIT_PRODUCE".to_string(),
                    array(vec![vec![text("Type")], vec![text("=Produce")]]),
                ),
                (
                    "CRIT_UNIQUE".to_string(),
                    array(vec![vec![text("Salesperson")], vec![text("=Buchanan")]]),
                ),
                (
                    "CRIT_NONE".to_string(),
                    array(vec![vec![text("Salesperson")], vec![text("=Nobody")]]),
                ),
                (
                    "CRIT_RANGE_OR".to_string(),
                    array(vec![
                        vec![text("Sales"), text("Sales")],
                        vec![text(">6000"), text("<6500")],
                        vec![text("<500"), ArrayCellValue::EmptyCell],
                    ]),
                ),
            ]),
        }
    }

    #[test]
    fn database_meta_shapes_match_family() {
        assert_eq!(DAVERAGE_META.arity, Arity::exact(3));
        assert_eq!(
            DVARP_META.arg_preparation_profile,
            ArgPreparationProfile::RefsVisibleInAdapter
        );
        assert_eq!(DGET_META.function_id, "FUNC.DGET");
    }

    #[test]
    fn dsum_daverage_dcount_and_dcounta_follow_bounded_database_slice() {
        let resolver = build_resolver();
        assert_eq!(
            eval_dsum_surface(
                &[ref_arg("DB"), field_text("Sales"), ref_arg("CRIT_DAV")],
                &resolver
            ),
            Ok(EvalValue::Number(10828.0))
        );
        assert_eq!(
            eval_daverage_surface(
                &[ref_arg("DB"), field_text("Sales"), ref_arg("CRIT_DAV")],
                &resolver
            ),
            Ok(EvalValue::Number(2707.0))
        );
        assert_eq!(
            eval_dcount_surface(
                &[ref_arg("DB"), CallArgValue::MissingArg, ref_arg("CRIT_DAV")],
                &resolver
            ),
            Ok(EvalValue::Number(4.0))
        );
        assert_eq!(
            eval_dcount_surface(
                &[
                    ref_arg("DB"),
                    CallArgValue::Eval(EvalValue::Number(3.0)),
                    ref_arg("CRIT_DAV")
                ],
                &resolver
            ),
            Ok(EvalValue::Number(4.0))
        );
        assert_eq!(
            eval_dcounta_surface(
                &[ref_arg("DB"), field_text("Units"), ref_arg("CRIT_PRODUCE")],
                &resolver
            ),
            Ok(EvalValue::Number(2.0))
        );
    }

    #[test]
    fn dmax_dmin_dproduct_and_variance_family_match_seeded_rows() {
        let resolver = build_resolver();
        assert_eq!(
            eval_dmax_surface(
                &[ref_arg("DB"), field_text("Sales"), ref_arg("CRIT_PRODUCE")],
                &resolver
            ),
            Ok(EvalValue::Number(6544.0))
        );
        assert_eq!(
            eval_dmin_surface(
                &[ref_arg("DB"), field_text("Sales"), ref_arg("CRIT_PRODUCE")],
                &resolver
            ),
            Ok(EvalValue::Number(3000.0))
        );
        assert_eq!(
            eval_dproduct_surface(
                &[ref_arg("DB"), field_text("Units"), ref_arg("CRIT_PRODUCE")],
                &resolver
            ),
            Ok(EvalValue::Number(50.0))
        );
        let dstdev = eval_dstdev_surface(
            &[ref_arg("DB"), field_text("Sales"), ref_arg("CRIT_PRODUCE")],
            &resolver,
        )
        .unwrap();
        let dstdevp = eval_dstdevp_surface(
            &[ref_arg("DB"), field_text("Sales"), ref_arg("CRIT_PRODUCE")],
            &resolver,
        )
        .unwrap();
        let dvar = eval_dvar_surface(
            &[ref_arg("DB"), field_text("Sales"), ref_arg("CRIT_PRODUCE")],
            &resolver,
        )
        .unwrap();
        let dvarp = eval_dvarp_surface(
            &[ref_arg("DB"), field_text("Sales"), ref_arg("CRIT_PRODUCE")],
            &resolver,
        )
        .unwrap();
        match dstdev {
            EvalValue::Number(n) => assert!((n - 1986.7131985602082).abs() < 1e-9),
            other => panic!("unexpected {other:?}"),
        }
        match dstdevp {
            EvalValue::Number(n) => assert!((n - 1622.1445339083964).abs() < 1e-9),
            other => panic!("unexpected {other:?}"),
        }
        match dvar {
            EvalValue::Number(n) => assert!((n - 3947029.333333333).abs() < 1e-6),
            other => panic!("unexpected {other:?}"),
        }
        match dvarp {
            EvalValue::Number(n) => assert!((n - 2631352.8888888885).abs() < 1e-6),
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn dget_and_or_criteria_lanes_are_pinned() {
        let resolver = build_resolver();
        assert_eq!(
            eval_dget_surface(
                &[ref_arg("DB"), field_text("Sales"), ref_arg("CRIT_UNIQUE")],
                &resolver
            ),
            Ok(EvalValue::Number(6328.0))
        );
        assert_eq!(
            eval_dget_surface(
                &[ref_arg("DB"), field_text("Sales"), ref_arg("CRIT_DAV")],
                &resolver
            ),
            Ok(EvalValue::Error(WorksheetErrorCode::Num))
        );
        assert_eq!(
            eval_dget_surface(
                &[ref_arg("DB"), field_text("Sales"), ref_arg("CRIT_NONE")],
                &resolver
            ),
            Ok(EvalValue::Error(WorksheetErrorCode::Value))
        );
        assert_eq!(
            eval_dcount_surface(
                &[
                    ref_arg("DB"),
                    CallArgValue::MissingArg,
                    ref_arg("CRIT_RANGE_OR")
                ],
                &resolver
            ),
            Ok(EvalValue::Number(2.0))
        );
    }

    #[test]
    fn domain_and_mapping_lanes_are_scoped() {
        let resolver = build_resolver();
        assert_eq!(
            eval_dsum_surface(
                &[ref_arg("DB"), field_text("Missing"), ref_arg("CRIT_DAV")],
                &resolver
            ),
            Err(DatabaseEvalError::Domain(WorksheetErrorCode::Value))
        );
        assert_eq!(
            eval_dsum_surface(&[ref_arg("DB"), field_text("Sales")], &resolver),
            Err(DatabaseEvalError::ArityMismatch {
                expected_min: 3,
                expected_max: 3,
                actual: 2
            })
        );
        assert_eq!(
            map_database_error_to_ws(&DatabaseEvalError::Domain(WorksheetErrorCode::Num)),
            WorksheetErrorCode::Num
        );
    }
}

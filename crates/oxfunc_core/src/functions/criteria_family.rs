use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::a1_refs::{
    A1Reference, A1ReferenceNotation, format_relative_target, parse_a1_reference,
};
use crate::functions::adapters::{PreparedArgValue, prepare_arg_values_only};
use crate::functions::excel_numeric_compare::compare_excel_numbers;
use crate::functions::xmatch::wildcard_match;
use crate::resolver::{ReferenceResolver, resolve_eval_value};
use crate::value::{
    ArrayCellValue, CallArgValue, EvalArray, EvalValue, ExcelText, ReferenceKind, ReferenceLike,
    WorksheetErrorCode,
};

const CRITERIA_BASE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.CRITERIA_BASE",
    arity: Arity { min: 2, max: 255 },
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

pub const COUNTIF_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.COUNTIF",
    arity: Arity::exact(2),
    ..CRITERIA_BASE_META
};
pub const COUNTIFS_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.COUNTIFS",
    arity: Arity { min: 2, max: 254 },
    ..CRITERIA_BASE_META
};
pub const SUMIF_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.SUMIF",
    arity: Arity { min: 2, max: 3 },
    ..CRITERIA_BASE_META
};
pub const SUMIFS_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.SUMIFS",
    arity: Arity { min: 3, max: 255 },
    ..CRITERIA_BASE_META
};
pub const AVERAGEIF_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.AVERAGEIF",
    arity: Arity { min: 2, max: 3 },
    ..CRITERIA_BASE_META
};
pub const AVERAGEIFS_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.AVERAGEIFS",
    arity: Arity { min: 3, max: 255 },
    ..CRITERIA_BASE_META
};
pub const MAXIFS_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.MAXIFS",
    arity: Arity { min: 3, max: 255 },
    ..CRITERIA_BASE_META
};
pub const MINIFS_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.MINIFS",
    arity: Arity { min: 3, max: 255 },
    ..CRITERIA_BASE_META
};

#[derive(Debug, Clone, PartialEq)]
pub enum CriteriaEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    PairStructureMismatch {
        actual: usize,
    },
    Coercion(CoercionError),
    Domain(WorksheetErrorCode),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FlatShape {
    rows: usize,
    cols: usize,
}
impl FlatShape {
    fn len(self) -> usize {
        self.rows * self.cols
    }
}

#[derive(Debug, Clone, PartialEq)]
struct FlatCells {
    shape: FlatShape,
    cells: Vec<ArrayCellValue>,
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
    Text(String),
    Logical(bool),
}

#[derive(Debug, Clone, PartialEq)]
struct CriteriaSpec {
    op: CompareOp,
    operand: CriteriaOperand,
    wildcard_text: bool,
}

fn parse_excel_number(text: &str) -> Option<f64> {
    let t = text.trim();
    if t.is_empty() {
        return None;
    }
    let n = t.parse::<f64>().ok()?;
    n.is_finite().then_some(n)
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

fn normalize_text(text: &ExcelText) -> String {
    text.to_string_lossy()
}

fn scalar_to_cell(value: EvalValue) -> Result<ArrayCellValue, CriteriaEvalError> {
    match value {
        EvalValue::Number(n) => Ok(ArrayCellValue::Number(n)),
        EvalValue::Text(t) => Ok(ArrayCellValue::Text(t)),
        EvalValue::Logical(b) => Ok(ArrayCellValue::Logical(b)),
        EvalValue::Error(code) => Ok(ArrayCellValue::Error(code)),
        EvalValue::Array(_) | EvalValue::Reference(_) | EvalValue::Lambda(_) => {
            Err(CriteriaEvalError::Domain(WorksheetErrorCode::Value))
        }
    }
}

fn resolve_arg_eval(
    arg: &CallArgValue,
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, CriteriaEvalError> {
    match arg {
        CallArgValue::Reference(reference)
        | CallArgValue::Eval(EvalValue::Reference(reference)) => {
            let resolved = resolve_eval_value(resolver, reference)
                .map_err(CoercionError::RefResolution)
                .map_err(CriteriaEvalError::Coercion)?;
            resolve_arg_eval(&CallArgValue::Eval(resolved), resolver)
        }
        CallArgValue::Eval(value) => Ok(value.clone()),
        CallArgValue::EmptyCell => Ok(EvalValue::Array(crate::value::EvalArray::from_scalar(
            ArrayCellValue::EmptyCell,
        ))),
        CallArgValue::MissingArg => Err(CriteriaEvalError::Coercion(CoercionError::MissingArg)),
    }
}

fn flatten_arg(
    arg: &CallArgValue,
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<FlatCells, CriteriaEvalError> {
    match resolve_arg_eval(arg, resolver)? {
        EvalValue::Array(array) => {
            let shape = array.shape();
            Ok(FlatCells {
                shape: FlatShape {
                    rows: shape.rows,
                    cols: shape.cols,
                },
                cells: array.iter_row_major().cloned().collect(),
            })
        }
        scalar => Ok(FlatCells {
            shape: FlatShape { rows: 1, cols: 1 },
            cells: vec![scalar_to_cell(scalar)?],
        }),
    }
}

fn shaped_value_error_from_direct_array(arg: &CallArgValue) -> Option<EvalValue> {
    let CallArgValue::Eval(EvalValue::Array(array)) = arg else {
        return None;
    };
    Some(EvalValue::Array(
        EvalArray::new(
            array.shape(),
            vec![ArrayCellValue::Error(WorksheetErrorCode::Value); array.shape().cell_count()],
        )
        .expect("shape preserved"),
    ))
}

fn extrema_ifs_direct_array_value_error(args: &[CallArgValue]) -> Option<EvalValue> {
    for (index, arg) in args.iter().enumerate() {
        let is_range_arg = index == 0 || index % 2 == 1;
        if is_range_arg {
            if let Some(err) = shaped_value_error_from_direct_array(arg) {
                return Some(err);
            }
        }
    }
    None
}

fn reference_like_from_arg(arg: &CallArgValue) -> Option<&ReferenceLike> {
    match arg {
        CallArgValue::Reference(reference)
        | CallArgValue::Eval(EvalValue::Reference(reference)) => Some(reference),
        _ => None,
    }
}

fn try_anchor_reference_to_shape(
    arg: &CallArgValue,
    desired_shape: FlatShape,
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<Option<FlatCells>, CriteriaEvalError> {
    let Some(reference) = reference_like_from_arg(arg) else {
        return Ok(None);
    };
    let Some(parsed) = parse_a1_reference(&reference.target) else {
        return Ok(None);
    };

    let end_row = parsed
        .start_row
        .checked_add(
            desired_shape
                .rows
                .checked_sub(1)
                .ok_or(CriteriaEvalError::Domain(WorksheetErrorCode::Value))?,
        )
        .ok_or(CriteriaEvalError::Domain(WorksheetErrorCode::Ref))?;
    let end_col = parsed
        .start_col
        .checked_add(
            desired_shape
                .cols
                .checked_sub(1)
                .ok_or(CriteriaEvalError::Domain(WorksheetErrorCode::Value))?,
        )
        .ok_or(CriteriaEvalError::Domain(WorksheetErrorCode::Ref))?;

    let anchored = A1Reference {
        prefix: parsed.prefix.clone(),
        start_row: parsed.start_row,
        start_col: parsed.start_col,
        end_row,
        end_col,
        notation: A1ReferenceNotation::Rect,
    };
    let target = format_relative_target(&anchored)
        .ok_or(CriteriaEvalError::Domain(WorksheetErrorCode::Ref))?;
    flatten_arg(
        &CallArgValue::Reference(ReferenceLike {
            kind: if desired_shape.rows == 1 && desired_shape.cols == 1 {
                ReferenceKind::A1
            } else {
                ReferenceKind::Area
            },
            target,
        }),
        resolver,
    )
    .map(Some)
}

fn parse_operator_prefix(text: &str) -> (CompareOp, &str) {
    if let Some(rest) = text.strip_prefix("<=") {
        (CompareOp::Le, rest)
    } else if let Some(rest) = text.strip_prefix(">=") {
        (CompareOp::Ge, rest)
    } else if let Some(rest) = text.strip_prefix("<>") {
        (CompareOp::Ne, rest)
    } else if let Some(rest) = text.strip_prefix('<') {
        (CompareOp::Lt, rest)
    } else if let Some(rest) = text.strip_prefix('>') {
        (CompareOp::Gt, rest)
    } else if let Some(rest) = text.strip_prefix('=') {
        (CompareOp::Eq, rest)
    } else {
        (CompareOp::Eq, text)
    }
}

fn criteria_from_prepared(prepared: &PreparedArgValue) -> Result<CriteriaSpec, CriteriaEvalError> {
    match prepared {
        PreparedArgValue::Eval(EvalValue::Number(n)) => Ok(CriteriaSpec {
            op: CompareOp::Eq,
            operand: CriteriaOperand::Number(*n),
            wildcard_text: false,
        }),
        PreparedArgValue::Eval(EvalValue::Logical(b)) => Ok(CriteriaSpec {
            op: CompareOp::Eq,
            operand: CriteriaOperand::Logical(*b),
            wildcard_text: false,
        }),
        PreparedArgValue::Eval(EvalValue::Text(text)) => {
            let raw = normalize_text(text);
            let (op, rhs) = parse_operator_prefix(&raw);
            let operand = if rhs.is_empty() {
                CriteriaOperand::Blank
            } else if let Some(n) = parse_excel_number(rhs) {
                CriteriaOperand::Number(n)
            } else if let Some(b) = parse_logical_text(rhs) {
                CriteriaOperand::Logical(b)
            } else {
                CriteriaOperand::Text(rhs.to_string())
            };
            Ok(CriteriaSpec {
                op,
                wildcard_text: matches!(op, CompareOp::Eq | CompareOp::Ne)
                    && contains_unescaped_wildcard(rhs),
                operand,
            })
        }
        PreparedArgValue::Eval(EvalValue::Error(code)) => Err(CriteriaEvalError::Domain(*code)),
        PreparedArgValue::Eval(EvalValue::Array(_))
        | PreparedArgValue::Eval(EvalValue::Reference(_))
        | PreparedArgValue::Eval(EvalValue::Lambda(_)) => {
            Err(CriteriaEvalError::Domain(WorksheetErrorCode::Value))
        }
        PreparedArgValue::MissingArg => Err(CriteriaEvalError::Coercion(CoercionError::MissingArg)),
        PreparedArgValue::EmptyCell => Ok(CriteriaSpec {
            op: CompareOp::Eq,
            operand: CriteriaOperand::Blank,
            wildcard_text: false,
        }),
    }
}

fn compare_numbers(op: CompareOp, lhs: f64, rhs: f64) -> bool {
    let ord = compare_excel_numbers(lhs, rhs);
    match op {
        CompareOp::Eq => ord == std::cmp::Ordering::Equal,
        CompareOp::Ne => ord != std::cmp::Ordering::Equal,
        CompareOp::Lt => ord == std::cmp::Ordering::Less,
        CompareOp::Le => ord != std::cmp::Ordering::Greater,
        CompareOp::Gt => ord == std::cmp::Ordering::Greater,
        CompareOp::Ge => ord != std::cmp::Ordering::Less,
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
fn compare_texts(op: CompareOp, lhs: &str, rhs: &str, wildcard: bool) -> bool {
    if wildcard {
        match op {
            CompareOp::Eq => wildcard_match(rhs, lhs),
            CompareOp::Ne => !wildcard_match(rhs, lhs),
            _ => false,
        }
    } else {
        let l = lhs.to_lowercase();
        let r = rhs.to_lowercase();
        match op {
            CompareOp::Eq => l == r,
            CompareOp::Ne => l != r,
            CompareOp::Lt => l < r,
            CompareOp::Le => l <= r,
            CompareOp::Gt => l > r,
            CompareOp::Ge => l >= r,
        }
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
                .is_some_and(|v| compare_numbers(criteria.op, v, *target)),
            _ => false,
        },
        CriteriaOperand::Logical(target) => match cell {
            ArrayCellValue::Logical(value) => compare_bools(criteria.op, *value, *target),
            ArrayCellValue::Text(text) => parse_logical_text(&normalize_text(text))
                .is_some_and(|v| compare_bools(criteria.op, v, *target)),
            _ => false,
        },
        CriteriaOperand::Text(target) => match cell {
            ArrayCellValue::Text(text) => compare_texts(
                criteria.op,
                &normalize_text(text),
                target,
                criteria.wildcard_text,
            ),
            _ => false,
        },
    }
}

fn ensure_same_shape(ranges: &[&FlatCells]) -> Result<(), CriteriaEvalError> {
    let Some(first) = ranges.first() else {
        return Ok(());
    };
    if ranges.iter().all(|range| range.shape == first.shape) {
        Ok(())
    } else {
        Err(CriteriaEvalError::Domain(WorksheetErrorCode::Value))
    }
}

fn parse_criteria_pairs(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<Vec<(FlatCells, CriteriaSpec)>, CriteriaEvalError> {
    if args.len() % 2 != 0 {
        return Err(CriteriaEvalError::PairStructureMismatch { actual: args.len() });
    }
    let mut pairs = Vec::with_capacity(args.len() / 2);
    for pair in args.chunks_exact(2) {
        let range = flatten_arg(&pair[0], resolver)?;
        let prepared =
            prepare_arg_values_only(&pair[1], resolver).map_err(CriteriaEvalError::Coercion)?;
        let criteria = criteria_from_prepared(&prepared)?;
        pairs.push((range, criteria));
    }
    Ok(pairs)
}

fn matching_mask(pairs: &[(FlatCells, CriteriaSpec)]) -> Result<Vec<bool>, CriteriaEvalError> {
    let ranges = pairs.iter().map(|(range, _)| range).collect::<Vec<_>>();
    ensure_same_shape(&ranges)?;
    let len = ranges.first().map_or(0, |range| range.shape.len());
    let mut mask = vec![true; len];
    for idx in 0..len {
        for (range, criteria) in pairs {
            if !criteria_matches_cell(criteria, &range.cells[idx]) {
                mask[idx] = false;
                break;
            }
        }
    }
    Ok(mask)
}

fn numeric_cell_for_aggregate(cell: &ArrayCellValue) -> Result<Option<f64>, CriteriaEvalError> {
    match cell {
        ArrayCellValue::Number(n) => Ok(Some(*n)),
        ArrayCellValue::Error(code) => Err(CriteriaEvalError::Domain(*code)),
        ArrayCellValue::Text(_) | ArrayCellValue::Logical(_) | ArrayCellValue::EmptyCell => {
            Ok(None)
        }
    }
}

fn count_matches(mask: &[bool]) -> f64 {
    mask.iter().filter(|matched| **matched).count() as f64
}

fn eval_sum_filtered(target: &FlatCells, mask: &[bool]) -> Result<EvalValue, CriteriaEvalError> {
    let mut sum = 0.0;
    for (idx, matched) in mask.iter().enumerate() {
        if *matched {
            if let Some(value) = numeric_cell_for_aggregate(&target.cells[idx])? {
                sum += value;
            }
        }
    }
    Ok(EvalValue::Number(sum))
}

fn eval_average_filtered(
    target: &FlatCells,
    mask: &[bool],
) -> Result<EvalValue, CriteriaEvalError> {
    let mut sum = 0.0;
    let mut count = 0usize;
    for (idx, matched) in mask.iter().enumerate() {
        if *matched {
            if let Some(value) = numeric_cell_for_aggregate(&target.cells[idx])? {
                sum += value;
                count += 1;
            }
        }
    }
    if count == 0 {
        Ok(EvalValue::Error(WorksheetErrorCode::Div0))
    } else {
        Ok(EvalValue::Number(sum / count as f64))
    }
}

fn eval_extreme_filtered(
    target: &FlatCells,
    mask: &[bool],
    pick_max: bool,
) -> Result<EvalValue, CriteriaEvalError> {
    let mut best: Option<f64> = None;
    for (idx, matched) in mask.iter().enumerate() {
        if *matched {
            if let Some(value) = numeric_cell_for_aggregate(&target.cells[idx])? {
                best = Some(match best {
                    Some(current) if pick_max => current.max(value),
                    Some(current) => current.min(value),
                    None => value,
                });
            }
        }
    }
    Ok(EvalValue::Number(best.unwrap_or(0.0)))
}

pub fn eval_countif_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, CriteriaEvalError> {
    if !COUNTIF_META.arity.accepts(args.len()) {
        return Err(CriteriaEvalError::ArityMismatch {
            expected_min: COUNTIF_META.arity.min,
            expected_max: COUNTIF_META.arity.max,
            actual: args.len(),
        });
    }
    let range = flatten_arg(&args[0], resolver)?;
    let prepared =
        prepare_arg_values_only(&args[1], resolver).map_err(CriteriaEvalError::Coercion)?;
    let criteria = criteria_from_prepared(&prepared)?;
    Ok(EvalValue::Number(
        range
            .cells
            .iter()
            .filter(|cell| criteria_matches_cell(&criteria, cell))
            .count() as f64,
    ))
}

pub fn eval_countifs_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, CriteriaEvalError> {
    if !COUNTIFS_META.arity.accepts(args.len()) {
        return Err(CriteriaEvalError::ArityMismatch {
            expected_min: COUNTIFS_META.arity.min,
            expected_max: COUNTIFS_META.arity.max,
            actual: args.len(),
        });
    }
    let pairs = parse_criteria_pairs(args, resolver)?;
    Ok(EvalValue::Number(count_matches(&matching_mask(&pairs)?)))
}

pub fn eval_sumifs_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, CriteriaEvalError> {
    if !SUMIFS_META.arity.accepts(args.len()) {
        return Err(CriteriaEvalError::ArityMismatch {
            expected_min: SUMIFS_META.arity.min,
            expected_max: SUMIFS_META.arity.max,
            actual: args.len(),
        });
    }
    if (args.len() - 1) % 2 != 0 {
        return Err(CriteriaEvalError::PairStructureMismatch { actual: args.len() });
    }
    let sum_range = flatten_arg(&args[0], resolver)?;
    let pairs = parse_criteria_pairs(&args[1..], resolver)?;
    let ranges = std::iter::once(&sum_range)
        .chain(pairs.iter().map(|(range, _)| range))
        .collect::<Vec<_>>();
    ensure_same_shape(&ranges)?;
    eval_sum_filtered(&sum_range, &matching_mask(&pairs)?)
}

pub fn eval_sumif_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, CriteriaEvalError> {
    if !SUMIF_META.arity.accepts(args.len()) {
        return Err(CriteriaEvalError::ArityMismatch {
            expected_min: SUMIF_META.arity.min,
            expected_max: SUMIF_META.arity.max,
            actual: args.len(),
        });
    }
    let criteria_range = flatten_arg(&args[0], resolver)?;
    let prepared =
        prepare_arg_values_only(&args[1], resolver).map_err(CriteriaEvalError::Coercion)?;
    let criteria = criteria_from_prepared(&prepared)?;
    let sum_range = if let Some(range) = args.get(2) {
        let direct = flatten_arg(range, resolver)?;
        if direct.shape == criteria_range.shape {
            direct
        } else if let Some(anchored) =
            try_anchor_reference_to_shape(range, criteria_range.shape, resolver)?
        {
            anchored
        } else {
            direct
        }
    } else {
        criteria_range.clone()
    };
    ensure_same_shape(&[&criteria_range, &sum_range])?;
    let mask = criteria_range
        .cells
        .iter()
        .map(|cell| criteria_matches_cell(&criteria, cell))
        .collect::<Vec<_>>();
    eval_sum_filtered(&sum_range, &mask)
}

pub fn eval_averageif_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, CriteriaEvalError> {
    if !AVERAGEIF_META.arity.accepts(args.len()) {
        return Err(CriteriaEvalError::ArityMismatch {
            expected_min: AVERAGEIF_META.arity.min,
            expected_max: AVERAGEIF_META.arity.max,
            actual: args.len(),
        });
    }
    let criteria_range = flatten_arg(&args[0], resolver)?;
    let prepared =
        prepare_arg_values_only(&args[1], resolver).map_err(CriteriaEvalError::Coercion)?;
    let criteria = criteria_from_prepared(&prepared)?;
    let average_range = if let Some(range) = args.get(2) {
        let direct = flatten_arg(range, resolver)?;
        if direct.shape == criteria_range.shape {
            direct
        } else if let Some(anchored) =
            try_anchor_reference_to_shape(range, criteria_range.shape, resolver)?
        {
            anchored
        } else {
            direct
        }
    } else {
        criteria_range.clone()
    };
    ensure_same_shape(&[&criteria_range, &average_range])?;
    let mask = criteria_range
        .cells
        .iter()
        .map(|cell| criteria_matches_cell(&criteria, cell))
        .collect::<Vec<_>>();
    eval_average_filtered(&average_range, &mask)
}

pub fn eval_averageifs_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, CriteriaEvalError> {
    if !AVERAGEIFS_META.arity.accepts(args.len()) {
        return Err(CriteriaEvalError::ArityMismatch {
            expected_min: AVERAGEIFS_META.arity.min,
            expected_max: AVERAGEIFS_META.arity.max,
            actual: args.len(),
        });
    }
    if (args.len() - 1) % 2 != 0 {
        return Err(CriteriaEvalError::PairStructureMismatch { actual: args.len() });
    }
    let average_range = flatten_arg(&args[0], resolver)?;
    let pairs = parse_criteria_pairs(&args[1..], resolver)?;
    let ranges = std::iter::once(&average_range)
        .chain(pairs.iter().map(|(range, _)| range))
        .collect::<Vec<_>>();
    ensure_same_shape(&ranges)?;
    eval_average_filtered(&average_range, &matching_mask(&pairs)?)
}

pub fn eval_maxifs_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, CriteriaEvalError> {
    if !MAXIFS_META.arity.accepts(args.len()) {
        return Err(CriteriaEvalError::ArityMismatch {
            expected_min: MAXIFS_META.arity.min,
            expected_max: MAXIFS_META.arity.max,
            actual: args.len(),
        });
    }
    if (args.len() - 1) % 2 != 0 {
        return Err(CriteriaEvalError::PairStructureMismatch { actual: args.len() });
    }
    if let Some(err) = extrema_ifs_direct_array_value_error(args) {
        return Ok(err);
    }
    let max_range = flatten_arg(&args[0], resolver)?;
    let pairs = parse_criteria_pairs(&args[1..], resolver)?;
    let ranges = std::iter::once(&max_range)
        .chain(pairs.iter().map(|(range, _)| range))
        .collect::<Vec<_>>();
    ensure_same_shape(&ranges)?;
    eval_extreme_filtered(&max_range, &matching_mask(&pairs)?, true)
}

pub fn eval_minifs_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, CriteriaEvalError> {
    if !MINIFS_META.arity.accepts(args.len()) {
        return Err(CriteriaEvalError::ArityMismatch {
            expected_min: MINIFS_META.arity.min,
            expected_max: MINIFS_META.arity.max,
            actual: args.len(),
        });
    }
    if (args.len() - 1) % 2 != 0 {
        return Err(CriteriaEvalError::PairStructureMismatch { actual: args.len() });
    }
    if let Some(err) = extrema_ifs_direct_array_value_error(args) {
        return Ok(err);
    }
    let min_range = flatten_arg(&args[0], resolver)?;
    let pairs = parse_criteria_pairs(&args[1..], resolver)?;
    let ranges = std::iter::once(&min_range)
        .chain(pairs.iter().map(|(range, _)| range))
        .collect::<Vec<_>>();
    ensure_same_shape(&ranges)?;
    eval_extreme_filtered(&min_range, &matching_mask(&pairs)?, false)
}

pub fn map_criteria_error_to_ws(error: &CriteriaEvalError) -> WorksheetErrorCode {
    match error {
        CriteriaEvalError::ArityMismatch { .. }
        | CriteriaEvalError::PairStructureMismatch { .. } => WorksheetErrorCode::Value,
        CriteriaEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        CriteriaEvalError::Coercion(_) => WorksheetErrorCode::Value,
        CriteriaEvalError::Domain(code) => *code,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::{RefResolutionError, ResolverCapabilities};
    use crate::value::{EvalArray, ReferenceLike};
    use std::collections::HashMap;

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

    struct MapResolver {
        refs: HashMap<String, EvalValue>,
    }

    impl ReferenceResolver for MapResolver {
        fn capabilities(&self) -> ResolverCapabilities {
            ResolverCapabilities::permissive_local()
        }

        fn resolve_reference(
            &self,
            reference: &ReferenceLike,
        ) -> Result<EvalValue, RefResolutionError> {
            self.refs.get(&reference.target).cloned().ok_or_else(|| {
                RefResolutionError::UnresolvedReference {
                    target: reference.target.clone(),
                }
            })
        }
    }

    fn text(s: &str) -> ArrayCellValue {
        ArrayCellValue::Text(ExcelText::from_interop_assignment(s))
    }
    fn array(rows: Vec<Vec<ArrayCellValue>>) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Array(EvalArray::from_rows(rows).unwrap()))
    }
    fn scalar_text(s: &str) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Text(ExcelText::from_interop_assignment(s)))
    }
    fn area_ref(target: &str) -> CallArgValue {
        CallArgValue::Reference(ReferenceLike {
            kind: ReferenceKind::Area,
            target: target.to_string(),
        })
    }

    #[test]
    fn countif_matches_numeric_and_text_comparisons() {
        let got = eval_countif_surface(
            &[
                array(vec![vec![
                    ArrayCellValue::Number(1.0),
                    ArrayCellValue::Number(2.0),
                    ArrayCellValue::Number(3.0),
                    text("3"),
                ]]),
                scalar_text(">=2"),
            ],
            &NoResolver,
        );
        assert_eq!(got, Ok(EvalValue::Number(3.0)));

        let eq_text = eval_countif_surface(
            &[
                array(vec![vec![text("North"), text("north"), text("South")]]),
                scalar_text("north"),
            ],
            &NoResolver,
        );
        assert_eq!(eq_text, Ok(EvalValue::Number(2.0)));
    }

    #[test]
    fn countif_uses_excel_near_equal_numeric_comparisons() {
        let range = array(vec![vec![ArrayCellValue::Number(0.3)]]);
        assert_eq!(
            eval_countif_surface(
                &[
                    range.clone(),
                    CallArgValue::Eval(EvalValue::Number(0.1 + 0.2))
                ],
                &NoResolver,
            ),
            Ok(EvalValue::Number(1.0))
        );
        assert_eq!(
            eval_countif_surface(
                &[range.clone(), scalar_text("<>0.30000000000000004")],
                &NoResolver
            ),
            Ok(EvalValue::Number(0.0))
        );
        assert_eq!(
            eval_countif_surface(
                &[range.clone(), scalar_text("<0.30000000000000004")],
                &NoResolver
            ),
            Ok(EvalValue::Number(0.0))
        );
        assert_eq!(
            eval_countif_surface(
                &[range.clone(), scalar_text("<=0.30000000000000004")],
                &NoResolver
            ),
            Ok(EvalValue::Number(1.0))
        );
        assert_eq!(
            eval_countif_surface(
                &[range.clone(), scalar_text(">0.30000000000000004")],
                &NoResolver
            ),
            Ok(EvalValue::Number(0.0))
        );
        assert_eq!(
            eval_countif_surface(&[range, scalar_text(">=0.30000000000000004")], &NoResolver),
            Ok(EvalValue::Number(1.0))
        );
    }

    #[test]
    fn criteria_boundary_pair_uses_excel_truncation_style_matching_across_ifs_family() {
        let boundary_probe = ((123_456_789_012_345_f64 * 10.0) + 5.0) / 1.0e25;
        let boundary_stored = ((123_456_789_012_345_f64 * 10.0) + 4.0) / 1.0e25;
        let resolver = MapResolver {
            refs: HashMap::from([
                (
                    "A1:B1".to_string(),
                    EvalValue::Array(
                        EvalArray::from_rows(vec![vec![
                            ArrayCellValue::Number(boundary_stored),
                            ArrayCellValue::Number(2.0),
                        ]])
                        .unwrap(),
                    ),
                ),
                (
                    "C1:D1".to_string(),
                    EvalValue::Array(
                        EvalArray::from_rows(vec![vec![
                            ArrayCellValue::Number(10.0),
                            ArrayCellValue::Number(20.0),
                        ]])
                        .unwrap(),
                    ),
                ),
                (
                    "E1:F1".to_string(),
                    EvalValue::Array(
                        EvalArray::from_rows(vec![vec![text("Y"), text("Y")]]).unwrap(),
                    ),
                ),
            ]),
        };
        let criteria_range = area_ref("A1:B1");
        let target_range = area_ref("C1:D1");
        let tag_range = area_ref("E1:F1");

        assert_eq!(
            eval_countifs_surface(
                &[
                    criteria_range.clone(),
                    CallArgValue::Eval(EvalValue::Number(boundary_probe)),
                    tag_range.clone(),
                    scalar_text("Y"),
                ],
                &resolver,
            ),
            Ok(EvalValue::Number(1.0))
        );
        assert_eq!(
            eval_sumifs_surface(
                &[
                    target_range.clone(),
                    criteria_range.clone(),
                    CallArgValue::Eval(EvalValue::Number(boundary_probe)),
                    tag_range.clone(),
                    scalar_text("Y"),
                ],
                &resolver,
            ),
            Ok(EvalValue::Number(10.0))
        );
        assert_eq!(
            eval_averageifs_surface(
                &[
                    target_range.clone(),
                    criteria_range.clone(),
                    CallArgValue::Eval(EvalValue::Number(boundary_probe)),
                    tag_range.clone(),
                    scalar_text("Y"),
                ],
                &resolver,
            ),
            Ok(EvalValue::Number(10.0))
        );
        assert_eq!(
            eval_maxifs_surface(
                &[
                    target_range.clone(),
                    criteria_range.clone(),
                    CallArgValue::Eval(EvalValue::Number(boundary_probe)),
                    tag_range.clone(),
                    scalar_text("Y"),
                ],
                &resolver,
            ),
            Ok(EvalValue::Number(10.0))
        );
        assert_eq!(
            eval_minifs_surface(
                &[
                    target_range,
                    criteria_range,
                    CallArgValue::Eval(EvalValue::Number(boundary_probe)),
                    tag_range,
                    scalar_text("Y"),
                ],
                &resolver,
            ),
            Ok(EvalValue::Number(10.0))
        );
    }

    #[test]
    fn countif_supports_wildcards_and_blank_criteria() {
        let wild = eval_countif_surface(
            &[
                array(vec![vec![
                    text("alpha"),
                    text("beta"),
                    text("alp~*"),
                    text("alpine"),
                ]]),
                scalar_text("alp*"),
            ],
            &NoResolver,
        );
        assert_eq!(wild, Ok(EvalValue::Number(3.0)));

        let blank = eval_countif_surface(
            &[
                array(vec![vec![ArrayCellValue::EmptyCell, text(""), text("x")]]),
                scalar_text(""),
            ],
            &NoResolver,
        );
        assert_eq!(blank, Ok(EvalValue::Number(2.0)));
    }

    #[test]
    fn countifs_intersects_multiple_criteria_and_checks_shape() {
        let got = eval_countifs_surface(
            &[
                array(vec![vec![text("North"), text("South"), text("North")]]),
                scalar_text("north"),
                array(vec![vec![
                    ArrayCellValue::Number(5.0),
                    ArrayCellValue::Number(7.0),
                    ArrayCellValue::Number(8.0),
                ]]),
                scalar_text(">=6"),
            ],
            &NoResolver,
        );
        assert_eq!(got, Ok(EvalValue::Number(1.0)));

        let mismatch = eval_countifs_surface(
            &[
                array(vec![vec![
                    ArrayCellValue::Number(1.0),
                    ArrayCellValue::Number(2.0),
                ]]),
                scalar_text(">0"),
                array(vec![
                    vec![ArrayCellValue::Number(1.0)],
                    vec![ArrayCellValue::Number(2.0)],
                ]),
                scalar_text(">0"),
            ],
            &NoResolver,
        );
        assert_eq!(
            mismatch,
            Err(CriteriaEvalError::Domain(WorksheetErrorCode::Value))
        );
    }

    #[test]
    fn sumifs_filters_sum_range_by_multiple_criteria() {
        let got = eval_sumifs_surface(
            &[
                array(vec![vec![
                    ArrayCellValue::Number(10.0),
                    ArrayCellValue::Number(20.0),
                    ArrayCellValue::Number(30.0),
                ]]),
                array(vec![vec![text("A"), text("B"), text("A")]]),
                scalar_text("A"),
                array(vec![vec![
                    ArrayCellValue::Number(1.0),
                    ArrayCellValue::Number(2.0),
                    ArrayCellValue::Number(3.0),
                ]]),
                scalar_text(">1"),
            ],
            &NoResolver,
        );
        assert_eq!(got, Ok(EvalValue::Number(30.0)));
    }

    #[test]
    fn sumif_uses_omitted_sum_range_and_anchors_reference_sum_range() {
        let direct = eval_sumif_surface(
            &[
                array(vec![vec![
                    ArrayCellValue::Number(1.0),
                    ArrayCellValue::Number(2.0),
                    ArrayCellValue::Number(3.0),
                ]]),
                scalar_text(">1"),
            ],
            &NoResolver,
        );
        assert_eq!(direct, Ok(EvalValue::Number(5.0)));

        let resolver = MapResolver {
            refs: HashMap::from([
                (
                    "A1:A3".to_string(),
                    EvalValue::Array(
                        EvalArray::from_rows(vec![
                            vec![ArrayCellValue::Number(1.0)],
                            vec![ArrayCellValue::Number(1.0)],
                            vec![ArrayCellValue::Number(1.0)],
                        ])
                        .unwrap(),
                    ),
                ),
                ("B2".to_string(), EvalValue::Number(20.0)),
                (
                    "B2:B4".to_string(),
                    EvalValue::Array(
                        EvalArray::from_rows(vec![
                            vec![ArrayCellValue::Number(20.0)],
                            vec![ArrayCellValue::Number(30.0)],
                            vec![ArrayCellValue::Number(40.0)],
                        ])
                        .unwrap(),
                    ),
                ),
            ]),
        };
        let anchored = eval_sumif_surface(
            &[
                CallArgValue::Reference(ReferenceLike {
                    kind: ReferenceKind::Area,
                    target: "A1:A3".to_string(),
                }),
                scalar_text("1"),
                CallArgValue::Reference(ReferenceLike {
                    kind: ReferenceKind::A1,
                    target: "B2".to_string(),
                }),
            ],
            &resolver,
        );
        assert_eq!(anchored, Ok(EvalValue::Number(90.0)));
    }

    #[test]
    fn averageif_uses_omitted_average_range_and_returns_div0_on_no_numeric_matches() {
        let direct = eval_averageif_surface(
            &[
                array(vec![vec![
                    ArrayCellValue::Number(1.0),
                    ArrayCellValue::Number(2.0),
                    ArrayCellValue::Number(3.0),
                ]]),
                scalar_text(">1"),
            ],
            &NoResolver,
        );
        assert_eq!(direct, Ok(EvalValue::Number(2.5)));

        let no_numeric = eval_averageif_surface(
            &[array(vec![vec![text("x"), text("y")]]), scalar_text("*")],
            &NoResolver,
        );
        assert_eq!(no_numeric, Ok(EvalValue::Error(WorksheetErrorCode::Div0)));
    }

    #[test]
    fn averageif_top_left_anchors_reference_average_range_to_criteria_shape() {
        let resolver = MapResolver {
            refs: HashMap::from([
                (
                    "A1:A3".to_string(),
                    EvalValue::Array(
                        EvalArray::from_rows(vec![
                            vec![ArrayCellValue::Number(1.0)],
                            vec![ArrayCellValue::Number(1.0)],
                            vec![ArrayCellValue::Number(1.0)],
                        ])
                        .unwrap(),
                    ),
                ),
                ("B2".to_string(), EvalValue::Number(20.0)),
                (
                    "B2:B4".to_string(),
                    EvalValue::Array(
                        EvalArray::from_rows(vec![
                            vec![ArrayCellValue::Number(20.0)],
                            vec![ArrayCellValue::Number(30.0)],
                            vec![ArrayCellValue::Number(40.0)],
                        ])
                        .unwrap(),
                    ),
                ),
            ]),
        };

        let got = eval_averageif_surface(
            &[
                CallArgValue::Reference(ReferenceLike {
                    kind: ReferenceKind::Area,
                    target: "A1:A3".to_string(),
                }),
                scalar_text("1"),
                CallArgValue::Reference(ReferenceLike {
                    kind: ReferenceKind::A1,
                    target: "B2".to_string(),
                }),
            ],
            &resolver,
        );
        assert_eq!(got, Ok(EvalValue::Number(30.0)));
    }

    #[test]
    fn averageifs_ignores_non_numeric_average_cells() {
        let got = eval_averageifs_surface(
            &[
                array(vec![vec![
                    ArrayCellValue::Number(10.0),
                    text("skip"),
                    ArrayCellValue::Number(30.0),
                ]]),
                array(vec![vec![text("A"), text("A"), text("B")]]),
                scalar_text("A"),
            ],
            &NoResolver,
        );
        assert_eq!(got, Ok(EvalValue::Number(10.0)));
    }

    #[test]
    fn averageifs_requires_exact_shape_and_does_not_anchor_average_range() {
        let got = eval_averageifs_surface(
            &[
                array(vec![vec![ArrayCellValue::Number(10.0)]]),
                array(vec![vec![
                    ArrayCellValue::Number(1.0),
                    ArrayCellValue::Number(1.0),
                    ArrayCellValue::Number(1.0),
                ]]),
                scalar_text("1"),
            ],
            &NoResolver,
        );
        assert_eq!(
            got,
            Err(CriteriaEvalError::Domain(WorksheetErrorCode::Value))
        );
    }

    #[test]
    fn ftc_0692_and_ftc_0693_direct_array_ranges_return_shaped_value_errors() {
        let max = eval_maxifs_surface(
            &[
                array(vec![vec![
                    ArrayCellValue::Number(10.0),
                    ArrayCellValue::Number(20.0),
                    ArrayCellValue::Number(30.0),
                    ArrayCellValue::Number(40.0),
                    ArrayCellValue::Number(50.0),
                ]]),
                array(vec![vec![
                    text("a"),
                    text("b"),
                    text("a"),
                    text("b"),
                    text("a"),
                ]]),
                scalar_text("a"),
            ],
            &NoResolver,
        );
        let min = eval_minifs_surface(
            &[
                array(vec![vec![
                    ArrayCellValue::Number(10.0),
                    ArrayCellValue::Number(20.0),
                    ArrayCellValue::Number(30.0),
                    ArrayCellValue::Number(40.0),
                    ArrayCellValue::Number(50.0),
                ]]),
                array(vec![vec![
                    text("a"),
                    text("b"),
                    text("a"),
                    text("b"),
                    text("a"),
                ]]),
                scalar_text("b"),
            ],
            &NoResolver,
        );
        let expected = Ok(EvalValue::Array(
            EvalArray::from_rows(vec![vec![
                ArrayCellValue::Error(WorksheetErrorCode::Value),
                ArrayCellValue::Error(WorksheetErrorCode::Value),
                ArrayCellValue::Error(WorksheetErrorCode::Value),
                ArrayCellValue::Error(WorksheetErrorCode::Value),
                ArrayCellValue::Error(WorksheetErrorCode::Value),
            ]])
            .unwrap(),
        ));
        assert_eq!(max, expected.clone());
        assert_eq!(min, expected);
    }

    #[test]
    fn maxifs_and_minifs_return_extrema_or_zero_when_no_numeric_match_survives() {
        let resolver = MapResolver {
            refs: HashMap::from([
                (
                    "A1:C1".to_string(),
                    EvalValue::Array(
                        EvalArray::from_rows(vec![vec![
                            ArrayCellValue::Number(10.0),
                            ArrayCellValue::Number(25.0),
                            ArrayCellValue::Number(8.0),
                        ]])
                        .unwrap(),
                    ),
                ),
                (
                    "D1:F1".to_string(),
                    EvalValue::Array(
                        EvalArray::from_rows(vec![vec![text("A"), text("B"), text("A")]]).unwrap(),
                    ),
                ),
                (
                    "G1:H1".to_string(),
                    EvalValue::Array(
                        EvalArray::from_rows(vec![vec![text("x"), ArrayCellValue::EmptyCell]])
                            .unwrap(),
                    ),
                ),
                (
                    "I1:J1".to_string(),
                    EvalValue::Array(
                        EvalArray::from_rows(vec![vec![text("A"), text("A")]]).unwrap(),
                    ),
                ),
            ]),
        };
        let max = eval_maxifs_surface(
            &[area_ref("A1:C1"), area_ref("D1:F1"), scalar_text("A")],
            &resolver,
        );
        assert_eq!(max, Ok(EvalValue::Number(10.0)));

        let min = eval_minifs_surface(
            &[area_ref("A1:C1"), area_ref("D1:F1"), scalar_text("A")],
            &resolver,
        );
        assert_eq!(min, Ok(EvalValue::Number(8.0)));

        let no_match = eval_maxifs_surface(
            &[area_ref("G1:H1"), area_ref("I1:J1"), scalar_text("A")],
            &resolver,
        );
        assert_eq!(no_match, Ok(EvalValue::Number(0.0)));
    }

    #[test]
    fn sumifs_and_extrema_family_require_exact_reference_shape() {
        let resolver = MapResolver {
            refs: HashMap::from([
                (
                    "A1".to_string(),
                    EvalValue::Array(
                        EvalArray::from_rows(vec![vec![ArrayCellValue::Number(20.0)]]).unwrap(),
                    ),
                ),
                (
                    "B1:D1".to_string(),
                    EvalValue::Array(
                        EvalArray::from_rows(vec![vec![
                            ArrayCellValue::Number(1.0),
                            ArrayCellValue::Number(1.0),
                            ArrayCellValue::Number(1.0),
                        ]])
                        .unwrap(),
                    ),
                ),
            ]),
        };
        let mismatch_sum = eval_sumifs_surface(
            &[area_ref("A1"), area_ref("B1:D1"), scalar_text("1")],
            &resolver,
        );
        assert_eq!(
            mismatch_sum,
            Err(CriteriaEvalError::Domain(WorksheetErrorCode::Value))
        );

        let mismatch_max = eval_maxifs_surface(
            &[area_ref("A1"), area_ref("B1:D1"), scalar_text("1")],
            &resolver,
        );
        assert_eq!(
            mismatch_max,
            Err(CriteriaEvalError::Domain(WorksheetErrorCode::Value))
        );

        let mismatch_min = eval_minifs_surface(
            &[area_ref("A1"), area_ref("B1:D1"), scalar_text("1")],
            &resolver,
        );
        assert_eq!(
            mismatch_min,
            Err(CriteriaEvalError::Domain(WorksheetErrorCode::Value))
        );
    }

    #[test]
    fn criteria_meta_shapes_match_family() {
        assert_eq!(COUNTIF_META.function_id, "FUNC.COUNTIF");
        assert_eq!(COUNTIFS_META.function_id, "FUNC.COUNTIFS");
        assert_eq!(SUMIF_META.function_id, "FUNC.SUMIF");
        assert_eq!(SUMIFS_META.function_id, "FUNC.SUMIFS");
        assert_eq!(AVERAGEIF_META.function_id, "FUNC.AVERAGEIF");
        assert_eq!(AVERAGEIFS_META.function_id, "FUNC.AVERAGEIFS");
        assert_eq!(MAXIFS_META.function_id, "FUNC.MAXIFS");
        assert_eq!(MINIFS_META.function_id, "FUNC.MINIFS");
        assert_eq!(
            COUNTIF_META.arg_preparation_profile,
            ArgPreparationProfile::RefsVisibleInAdapter
        );
    }
}

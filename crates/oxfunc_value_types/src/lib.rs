#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvalError {
    ArityMismatch { expected: usize, actual: usize },
}

pub const EXCEL_TEXT_MAX_UTF16_CODE_UNITS: usize = 32_767;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Value {
    Number(f64),
    Error(EvalError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorksheetErrorCode {
    Null,
    Div0,
    Value,
    Ref,
    Name,
    Num,
    NA,
    Busy,
    GettingData,
    Spill,
    Calc,
    Field,
    Blocked,
    Connect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueTag {
    Number,
    Text,
    Logical,
    Error,
    Array,
    RichValue,
    ReferenceLike,
    MissingArg,
    EmptyCell,
    LambdaValue,
    ExtendedWrapper,
    NullLike,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallableOriginKind {
    HelperLambda,
    DefinedNameCallable,
    BuiltInCallable,
    ExternalRegisteredCallable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallableCaptureMode {
    NoCapture,
    LexicalCapture,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CallableArityShape {
    pub min: usize,
    pub max: usize,
}

impl CallableArityShape {
    pub const fn exact(n: usize) -> Self {
        Self { min: n, max: n }
    }

    pub const fn range(min: usize, max: usize) -> Self {
        Self { min, max }
    }

    pub const fn accepts(self, argc: usize) -> bool {
        argc >= self.min && argc <= self.max
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenceKind {
    A1,
    Area,
    MultiArea,
    ThreeD,
    Structured,
    SpillAnchor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceLike {
    pub kind: ReferenceKind,
    pub target: String,
}

impl ReferenceLike {
    pub fn new(kind: ReferenceKind, target: impl Into<String>) -> Self {
        Self {
            kind,
            target: target.into(),
        }
    }

    pub fn multi_area(targets: Vec<String>) -> Option<Self> {
        let normalized = normalize_multi_area_parts(targets)?;
        Some(Self {
            kind: ReferenceKind::MultiArea,
            target: format!("({})", normalized.join(",")),
        })
    }

    pub fn normalized(self) -> Self {
        match self.kind {
            ReferenceKind::MultiArea => {
                if let Some(parts) = self.multi_area_targets() {
                    return Self::multi_area(parts).unwrap_or(self);
                }
                Self {
                    kind: self.kind,
                    target: self.target.trim().to_string(),
                }
            }
            _ => Self {
                kind: self.kind,
                target: self.target.trim().to_string(),
            },
        }
    }

    pub fn multi_area_targets(&self) -> Option<Vec<String>> {
        if !matches!(self.kind, ReferenceKind::MultiArea) {
            return None;
        }
        split_multi_area_target(&self.target)
    }

    pub fn area_count(&self) -> usize {
        self.multi_area_targets().map_or(1, |parts| parts.len())
    }
}

fn normalize_multi_area_parts(targets: Vec<String>) -> Option<Vec<String>> {
    let mut normalized = Vec::with_capacity(targets.len());
    for target in targets {
        let trimmed = target.trim();
        if trimmed.is_empty() {
            return None;
        }
        normalized.push(trimmed.to_string());
    }
    if normalized.len() < 2 {
        return None;
    }
    Some(normalized)
}

fn split_multi_area_target(target: &str) -> Option<Vec<String>> {
    let trimmed = target.trim();
    if !(trimmed.starts_with('(') && trimmed.ends_with(')')) {
        return None;
    }

    let inner = &trimmed[1..trimmed.len() - 1];
    let mut parts = Vec::new();
    let mut depth = 0usize;
    let mut in_single_quote = false;
    let mut bracket_depth = 0usize;
    let mut start = 0usize;
    for (index, ch) in inner.char_indices() {
        match ch {
            '\'' => in_single_quote = !in_single_quote,
            '[' if !in_single_quote => bracket_depth += 1,
            ']' if !in_single_quote && bracket_depth > 0 => bracket_depth -= 1,
            '(' if !in_single_quote && bracket_depth == 0 => depth += 1,
            ')' if !in_single_quote && bracket_depth == 0 => depth = depth.checked_sub(1)?,
            ',' if !in_single_quote && bracket_depth == 0 && depth == 0 => {
                let part = inner[start..index].trim();
                if part.is_empty() {
                    return None;
                }
                parts.push(part.to_string());
                start = index + ch.len_utf8();
            }
            _ => {}
        }
    }

    if depth != 0 {
        return None;
    }

    let tail = inner[start..].trim();
    if tail.is_empty() {
        return None;
    }
    parts.push(tail.to_string());
    Some(parts)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArrayShape {
    pub rows: usize,
    pub cols: usize,
}

impl ArrayShape {
    pub const fn cell_count(self) -> usize {
        self.rows * self.cols
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExcelText {
    utf16_code_units: Vec<u16>,
}

impl ExcelText {
    pub fn from_utf16_code_units(utf16_code_units: Vec<u16>) -> Self {
        Self { utf16_code_units }
    }

    pub fn from_interop_assignment(input: &str) -> Self {
        let mut utf16_code_units: Vec<u16> = input.encode_utf16().collect();
        utf16_code_units.truncate(EXCEL_TEXT_MAX_UTF16_CODE_UNITS);
        Self { utf16_code_units }
    }

    pub fn len_utf16_code_units(&self) -> usize {
        self.utf16_code_units.len()
    }

    pub fn utf16_code_units(&self) -> &[u16] {
        &self.utf16_code_units
    }

    pub fn to_string_lossy(&self) -> String {
        String::from_utf16_lossy(&self.utf16_code_units)
    }

    pub fn has_dangling_high_surrogate_tail(&self) -> bool {
        self.utf16_code_units
            .last()
            .is_some_and(|u| (0xD800..=0xDBFF).contains(u))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArrayCellValue {
    Number(f64),
    Text(ExcelText),
    Logical(bool),
    Error(WorksheetErrorCode),
    EmptyCell,
}

impl ArrayCellValue {
    pub fn to_eval_value(&self) -> Option<EvalValue> {
        match self {
            Self::Number(n) => Some(EvalValue::Number(*n)),
            Self::Text(t) => Some(EvalValue::Text(t.clone())),
            Self::Logical(b) => Some(EvalValue::Logical(*b)),
            Self::Error(code) => Some(EvalValue::Error(*code)),
            Self::EmptyCell => None,
        }
    }
}

pub const INLINE_EVAL_ARRAY_CELL_CAPACITY: usize = 8;

#[derive(Debug, Clone)]
pub struct EvalArray {
    shape: ArrayShape,
    storage: EvalArrayStorage,
}

#[derive(Debug, Clone, PartialEq)]
enum EvalArrayStorage {
    Inline {
        len: usize,
        cells: [ArrayCellValue; INLINE_EVAL_ARRAY_CELL_CAPACITY],
    },
    Heap {
        cells: Vec<ArrayCellValue>,
    },
}

impl PartialEq for EvalArray {
    fn eq(&self, other: &Self) -> bool {
        self.shape == other.shape && self.cells() == other.cells()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RichValueKeyFlag {
    pub key: String,
    pub flag: String,
    pub value: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RichValueType {
    pub type_name: String,
    pub required_keys: Vec<String>,
    pub key_flags: Vec<RichValueKeyFlag>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RichValueData {
    Number(f64),
    Text(ExcelText),
    Logical(bool),
    Error(WorksheetErrorCode),
    EmptyCell,
    Array(RichArray),
    RichValue(Box<RichValue>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct RichArray {
    shape: ArrayShape,
    cells: Vec<RichValueData>,
}

impl RichArray {
    pub fn new(shape: ArrayShape, cells: Vec<RichValueData>) -> Option<Self> {
        if shape.rows == 0 || shape.cols == 0 || cells.len() != shape.cell_count() {
            return None;
        }
        Some(Self { shape, cells })
    }

    pub const fn shape(&self) -> ArrayShape {
        self.shape
    }

    pub fn get(&self, row: usize, col: usize) -> Option<&RichValueData> {
        if row >= self.shape.rows || col >= self.shape.cols {
            return None;
        }
        let index = row.checked_mul(self.shape.cols)?.checked_add(col)?;
        self.cells.get(index)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RichValueKeyValue {
    pub key: String,
    pub value: RichValueData,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RichValue {
    pub value_type: RichValueType,
    pub fallback: RichValueData,
    pub kvps: Vec<RichValueKeyValue>,
}

impl EvalArray {
    pub fn new(shape: ArrayShape, cells: Vec<ArrayCellValue>) -> Option<Self> {
        if shape.rows == 0 || shape.cols == 0 || cells.len() != shape.cell_count() {
            return None;
        }
        Some(Self {
            shape,
            storage: Self::storage_from_vec(cells),
        })
    }

    pub fn from_scalar(value: ArrayCellValue) -> Self {
        let mut cells = Self::empty_inline_cells();
        cells[0] = value;
        Self {
            shape: ArrayShape { rows: 1, cols: 1 },
            storage: EvalArrayStorage::Inline { len: 1, cells },
        }
    }

    pub fn from_cells_iter(
        shape: ArrayShape,
        cells: impl IntoIterator<Item = ArrayCellValue>,
    ) -> Option<Self> {
        if shape.rows == 0 || shape.cols == 0 {
            return None;
        }

        let expected = shape.cell_count();
        let mut inline = Self::empty_inline_cells();
        let mut inline_len = 0;
        let mut heap: Option<Vec<ArrayCellValue>> = None;

        for cell in cells {
            if let Some(heap_cells) = heap.as_mut() {
                heap_cells.push(cell);
            } else if inline_len < INLINE_EVAL_ARRAY_CELL_CAPACITY {
                inline[inline_len] = cell;
                inline_len += 1;
            } else {
                let mut heap_cells =
                    Vec::with_capacity(expected.max(INLINE_EVAL_ARRAY_CELL_CAPACITY + 1));
                heap_cells.extend(inline[..inline_len].iter().cloned());
                heap_cells.push(cell);
                heap = Some(heap_cells);
            }
        }

        match heap {
            Some(cells) if cells.len() == expected => Some(Self {
                shape,
                storage: EvalArrayStorage::Heap { cells },
            }),
            None if inline_len == expected => Some(Self {
                shape,
                storage: EvalArrayStorage::Inline {
                    len: inline_len,
                    cells: inline,
                },
            }),
            _ => None,
        }
    }

    pub fn from_rows(rows: Vec<Vec<ArrayCellValue>>) -> Option<Self> {
        let row_count = rows.len();
        let col_count = rows.first()?.len();
        if row_count == 0 || col_count == 0 || rows.iter().any(|row| row.len() != col_count) {
            return None;
        }

        let mut cells = Vec::with_capacity(row_count * col_count);
        for row in rows {
            cells.extend(row);
        }

        Self::new(
            ArrayShape {
                rows: row_count,
                cols: col_count,
            },
            cells,
        )
    }

    pub const fn shape(&self) -> ArrayShape {
        self.shape
    }

    pub fn get(&self, row: usize, col: usize) -> Option<&ArrayCellValue> {
        if row >= self.shape.rows || col >= self.shape.cols {
            return None;
        }
        let index = row.checked_mul(self.shape.cols)?.checked_add(col)?;
        self.cells().get(index)
    }

    pub fn iter_row_major(&self) -> impl Iterator<Item = &ArrayCellValue> {
        self.cells().iter()
    }

    pub fn row_slice(&self, row: usize) -> Option<&[ArrayCellValue]> {
        if row >= self.shape.rows {
            return None;
        }
        let start = row.checked_mul(self.shape.cols)?;
        let end = start.checked_add(self.shape.cols)?;
        self.cells().get(start..end)
    }

    fn cells(&self) -> &[ArrayCellValue] {
        match &self.storage {
            EvalArrayStorage::Inline { len, cells } => &cells[..*len],
            EvalArrayStorage::Heap { cells } => cells,
        }
    }

    fn storage_from_vec(cells: Vec<ArrayCellValue>) -> EvalArrayStorage {
        if cells.len() <= INLINE_EVAL_ARRAY_CELL_CAPACITY {
            let len = cells.len();
            let mut inline = Self::empty_inline_cells();
            for (index, cell) in cells.into_iter().enumerate() {
                inline[index] = cell;
            }
            EvalArrayStorage::Inline { len, cells: inline }
        } else {
            EvalArrayStorage::Heap { cells }
        }
    }

    fn empty_inline_cells() -> [ArrayCellValue; INLINE_EVAL_ARRAY_CELL_CAPACITY] {
        std::array::from_fn(|_| ArrayCellValue::EmptyCell)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LambdaValue {
    pub callable_token: String,
    pub origin_kind: CallableOriginKind,
    pub arity_shape: CallableArityShape,
    pub capture_mode: CallableCaptureMode,
    pub invocation_contract_ref: String,
}

impl LambdaValue {
    pub fn new(
        callable_token: impl Into<String>,
        origin_kind: CallableOriginKind,
        arity_shape: CallableArityShape,
        capture_mode: CallableCaptureMode,
        invocation_contract_ref: impl Into<String>,
    ) -> Self {
        Self {
            callable_token: callable_token.into(),
            origin_kind,
            arity_shape,
            capture_mode,
            invocation_contract_ref: invocation_contract_ref.into(),
        }
    }

    pub fn helper_lambda(
        callable_token: impl Into<String>,
        arity_shape: CallableArityShape,
        capture_mode: CallableCaptureMode,
        invocation_contract_ref: impl Into<String>,
    ) -> Self {
        Self::new(
            callable_token,
            CallableOriginKind::HelperLambda,
            arity_shape,
            capture_mode,
            invocation_contract_ref,
        )
    }

    pub fn defined_name_callable(
        callable_token: impl Into<String>,
        arity_shape: CallableArityShape,
        capture_mode: CallableCaptureMode,
        invocation_contract_ref: impl Into<String>,
    ) -> Self {
        Self::new(
            callable_token,
            CallableOriginKind::DefinedNameCallable,
            arity_shape,
            capture_mode,
            invocation_contract_ref,
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum EvalValue {
    Number(f64),
    Text(ExcelText),
    Logical(bool),
    Error(WorksheetErrorCode),
    Array(EvalArray),
    Reference(ReferenceLike),
    Lambda(LambdaValue),
}

#[derive(Debug, Clone, PartialEq)]
pub enum CellContentValue {
    Number(f64),
    Text(ExcelText),
    Logical(bool),
    Error(WorksheetErrorCode),
    EmptyCell,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CallArgValue {
    Eval(EvalValue),
    MissingArg,
    EmptyCell,
    Reference(ReferenceLike),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumberFormatHint {
    General,
    DateLike,
    Percentage,
    Currency,
    Scientific,
    Fraction,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellStyleHint {
    Hyperlink,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PresentationHint {
    pub number_format: Option<NumberFormatHint>,
    pub style: Option<CellStyleHint>,
}

impl PresentationHint {
    pub const fn number_format(number_format: NumberFormatHint) -> Self {
        Self {
            number_format: Some(number_format),
            style: None,
        }
    }

    pub const fn style(style: CellStyleHint) -> Self {
        Self {
            number_format: None,
            style: Some(style),
        }
    }

    pub const fn with_both(number_format: NumberFormatHint, style: CellStyleHint) -> Self {
        Self {
            number_format: Some(number_format),
            style: Some(style),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSurface {
    Worksheet,
    XllTransferable,
    ExtendedWorksheetOnly,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExtendedValue {
    Core(EvalValue),
    RichValue(Box<RichValue>),
    ValueWithPresentation {
        value: EvalValue,
        hint: PresentationHint,
    },
    ErrorWithMetadata {
        code: WorksheetErrorCode,
        surface: ErrorSurface,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueBoundary {
    CellContent,
    RawFunctionReturn,
    PublishedFormulaResult,
    CallArg,
    ReferenceDomain,
    ExtendedDomain,
}

impl ValueBoundary {
    pub const fn allows(self, tag: ValueTag) -> bool {
        match self {
            Self::CellContent => matches!(
                tag,
                ValueTag::Number
                    | ValueTag::Text
                    | ValueTag::Logical
                    | ValueTag::Error
                    | ValueTag::EmptyCell
            ),
            Self::RawFunctionReturn => matches!(
                tag,
                ValueTag::Number
                    | ValueTag::Text
                    | ValueTag::Logical
                    | ValueTag::Error
                    | ValueTag::Array
                    | ValueTag::RichValue
                    | ValueTag::ReferenceLike
                    | ValueTag::EmptyCell
                    | ValueTag::LambdaValue
            ),
            Self::PublishedFormulaResult => matches!(
                tag,
                ValueTag::Number
                    | ValueTag::Text
                    | ValueTag::Logical
                    | ValueTag::Error
                    | ValueTag::Array
                    | ValueTag::RichValue
                    | ValueTag::ReferenceLike
                    | ValueTag::LambdaValue
            ),
            Self::CallArg => matches!(
                tag,
                ValueTag::Number
                    | ValueTag::Text
                    | ValueTag::Logical
                    | ValueTag::Error
                    | ValueTag::Array
                    | ValueTag::RichValue
                    | ValueTag::ReferenceLike
                    | ValueTag::MissingArg
                    | ValueTag::EmptyCell
                    | ValueTag::LambdaValue
            ),
            Self::ReferenceDomain => matches!(tag, ValueTag::ReferenceLike),
            Self::ExtendedDomain => matches!(
                tag,
                ValueTag::Number
                    | ValueTag::Text
                    | ValueTag::Logical
                    | ValueTag::Error
                    | ValueTag::Array
                    | ValueTag::RichValue
                    | ValueTag::ReferenceLike
                    | ValueTag::LambdaValue
                    | ValueTag::ExtendedWrapper
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ArrayCellValue, ArrayShape, CallableArityShape, CallableCaptureMode, CallableOriginKind,
        CellStyleHint, EXCEL_TEXT_MAX_UTF16_CODE_UNITS, EvalArray, EvalValue, ExcelText,
        ExtendedValue, LambdaValue, NumberFormatHint, PresentationHint, RichArray, RichValue,
        RichValueData, RichValueKeyFlag, RichValueKeyValue, RichValueType, ValueBoundary, ValueTag,
    };

    #[test]
    fn published_formula_result_excludes_missing_empty_and_null() {
        assert!(!ValueBoundary::PublishedFormulaResult.allows(ValueTag::MissingArg));
        assert!(!ValueBoundary::PublishedFormulaResult.allows(ValueTag::EmptyCell));
        assert!(!ValueBoundary::PublishedFormulaResult.allows(ValueTag::NullLike));
    }

    #[test]
    fn raw_function_return_allows_empty_cell_but_not_missing_or_null() {
        assert!(ValueBoundary::RawFunctionReturn.allows(ValueTag::EmptyCell));
        assert!(!ValueBoundary::RawFunctionReturn.allows(ValueTag::MissingArg));
        assert!(!ValueBoundary::RawFunctionReturn.allows(ValueTag::NullLike));
    }

    #[test]
    fn cell_content_boundary_excludes_missing_lambda_and_null() {
        assert!(!ValueBoundary::CellContent.allows(ValueTag::MissingArg));
        assert!(!ValueBoundary::CellContent.allows(ValueTag::LambdaValue));
        assert!(!ValueBoundary::CellContent.allows(ValueTag::NullLike));
    }

    #[test]
    fn call_arg_boundary_allows_missing_and_empty() {
        assert!(ValueBoundary::CallArg.allows(ValueTag::MissingArg));
        assert!(ValueBoundary::CallArg.allows(ValueTag::EmptyCell));
        assert!(ValueBoundary::CallArg.allows(ValueTag::ReferenceLike));
    }

    #[test]
    fn reference_boundary_only_allows_reference_like() {
        assert!(ValueBoundary::ReferenceDomain.allows(ValueTag::ReferenceLike));
        assert!(!ValueBoundary::ReferenceDomain.allows(ValueTag::Number));
        assert!(!ValueBoundary::ReferenceDomain.allows(ValueTag::Array));
    }

    #[test]
    fn published_formula_result_allows_rich_value() {
        assert!(ValueBoundary::PublishedFormulaResult.allows(ValueTag::RichValue));
        assert!(ValueBoundary::CallArg.allows(ValueTag::RichValue));
        assert!(ValueBoundary::ExtendedDomain.allows(ValueTag::RichValue));
    }

    #[test]
    fn interop_assignment_truncates_ascii_to_32767_utf16_units() {
        let text = ExcelText::from_interop_assignment(&"x".repeat(40_000));
        assert_eq!(text.len_utf16_code_units(), EXCEL_TEXT_MAX_UTF16_CODE_UNITS);
        assert!(!text.has_dangling_high_surrogate_tail());
    }

    #[test]
    fn interop_assignment_can_leave_dangling_surrogate_tail() {
        let text = ExcelText::from_interop_assignment(&"😀".repeat(40_000));
        assert_eq!(text.len_utf16_code_units(), EXCEL_TEXT_MAX_UTF16_CODE_UNITS);
        assert!(text.has_dangling_high_surrogate_tail());
        assert!(text.to_string_lossy().ends_with('\u{FFFD}'));
    }

    #[test]
    fn eval_array_preserves_shape_and_row_major_access() {
        let array = EvalArray::from_rows(vec![
            vec![
                ArrayCellValue::Number(1.0),
                ArrayCellValue::Text(ExcelText::from_utf16_code_units(
                    "x".encode_utf16().collect(),
                )),
            ],
            vec![ArrayCellValue::Logical(true), ArrayCellValue::EmptyCell],
        ])
        .unwrap();

        assert_eq!(array.shape(), ArrayShape { rows: 2, cols: 2 });
        assert_eq!(array.get(0, 0), Some(&ArrayCellValue::Number(1.0)));
        assert_eq!(array.get(1, 1), Some(&ArrayCellValue::EmptyCell));
        assert_eq!(array.iter_row_major().count(), 4);
    }

    #[test]
    fn callable_arity_shape_exact_accepts_only_matching_arity() {
        let arity = CallableArityShape::exact(2);
        assert!(!arity.accepts(1));
        assert!(arity.accepts(2));
        assert!(!arity.accepts(3));
    }

    #[test]
    fn helper_and_defined_name_lambda_constructors_preserve_metadata() {
        let helper = LambdaValue::helper_lambda(
            "helper.lambda.1",
            CallableArityShape::exact(1),
            CallableCaptureMode::LexicalCapture,
            "helper.invoke.v1",
        );
        assert_eq!(helper.origin_kind, CallableOriginKind::HelperLambda);
        assert_eq!(helper.arity_shape, CallableArityShape::exact(1));
        assert_eq!(helper.capture_mode, CallableCaptureMode::LexicalCapture);
        assert_eq!(helper.invocation_contract_ref, "helper.invoke.v1");

        let defined = LambdaValue::defined_name_callable(
            "name.MyAdder",
            CallableArityShape::range(1, 1),
            CallableCaptureMode::NoCapture,
            "name.invoke.v1",
        );
        assert_eq!(defined.origin_kind, CallableOriginKind::DefinedNameCallable);
        assert_eq!(defined.arity_shape, CallableArityShape::exact(1));
        assert_eq!(defined.capture_mode, CallableCaptureMode::NoCapture);
        assert_eq!(defined.invocation_contract_ref, "name.invoke.v1");
    }

    #[test]
    fn rich_value_model_supports_fallback_and_nested_array_data() {
        let web_image_type = RichValueType {
            type_name: "_webimage".to_string(),
            required_keys: vec!["WebImageIdentifier".to_string()],
            key_flags: vec![RichValueKeyFlag {
                key: "_DisplayString".to_string(),
                flag: "ExcludeFromCalcComparison".to_string(),
                value: true,
            }],
        };

        let rich_array = RichArray::new(
            ArrayShape { rows: 1, cols: 2 },
            vec![
                RichValueData::Text(ExcelText::from_utf16_code_units(
                    "A".encode_utf16().collect(),
                )),
                RichValueData::Number(2.0),
            ],
        )
        .unwrap();

        let rich = RichValue {
            value_type: web_image_type,
            fallback: RichValueData::Text(ExcelText::from_utf16_code_units(
                "Sphere".encode_utf16().collect(),
            )),
            kvps: vec![
                RichValueKeyValue {
                    key: "WebImageIdentifier".to_string(),
                    value: RichValueData::Text(ExcelText::from_utf16_code_units(
                        "img-1".encode_utf16().collect(),
                    )),
                },
                RichValueKeyValue {
                    key: "Preview".to_string(),
                    value: RichValueData::Array(rich_array.clone()),
                },
            ],
        };

        assert_eq!(rich.value_type.type_name, "_webimage");
        assert_eq!(rich.kvps.len(), 2);
        match &rich.kvps[1].value {
            RichValueData::Array(arr) => {
                assert_eq!(arr.shape(), ArrayShape { rows: 1, cols: 2 });
                assert!(matches!(arr.get(1, 0), None));
            }
            _ => panic!("expected nested rich array"),
        }
    }

    #[test]
    fn presentation_hint_can_carry_number_format_only() {
        let wrapped = ExtendedValue::ValueWithPresentation {
            value: EvalValue::Number(46_102.0),
            hint: PresentationHint::number_format(NumberFormatHint::DateLike),
        };

        match wrapped {
            ExtendedValue::ValueWithPresentation { value, hint } => {
                assert_eq!(value, EvalValue::Number(46_102.0));
                assert_eq!(hint.number_format, Some(NumberFormatHint::DateLike));
                assert_eq!(hint.style, None);
            }
            _ => panic!("expected presentation wrapper"),
        }
    }

    #[test]
    fn presentation_hint_can_carry_style_only() {
        let wrapped = ExtendedValue::ValueWithPresentation {
            value: EvalValue::Text(ExcelText::from_interop_assignment("Go")),
            hint: PresentationHint::style(CellStyleHint::Hyperlink),
        };

        match wrapped {
            ExtendedValue::ValueWithPresentation { value, hint } => {
                assert_eq!(
                    value,
                    EvalValue::Text(ExcelText::from_interop_assignment("Go"))
                );
                assert_eq!(hint.number_format, None);
                assert_eq!(hint.style, Some(CellStyleHint::Hyperlink));
            }
            _ => panic!("expected presentation wrapper"),
        }
    }
}

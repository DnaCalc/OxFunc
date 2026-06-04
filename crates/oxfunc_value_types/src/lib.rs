use std::any::Any;
use std::rc::Rc;

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
    Empty,
    Missing,
    Array,
    ReferenceLike,
    RichValue,
    Callable,
    Presentation,
    ErrorMetadata,
    NullLike,
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
pub struct ReferenceSystemId(pub String);

impl ReferenceSystemId {
    pub fn excel_grid_v1() -> Self {
        Self("excel.grid.v1".to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextualReferenceIdentity {
    pub kind: ReferenceKind,
    pub text: ExcelText,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceHandleId {
    pub bytes: Vec<u8>,
}

impl ReferenceHandleId {
    pub fn from_bytes(bytes: impl Into<Vec<u8>>) -> Self {
        Self {
            bytes: bytes.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceHandle {
    pub id: ReferenceHandleId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompositeReferenceOperation {
    Union,
    Intersection,
    SpillExpansion,
    Selector,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompositeReferenceIdentity {
    pub operation: CompositeReferenceOperation,
    pub members: Vec<ReferenceLike>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReferenceIdentity {
    Textual(TextualReferenceIdentity),
    Opaque(ReferenceHandle),
    Composite(CompositeReferenceIdentity),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceDisplay {
    pub text: ExcelText,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceLike {
    pub system: ReferenceSystemId,
    pub identity: ReferenceIdentity,
    pub display: Option<ReferenceDisplay>,
    kind_cache: ReferenceKind,
    target_cache: String,
}

impl ReferenceLike {
    pub fn new(kind: ReferenceKind, target: impl Into<String>) -> Self {
        let target = target.into();
        Self {
            system: ReferenceSystemId::excel_grid_v1(),
            identity: ReferenceIdentity::Textual(TextualReferenceIdentity {
                kind,
                text: ExcelText::from_interop_assignment(&target),
            }),
            display: Some(ReferenceDisplay {
                text: ExcelText::from_interop_assignment(&target),
            }),
            kind_cache: kind,
            target_cache: target,
        }
    }

    pub fn typed(
        system: ReferenceSystemId,
        identity: ReferenceIdentity,
        display: Option<ReferenceDisplay>,
    ) -> Self {
        let (kind_cache, target_cache) = reference_display_projection(&identity, display.as_ref());
        Self {
            system,
            identity,
            display,
            kind_cache,
            target_cache,
        }
    }

    pub fn textual(
        system: ReferenceSystemId,
        kind: ReferenceKind,
        text: ExcelText,
        display: Option<ReferenceDisplay>,
    ) -> Self {
        Self::typed(
            system,
            ReferenceIdentity::Textual(TextualReferenceIdentity { kind, text }),
            display,
        )
    }

    pub fn opaque(
        system: ReferenceSystemId,
        handle: ReferenceHandle,
        display: Option<ReferenceDisplay>,
    ) -> Self {
        Self::typed(system, ReferenceIdentity::Opaque(handle), display)
    }

    pub fn composite(
        system: ReferenceSystemId,
        operation: CompositeReferenceOperation,
        members: Vec<ReferenceLike>,
        display: Option<ReferenceDisplay>,
    ) -> Self {
        Self::typed(
            system,
            ReferenceIdentity::Composite(CompositeReferenceIdentity { operation, members }),
            display,
        )
    }

    pub fn multi_area(targets: Vec<String>) -> Option<Self> {
        let normalized = normalize_multi_area_parts(targets)?;
        Some(Self::new(
            ReferenceKind::MultiArea,
            format!("({})", normalized.join(",")),
        ))
    }

    pub fn kind(&self) -> ReferenceKind {
        self.kind_cache
    }

    pub fn target(&self) -> &str {
        self.target_cache.as_str()
    }

    pub fn normalized(self) -> Self {
        if !matches!(self.identity, ReferenceIdentity::Textual(_)) {
            return self;
        }

        match self.kind() {
            ReferenceKind::MultiArea => {
                if let Some(parts) = self.multi_area_targets() {
                    return Self::multi_area(parts).unwrap_or(self);
                }
                Self::new(self.kind(), self.target().trim().to_string())
            }
            _ => Self::new(self.kind(), self.target().trim().to_string()),
        }
    }

    pub fn multi_area_targets(&self) -> Option<Vec<String>> {
        if !matches!(self.kind(), ReferenceKind::MultiArea) {
            return None;
        }
        split_multi_area_target(self.target())
    }

    pub fn area_count(&self) -> usize {
        self.multi_area_targets().map_or(1, |parts| parts.len())
    }
}

fn reference_display_projection(
    identity: &ReferenceIdentity,
    display: Option<&ReferenceDisplay>,
) -> (ReferenceKind, String) {
    match identity {
        ReferenceIdentity::Textual(textual) => (textual.kind, textual.text.to_string_lossy()),
        ReferenceIdentity::Opaque(_) => (
            ReferenceKind::Structured,
            display
                .map(|value| value.text.to_string_lossy())
                .unwrap_or_else(|| "<opaque-reference>".to_string()),
        ),
        ReferenceIdentity::Composite(composite) => {
            let target = display
                .map(|value| value.text.to_string_lossy())
                .unwrap_or_else(|| {
                    composite
                        .members
                        .iter()
                        .map(|member| member.target())
                        .collect::<Vec<_>>()
                        .join(",")
                });
            (ReferenceKind::MultiArea, target)
        }
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
pub enum CoreValue {
    Number(f64),
    Text(ExcelText),
    Logical(bool),
    Error(WorksheetErrorCode),
    Empty,
    Missing,
    Array(CalcArray),
    Reference(ReferenceLike),
}

#[derive(Debug, Clone)]
pub struct CalcValue {
    pub core: CoreValue,
    pub rich: Option<Rc<RichValue>>,
}

impl PartialEq for CalcValue {
    fn eq(&self, other: &Self) -> bool {
        self.core == other.core
            && match (&self.rich, &other.rich) {
                (None, None) => true,
                (Some(left), Some(right)) => left.as_ref() == right.as_ref(),
                _ => false,
            }
    }
}

impl CalcValue {
    pub const fn new(core: CoreValue) -> Self {
        Self { core, rich: None }
    }

    pub fn with_rich(core: CoreValue, rich: RichValue) -> Self {
        Self {
            core,
            rich: Some(Rc::new(rich)),
        }
    }

    pub fn rich_object(core: CoreValue, object: RichObjectValue) -> Self {
        Self::with_rich(core, RichValue::Object(object))
    }

    pub fn with_presentation(core: CoreValue, hint: PresentationHint) -> Self {
        Self::with_rich(core, RichValue::Presentation(PresentationValue { hint }))
    }

    pub fn with_error_metadata(core: CoreValue, metadata: ErrorMetadataValue) -> Self {
        Self::with_rich(core, RichValue::ErrorMetadata(metadata))
    }

    pub fn number(value: f64) -> Self {
        Self::new(CoreValue::Number(value))
    }

    pub fn text(value: ExcelText) -> Self {
        Self::new(CoreValue::Text(value))
    }

    pub fn logical(value: bool) -> Self {
        Self::new(CoreValue::Logical(value))
    }

    pub fn error(code: WorksheetErrorCode) -> Self {
        Self::new(CoreValue::Error(code))
    }

    pub fn empty() -> Self {
        Self::new(CoreValue::Empty)
    }

    pub fn missing() -> Self {
        Self::new(CoreValue::Missing)
    }

    pub fn array(array: CalcArray) -> Self {
        Self::new(CoreValue::Array(array))
    }

    pub fn reference(reference: ReferenceLike) -> Self {
        Self::new(CoreValue::Reference(reference))
    }

    pub fn callable(callable: CallableValue) -> Self {
        Self::with_rich(
            CoreValue::Error(WorksheetErrorCode::Calc),
            RichValue::Callable(callable),
        )
    }

    pub fn tag(&self) -> ValueTag {
        if let Some(rich) = self.rich.as_deref() {
            return match rich {
                RichValue::Object(_) => ValueTag::RichValue,
                RichValue::Callable(_) => ValueTag::Callable,
                RichValue::Presentation(_) => ValueTag::Presentation,
                RichValue::ErrorMetadata(_) => ValueTag::ErrorMetadata,
            };
        }

        match &self.core {
            CoreValue::Number(_) => ValueTag::Number,
            CoreValue::Text(_) => ValueTag::Text,
            CoreValue::Logical(_) => ValueTag::Logical,
            CoreValue::Error(_) => ValueTag::Error,
            CoreValue::Empty => ValueTag::Empty,
            CoreValue::Missing => ValueTag::Missing,
            CoreValue::Array(_) => ValueTag::Array,
            CoreValue::Reference(_) => ValueTag::ReferenceLike,
        }
    }

    pub fn core(&self) -> &CoreValue {
        &self.core
    }

    pub fn rich(&self) -> Option<&RichValue> {
        self.rich.as_deref()
    }

    pub fn as_number(&self) -> Option<f64> {
        match self.core {
            CoreValue::Number(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_text(&self) -> Option<&ExcelText> {
        match &self.core {
            CoreValue::Text(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_logical(&self) -> Option<bool> {
        match self.core {
            CoreValue::Logical(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_error(&self) -> Option<WorksheetErrorCode> {
        match self.core {
            CoreValue::Error(code) => Some(code),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&CalcArray> {
        match &self.core {
            CoreValue::Array(array) => Some(array),
            _ => None,
        }
    }

    pub fn as_reference(&self) -> Option<&ReferenceLike> {
        match &self.core {
            CoreValue::Reference(reference) => Some(reference),
            _ => None,
        }
    }

    pub fn is_empty(&self) -> bool {
        matches!(self.core, CoreValue::Empty)
    }

    pub fn is_missing(&self) -> bool {
        matches!(self.core, CoreValue::Missing)
    }

    pub fn rich_object_value(&self) -> Option<&RichObjectValue> {
        match self.rich() {
            Some(RichValue::Object(value)) => Some(value),
            _ => None,
        }
    }

    pub fn callable_value(&self) -> Option<&CallableValue> {
        match self.rich() {
            Some(RichValue::Callable(value)) => Some(value),
            _ => None,
        }
    }

    pub fn presentation_value(&self) -> Option<&PresentationValue> {
        match self.rich() {
            Some(RichValue::Presentation(value)) => Some(value),
            _ => None,
        }
    }

    pub fn error_metadata_value(&self) -> Option<&ErrorMetadataValue> {
        match self.rich() {
            Some(RichValue::ErrorMetadata(value)) => Some(value),
            _ => None,
        }
    }

    pub fn allowed_at(&self, boundary: ValueBoundary) -> bool {
        boundary.allows(self.tag())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CalcArray {
    shape: ArrayShape,
    cells: Vec<CalcValue>,
}

impl CalcArray {
    pub fn new(shape: ArrayShape, cells: Vec<CalcValue>) -> Option<Self> {
        if shape.rows == 0 || shape.cols == 0 || cells.len() != shape.cell_count() {
            return None;
        }
        Some(Self { shape, cells })
    }

    pub fn from_scalar(value: CalcValue) -> Option<Self> {
        Self::new(ArrayShape { rows: 1, cols: 1 }, vec![value])
    }

    pub fn from_rows(rows: Vec<Vec<CalcValue>>) -> Option<Self> {
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

    pub fn from_cells_iter(
        shape: ArrayShape,
        cells: impl IntoIterator<Item = CalcValue>,
    ) -> Option<Self> {
        if shape.rows == 0 || shape.cols == 0 {
            return None;
        }
        let cells: Vec<CalcValue> = cells.into_iter().collect();
        Self::new(shape, cells)
    }

    pub const fn shape(&self) -> ArrayShape {
        self.shape
    }

    pub const fn cell_count(&self) -> usize {
        self.shape.cell_count()
    }

    pub fn get(&self, row: usize, col: usize) -> Option<&CalcValue> {
        if row >= self.shape.rows || col >= self.shape.cols {
            return None;
        }
        let index = row.checked_mul(self.shape.cols)?.checked_add(col)?;
        self.cells.get(index)
    }

    pub fn iter_row_major(&self) -> impl Iterator<Item = &CalcValue> {
        self.cells.iter()
    }

    pub fn row_slice(&self, row: usize) -> Option<&[CalcValue]> {
        if row >= self.shape.rows {
            return None;
        }
        let start = row.checked_mul(self.shape.cols)?;
        let end = start.checked_add(self.shape.cols)?;
        self.cells.get(start..end)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RichValueKeyFlag {
    pub key: String,
    pub flag: String,
    pub value: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RichObjectType {
    pub type_name: String,
    pub required_keys: Vec<String>,
    pub key_flags: Vec<RichValueKeyFlag>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RichObjectData {
    Number(f64),
    Text(ExcelText),
    Logical(bool),
    Error(WorksheetErrorCode),
    Empty,
    Array(CalcArray),
    Object(Box<RichObjectValue>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct RichObjectKeyValue {
    pub key: String,
    pub value: RichObjectData,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RichObjectValue {
    pub value_type: RichObjectType,
    pub fallback: RichObjectData,
    pub kvps: Vec<RichObjectKeyValue>,
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

pub trait OpaqueCallable: std::fmt::Debug + 'static {
    fn as_any(&self) -> &dyn Any;
}

#[derive(Debug, Clone)]
pub struct CallableValue {
    pub arity: CallableArityShape,
    pub summary: String,
    pub handle: Rc<dyn OpaqueCallable>,
}

impl PartialEq for CallableValue {
    fn eq(&self, other: &Self) -> bool {
        self.arity == other.arity && Rc::ptr_eq(&self.handle, &other.handle)
    }
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PresentationValue {
    pub hint: PresentationHint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSurface {
    Worksheet,
    XllTransferable,
    ExtendedWorksheetOnly,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorMetadataValue {
    pub surface: ErrorSurface,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RichValue {
    Object(RichObjectValue),
    Callable(CallableValue),
    Presentation(PresentationValue),
    ErrorMetadata(ErrorMetadataValue),
}

pub type RichValueType = RichObjectType;
pub type RichValueData = RichObjectData;
pub type RichValueKeyValue = RichObjectKeyValue;

#[derive(Debug, Clone, PartialEq)]
pub enum CellContentValue {
    Number(f64),
    Text(ExcelText),
    Logical(bool),
    Error(WorksheetErrorCode),
    Empty,
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
                    | ValueTag::Empty
            ),
            Self::RawFunctionReturn => matches!(
                tag,
                ValueTag::Number
                    | ValueTag::Text
                    | ValueTag::Logical
                    | ValueTag::Error
                    | ValueTag::Empty
                    | ValueTag::Array
                    | ValueTag::ReferenceLike
                    | ValueTag::RichValue
                    | ValueTag::Callable
                    | ValueTag::Presentation
                    | ValueTag::ErrorMetadata
            ),
            Self::PublishedFormulaResult => matches!(
                tag,
                ValueTag::Number
                    | ValueTag::Text
                    | ValueTag::Logical
                    | ValueTag::Error
                    | ValueTag::Array
                    | ValueTag::ReferenceLike
                    | ValueTag::RichValue
                    | ValueTag::Callable
                    | ValueTag::Presentation
                    | ValueTag::ErrorMetadata
            ),
            Self::CallArg => matches!(
                tag,
                ValueTag::Number
                    | ValueTag::Text
                    | ValueTag::Logical
                    | ValueTag::Error
                    | ValueTag::Empty
                    | ValueTag::Missing
                    | ValueTag::Array
                    | ValueTag::ReferenceLike
                    | ValueTag::RichValue
                    | ValueTag::Callable
                    | ValueTag::Presentation
                    | ValueTag::ErrorMetadata
            ),
            Self::ReferenceDomain => matches!(tag, ValueTag::ReferenceLike),
            Self::ExtendedDomain => matches!(
                tag,
                ValueTag::Number
                    | ValueTag::Text
                    | ValueTag::Logical
                    | ValueTag::Error
                    | ValueTag::Empty
                    | ValueTag::Array
                    | ValueTag::ReferenceLike
                    | ValueTag::RichValue
                    | ValueTag::Callable
                    | ValueTag::Presentation
                    | ValueTag::ErrorMetadata
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ArrayShape, CalcArray, CalcValue, CallableArityShape, CellStyleHint,
        CompositeReferenceOperation, CoreValue, EXCEL_TEXT_MAX_UTF16_CODE_UNITS,
        ErrorMetadataValue, ErrorSurface, ExcelText, NumberFormatHint, PresentationHint,
        PresentationValue, ReferenceDisplay, ReferenceHandle, ReferenceHandleId, ReferenceIdentity,
        ReferenceKind, ReferenceLike, ReferenceSystemId, RichObjectData, RichObjectKeyValue,
        RichObjectType, RichObjectValue, RichValue, RichValueKeyFlag, TextualReferenceIdentity,
        ValueBoundary, ValueTag, WorksheetErrorCode,
    };

    #[test]
    fn published_formula_result_excludes_missing_empty_and_null() {
        assert!(!ValueBoundary::PublishedFormulaResult.allows(ValueTag::Missing));
        assert!(!ValueBoundary::PublishedFormulaResult.allows(ValueTag::Empty));
        assert!(!ValueBoundary::PublishedFormulaResult.allows(ValueTag::NullLike));
    }

    #[test]
    fn raw_function_return_allows_empty_but_not_missing_or_null() {
        assert!(ValueBoundary::RawFunctionReturn.allows(ValueTag::Empty));
        assert!(!ValueBoundary::RawFunctionReturn.allows(ValueTag::Missing));
        assert!(!ValueBoundary::RawFunctionReturn.allows(ValueTag::NullLike));
    }

    #[test]
    fn cell_content_boundary_excludes_missing_callable_and_null() {
        assert!(!ValueBoundary::CellContent.allows(ValueTag::Missing));
        assert!(!ValueBoundary::CellContent.allows(ValueTag::Callable));
        assert!(!ValueBoundary::CellContent.allows(ValueTag::NullLike));
    }

    #[test]
    fn call_arg_boundary_allows_missing_and_empty() {
        assert!(ValueBoundary::CallArg.allows(ValueTag::Missing));
        assert!(ValueBoundary::CallArg.allows(ValueTag::Empty));
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
    fn calc_array_preserves_shape_and_row_major_access() {
        let array = CalcArray::from_rows(vec![
            vec![
                CalcValue::number(1.0),
                CalcValue::text(ExcelText::from_utf16_code_units(
                    "x".encode_utf16().collect(),
                )),
            ],
            vec![CalcValue::logical(true), CalcValue::empty()],
        ])
        .unwrap();

        assert_eq!(array.shape(), ArrayShape { rows: 2, cols: 2 });
        assert_eq!(
            array.get(0, 0),
            Some(&CalcValue {
                core: CoreValue::Number(1.0),
                rich: None
            })
        );
        assert_eq!(array.get(1, 1), Some(&CalcValue::empty()));
        assert_eq!(array.iter_row_major().count(), 4);
    }

    #[test]
    fn calc_array_admits_nested_arrays_and_missing_elements() {
        let nested = CalcValue::array(CalcArray::from_scalar(CalcValue::number(1.0)).unwrap());
        assert!(CalcArray::from_scalar(nested).is_some());
        assert!(CalcArray::from_scalar(CalcValue::missing()).is_some());
    }

    #[test]
    fn calc_value_scalar_projection_helpers_are_core_explicit() {
        let number = CalcValue::number(12.5);
        let text = CalcValue::text(ExcelText::from_interop_assignment("x"));
        let logical = CalcValue::logical(true);
        let error = CalcValue::error(WorksheetErrorCode::NA);

        assert_eq!(number.core(), &CoreValue::Number(12.5));
        assert_eq!(number.as_number(), Some(12.5));
        assert_eq!(
            text.as_text().map(ExcelText::to_string_lossy),
            Some("x".to_string())
        );
        assert_eq!(logical.as_logical(), Some(true));
        assert_eq!(error.as_error(), Some(WorksheetErrorCode::NA));
        assert!(CalcValue::empty().is_empty());
        assert!(CalcValue::missing().is_missing());
    }

    #[test]
    fn calc_value_reference_array_and_rich_projection_helpers_are_typed() {
        let reference = ReferenceLike::new(ReferenceKind::A1, "A1");
        let reference_value = CalcValue::reference(reference.clone());
        assert_eq!(reference_value.as_reference(), Some(&reference));

        let array = CalcArray::from_scalar(CalcValue::number(1.0)).unwrap();
        let array_value = CalcValue::array(array.clone());
        assert_eq!(array_value.as_array(), Some(&array));

        let presentation = CalcValue::with_presentation(
            CoreValue::Number(46_102.0),
            PresentationHint::number_format(NumberFormatHint::DateLike),
        );
        assert!(presentation.presentation_value().is_some());
        assert!(presentation.allowed_at(ValueBoundary::PublishedFormulaResult));

        let object = RichObjectValue {
            value_type: RichObjectType {
                type_name: "test.object".to_string(),
                required_keys: vec![],
                key_flags: vec![],
            },
            fallback: RichObjectData::Text(ExcelText::from_interop_assignment("fallback")),
            kvps: vec![],
        };
        let rich_object = CalcValue::rich_object(
            CoreValue::Text(ExcelText::from_interop_assignment("fallback")),
            object.clone(),
        );
        assert_eq!(rich_object.rich_object_value(), Some(&object));
        assert!(rich_object.allowed_at(ValueBoundary::ExtendedDomain));

        let metadata = CalcValue::with_error_metadata(
            CoreValue::Error(WorksheetErrorCode::Value),
            ErrorMetadataValue {
                surface: ErrorSurface::XllTransferable,
            },
        );
        assert_eq!(
            metadata.error_metadata_value().map(|value| value.surface),
            Some(ErrorSurface::XllTransferable)
        );
    }

    #[test]
    fn calc_value_callable_constructor_and_projection_are_rich_only() {
        #[derive(Debug)]
        struct TestCallable;

        impl super::OpaqueCallable for TestCallable {
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
        }

        let value = CalcValue::callable(super::CallableValue {
            arity: CallableArityShape::exact(1),
            summary: "test.callable".to_string(),
            handle: std::rc::Rc::new(TestCallable),
        });

        assert_eq!(value.core, CoreValue::Error(WorksheetErrorCode::Calc));
        assert_eq!(
            value.callable_value().map(|callable| callable.arity),
            Some(CallableArityShape::exact(1))
        );
    }

    #[test]
    fn textual_reference_carries_typed_identity_and_display_projection() {
        let reference = ReferenceLike::new(ReferenceKind::Area, " A1:B2 ").normalized();

        assert_eq!(reference.system, ReferenceSystemId::excel_grid_v1());
        assert_eq!(reference.kind(), ReferenceKind::Area);
        assert_eq!(reference.target(), "A1:B2");
        match &reference.identity {
            ReferenceIdentity::Textual(TextualReferenceIdentity { kind, text }) => {
                assert_eq!(*kind, ReferenceKind::Area);
                assert_eq!(text.to_string_lossy(), "A1:B2");
            }
            _ => panic!("expected textual reference identity"),
        }
    }

    #[test]
    fn opaque_reference_identity_does_not_require_display_as_identity() {
        let reference = ReferenceLike::opaque(
            ReferenceSystemId("dna.treecalc.v1".to_string()),
            ReferenceHandle {
                id: ReferenceHandleId::from_bytes([1, 2, 3]),
            },
            Some(ReferenceDisplay {
                text: ExcelText::from_interop_assignment("Node A"),
            }),
        );

        assert_eq!(reference.system.0, "dna.treecalc.v1");
        assert_eq!(reference.target(), "Node A");
        assert!(matches!(reference.identity, ReferenceIdentity::Opaque(_)));
        assert_eq!(
            reference
                .display
                .as_ref()
                .map(|display| display.text.to_string_lossy()),
            Some("Node A".to_string())
        );
    }

    #[test]
    fn composite_reference_can_represent_union_without_reparsing_display() {
        let left = ReferenceLike::new(ReferenceKind::Area, "A1:A2");
        let right = ReferenceLike::new(ReferenceKind::Area, "C1:C2");
        let composite = ReferenceLike::composite(
            ReferenceSystemId::excel_grid_v1(),
            CompositeReferenceOperation::Union,
            vec![left.clone(), right.clone()],
            Some(ReferenceDisplay {
                text: ExcelText::from_interop_assignment("(A1:A2,C1:C2)"),
            }),
        );

        assert_eq!(composite.kind(), ReferenceKind::MultiArea);
        assert_eq!(composite.target(), "(A1:A2,C1:C2)");
        match &composite.identity {
            ReferenceIdentity::Composite(identity) => {
                assert_eq!(identity.operation, CompositeReferenceOperation::Union);
                assert_eq!(identity.members, vec![left, right]);
            }
            _ => panic!("expected composite reference identity"),
        }
    }

    #[test]
    fn callable_arity_shape_exact_accepts_only_matching_arity() {
        let arity = CallableArityShape::exact(2);
        assert!(!arity.accepts(1));
        assert!(arity.accepts(2));
        assert!(!arity.accepts(3));
    }

    #[test]
    fn rich_value_model_supports_object_fallback_and_nested_array_data() {
        let web_image_type = RichObjectType {
            type_name: "_webimage".to_string(),
            required_keys: vec!["WebImageIdentifier".to_string()],
            key_flags: vec![RichValueKeyFlag {
                key: "_DisplayString".to_string(),
                flag: "ExcludeFromCalcComparison".to_string(),
                value: true,
            }],
        };

        let rich_array = CalcArray::new(
            ArrayShape { rows: 1, cols: 2 },
            vec![
                CalcValue::text(ExcelText::from_utf16_code_units(
                    "A".encode_utf16().collect(),
                )),
                CalcValue::number(2.0),
            ],
        )
        .unwrap();

        let rich = RichValue::Object(RichObjectValue {
            value_type: web_image_type,
            fallback: RichObjectData::Text(ExcelText::from_utf16_code_units(
                "Sphere".encode_utf16().collect(),
            )),
            kvps: vec![
                RichObjectKeyValue {
                    key: "WebImageIdentifier".to_string(),
                    value: RichObjectData::Text(ExcelText::from_utf16_code_units(
                        "img-1".encode_utf16().collect(),
                    )),
                },
                RichObjectKeyValue {
                    key: "Preview".to_string(),
                    value: RichObjectData::Array(rich_array.clone()),
                },
            ],
        });

        match rich {
            RichValue::Object(object) => {
                assert_eq!(object.value_type.type_name, "_webimage");
                assert_eq!(object.kvps.len(), 2);
                match &object.kvps[1].value {
                    RichObjectData::Array(arr) => {
                        assert_eq!(arr.shape(), ArrayShape { rows: 1, cols: 2 });
                        assert!(matches!(arr.get(1, 0), None));
                    }
                    _ => panic!("expected nested rich array"),
                }
            }
            _ => panic!("expected object rich value"),
        }
    }

    #[test]
    fn presentation_hint_is_rich_metadata_on_calc_value() {
        let value = CalcValue::with_rich(
            CoreValue::Number(46_102.0),
            RichValue::Presentation(PresentationValue {
                hint: PresentationHint::number_format(NumberFormatHint::DateLike),
            }),
        );

        assert_eq!(value.core, CoreValue::Number(46_102.0));
        assert_eq!(value.tag(), ValueTag::Presentation);
        match value.rich.as_deref() {
            Some(RichValue::Presentation(presentation)) => {
                assert_eq!(
                    presentation.hint.number_format,
                    Some(NumberFormatHint::DateLike)
                );
                assert_eq!(presentation.hint.style, None);
            }
            _ => panic!("expected presentation rich value"),
        }

        let hyperlink = CalcValue::with_rich(
            CoreValue::Text(ExcelText::from_interop_assignment("Go")),
            RichValue::Presentation(PresentationValue {
                hint: PresentationHint::style(CellStyleHint::Hyperlink),
            }),
        );
        assert!(matches!(
            hyperlink.rich.as_deref(),
            Some(RichValue::Presentation(_))
        ));
    }

    #[test]
    fn rich_object_tag_is_not_confused_with_core_fallback() {
        let value = CalcValue::with_rich(
            CoreValue::Text(ExcelText::from_interop_assignment("fallback")),
            RichValue::Object(RichObjectValue {
                value_type: RichObjectType {
                    type_name: "_webimage".to_string(),
                    required_keys: Vec::new(),
                    key_flags: Vec::new(),
                },
                fallback: RichObjectData::Text(ExcelText::from_interop_assignment("fallback")),
                kvps: Vec::new(),
            }),
        );

        assert_eq!(value.tag(), ValueTag::RichValue);
    }

    #[test]
    fn callable_values_have_calc_error_core_and_callable_tag() {
        #[derive(Debug)]
        struct TestCallable;
        impl super::OpaqueCallable for TestCallable {
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
        }

        let callable = CalcValue::callable(super::CallableValue {
            arity: CallableArityShape::exact(1),
            summary: "LAMBDA(x, x)".to_string(),
            handle: std::rc::Rc::new(TestCallable),
        });

        assert_eq!(callable.core, CoreValue::Error(WorksheetErrorCode::Calc));
        assert_eq!(callable.tag(), ValueTag::Callable);
    }
}

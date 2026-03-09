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
    ReferenceLike,
    MissingArg,
    EmptyCell,
    LambdaValue,
    ExtendedWrapper,
    NullLike,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenceKind {
    A1,
    Area,
    ThreeD,
    Structured,
    SpillAnchor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceLike {
    pub kind: ReferenceKind,
    pub target: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArrayShape {
    pub rows: usize,
    pub cols: usize,
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
pub enum EvalValue {
    Number(f64),
    Text(ExcelText),
    Logical(bool),
    Error(WorksheetErrorCode),
    Array(ArrayShape),
    Reference(ReferenceLike),
    Lambda(String),
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
pub enum ErrorSurface {
    Worksheet,
    XllTransferable,
    ExtendedWorksheetOnly,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExtendedValue {
    Core(EvalValue),
    NumberWithFormat {
        value: f64,
        format_hint: NumberFormatHint,
    },
    ErrorWithMetadata {
        code: WorksheetErrorCode,
        surface: ErrorSurface,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueBoundary {
    CellContent,
    EvalResult,
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
            Self::EvalResult => matches!(
                tag,
                ValueTag::Number
                    | ValueTag::Text
                    | ValueTag::Logical
                    | ValueTag::Error
                    | ValueTag::Array
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
                    | ValueTag::ReferenceLike
                    | ValueTag::LambdaValue
                    | ValueTag::ExtendedWrapper
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{EXCEL_TEXT_MAX_UTF16_CODE_UNITS, ExcelText, ValueBoundary, ValueTag};

    #[test]
    fn eval_boundary_excludes_missing_empty_and_null() {
        assert!(!ValueBoundary::EvalResult.allows(ValueTag::MissingArg));
        assert!(!ValueBoundary::EvalResult.allows(ValueTag::EmptyCell));
        assert!(!ValueBoundary::EvalResult.allows(ValueTag::NullLike));
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
}

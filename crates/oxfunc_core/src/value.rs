#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvalError {
    ArityMismatch { expected: usize, actual: usize },
}

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

#[derive(Debug, Clone, PartialEq)]
pub enum EvalValue {
    Number(f64),
    Text(String),
    Logical(bool),
    Error(WorksheetErrorCode),
    Array(ArrayShape),
    Reference(ReferenceLike),
    Lambda(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum CellContentValue {
    Number(f64),
    Text(String),
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
    use super::{ValueBoundary, ValueTag};

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
}

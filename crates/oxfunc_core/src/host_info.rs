use crate::value::{ArrayShape, EvalValue, ExcelText, ReferenceLike, WorksheetErrorCode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellInfoQuery {
    Address,
    Row,
    Col,
    Contents,
    Type,
    Filename,
    Format,
    Color,
    Parentheses,
    Prefix,
    Protect,
    Width,
    IsFormula,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InfoQuery {
    Directory,
    NumFile,
    Origin,
    OsVersion,
    Recalc,
    Release,
    System,
    MemAvail,
    MemUsed,
    TotMem,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HostInfoError {
    UnsupportedCellInfoQuery(CellInfoQuery),
    UnsupportedInfoQuery(InfoQuery),
    UnsupportedFormulaTextQuery,
    UnsupportedSheetIndexQuery,
    UnsupportedSheetCountQuery,
    UnsupportedAggregateReferenceContextQuery,
    UnsupportedWidthConversionProfileQuery(WidthConversionFunction),
    UnsupportedTranslateQuery,
    ProviderFailure { detail: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SheetIdentitySpec {
    CurrentSheet,
    Reference(ReferenceLike),
    SheetNameText(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SheetCountSpec {
    Workbook,
    Reference(ReferenceLike),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WidthConversionFunction {
    Asc,
    Dbcs,
    Jis,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WidthConversionMode {
    PassThrough,
    NarrowBasicWidthAndKana,
    WidenBasicWidthAndKana,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranslateRequest {
    pub text: ExcelText,
    pub source_language: Option<ExcelText>,
    pub target_language: Option<ExcelText>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TranslateProviderResult {
    Text(ExcelText),
    Busy,
    CapabilityDenied,
    ProviderError(WorksheetErrorCode),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AggregateCellContext {
    pub row_hidden_manual: bool,
    pub row_filtered_out: bool,
    pub nested_subtotal_or_aggregate: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AggregateReferenceContext {
    pub shape: ArrayShape,
    pub cells: Vec<AggregateCellContext>,
}

impl AggregateReferenceContext {
    pub fn new(shape: ArrayShape, cells: Vec<AggregateCellContext>) -> Option<Self> {
        if shape.rows == 0 || shape.cols == 0 || cells.len() != shape.cell_count() {
            return None;
        }
        Some(Self { shape, cells })
    }

    pub fn get(&self, row: usize, col: usize) -> Option<&AggregateCellContext> {
        if row >= self.shape.rows || col >= self.shape.cols {
            return None;
        }
        let index = row.checked_mul(self.shape.cols)?.checked_add(col)?;
        self.cells.get(index)
    }
}

pub trait HostInfoProvider {
    fn query_cell_info(
        &self,
        query: CellInfoQuery,
        _reference: Option<&ReferenceLike>,
    ) -> Result<EvalValue, HostInfoError> {
        Err(HostInfoError::UnsupportedCellInfoQuery(query))
    }

    fn query_info(&self, query: InfoQuery) -> Result<EvalValue, HostInfoError> {
        Err(HostInfoError::UnsupportedInfoQuery(query))
    }

    fn query_formula_text(&self, _reference: &ReferenceLike) -> Result<EvalValue, HostInfoError> {
        Err(HostInfoError::UnsupportedFormulaTextQuery)
    }

    fn query_sheet_index(&self, _spec: &SheetIdentitySpec) -> Result<EvalValue, HostInfoError> {
        Err(HostInfoError::UnsupportedSheetIndexQuery)
    }

    fn query_sheet_count(&self, _spec: &SheetCountSpec) -> Result<EvalValue, HostInfoError> {
        Err(HostInfoError::UnsupportedSheetCountQuery)
    }

    fn query_aggregate_reference_context(
        &self,
        _reference: &ReferenceLike,
    ) -> Result<AggregateReferenceContext, HostInfoError> {
        Err(HostInfoError::UnsupportedAggregateReferenceContextQuery)
    }

    fn query_width_conversion_mode(
        &self,
        function: WidthConversionFunction,
    ) -> Result<WidthConversionMode, HostInfoError> {
        Err(HostInfoError::UnsupportedWidthConversionProfileQuery(
            function,
        ))
    }

    fn query_translate(
        &self,
        _request: &TranslateRequest,
    ) -> Result<TranslateProviderResult, HostInfoError> {
        Err(HostInfoError::UnsupportedTranslateQuery)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::ReferenceKind;

    struct EmptyProvider;

    impl HostInfoProvider for EmptyProvider {}

    #[test]
    fn default_provider_rejects_cell_info_queries() {
        let provider = EmptyProvider;
        let got = provider.query_cell_info(
            CellInfoQuery::Filename,
            Some(&ReferenceLike {
                kind: ReferenceKind::A1,
                target: "A1".to_string(),
            }),
        );
        assert_eq!(
            got,
            Err(HostInfoError::UnsupportedCellInfoQuery(
                CellInfoQuery::Filename
            ))
        );
    }

    #[test]
    fn default_provider_rejects_info_queries() {
        let provider = EmptyProvider;
        let got = provider.query_info(InfoQuery::Directory);
        assert_eq!(
            got,
            Err(HostInfoError::UnsupportedInfoQuery(InfoQuery::Directory))
        );
    }

    #[test]
    fn default_provider_rejects_formula_text_query() {
        let provider = EmptyProvider;
        let got = provider.query_formula_text(&ReferenceLike {
            kind: ReferenceKind::A1,
            target: "A1".to_string(),
        });
        assert_eq!(got, Err(HostInfoError::UnsupportedFormulaTextQuery));
    }

    #[test]
    fn default_provider_rejects_sheet_queries() {
        let provider = EmptyProvider;
        assert_eq!(
            provider.query_sheet_index(&SheetIdentitySpec::CurrentSheet),
            Err(HostInfoError::UnsupportedSheetIndexQuery)
        );
        assert_eq!(
            provider.query_sheet_count(&SheetCountSpec::Workbook),
            Err(HostInfoError::UnsupportedSheetCountQuery)
        );
    }

    #[test]
    fn default_provider_rejects_aggregate_reference_context_query() {
        let provider = EmptyProvider;
        assert_eq!(
            provider.query_aggregate_reference_context(&ReferenceLike {
                kind: ReferenceKind::Area,
                target: "A1:A2".to_string(),
            }),
            Err(HostInfoError::UnsupportedAggregateReferenceContextQuery)
        );
    }

    #[test]
    fn default_provider_rejects_width_conversion_and_translate_queries() {
        let provider = EmptyProvider;
        assert_eq!(
            provider.query_width_conversion_mode(WidthConversionFunction::Asc),
            Err(HostInfoError::UnsupportedWidthConversionProfileQuery(
                WidthConversionFunction::Asc
            ))
        );
        assert_eq!(
            provider.query_translate(&TranslateRequest {
                text: ExcelText::from_utf16_code_units("hello".encode_utf16().collect()),
                source_language: Some(ExcelText::from_utf16_code_units(
                    "en".encode_utf16().collect()
                )),
                target_language: Some(ExcelText::from_utf16_code_units(
                    "es".encode_utf16().collect()
                )),
            }),
            Err(HostInfoError::UnsupportedTranslateQuery)
        );
    }

    #[test]
    fn aggregate_reference_context_validates_shape() {
        let shape = ArrayShape { rows: 2, cols: 1 };
        let cells = vec![
            AggregateCellContext {
                row_hidden_manual: false,
                row_filtered_out: false,
                nested_subtotal_or_aggregate: false,
            },
            AggregateCellContext {
                row_hidden_manual: true,
                row_filtered_out: false,
                nested_subtotal_or_aggregate: true,
            },
        ];
        let context = AggregateReferenceContext::new(shape, cells).expect("valid context");
        assert!(context.get(1, 0).unwrap().nested_subtotal_or_aggregate);
        assert!(AggregateReferenceContext::new(shape, Vec::new()).is_none());
    }
}

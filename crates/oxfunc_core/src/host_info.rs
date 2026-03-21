use crate::value::{EvalValue, ReferenceLike};

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
}

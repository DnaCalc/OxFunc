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
    ProviderFailure { detail: String },
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
}

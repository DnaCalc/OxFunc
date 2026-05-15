use crate::value::WorksheetErrorCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumericalReductionPolicy {
    SequentialLeftFold,
    PairwiseTree,
    KahanCompensated,
}

impl NumericalReductionPolicy {
    pub const fn stable_key(self) -> &'static str {
        match self {
            Self::SequentialLeftFold => "SequentialLeftFold",
            Self::PairwiseTree => "PairwiseTree",
            Self::KahanCompensated => "KahanCompensated",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorAlgebra {
    CanonicalExcelLegacy,
}

impl ErrorAlgebra {
    pub const fn stable_key(self) -> &'static str {
        match self {
            Self::CanonicalExcelLegacy => "CanonicalExcelLegacy",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemanticKernelRuntimeError {
    DeferredNumericalReductionPolicy { policy_key: &'static str },
}

pub fn reduce_numeric_sum<I>(
    policy: NumericalReductionPolicy,
    values: I,
) -> Result<f64, SemanticKernelRuntimeError>
where
    I: IntoIterator<Item = f64>,
{
    match policy {
        NumericalReductionPolicy::SequentialLeftFold => {
            let mut acc = 0.0;
            for value in values {
                acc += value;
            }
            Ok(acc)
        }
        NumericalReductionPolicy::PairwiseTree | NumericalReductionPolicy::KahanCompensated => Err(
            SemanticKernelRuntimeError::DeferredNumericalReductionPolicy {
                policy_key: policy.stable_key(),
            },
        ),
    }
}

pub fn collapse_worksheet_errors<I>(algebra: ErrorAlgebra, errors: I) -> Option<WorksheetErrorCode>
where
    I: IntoIterator<Item = WorksheetErrorCode>,
{
    match algebra {
        ErrorAlgebra::CanonicalExcelLegacy => errors
            .into_iter()
            .min_by_key(|code| canonical_excel_legacy_error_rank(*code)),
    }
}

fn canonical_excel_legacy_error_rank(code: WorksheetErrorCode) -> usize {
    match code {
        WorksheetErrorCode::Null => 0,
        WorksheetErrorCode::Div0 => 1,
        WorksheetErrorCode::Value => 2,
        WorksheetErrorCode::Ref => 3,
        WorksheetErrorCode::Name => 4,
        WorksheetErrorCode::Num => 5,
        WorksheetErrorCode::NA => 6,
        WorksheetErrorCode::Busy => 7,
        WorksheetErrorCode::GettingData => 8,
        WorksheetErrorCode::Spill => 9,
        WorksheetErrorCode::Calc => 10,
        WorksheetErrorCode::Field => 11,
        WorksheetErrorCode::Blocked => 12,
        WorksheetErrorCode::Connect => 13,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sequential_left_fold_is_order_visible_for_reduction_sensitive_sums() {
        let first = reduce_numeric_sum(
            NumericalReductionPolicy::SequentialLeftFold,
            [1.0e16, 1.0, -1.0e16],
        );
        let second = reduce_numeric_sum(
            NumericalReductionPolicy::SequentialLeftFold,
            [1.0e16, -1.0e16, 1.0],
        );

        assert_eq!(first, Ok(0.0));
        assert_eq!(second, Ok(1.0));
    }

    #[test]
    fn non_current_reduction_policies_are_explicitly_deferred() {
        assert_eq!(
            reduce_numeric_sum(NumericalReductionPolicy::PairwiseTree, [1.0, 2.0]),
            Err(
                SemanticKernelRuntimeError::DeferredNumericalReductionPolicy {
                    policy_key: "PairwiseTree",
                }
            )
        );
        assert_eq!(
            reduce_numeric_sum(NumericalReductionPolicy::KahanCompensated, [1.0, 2.0]),
            Err(
                SemanticKernelRuntimeError::DeferredNumericalReductionPolicy {
                    policy_key: "KahanCompensated",
                }
            )
        );
    }

    #[test]
    fn canonical_excel_legacy_error_algebra_collapses_by_precedence() {
        assert_eq!(
            collapse_worksheet_errors(
                ErrorAlgebra::CanonicalExcelLegacy,
                [
                    WorksheetErrorCode::NA,
                    WorksheetErrorCode::Value,
                    WorksheetErrorCode::Div0,
                ],
            ),
            Some(WorksheetErrorCode::Div0)
        );
    }
}

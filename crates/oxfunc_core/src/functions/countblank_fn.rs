use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{AggregateArgOrigin, AggregateArrayProvenance};
use crate::functions::adapters::{expand_aggregate_arg, sparse_reference_values_for_aggregate_arg};
use crate::resolver::ReferenceSystemProvider;
use crate::value::CalcValue;
use crate::value::{CalcArray, CoreValue, WorksheetErrorCode};

pub const COUNTBLANK_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.COUNTBLANK",
    arity: Arity { min: 1, max: 255 },
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::RefsVisibleInAdapter,
    coercion_lift_profile: CoercionLiftProfile::AggregateDirectAndRangeDualPolicy,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
    real_result_policy: FunctionMeta::DEFAULT_REAL_RESULT_POLICY,
};

#[derive(Debug, Clone, PartialEq)]
pub enum CountBlankEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Preparation(CoercionError),
}

fn value_error_array_like(array: &CalcArray) -> CalcValue {
    let shape = array.shape();
    let rows: Vec<Vec<CalcValue>> = (0..shape.rows)
        .map(|_| {
            (0..shape.cols)
                .map(|_| CalcValue::error(WorksheetErrorCode::Value))
                .collect()
        })
        .collect();
    CalcValue::array(CalcArray::from_rows(rows).expect("countblank error array shape"))
}

fn calc_value_counts_as_blank(value: &CalcValue) -> Result<bool, CoercionError> {
    match value.core() {
        CoreValue::Empty => Ok(true),
        CoreValue::Text(t) => Ok(t.utf16_code_units().is_empty()),
        CoreValue::Error(code) => Err(CoercionError::WorksheetError(*code)),
        CoreValue::Missing => Ok(false),
        _ => Ok(false),
    }
}

fn count_sparse_reference_blanks(
    arg: &CalcValue,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<Option<f64>, CountBlankEvalError> {
    let Some(values) = sparse_reference_values_for_aggregate_arg(arg, resolver)
        .map_err(CountBlankEvalError::Preparation)?
    else {
        return Ok(None);
    };

    let mut count = values
        .declared_cell_count()
        .saturating_sub(values.defined_cells.len()) as f64;
    for cell in &values.defined_cells {
        if calc_value_counts_as_blank(&cell.value).map_err(CountBlankEvalError::Preparation)? {
            count += 1.0;
        }
    }
    Ok(Some(count))
}

pub fn eval_countblank_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, CountBlankEvalError> {
    let argc = args.len();
    if !COUNTBLANK_META.arity.accepts(argc) {
        return Err(CountBlankEvalError::ArityMismatch {
            expected_min: COUNTBLANK_META.arity.min,
            expected_max: COUNTBLANK_META.arity.max,
            actual: argc,
        });
    }

    if argc == 1 {
        if let CoreValue::Array(array) = args[0].core() {
            return Ok(value_error_array_like(array));
        }
    }

    let mut count = 0.0;
    for arg in args {
        if let Some(sparse_count) = count_sparse_reference_blanks(arg, resolver)? {
            count += sparse_count;
            continue;
        }
        for item in expand_aggregate_arg(arg, resolver).map_err(CountBlankEvalError::Preparation)? {
            if matches!(
                item.1,
                AggregateArgOrigin::ArrayLike(AggregateArrayProvenance::OpaqueArrayValue)
                    | AggregateArgOrigin::ArrayLike(AggregateArrayProvenance::DirectArrayLiteral)
            ) {
                return Err(CountBlankEvalError::Preparation(
                    CoercionError::UnsupportedValueKind("countblank_array_substitute"),
                ));
            }
            if calc_value_counts_as_blank(&item.0).map_err(CountBlankEvalError::Preparation)? {
                count += 1.0;
            }
        }
    }

    Ok(CalcValue::number(count))
}

pub fn map_countblank_error_to_ws(e: &CountBlankEvalError) -> WorksheetErrorCode {
    match e {
        CountBlankEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        CountBlankEvalError::Preparation(CoercionError::WorksheetError(code)) => *code,
        CountBlankEvalError::Preparation(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::{
        ReferenceResolutionError, ReferenceSystemCapabilities, ResolvedReferenceCell,
        ResolvedReferenceExtent, ResolvedReferenceValues,
    };
    use crate::value::{CalcArray, ExcelText, ReferenceKind, ReferenceLike};
    use std::cell::Cell;

    struct MockResolver {
        resolved: Option<CalcValue>,
    }

    impl ReferenceSystemProvider for MockResolver {
        fn capabilities(&self) -> ReferenceSystemCapabilities {
            ReferenceSystemCapabilities::permissive_local()
        }

        fn dereference(
            &self,
            request: &crate::resolver::ReferenceDereferenceRequest,
        ) -> Result<CalcValue, crate::resolver::ReferenceResolutionError> {
            let reference = &request.reference;
            self.resolved.clone().ok_or(
                crate::resolver::ReferenceResolutionError::UnresolvedReference {
                    target: reference.target().to_string(),
                },
            )
        }
    }

    struct SparseResolver {
        values: ResolvedReferenceValues,
        dense_calls: Cell<usize>,
    }

    impl ReferenceSystemProvider for SparseResolver {
        fn capabilities(&self) -> ReferenceSystemCapabilities {
            ReferenceSystemCapabilities::permissive_local()
        }

        fn dereference(
            &self,
            request: &crate::resolver::ReferenceDereferenceRequest,
        ) -> Result<CalcValue, crate::resolver::ReferenceResolutionError> {
            let reference = &request.reference;
            self.dense_calls.set(self.dense_calls.get() + 1);
            Err(
                crate::resolver::ReferenceResolutionError::UnresolvedReference {
                    target: reference.target().to_string(),
                },
            )
        }

        fn enumerate_values(
            &self,
            _request: &crate::resolver::ReferenceEnumerationRequest,
        ) -> Result<Option<ResolvedReferenceValues>, ReferenceResolutionError> {
            Ok(Some(self.values.clone()))
        }
    }

    #[test]
    fn countblank_counts_empty_cells_and_empty_strings() {
        let got = eval_countblank_surface(
            &[CalcValue::reference(ReferenceLike::new(
                ReferenceKind::Area,
                "D1:D3".to_string(),
            ))],
            &MockResolver {
                resolved: Some(CalcValue::array(
                    CalcArray::from_rows(vec![vec![
                        CalcValue::empty(),
                        CalcValue::text(ExcelText::from_utf16_code_units(Vec::new())),
                        CalcValue::text(ExcelText::from_utf16_code_units(
                            "x".encode_utf16().collect(),
                        )),
                    ]])
                    .unwrap(),
                )),
            },
        );
        assert_eq!(got, Ok(CalcValue::number(2.0)));
    }

    #[test]
    fn countblank_admits_opaque_reference_value_through_generic_resolver() {
        let got = eval_countblank_surface(
            &[(CalcValue::reference(ReferenceLike::new(
                ReferenceKind::Area,
                "NameBackedRange".to_string(),
            )))],
            &MockResolver {
                resolved: Some(CalcValue::array(
                    CalcArray::from_rows(vec![vec![
                        CalcValue::empty(),
                        CalcValue::text(ExcelText::from_utf16_code_units(Vec::new())),
                        CalcValue::text(ExcelText::from_utf16_code_units(
                            "x".encode_utf16().collect(),
                        )),
                    ]])
                    .unwrap(),
                )),
            },
        );
        assert_eq!(got, Ok(CalcValue::number(2.0)));
    }

    #[test]
    fn countblank_propagates_errors() {
        let got = eval_countblank_surface(
            &[CalcValue::reference(ReferenceLike::new(
                ReferenceKind::Area,
                "D1".to_string(),
            ))],
            &MockResolver {
                resolved: Some(CalcValue::array(
                    CalcArray::from_rows(vec![vec![CalcValue::error(WorksheetErrorCode::NA)]])
                        .unwrap(),
                )),
            },
        );
        assert_eq!(
            got,
            Err(CountBlankEvalError::Preparation(
                CoercionError::WorksheetError(WorksheetErrorCode::NA,)
            ))
        );
    }

    #[test]
    fn countblank_single_array_valued_substitute_returns_shaped_value_error_array() {
        let got = eval_countblank_surface(
            &[(CalcValue::array(
                CalcArray::from_rows(vec![vec![
                    CalcValue::text(ExcelText::from_utf16_code_units(
                        "a".encode_utf16().collect(),
                    )),
                    CalcValue::number(1.0),
                    CalcValue::text(ExcelText::from_utf16_code_units(Vec::new())),
                    CalcValue::number(2.0),
                    CalcValue::text(ExcelText::from_utf16_code_units(
                        "b".encode_utf16().collect(),
                    )),
                ]])
                .unwrap(),
            ))],
            &MockResolver { resolved: None },
        );
        assert_eq!(
            got,
            Ok(CalcValue::array(
                CalcArray::from_rows(vec![vec![
                    CalcValue::error(WorksheetErrorCode::Value),
                    CalcValue::error(WorksheetErrorCode::Value),
                    CalcValue::error(WorksheetErrorCode::Value),
                    CalcValue::error(WorksheetErrorCode::Value),
                    CalcValue::error(WorksheetErrorCode::Value),
                ]])
                .unwrap(),
            ))
        );
    }

    #[test]
    fn countblank_multi_arg_array_valued_substitute_still_maps_to_scalar_value_error() {
        let err = eval_countblank_surface(
            &[
                (CalcValue::array(
                    CalcArray::from_rows(vec![vec![CalcValue::text(
                        ExcelText::from_utf16_code_units("a".encode_utf16().collect()),
                    )]])
                    .unwrap(),
                )),
                (CalcValue::number(1.0)),
            ],
            &MockResolver { resolved: None },
        )
        .unwrap_err();

        assert_eq!(map_countblank_error_to_ws(&err), WorksheetErrorCode::Value);
    }

    #[test]
    fn countblank_meta_preserves_reference_visibility_for_true_ranges() {
        assert_eq!(
            COUNTBLANK_META.arg_preparation_profile,
            ArgPreparationProfile::RefsVisibleInAdapter
        );
    }

    #[test]
    fn countblank_uses_sparse_extent_without_dense_resolution() {
        let resolver = SparseResolver {
            values: ResolvedReferenceValues::new(
                ResolvedReferenceExtent::new(1000, 1),
                vec![
                    ResolvedReferenceCell::new(1, 1, CalcValue::number(2.0)),
                    ResolvedReferenceCell::new(
                        2,
                        1,
                        CalcValue::text(ExcelText::from_utf16_code_units(Vec::new())),
                    ),
                    ResolvedReferenceCell::new(
                        3,
                        1,
                        CalcValue::text(ExcelText::from_utf16_code_units(
                            "x".encode_utf16().collect(),
                        )),
                    ),
                ],
                Some("reader:countblank-sparse".to_string()),
            ),
            dense_calls: Cell::new(0),
        };

        let got = eval_countblank_surface(
            &[CalcValue::reference(ReferenceLike::new(
                ReferenceKind::Area,
                "A1:A1000",
            ))],
            &resolver,
        );

        assert_eq!(got, Ok(CalcValue::number(998.0)));
        assert_eq!(resolver.dense_calls.get(), 0);
    }
}

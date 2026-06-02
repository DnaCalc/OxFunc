use crate::functions::adapters::{
    PreparedValue, expand_lookup_vector_arg, prepare_arg_values_only,
};
use crate::functions::xmatch::{
    XmatchEvalError, eval_xmatch_adapter_prepared, eval_xmatch_adapter_prepared_value,
    validate_xmatch_surface_arity,
};
use crate::resolver::ReferenceSystemProvider;
use crate::value::{
    FunctionArg, FunctionArray, FunctionArrayCell, FunctionValue, WorksheetErrorCode,
};

fn prepare_lookup_vector(
    lookup_array: &[FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<Vec<PreparedValue>, XmatchEvalError> {
    let mut prepared = Vec::new();
    for arg in lookup_array {
        prepared
            .extend(expand_lookup_vector_arg(arg, resolver).map_err(XmatchEvalError::Coercion)?);
    }
    Ok(prepared)
}

fn prepared_from_lookup_value_cell(cell: &FunctionArrayCell) -> PreparedValue {
    match cell {
        FunctionArrayCell::Number(n) => PreparedValue::Eval(FunctionValue::Number(*n)),
        FunctionArrayCell::Text(t) => PreparedValue::Eval(FunctionValue::Text(t.clone())),
        FunctionArrayCell::Logical(b) => PreparedValue::Eval(FunctionValue::Logical(*b)),
        FunctionArrayCell::Error(code) => PreparedValue::Eval(FunctionValue::Error(*code)),
        FunctionArrayCell::EmptyCell => PreparedValue::EmptyCell,
    }
}

fn map_xmatch_error_to_ws(e: &XmatchEvalError) -> WorksheetErrorCode {
    match e {
        XmatchEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        XmatchEvalError::EmptyLookupArray => WorksheetErrorCode::NA,
        XmatchEvalError::MissingArg => WorksheetErrorCode::Value,
        XmatchEvalError::EmptyCell => WorksheetErrorCode::NA,
        XmatchEvalError::Coercion(crate::coercion::CoercionError::WorksheetError(code)) => *code,
        XmatchEvalError::InvalidMatchMode(_) => WorksheetErrorCode::Value,
        XmatchEvalError::InvalidSearchMode(_) => WorksheetErrorCode::Value,
        XmatchEvalError::UnsupportedMatchModeForSeed(_)
        | XmatchEvalError::UnsupportedSearchModeForSeed(_) => WorksheetErrorCode::Value,
        XmatchEvalError::NotAvailable => WorksheetErrorCode::NA,
        XmatchEvalError::Coercion(_) | XmatchEvalError::UnsupportedValueKind(_) => {
            WorksheetErrorCode::Value
        }
    }
}

fn xmatch_result_to_array_cell(
    result: Result<FunctionValue, XmatchEvalError>,
) -> FunctionArrayCell {
    match result {
        Ok(FunctionValue::Number(n)) => FunctionArrayCell::Number(n),
        Ok(FunctionValue::Error(code)) => FunctionArrayCell::Error(code),
        Ok(_) => FunctionArrayCell::Error(WorksheetErrorCode::Value),
        Err(err) => FunctionArrayCell::Error(map_xmatch_error_to_ws(&err)),
    }
}

fn eval_xmatch_surface_prepared_value(
    lookup_value: &PreparedValue,
    lookup_array: &[PreparedValue],
    match_mode: Option<&PreparedValue>,
    search_mode: Option<&PreparedValue>,
) -> Result<FunctionValue, XmatchEvalError> {
    match lookup_value {
        PreparedValue::Eval(FunctionValue::Array(array)) => {
            let cells = array
                .iter_row_major()
                .map(prepared_from_lookup_value_cell)
                .map(|cell| {
                    xmatch_result_to_array_cell(eval_xmatch_adapter_prepared_value(
                        &cell,
                        lookup_array,
                        match_mode,
                        search_mode,
                    ))
                })
                .collect();
            Ok(FunctionValue::Array(
                FunctionArray::new(array.shape(), cells)
                    .expect("lookup-value array shape is valid"),
            ))
        }
        _ => {
            eval_xmatch_adapter_prepared_value(lookup_value, lookup_array, match_mode, search_mode)
        }
    }
}

pub fn eval_xmatch_surface(
    lookup_value: &FunctionArg,
    lookup_array: &[FunctionArg],
    match_mode: Option<&FunctionArg>,
    search_mode: Option<&FunctionArg>,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<f64, XmatchEvalError> {
    let argc = 2 + usize::from(match_mode.is_some()) + usize::from(search_mode.is_some());
    validate_xmatch_surface_arity(argc)?;

    let prepared_lookup_value =
        prepare_arg_values_only(lookup_value, resolver).map_err(XmatchEvalError::Coercion)?;
    let prepared_lookup_array = prepare_lookup_vector(lookup_array, resolver)?;
    let prepared_match_mode = match_mode
        .map(|arg| prepare_arg_values_only(arg, resolver))
        .transpose()
        .map_err(XmatchEvalError::Coercion)?;
    let prepared_search_mode = search_mode
        .map(|arg| prepare_arg_values_only(arg, resolver))
        .transpose()
        .map_err(XmatchEvalError::Coercion)?;

    eval_xmatch_adapter_prepared(
        &prepared_lookup_value,
        &prepared_lookup_array,
        prepared_match_mode.as_ref(),
        prepared_search_mode.as_ref(),
    )
}

pub fn eval_xmatch_surface_value(
    lookup_value: &FunctionArg,
    lookup_array: &[FunctionArg],
    match_mode: Option<&FunctionArg>,
    search_mode: Option<&FunctionArg>,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<FunctionValue, XmatchEvalError> {
    let argc = 2 + usize::from(match_mode.is_some()) + usize::from(search_mode.is_some());
    validate_xmatch_surface_arity(argc)?;

    let prepared_lookup_value =
        prepare_arg_values_only(lookup_value, resolver).map_err(XmatchEvalError::Coercion)?;
    let prepared_lookup_array = prepare_lookup_vector(lookup_array, resolver)?;
    let prepared_match_mode = match_mode
        .map(|arg| prepare_arg_values_only(arg, resolver))
        .transpose()
        .map_err(XmatchEvalError::Coercion)?;
    let prepared_search_mode = search_mode
        .map(|arg| prepare_arg_values_only(arg, resolver))
        .transpose()
        .map_err(XmatchEvalError::Coercion)?;

    eval_xmatch_surface_prepared_value(
        &prepared_lookup_value,
        &prepared_lookup_array,
        prepared_match_mode.as_ref(),
        prepared_search_mode.as_ref(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coercion::CoercionError;
    use crate::function::Arity;
    use crate::resolver::ReferenceSystemCapabilities;
    use crate::value::{
        ExcelText, FunctionArray, FunctionArrayCell, ReferenceKind, ReferenceLike,
        WorksheetErrorCode,
    };
    use std::collections::BTreeMap;

    struct MockResolver {
        caps: ReferenceSystemCapabilities,
        resolved_value: Option<FunctionValue>,
        by_target: BTreeMap<String, FunctionValue>,
    }

    impl ReferenceSystemProvider for MockResolver {
        fn capabilities(&self) -> ReferenceSystemCapabilities {
            self.caps
        }

        fn dereference(
            &self,
            request: &crate::resolver::ReferenceDereferenceRequest,
        ) -> Result<FunctionValue, crate::resolver::ReferenceResolutionError> {
            let reference = &request.reference;
            if let Some(value) = self.by_target.get(reference.target()) {
                return Ok(value.clone());
            }
            self.resolved_value.clone().ok_or(
                crate::resolver::ReferenceResolutionError::UnresolvedReference {
                    target: reference.target().to_string(),
                },
            )
        }
    }

    fn resolver() -> MockResolver {
        MockResolver {
            caps: ReferenceSystemCapabilities::permissive_local(),
            resolved_value: None,
            by_target: BTreeMap::new(),
        }
    }

    fn text_arg(s: &str) -> FunctionArg {
        FunctionArg::Eval(FunctionValue::Text(ExcelText::from_utf16_code_units(
            s.encode_utf16().collect(),
        )))
    }

    #[test]
    fn eval_xmatch_surface_uses_reference_preparation_for_lookup_value() {
        let r = MockResolver {
            caps: ReferenceSystemCapabilities::permissive_local(),
            resolved_value: Some(FunctionValue::Number(2.0)),
            by_target: BTreeMap::new(),
        };

        let got = eval_xmatch_surface(
            &FunctionArg::Reference(ReferenceLike::new(ReferenceKind::A1, "A1".to_string())),
            &[
                FunctionArg::Eval(FunctionValue::Number(1.0)),
                FunctionArg::Eval(FunctionValue::Number(2.0)),
            ],
            None,
            None,
            &r,
        );
        assert_eq!(got, Ok(2.0));
    }

    #[test]
    fn eval_xmatch_surface_flattens_lookup_array_argument() {
        let got = eval_xmatch_surface(
            &FunctionArg::Eval(FunctionValue::Number(2.0)),
            &[FunctionArg::Eval(FunctionValue::Array(
                FunctionArray::from_rows(vec![vec![
                    FunctionArrayCell::Number(1.0),
                    FunctionArrayCell::Number(2.0),
                    FunctionArrayCell::Number(3.0),
                ]])
                .unwrap(),
            ))],
            None,
            None,
            &resolver(),
        );
        assert_eq!(got, Ok(2.0));
    }

    #[test]
    fn eval_xmatch_surface_rejects_two_dimensional_lookup_array() {
        let got = eval_xmatch_surface(
            &FunctionArg::Eval(FunctionValue::Number(3.0)),
            &[FunctionArg::Eval(FunctionValue::Array(
                FunctionArray::from_rows(vec![
                    vec![
                        FunctionArrayCell::Number(1.0),
                        FunctionArrayCell::Number(2.0),
                    ],
                    vec![
                        FunctionArrayCell::Number(3.0),
                        FunctionArrayCell::Number(4.0),
                    ],
                ])
                .unwrap(),
            ))],
            None,
            None,
            &resolver(),
        );
        assert_eq!(
            got,
            Err(XmatchEvalError::Coercion(
                CoercionError::UnsupportedValueKind("two_dimensional_array")
            ))
        );
    }

    #[test]
    fn eval_xmatch_surface_value_wraps_index_as_eval_number() {
        let got = eval_xmatch_surface_value(
            &FunctionArg::Eval(FunctionValue::Number(3.0)),
            &[FunctionArg::Eval(FunctionValue::Number(3.0))],
            None,
            None,
            &resolver(),
        );
        assert_eq!(got, Ok(FunctionValue::Number(1.0)));
    }

    #[test]
    fn eval_xmatch_surface_value_spills_array_lookup_value_results() {
        let got = eval_xmatch_surface_value(
            &FunctionArg::Eval(FunctionValue::Array(
                FunctionArray::from_rows(vec![vec![
                    FunctionArrayCell::Number(1.0),
                    FunctionArrayCell::Number(2.0),
                    FunctionArrayCell::Number(3.0),
                ]])
                .unwrap(),
            )),
            &[FunctionArg::Eval(FunctionValue::Array(
                FunctionArray::from_rows(vec![vec![
                    FunctionArrayCell::Number(2.0),
                    FunctionArrayCell::Number(4.0),
                    FunctionArrayCell::Number(6.0),
                    FunctionArrayCell::Number(8.0),
                ]])
                .unwrap(),
            ))],
            None,
            None,
            &resolver(),
        );
        assert_eq!(
            got,
            Ok(FunctionValue::Array(
                FunctionArray::from_rows(vec![vec![
                    FunctionArrayCell::Error(WorksheetErrorCode::NA),
                    FunctionArrayCell::Number(1.0),
                    FunctionArrayCell::Error(WorksheetErrorCode::NA),
                ]])
                .unwrap(),
            ))
        );
    }

    #[test]
    fn eval_xmatch_surface_search_mode_uses_prepared_coercion() {
        let got = eval_xmatch_surface(
            &FunctionArg::Eval(FunctionValue::Number(2.0)),
            &[
                FunctionArg::Eval(FunctionValue::Number(2.0)),
                FunctionArg::Eval(FunctionValue::Number(2.0)),
            ],
            Some(&FunctionArg::Eval(FunctionValue::Number(0.0))),
            Some(&FunctionArg::Eval(FunctionValue::Number(-1.0))),
            &resolver(),
        );
        assert_eq!(got, Ok(2.0));
    }

    #[test]
    fn eval_xmatch_surface_accepts_provider_materialized_multi_area_lookup_array() {
        let mut by_target = BTreeMap::new();
        by_target.insert(
            "(A1:A2,C1)".to_string(),
            FunctionValue::Array(
                FunctionArray::from_rows(vec![vec![
                    FunctionArrayCell::Number(1.0),
                    FunctionArrayCell::Number(2.0),
                    FunctionArrayCell::Number(3.0),
                ]])
                .unwrap(),
            ),
        );
        let resolver = MockResolver {
            caps: ReferenceSystemCapabilities::permissive_local(),
            resolved_value: None,
            by_target,
        };

        let got = eval_xmatch_surface(
            &FunctionArg::Eval(FunctionValue::Number(3.0)),
            &[FunctionArg::Reference(ReferenceLike::new(
                ReferenceKind::MultiArea,
                "(A1:A2,C1)",
            ))],
            None,
            None,
            &resolver,
        );
        assert_eq!(got, Ok(3.0));
    }

    #[test]
    fn eval_xmatch_surface_lookup_array_error_is_skipped_as_non_match() {
        let got = eval_xmatch_surface(
            &FunctionArg::Eval(FunctionValue::Number(9.0)),
            &[
                FunctionArg::Eval(FunctionValue::Error(WorksheetErrorCode::Value)),
                FunctionArg::Eval(FunctionValue::Number(1.0)),
            ],
            None,
            None,
            &resolver(),
        );
        assert_eq!(got, Err(XmatchEvalError::NotAvailable));
    }

    #[test]
    fn xmatch_meta_arity_is_two_to_four_in_surface_context() {
        assert_eq!(
            crate::functions::xmatch::XMATCH_META.arity,
            Arity { min: 2, max: 4 }
        );
    }

    #[test]
    fn eval_xmatch_surface_coercion_error_from_mode_is_propagated() {
        let got = eval_xmatch_surface(
            &FunctionArg::Eval(FunctionValue::Number(1.0)),
            &[FunctionArg::Eval(FunctionValue::Number(1.0))],
            Some(&text_arg("asd")),
            None,
            &resolver(),
        );
        assert_eq!(
            got,
            Err(XmatchEvalError::Coercion(CoercionError::NonNumericText(
                "asd".to_string()
            )))
        );
    }
}

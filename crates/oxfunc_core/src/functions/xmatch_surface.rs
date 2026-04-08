use crate::functions::adapters::{
    PreparedArgValue, expand_lookup_vector_arg, prepare_arg_values_only,
};
use crate::functions::xmatch::{
    XmatchEvalError, eval_xmatch_adapter_prepared, eval_xmatch_adapter_prepared_value,
    validate_xmatch_surface_arity,
};
use crate::resolver::ReferenceResolver;
use crate::value::{ArrayCellValue, CallArgValue, EvalArray, EvalValue, WorksheetErrorCode};

fn prepare_lookup_vector(
    lookup_array: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<Vec<PreparedArgValue>, XmatchEvalError> {
    let mut prepared = Vec::new();
    for arg in lookup_array {
        prepared
            .extend(expand_lookup_vector_arg(arg, resolver).map_err(XmatchEvalError::Coercion)?);
    }
    Ok(prepared)
}

fn prepared_from_lookup_value_cell(cell: &ArrayCellValue) -> PreparedArgValue {
    match cell {
        ArrayCellValue::Number(n) => PreparedArgValue::Eval(EvalValue::Number(*n)),
        ArrayCellValue::Text(t) => PreparedArgValue::Eval(EvalValue::Text(t.clone())),
        ArrayCellValue::Logical(b) => PreparedArgValue::Eval(EvalValue::Logical(*b)),
        ArrayCellValue::Error(code) => PreparedArgValue::Eval(EvalValue::Error(*code)),
        ArrayCellValue::EmptyCell => PreparedArgValue::EmptyCell,
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

fn xmatch_result_to_array_cell(result: Result<EvalValue, XmatchEvalError>) -> ArrayCellValue {
    match result {
        Ok(EvalValue::Number(n)) => ArrayCellValue::Number(n),
        Ok(EvalValue::Error(code)) => ArrayCellValue::Error(code),
        Ok(_) => ArrayCellValue::Error(WorksheetErrorCode::Value),
        Err(err) => ArrayCellValue::Error(map_xmatch_error_to_ws(&err)),
    }
}

fn eval_xmatch_surface_prepared_value(
    lookup_value: &PreparedArgValue,
    lookup_array: &[PreparedArgValue],
    match_mode: Option<&PreparedArgValue>,
    search_mode: Option<&PreparedArgValue>,
) -> Result<EvalValue, XmatchEvalError> {
    match lookup_value {
        PreparedArgValue::Eval(EvalValue::Array(array)) => {
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
            Ok(EvalValue::Array(
                EvalArray::new(array.shape(), cells).expect("lookup-value array shape is valid"),
            ))
        }
        _ => {
            eval_xmatch_adapter_prepared_value(lookup_value, lookup_array, match_mode, search_mode)
        }
    }
}

pub fn eval_xmatch_surface(
    lookup_value: &CallArgValue,
    lookup_array: &[CallArgValue],
    match_mode: Option<&CallArgValue>,
    search_mode: Option<&CallArgValue>,
    resolver: &impl ReferenceResolver,
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
    lookup_value: &CallArgValue,
    lookup_array: &[CallArgValue],
    match_mode: Option<&CallArgValue>,
    search_mode: Option<&CallArgValue>,
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, XmatchEvalError> {
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
    use crate::resolver::{RefResolutionError, ResolverCapabilities};
    use crate::value::{
        ArrayCellValue, EvalArray, ExcelText, ReferenceKind, ReferenceLike, WorksheetErrorCode,
    };
    use std::collections::BTreeMap;

    struct MockResolver {
        caps: ResolverCapabilities,
        resolved_value: Option<EvalValue>,
        by_target: BTreeMap<String, EvalValue>,
    }

    impl ReferenceResolver for MockResolver {
        fn capabilities(&self) -> ResolverCapabilities {
            self.caps
        }

        fn resolve_reference(
            &self,
            reference: &ReferenceLike,
        ) -> Result<EvalValue, RefResolutionError> {
            if let Some(value) = self.by_target.get(&reference.target) {
                return Ok(value.clone());
            }
            self.resolved_value
                .clone()
                .ok_or(RefResolutionError::UnresolvedReference {
                    target: reference.target.clone(),
                })
        }
    }

    fn resolver() -> MockResolver {
        MockResolver {
            caps: ResolverCapabilities::permissive_local(),
            resolved_value: None,
            by_target: BTreeMap::new(),
        }
    }

    fn text_arg(s: &str) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
            s.encode_utf16().collect(),
        )))
    }

    #[test]
    fn eval_xmatch_surface_uses_reference_preparation_for_lookup_value() {
        let r = MockResolver {
            caps: ResolverCapabilities::permissive_local(),
            resolved_value: Some(EvalValue::Number(2.0)),
            by_target: BTreeMap::new(),
        };

        let got = eval_xmatch_surface(
            &CallArgValue::Reference(ReferenceLike {
                kind: ReferenceKind::A1,
                target: "A1".to_string(),
            }),
            &[
                CallArgValue::Eval(EvalValue::Number(1.0)),
                CallArgValue::Eval(EvalValue::Number(2.0)),
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
            &CallArgValue::Eval(EvalValue::Number(2.0)),
            &[CallArgValue::Eval(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Number(1.0),
                    ArrayCellValue::Number(2.0),
                    ArrayCellValue::Number(3.0),
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
            &CallArgValue::Eval(EvalValue::Number(3.0)),
            &[CallArgValue::Eval(EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Number(1.0), ArrayCellValue::Number(2.0)],
                    vec![ArrayCellValue::Number(3.0), ArrayCellValue::Number(4.0)],
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
            &CallArgValue::Eval(EvalValue::Number(3.0)),
            &[CallArgValue::Eval(EvalValue::Number(3.0))],
            None,
            None,
            &resolver(),
        );
        assert_eq!(got, Ok(EvalValue::Number(1.0)));
    }

    #[test]
    fn eval_xmatch_surface_value_spills_array_lookup_value_results() {
        let got = eval_xmatch_surface_value(
            &CallArgValue::Eval(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Number(1.0),
                    ArrayCellValue::Number(2.0),
                    ArrayCellValue::Number(3.0),
                ]])
                .unwrap(),
            )),
            &[CallArgValue::Eval(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Number(2.0),
                    ArrayCellValue::Number(4.0),
                    ArrayCellValue::Number(6.0),
                    ArrayCellValue::Number(8.0),
                ]])
                .unwrap(),
            ))],
            None,
            None,
            &resolver(),
        );
        assert_eq!(
            got,
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Error(WorksheetErrorCode::NA),
                    ArrayCellValue::Number(1.0),
                    ArrayCellValue::Error(WorksheetErrorCode::NA),
                ]])
                .unwrap(),
            ))
        );
    }

    #[test]
    fn eval_xmatch_surface_search_mode_uses_prepared_coercion() {
        let got = eval_xmatch_surface(
            &CallArgValue::Eval(EvalValue::Number(2.0)),
            &[
                CallArgValue::Eval(EvalValue::Number(2.0)),
                CallArgValue::Eval(EvalValue::Number(2.0)),
            ],
            Some(&CallArgValue::Eval(EvalValue::Number(0.0))),
            Some(&CallArgValue::Eval(EvalValue::Number(-1.0))),
            &resolver(),
        );
        assert_eq!(got, Ok(2.0));
    }

    #[test]
    fn eval_xmatch_surface_flattens_multi_area_lookup_array_argument() {
        let mut by_target = BTreeMap::new();
        by_target.insert(
            "A1:A2".to_string(),
            EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Number(1.0)],
                    vec![ArrayCellValue::Number(2.0)],
                ])
                .unwrap(),
            ),
        );
        by_target.insert("C1".to_string(), EvalValue::Number(3.0));
        let resolver = MockResolver {
            caps: ResolverCapabilities::permissive_local(),
            resolved_value: None,
            by_target,
        };

        let got = eval_xmatch_surface(
            &CallArgValue::Eval(EvalValue::Number(3.0)),
            &[CallArgValue::Reference(ReferenceLike::new(
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
            &CallArgValue::Eval(EvalValue::Number(9.0)),
            &[
                CallArgValue::Eval(EvalValue::Error(WorksheetErrorCode::Value)),
                CallArgValue::Eval(EvalValue::Number(1.0)),
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
            &CallArgValue::Eval(EvalValue::Number(1.0)),
            &[CallArgValue::Eval(EvalValue::Number(1.0))],
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

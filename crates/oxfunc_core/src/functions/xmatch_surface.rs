use crate::functions::adapters::{expand_lookup_vector_arg, prepare_arg_values_only};
use crate::functions::xmatch::{
    XmatchEvalError, eval_xmatch_adapter_prepared, eval_xmatch_adapter_prepared_value,
    validate_xmatch_surface_arity,
};
use crate::resolver::ReferenceSystemProvider;
use crate::value::CalcValue;
use crate::value::{CalcArray, CoreValue, WorksheetErrorCode};

fn prepare_lookup_vector(
    lookup_array: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<Vec<CalcValue>, XmatchEvalError> {
    let mut prepared = Vec::new();
    for arg in lookup_array {
        prepared
            .extend(expand_lookup_vector_arg(arg, resolver).map_err(XmatchEvalError::Coercion)?);
    }
    Ok(prepared)
}

fn prepared_from_lookup_value_cell(cell: &CalcValue) -> CalcValue {
    match cell.core() {
        CoreValue::Number(n) => CalcValue::number(*n),
        CoreValue::Text(t) => CalcValue::text(t.clone()),
        CoreValue::Logical(b) => CalcValue::logical(*b),
        CoreValue::Error(code) => CalcValue::error(*code),
        CoreValue::Empty | CoreValue::Missing => CalcValue::empty(),
        CoreValue::Array(_) | CoreValue::Reference(_) => {
            CalcValue::error(WorksheetErrorCode::Value)
        }
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

fn xmatch_result_to_array_cell(result: Result<CalcValue, XmatchEvalError>) -> CalcValue {
    match result {
        Ok(value) if matches!(value.core(), CoreValue::Number(_)) => value,
        Ok(value) if matches!(value.core(), CoreValue::Error(_)) => value,
        Ok(_) => CalcValue::error(WorksheetErrorCode::Value),
        Err(err) => CalcValue::error(map_xmatch_error_to_ws(&err)),
    }
}

fn eval_xmatch_surface_prepared_value(
    lookup_value: &CalcValue,
    lookup_array: &[CalcValue],
    match_mode: Option<&CalcValue>,
    search_mode: Option<&CalcValue>,
) -> Result<CalcValue, XmatchEvalError> {
    match lookup_value.core() {
        CoreValue::Array(array) => {
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
            Ok(CalcValue::array(
                CalcArray::new(array.shape(), cells).expect("lookup-value array shape is valid"),
            ))
        }
        _ => {
            eval_xmatch_adapter_prepared_value(lookup_value, lookup_array, match_mode, search_mode)
        }
    }
}

pub fn eval_xmatch_surface(
    lookup_value: &CalcValue,
    lookup_array: &[CalcValue],
    match_mode: Option<&CalcValue>,
    search_mode: Option<&CalcValue>,
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
    lookup_value: &CalcValue,
    lookup_array: &[CalcValue],
    match_mode: Option<&CalcValue>,
    search_mode: Option<&CalcValue>,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, XmatchEvalError> {
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
    use crate::resolver::{
        ReferenceIdentityKey, ReferenceSystemCapabilities, reference_identity_key,
    };
    use crate::value::{CalcArray, ExcelText, ReferenceKind, ReferenceLike, WorksheetErrorCode};
    use std::collections::BTreeMap;

    struct MockResolver {
        caps: ReferenceSystemCapabilities,
        resolved_value: Option<CalcValue>,
        by_reference: BTreeMap<ReferenceIdentityKey, CalcValue>,
    }

    impl ReferenceSystemProvider for MockResolver {
        fn capabilities(&self) -> ReferenceSystemCapabilities {
            self.caps
        }

        fn dereference(
            &self,
            request: &crate::resolver::ReferenceDereferenceRequest,
        ) -> Result<CalcValue, crate::resolver::ReferenceResolutionError> {
            let reference = &request.reference;
            if let Some(value) = self.by_reference.get(&reference_identity_key(reference)) {
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
            by_reference: BTreeMap::new(),
        }
    }

    fn text_arg(s: &str) -> CalcValue {
        CalcValue::text(ExcelText::from_utf16_code_units(s.encode_utf16().collect()))
    }

    #[test]
    fn eval_xmatch_surface_uses_reference_preparation_for_lookup_value() {
        let r = MockResolver {
            caps: ReferenceSystemCapabilities::permissive_local(),
            resolved_value: Some(CalcValue::number(2.0)),
            by_reference: BTreeMap::new(),
        };

        let got = eval_xmatch_surface(
            &CalcValue::reference(ReferenceLike::new(ReferenceKind::A1, "A1".to_string())),
            &[(CalcValue::number(1.0)), (CalcValue::number(2.0))],
            None,
            None,
            &r,
        );
        assert_eq!(got, Ok(2.0));
    }

    #[test]
    fn eval_xmatch_surface_flattens_lookup_array_argument() {
        let got = eval_xmatch_surface(
            &(CalcValue::number(2.0)),
            &[(CalcValue::array(
                CalcArray::from_rows(vec![vec![
                    CalcValue::number(1.0),
                    CalcValue::number(2.0),
                    CalcValue::number(3.0),
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
            &(CalcValue::number(3.0)),
            &[(CalcValue::array(
                CalcArray::from_rows(vec![
                    vec![CalcValue::number(1.0), CalcValue::number(2.0)],
                    vec![CalcValue::number(3.0), CalcValue::number(4.0)],
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
            &(CalcValue::number(3.0)),
            &[(CalcValue::number(3.0))],
            None,
            None,
            &resolver(),
        );
        assert_eq!(got, Ok(CalcValue::number(1.0)));
    }

    #[test]
    fn eval_xmatch_surface_value_spills_array_lookup_value_results() {
        let got = eval_xmatch_surface_value(
            &(CalcValue::array(
                CalcArray::from_rows(vec![vec![
                    CalcValue::number(1.0),
                    CalcValue::number(2.0),
                    CalcValue::number(3.0),
                ]])
                .unwrap(),
            )),
            &[(CalcValue::array(
                CalcArray::from_rows(vec![vec![
                    CalcValue::number(2.0),
                    CalcValue::number(4.0),
                    CalcValue::number(6.0),
                    CalcValue::number(8.0),
                ]])
                .unwrap(),
            ))],
            None,
            None,
            &resolver(),
        );
        assert_eq!(
            got,
            Ok(CalcValue::array(
                CalcArray::from_rows(vec![vec![
                    CalcValue::error(WorksheetErrorCode::NA),
                    CalcValue::number(1.0),
                    CalcValue::error(WorksheetErrorCode::NA),
                ]])
                .unwrap(),
            ))
        );
    }

    #[test]
    fn eval_xmatch_surface_search_mode_uses_prepared_coercion() {
        let got = eval_xmatch_surface(
            &(CalcValue::number(2.0)),
            &[(CalcValue::number(2.0)), (CalcValue::number(2.0))],
            Some(&(CalcValue::number(0.0))),
            Some(&(CalcValue::number(-1.0))),
            &resolver(),
        );
        assert_eq!(got, Ok(2.0));
    }

    #[test]
    fn eval_xmatch_surface_accepts_provider_materialized_multi_area_lookup_array() {
        let reference = ReferenceLike::new(ReferenceKind::MultiArea, "(A1:A2,C1)");
        let mut by_reference = BTreeMap::new();
        by_reference.insert(
            reference_identity_key(&reference),
            CalcValue::array(
                CalcArray::from_rows(vec![vec![
                    CalcValue::number(1.0),
                    CalcValue::number(2.0),
                    CalcValue::number(3.0),
                ]])
                .unwrap(),
            ),
        );
        let resolver = MockResolver {
            caps: ReferenceSystemCapabilities::permissive_local(),
            resolved_value: None,
            by_reference,
        };

        let got = eval_xmatch_surface(
            &(CalcValue::number(3.0)),
            &[CalcValue::reference(reference)],
            None,
            None,
            &resolver,
        );
        assert_eq!(got, Ok(3.0));
    }

    #[test]
    fn eval_xmatch_surface_lookup_array_error_is_skipped_as_non_match() {
        let got = eval_xmatch_surface(
            &(CalcValue::number(9.0)),
            &[
                (CalcValue::error(WorksheetErrorCode::Value)),
                (CalcValue::number(1.0)),
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
            &(CalcValue::number(1.0)),
            &[(CalcValue::number(1.0))],
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

use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::a1_refs::{format_absolute_address, parse_a1_reference};
use crate::functions::adapters::{coerce_prepared_to_text, prepare_arg_values_only};
use crate::host_info::{CellInfoQuery, HostInfoError, HostInfoProvider};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, ExcelText, ReferenceLike, WorksheetErrorCode};

pub const CELL_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.CELL",
    arity: Arity { min: 1, max: 2 },
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::VolatileContextual,
    host_interaction: HostInteractionClass::WorkbookState,
    thread_safety: ThreadSafetyClass::HostSerialized,
    arg_preparation_profile: ArgPreparationProfile::RefsVisibleInAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::CallerContext,
    surface_fec_dependency_profile: FecDependencyProfile::CallerContext,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CellInfoType {
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
}

#[derive(Debug, Clone, PartialEq)]
pub enum CellEvalError {
    ArityMismatch { expected: usize, actual: usize },
    InfoTypeCoercion(CoercionError),
    RefArgRequired,
    InvalidReferenceText(String),
    UnsupportedInfoType(String),
    RefResolution(CoercionError),
    HostInfoProviderMissing(CellInfoQuery),
    HostInfo(HostInfoError),
}

fn parse_info_type(
    arg: &CallArgValue,
    resolver: &impl ReferenceResolver,
) -> Result<CellInfoType, CellEvalError> {
    let prepared =
        prepare_arg_values_only(arg, resolver).map_err(CellEvalError::InfoTypeCoercion)?;
    let info = coerce_prepared_to_text(&prepared)
        .map_err(CellEvalError::InfoTypeCoercion)?
        .to_string_lossy()
        .trim()
        .to_ascii_lowercase();

    match info.as_str() {
        "address" => Ok(CellInfoType::Address),
        "row" => Ok(CellInfoType::Row),
        "col" => Ok(CellInfoType::Col),
        "contents" => Ok(CellInfoType::Contents),
        "type" => Ok(CellInfoType::Type),
        "filename" => Ok(CellInfoType::Filename),
        "format" => Ok(CellInfoType::Format),
        "color" => Ok(CellInfoType::Color),
        "parentheses" => Ok(CellInfoType::Parentheses),
        "prefix" => Ok(CellInfoType::Prefix),
        "protect" => Ok(CellInfoType::Protect),
        "width" => Ok(CellInfoType::Width),
        _ => Err(CellEvalError::UnsupportedInfoType(info)),
    }
}

fn parse_reference_arg(arg: &CallArgValue) -> Result<ReferenceLike, CellEvalError> {
    match arg {
        CallArgValue::Reference(r) => Ok(r.clone()),
        CallArgValue::Eval(EvalValue::Reference(r)) => Ok(r.clone()),
        _ => Err(CellEvalError::RefArgRequired),
    }
}

fn classify_type(value: &EvalValue) -> &'static str {
    match value {
        EvalValue::Text(_) => "l",
        EvalValue::Number(_) | EvalValue::Logical(_) | EvalValue::Error(_) => "v",
        EvalValue::Array(_) | EvalValue::Reference(_) | EvalValue::Lambda(_) => "v",
    }
}

fn host_query_for_info_type(info_type: CellInfoType) -> Option<CellInfoQuery> {
    match info_type {
        CellInfoType::Address => Some(CellInfoQuery::Address),
        CellInfoType::Row => Some(CellInfoQuery::Row),
        CellInfoType::Col => Some(CellInfoQuery::Col),
        CellInfoType::Contents => Some(CellInfoQuery::Contents),
        CellInfoType::Type => Some(CellInfoQuery::Type),
        CellInfoType::Filename => Some(CellInfoQuery::Filename),
        CellInfoType::Format => Some(CellInfoQuery::Format),
        CellInfoType::Color => Some(CellInfoQuery::Color),
        CellInfoType::Parentheses => Some(CellInfoQuery::Parentheses),
        CellInfoType::Prefix => Some(CellInfoQuery::Prefix),
        CellInfoType::Protect => Some(CellInfoQuery::Protect),
        CellInfoType::Width => Some(CellInfoQuery::Width),
    }
}

pub fn eval_cell_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
    host_info: Option<&dyn HostInfoProvider>,
) -> Result<EvalValue, CellEvalError> {
    if !CELL_META.arity.accepts(args.len()) {
        return Err(CellEvalError::ArityMismatch {
            expected: CELL_META.arity.min,
            actual: args.len(),
        });
    }

    let info_type = parse_info_type(&args[0], resolver)?;
    let reference = if args.len() >= 2 {
        Some(parse_reference_arg(&args[1])?)
    } else {
        None
    };

    if reference.is_none() {
        let query = host_query_for_info_type(info_type).expect("cell info query mapping");
        let provider = host_info.ok_or(CellEvalError::HostInfoProviderMissing(query))?;
        return provider
            .query_cell_info(query, None)
            .map_err(CellEvalError::HostInfo);
    }

    let reference = reference.expect("reference present");

    match info_type {
        CellInfoType::Address => {
            let parsed = parse_a1_reference(&reference.target)
                .ok_or_else(|| CellEvalError::InvalidReferenceText(reference.target.clone()))?;
            Ok(EvalValue::Text(ExcelText::from_utf16_code_units(
                format_absolute_address(&parsed)
                    .ok_or_else(|| CellEvalError::InvalidReferenceText(reference.target.clone()))?
                    .encode_utf16()
                    .collect(),
            )))
        }
        CellInfoType::Row => {
            let parsed = parse_a1_reference(&reference.target)
                .ok_or_else(|| CellEvalError::InvalidReferenceText(reference.target.clone()))?;
            Ok(EvalValue::Number(parsed.start_row as f64))
        }
        CellInfoType::Col => {
            let parsed = parse_a1_reference(&reference.target)
                .ok_or_else(|| CellEvalError::InvalidReferenceText(reference.target.clone()))?;
            Ok(EvalValue::Number(parsed.start_col as f64))
        }
        CellInfoType::Contents => resolver
            .resolve_reference(&reference)
            .map_err(CoercionError::RefResolution)
            .map_err(CellEvalError::RefResolution),
        CellInfoType::Type => {
            let value = resolver
                .resolve_reference(&reference)
                .map_err(CoercionError::RefResolution)
                .map_err(CellEvalError::RefResolution)?;
            Ok(EvalValue::Text(ExcelText::from_utf16_code_units(
                classify_type(&value).encode_utf16().collect(),
            )))
        }
        _ => {
            let query = host_query_for_info_type(info_type).expect("host query mapping");
            let provider = host_info.ok_or(CellEvalError::HostInfoProviderMissing(query))?;
            provider
                .query_cell_info(query, Some(&reference))
                .map_err(CellEvalError::HostInfo)
        }
    }
}

pub fn map_cell_error_to_ws(e: &CellEvalError) -> WorksheetErrorCode {
    match e {
        CellEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        CellEvalError::InfoTypeCoercion(CoercionError::WorksheetError(code)) => *code,
        CellEvalError::RefArgRequired => WorksheetErrorCode::Value,
        CellEvalError::InvalidReferenceText(_) => WorksheetErrorCode::Ref,
        CellEvalError::UnsupportedInfoType(_) => WorksheetErrorCode::Value,
        CellEvalError::RefResolution(CoercionError::WorksheetError(code)) => *code,
        CellEvalError::RefResolution(CoercionError::RefResolution(_)) => WorksheetErrorCode::Ref,
        CellEvalError::HostInfoProviderMissing(_) => WorksheetErrorCode::Value,
        CellEvalError::HostInfo(_) => WorksheetErrorCode::Value,
        CellEvalError::InfoTypeCoercion(_) | CellEvalError::RefResolution(_) => {
            WorksheetErrorCode::Value
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::{RefResolutionError, ResolverCapabilities};

    struct MockResolver {
        resolved: Option<EvalValue>,
    }

    struct MockHostInfoProvider {
        result: EvalValue,
    }

    impl HostInfoProvider for MockHostInfoProvider {
        fn query_cell_info(
            &self,
            query: CellInfoQuery,
            _reference: Option<&ReferenceLike>,
        ) -> Result<EvalValue, HostInfoError> {
            match query {
                CellInfoQuery::Filename => Ok(self.result.clone()),
                CellInfoQuery::Parentheses => Ok(self.result.clone()),
                CellInfoQuery::Row => Ok(self.result.clone()),
                other => Err(HostInfoError::UnsupportedCellInfoQuery(other)),
            }
        }
    }

    impl ReferenceResolver for MockResolver {
        fn capabilities(&self) -> ResolverCapabilities {
            ResolverCapabilities::permissive_local()
        }

        fn resolve_reference(
            &self,
            reference: &ReferenceLike,
        ) -> Result<EvalValue, RefResolutionError> {
            self.resolved
                .clone()
                .ok_or(RefResolutionError::UnresolvedReference {
                    target: reference.target.clone(),
                })
        }
    }

    fn text_arg(text: &str) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
            text.encode_utf16().collect(),
        )))
    }

    fn ref_arg(target: &str) -> CallArgValue {
        CallArgValue::Reference(ReferenceLike {
            kind: crate::value::ReferenceKind::A1,
            target: target.to_string(),
        })
    }

    #[test]
    fn eval_cell_address_returns_absolute_a1() {
        let got = eval_cell_surface(
            &[text_arg("address"), ref_arg("B3")],
            &MockResolver { resolved: None },
            None,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Text(ExcelText::from_utf16_code_units(
                "$B$3".encode_utf16().collect(),
            )))
        );
    }

    #[test]
    fn eval_cell_contents_uses_resolver() {
        let got = eval_cell_surface(
            &[text_arg("contents"), ref_arg("A1")],
            &MockResolver {
                resolved: Some(EvalValue::Number(7.0)),
            },
            None,
        );
        assert_eq!(got, Ok(EvalValue::Number(7.0)));
    }

    #[test]
    fn eval_cell_type_returns_text_marker() {
        let got = eval_cell_surface(
            &[text_arg("type"), ref_arg("A1")],
            &MockResolver {
                resolved: Some(EvalValue::Text(ExcelText::from_utf16_code_units(
                    "x".encode_utf16().collect(),
                ))),
            },
            None,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Text(ExcelText::from_utf16_code_units(
                "l".encode_utf16().collect(),
            )))
        );
    }

    #[test]
    fn eval_cell_filename_uses_host_provider() {
        let got = eval_cell_surface(
            &[text_arg("filename"), ref_arg("A1")],
            &MockResolver { resolved: None },
            Some(&MockHostInfoProvider {
                result: EvalValue::Text(ExcelText::from_utf16_code_units(
                    "[Book1]Sheet1".encode_utf16().collect(),
                )),
            }),
        );
        assert_eq!(
            got,
            Ok(EvalValue::Text(ExcelText::from_utf16_code_units(
                "[Book1]Sheet1".encode_utf16().collect(),
            )))
        );
    }

    #[test]
    fn eval_cell_parentheses_uses_host_provider() {
        let got = eval_cell_surface(
            &[text_arg("parentheses"), ref_arg("A1")],
            &MockResolver { resolved: None },
            Some(&MockHostInfoProvider {
                result: EvalValue::Number(1.0),
            }),
        );
        assert_eq!(got, Ok(EvalValue::Number(1.0)));
    }

    #[test]
    fn eval_cell_omitted_reference_uses_host_provider() {
        let got = eval_cell_surface(
            &[text_arg("row")],
            &MockResolver { resolved: None },
            Some(&MockHostInfoProvider {
                result: EvalValue::Number(7.0),
            }),
        );
        assert_eq!(got, Ok(EvalValue::Number(7.0)));
    }
}

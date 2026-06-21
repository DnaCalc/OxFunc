use crate::coercion::CoercionError;
use crate::function::{
    Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile, FunctionMeta,
    HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{coerce_prepared_to_text, prepare_arg_values_only};
use crate::host_info::{HostInfoError, HostInfoProvider, InfoQuery};
use crate::resolver::ReferenceSystemProvider;
use crate::value::CalcValue;
use crate::value::WorksheetErrorCode;

pub const INFO_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.INFO",
    arity: Arity::exact(1),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::VolatileContextual,
    host_interaction: HostInteractionClass::WorkbookState,
    thread_safety: ThreadSafetyClass::HostSerialized,
    arg_preparation_profile: FunctionMeta::DEFAULT_ARG_PREPARATION_PROFILE,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    lift_broadcast_profile: FunctionMeta::DEFAULT_LIFT_BROADCAST_PROFILE,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::Composite,
    surface_fec_dependency_profile: FecDependencyProfile::Composite,
    real_result_policy: FunctionMeta::DEFAULT_REAL_RESULT_POLICY,
    error_collapse_profile: FunctionMeta::DEFAULT_ERROR_COLLAPSE_PROFILE,
    precision_rounding_profile: FunctionMeta::DEFAULT_PRECISION_ROUNDING_PROFILE,
};

#[derive(Debug, Clone, PartialEq)]
pub enum InfoEvalError {
    ArityMismatch { expected: usize, actual: usize },
    TypeTextCoercion(CoercionError),
    UnsupportedTypeText(String),
    HostInfoProviderMissing(InfoQuery),
    HostInfo(HostInfoError),
}

fn parse_info_query(
    arg: &CalcValue,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<InfoQuery, InfoEvalError> {
    let prepared =
        prepare_arg_values_only(arg, resolver).map_err(InfoEvalError::TypeTextCoercion)?;
    let info = coerce_prepared_to_text(&prepared)
        .map_err(InfoEvalError::TypeTextCoercion)?
        .to_string_lossy()
        .trim()
        .to_ascii_lowercase();

    match info.as_str() {
        "directory" => Ok(InfoQuery::Directory),
        "numfile" => Ok(InfoQuery::NumFile),
        "origin" => Ok(InfoQuery::Origin),
        "osversion" => Ok(InfoQuery::OsVersion),
        "recalc" => Ok(InfoQuery::Recalc),
        "release" => Ok(InfoQuery::Release),
        "system" => Ok(InfoQuery::System),
        "memavail" => Ok(InfoQuery::MemAvail),
        "memused" => Ok(InfoQuery::MemUsed),
        "totmem" => Ok(InfoQuery::TotMem),
        _ => Err(InfoEvalError::UnsupportedTypeText(info)),
    }
}

pub fn eval_info_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
    host_info: Option<&dyn HostInfoProvider>,
) -> Result<CalcValue, InfoEvalError> {
    if !INFO_META.arity.accepts(args.len()) {
        return Err(InfoEvalError::ArityMismatch {
            expected: INFO_META.arity.min,
            actual: args.len(),
        });
    }

    let query = parse_info_query(&args[0], resolver)?;
    let provider = host_info.ok_or(InfoEvalError::HostInfoProviderMissing(query))?;
    provider.query_info(query).map_err(InfoEvalError::HostInfo)
}

pub fn map_info_error_to_ws(e: &InfoEvalError) -> WorksheetErrorCode {
    match e {
        InfoEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        InfoEvalError::TypeTextCoercion(CoercionError::WorksheetError(code)) => *code,
        InfoEvalError::UnsupportedTypeText(_) => WorksheetErrorCode::Value,
        InfoEvalError::HostInfoProviderMissing(_) => WorksheetErrorCode::Value,
        InfoEvalError::HostInfo(_) => WorksheetErrorCode::Value,
        InfoEvalError::TypeTextCoercion(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::ReferenceSystemCapabilities;
    use crate::value::ExcelText;

    struct MockResolver;

    impl ReferenceSystemProvider for MockResolver {
        fn capabilities(&self) -> ReferenceSystemCapabilities {
            ReferenceSystemCapabilities::permissive_local()
        }

        fn dereference(
            &self,
            request: &crate::resolver::ReferenceDereferenceRequest,
        ) -> Result<CalcValue, crate::resolver::ReferenceResolutionError> {
            let reference = &request.reference;
            Err(
                crate::resolver::ReferenceResolutionError::UnresolvedReference {
                    target: reference.target().to_string(),
                },
            )
        }
    }

    struct MockProvider {
        result: CalcValue,
    }

    impl HostInfoProvider for MockProvider {
        fn query_info(&self, query: InfoQuery) -> Result<CalcValue, HostInfoError> {
            match query {
                InfoQuery::Release => Ok(self.result.clone()),
                other => Err(HostInfoError::UnsupportedInfoQuery(other)),
            }
        }
    }

    fn text_arg(text: &str) -> CalcValue {
        CalcValue::text(ExcelText::from_utf16_code_units(
            text.encode_utf16().collect(),
        ))
    }

    #[test]
    fn eval_info_release_uses_provider() {
        let got = eval_info_surface(
            &[text_arg("release")],
            &MockResolver,
            Some(&MockProvider {
                result: CalcValue::text(ExcelText::from_utf16_code_units(
                    "16.0".encode_utf16().collect(),
                )),
            }),
        );
        assert_eq!(
            got,
            Ok(CalcValue::text(ExcelText::from_utf16_code_units(
                "16.0".encode_utf16().collect(),
            )))
        );
    }

    #[test]
    fn eval_info_requires_provider() {
        let got = eval_info_surface(&[text_arg("release")], &MockResolver, None);
        assert_eq!(
            got,
            Err(InfoEvalError::HostInfoProviderMissing(InfoQuery::Release))
        );
    }
}

use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{PreparedArgValue, prepare_arg_values_only};
use crate::functions::xmatch::{XmatchEvalError, comparable_eq, prepared_lookup_comparable};
use crate::host_info::{CellInfoQuery, HostInfoError, HostInfoProvider};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, ReferenceLike, WorksheetErrorCode};

pub const SWITCH_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.SWITCH",
    arity: Arity { min: 3, max: 255 },
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::RefsVisibleInAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
};

pub const ISFORMULA_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.ISFORMULA",
    arity: Arity::exact(1),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::WorkbookState,
    thread_safety: ThreadSafetyClass::HostSerialized,
    arg_preparation_profile: ArgPreparationProfile::RefsVisibleInAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::Composite,
    surface_fec_dependency_profile: FecDependencyProfile::Composite,
};

#[derive(Debug, Clone, PartialEq)]
pub enum MiscSwitchInfoEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
    HostInfo(HostInfoError),
    InvalidOperand,
}

fn prepared_to_eval(arg: PreparedArgValue) -> EvalValue {
    match arg {
        PreparedArgValue::Eval(value) => value,
        PreparedArgValue::MissingArg => EvalValue::Error(WorksheetErrorCode::NA),
        PreparedArgValue::EmptyCell => EvalValue::Number(0.0),
    }
}

fn map_xmatch_coercion(err: XmatchEvalError) -> MiscSwitchInfoEvalError {
    match err {
        XmatchEvalError::Coercion(inner) => MiscSwitchInfoEvalError::Coercion(inner),
        XmatchEvalError::MissingArg | XmatchEvalError::EmptyCell => {
            MiscSwitchInfoEvalError::InvalidOperand
        }
        XmatchEvalError::UnsupportedValueKind(_) => MiscSwitchInfoEvalError::InvalidOperand,
        XmatchEvalError::ArityMismatch { .. }
        | XmatchEvalError::InvalidMatchMode(_)
        | XmatchEvalError::InvalidSearchMode(_)
        | XmatchEvalError::UnsupportedMatchModeForSeed(_)
        | XmatchEvalError::UnsupportedSearchModeForSeed(_)
        | XmatchEvalError::NotAvailable
        | XmatchEvalError::EmptyLookupArray => MiscSwitchInfoEvalError::InvalidOperand,
    }
}

fn eval_switch_expression(
    arg: &CallArgValue,
    resolver: &impl ReferenceResolver,
) -> Result<PreparedArgValue, MiscSwitchInfoEvalError> {
    prepare_arg_values_only(arg, resolver).map_err(MiscSwitchInfoEvalError::Coercion)
}

fn switch_values_equal(
    lhs: &PreparedArgValue,
    rhs: &PreparedArgValue,
) -> Result<bool, MiscSwitchInfoEvalError> {
    match (lhs, rhs) {
        (PreparedArgValue::EmptyCell, PreparedArgValue::EmptyCell) => Ok(true),
        (PreparedArgValue::MissingArg, PreparedArgValue::MissingArg) => Ok(true),
        (PreparedArgValue::EmptyCell, _) | (_, PreparedArgValue::EmptyCell) => Ok(false),
        (PreparedArgValue::MissingArg, _) | (_, PreparedArgValue::MissingArg) => Ok(false),
        _ => {
            let lhs = prepared_lookup_comparable(lhs).map_err(map_xmatch_coercion)?;
            let rhs = prepared_lookup_comparable(rhs).map_err(map_xmatch_coercion)?;
            Ok(comparable_eq(&lhs, &rhs))
        }
    }
}

pub fn eval_switch_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, MiscSwitchInfoEvalError> {
    if !SWITCH_META.arity.accepts(args.len()) {
        return Err(MiscSwitchInfoEvalError::ArityMismatch {
            expected_min: SWITCH_META.arity.min,
            expected_max: SWITCH_META.arity.max,
            actual: args.len(),
        });
    }

    let expression = eval_switch_expression(&args[0], resolver)?;
    let pair_count = (args.len() - 1) / 2;
    let has_default = (args.len() - 1) % 2 == 1;

    for pair_idx in 0..pair_count {
        let candidate_idx = 1 + pair_idx * 2;
        let result_idx = candidate_idx + 1;
        let candidate = eval_switch_expression(&args[candidate_idx], resolver)?;
        if switch_values_equal(&expression, &candidate)? {
            let selected = prepare_arg_values_only(&args[result_idx], resolver)
                .map_err(MiscSwitchInfoEvalError::Coercion)?;
            return Ok(prepared_to_eval(selected));
        }
    }

    if has_default {
        let default_idx = args.len() - 1;
        let selected = prepare_arg_values_only(&args[default_idx], resolver)
            .map_err(MiscSwitchInfoEvalError::Coercion)?;
        return Ok(prepared_to_eval(selected));
    }

    Ok(EvalValue::Error(WorksheetErrorCode::NA))
}

fn isformula_reference_from_arg(arg: &CallArgValue) -> Option<ReferenceLike> {
    match arg {
        CallArgValue::Reference(reference) => Some(reference.clone()),
        CallArgValue::Eval(EvalValue::Reference(reference)) => Some(reference.clone()),
        _ => None,
    }
}

pub fn eval_isformula_surface(
    args: &[CallArgValue],
    host_info: Option<&dyn HostInfoProvider>,
) -> Result<EvalValue, MiscSwitchInfoEvalError> {
    if !ISFORMULA_META.arity.accepts(args.len()) {
        return Err(MiscSwitchInfoEvalError::ArityMismatch {
            expected_min: ISFORMULA_META.arity.min,
            expected_max: ISFORMULA_META.arity.max,
            actual: args.len(),
        });
    }

    let reference =
        isformula_reference_from_arg(&args[0]).ok_or(MiscSwitchInfoEvalError::InvalidOperand)?;
    let provider = host_info.ok_or_else(|| {
        MiscSwitchInfoEvalError::HostInfo(HostInfoError::UnsupportedCellInfoQuery(
            CellInfoQuery::IsFormula,
        ))
    })?;
    provider
        .query_cell_info(CellInfoQuery::IsFormula, Some(&reference))
        .map_err(MiscSwitchInfoEvalError::HostInfo)
}

pub fn map_misc_switch_info_error_to_ws(e: &MiscSwitchInfoEvalError) -> WorksheetErrorCode {
    match e {
        MiscSwitchInfoEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        MiscSwitchInfoEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        MiscSwitchInfoEvalError::HostInfo(HostInfoError::ProviderFailure { .. }) => {
            WorksheetErrorCode::Value
        }
        MiscSwitchInfoEvalError::HostInfo(HostInfoError::UnsupportedCellInfoQuery(_)) => {
            WorksheetErrorCode::Value
        }
        MiscSwitchInfoEvalError::HostInfo(HostInfoError::UnsupportedInfoQuery(_)) => {
            WorksheetErrorCode::Value
        }
        MiscSwitchInfoEvalError::HostInfo(HostInfoError::UnsupportedFormulaTextQuery) => {
            WorksheetErrorCode::Value
        }
        MiscSwitchInfoEvalError::HostInfo(HostInfoError::UnsupportedSheetIndexQuery) => {
            WorksheetErrorCode::Value
        }
        MiscSwitchInfoEvalError::HostInfo(HostInfoError::UnsupportedSheetCountQuery) => {
            WorksheetErrorCode::Value
        }
        MiscSwitchInfoEvalError::InvalidOperand => WorksheetErrorCode::Value,
        MiscSwitchInfoEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::{RefResolutionError, ResolverCapabilities};
    use crate::value::{ExcelText, ReferenceKind};

    struct NoResolver;

    impl ReferenceResolver for NoResolver {
        fn capabilities(&self) -> ResolverCapabilities {
            ResolverCapabilities::permissive_local()
        }

        fn resolve_reference(
            &self,
            reference: &ReferenceLike,
        ) -> Result<EvalValue, RefResolutionError> {
            Err(RefResolutionError::UnresolvedReference {
                target: reference.target.clone(),
            })
        }
    }

    struct MockHostInfoProvider;

    impl HostInfoProvider for MockHostInfoProvider {
        fn query_cell_info(
            &self,
            query: CellInfoQuery,
            reference: Option<&ReferenceLike>,
        ) -> Result<EvalValue, HostInfoError> {
            assert_eq!(query, CellInfoQuery::IsFormula);
            let target = reference.expect("reference required").target.as_str();
            Ok(EvalValue::Logical(matches!(target, "A1" | "A1:A2")))
        }

        fn query_info(
            &self,
            query: crate::host_info::InfoQuery,
        ) -> Result<EvalValue, HostInfoError> {
            Err(HostInfoError::UnsupportedInfoQuery(query))
        }
    }

    #[test]
    fn switch_matches_case_insensitive_text_and_is_lazy() {
        let got = eval_switch_surface(
            &[
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
                    "A".encode_utf16().collect(),
                ))),
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
                    "a".encode_utf16().collect(),
                ))),
                CallArgValue::Eval(EvalValue::Number(1.0)),
                CallArgValue::Eval(EvalValue::Number(2.0)),
                CallArgValue::Eval(EvalValue::Error(WorksheetErrorCode::Div0)),
            ],
            &NoResolver,
        );
        assert_eq!(got, Ok(EvalValue::Number(1.0)));
    }

    #[test]
    fn switch_returns_default_or_na() {
        let with_default = eval_switch_surface(
            &[
                CallArgValue::Eval(EvalValue::Number(3.0)),
                CallArgValue::Eval(EvalValue::Number(1.0)),
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
                    "a".encode_utf16().collect(),
                ))),
                CallArgValue::Eval(EvalValue::Number(2.0)),
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
                    "b".encode_utf16().collect(),
                ))),
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
                    "other".encode_utf16().collect(),
                ))),
            ],
            &NoResolver,
        );
        assert_eq!(
            with_default,
            Ok(EvalValue::Text(ExcelText::from_utf16_code_units(
                "other".encode_utf16().collect(),
            )))
        );

        let no_default = eval_switch_surface(
            &[
                CallArgValue::Eval(EvalValue::Number(3.0)),
                CallArgValue::Eval(EvalValue::Number(1.0)),
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
                    "a".encode_utf16().collect(),
                ))),
                CallArgValue::Eval(EvalValue::Number(2.0)),
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
                    "b".encode_utf16().collect(),
                ))),
            ],
            &NoResolver,
        );
        assert_eq!(no_default, Ok(EvalValue::Error(WorksheetErrorCode::NA)));
    }

    #[test]
    fn isformula_uses_host_query_on_reference_only() {
        let provider = MockHostInfoProvider;
        let got = eval_isformula_surface(
            &[CallArgValue::Reference(ReferenceLike {
                kind: ReferenceKind::Area,
                target: "A1:A2".to_string(),
            })],
            Some(&provider),
        );
        assert_eq!(got, Ok(EvalValue::Logical(true)));

        let scalar = eval_isformula_surface(
            &[CallArgValue::Eval(EvalValue::Number(1.0))],
            Some(&provider),
        );
        assert_eq!(scalar, Err(MiscSwitchInfoEvalError::InvalidOperand));
    }
}

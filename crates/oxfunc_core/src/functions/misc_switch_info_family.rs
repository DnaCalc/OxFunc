use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::prepare_arg_values_only;
use crate::functions::excel_numeric_compare::excel_numbers_equal;
use crate::functions::xmatch::{XmatchEvalError, comparable_eq, prepared_lookup_comparable};
use crate::host_info::{CellInfoQuery, HostInfoError, HostInfoProvider};
use crate::resolver::ReferenceSystemProvider;
use crate::value::{CalcValue, CoreValue, ReferenceLike, WorksheetErrorCode};

pub const SWITCH_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.SWITCH",
    arity: Arity { min: 3, max: 255 },
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::RefsVisibleInAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    // SWITCH broadcasts its expression and the first value/result pair (`[0,1,3]`) over an
    // array. Verified live Excel 16.0 build 20026.
    lift_broadcast_profile: FunctionMeta::lift_at(&[0, 1, 3]),
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
    real_result_policy: FunctionMeta::DEFAULT_REAL_RESULT_POLICY,
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
    lift_broadcast_profile: FunctionMeta::DEFAULT_LIFT_BROADCAST_PROFILE,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::Composite,
    surface_fec_dependency_profile: FecDependencyProfile::Composite,
    real_result_policy: FunctionMeta::DEFAULT_REAL_RESULT_POLICY,
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

fn prepared_to_eval(arg: CalcValue) -> CalcValue {
    match arg.core() {
        CoreValue::Missing => CalcValue::error(WorksheetErrorCode::NA),
        CoreValue::Empty => CalcValue::number(0.0),
        _ => arg,
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
    arg: &CalcValue,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, MiscSwitchInfoEvalError> {
    prepare_arg_values_only(arg, resolver).map_err(MiscSwitchInfoEvalError::Coercion)
}

fn switch_values_equal(lhs: &CalcValue, rhs: &CalcValue) -> Result<bool, MiscSwitchInfoEvalError> {
    match (lhs.core(), rhs.core()) {
        (CoreValue::Empty, CoreValue::Empty) => Ok(true),
        (CoreValue::Missing, CoreValue::Missing) => Ok(true),
        (CoreValue::Empty, _) | (_, CoreValue::Empty) => Ok(false),
        (CoreValue::Missing, _) | (_, CoreValue::Missing) => Ok(false),
        _ => {
            let lhs = prepared_lookup_comparable(lhs).map_err(map_xmatch_coercion)?;
            let rhs = prepared_lookup_comparable(rhs).map_err(map_xmatch_coercion)?;
            Ok(match (&lhs, &rhs) {
                (
                    crate::functions::xmatch::XmatchComparable::Number(lhs),
                    crate::functions::xmatch::XmatchComparable::Number(rhs),
                ) => excel_numbers_equal(*lhs, *rhs),
                _ => comparable_eq(&lhs, &rhs),
            })
        }
    }
}

pub fn eval_switch_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, MiscSwitchInfoEvalError> {
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

    Ok(CalcValue::error(WorksheetErrorCode::NA))
}

fn isformula_reference_from_arg(arg: &CalcValue) -> Option<ReferenceLike> {
    arg.as_reference().cloned()
}

pub fn eval_isformula_surface(
    args: &[CalcValue],
    host_info: Option<&dyn HostInfoProvider>,
) -> Result<CalcValue, MiscSwitchInfoEvalError> {
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
        MiscSwitchInfoEvalError::HostInfo(
            HostInfoError::ProviderFailure { .. }
            | HostInfoError::UnsupportedCellInfoQuery(_)
            | HostInfoError::UnsupportedInfoQuery(_)
            | HostInfoError::UnsupportedFormulaTextQuery
            | HostInfoError::UnsupportedSheetIndexQuery
            | HostInfoError::UnsupportedSheetCountQuery
            | HostInfoError::UnsupportedAggregateReferenceContextQuery
            | HostInfoError::UnsupportedWidthConversionProfileQuery(_)
            | HostInfoError::UnsupportedImageQuery
            | HostInfoError::UnsupportedTranslateQuery,
        ) => WorksheetErrorCode::Value,
        MiscSwitchInfoEvalError::InvalidOperand => WorksheetErrorCode::Value,
        MiscSwitchInfoEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::ReferenceSystemCapabilities;
    use crate::value::{ExcelText, ReferenceKind};

    struct NoResolver;

    impl ReferenceSystemProvider for NoResolver {
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

    struct MockHostInfoProvider;

    impl HostInfoProvider for MockHostInfoProvider {
        fn query_cell_info(
            &self,
            query: CellInfoQuery,
            reference: Option<&ReferenceLike>,
        ) -> Result<CalcValue, HostInfoError> {
            assert_eq!(query, CellInfoQuery::IsFormula);
            let target = reference.expect("reference required").target();
            Ok(CalcValue::logical(matches!(target, "A1" | "A1:A2")))
        }

        fn query_info(
            &self,
            query: crate::host_info::InfoQuery,
        ) -> Result<CalcValue, HostInfoError> {
            Err(HostInfoError::UnsupportedInfoQuery(query))
        }
    }

    #[test]
    fn switch_matches_case_insensitive_text_and_is_lazy() {
        let got = eval_switch_surface(
            &[
                (CalcValue::text(ExcelText::from_utf16_code_units(
                    "A".encode_utf16().collect(),
                ))),
                (CalcValue::text(ExcelText::from_utf16_code_units(
                    "a".encode_utf16().collect(),
                ))),
                (CalcValue::number(1.0)),
                (CalcValue::number(2.0)),
                (CalcValue::error(WorksheetErrorCode::Div0)),
            ],
            &NoResolver,
        );
        assert_eq!(got, Ok(CalcValue::number(1.0)));
    }

    #[test]
    fn switch_returns_default_or_na() {
        let with_default = eval_switch_surface(
            &[
                (CalcValue::number(3.0)),
                (CalcValue::number(1.0)),
                (CalcValue::text(ExcelText::from_utf16_code_units(
                    "a".encode_utf16().collect(),
                ))),
                (CalcValue::number(2.0)),
                (CalcValue::text(ExcelText::from_utf16_code_units(
                    "b".encode_utf16().collect(),
                ))),
                (CalcValue::text(ExcelText::from_utf16_code_units(
                    "other".encode_utf16().collect(),
                ))),
            ],
            &NoResolver,
        );
        assert_eq!(
            with_default,
            Ok(CalcValue::text(ExcelText::from_utf16_code_units(
                "other".encode_utf16().collect(),
            )))
        );

        let no_default = eval_switch_surface(
            &[
                (CalcValue::number(3.0)),
                (CalcValue::number(1.0)),
                (CalcValue::text(ExcelText::from_utf16_code_units(
                    "a".encode_utf16().collect(),
                ))),
                (CalcValue::number(2.0)),
                (CalcValue::text(ExcelText::from_utf16_code_units(
                    "b".encode_utf16().collect(),
                ))),
            ],
            &NoResolver,
        );
        assert_eq!(no_default, Ok(CalcValue::error(WorksheetErrorCode::NA)));
    }

    #[test]
    fn switch_uses_excel_near_equal_numeric_equality() {
        let near_equal = eval_switch_surface(
            &[
                (CalcValue::number(5.0 + 2.0e-15)),
                (CalcValue::number(5.0)),
                (CalcValue::number(1.0)),
                (CalcValue::number(2.0)),
            ],
            &NoResolver,
        );
        assert_eq!(near_equal, Ok(CalcValue::number(1.0)));

        let far_equal = eval_switch_surface(
            &[
                (CalcValue::number(1.0 + 1.0e-14)),
                (CalcValue::number(1.0)),
                (CalcValue::number(1.0)),
                (CalcValue::number(2.0)),
            ],
            &NoResolver,
        );
        assert_eq!(far_equal, Ok(CalcValue::number(2.0)));

        let boundary_equal = eval_switch_surface(
            &[
                (CalcValue::number(((123_456_789_012_345_f64 * 10.0) + 5.0) / 1.0e25)),
                (CalcValue::number(((123_456_789_012_345_f64 * 10.0) + 4.0) / 1.0e25)),
                (CalcValue::number(1.0)),
                (CalcValue::number(2.0)),
            ],
            &NoResolver,
        );
        assert_eq!(boundary_equal, Ok(CalcValue::number(1.0)));
    }

    #[test]
    fn isformula_uses_host_query_on_reference_only() {
        let provider = MockHostInfoProvider;
        let got = eval_isformula_surface(
            &[CalcValue::reference(ReferenceLike::new(
                ReferenceKind::Area,
                "A1:A2".to_string(),
            ))],
            Some(&provider),
        );
        assert_eq!(got, Ok(CalcValue::logical(true)));

        let scalar = eval_isformula_surface(&[(CalcValue::number(1.0))], Some(&provider));
        assert_eq!(scalar, Err(MiscSwitchInfoEvalError::InvalidOperand));
    }
}

use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::covariance_p_fn::{eval_covariance_p_surface, map_covariance_p_error_to_ws};
use crate::functions::mode_sngl_fn::{eval_mode_sngl_surface, map_mode_sngl_error_to_ws};
use crate::functions::normal_log_family::{eval_lognorm_inv_surface, map_normal_log_error_to_ws};
use crate::functions::percentile_inc_fn::{
    eval_percentile_inc_surface, map_percentile_inc_error_to_ws,
};
use crate::functions::percentrank_inc_fn::{
    eval_percentrank_inc_surface, map_percentrank_inc_error_to_ws,
};
use crate::functions::quartile_inc_fn::{eval_quartile_inc_surface, map_quartile_inc_error_to_ws};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

const LEGACY_ALIAS_BASE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.LEGACY_STATS_ALIAS_BASE",
    arity: Arity::exact(1),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::RefsVisibleInAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::RefOnly,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
};

pub const COVAR_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.COVAR",
    arity: Arity::exact(2),
    ..LEGACY_ALIAS_BASE_META
};
pub const MODE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.MODE",
    arity: Arity { min: 1, max: 255 },
    ..LEGACY_ALIAS_BASE_META
};
pub const PERCENTILE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.PERCENTILE",
    arity: Arity::exact(2),
    ..LEGACY_ALIAS_BASE_META
};
pub const PERCENTRANK_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.PERCENTRANK",
    arity: Arity { min: 2, max: 3 },
    ..LEGACY_ALIAS_BASE_META
};
pub const QUARTILE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.QUARTILE",
    arity: Arity::exact(2),
    ..LEGACY_ALIAS_BASE_META
};
pub const LOGINV_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.LOGINV",
    arity: Arity::exact(3),
    ..LEGACY_ALIAS_BASE_META
};

#[derive(Debug, Clone, PartialEq)]
pub enum LegacyStatsAliasEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Worksheet(WorksheetErrorCode),
}

fn guard_arity(
    meta: &FunctionMeta,
    args: &[CallArgValue],
) -> Result<(), LegacyStatsAliasEvalError> {
    if meta.arity.accepts(args.len()) {
        Ok(())
    } else {
        Err(LegacyStatsAliasEvalError::ArityMismatch {
            expected_min: meta.arity.min,
            expected_max: meta.arity.max,
            actual: args.len(),
        })
    }
}

pub fn eval_covar_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, LegacyStatsAliasEvalError> {
    guard_arity(&COVAR_META, args)?;
    eval_covariance_p_surface(args, resolver)
        .map_err(|e| LegacyStatsAliasEvalError::Worksheet(map_covariance_p_error_to_ws(&e)))
}

pub fn eval_mode_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, LegacyStatsAliasEvalError> {
    guard_arity(&MODE_META, args)?;
    eval_mode_sngl_surface(args, resolver)
        .map_err(|e| LegacyStatsAliasEvalError::Worksheet(map_mode_sngl_error_to_ws(&e)))
}

pub fn eval_percentile_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, LegacyStatsAliasEvalError> {
    guard_arity(&PERCENTILE_META, args)?;
    eval_percentile_inc_surface(args, resolver)
        .map_err(|e| LegacyStatsAliasEvalError::Worksheet(map_percentile_inc_error_to_ws(&e)))
}

pub fn eval_percentrank_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, LegacyStatsAliasEvalError> {
    guard_arity(&PERCENTRANK_META, args)?;
    eval_percentrank_inc_surface(args, resolver)
        .map_err(|e| LegacyStatsAliasEvalError::Worksheet(map_percentrank_inc_error_to_ws(&e)))
}

pub fn eval_quartile_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, LegacyStatsAliasEvalError> {
    guard_arity(&QUARTILE_META, args)?;
    eval_quartile_inc_surface(args, resolver)
        .map_err(|e| LegacyStatsAliasEvalError::Worksheet(map_quartile_inc_error_to_ws(&e)))
}

pub fn eval_loginv_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, LegacyStatsAliasEvalError> {
    guard_arity(&LOGINV_META, args)?;
    eval_lognorm_inv_surface(args, resolver)
        .map_err(|e| LegacyStatsAliasEvalError::Worksheet(map_normal_log_error_to_ws(&e)))
}

pub fn map_legacy_stats_alias_error_to_ws(error: &LegacyStatsAliasEvalError) -> WorksheetErrorCode {
    match error {
        LegacyStatsAliasEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        LegacyStatsAliasEvalError::Worksheet(code) => *code,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::{RefResolutionError, ReferenceResolver, ResolverCapabilities};
    use crate::value::{ArrayCellValue, EvalArray, ReferenceKind, ReferenceLike};
    use std::collections::HashMap;

    struct MockResolver {
        cells: HashMap<String, EvalValue>,
    }

    impl ReferenceResolver for MockResolver {
        fn capabilities(&self) -> ResolverCapabilities {
            ResolverCapabilities::permissive_local()
        }

        fn resolve_reference(
            &self,
            reference: &ReferenceLike,
        ) -> Result<EvalValue, RefResolutionError> {
            self.cells.get(&reference.target).cloned().ok_or_else(|| {
                RefResolutionError::UnresolvedReference {
                    target: reference.target.clone(),
                }
            })
        }
    }

    fn ref_arg(target: &str) -> CallArgValue {
        CallArgValue::Reference(ReferenceLike {
            kind: ReferenceKind::A1,
            target: target.to_string(),
        })
    }

    fn num(n: f64) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Number(n))
    }

    fn array(values: &[f64]) -> EvalValue {
        EvalValue::Array(
            EvalArray::from_rows(vec![
                values.iter().copied().map(ArrayCellValue::Number).collect(),
            ])
            .unwrap(),
        )
    }

    #[test]
    fn alias_metadata_matches_expected_shape() {
        assert_eq!(COVAR_META.arity, Arity::exact(2));
        assert_eq!(MODE_META.arity.min, 1);
        assert_eq!(MODE_META.arity.max, 255);
        assert_eq!(PERCENTRANK_META.arity.max, 3);
        assert_eq!(LOGINV_META.function_id, "FUNC.LOGINV");
    }

    #[test]
    fn covar_alias_matches_covariance_p() {
        let resolver = MockResolver {
            cells: HashMap::from([
                ("A1:A3".to_string(), array(&[2.0, 4.0, 8.0])),
                ("B1:B3".to_string(), array(&[1.0, 3.0, 9.0])),
            ]),
        };
        let got = eval_covar_surface(&[ref_arg("A1:A3"), ref_arg("B1:B3")], &resolver);
        let modern = eval_covariance_p_surface(&[ref_arg("A1:A3"), ref_arg("B1:B3")], &resolver)
            .map_err(|e| LegacyStatsAliasEvalError::Worksheet(map_covariance_p_error_to_ws(&e)));
        assert_eq!(got, modern);
    }

    #[test]
    fn mode_percentile_percentrank_and_quartile_aliases_follow_modern_kernels() {
        let resolver = MockResolver {
            cells: HashMap::from([("A1:A5".to_string(), array(&[1.0, 2.0, 2.0, 4.0, 5.0]))]),
        };
        assert_eq!(
            eval_mode_surface(&[ref_arg("A1:A5")], &resolver),
            eval_mode_sngl_surface(&[ref_arg("A1:A5")], &resolver)
                .map_err(|e| LegacyStatsAliasEvalError::Worksheet(map_mode_sngl_error_to_ws(&e)))
        );
        assert_eq!(
            eval_percentile_surface(&[ref_arg("A1:A5"), num(0.3)], &resolver),
            eval_percentile_inc_surface(&[ref_arg("A1:A5"), num(0.3)], &resolver).map_err(|e| {
                LegacyStatsAliasEvalError::Worksheet(map_percentile_inc_error_to_ws(&e))
            })
        );
        assert_eq!(
            eval_percentrank_surface(&[ref_arg("A1:A5"), num(3.5)], &resolver),
            eval_percentrank_inc_surface(&[ref_arg("A1:A5"), num(3.5)], &resolver).map_err(|e| {
                LegacyStatsAliasEvalError::Worksheet(map_percentrank_inc_error_to_ws(&e))
            })
        );
        assert_eq!(
            eval_quartile_surface(&[ref_arg("A1:A5"), num(4.0)], &resolver),
            eval_quartile_inc_surface(&[ref_arg("A1:A5"), num(4.0)], &resolver).map_err(|e| {
                LegacyStatsAliasEvalError::Worksheet(map_quartile_inc_error_to_ws(&e))
            })
        );
    }

    #[test]
    fn loginv_alias_matches_lognorm_inv_seed_row() {
        let resolver = MockResolver {
            cells: HashMap::new(),
        };
        assert_eq!(
            eval_loginv_surface(&[num(0.75), num(1.5), num(0.4)], &resolver),
            eval_lognorm_inv_surface(&[num(0.75), num(1.5), num(0.4)], &resolver)
                .map_err(|e| LegacyStatsAliasEvalError::Worksheet(map_normal_log_error_to_ws(&e)))
        );
    }

    #[test]
    fn alias_error_mapping_passes_through_worksheet_codes() {
        let err = LegacyStatsAliasEvalError::Worksheet(WorksheetErrorCode::NA);
        assert_eq!(
            map_legacy_stats_alias_error_to_ws(&err),
            WorksheetErrorCode::NA
        );
        assert_eq!(
            map_legacy_stats_alias_error_to_ws(&LegacyStatsAliasEvalError::ArityMismatch {
                expected_min: 2,
                expected_max: 2,
                actual: 1,
            }),
            WorksheetErrorCode::Value
        );
    }

    #[test]
    fn alias_wrappers_preserve_underlying_arity_failures() {
        let resolver = MockResolver {
            cells: HashMap::new(),
        };
        assert_eq!(
            eval_covar_surface(&[num(1.0)], &resolver),
            Err(LegacyStatsAliasEvalError::ArityMismatch {
                expected_min: 2,
                expected_max: 2,
                actual: 1,
            })
        );
        assert_eq!(
            eval_loginv_surface(&[num(0.5), num(1.0)], &resolver),
            Err(LegacyStatsAliasEvalError::ArityMismatch {
                expected_min: 3,
                expected_max: 3,
                actual: 2,
            })
        );
    }
}

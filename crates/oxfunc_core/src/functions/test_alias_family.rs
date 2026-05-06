use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::confidence_test_family::{
    eval_z_test_surface, map_confidence_test_error_to_ws,
};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

const TEST_ALIAS_BASE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.TEST_ALIAS_BASE",
    arity: Arity::exact(2),
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

pub const CHISQ_TEST_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.CHISQ.TEST",
    arity: Arity::exact(2),
    ..TEST_ALIAS_BASE_META
};

pub const CHITEST_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.CHITEST",
    arity: Arity::exact(2),
    ..TEST_ALIAS_BASE_META
};

pub const F_TEST_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.F.TEST",
    arity: Arity::exact(2),
    ..TEST_ALIAS_BASE_META
};

pub const FTEST_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.FTEST",
    arity: Arity::exact(2),
    ..TEST_ALIAS_BASE_META
};

pub const T_TEST_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.T.TEST",
    arity: Arity::exact(4),
    ..TEST_ALIAS_BASE_META
};

pub const TTEST_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.TTEST",
    arity: Arity::exact(4),
    ..TEST_ALIAS_BASE_META
};

pub const ZTEST_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.ZTEST",
    arity: Arity { min: 2, max: 3 },
    ..TEST_ALIAS_BASE_META
};

#[derive(Debug, Clone, PartialEq)]
pub enum TestAliasEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Worksheet(WorksheetErrorCode),
    MissingModernTarget(&'static str),
}

fn guard_arity(meta: &FunctionMeta, args: &[CallArgValue]) -> Result<(), TestAliasEvalError> {
    if meta.arity.accepts(args.len()) {
        Ok(())
    } else {
        Err(TestAliasEvalError::ArityMismatch {
            expected_min: meta.arity.min,
            expected_max: meta.arity.max,
            actual: args.len(),
        })
    }
}

fn missing_target(function_id: &'static str) -> Result<EvalValue, TestAliasEvalError> {
    Err(TestAliasEvalError::MissingModernTarget(function_id))
}

pub fn eval_chisq_test_surface(
    args: &[CallArgValue],
    _resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, TestAliasEvalError> {
    guard_arity(&CHISQ_TEST_META, args)?;
    missing_target(CHISQ_TEST_META.function_id)
}

pub fn eval_chitest_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, TestAliasEvalError> {
    guard_arity(&CHITEST_META, args)?;
    eval_chisq_test_surface(args, resolver)
}

pub fn eval_f_test_surface(
    args: &[CallArgValue],
    _resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, TestAliasEvalError> {
    guard_arity(&F_TEST_META, args)?;
    missing_target(F_TEST_META.function_id)
}

pub fn eval_ftest_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, TestAliasEvalError> {
    guard_arity(&FTEST_META, args)?;
    eval_f_test_surface(args, resolver)
}

pub fn eval_t_test_surface(
    args: &[CallArgValue],
    _resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, TestAliasEvalError> {
    guard_arity(&T_TEST_META, args)?;
    missing_target(T_TEST_META.function_id)
}

pub fn eval_ttest_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, TestAliasEvalError> {
    guard_arity(&TTEST_META, args)?;
    eval_t_test_surface(args, resolver)
}

pub fn eval_ztest_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, TestAliasEvalError> {
    guard_arity(&ZTEST_META, args)?;
    eval_z_test_surface(args, resolver)
        .map_err(|e| TestAliasEvalError::Worksheet(map_confidence_test_error_to_ws(&e)))
}

pub fn map_test_alias_error_to_ws(error: &TestAliasEvalError) -> WorksheetErrorCode {
    match error {
        TestAliasEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        TestAliasEvalError::Worksheet(code) => *code,
        TestAliasEvalError::MissingModernTarget(_) => WorksheetErrorCode::Name,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::{RefResolutionError, ResolverCapabilities};
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
            kind: ReferenceKind::Area,
            target: target.to_string(),
        })
    }

    fn num(n: f64) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Number(n))
    }

    fn col(values: &[f64]) -> EvalValue {
        EvalValue::Array(
            EvalArray::from_rows(
                values
                    .iter()
                    .copied()
                    .map(|n| vec![ArrayCellValue::Number(n)])
                    .collect(),
            )
            .unwrap(),
        )
    }

    #[test]
    fn alias_metadata_shapes_are_frozen() {
        assert_eq!(CHISQ_TEST_META.arity, Arity::exact(2));
        assert_eq!(F_TEST_META.arity, Arity::exact(2));
        assert_eq!(T_TEST_META.arity, Arity::exact(4));
        assert_eq!(ZTEST_META.arity.min, 2);
        assert_eq!(ZTEST_META.arity.max, 3);
        assert_eq!(ZTEST_META.function_id, "FUNC.ZTEST");
    }

    #[test]
    fn ztest_alias_matches_modern_z_test_surface() {
        let resolver = MockResolver {
            cells: HashMap::from([("A1:A5".to_string(), col(&[3.0, 6.0, 7.0, 8.0, 6.0]))]),
        };
        let args = [ref_arg("A1:A5"), num(4.0), num(1.5)];
        assert_eq!(
            eval_ztest_surface(&args, &resolver),
            eval_z_test_surface(&args, &resolver)
                .map_err(|e| TestAliasEvalError::Worksheet(map_confidence_test_error_to_ws(&e)))
        );
    }

    #[test]
    fn chi_f_t_aliases_are_explicitly_open_without_modern_targets() {
        let resolver = MockResolver {
            cells: HashMap::new(),
        };
        assert_eq!(
            eval_chisq_test_surface(&[num(1.0), num(2.0)], &resolver),
            Err(TestAliasEvalError::MissingModernTarget("FUNC.CHISQ.TEST"))
        );
        assert_eq!(
            eval_f_test_surface(&[num(1.0), num(2.0)], &resolver),
            Err(TestAliasEvalError::MissingModernTarget("FUNC.F.TEST"))
        );
        assert_eq!(
            eval_t_test_surface(&[num(1.0), num(2.0), num(3.0), num(4.0)], &resolver),
            Err(TestAliasEvalError::MissingModernTarget("FUNC.T.TEST"))
        );
    }

    #[test]
    fn compatibility_aliases_follow_their_modern_entrypoints() {
        let resolver = MockResolver {
            cells: HashMap::new(),
        };
        assert_eq!(
            eval_chitest_surface(&[num(1.0), num(2.0)], &resolver),
            eval_chisq_test_surface(&[num(1.0), num(2.0)], &resolver)
        );
        assert_eq!(
            eval_ftest_surface(&[num(1.0), num(2.0)], &resolver),
            eval_f_test_surface(&[num(1.0), num(2.0)], &resolver)
        );
        assert_eq!(
            eval_ttest_surface(&[num(1.0), num(2.0), num(3.0), num(4.0)], &resolver),
            eval_t_test_surface(&[num(1.0), num(2.0), num(3.0), num(4.0)], &resolver)
        );
    }

    #[test]
    fn alias_error_mapping_is_explicit() {
        assert_eq!(
            map_test_alias_error_to_ws(&TestAliasEvalError::MissingModernTarget("FUNC.F.TEST")),
            WorksheetErrorCode::Name
        );
        assert_eq!(
            map_test_alias_error_to_ws(&TestAliasEvalError::ArityMismatch {
                expected_min: 2,
                expected_max: 2,
                actual: 1,
            }),
            WorksheetErrorCode::Value
        );
    }
}

use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    PreparedArgValue, coerce_prepared_to_number, run_values_only_prepared,
};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

pub const ISEVEN_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.ISEVEN",
    arity: Arity::exact(1),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
};

#[derive(Debug, Clone, PartialEq)]
pub enum IsEvenEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
}

fn coerce_iseven_number(arg: &PreparedArgValue) -> Result<f64, CoercionError> {
    match arg {
        PreparedArgValue::MissingArg | PreparedArgValue::EmptyCell => Ok(0.0),
        PreparedArgValue::Eval(EvalValue::Logical(_)) => {
            Err(CoercionError::UnsupportedValueKind("logical"))
        }
        _ => coerce_prepared_to_number(arg),
    }
}

pub fn iseven_kernel(n: f64) -> bool {
    (n.trunc() as i64).rem_euclid(2) == 0
}

pub fn eval_iseven_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, IsEvenEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| {
            if !ISEVEN_META.arity.accepts(prepared.len()) {
                return Err(IsEvenEvalError::ArityMismatch {
                    expected_min: ISEVEN_META.arity.min,
                    expected_max: ISEVEN_META.arity.max,
                    actual: prepared.len(),
                });
            }
            Ok(EvalValue::Logical(iseven_kernel(
                coerce_iseven_number(&prepared[0]).map_err(IsEvenEvalError::Coercion)?,
            )))
        },
        IsEvenEvalError::Coercion,
    )
}

pub fn map_iseven_error_to_ws(e: &IsEvenEvalError) -> WorksheetErrorCode {
    match e {
        IsEvenEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        IsEvenEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        IsEvenEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::{RefResolutionError, ResolverCapabilities};
    use crate::value::{ExcelText, ReferenceKind, ReferenceLike};

    struct MockResolver {
        resolved: Option<EvalValue>,
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

    fn txt(s: &str) -> ExcelText {
        ExcelText::from_utf16_code_units(s.encode_utf16().collect())
    }

    #[test]
    fn iseven_matches_probe_rows() {
        assert!(iseven_kernel(2.9));
        assert!(!iseven_kernel(-1.2));
        assert!(iseven_kernel(0.0));
    }

    #[test]
    fn eval_iseven_accepts_numeric_text_and_blank_reference() {
        assert_eq!(
            eval_iseven_surface(
                &[CallArgValue::Eval(EvalValue::Text(txt("2")))],
                &MockResolver { resolved: None },
            ),
            Ok(EvalValue::Logical(true))
        );
        assert_eq!(
            eval_iseven_surface(
                &[CallArgValue::Reference(ReferenceLike {
                    kind: ReferenceKind::A1,
                    target: "D1".to_string(),
                })],
                &MockResolver {
                    resolved: Some(EvalValue::Array(
                        crate::value::EvalArray::from_rows(vec![vec![
                            crate::value::ArrayCellValue::EmptyCell,
                        ]])
                        .unwrap(),
                    )),
                },
            ),
            Ok(EvalValue::Logical(true))
        );
    }

    #[test]
    fn eval_iseven_rejects_logicals_and_non_numeric_text() {
        assert!(matches!(
            eval_iseven_surface(
                &[CallArgValue::Eval(EvalValue::Logical(true))],
                &MockResolver { resolved: None },
            ),
            Err(IsEvenEvalError::Coercion(
                CoercionError::UnsupportedValueKind(_)
            )) | Err(IsEvenEvalError::Coercion(CoercionError::NonNumericText(_)))
        ));
        assert!(matches!(
            eval_iseven_surface(
                &[CallArgValue::Eval(EvalValue::Text(txt("x")))],
                &MockResolver { resolved: None },
            ),
            Err(IsEvenEvalError::Coercion(CoercionError::NonNumericText(_)))
        ));
    }
}

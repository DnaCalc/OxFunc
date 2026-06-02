use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    PreparedValue, coerce_prepared_to_number, run_values_only_prepared_lifted,
};
use crate::resolver::ReferenceSystemProvider;
use crate::value::{FunctionArg, FunctionValue, WorksheetErrorCode};

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

fn coerce_iseven_number(arg: &PreparedValue) -> Result<f64, CoercionError> {
    match arg {
        PreparedValue::MissingArg | PreparedValue::EmptyCell => Ok(0.0),
        PreparedValue::Eval(FunctionValue::Logical(_)) => {
            Err(CoercionError::UnsupportedValueKind("logical"))
        }
        _ => coerce_prepared_to_number(arg),
    }
}

pub fn iseven_kernel(n: f64) -> bool {
    (n.trunc() as i64).rem_euclid(2) == 0
}

pub fn eval_iseven_surface(
    args: &[FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<FunctionValue, IsEvenEvalError> {
    run_values_only_prepared_lifted(
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
            Ok(FunctionValue::Logical(iseven_kernel(
                coerce_iseven_number(&prepared[0]).map_err(IsEvenEvalError::Coercion)?,
            )))
        },
        map_iseven_error_to_ws,
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
    use crate::resolver::ReferenceSystemCapabilities;
    use crate::value::{ExcelText, ReferenceKind, ReferenceLike};

    struct MockResolver {
        resolved: Option<FunctionValue>,
    }

    impl ReferenceSystemProvider for MockResolver {
        fn capabilities(&self) -> ReferenceSystemCapabilities {
            ReferenceSystemCapabilities::permissive_local()
        }

        fn dereference(
            &self,
            request: &crate::resolver::ReferenceDereferenceRequest,
        ) -> Result<FunctionValue, crate::resolver::ReferenceResolutionError> {
            let reference = &request.reference;
            self.resolved.clone().ok_or(
                crate::resolver::ReferenceResolutionError::UnresolvedReference {
                    target: reference.target().to_string(),
                },
            )
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
                &[FunctionArg::Eval(FunctionValue::Text(txt("2")))],
                &MockResolver { resolved: None },
            ),
            Ok(FunctionValue::Logical(true))
        );
        assert_eq!(
            eval_iseven_surface(
                &[FunctionArg::Reference(ReferenceLike::new(
                    ReferenceKind::A1,
                    "D1".to_string()
                ))],
                &MockResolver {
                    resolved: Some(FunctionValue::Array(
                        crate::value::FunctionArray::from_rows(vec![vec![
                            crate::value::FunctionArrayCell::EmptyCell,
                        ]])
                        .unwrap(),
                    )),
                },
            ),
            Ok(FunctionValue::Logical(true))
        );
    }

    #[test]
    fn eval_iseven_rejects_logicals_and_non_numeric_text() {
        assert!(matches!(
            eval_iseven_surface(
                &[FunctionArg::Eval(FunctionValue::Logical(true))],
                &MockResolver { resolved: None },
            ),
            Err(IsEvenEvalError::Coercion(
                CoercionError::UnsupportedValueKind(_)
            )) | Err(IsEvenEvalError::Coercion(CoercionError::NonNumericText(_)))
        ));
        assert!(matches!(
            eval_iseven_surface(
                &[FunctionArg::Eval(FunctionValue::Text(txt("x")))],
                &MockResolver { resolved: None },
            ),
            Err(IsEvenEvalError::Coercion(CoercionError::NonNumericText(_)))
        ));
    }
}

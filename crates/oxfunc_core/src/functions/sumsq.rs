use crate::coercion::CoercionError;
use crate::function::{
    Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile, FunctionMeta,
    HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{AggregatePreparedItem, expand_aggregate_arg};
use crate::functions::aggregate_common::dual_policy_numeric_value;
use crate::resolver::ReferenceSystemProvider;
use crate::value::CalcValue;
use crate::value::WorksheetErrorCode;

pub const SUMSQ_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.SUMSQ",
    arity: Arity { min: 1, max: 255 },
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: FunctionMeta::DEFAULT_ARG_PREPARATION_PROFILE,
    coercion_lift_profile: CoercionLiftProfile::AggregateDirectAndRangeDualPolicy,
    lift_broadcast_profile: FunctionMeta::DEFAULT_LIFT_BROADCAST_PROFILE,
    kernel_signature_class: KernelSignatureClass::NumsToNum,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
    real_result_policy: FunctionMeta::DEFAULT_REAL_RESULT_POLICY,
};

#[derive(Debug, Clone, PartialEq)]
pub enum SumsqEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
}

fn eval_sumsq_aggregate(args: &[AggregatePreparedItem]) -> Result<CalcValue, SumsqEvalError> {
    let mut acc = 0.0;
    for arg in args {
        if let Some(value) = dual_policy_numeric_value(arg).map_err(SumsqEvalError::Coercion)? {
            acc += value * value;
        }
    }
    Ok(CalcValue::number(acc))
}

pub fn eval_sumsq_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, SumsqEvalError> {
    let argc = args.len();
    if !SUMSQ_META.arity.accepts(argc) {
        return Err(SumsqEvalError::ArityMismatch {
            expected_min: SUMSQ_META.arity.min,
            expected_max: SUMSQ_META.arity.max,
            actual: argc,
        });
    }

    let mut prepared = Vec::new();
    for arg in args {
        prepared.extend(expand_aggregate_arg(arg, resolver).map_err(SumsqEvalError::Coercion)?);
    }
    eval_sumsq_aggregate(&prepared)
}

pub fn map_sumsq_error_to_ws(e: &SumsqEvalError) -> WorksheetErrorCode {
    match e {
        SumsqEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        SumsqEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        SumsqEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::ReferenceSystemCapabilities;
    use crate::value::{CalcArray, ExcelText, ReferenceKind, ReferenceLike};

    struct MockResolver {
        resolved_value: Option<CalcValue>,
    }

    impl ReferenceSystemProvider for MockResolver {
        fn capabilities(&self) -> ReferenceSystemCapabilities {
            ReferenceSystemCapabilities::permissive_local()
        }

        fn dereference(
            &self,
            request: &crate::resolver::ReferenceDereferenceRequest,
        ) -> Result<CalcValue, crate::resolver::ReferenceResolutionError> {
            let reference = &request.reference;
            self.resolved_value.clone().ok_or(
                crate::resolver::ReferenceResolutionError::UnresolvedReference {
                    target: reference.target().to_string(),
                },
            )
        }
    }

    #[test]
    fn eval_sumsq_accumulates_direct_numbers() {
        let args = vec![
            (CalcValue::number(2.0)),
            (CalcValue::number(3.0)),
            (CalcValue::number(4.0)),
        ];
        let got = eval_sumsq_surface(
            &args,
            &MockResolver {
                resolved_value: None,
            },
        );
        assert_eq!(got, Ok(CalcValue::number(29.0)));
    }

    #[test]
    fn eval_sumsq_counts_direct_numeric_text_and_logical() {
        let args = vec![
            (CalcValue::logical(true)),
            (CalcValue::text(ExcelText::from_utf16_code_units(
                "2".encode_utf16().collect(),
            ))),
        ];
        let got = eval_sumsq_surface(
            &args,
            &MockResolver {
                resolved_value: None,
            },
        );
        assert_eq!(got, Ok(CalcValue::number(5.0)));
    }

    #[test]
    fn eval_sumsq_rejects_direct_non_numeric_text() {
        let args = vec![
            (CalcValue::text(ExcelText::from_utf16_code_units(
                "x".encode_utf16().collect(),
            ))),
        ];
        let got = eval_sumsq_surface(
            &args,
            &MockResolver {
                resolved_value: None,
            },
        );
        assert!(matches!(got, Err(SumsqEvalError::Coercion(_))));
    }

    #[test]
    fn eval_sumsq_ignores_reference_derived_text_and_logical() {
        let args = vec![CalcValue::reference(ReferenceLike::new(
            ReferenceKind::Area,
            "A1:A2".to_string(),
        ))];
        let got = eval_sumsq_surface(
            &args,
            &MockResolver {
                resolved_value: Some(CalcValue::array(
                    CalcArray::from_rows(vec![vec![
                        CalcValue::text(ExcelText::from_utf16_code_units(
                            "x".encode_utf16().collect(),
                        )),
                        CalcValue::logical(true),
                    ]])
                    .unwrap(),
                )),
            },
        );
        assert_eq!(got, Ok(CalcValue::number(0.0)));
    }

    #[test]
    fn eval_sumsq_propagates_reference_derived_errors() {
        let args = vec![CalcValue::reference(ReferenceLike::new(
            ReferenceKind::Area,
            "A1:A2".to_string(),
        ))];
        let got = eval_sumsq_surface(
            &args,
            &MockResolver {
                resolved_value: Some(CalcValue::array(
                    CalcArray::from_rows(vec![vec![
                        CalcValue::number(2.0),
                        CalcValue::error(WorksheetErrorCode::NA),
                    ]])
                    .unwrap(),
                )),
            },
        );
        assert_eq!(
            got,
            Err(SumsqEvalError::Coercion(CoercionError::WorksheetError(
                WorksheetErrorCode::NA
            )))
        );
    }
}

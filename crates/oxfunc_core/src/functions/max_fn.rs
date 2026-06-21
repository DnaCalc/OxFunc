use crate::coercion::CoercionError;
use crate::function::{
    Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile, FunctionMeta,
    HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{AggregatePreparedItem, expand_aggregate_arg};
use crate::functions::aggregate_common::sum_argument_value;
use crate::resolver::ReferenceSystemProvider;
use crate::value::CalcValue;
use crate::value::WorksheetErrorCode;

pub const MAX_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.MAX",
    arity: Arity { min: 1, max: 255 },
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: FunctionMeta::DEFAULT_ARG_PREPARATION_PROFILE,
    coercion_lift_profile: CoercionLiftProfile::AggregateDirectAndRangeDualPolicy,
    kernel_signature_class: KernelSignatureClass::NumsToNum,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
    real_result_policy: FunctionMeta::DEFAULT_REAL_RESULT_POLICY,
};

#[derive(Debug, Clone, PartialEq)]
pub enum MaxEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
}

fn eval_max_aggregate(args: &[AggregatePreparedItem]) -> Result<CalcValue, MaxEvalError> {
    let mut acc: Option<f64> = None;
    for arg in args {
        if let Some(value) = sum_argument_value(arg).map_err(MaxEvalError::Coercion)? {
            acc = Some(match acc {
                Some(current) => current.max(value),
                None => value,
            });
        }
    }
    Ok(CalcValue::number(acc.unwrap_or(0.0)))
}

pub fn eval_max_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, MaxEvalError> {
    let argc = args.len();
    if !MAX_META.arity.accepts(argc) {
        return Err(MaxEvalError::ArityMismatch {
            expected_min: MAX_META.arity.min,
            expected_max: MAX_META.arity.max,
            actual: argc,
        });
    }

    let mut prepared = Vec::new();
    for arg in args {
        prepared.extend(expand_aggregate_arg(arg, resolver).map_err(MaxEvalError::Coercion)?);
    }
    eval_max_aggregate(&prepared)
}

pub fn map_max_error_to_ws(e: &MaxEvalError) -> WorksheetErrorCode {
    match e {
        MaxEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        MaxEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        MaxEvalError::Coercion(_) => WorksheetErrorCode::Value,
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
    fn eval_max_accumulates_direct_numbers() {
        let args = vec![
            (CalcValue::number(2.0)),
            (CalcValue::number(3.0)),
            (CalcValue::number(4.0)),
        ];
        let got = eval_max_surface(
            &args,
            &MockResolver {
                resolved_value: None,
            },
        );
        assert_eq!(got, Ok(CalcValue::number(4.0)));
    }

    #[test]
    fn eval_max_counts_direct_numeric_text_and_logical() {
        let args = vec![
            (CalcValue::logical(true)),
            (CalcValue::text(ExcelText::from_utf16_code_units(
                "2".encode_utf16().collect(),
            ))),
        ];
        let got = eval_max_surface(
            &args,
            &MockResolver {
                resolved_value: None,
            },
        );
        assert_eq!(got, Ok(CalcValue::number(2.0)));
    }

    #[test]
    fn eval_max_ignores_reference_derived_text_and_logical() {
        let args = vec![CalcValue::reference(ReferenceLike::new(
            ReferenceKind::Area,
            "A1:A2".to_string(),
        ))];
        let got = eval_max_surface(
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
    fn eval_max_propagates_reference_derived_errors() {
        let args = vec![CalcValue::reference(ReferenceLike::new(
            ReferenceKind::Area,
            "A1:A3".to_string(),
        ))];
        let got = eval_max_surface(
            &args,
            &MockResolver {
                resolved_value: Some(CalcValue::array(
                    CalcArray::from_rows(vec![vec![
                        CalcValue::number(3.0),
                        CalcValue::error(WorksheetErrorCode::NA),
                    ]])
                    .unwrap(),
                )),
            },
        );
        assert_eq!(
            got,
            Err(MaxEvalError::Coercion(CoercionError::WorksheetError(
                WorksheetErrorCode::NA
            )))
        );
    }
}

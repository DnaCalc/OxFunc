use crate::coercion::CoercionError;
use crate::function::{
    Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile, FunctionMeta,
    HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{AggregatePreparedItem, expand_aggregate_arg};
use crate::functions::aggregate_common::average_argument_value;
use crate::resolver::ReferenceSystemProvider;
use crate::value::CalcValue;
use crate::value::WorksheetErrorCode;

pub const DEVSQ_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.DEVSQ",
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
    error_collapse_profile: FunctionMeta::DEFAULT_ERROR_COLLAPSE_PROFILE,
    precision_rounding_profile: FunctionMeta::DEFAULT_PRECISION_ROUNDING_PROFILE,
};

#[derive(Debug, Clone, PartialEq)]
pub enum DevSqEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
}

fn eval_devsq_aggregate(args: &[AggregatePreparedItem]) -> Result<CalcValue, DevSqEvalError> {
    let mut values = Vec::new();
    for arg in args {
        if let Some(value) = average_argument_value(arg).map_err(DevSqEvalError::Coercion)? {
            values.push(value);
        }
    }

    if values.is_empty() {
        return Ok(CalcValue::error(WorksheetErrorCode::Num));
    }

    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let devsq = values
        .iter()
        .map(|value| {
            let delta = value - mean;
            delta * delta
        })
        .sum::<f64>();
    Ok(CalcValue::number(devsq))
}

pub fn eval_devsq_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, DevSqEvalError> {
    let argc = args.len();
    if !DEVSQ_META.arity.accepts(argc) {
        return Err(DevSqEvalError::ArityMismatch {
            expected_min: DEVSQ_META.arity.min,
            expected_max: DEVSQ_META.arity.max,
            actual: argc,
        });
    }

    let mut prepared = Vec::new();
    for arg in args {
        prepared.extend(expand_aggregate_arg(arg, resolver).map_err(DevSqEvalError::Coercion)?);
    }
    eval_devsq_aggregate(&prepared)
}

pub fn map_devsq_error_to_ws(e: &DevSqEvalError) -> WorksheetErrorCode {
    match e {
        DevSqEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        DevSqEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        DevSqEvalError::Coercion(_) => WorksheetErrorCode::Value,
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
    fn eval_devsq_accumulates_direct_numeric_text_and_logical() {
        let args = vec![
            (CalcValue::logical(true)),
            (CalcValue::text(ExcelText::from_utf16_code_units(
                "2".encode_utf16().collect(),
            ))),
        ];
        let got = eval_devsq_surface(
            &args,
            &MockResolver {
                resolved_value: None,
            },
        );
        assert_eq!(got, Ok(CalcValue::number(0.5)));
    }

    #[test]
    fn eval_devsq_ignores_reference_derived_text_and_logical() {
        let args = vec![CalcValue::reference(ReferenceLike::new(
            ReferenceKind::Area,
            "A1:A2".to_string(),
        ))];
        let got = eval_devsq_surface(
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
        assert_eq!(got, Ok(CalcValue::error(WorksheetErrorCode::Num)));
    }
}

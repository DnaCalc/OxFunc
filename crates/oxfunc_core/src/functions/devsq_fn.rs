use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{AggregatePreparedValue, expand_aggregate_arg};
use crate::functions::aggregate_common::average_argument_value;
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

pub const DEVSQ_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.DEVSQ",
    arity: Arity { min: 1, max: 255 },
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
    coercion_lift_profile: CoercionLiftProfile::AggregateDirectAndRangeDualPolicy,
    kernel_signature_class: KernelSignatureClass::NumsToNum,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
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

fn eval_devsq_aggregate(args: &[AggregatePreparedValue]) -> Result<EvalValue, DevSqEvalError> {
    let mut values = Vec::new();
    for arg in args {
        if let Some(value) = average_argument_value(arg).map_err(DevSqEvalError::Coercion)? {
            values.push(value);
        }
    }

    if values.is_empty() {
        return Ok(EvalValue::Error(WorksheetErrorCode::Num));
    }

    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let devsq = values
        .iter()
        .map(|value| {
            let delta = value - mean;
            delta * delta
        })
        .sum::<f64>();
    Ok(EvalValue::Number(devsq))
}

pub fn eval_devsq_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DevSqEvalError> {
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
    use crate::resolver::{RefResolutionError, ResolverCapabilities};
    use crate::value::{ArrayCellValue, EvalArray, ExcelText, ReferenceKind, ReferenceLike};

    struct MockResolver {
        resolved_value: Option<EvalValue>,
    }

    impl ReferenceResolver for MockResolver {
        fn capabilities(&self) -> ResolverCapabilities {
            ResolverCapabilities::permissive_local()
        }

        fn resolve_reference(
            &self,
            reference: &ReferenceLike,
        ) -> Result<EvalValue, RefResolutionError> {
            self.resolved_value
                .clone()
                .ok_or(RefResolutionError::UnresolvedReference {
                    target: reference.target.clone(),
                })
        }
    }

    #[test]
    fn eval_devsq_accumulates_direct_numeric_text_and_logical() {
        let args = vec![
            CallArgValue::Eval(EvalValue::Logical(true)),
            CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
                "2".encode_utf16().collect(),
            ))),
        ];
        let got = eval_devsq_surface(
            &args,
            &MockResolver {
                resolved_value: None,
            },
        );
        assert_eq!(got, Ok(EvalValue::Number(0.5)));
    }

    #[test]
    fn eval_devsq_ignores_reference_derived_text_and_logical() {
        let args = vec![CallArgValue::Reference(ReferenceLike {
            kind: ReferenceKind::Area,
            target: "A1:A2".to_string(),
        })];
        let got = eval_devsq_surface(
            &args,
            &MockResolver {
                resolved_value: Some(EvalValue::Array(
                    EvalArray::from_rows(vec![vec![
                        ArrayCellValue::Text(ExcelText::from_utf16_code_units(
                            "x".encode_utf16().collect(),
                        )),
                        ArrayCellValue::Logical(true),
                    ]])
                    .unwrap(),
                )),
            },
        );
        assert_eq!(got, Ok(EvalValue::Error(WorksheetErrorCode::Num)));
    }
}

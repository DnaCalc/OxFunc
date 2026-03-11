use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::expand_aggregate_arg;
use crate::functions::aggregate_common::count_argument_included;
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

pub const COUNT_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.COUNT",
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
pub enum CountEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
}

pub fn eval_count_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, CountEvalError> {
    let argc = args.len();
    if !COUNT_META.arity.accepts(argc) {
        return Err(CountEvalError::ArityMismatch {
            expected_min: COUNT_META.arity.min,
            expected_max: COUNT_META.arity.max,
            actual: argc,
        });
    }

    let mut count = 0.0;
    for arg in args {
        for item in expand_aggregate_arg(arg, resolver).map_err(CountEvalError::Coercion)? {
            if count_argument_included(&item).map_err(CountEvalError::Coercion)? {
                count += 1.0;
            }
        }
    }

    Ok(EvalValue::Number(count))
}

pub fn map_count_error_to_ws(e: &CountEvalError) -> WorksheetErrorCode {
    match e {
        CountEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        CountEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        CountEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::{RefResolutionError, ResolverCapabilities};
    use crate::value::{ArrayCellValue, EvalArray, ExcelText, ReferenceKind, ReferenceLike};

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

    #[test]
    fn eval_count_counts_direct_numeric_text_and_logical() {
        let args = vec![
            CallArgValue::Eval(EvalValue::Number(1.0)),
            CallArgValue::Eval(EvalValue::Logical(true)),
            CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
                "2".encode_utf16().collect(),
            ))),
            CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
                "bad".encode_utf16().collect(),
            ))),
            CallArgValue::EmptyCell,
        ];
        let got = eval_count_surface(&args, &MockResolver { resolved: None });
        assert_eq!(got, Ok(EvalValue::Number(3.0)));
    }

    #[test]
    fn eval_count_ignores_reference_text_and_logical() {
        let got = eval_count_surface(
            &[CallArgValue::Reference(ReferenceLike {
                kind: ReferenceKind::Area,
                target: "A1:A3".to_string(),
            })],
            &MockResolver {
                resolved: Some(EvalValue::Array(
                    EvalArray::from_rows(vec![vec![
                        ArrayCellValue::Number(2.0),
                        ArrayCellValue::Text(ExcelText::from_utf16_code_units(
                            "3".encode_utf16().collect(),
                        )),
                        ArrayCellValue::Logical(true),
                    ]])
                    .unwrap(),
                )),
            },
        );
        assert_eq!(got, Ok(EvalValue::Number(1.0)));
    }

    #[test]
    fn eval_count_propagates_worksheet_errors() {
        let got = eval_count_surface(
            &[CallArgValue::Eval(EvalValue::Error(WorksheetErrorCode::Div0))],
            &MockResolver { resolved: None },
        );
        assert_eq!(
            got,
            Err(CountEvalError::Coercion(CoercionError::WorksheetError(
                WorksheetErrorCode::Div0
            )))
        );
    }
}

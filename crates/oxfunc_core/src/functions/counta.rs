use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::expand_aggregate_arg;
use crate::functions::aggregate_common::counta_argument_included;
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

pub const COUNTA_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.COUNTA",
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
pub enum CountaEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Preparation(CoercionError),
}

pub fn eval_counta_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, CountaEvalError> {
    let argc = args.len();
    if !COUNTA_META.arity.accepts(argc) {
        return Err(CountaEvalError::ArityMismatch {
            expected_min: COUNTA_META.arity.min,
            expected_max: COUNTA_META.arity.max,
            actual: argc,
        });
    }

    let mut count = 0.0;
    for arg in args {
        for item in expand_aggregate_arg(arg, resolver).map_err(CountaEvalError::Preparation)? {
            if counta_argument_included(&item).map_err(CountaEvalError::Preparation)? {
                count += 1.0;
            }
        }
    }

    Ok(EvalValue::Number(count))
}

pub fn map_counta_error_to_ws(e: &CountaEvalError) -> WorksheetErrorCode {
    match e {
        CountaEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        CountaEvalError::Preparation(CoercionError::WorksheetError(code)) => *code,
        CountaEvalError::Preparation(_) => WorksheetErrorCode::Value,
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
    fn eval_counta_counts_non_empty_values() {
        let args = vec![
            CallArgValue::Eval(EvalValue::Number(1.0)),
            CallArgValue::Eval(EvalValue::Text(
                ExcelText::from_utf16_code_units(Vec::new()),
            )),
            CallArgValue::MissingArg,
            CallArgValue::EmptyCell,
        ];
        let got = eval_counta_surface(&args, &MockResolver { resolved: None });
        assert_eq!(got, Ok(EvalValue::Number(2.0)));
    }

    #[test]
    fn eval_counta_counts_reference_derived_error_and_empty_string_but_not_empty_cells() {
        let args = vec![CallArgValue::Reference(ReferenceLike {
            kind: ReferenceKind::Area,
            target: "A1:A3".to_string(),
        })];
        let got = eval_counta_surface(
            &args,
            &MockResolver {
                resolved: Some(EvalValue::Array(
                    EvalArray::from_rows(vec![vec![
                        ArrayCellValue::Text(ExcelText::from_utf16_code_units(Vec::new())),
                        ArrayCellValue::Error(WorksheetErrorCode::NA),
                        ArrayCellValue::EmptyCell,
                    ]])
                    .unwrap(),
                )),
            },
        );
        assert_eq!(got, Ok(EvalValue::Number(2.0)));
    }

    #[test]
    fn eval_counta_direct_array_counts_empty_string_and_error() {
        let got = eval_counta_surface(
            &[CallArgValue::Eval(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Text(ExcelText::from_utf16_code_units(Vec::new())),
                    ArrayCellValue::Error(WorksheetErrorCode::NA),
                    ArrayCellValue::EmptyCell,
                ]])
                .unwrap(),
            ))],
            &MockResolver { resolved: None },
        );
        assert_eq!(got, Ok(EvalValue::Number(2.0)));
    }
}

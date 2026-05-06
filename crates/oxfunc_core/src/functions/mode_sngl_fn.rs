use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{AggregatePreparedValue, PreparedArgValue, expand_aggregate_arg};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};
use std::collections::BTreeMap;

pub const MODE_SNGL_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.MODE.SNGL",
    arity: Arity { min: 1, max: 255 },
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::NumsToNum,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
};

#[derive(Debug, Clone, PartialEq)]
pub enum ModeSnglEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
}

fn mode_argument_value(item: &AggregatePreparedValue) -> Result<Option<f64>, CoercionError> {
    match &item.value {
        PreparedArgValue::Eval(EvalValue::Number(n)) => Ok(Some(*n)),
        PreparedArgValue::Eval(EvalValue::Error(code)) => Err(CoercionError::WorksheetError(*code)),
        PreparedArgValue::Eval(EvalValue::Text(_))
        | PreparedArgValue::Eval(EvalValue::Logical(_))
        | PreparedArgValue::MissingArg
        | PreparedArgValue::EmptyCell => Ok(None),
        PreparedArgValue::Eval(EvalValue::Array(_)) => {
            Err(CoercionError::UnsupportedValueKind("array"))
        }
        PreparedArgValue::Eval(EvalValue::Reference(_)) => {
            Err(CoercionError::UnsupportedValueKind("reference_like"))
        }
        PreparedArgValue::Eval(EvalValue::Lambda(_)) => {
            Err(CoercionError::UnsupportedValueKind("lambda_value"))
        }
    }
}

pub fn eval_mode_sngl_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, ModeSnglEvalError> {
    let argc = args.len();
    if !MODE_SNGL_META.arity.accepts(argc) {
        return Err(ModeSnglEvalError::ArityMismatch {
            expected_min: MODE_SNGL_META.arity.min,
            expected_max: MODE_SNGL_META.arity.max,
            actual: argc,
        });
    }

    let mut counts: BTreeMap<u64, (f64, usize)> = BTreeMap::new();
    for arg in args {
        for item in expand_aggregate_arg(arg, resolver).map_err(ModeSnglEvalError::Coercion)? {
            if let Some(value) = mode_argument_value(&item).map_err(ModeSnglEvalError::Coercion)? {
                let key = value.to_bits();
                let entry = counts.entry(key).or_insert((value, 0));
                entry.1 += 1;
            }
        }
    }

    let mut best: Option<(f64, usize)> = None;
    for (_, (value, count)) in counts {
        if count < 2 {
            continue;
        }
        best = match best {
            None => Some((value, count)),
            Some((_best_value, best_count)) if count > best_count => Some((value, count)),
            Some((best_value, best_count)) if count == best_count && value < best_value => {
                Some((value, count))
            }
            other => other,
        };
    }

    match best {
        Some((value, _)) => Ok(EvalValue::Number(value)),
        None => Ok(EvalValue::Error(WorksheetErrorCode::NA)),
    }
}

pub fn map_mode_sngl_error_to_ws(e: &ModeSnglEvalError) -> WorksheetErrorCode {
    match e {
        ModeSnglEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        ModeSnglEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        ModeSnglEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::{RefResolutionError, ReferenceResolver, ResolverCapabilities};
    use crate::value::{ArrayCellValue, CallArgValue, EvalArray, ReferenceKind, ReferenceLike};

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
    fn eval_mode_sngl_basic_and_tie_lanes() {
        let args = vec![
            CallArgValue::Eval(EvalValue::Number(2.0)),
            CallArgValue::Eval(EvalValue::Number(2.0)),
            CallArgValue::Eval(EvalValue::Number(3.0)),
            CallArgValue::Eval(EvalValue::Number(3.0)),
            CallArgValue::Eval(EvalValue::Number(4.0)),
        ];
        let got = eval_mode_sngl_surface(
            &args,
            &MockResolver {
                resolved_value: None,
            },
        );
        assert_eq!(got, Ok(EvalValue::Number(2.0)));
    }

    #[test]
    fn eval_mode_sngl_returns_na_when_no_mode_survives() {
        let args = vec![
            CallArgValue::Eval(EvalValue::Logical(true)),
            CallArgValue::Eval(EvalValue::Text(
                crate::value::ExcelText::from_utf16_code_units("2".encode_utf16().collect()),
            )),
        ];
        let got = eval_mode_sngl_surface(
            &args,
            &MockResolver {
                resolved_value: None,
            },
        );
        assert_eq!(got, Ok(EvalValue::Error(WorksheetErrorCode::NA)));
    }

    #[test]
    fn eval_mode_sngl_propagates_reference_error_lane() {
        let args = vec![CallArgValue::Reference(ReferenceLike {
            kind: ReferenceKind::Area,
            target: "A1:A3".to_string(),
        })];
        let array = EvalArray::from_rows(vec![vec![
            ArrayCellValue::Text(crate::value::ExcelText::from_utf16_code_units(
                "x".encode_utf16().collect(),
            )),
            ArrayCellValue::Logical(true),
            ArrayCellValue::Error(WorksheetErrorCode::NA),
        ]])
        .unwrap();
        let got = eval_mode_sngl_surface(
            &args,
            &MockResolver {
                resolved_value: Some(EvalValue::Array(array)),
            },
        );
        assert_eq!(
            got,
            Err(ModeSnglEvalError::Coercion(CoercionError::WorksheetError(
                WorksheetErrorCode::NA,
            )))
        );
    }
}

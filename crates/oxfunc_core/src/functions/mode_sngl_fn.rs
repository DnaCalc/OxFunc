use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{AggregatePreparedItem, expand_aggregate_arg};
use crate::resolver::ReferenceSystemProvider;
use crate::value::CalcValue;
use crate::value::{CoreValue, WorksheetErrorCode};
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
    real_result_policy: FunctionMeta::DEFAULT_REAL_RESULT_POLICY,
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

fn mode_argument_value(item: &AggregatePreparedItem) -> Result<Option<f64>, CoercionError> {
    match item.0.core() {
        CoreValue::Number(n) => Ok(Some(*n)),
        CoreValue::Error(code) => Err(CoercionError::WorksheetError(*code)),
        CoreValue::Text(_) | CoreValue::Logical(_) | CoreValue::Missing | CoreValue::Empty => {
            Ok(None)
        }
        CoreValue::Array(_) => Err(CoercionError::UnsupportedValueKind("array")),
        CoreValue::Reference(_) => Err(CoercionError::UnsupportedValueKind("reference_like")),
    }
}

pub fn eval_mode_sngl_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, ModeSnglEvalError> {
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
        Some((value, _)) => Ok(CalcValue::number(value)),
        None => Ok(CalcValue::error(WorksheetErrorCode::NA)),
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
    use crate::resolver::{ReferenceSystemCapabilities, ReferenceSystemProvider};
    use crate::value::{CalcArray, ReferenceKind, ReferenceLike};

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
    fn eval_mode_sngl_basic_and_tie_lanes() {
        let args = vec![
            (CalcValue::number(2.0)),
            (CalcValue::number(2.0)),
            (CalcValue::number(3.0)),
            (CalcValue::number(3.0)),
            (CalcValue::number(4.0)),
        ];
        let got = eval_mode_sngl_surface(
            &args,
            &MockResolver {
                resolved_value: None,
            },
        );
        assert_eq!(got, Ok(CalcValue::number(2.0)));
    }

    #[test]
    fn eval_mode_sngl_returns_na_when_no_mode_survives() {
        let args = vec![
            (CalcValue::logical(true)),
            (CalcValue::text(crate::value::ExcelText::from_utf16_code_units(
                "2".encode_utf16().collect(),
            ))),
        ];
        let got = eval_mode_sngl_surface(
            &args,
            &MockResolver {
                resolved_value: None,
            },
        );
        assert_eq!(got, Ok(CalcValue::error(WorksheetErrorCode::NA)));
    }

    #[test]
    fn eval_mode_sngl_propagates_reference_error_lane() {
        let args = vec![CalcValue::reference(ReferenceLike::new(
            ReferenceKind::Area,
            "A1:A3".to_string(),
        ))];
        let array = CalcArray::from_rows(vec![vec![
            CalcValue::text(crate::value::ExcelText::from_utf16_code_units(
                "x".encode_utf16().collect(),
            )),
            CalcValue::logical(true),
            CalcValue::error(WorksheetErrorCode::NA),
        ]])
        .unwrap();
        let got = eval_mode_sngl_surface(
            &args,
            &MockResolver {
                resolved_value: Some(CalcValue::array(array)),
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

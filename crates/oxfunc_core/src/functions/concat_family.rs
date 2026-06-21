use crate::coercion::CoercionError;
use crate::function::{
    Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile, FunctionMeta,
    HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    coerce_prepared_to_text, expand_arg_values_only, prepare_arg_values_only,
};
use crate::resolver::ReferenceSystemProvider;
use crate::value::CalcValue;
use crate::value::{EXCEL_TEXT_MAX_UTF16_CODE_UNITS, ExcelText, WorksheetErrorCode};

pub const CONCAT_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.CONCAT",
    arity: Arity { min: 1, max: 253 },
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: FunctionMeta::DEFAULT_ARG_PREPARATION_PROFILE,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    lift_broadcast_profile: FunctionMeta::DEFAULT_LIFT_BROADCAST_PROFILE,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
    real_result_policy: FunctionMeta::DEFAULT_REAL_RESULT_POLICY,
    error_collapse_profile: FunctionMeta::DEFAULT_ERROR_COLLAPSE_PROFILE,
};

// Legacy CONCATENATE is scalar-shaped by-index and broadcasts its first three arguments over an
// array (`[0,1,2]`); modern CONCAT lifts natively (default). Verified live Excel 16.0 build 20026.
pub const CONCATENATE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.CONCATENATE",
    arity: Arity { min: 1, max: 255 },
    lift_broadcast_profile: FunctionMeta::lift_at(&[0, 1, 2]),
    ..CONCAT_META
};

#[derive(Debug, Clone, PartialEq)]
pub enum ConcatEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
    ResultTooLong {
        actual_utf16_len: usize,
    },
}

fn push_text(out: &mut Vec<u16>, text: &ExcelText) -> Result<(), ConcatEvalError> {
    let next_total = out.len().saturating_add(text.utf16_code_units().len());
    if next_total > EXCEL_TEXT_MAX_UTF16_CODE_UNITS {
        return Err(ConcatEvalError::ResultTooLong {
            actual_utf16_len: next_total,
        });
    }
    out.extend_from_slice(text.utf16_code_units());
    Ok(())
}

fn finalize_concat_text(out: Vec<u16>) -> CalcValue {
    CalcValue::text(ExcelText::from_utf16_code_units(out))
}

pub fn eval_concat_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, ConcatEvalError> {
    if !CONCAT_META.arity.accepts(args.len()) {
        return Err(ConcatEvalError::ArityMismatch {
            expected_min: CONCAT_META.arity.min,
            expected_max: CONCAT_META.arity.max,
            actual: args.len(),
        });
    }

    let mut out = Vec::new();
    for arg in args {
        for prepared in expand_arg_values_only(arg, resolver).map_err(ConcatEvalError::Coercion)? {
            let text = coerce_prepared_to_text(&prepared).map_err(ConcatEvalError::Coercion)?;
            push_text(&mut out, &text)?;
        }
    }
    Ok(finalize_concat_text(out))
}

pub fn eval_concatenate_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, ConcatEvalError> {
    if !CONCATENATE_META.arity.accepts(args.len()) {
        return Err(ConcatEvalError::ArityMismatch {
            expected_min: CONCATENATE_META.arity.min,
            expected_max: CONCATENATE_META.arity.max,
            actual: args.len(),
        });
    }

    let mut out = Vec::new();
    for arg in args {
        let prepared = prepare_arg_values_only(arg, resolver).map_err(ConcatEvalError::Coercion)?;
        let text = coerce_prepared_to_text(&prepared).map_err(ConcatEvalError::Coercion)?;
        push_text(&mut out, &text)?;
    }
    Ok(finalize_concat_text(out))
}

pub fn map_concat_error_to_ws(e: &ConcatEvalError) -> WorksheetErrorCode {
    match e {
        ConcatEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        ConcatEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        ConcatEvalError::Coercion(_) => WorksheetErrorCode::Value,
        ConcatEvalError::ResultTooLong { .. } => WorksheetErrorCode::Calc,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::ReferenceSystemCapabilities;
    use crate::value::{CalcArray, ReferenceKind, ReferenceLike};

    struct MockResolver {
        resolved: Option<CalcValue>,
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
            self.resolved.clone().ok_or(
                crate::resolver::ReferenceResolutionError::UnresolvedReference {
                    target: reference.target().to_string(),
                },
            )
        }
    }

    fn txt(s: &str) -> CalcValue {
        CalcValue::text(ExcelText::from_utf16_code_units(s.encode_utf16().collect()))
    }

    #[test]
    fn concat_matches_probe_rows() {
        assert_eq!(
            eval_concat_surface(
                &[
                    (txt("a")),
                    (CalcValue::number(1.0)),
                    (CalcValue::logical(true)),
                ],
                &MockResolver { resolved: None },
            ),
            Ok(txt("a1TRUE"))
        );
    }

    #[test]
    fn concat_flattens_ranges_but_concatenate_rejects_multi_cell_ranges() {
        let range_arg =
            CalcValue::reference(ReferenceLike::new(ReferenceKind::Area, "D1:D3".to_string()));
        let resolved = Some(CalcValue::array(
            CalcArray::from_rows(vec![vec![
                CalcValue::empty(),
                CalcValue::text(ExcelText::from_utf16_code_units(Vec::new())),
                CalcValue::text(ExcelText::from_utf16_code_units(
                    "x".encode_utf16().collect(),
                )),
            ]])
            .unwrap(),
        ));
        assert_eq!(
            eval_concat_surface(
                std::slice::from_ref(&range_arg),
                &MockResolver {
                    resolved: resolved.clone(),
                }
            ),
            Ok(txt("x"))
        );
        assert!(matches!(
            eval_concatenate_surface(std::slice::from_ref(&range_arg), &MockResolver { resolved }),
            Err(ConcatEvalError::Coercion(
                CoercionError::UnsupportedValueKind("array")
            ))
        ));
    }

    #[test]
    fn concatenate_accepts_single_cell_refs() {
        assert_eq!(
            eval_concatenate_surface(
                &[
                    CalcValue::reference(ReferenceLike::new(ReferenceKind::A1, "D1".to_string())),
                    CalcValue::reference(ReferenceLike::new(ReferenceKind::A1, "D2".to_string())),
                    CalcValue::reference(ReferenceLike::new(ReferenceKind::A1, "D3".to_string())),
                ],
                &MockResolver {
                    resolved: Some(txt("x")),
                },
            ),
            Ok(txt("xxx"))
        );
    }
}

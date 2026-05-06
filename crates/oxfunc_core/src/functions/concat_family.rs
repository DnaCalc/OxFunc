use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    coerce_prepared_to_text, expand_arg_values_only, prepare_arg_values_only,
};
use crate::resolver::ReferenceResolver;
use crate::value::{
    CallArgValue, EXCEL_TEXT_MAX_UTF16_CODE_UNITS, EvalValue, ExcelText, WorksheetErrorCode,
};

pub const CONCAT_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.CONCAT",
    arity: Arity { min: 1, max: 253 },
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

pub const CONCATENATE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.CONCATENATE",
    arity: Arity { min: 1, max: 255 },
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

fn finalize_concat_text(out: Vec<u16>) -> EvalValue {
    EvalValue::Text(ExcelText::from_utf16_code_units(out))
}

pub fn eval_concat_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, ConcatEvalError> {
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
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, ConcatEvalError> {
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
    use crate::resolver::{RefResolutionError, ResolverCapabilities};
    use crate::value::{ArrayCellValue, EvalArray, ReferenceKind, ReferenceLike};

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

    fn txt(s: &str) -> EvalValue {
        EvalValue::Text(ExcelText::from_utf16_code_units(s.encode_utf16().collect()))
    }

    #[test]
    fn concat_matches_probe_rows() {
        assert_eq!(
            eval_concat_surface(
                &[
                    CallArgValue::Eval(txt("a")),
                    CallArgValue::Eval(EvalValue::Number(1.0)),
                    CallArgValue::Eval(EvalValue::Logical(true)),
                ],
                &MockResolver { resolved: None },
            ),
            Ok(txt("a1TRUE"))
        );
    }

    #[test]
    fn concat_flattens_ranges_but_concatenate_rejects_multi_cell_ranges() {
        let range_arg = CallArgValue::Reference(ReferenceLike {
            kind: ReferenceKind::Area,
            target: "D1:D3".to_string(),
        });
        let resolved = Some(EvalValue::Array(
            EvalArray::from_rows(vec![vec![
                ArrayCellValue::EmptyCell,
                ArrayCellValue::Text(ExcelText::from_utf16_code_units(Vec::new())),
                ArrayCellValue::Text(ExcelText::from_utf16_code_units(
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
                    CallArgValue::Reference(ReferenceLike {
                        kind: ReferenceKind::A1,
                        target: "D1".to_string(),
                    }),
                    CallArgValue::Reference(ReferenceLike {
                        kind: ReferenceKind::A1,
                        target: "D2".to_string(),
                    }),
                    CallArgValue::Reference(ReferenceLike {
                        kind: ReferenceKind::A1,
                        target: "D3".to_string(),
                    }),
                ],
                &MockResolver {
                    resolved: Some(txt("x")),
                },
            ),
            Ok(txt("xxx"))
        );
    }
}

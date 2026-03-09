use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    PreparedArgValue, coerce_prepared_to_number, run_values_only_prepared,
};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, ReferenceKind, ReferenceLike, WorksheetErrorCode};

pub const INDIRECT_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.INDIRECT",
    arity: Arity { min: 1, max: 2 },
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::VolatileContextual,
    host_interaction: HostInteractionClass::WorkbookState,
    thread_safety: ThreadSafetyClass::HostSerialized,
    arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::CallerContext,
    surface_fec_dependency_profile: FecDependencyProfile::CallerContext,
};

#[derive(Debug, Clone, PartialEq)]
pub enum IndirectEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
    UnsupportedR1C1Seed,
    InvalidReferenceText(String),
}

fn parse_a1_flag(arg: Option<&PreparedArgValue>) -> Result<bool, IndirectEvalError> {
    match arg {
        None => Ok(true),
        Some(p) => {
            let n = coerce_prepared_to_number(p).map_err(IndirectEvalError::Coercion)?;
            Ok(n != 0.0)
        }
    }
}

fn parse_ref_text(arg: &PreparedArgValue) -> Result<String, IndirectEvalError> {
    match arg {
        PreparedArgValue::Eval(EvalValue::Text(t)) => {
            let s = t.to_string_lossy().trim().to_string();
            if s.is_empty() {
                return Err(IndirectEvalError::InvalidReferenceText(String::new()));
            }
            Ok(s)
        }
        PreparedArgValue::Eval(EvalValue::Error(code)) => Err(IndirectEvalError::Coercion(
            CoercionError::WorksheetError(*code),
        )),
        PreparedArgValue::MissingArg => Err(IndirectEvalError::Coercion(CoercionError::MissingArg)),
        PreparedArgValue::EmptyCell => Err(IndirectEvalError::Coercion(CoercionError::EmptyCell)),
        PreparedArgValue::Eval(other) => {
            let kind = match other {
                EvalValue::Number(_) => "number",
                EvalValue::Logical(_) => "logical",
                EvalValue::Array(_) => "array",
                EvalValue::Reference(_) => "reference_like",
                EvalValue::Lambda(_) => "lambda_value",
                EvalValue::Text(_) | EvalValue::Error(_) => unreachable!(),
            };
            Err(IndirectEvalError::InvalidReferenceText(kind.to_string()))
        }
    }
}

pub fn eval_indirect_adapter_prepared(
    args: &[PreparedArgValue],
) -> Result<EvalValue, IndirectEvalError> {
    let argc = args.len();
    if !INDIRECT_META.arity.accepts(argc) {
        return Err(IndirectEvalError::ArityMismatch {
            expected_min: INDIRECT_META.arity.min,
            expected_max: INDIRECT_META.arity.max,
            actual: argc,
        });
    }

    let text = parse_ref_text(&args[0])?;
    let a1_style = parse_a1_flag(args.get(1))?;

    if !a1_style {
        return Err(IndirectEvalError::UnsupportedR1C1Seed);
    }

    Ok(EvalValue::Reference(ReferenceLike {
        kind: ReferenceKind::A1,
        target: text,
    }))
}

pub fn eval_indirect_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, IndirectEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_indirect_adapter_prepared,
        IndirectEvalError::Coercion,
    )
}

pub fn map_indirect_error_to_ws(e: &IndirectEvalError) -> WorksheetErrorCode {
    match e {
        IndirectEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        IndirectEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        IndirectEvalError::UnsupportedR1C1Seed => WorksheetErrorCode::Ref,
        IndirectEvalError::InvalidReferenceText(_) => WorksheetErrorCode::Ref,
        IndirectEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::{RefResolutionError, ResolverCapabilities};
    use crate::value::ExcelText;

    struct NoResolver;
    impl ReferenceResolver for NoResolver {
        fn capabilities(&self) -> ResolverCapabilities {
            ResolverCapabilities::permissive_local()
        }

        fn resolve_reference(
            &self,
            reference: &ReferenceLike,
        ) -> Result<EvalValue, RefResolutionError> {
            Err(RefResolutionError::UnresolvedReference {
                target: reference.target.clone(),
            })
        }
    }

    fn text_arg(s: &str) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
            s.encode_utf16().collect(),
        )))
    }

    #[test]
    fn eval_indirect_a1_text_returns_reference_like() {
        let got = eval_indirect_surface(&[text_arg("Sheet1!A1")], &NoResolver);
        assert_eq!(
            got,
            Ok(EvalValue::Reference(ReferenceLike {
                kind: ReferenceKind::A1,
                target: "Sheet1!A1".to_string(),
            }))
        );
    }

    #[test]
    fn eval_indirect_r1c1_seed_is_explicitly_unsupported() {
        let got = eval_indirect_surface(
            &[text_arg("R1C1"), CallArgValue::Eval(EvalValue::Number(0.0))],
            &NoResolver,
        );
        assert_eq!(got, Err(IndirectEvalError::UnsupportedR1C1Seed));
    }

    #[test]
    fn eval_indirect_rejects_non_text_reference_expression() {
        let got = eval_indirect_surface(&[CallArgValue::Eval(EvalValue::Number(1.0))], &NoResolver);
        assert_eq!(
            got,
            Err(IndirectEvalError::InvalidReferenceText(
                "number".to_string()
            ))
        );
    }
}

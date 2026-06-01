use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::function_call::FunctionExecutionContext;
use crate::functions::adapters::{
    PreparedArgValue, coerce_prepared_to_number, run_values_only_prepared,
};
use crate::resolver::{
    CallerContext, ReferenceTextResolutionError, ReferenceTextResolutionMode,
    ReferenceTextResolutionRequest, ReferenceTextResolver,
};
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

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
    InvalidReferenceText(String),
    ReferenceTextResolution(ReferenceTextResolutionError),
}

fn parse_a1_flag(arg: Option<&PreparedArgValue>) -> Result<bool, IndirectEvalError> {
    match arg {
        None => Ok(true),
        Some(PreparedArgValue::MissingArg | PreparedArgValue::EmptyCell) => Ok(false),
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

pub fn eval_indirect_surface(
    args: &[CallArgValue],
    fec: &dyn FunctionExecutionContext,
) -> Result<EvalValue, IndirectEvalError> {
    let caller_context = fec.caller_context();
    run_values_only_prepared(
        args,
        fec.reference_resolver(),
        |prepared| {
            let Some(reference_text_resolver) = fec.reference_text_resolver() else {
                return Err(IndirectEvalError::ReferenceTextResolution(
                    ReferenceTextResolutionError::Unsupported,
                ));
            };
            eval_indirect_with_reference_text_resolver(
                prepared,
                caller_context.clone(),
                reference_text_resolver,
            )
        },
        IndirectEvalError::Coercion,
    )
}

fn eval_indirect_with_reference_text_resolver(
    args: &[PreparedArgValue],
    caller_context: Option<CallerContext>,
    reference_text_resolver: &dyn ReferenceTextResolver,
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
    let reference = reference_text_resolver
        .resolve_reference_text(&ReferenceTextResolutionRequest {
            text,
            mode: ReferenceTextResolutionMode::Indirect,
            a1_style: Some(a1_style),
            caller_context,
        })
        .map_err(IndirectEvalError::ReferenceTextResolution)?;
    Ok(EvalValue::Reference(reference))
}

pub fn map_indirect_error_to_ws(e: &IndirectEvalError) -> WorksheetErrorCode {
    match e {
        IndirectEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        IndirectEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        IndirectEvalError::InvalidReferenceText(_) => WorksheetErrorCode::Ref,
        IndirectEvalError::ReferenceTextResolution(_) => WorksheetErrorCode::Ref,
        IndirectEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::{RefResolutionError, ReferenceResolver, ResolverCapabilities};
    use crate::value::{ExcelText, ReferenceKind, ReferenceLike};

    struct MockResolver {
        caller: Option<CallerContext>,
    }

    struct MockReferenceTextResolver;

    impl ReferenceResolver for MockResolver {
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

        fn caller_context(&self) -> Option<CallerContext> {
            self.caller.clone()
        }
    }

    fn text_arg(s: &str) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
            s.encode_utf16().collect(),
        )))
    }

    fn eval_with_mock(
        args: &[CallArgValue],
        caller: Option<CallerContext>,
        reference_text_resolver: Option<&dyn ReferenceTextResolver>,
    ) -> Result<EvalValue, IndirectEvalError> {
        let resolver = MockResolver { caller };
        let fec = crate::function_call::FunctionExecutionContextRef::new(&resolver)
            .with_reference_text_resolver(reference_text_resolver);
        eval_indirect_surface(args, &fec)
    }

    #[test]
    fn eval_indirect_requires_reference_text_resolver() {
        let got = eval_with_mock(&[text_arg("Sheet1!A1")], None, None);
        assert_eq!(
            got,
            Err(IndirectEvalError::ReferenceTextResolution(
                ReferenceTextResolutionError::Unsupported
            ))
        );
    }

    #[test]
    fn eval_indirect_rejects_non_text_reference_expression() {
        let got = eval_with_mock(
            &[CallArgValue::Eval(EvalValue::Number(1.0))],
            None,
            Some(&MockReferenceTextResolver),
        );
        assert_eq!(
            got,
            Err(IndirectEvalError::InvalidReferenceText(
                "number".to_string()
            ))
        );
    }

    #[test]
    fn eval_indirect_missing_a1_flag_passes_false_to_reference_text_resolver() {
        struct MissingFlagResolver;

        impl ReferenceTextResolver for MissingFlagResolver {
            fn resolve_reference_text(
                &self,
                request: &ReferenceTextResolutionRequest,
            ) -> Result<ReferenceLike, ReferenceTextResolutionError> {
                assert_eq!(request.text, "R1C2");
                assert_eq!(request.mode, ReferenceTextResolutionMode::Indirect);
                assert_eq!(request.a1_style, Some(false));
                Ok(ReferenceLike::new(
                    ReferenceKind::A1,
                    "hostref:R1C2".to_string(),
                ))
            }
        }

        let got = eval_with_mock(
            &[text_arg("R1C2"), CallArgValue::MissingArg],
            Some(CallerContext {
                prefix: Some("Sheet1".to_string()),
                row: 3,
                col: 3,
            }),
            Some(&MissingFlagResolver),
        );
        assert_eq!(
            got,
            Ok(EvalValue::Reference(ReferenceLike::new(
                ReferenceKind::A1,
                "hostref:R1C2".to_string()
            )))
        );
    }

    impl ReferenceTextResolver for MockReferenceTextResolver {
        fn resolve_reference_text(
            &self,
            request: &ReferenceTextResolutionRequest,
        ) -> Result<ReferenceLike, ReferenceTextResolutionError> {
            assert_eq!(request.text, "Tree.Node");
            assert_eq!(request.mode, ReferenceTextResolutionMode::Indirect);
            assert_eq!(request.a1_style, Some(true));
            assert_eq!(
                request.caller_context,
                Some(CallerContext {
                    prefix: Some("Tree".to_string()),
                    row: 7,
                    col: 8,
                })
            );
            Ok(ReferenceLike::new(
                ReferenceKind::Structured,
                "hostref:Tree.Node".to_string(),
            ))
        }
    }

    #[test]
    fn eval_indirect_uses_reference_text_resolver_when_supplied() {
        let got = eval_with_mock(
            &[text_arg("Tree.Node")],
            Some(CallerContext {
                prefix: Some("Tree".to_string()),
                row: 7,
                col: 8,
            }),
            Some(&MockReferenceTextResolver),
        );
        assert_eq!(
            got,
            Ok(EvalValue::Reference(ReferenceLike::new(
                ReferenceKind::Structured,
                "hostref:Tree.Node".to_string()
            )))
        );
    }
}
